# API coverage and compatibility

This document describes the public surface implemented by this repository. It
does not promise that every operation is available on every UniFi controller,
UniFi Network application version, firmware release, site, or account.

The SDK is unofficial and relies in part on observed or reverse-engineered
controller APIs. Ubiquiti can change those APIs independently of this crate.

## Core client

| Area | Implemented behavior | Boundary |
| ---- | -------------------- | -------- |
| Connection | HTTPS is assumed when the host has no scheme; a reqwest request timeout is configured | The SDK does not accept a caller-built reqwest client or expose proxy configuration |
| Authentication | Username/password login against UniFi OS or classic endpoints | No OAuth, API-token, cloud-account, or MFA flow is implemented |
| Controller routing | A GET to `/api/auth/login` returning 405 selects UniFi OS; any other response selects classic routing | This is a heuristic, not controller/version negotiation |
| Session data | Set-Cookie values and CSRF tokens are collected and sent on later calls | There is no persistent cookie jar across client instances |
| Low-level requests | Typed GET, query GET, POST, PUT, DELETE, and command helpers; raw GET helpers | Endpoint paths and payloads remain controller-version-sensitive |
| Errors | `UnifiError`, `Result<T>`, API envelope parsing, rate-limit metadata, and retry/auth classifiers | The classifiers do not execute retries |

### Session expiration

Typed GET/POST/PUT responses (including command requests) and raw GET responses
mark the in-memory session unauthenticated when they receive 401 and return
`UnifiError::SessionExpired`. The failed request is not replayed. If the
caller retries, `ensure_authenticated` logs in before that next request.

DELETE uses a separate response path: a 401 is currently returned as
`UnifiError::ApiError` and does not clear the session flag. Callers must not
assume uniform automatic recovery across request methods.

The SDK has no retry or backoff loop. Blindly replaying a mutation can duplicate
an operation when the controller applied it before the response was lost.

## Typed services

All service modules are public and compile regardless of Cargo service-name
features.

| Service | Source-derived capability groups |
| ------- | -------------------------------- |
| `NetworkService` | List/lookup, CRUD, corporate/guest/VLAN creation, enablement, DHCP, DNS, and custom fields |
| `FirewallService` | Rule and group CRUD, ruleset filtering, ordering, enablement, membership, and helper rules |
| `PortForwardService` | CRUD, enablement, TCP/UDP helpers, restrictions, destinations, and WAN ports |
| `ClientService` | Client lookup/filtering, block/unblock, kick, guest authorization, names/notes, fixed IPs, groups, and stats |
| `DeviceService` | Device lookup/filtering, adoption/forget, restart/provision, upgrades, naming, disablement, LEDs, ports, and PoE |
| `DhcpService` | Reservation CRUD/helpers and lease lookup |
| `DnsService` | Record CRUD/helpers, settings, upstream servers, and filtering |
| `TrafficService` | Rule CRUD, enablement, domain/IP blocks, client rate limits, traffic stats, and DPI stats |
| `VpnService` | WireGuard server/peer CRUD, site-to-site VPN CRUD, status, and generated peer configs |
| `RoutingService` | Static-route CRUD/helpers, route-table lookup, and route searches |
| `BackupService` | Backup lifecycle, download/restore, latest backup, settings, config export, and pruning |
| `SiteService` | Site CRUD/lookup, health, stats, system information, events, alarms, DPI, and provisioning |

The public model types are re-exported from the crate root. Some service methods
return `serde_json::Value` where the controller response does not have a
stable typed model.

## Declarative configuration and sync

`load_config` accepts YAML or JSON and substitutes `${NAME}` in string
values. The sync engine supports these resource kinds:

- Networks
- Firewall groups and rules
- Port forwards
- Traffic rules
- WireGuard servers and peers
- Site-to-site VPNs
- DHCP reservations
- DNS records
- Client groups and blocked clients

`sync::diff` performs controller reads and computes changes without writes.
`sync::apply` can create, update, block/unblock, and optionally prune
resources. A `SyncReport` records inverse actions and exposes `rollback`,
but apply and rollback are not transactions: either can partially complete.
Always inspect a diff and test against a non-production site.

`sync::export_config` exports the resource groups implemented by the sync
engine. It is not a byte-for-byte backup of every controller setting; use
`BackupService` for the controller backup endpoints.

## What the tests establish

The default test suite covers model construction/serialization, error helpers,
configuration defaults, URL parsing, declarative conversion, and sync helper
logic. It does not mock the controller login heuristic or every service
endpoint.

The `integration-tests` Cargo feature enables three ignored tests requiring a
real controller:

1. A smoke test for system information, site lookup, and controller detection.
2. A network VLAN create/read/delete lifecycle.
3. A firewall address-group create/read/delete lifecycle.

Those tests validate the configured controller only. The repository does not
currently publish a tested matrix for UDM models, Cloud Keys, self-hosted
controllers, UniFi Network versions, or firmware versions.

## Explicit non-guarantees

- This is not an official Ubiquiti SDK or API compatibility contract.
- Implemented operations are not proof of availability on a particular
  controller or account.
- There is no SDK-level automatic request replay, exponential backoff, or
  idempotency-key support.
- There is no compile-time service reduction from the service-name Cargo
  features.
- There is no API schema discovery or version negotiation.

For runtime behavior and production safeguards, see
[configuration and security](configuration.md).
