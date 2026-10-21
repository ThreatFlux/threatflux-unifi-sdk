#!/usr/bin/env python3
"""Exercise the actual image's DNS, TLS verification and mock-controller login.

Requires Docker and OpenSSL. Uses generated test certificates and dummy credentials.
"""
import http.server
import json
import pathlib
import ssl
import subprocess
import sys
import tempfile
import threading

image = sys.argv[1]
platform = sys.argv[2] if len(sys.argv) > 2 else None
requests = []


class Controller(http.server.BaseHTTPRequestHandler):
    def reply(self, status, payload):
        body = json.dumps(payload).encode()
        self.send_response(status)
        self.send_header("Content-Type", "application/json")
        self.send_header("Content-Length", str(len(body)))
        self.send_header("X-CSRF-Token", "fixture-token")
        self.end_headers()
        self.wfile.write(body)

    def do_GET(self):
        requests.append(self.path)
        if self.path == "/api/auth/login":
            self.reply(405, {})
        else:
            self.reply(200, {"meta": {"rc": "ok"}, "data": []})

    def do_POST(self):
        requests.append(self.path)
        self.rfile.read(int(self.headers.get("Content-Length", "0")))
        self.reply(200, {})

    def log_message(self, *args):
        pass


with tempfile.TemporaryDirectory(prefix="unifi-image-fixture-") as temporary:
    root = pathlib.Path(temporary)
    key, cert = root / "server.key", root / "server.crt"
    ca_key, ca_cert = root / "ca.key", root / "ca.crt"
    subprocess.run([
        "openssl", "req", "-x509", "-newkey", "rsa:2048", "-nodes",
        "-keyout", str(ca_key), "-out", str(ca_cert), "-days", "1",
        "-subj", "/CN=Temporary UniFi Fixture CA",
        "-addext", "basicConstraints=critical,CA:TRUE",
    ], check=True, capture_output=True)
    subprocess.run([
        "openssl", "req", "-new", "-newkey", "rsa:2048", "-nodes",
        "-keyout", str(key), "-out", str(root / "server.csr"),
        "-subj", "/CN=host.docker.internal",
    ], check=True, capture_output=True)
    extensions = root / "extensions.cnf"
    extensions.write_text("subjectAltName=DNS:host.docker.internal\n"
                          "basicConstraints=critical,CA:FALSE\n"
                          "extendedKeyUsage=serverAuth\n")
    subprocess.run([
        "openssl", "x509", "-req", "-in", str(root / "server.csr"),
        "-CA", str(ca_cert), "-CAkey", str(ca_key), "-CAcreateserial",
        "-out", str(cert), "-days", "1", "-extfile", str(extensions),
    ], check=True, capture_output=True)
    ca_cert.chmod(0o644)
    cert.chmod(0o644)
    server = http.server.ThreadingHTTPServer(("0.0.0.0", 0), Controller)
    context = ssl.SSLContext(ssl.PROTOCOL_TLS_SERVER)
    context.load_cert_chain(str(cert), str(key))
    server.socket = context.wrap_socket(server.socket, server_side=True)
    thread = threading.Thread(target=server.serve_forever, daemon=True)
    thread.start()
    host = f"https://host.docker.internal:{server.server_port}"
    base = ["docker", "run", "--rm", "--read-only"]
    if platform:
        base += ["--platform", platform]
    if sys.platform != "darwin":
        base += ["--add-host", "host.docker.internal:host-gateway"]
    cli = ["--host", host, "--username", "fixture-user", "--password",
           "fixture-password", "--verify-ssl", "clients", "list"]
    try:
        untrusted = subprocess.run(base + [image] + cli, capture_output=True, text=True)
        assert untrusted.returncode != 0, "TLS unexpectedly trusted generated CA"
        trusted = subprocess.run(
            base + ["-v", f"{ca_cert}:/fixture/ca.crt:ro", "-e", "SSL_CERT_FILE=/fixture/ca.crt",
                    image] + cli, capture_output=True, text=True,
        )
        if trusted.returncode:
            raise AssertionError(trusted.stderr)
        assert json.loads(trusted.stdout) == [], trusted.stdout
        assert requests.count("/api/auth/login") >= 2
        print(json.dumps({"untrusted_ca_rejected": True, "trusted_ca_login_passed": True,
                          "controller_paths": requests}, indent=2))
    finally:
        server.shutdown()
        thread.join()
