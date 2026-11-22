"""Bind release publication to the reviewed automated version bump."""

from __future__ import annotations

import os
import re
import subprocess
import sys
import tomllib

from container_publish_policy import api


def git(*args: str) -> str:
    return subprocess.check_output(["git", *args], encoding="utf-8").strip()


def version_at(revision: str) -> str:
    return tomllib.loads(git("show", f"{revision}:Cargo.toml"))["package"]["version"]


def matches_proposal(pull: dict, repository: str, source: str, version: str) -> bool:
    return (
        pull["merged_at"] is not None
        and pull["merge_commit_sha"] == source
        and pull["base"]["ref"] == "main"
        and (pull["head"]["repo"] or {}).get("full_name") == repository
        and pull["head"]["ref"] == "automation/release"
        and pull["user"]["login"] == "threatflux-automation[bot]"
        and pull["title"] == f"chore(release): v{version}"
    )


def same_reviewed_tree(proposal: dict, parent: str, tree: str) -> bool:
    return (
        (proposal["author"] or {}).get("login") == "threatflux-automation[bot]"
        and proposal["commit"]["verification"]["verified"]
        and [item["sha"] for item in proposal["parents"]] == [parent]
        and proposal["commit"]["tree"]["sha"] == tree
    )


def require_release_proposal(repository: str, source: str, version: str) -> None:
    base = f"repos/{repository}"
    parent = git("rev-parse", "HEAD^1")
    tree = git("rev-parse", "HEAD^{tree}")
    candidates = api(f"{base}/commits/{source}/pulls?per_page=100", pages=True)
    for candidate in candidates:
        if not matches_proposal(candidate, repository, source, version):
            continue
        proposal = api(f"{base}/commits/{candidate['head']['sha']}")
        if same_reviewed_tree(proposal, parent, tree):
            return
    raise ValueError(
        "release requires the merged automated proposal on its reviewed base"
    )


def main() -> None:
    version = version_at("HEAD")
    if sys.argv[1:] == ["--docker"]:
        # Only the tag run publishes a version-bump commit. The first-parent
        # comparison works for squash, rebase and merge commits and for retries.
        branch_build = version == version_at("HEAD^1")
        print(f"build={str(branch_build).lower()}")
        return

    if not re.fullmatch(r"(0|[1-9][0-9]*)\.(0|[1-9][0-9]*)\.(0|[1-9][0-9]*)", version):
        raise ValueError("expected a stable manifest version")
    source = git("rev-parse", "HEAD")
    repository = os.environ["GITHUB_REPOSITORY"]
    if source != api(f"repos/{repository}/commits/main")["sha"]:
        print("main advanced before release authorization", file=sys.stderr)
        print("authorized=false")
        return
    if f"v{version}" not in git("tag", "--list", f"v{version}").splitlines():
        require_release_proposal(repository, source, version)
    print("authorized=true")


if __name__ == "__main__":
    main()
