"""Bind release publication to the reviewed automated version bump."""

from __future__ import annotations

import os
import re
import sys
import tomllib
from pathlib import Path
from urllib.parse import urlencode

from container_publish_policy import api

RELEASE_BRANCH = "automation/release"


def matches_proposal(pull: dict, repository: str, source: str, version: str) -> bool:
    return (
        pull["merged_at"] is not None
        and pull["merge_commit_sha"] == source
        and pull["base"]["ref"] == "main"
        and (pull["head"]["repo"] or {}).get("full_name") == repository
        and pull["head"]["ref"] == RELEASE_BRANCH
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


def require_release_proposal(
    repository: str, source: str, version: str, parent: str, tree: str
) -> None:
    base = f"repos/{repository}"
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


def manifest_version() -> str:
    with Path("Cargo.toml").open("rb") as manifest:
        return tomllib.load(manifest)["package"]["version"]


def open_proposal(repository: str) -> bool:
    owner = repository.split("/", 1)[0]
    query = urlencode(
        {"state": "open", "base": "main", "head": f"{owner}:{RELEASE_BRANCH}"}
    )
    pulls = api(f"repos/{repository}/pulls?{query}")
    return bool(pulls)


def main() -> None:
    version = manifest_version()
    if sys.argv[1:] == ["--docker"]:
        # Only the tag run publishes a version-bump commit. The first-parent
        # comparison works for squash, rebase and merge commits and for retries.
        parent_version = tomllib.load(sys.stdin.buffer)["package"]["version"]
        branch_build = version == parent_version
        if not branch_build:
            print("version-bump images are built by the release tag", file=sys.stderr)
        print(f"build={str(branch_build).lower()}")
        return

    if not re.fullmatch(r"(0|[1-9][0-9]*)\.(0|[1-9][0-9]*)\.(0|[1-9][0-9]*)", version):
        raise ValueError("expected a stable manifest version")
    source = os.environ["SOURCE_SHA"]
    repository = os.environ["GITHUB_REPOSITORY"]
    parent = os.environ["SOURCE_PARENT"]
    tree = os.environ["SOURCE_TREE"]
    if sys.argv[1:] == ["--tagged"]:
        require_release_proposal(repository, source, version, parent, tree)
        return

    if source != api(f"repos/{repository}/commits/main")["sha"]:
        print("main advanced before release authorization", file=sys.stderr)
        print("authorized=false")
        return
    if f"v{version}" not in sys.stdin.read().splitlines():
        require_release_proposal(repository, source, version, parent, tree)
    elif open_proposal(repository):
        # The pinned action can force-update its branch with a lower version.
        # Preserve the reviewed proposal until it is merged or closed.
        print(
            "a release proposal is open; preserving its version and source",
            file=sys.stderr,
        )
        print("authorized=false")
        return
    print("authorized=true")


if __name__ == "__main__":
    main()
