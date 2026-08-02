# `unifi-cli` guide

The crate packages a binary named `unifi-cli`. The CLI is a consumer of the
SDK; its environment-variable behavior is not implicit library behavior.

## Installation

```bash
cargo install --locked threatflux-unifi-sdk
unifi-cli --help
```

To run the checkout instead:

```bash
cargo run --locked --bin unifi-cli -- --help
```

## Secure connection setup

Connection-based commands accept `--host`, `--username`, `--password`,
`--site`, `--verify-ssl`, and `--timeout-secs`. The equivalent environment
variables are:

```bash
export UNIFI_HOST=unifi.example.net
export UNIFI_USERNAME=automation
export UNIFI_PASSWORD='retrieve-this-from-a-secret-manager'
export UNIFI_SITE=default
export UNIFI_VERIFY_SSL=true
export UNIFI_TIMEOUT=30
```

TLS verification currently defaults to false. Always set
`UNIFI_VERIFY_SSL=true` in production after configuring a trusted controller
certificate. Prefer an environment/secret injection mechanism over
`--password`, which can be retained in shell history or process metadata.

## Commands

| Command | Purpose |
| ------- | ------- |
| `status` | Show controller health and system information |
| `export [--format FORMAT]` | Export the sync engine's declarative resource groups as YAML or JSON |
| `clients list` / `clients active` | List known or online clients |
| `clients block MAC` / `clients unblock MAC` | Change client blocking state |
| `devices list` | List managed devices |
| `devices restart MAC` | Restart a device |
| `devices upgrade-all` | Request upgrades for every device reported as upgradable |
| `vpn status` | Show VPN status |
| `vpn wireguard list-clients` | List WireGuard peers |
| `vpn wireguard add-client` | Add a WireGuard peer |
| `firewall list-rules` | List firewall rules, optionally filtered by ruleset |
| `firewall add-rule` | Add a basic firewall rule |
| `port-forward list` | List port-forward rules |
| `port-forward add` | Add a port-forward rule |
| `traffic stats HOURS` | Query traffic statistics for the requested period |

Use `unifi-cli COMMAND --help` for exact arguments and accepted enum values.
Commands that mutate controllers are not automatically retried.

## Declarative diff and sync

`diff` and `sync` get their connection settings from the `unifi` section
of the supplied YAML/JSON file. They do not use the CLI's global connection
arguments.

```bash
export UNIFI_HOST=unifi.example.net
export UNIFI_USERNAME=automation
export UNIFI_PASSWORD='retrieve-this-from-a-secret-manager'

unifi-cli diff --config config/unifi.example.yaml
unifi-cli sync --config config/unifi.example.yaml --dry-run
```

After reviewing the plan, omit `--dry-run` to apply it:

```bash
unifi-cli sync --config config/unifi.example.yaml
```

`--prune` allows deletion of supported resource types that are present on the
controller but absent from the desired configuration. It is disabled by
default. Test prune behavior on a non-production site and keep a controller
backup.

The declarative loader substitutes `${NAME}` inside string values. Missing
variables remain literal, so validate the required environment before running
a command.

## Operational notes

- `devices upgrade-all`, firewall changes, VPN changes, and sync/prune can
  disrupt network service.
- A command that receives an uncertain response may have been applied by the
  controller; reconcile state before rerunning it.
- Export covers the sync engine's modeled resource groups, not every controller
  setting and not a controller backup archive.
- Controller endpoints can change across UniFi Network and firmware versions.

For all configuration fields and session behavior, see
[configuration and security](configuration.md). For service and integration
test coverage, see [API coverage](api-coverage.md).
