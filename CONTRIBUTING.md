# Contributing to ThreatFlux UniFi SDK

Contributions are welcome. This guide covers local validation, documentation
accuracy, and the extra care required for controller-mutating tests.

## Getting started

1. Fork the repository.
2. Clone your fork:
   `gh repo fork ThreatFlux/threatflux-unifi-sdk --clone`.
3. Create a branch: `git checkout -b feat/your-change`.
4. Make the change and update relevant tests and documentation.
5. Run `make ci`.
6. Open a pull request using a conventional-commit title.

## Development setup

The minimum supported Rust version is 1.96.0. The repository's
`rust-toolchain.toml` installs the pinned toolchain and required components.

```bash
make dev-setup
make build
make ci
```

Useful focused commands:

```bash
make fmt-check
make lint-strict
make test
make test-features
make test-doc
make docs-check
make docs
cargo package --locked --allow-dirty
```

## Code guidelines

- Preserve the async API unless a change explicitly introduces a new surface.
- Return `UnifiError` through the crate's `Result<T>` alias in library code.
- Re-export public services and models from `src/lib.rs` when appropriate.
- Keep model serialization aligned with controller payloads.
- Avoid `unwrap()` in production paths.
- Run the strict Clippy flags from `Makefile` for API or implementation
  changes.

## Documentation contract

Public claims must be traceable to source or tests:

- Edit `examples/quickstart.rs` and copy it exactly between the README
  `BEGIN QUICKSTART` and `END QUICKSTART` markers.
- Update [API coverage](docs/api-coverage.md) when services, models, sync
  resources, or compatibility evidence changes.
- Update [configuration and security](docs/configuration.md) when TLS,
  authentication, environment variables, timeouts, or retry behavior changes.
- Update the [CLI guide](docs/cli.md) when clap arguments or commands change.
- Do not describe a Cargo feature as gating code unless a corresponding
  `cfg(feature)` exists and is validated.
- Qualify controller compatibility by the tested controller/version evidence;
  an implemented endpoint is not a universal support guarantee.

`make docs-check` validates MSRV/version claims, the exact Cargo feature table,
quickstart synchronization, TLS guidance, local links, and template
placeholders.

## Tests

```bash
cargo test --locked --all-features
cargo clippy --locked --all-features --all-targets -- -D warnings
RUSTDOCFLAGS="-D warnings" cargo doc --locked --all-features --no-deps
```

### Credentialed controller tests

The `integration_tests` target is ignored by default and requires:

- `UNIFI_HOST`
- `UNIFI_USERNAME`
- `UNIFI_PASSWORD`
- optional `UNIFI_SITE` (default: `default`)
- optional `UNIFI_VERIFY_SSL` (default: `false`)
- optional `UNIFI_TIMEOUT_SECS` (default: `30`)

Run it explicitly:

```bash
cargo test --locked --test integration_tests \
  --features integration-tests -- --ignored --test-threads=1
```

Use `UNIFI_VERIFY_SSL=true` with a trusted certificate. These tests create and
delete a VLAN network and firewall address group; cleanup is best-effort. Use a
disposable/non-production site and a least-privilege test account.

## Commit and pull request guidelines

Use [Conventional Commits](https://www.conventionalcommits.org/):

- `feat`: new capability
- `fix`: bug fix
- `docs`: documentation-only change
- `refactor`: behavior-preserving code change
- `test`: test coverage
- `chore`: maintenance

Pull requests should explain the change, motivation, compatibility impact,
security impact, and validation performed.

## Security issues

Do not open public issues for vulnerabilities. Follow
[SECURITY.md](SECURITY.md).
