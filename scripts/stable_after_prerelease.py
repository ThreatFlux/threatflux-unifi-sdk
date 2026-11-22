"""Prepare a signed-commit request for the stable version proposed after a prerelease."""

import base64
import json
import os
import re
import tomllib
from pathlib import Path

from release_source import GRADUATION_PREFIX, STABLE_VERSION, TAGGED_VERSION

COMMIT_MUTATION = """
mutation ($input: CreateCommitOnBranchInput!) {
  createCommitOnBranch(input: $input) {
    commit { signature { isValid } author { user { login } } }
  }
}
"""


def replace_version(block: str, previous: str, version: str) -> str:
    updated, count = re.subn(
        rf'(?m)^version = "{re.escape(previous)}"$', f'version = "{version}"', block
    )
    if count != 1:
        raise ValueError("expected one version matching the manifest")
    return updated


def version_files(manifest: str, lock: str, version: str) -> dict[str, str]:
    package = tomllib.loads(manifest)["package"]
    previous = package["version"]
    if not re.fullmatch(TAGGED_VERSION, previous) or "-" not in previous:
        raise ValueError("stable-after-prerelease proposal needs a prerelease manifest")
    if not re.fullmatch(STABLE_VERSION, version):
        raise ValueError("expected a stable proposed version")
    number = tuple(map(int, version.split(".")))
    baseline = tuple(map(int, previous.split("-", 1)[0].split(".")))
    if number < baseline:
        raise ValueError("proposed stable version is older than the prerelease")
    manifest_sections = re.split(r"(?m)(?=^\[)", manifest)
    package_index = next(
        index
        for index, section in enumerate(manifest_sections)
        if section.startswith("[package]\n")
    )
    manifest_sections[package_index] = replace_version(
        manifest_sections[package_index], previous, version
    )
    lock_sections = re.split(r"(?m)(?=^\[\[package\]\])", lock)
    local_entry = f'[[package]]\nname = "{package["name"]}"\nversion = "{previous}"\n'
    lock_index = next(
        index
        for index, section in enumerate(lock_sections)
        if section.startswith(local_entry) and "\nsource = " not in section
    )
    lock_sections[lock_index] = replace_version(
        lock_sections[lock_index], previous, version
    )
    return {
        "Cargo.toml": "".join(manifest_sections),
        "Cargo.lock": "".join(lock_sections),
    }


def main() -> None:
    source = os.environ["EXPECTED_SOURCE"]
    if not re.fullmatch(r"[0-9a-f]{40}", source):
        raise ValueError("expected an immutable checked main SHA")
    version = os.environ["VERSION"]
    files = version_files(
        Path("Cargo.toml").read_text(encoding="utf-8"),
        Path("Cargo.lock").read_text(encoding="utf-8"),
        version,
    )
    branch = f"{GRADUATION_PREFIX}{source}/v{version}"
    request = {
        "query": COMMIT_MUTATION,
        "variables": {
            "input": {
                "branch": {
                    "repositoryNameWithOwner": os.environ["GITHUB_REPOSITORY"],
                    "branchName": branch,
                },
                "expectedHeadOid": source,
                "message": {"headline": f"chore: release v{version}"},
                "fileChanges": {
                    "additions": [
                        {
                            "path": path,
                            "contents": base64.b64encode(contents.encode()).decode(
                                "ascii"
                            ),
                        }
                        for path, contents in files.items()
                    ]
                },
            }
        },
    }
    output = Path(os.environ["RUNNER_TEMP"])
    (output / "stable-proposal.json").write_text(json.dumps(request), encoding="utf-8")
    for path, contents in files.items():
        (output / f"stable-proposal-{path}").write_text(
            contents, encoding="utf-8", newline="\n"
        )
    print(f"branch={branch}")


if __name__ == "__main__":
    main()
