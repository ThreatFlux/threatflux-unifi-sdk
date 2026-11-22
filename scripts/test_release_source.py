"""Exercise the release source checks without GitHub credentials."""

import unittest
from unittest.mock import patch

import release_source as policy

REPOSITORY = "ThreatFlux/threatflux-unifi-sdk"
SOURCE = "1" * 40
PROPOSAL = "2" * 40
BASE = "3" * 40
TREE = "4" * 40
VERSION = "0.7.13"


class ReleaseSourceTests(unittest.TestCase):
    def setUp(self):
        self.pull = {
            "merged_at": "2026-11-22T00:00:00Z",
            "merge_commit_sha": SOURCE,
            "base": {"ref": "main"},
            "head": {
                "repo": {"full_name": REPOSITORY},
                "ref": "automation/release",
                "sha": PROPOSAL,
            },
            "user": {"login": "threatflux-automation[bot]"},
            "title": f"chore(release): v{VERSION}",
        }
        self.proposal = {
            "author": self.pull["user"],
            "parents": [{"sha": BASE}],
            "commit": {"verification": {"verified": True}, "tree": {"sha": TREE}},
        }

    def test_exact_merged_proposal_authorizes_release(self):
        with (
            patch.object(policy, "api", side_effect=[[self.pull], self.proposal]),
            patch.object(policy, "git", side_effect=[BASE, TREE]),
        ):
            policy.require_release_proposal(REPOSITORY, SOURCE, VERSION)

    def test_later_commits_and_ordinary_version_bumps_do_not_match(self):
        self.assertFalse(
            policy.matches_proposal(self.pull, REPOSITORY, PROPOSAL, VERSION)
        )
        self.pull["user"] = {"login": "developer"}
        self.assertFalse(
            policy.matches_proposal(self.pull, REPOSITORY, SOURCE, VERSION)
        )

    def test_stale_base_and_changed_tree_are_rejected(self):
        self.assertTrue(policy.same_reviewed_tree(self.proposal, BASE, TREE))
        self.assertFalse(policy.same_reviewed_tree(self.proposal, "5" * 40, TREE))
        self.assertFalse(policy.same_reviewed_tree(self.proposal, BASE, "5" * 40))
        self.proposal["commit"]["verification"]["verified"] = False
        self.assertFalse(policy.same_reviewed_tree(self.proposal, BASE, TREE))

    def test_branch_images_only_publish_when_the_manifest_version_is_unchanged(self):
        for parent_version, expected in [
            (VERSION, "build=true"),
            ("0.7.12", "build=false"),
        ]:
            with (
                self.subTest(parent_version=parent_version),
                patch.object(policy.sys, "argv", ["release_source.py", "--docker"]),
                patch.object(
                    policy, "version_at", side_effect=[VERSION, parent_version]
                ),
                patch("builtins.print") as output,
            ):
                policy.main()
                output.assert_called_once_with(expected)


if __name__ == "__main__":
    unittest.main()
