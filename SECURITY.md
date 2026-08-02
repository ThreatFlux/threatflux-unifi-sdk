# Security policy

## Supported versions

| Version | Supported |
| ------- | --------- |
| `0.5.x` | Yes |
| `< 0.5` | No |

## Reporting a vulnerability

Do not report security vulnerabilities through public GitHub issues.

Open a private report through the repository's
[GitHub Security Advisories](https://github.com/ThreatFlux/threatflux-unifi-sdk/security/advisories).

Include:

- The vulnerability type and impact
- Affected release, commit, and source paths
- Reproduction steps
- A minimal proof of concept when safe
- Any known mitigations

## Response targets

- Initial response: within 48 hours
- Status update: within 5 business days
- Resolution target: within 90 days

These are targets, not disclosure deadlines. Coordinate public disclosure with
the maintainers.

## Deployment security

`UnifiConfig` currently disables TLS certificate verification by default.
Production deployments should enable verification whenever a trusted
controller certificate or certificate authority can be configured. If
compatibility forces use of the insecure default, isolate the management
network and explicitly accept and mitigate the resulting man-in-the-middle
risk. See [configuration and security](docs/configuration.md) for the exact
behavior.

Do not log `UnifiConfig`: its derived `Debug` representation includes the
password. Use a least-privilege controller account, keep credentials in a
secret manager, and restrict management-interface access.

## Safe harbor

We consider security research conducted in good faith to be authorized. We will
not pursue legal action against researchers who:

- Make good-faith efforts to avoid privacy violations
- Avoid data destruction or service disruption
- Report vulnerabilities promptly
- Allow reasonable remediation time before disclosure
