#!/usr/bin/env python3
"""Validate the repository's machine-checkable documentation contract."""

from __future__ import annotations

import argparse
import difflib
import os
import re
import sys
import tomllib
from dataclasses import dataclass
from pathlib import Path
from urllib.parse import unquote, urlsplit


QUICKSTART_BEGIN = "<!-- BEGIN QUICKSTART -->"
QUICKSTART_END = "<!-- END QUICKSTART -->"
FEATURE_HEADING = re.compile(r"^## Cargo features\s*$", re.MULTILINE)
NEXT_H2 = re.compile(r"^## ", re.MULTILINE)
INLINE_LINK = re.compile(r"!?\[[^\]\n]*\]\(\s*(?P<target><[^>\n]+>|[^\s)]+)")
REFERENCE_LINK = re.compile(
    r"^\s{0,3}\[[^\]\n]+\]:\s*(?P<target><[^>\n]+>|\S+)", re.MULTILINE
)
VERSION = re.compile(r"(?<![\w.])(\d+\.\d+(?:\.\d+)?)(?![\w.])")


@dataclass(frozen=True)
class Manifest:
    rust_version: str
    features: frozenset[str]


@dataclass(frozen=True)
class Problem:
    path: Path
    line: int
    message: str


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "--root",
        type=Path,
        default=Path(__file__).resolve().parents[1],
        help="repository root",
    )
    parser.add_argument("--print-msrv", action="store_true")
    return parser.parse_args()


def read_manifest(path: Path) -> Manifest:
    with path.open("rb") as cargo_file:
        cargo = tomllib.load(cargo_file)

    package = cargo["package"]
    features = cargo.get("features", {})
    return Manifest(
        rust_version=str(package["rust-version"]),
        features=frozenset(features),
    )


def line_number(text: str, offset: int) -> int:
    return text.count("\n", 0, offset) + 1


def add_problem(
    problems: list[Problem], path: Path, text: str, offset: int, message: str
) -> None:
    problems.append(Problem(path, line_number(text, offset), message))


def check_msrv(
    path: Path, readme: str, manifest: Manifest, problems: list[Problem]
) -> None:
    matching_lines = [
        line
        for line in readme.splitlines()
        if "MSRV" in line or "minimum supported Rust version" in line
    ]
    versions = {version for line in matching_lines for version in VERSION.findall(line)}
    if versions != {manifest.rust_version}:
        problems.append(
            Problem(
                path,
                1,
                "README MSRV declarations must exactly match Cargo.toml "
                f"({manifest.rust_version}); found {sorted(versions)}",
            )
        )


def feature_names(section: str) -> list[str]:
    names: list[str] = []
    for line in section.splitlines():
        if not line.lstrip().startswith("|"):
            continue
        first_cell = line.strip().strip("|").split("|", maxsplit=1)[0]
        match = re.search(r"`([A-Za-z0-9_-]+)`", first_cell)
        if match:
            names.append(match.group(1))
    return names


def check_features(
    path: Path, readme: str, manifest: Manifest, problems: list[Problem]
) -> None:
    heading = FEATURE_HEADING.search(readme)
    if heading is None:
        problems.append(Problem(path, 1, "README must contain '## Cargo features'"))
        return

    next_heading = NEXT_H2.search(readme, heading.end())
    end = next_heading.start() if next_heading else len(readme)
    documented = feature_names(readme[heading.end() : end])
    documented_set = set(documented)
    if documented_set != manifest.features:
        missing = sorted(manifest.features - documented_set)
        unknown = sorted(documented_set - manifest.features)
        add_problem(
            problems,
            path,
            readme,
            heading.start(),
            f"Cargo feature table mismatch; missing={missing}, unknown={unknown}",
        )
    if len(documented) != len(documented_set):
        add_problem(
            problems, path, readme, heading.start(), "duplicate Cargo feature entry"
        )


def check_install_guidance(path: Path, readme: str, problems: list[Problem]) -> None:
    if "cargo add threatflux-unifi-sdk" not in readme:
        problems.append(
            Problem(path, 1, "README must use version-independent cargo add guidance")
        )

    dependency = re.search(r'(?m)^\s*threatflux-unifi-sdk\s*=\s*"[^"]+"\s*$', readme)
    if dependency is not None:
        add_problem(
            problems,
            path,
            readme,
            dependency.start(),
            "README crates.io install guidance must not hard-code a version",
        )


def extract_quickstart(path: Path, readme: str, problems: list[Problem]) -> str | None:
    if readme.count(QUICKSTART_BEGIN) != 1 or readme.count(QUICKSTART_END) != 1:
        problems.append(
            Problem(path, 1, "README must contain one quickstart marker pair")
        )
        return None

    begin = readme.index(QUICKSTART_BEGIN) + len(QUICKSTART_BEGIN)
    end = readme.index(QUICKSTART_END)
    region = readme[begin:end].strip("\n")
    match = re.fullmatch(r"```rust\n(?P<code>.*)\n```", region, re.DOTALL)
    if match is None:
        add_problem(
            problems, path, readme, begin, "quickstart markers must wrap one Rust block"
        )
        return None
    return match.group("code")


