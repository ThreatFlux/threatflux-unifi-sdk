# Changelog

Per-release notes are generated from Conventional Commits and attached to each
[GitHub Release](https://github.com/ThreatFlux/threatflux-unifi-sdk/releases).
This file records changes that need more context than a commit subject.

## [Unreleased]

### Changed

- The package version in `Cargo.toml` is aligned with the existing `v0.7.5`
  tag, so the next automated release is `0.7.6` or later instead of a
  `0.5.x`/`0.6.x` version that would sort below the latest tag.
- `release.yml` no longer rewrites `Cargo.toml` before packaging. A release
  now fails before any tag or GitHub Release is created when the requested
  version differs from the manifest version.

## [0.7.5]

### Notes

- The `v0.7.5` tag and GitHub Release were created from a commit whose
  `Cargo.toml` still declared `0.5.4`, and the crates.io publish for that
  release failed. crates.io therefore has no `0.6.x` or `0.7.x` versions;
  the release after `0.5.4` on crates.io is the next automated release.
