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
GRADUATION_PREFIX = "automation/stable-after-prerelease/"
STABLE_VERSION = r"(0|[1-9][0-9]*)\.(0|[1-9][0-9]*)\.(0|[1-9][0-9]*)"
# Match the Release workflow's existing version format for tag publication.
TAGGED_VERSION = r"[0-9]+\.[0-9]+\.[0-9]+(?:-[a-zA-Z0-9]+)?"


def matches_proposal(
    pull: dict, repository: str, source: str, version: str, parent: str = ""
) -> bool:
    if not (
        pull["merged_at"] is not None
        and pull["merge_commit_sha"] == source
        and pull["base"]["ref"] == "main"
        and (pull["head"]["repo"] or {}).get("full_name") == repository
        and pull["title"] == f"chore(release): v{version}"
    ):
        return False
    if "-" in version:
        return (
            pull["head"]["ref"] == f"release/prerelease/v{version}"
            and pull["user"].get("type") == "User"
        )
    return (
        pull["head"]["ref"]
        in {RELEASE_BRANCH, f"{GRADUATION_PREFIX}{parent}/v{version}"}
        and pull["user"]["login"] == "threatflux-automation[bot]"
    )


def same_reviewed_tree(
    proposal: dict, parent: str, tree: str, author: str = "threatflux-automation[bot]"
) -> bool:
    return (
        (proposal["author"] or {}).get("login") == author
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
        if not matches_proposal(candidate, repository, source, version, parent):
            continue
        proposal = api(f"{base}/commits/{candidate['head']['sha']}")
        if same_reviewed_tree(proposal, parent, tree, candidate["user"]["login"]):
            return
    raise ValueError("release requires the exact merged proposal on its reviewed base")


def manifest_version() -> str:
    with Path("Cargo.toml").open("rb") as manifest:
        return tomllib.load(manifest)["package"]["version"]


def open_proposal(repository: str) -> bool:
    query = urlencode({"state": "open", "base": "main", "per_page": 100})
    pulls = api(f"repos/{repository}/pulls?{query}", pages=True)
    return any(
        (pull["head"]["repo"] or {}).get("full_name") == repository
        and (
            pull["head"]["ref"] == RELEASE_BRANCH
            or pull["head"]["ref"].startswith(GRADUATION_PREFIX)
        )
        for pull in pulls
    )


def main() -> None:
    version = manifest_version()
    if sys.argv[1:] == ["--docker"]:
        parent_version = tomllib.load(sys.stdin.buffer)["package"]["version"]
        publish = version == parent_version
        build = publish or os.environ["EVENT_NAME"] in {"schedule", "workflow_dispatch"}
        if not publish:
            print(
                "version-bump images are published by the release tag", file=sys.stderr
            )
        print(f"build={str(build).lower()}")
        print(f"publish={str(publish).lower()}")
        return

    tagged = sys.argv[1:] == ["--tagged"]
    if not re.fullmatch(TAGGED_VERSION, version):
        raise ValueError("invalid manifest version for this release workflow")
    source = os.environ["SOURCE_SHA"]
    repository = os.environ["GITHUB_REPOSITORY"]
    parent = os.environ["SOURCE_PARENT"]
    tree = os.environ["SOURCE_TREE"]
    if tagged:
        require_release_proposal(repository, source, version, parent, tree)
        return

    if source != api(f"repos/{repository}/commits/main")["sha"]:
        print("main advanced before release authorization", file=sys.stderr)
        print("authorized=false")
        return
    version_is_tagged = f"v{version}" in sys.stdin.read().splitlines()
    if "-" in version:
        if not version_is_tagged:
            raise ValueError(
                "prerelease must be published before planning a stable release"
            )
        release = api(f"repos/{repository}/releases/tags/v{version}")
        if release["draft"] or not release["prerelease"]:
            raise ValueError("expected a published GitHub prerelease")
        ancestry = api(f"repos/{repository}/compare/v{version}...{source}")
        if ancestry["status"] not in {"ahead", "identical"}:
            raise ValueError("published prerelease is outside the checked main history")
    elif not re.fullmatch(STABLE_VERSION, version):
        raise ValueError("invalid stable manifest version")
    if not version_is_tagged:
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
    if "-" in version:
        print("graduate=true")


if __name__ == "__main__":
    main()
