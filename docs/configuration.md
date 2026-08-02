# Configuration, authentication, and security

The SDK has three distinct configuration paths:

1. Applications construct `UnifiConfig` directly.
2. `load_config` parses the declarative YAML/JSON schema.
3. The `unifi-cli` binary maps flags and CLI-specific environment variables
   into one of those paths.

Keeping these boundaries explicit prevents applications from depending on
environment discovery that the library does not implement.

## Programmatic SDK configuration

`UnifiConfig::new(host, username, password)` sets these defaults:

| Field | Default | Behavior |
| ----- | ------- | -------- |
| `host` | Required | Used as given when it starts with `http://` or `https://`; otherwise prefixed with `https://` |
| `username` | Required | Included in the login JSON payload |
| `password` | Required | Included in the login JSON payload |
| `site` | `default` | Used in controller API paths |
| `verify_ssl` | `false` | Disables certificate validation through reqwest |
| `timeout_secs` | `30` | Configures the reqwest request timeout |

The library does not read `UNIFI_*` variables. The primary README example
uses `std::env` as application code; a secret manager or typed application
configuration can provide the same values.

`UnifiConfig` derives `Debug`, and its password is a public `String`.
Do not log the configuration and do not retain it longer than needed.

## TLS in production

The current `verify_ssl = false` default is a legacy compatibility choice for
self-signed controller certificates. It disables certificate validation and
can allow a man-in-the-middle attacker to capture controller credentials,
cookies, and mutations.

Use a trusted HTTPS endpoint in production:

```rust
use threatflux_unifi_sdk::UnifiConfig;

let config = UnifiConfig::new("https://unifi.example.net", "automation", "secret")
    .with_verify_ssl(true);
```

Provision a certificate trusted by the client runtime or add the private
issuing CA to the runtime trust store. Do not work around a hostname, expiry,
or trust failure by disabling verification on an untrusted network. Avoid an
explicit `http://` host because credentials and session data would be sent
without transport encryption.

A future secure-default migration would change connection behavior for current
users and therefore needs a separately planned compatibility strategy. The
present code still defaults to false; callers must opt in to verification.

## Authentication and controller detection

`UnifiClient::connect`:

1. Parses the controller base URL.
2. sends GET `/api/auth/login`.
3. Treats HTTP 405 as UniFi OS and every other response as classic.
4. Logs in at `/api/auth/login` for UniFi OS or `/api/login` for classic.
5. Accepts HTTP 200 or 204 as login success.
6. Stores response cookies and CSRF tokens in memory.

The detection rule is a heuristic. A proxy, authentication gateway, transient
error, or changed controller response can cause classic routing to be selected.
There is no public controller-type override.

The only implemented credential flow is username/password JSON login. Cloud
OAuth, MFA prompts, API keys, and controller API tokens are not implemented.

### Expired sessions

Typed GET/POST/PUT and raw GET paths treat 401 as
`UnifiError::SessionExpired`, set the session state to unauthenticated, and
return without replaying the request. If application code retries, the next
request first calls `login`. DELETE has separate status handling and currently
returns a generic `ApiError` for 401 without clearing session state.

For reads, a bounded caller-owned retry after `SessionExpired` may be
reasonable. For creates, updates, commands, and deletes, determine whether the
controller applied the operation before retrying.

## Timeouts, rate limits, and retries

- `timeout_secs` configures reqwest's request timeout. The SDK does not
  validate a minimum or maximum.
- The typed response handler maps 429 to `UnifiError::RateLimited` and parses
  an optional integer `Retry-After` value.
- `UnifiError::is_retryable()` classifies selected variants; it does not
  sleep, retry, or add jitter.
- There is no SDK-level retry count, exponential backoff, circuit breaker, or
  idempotency key.
- There is no public custom reqwest-client or proxy configuration hook.

Build bounded retries around application semantics. Respect `Retry-After`,
cap total elapsed time, add jitter, and avoid replaying non-idempotent writes
without reconciliation.

## Declarative files and environment substitution

`load_config(path)` chooses JSON for a `.json` extension and YAML otherwise.
It resolves `${NAME}` in every string value after parsing. An unset variable
is preserved literally.

The declarative `unifi.verify_ssl` field also defaults to false when omitted.
Set it explicitly:

```yaml
unifi:
  host: "${UNIFI_HOST}"
  username: "${UNIFI_USERNAME}"
  password: "${UNIFI_PASSWORD}"
  site: default
  verify_ssl: true
  timeout_secs: 30
```

Unknown substitutions do not fail fast. Validate required variables before
calling `load_config` or inspect the parsed configuration before connecting.
Keep secret-bearing configuration files out of source control.

## CLI environment variables

These variables are implemented by clap in the `unifi-cli` binary. They are
not automatically consumed by `UnifiConfig`.

| CLI variable | Flag | Default |
| ------------ | ---- | ------- |
| `UNIFI_HOST` | `--host` | Required for connection-based commands |
| `UNIFI_USERNAME` | `--username` | Required for connection-based commands |
| `UNIFI_PASSWORD` | `--password` | Required for connection-based commands |
| `UNIFI_SITE` | `--site` | `default` |
| `UNIFI_VERIFY_SSL` | `--verify-ssl` | `false` |
| `UNIFI_TIMEOUT` | `--timeout-secs` | `30` |

`UNIFI_TIMEOUT_SECS` is used only by the credentialed integration-test
harness; it is not the CLI timeout variable.

Prefer environment injection from a secret manager over command-line password
flags, which can be exposed through process listings and shell history. Set
`UNIFI_VERIFY_SSL=true` for production CLI use.

## Declarative apply safety

- Start with `unifi-cli diff --config PATH`.
- Run `unifi-cli sync --config PATH --dry-run` before a real apply.
- Leave `--prune` off until deletion scope has been reviewed.
- Use a dedicated automation account with the minimum controller permissions
  that satisfy the desired operations.
- Test controller upgrades and SDK upgrades against a non-production site.

Rollback data is accumulated as changes are applied, but rollback is
best-effort and non-transactional. A controller or network failure can leave a
partially applied or partially rolled-back configuration.

See the [CLI guide](cli.md) and
[API coverage](api-coverage.md) for the supported resource groups.
