#!/usr/bin/env python3
"""Serve and verify the mock controller used by test_image_fixture.sh."""
import http.server
import json
import pathlib
import ssl
import sys

root = pathlib.Path(sys.argv[2])


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
        with (root / "requests.jsonl").open("a", encoding="utf-8") as log:
            log.write(json.dumps(self.path) + "\n")
        if self.path == "/api/auth/login":
            self.reply(405, {})
        else:
            self.reply(200, {"meta": {"rc": "ok"}, "data": []})

    def do_POST(self):
        with (root / "requests.jsonl").open("a", encoding="utf-8") as log:
            log.write(json.dumps(self.path) + "\n")
        self.rfile.read(int(self.headers.get("Content-Length", "0")))
        self.reply(200, {})

    def log_message(self, *args):
        pass


if sys.argv[1] == "serve":
    # macOS uses loopback; Linux uses only the Docker bridge gateway interface.
    server = http.server.ThreadingHTTPServer((sys.argv[3], 0), Controller)
    context = ssl.SSLContext(ssl.PROTOCOL_TLS_SERVER)
    context.load_cert_chain(str(root / "server.crt"), str(root / "server.key"))
    server.socket = context.wrap_socket(server.socket, server_side=True)
    (root / "port").write_text(str(server.server_port), encoding="utf-8")
    server.serve_forever()
elif sys.argv[1] == "verify":
    output = (root / "trusted.stdout").read_text(encoding="utf-8")
    if json.loads(output) != []:
        raise AssertionError(output)
    requests = [json.loads(line) for line in
                (root / "requests.jsonl").read_text(encoding="utf-8").splitlines()]
    if requests.count("/api/auth/login") < 2 or "/proxy/network/api/s/default/rest/user" not in requests:
        raise AssertionError(requests)
    print(json.dumps({"untrusted_ca_rejected": True, "trusted_ca_login_passed": True,
                      "controller_paths": requests}, indent=2))
else:
    raise ValueError("Expected serve or verify")
