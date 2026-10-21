#!/usr/bin/env bash
# Ensure a non-TLS Docker failure cannot count as proof of CA rejection.
set -euo pipefail

image="${1:?Usage: test_image_fixture_errors.sh IMAGE [PLATFORM]}"
platform="${2:-}"
script_dir="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
temporary="$(mktemp -d "${TMPDIR:-/tmp}/unifi-image-errors.XXXXXXXX")"
trap 'rm -rf -- "$temporary"' EXIT
export FIXTURE_DOCKER_EXECUTABLE
FIXTURE_DOCKER_EXECUTABLE="$(command -v docker)"
mkdir "$temporary/bin"
cat > "$temporary/bin/docker" <<'DOCKER'
#!/usr/bin/env bash
set -euo pipefail
if [[ "${1:-}" == run && "$*" != *SSL_CERT_FILE=* ]]; then
  echo 'DNS lookup failed for host.docker.internal' >&2
  exit 1
fi
exec "$FIXTURE_DOCKER_EXECUTABLE" "$@"
DOCKER
chmod +x "$temporary/bin/docker"
if PATH="$temporary/bin:$PATH" bash "$script_dir/test_image_fixture.sh" \
  "$image" "$platform" > "$temporary/output" 2>&1; then
  echo 'Fixture incorrectly reported DNS failure as untrusted-CA rejection' >&2
  cat "$temporary/output" >&2
  exit 1
fi
grep -q 'Expected an untrusted-issuer TLS error' "$temporary/output"
grep -q 'DNS lookup failed for host.docker.internal' "$temporary/output"
echo 'Fixture correctly rejects non-TLS failures'
