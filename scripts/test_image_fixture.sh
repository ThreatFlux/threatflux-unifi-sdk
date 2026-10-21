#!/usr/bin/env bash
# Exercise actual-image DNS, TLS verification and mock-controller login.
# Requires Docker, OpenSSL and Python 3; uses only generated test credentials.
set -euo pipefail

image="${1:?Usage: test_image_fixture.sh IMAGE [PLATFORM]}"
platform="${2:-}"
script_dir="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
root="$(mktemp -d "${TMPDIR:-/tmp}/unifi-image-fixture.XXXXXXXX")"
server_pid=""
# Stop the fixture and remove temporary certificates and captured output.
cleanup() {
  if [[ -n "$server_pid" ]]; then
    kill "$server_pid" 2>/dev/null || true
    wait "$server_pid" 2>/dev/null || true
  fi
  rm -rf -- "$root"
}
trap cleanup EXIT

openssl req -x509 -newkey rsa:2048 -nodes \
  -keyout "$root/ca.key" -out "$root/ca.crt" -days 1 \
  -subj '/CN=Temporary UniFi Fixture CA' \
  -addext 'basicConstraints=critical,CA:TRUE' > "$root/openssl.log" 2>&1
openssl req -new -newkey rsa:2048 -nodes \
  -keyout "$root/server.key" -out "$root/server.csr" \
  -subj '/CN=host.docker.internal' >> "$root/openssl.log" 2>&1
printf '%s\n' 'subjectAltName=DNS:host.docker.internal' \
  'basicConstraints=critical,CA:FALSE' 'extendedKeyUsage=serverAuth' > "$root/extensions.cnf"
openssl x509 -req -in "$root/server.csr" \
  -CA "$root/ca.crt" -CAkey "$root/ca.key" -CAcreateserial \
  -out "$root/server.crt" -days 1 -extfile "$root/extensions.cnf" >> "$root/openssl.log" 2>&1
chmod 644 "$root/ca.crt" "$root/server.crt"

docker_args=(run --rm --read-only)
if [[ -n "$platform" ]]; then
  docker_args+=(--platform "$platform")
fi
bind_address=127.0.0.1
if [[ "$(uname -s)" != Darwin ]]; then
  # Bind only the default bridge interface that the test container can reach.
  bind_address="$(docker network inspect bridge --format '{{(index .IPAM.Config 0).Gateway}}')"
  docker_args+=(--add-host "host.docker.internal:$bind_address")
fi
python3 "$script_dir/test_image_fixture.py" serve "$root" "$bind_address" > "$root/server.log" 2>&1 &
server_pid=$!
for ((attempt = 0; attempt < 100; attempt++)); do
  if [[ -s "$root/port" ]]; then
    break
  fi
  if ! kill -0 "$server_pid" 2>/dev/null; then
    cat "$root/server.log" >&2
    exit 1
  fi
  sleep 0.1
done
if [[ ! -s "$root/port" ]]; then
  echo 'Fixture did not start within 10 seconds' >&2
  exit 1
fi
cli=(--host "https://host.docker.internal:$(cat "$root/port")"
  --username fixture-user --password fixture-password --verify-ssl clients list)
if docker "${docker_args[@]}" -- "$image" "${cli[@]}" > "$root/untrusted.stdout" 2> "$root/untrusted.stderr"; then
  echo 'TLS unexpectedly trusted generated CA' >&2
  exit 1
fi
# The SDK currently flattens connection errors, so CLI stderr omits the TLS
# cause. Require the controller to observe the client's unknown-CA TLS alert.
if ! python3 "$script_dir/test_image_fixture.py" verify-untrusted "$root"; then
  cat "$root/untrusted.stderr" >&2
  exit 1
fi
if ! docker "${docker_args[@]}" \
  -v "$root/ca.crt:/fixture/ca.crt:ro" -e SSL_CERT_FILE=/fixture/ca.crt \
  -- "$image" "${cli[@]}" > "$root/trusted.stdout" 2> "$root/trusted.stderr"; then
  cat "$root/trusted.stderr" >&2
  exit 1
fi
python3 "$script_dir/test_image_fixture.py" verify "$root"