def check_quickstart(
    readme_path: Path,
    readme: str,
    example_path: Path,
    problems: list[Problem],
) -> None:
    actual = extract_quickstart(readme_path, readme, problems)
    if actual is None:
        return
    expected = example_path.read_text(encoding="utf-8").rstrip("\n")
    if actual == expected:
        return

    diff = "\n".join(
        difflib.unified_diff(
            expected.splitlines(),
            actual.splitlines(),
            fromfile="examples/quickstart.rs",
            tofile="README quickstart",
            lineterm="",
        )
    )
    problems.append(Problem(readme_path, 1, f"quickstart is out of sync\n{diff}"))


def markdown_files(root: Path) -> list[Path]:
    return sorted(
        path
        for path in root.rglob("*.md")
        if ".git" not in path.parts and "target" not in path.parts
    )


def link_targets(markdown: str) -> list[tuple[int, str]]:
    matches = list(INLINE_LINK.finditer(markdown))
    matches.extend(REFERENCE_LINK.finditer(markdown))
    matches.sort(key=lambda match: match.start())
    return [
        (
            line_number(markdown, match.start("target")),
            match.group("target").strip("<>"),
        )
        for match in matches
    ]


def check_local_links(root: Path, problems: list[Problem]) -> None:
    for path in markdown_files(root):
        markdown = path.read_text(encoding="utf-8")
        for line, target in link_targets(markdown):
            parsed = urlsplit(target)
            if (
                parsed.scheme
                or parsed.netloc
                or not parsed.path
                or parsed.path.startswith("/")
            ):
                continue
            resolved = (path.parent / unquote(parsed.path)).resolve()
            if not resolved.is_relative_to(root):
                problems.append(
                    Problem(path, line, f"local link escapes repository: {target}")
                )
                continue
            if not resolved.exists():
                problems.append(Problem(path, line, f"broken local link: {target}"))


def check_readme_contract(path: Path, readme: str, problems: list[Problem]) -> None:
    normalized = " ".join(readme.split())
    required = (
        "not an official",
        "docs/api-coverage.md",
        "docs/configuration.md",
        "docs/cli.md",
        "TLS verification is disabled by default",
        "`UnifiConfig` does not read environment variables",
        "failed request is not replayed",
        "features do not currently gate",
        "`unifi-cli`",
    )
    for phrase in required:
        if phrase not in normalized:
            problems.append(Problem(path, 1, f"README must contain: {phrase}"))

    banned = (
        "binary named `threatflux`",
        "automatic re-login",
        "Full support",
        "Firewall management only",
        "VPN configuration only",
        "UNIFI_VERIFY_SSL=false",
        "UNIFI_TIMEOUT_SECS",
    )
    for phrase in banned:
        offset = readme.find(phrase)
        if offset >= 0:
            add_problem(
                problems, path, readme, offset, f"obsolete README text: {phrase}"
            )


def check_tls_examples(
    quickstart_path: Path,
    example_config_path: Path,
    problems: list[Problem],
) -> None:
    quickstart = quickstart_path.read_text(encoding="utf-8")
    if ".with_verify_ssl(true)" not in quickstart:
        problems.append(
            Problem(quickstart_path, 1, "quickstart must enable TLS verification")
        )
    if ".with_verify_ssl(false)" in quickstart:
        problems.append(
            Problem(quickstart_path, 1, "quickstart must not disable TLS verification")
        )

    example_config = example_config_path.read_text(encoding="utf-8")
    if re.search(r"(?m)^\s*verify_ssl:\s*true\s*$", example_config) is None:
        problems.append(
            Problem(
                example_config_path,
                1,
                "example configuration must set verify_ssl: true",
            )
        )
    false_match = re.search(r"(?m)^\s*verify_ssl:\s*false\s*$", example_config)
    if false_match is not None:
        add_problem(
            problems,
            example_config_path,
            example_config,
            false_match.start(),
            "example configuration must not disable TLS verification",
        )


def report(root: Path, problems: list[Problem]) -> int:
    if not problems:
        print("Documentation contract passed.")
        return 0

    for problem in problems:
        relative = problem.path.relative_to(root).as_posix()
        if os.environ.get("GITHUB_ACTIONS") == "true":
            message = problem.message.replace("%", "%25").replace("\n", "%0A")
            print(f"::error file={relative},line={problem.line}::{message}")
        print(f"{relative}:{problem.line}: error: {problem.message}")
    print(f"Documentation contract failed with {len(problems)} error(s).")
    return 1


def main() -> int:
    args = parse_args()
    root = args.root.resolve()
    manifest = read_manifest(root / "Cargo.toml")
    if args.print_msrv:
        print(manifest.rust_version)
        return 0

    readme_path = root / "README.md"
    readme = readme_path.read_text(encoding="utf-8")
    problems: list[Problem] = []
    check_msrv(readme_path, readme, manifest, problems)
    check_features(readme_path, readme, manifest, problems)
    check_install_guidance(readme_path, readme, problems)
    quickstart_path = root / "examples/quickstart.rs"
    check_quickstart(readme_path, readme, quickstart_path, problems)
    check_readme_contract(readme_path, readme, problems)
    check_tls_examples(
        quickstart_path,
        root / "config/unifi.example.yaml",
        problems,
    )
    check_local_links(root, problems)
    return report(root, problems)


if __name__ == "__main__":
    sys.exit(main())
