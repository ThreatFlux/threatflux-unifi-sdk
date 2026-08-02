# ThreatFlux UniFi SDK

[![Crates.io](https://img.shields.io/crates/v/threatflux-unifi-sdk.svg)](https://crates.io/crates/threatflux-unifi-sdk)
[![Documentation](https://docs.rs/threatflux-unifi-sdk/badge.svg)](https://docs.rs/threatflux-unifi-sdk)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/rust-1.96.0%2B-orange.svg)](https://www.rust-lang.org)
[![CI](https://github.com/ThreatFlux/threatflux-unifi-sdk/actions/workflows/ci.yml/badge.svg)](https://github.com/ThreatFlux/threatflux-unifi-sdk/actions/workflows/ci.yml)
[![Security](https://github.com/ThreatFlux/threatflux-unifi-sdk/actions/workflows/security.yml/badge.svg)](https://github.com/ThreatFlux/threatflux-unifi-sdk/actions/workflows/security.yml)

An async Rust client, typed service layer, declarative sync engine, and CLI for
automating UniFi Network controllers.

> **Unofficial community SDK:** this project is not an official Ubiquiti or
> UniFi SDK and is not affiliated with or endorsed by Ubiquiti Inc. It uses
> controller APIs whose observed behavior can change between UniFi Network and
> firmware releases.

The crate is pre-1.0. Public APIs and controller compatibility may change as
real-world coverage grows.

## What it covers

- Session login, cookie and CSRF handling, and UniFi OS/classic URL routing
- Networks and VLANs, firewall rules and groups, and port forwarding
- Clients, devices, DHCP reservations, local DNS, and traffic controls
- WireGuard and site-to-site VPNs, static routes, backups, and site information
- YAML/JSON diff, apply, export, optional pruning, and best-effort rollback
- The `unifi-cli` command-line interface for common operations
- Typed models plus low-level GET, POST, PUT, DELETE, and command requests

See [API coverage](docs/api-coverage.md) for the source-derived surface and its
boundaries.

## Compatibility status

The client implements both the UniFi OS proxy path and the classic controller
path. Detection is a login-endpoint heuristic, not version negotiation. The
repository's credentialed tests exercise a smoke check, a network lifecycle,
and a firewall-group lifecycle against whichever controller the maintainer
configures; they do not establish a controller-model or firmware support
matrix.

Many controller endpoints are internal, undocumented, or reverse-engineered.
Treat an implemented operation as available in the SDK, not as a guarantee
that every UniFi Network release exposes it. Validate read and write workflows
against a non-production site before deployment.

## Requirements

- Rust 1.96.0 or newer (MSRV: 1.96.0)
- Network access to a UniFi Network controller
- Controller credentials accepted by its local login endpoint
- A controller certificate trusted by the client host for secure TLS

## Installation

Use Cargo to add the latest published crates.io release for normal
applications:

```bash
cargo add threatflux-unifi-sdk
```

Cargo writes the selected release requirement to your application's
`Cargo.toml`, so this command remains correct when a new SDK version is
published.

Track the Git repository only when you intentionally need unreleased changes:

```toml
[dependencies]
threatflux-unifi-sdk = { git = "https://github.com/ThreatFlux/threatflux-unifi-sdk", branch = "main" }
```

A branch dependency is not reproducible over time. Before committing a
production dependency, replace `branch = "main"` with a pinned `rev`.

Install the packaged CLI from crates.io with:

```bash
cargo install --locked threatflux-unifi-sdk
unifi-cli --help
```

## Security-sensitive TLS default

**TLS verification is disabled by default** in `UnifiConfig::new` and in the
declarative connection schema for compatibility with controllers that ship
self-signed certificates. Internally, this enables reqwest's
`danger_accept_invalid_certs` option. It can expose credentials and session
cookies to an active network attacker.

For production, install a certificate trusted by the client runtime (or trust
the controller's issuing CA) and always call `.with_verify_ssl(true)`. Limit
`.with_verify_ssl(false)` to an isolated, trusted development network after
accepting the risk. Changing the library default would be compatibility
sensitive and is not part of this documentation change.

See [configuration and security](docs/configuration.md) before connecting to a
controller.

## Quick start

After adding the crate to an application, use it from Rust code as below.
`UNIFI_HOST`, `UNIFI_USERNAME`, and `UNIFI_PASSWORD` are required at runtime;
`UNIFI_SITE` defaults to `default`. This exact example is compiled on the MSRV
and stable Rust in CI.

<!-- BEGIN QUICKSTART -->
```rust
use std::env;

use threatflux_unifi_sdk::{NetworkService, UnifiClient, UnifiConfig};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = UnifiConfig::new(
        env::var("UNIFI_HOST")?,
        env::var("UNIFI_USERNAME")?,
        env::var("UNIFI_PASSWORD")?,
    )
    .with_site(env::var("UNIFI_SITE").unwrap_or_else(|_| "default".to_owned()))
    .with_verify_ssl(true)
    .with_timeout(30);

    let client = UnifiClient::connect(config).await?;
    let networks = NetworkService::new(&client).list().await?;

    for network in networks {
        println!("{}", network.name);
    }

    Ok(())
}
```
<!-- END QUICKSTART -->

The environment loading above belongs to the application. `UnifiConfig` does
not read environment variables. Pass values from your preferred secret/config
provider instead of hardcoding credentials.

## Service surface

| Service | Implemented operations |
| ------- | ---------------------- |
| `NetworkService` | Network/VLAN list, lookup, create, update, delete, DHCP, DNS, and enablement |
| `FirewallService` | Rules, rulesets, groups, ordering, state toggles, and common rule helpers |
| `PortForwardService` | Port-forward CRUD, state toggles, and TCP/UDP helpers |
| `ClientService` | Client lookup, block, kick, guest access, fixed IPs, groups, and statistics |
| `DeviceService` | Device lookup, adoption, restart, provisioning, upgrades, ports, PoE, and LEDs |
| `DhcpService` | Reservation and lease operations |
| `DnsService` | Local records, upstream servers, and filtering settings |
| `TrafficService` | Traffic rules, domain/IP blocking, rate limits, and statistics |
| `VpnService` | WireGuard servers/peers, site-to-site VPNs, status, and peer configs |
| `RoutingService` | Static-route CRUD and route-table queries |
| `BackupService` | Backup lifecycle, settings, downloads, restore, export, and pruning |
| `SiteService` | Sites, health, system information, events, alarms, and DPI statistics |

The service modules and models are always exported. The Cargo service-name
features do not currently gate this surface.

## Configuration and authentication

`UnifiConfig` is the programmatic SDK configuration:

| Field | Default | Notes |
| ----- | ------- | ----- |
| `host` | Required | URL, hostname, or IP; HTTPS is added when no scheme is present |
| `username` | Required | Sent to the controller login endpoint |
| `password` | Required | Keep in a secret provider; `Debug` on the config includes this field |
| `site` | `default` | Active UniFi site |
| `verify_ssl` | `false` | Security-sensitive compatibility default; enable in production |
| `timeout_secs` | `30` | reqwest request timeout |

`UnifiClient::connect` detects a controller path and logs in. Typed
GET/POST/PUT and raw GET paths mark the session unauthenticated on 401 and
return `UnifiError::SessionExpired`; the failed request is not replayed. If
the caller explicitly retries, the next request attempts login first. DELETE
currently uses a separate error path and does not clear session state on 401.
Do not blindly retry mutations whose outcome is unknown.

There is no SDK-level retry/backoff policy. Use
`UnifiError::is_retryable()` as classification input for an
application-owned policy. See [configuration and security](docs/configuration.md)
for CLI environment variables, declarative substitution, rate limits, and
timeout behavior.

## Declarative configuration

`load_config` reads YAML or JSON and substitutes `${NAME}` in string values.
An unset variable remains literal so configuration validation does not silently
replace it with an empty secret.

```bash
export UNIFI_HOST=unifi.example.net
export UNIFI_USERNAME=automation
export UNIFI_PASSWORD='use-a-secret-manager-in-production'

unifi-cli diff --config config/unifi.example.yaml
unifi-cli sync --config config/unifi.example.yaml --dry-run
```

Review [the example configuration](config/unifi.example.yaml) and the
[configuration guide](docs/configuration.md). `--prune` enables destructive
deletion of managed resource types that are absent from the desired config; it
is off by default.

## CLI

The binary is named `unifi-cli`. It reads connection flags or CLI-specific
environment variables:

```bash
export UNIFI_HOST=unifi.example.net
export UNIFI_USERNAME=automation
export UNIFI_PASSWORD='use-a-secret-manager-in-production'
export UNIFI_VERIFY_SSL=true

unifi-cli status
unifi-cli clients active
unifi-cli devices list
```

See the [CLI guide](docs/cli.md) for the command surface, exact environment
variable names, TLS guidance, and declarative workflows.

## Cargo features

The service-name features are compatibility markers today: there are no
`cfg(feature = "...")` gates in the crate, so disabling them does not remove
modules, services, dependencies, or CLI commands. They must not be used as a
security or footprint boundary.

| Feature | Default | Current behavior |
| ------- | ------- | ---------------- |
| `default` | — | Enables `full` |
| `full` | Yes | Enables all service-name marker features |
| `firewall` | Via `full` | Marker only; `FirewallService` is always compiled |
| `vpn` | Via `full` | Marker only; `VpnService` is always compiled |
| `traffic` | Via `full` | Marker only; `TrafficService` is always compiled |
| `clients` | Via `full` | Marker only; `ClientService` is always compiled |
| `devices` | Via `full` | Marker only; `DeviceService` is always compiled |
| `dhcp` | Via `full` | Marker only; `DhcpService` is always compiled |
| `dns` | Via `full` | Marker only; `DnsService` is always compiled |
| `integration-tests` | No | Enables the credentialed `integration_tests` target |

## Errors and low-level access

Public methods return `threatflux_unifi_sdk::Result<T>` with
`UnifiError`. Common variants distinguish authentication, expired sessions,
rate limits, not-found responses, connection failures, invalid responses, and
controller API errors. The raw GET helpers return response bodies for custom
handling and only special-case 401; callers must interpret other HTTP statuses.

`UnifiClient` also exposes typed low-level `get`, `get_with_query`,
`post`, `put`, `delete`, and `command` methods for endpoints not yet
covered by a service. Endpoint strings remain controller-version-sensitive.

## Documentation

- [API coverage and compatibility](docs/api-coverage.md)
- [Configuration, authentication, TLS, and retries](docs/configuration.md)
- [CLI guide](docs/cli.md)
- [Generated API reference](https://docs.rs/threatflux-unifi-sdk)
- [Example configuration](config/unifi.example.yaml)

## Development

```bash
make docs-check
make ci
```

Credentialed controller tests are ignored by default. Their setup and safety
notes are in [CONTRIBUTING.md](CONTRIBUTING.md).

## Project links

- [Contributing](CONTRIBUTING.md)
- [Security policy](SECURITY.md)
- [Code of conduct](CODE_OF_CONDUCT.md)
- [Issue tracker](https://github.com/ThreatFlux/threatflux-unifi-sdk/issues)
- [Releases](https://github.com/ThreatFlux/threatflux-unifi-sdk/releases)
- [License](LICENSE)

## License

Licensed under the [MIT License](LICENSE).

Ubiquiti and UniFi are trademarks of their respective owner and are used here
only to describe interoperability.
