# Pull request

## Summary

- What changed?
- Why is it needed?

## Compatibility and security

- Controller types/versions exercised:
- Public API or CLI compatibility impact:
- TLS, credential, mutation, or retry impact:

## Validation

- [ ] `make ci`
- [ ] `make docs-check`
- [ ] Credentialed controller tests, if applicable
- [ ] Additional checks documented below

## Checklist

- [ ] Public behavior and controller assumptions are documented
- [ ] Examples compile and use secure TLS guidance
- [ ] No credentials, session data, or private controller details are included
- [ ] Compatibility-sensitive changes have a migration plan
- [ ] Destructive controller operations were tested on a non-production site
