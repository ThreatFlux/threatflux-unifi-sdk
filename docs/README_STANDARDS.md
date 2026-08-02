# ThreatFlux README standards

These standards describe the machine-checked documentation contract used by
this repository. They are useful defaults for other Rust SDKs, but every
provider/controller claim must be derived from that project's code and tests.

## Required content

A public SDK README should make these facts discoverable without source-code
inspection:

1. Purpose, maintenance/pre-1.0 status, and official-affiliation status.
2. Package, API docs, license, MSRV, CI, and security badges.
3. A crates.io dependency and a clearly distinguished Git dependency.
4. Prerequisites and a minimal quickstart compiled on MSRV and stable Rust.
5. Source-derived service/API coverage with explicit compatibility boundaries.
6. Every Cargo feature and whether it actually gates code or dependencies.
7. Library configuration separately from CLI flags/environment variables.
8. Authentication, TLS defaults, timeout, retry, and session-expiry behavior.
9. Runnable examples and navigation to deeper docs.
10. Contribution, support, vulnerability-reporting, release, and license links.

## Accuracy rules

- Put security-sensitive defaults before the first copyable example.
- A secure example must not silently contradict an insecure implementation
  default; show the explicit secure override and describe migration separately.
- Distinguish the package name, Rust import name, and installed binary name.
- Do not call CLI environment discovery library behavior.
- Do not describe Cargo feature names as compile-time gates without matching
  `cfg(feature)` usage.
- Treat implemented endpoints separately from controller/model/firmware
  compatibility. Publish only compatibility backed by identified tests.
- Describe authentication recovery as a request/state sequence. Do not call a
  later login attempt an automatic replay of the failed request.
- Call out undocumented or reverse-engineered API risk without claiming
  provider endorsement or universal availability.
- Link deeper operational details instead of turning the README into an
  unmaintainable method catalog.

## Machine-checked contract

Run:

```bash
make docs-check
```

The checker verifies:

- README MSRV claims match `package.rust-version`.
- The crates.io dependency matches the crate's current major/minor version.
- The Cargo feature table matches `[features]` exactly.
- The README quickstart is identical to `examples/quickstart.rs`.
- The secure TLS call remains in the quickstart and example configuration.
- Required affiliation, compatibility, CLI/library, and navigation text
  remains present.
- Obsolete binary, compatibility, feature-gating, and re-login claims do not
  return.
- Local Markdown links resolve inside the repository.
- Template placeholders do not leak into published files.

The documentation workflow also lints maintained Markdown, checks external
links, compiles the quickstart on the MSRV and stable Rust, runs doctests, and
builds rustdoc with warnings denied.

## Template files

`README_TEMPLATE.md` and `docs/TEMPLATE_BOOTSTRAP_CHECKLIST.md` remain generic
template assets. They are excluded from project-specific Markdown style checks
and from placeholder failures.
