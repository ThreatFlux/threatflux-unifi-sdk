"""Exercise the release source checks without GitHub credentials."""

import unittest
from io import BytesIO, StringIO, TextIOWrapper
from unittest.mock import patch

import release_source as policy

REPOSITORY = "ThreatFlux/threatflux-unifi-sdk"
SOURCE = "1" * 40
PROPOSAL = "2" * 40
BASE = "3" * 40
TREE = "4" * 40
VERSION = "0.7.13"
RELEASE_ENV = {
    "SOURCE_SHA": SOURCE,
    "GITHUB_REPOSITORY": REPOSITORY,
    "SOURCE_PARENT": BASE,
    "SOURCE_TREE": TREE,
}


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
        with patch.object(policy, "api", side_effect=[[self.pull], self.proposal]):
            policy.require_release_proposal(REPOSITORY, SOURCE, VERSION, BASE, TREE)

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

    def test_automatic_run_preserves_an_open_proposal(self):
        with (
            patch.dict(policy.os.environ, RELEASE_ENV),
            patch.object(policy.sys, "argv", ["release_source.py"]),
            patch.object(policy.sys, "stdin", StringIO(f"v{VERSION}\n")),
            patch.object(policy.sys, "stdout", StringIO()) as output,
            patch.object(policy, "manifest_version", return_value=VERSION),
            patch.object(policy, "api", side_effect=[{"sha": SOURCE}, [self.pull]]),
        ):
            policy.main()
            self.assertEqual(output.getvalue(), "authorized=false\n")

    def test_tag_needs_the_exact_proposal_even_after_main_advances(self):
        self.check_tagged_release(VERSION, [self.pull], self.proposal)

    def set_prerelease_proposal(self, version):
        author = {"login": "maintainer", "type": "User"}
        self.pull["title"] = f"chore(release): v{version}"
        self.pull["head"]["ref"] = f"release/prerelease/v{version}"
        self.pull["user"] = author
        self.proposal["author"] = author

    def check_tagged_release(self, version, candidates, proposal):
        with (
            patch.dict(policy.os.environ, RELEASE_ENV),
            patch.object(policy.sys, "argv", ["release_source.py", "--tagged"]),
            patch.object(policy, "manifest_version", return_value=version),
            patch.object(policy, "api", side_effect=[candidates, proposal]) as api,
        ):
            policy.main()
            self.assertNotIn("commits/main", api.call_args_list[0].args[0])

    def test_prerelease_checks_the_version_and_reviewed_tree(self):
        version = f"{VERSION}-rc1"
        with self.assertRaisesRegex(ValueError, "exact merged proposal"):
            self.check_tagged_release(version, [self.pull], self.proposal)
        self.set_prerelease_proposal(version)
        self.pull["head"]["ref"] = "release/prerelease/v0.7.13-rc2"
        with self.assertRaisesRegex(ValueError, "exact merged proposal"):
            self.check_tagged_release(version, [self.pull], self.proposal)
        self.set_prerelease_proposal(version)
        self.proposal["commit"]["tree"]["sha"] = "5" * 40
        with self.assertRaisesRegex(ValueError, "exact merged proposal"):
            self.check_tagged_release(version, [self.pull], self.proposal)

    def test_tagged_prerelease_accepts_a_signed_same_repo_proposal(self):
        for version in ("0.7.13-rc1", "0.7.13-Alpha2"):
            with self.subTest(version=version):
                self.set_prerelease_proposal(version)
                self.check_tagged_release(version, [self.pull], self.proposal)
        self.proposal["commit"]["verification"]["verified"] = False
        with self.assertRaisesRegex(ValueError, "exact merged proposal"):
            self.check_tagged_release(version, [self.pull], self.proposal)

    def test_invalid_versions_never_reach_proposal_lookup(self):
        for args, version in (
            (["--tagged"], f"{VERSION}-rc.1"),
            (["--tagged"], f"{VERSION}+build1"),
            ([], f"{VERSION}-rc1"),
        ):
            with (
                self.subTest(args=args, version=version),
                patch.object(policy.sys, "argv", ["release_source.py", *args]),
                patch.object(policy, "manifest_version", return_value=version),
                patch.object(policy, "api") as api,
            ):
                with self.assertRaisesRegex(ValueError, "invalid manifest version"):
                    policy.main()
                api.assert_not_called()

    def test_docker_branch_events_at_a_release_commit(self):
        for event, build in (
            ("push", "false"),
            ("schedule", "true"),
            ("workflow_dispatch", "true"),
        ):
            parent_manifest = b'[package]\nversion = "0.7.12"\n'
            with (
                self.subTest(event=event),
                TextIOWrapper(BytesIO(parent_manifest)) as parent_input,
                patch.dict(policy.os.environ, {"EVENT_NAME": event}),
                patch.object(policy.sys, "argv", ["release_source.py", "--docker"]),
                patch.object(policy.sys, "stdin", parent_input),
                patch.object(policy, "manifest_version", return_value=VERSION),
                patch.object(policy.sys, "stdout", StringIO()) as output,
            ):
                policy.main()
                self.assertEqual(output.getvalue(), f"build={build}\npublish=false\n")

    def test_docker_ordinary_branch_push_still_builds_and_publishes(self):
        manifest = f'[package]\nversion = "{VERSION}"\n'.encode()
        with (
            TextIOWrapper(BytesIO(manifest)) as parent_input,
            patch.dict(policy.os.environ, {"EVENT_NAME": "push"}),
            patch.object(policy.sys, "argv", ["release_source.py", "--docker"]),
            patch.object(policy.sys, "stdin", parent_input),
            patch.object(policy, "manifest_version", return_value=VERSION),
            patch.object(policy.sys, "stdout", StringIO()) as output,
        ):
            policy.main()
            self.assertEqual(output.getvalue(), "build=true\npublish=true\n")


if __name__ == "__main__":
    unittest.main()
