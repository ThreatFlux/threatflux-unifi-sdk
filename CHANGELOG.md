# Changelog

Per-release notes are generated from Conventional Commits and attached to each
[GitHub Release](https://github.com/ThreatFlux/threatflux-unifi-sdk/releases).
This file records changes that need more context than a commit subject.

## [Unreleased]

### Added

- `release.yml` and `auto-release.yml` accept a `dry_run` dispatch input.
  A release dry run builds the binaries, SBOM, crate package
  (`cargo publish --dry-run`) and container image without creating a tag or
  GitHub Release, uploading assets, publishing the crate or pushing images.

### Changed

- The package version in `Cargo.toml` is aligned with the existing `v0.7.5`
  tag, so the next automated release is `0.7.6` or later instead of a
  `0.5.x`/`0.6.x` version that would sort below the latest tag.
- `release.yml` no longer rewrites `Cargo.toml` before packaging. A release
  now fails before any tag or GitHub Release is created when the requested
  version differs from the manifest version.
- `release.yml` keeps the notes auto-release already wrote on an existing
  GitHub Release, creates a missing tag at the commit it builds, and refuses
  to release an existing tag that points at a different commit.
- A failed crates.io publish now fails the `release.yml` run instead of being
  ignored. Versions with a `-` pre-release suffix are still never published
  to crates.io.
- `release.yml` publishes to crates.io with trusted publishing (OIDC) instead
  of a long-lived registry token secret: the publish job runs in the
  `crates-io` environment and `rust-lang/crates-io-auth-action` exchanges the
  job's GitHub identity for a short-lived token. A release whose version is
  already on crates.io skips the publish instead of failing, and dry runs
  request no token. The workflow token is read-only except where a job writes
  the GitHub Release.
- `auto-release.yml` uses the `ThreatFlux/github_actions` reusable release
  workflow at `v0.7.6`. That version stops a release whose `Cargo.toml`
  version is lower than the latest release tag before anything is written,
  and its dry run never dispatches the downstream release workflows.
- The container image moves off Debian 12 (bookworm). It builds on
  `rust:1.99.0-trixie` and runs on `gcr.io/distroless/cc-debian13:nonroot`,
  both pinned by digest. The runtime image has no shell or package manager,
  runs as the distroless `nonroot` user (UID/GID 65532 instead of the former
  `app` user, UID 1000) with `/data` as its working directory, and keeps
  `tini` as PID 1, the `--version` health check, the CycloneDX SBOM at
  `/usr/share/doc/app/sbom.cdx.json` and the `linux/amd64` and `linux/arm64`
  platforms. `unifi-cli` is also on `PATH` next to `/usr/local/bin/app`.

## [0.7.5]

### Notes

- The `v0.7.5` tag and GitHub Release were created from a commit whose
  `Cargo.toml` still declared `0.5.4`, and the crates.io publish for that
  release failed. crates.io therefore has no `0.6.x` or `0.7.x` versions;
  the release after `0.5.4` on crates.io is the next automated release.
