#!/usr/bin/env python3
"""
webhook-catcher: Simple HTTP server that receives webhook payloads
and stores them in memory for later retrieval. Used for UAT demo testing.

Copyright (c) 2025 Dylan Storey
Licensed under the Elastic License 2.0.
"""

import json
import os
import threading
from datetime import datetime, timezone
from http.server import HTTPServer, BaseHTTPRequestHandler
from typing import Any

# In-memory message store
messages: list[dict[str, Any]] = []
counter = 0
lock = threading.Lock()


class WebhookHandler(BaseHTTPRequestHandler):
    def log_message(self, format: str, *args) -> None:
        print(f"[{datetime.now().isoformat()}] {format % args}")

    def send_cors_headers(self) -> None:
        """Add CORS headers for browser access."""
        self.send_header("Access-Control-Allow-Origin", "*")
        self.send_header("Access-Control-Allow-Methods", "GET, POST, DELETE, OPTIONS")
        self.send_header("Access-Control-Allow-Headers", "Content-Type, Authorization")

    def send_json(self, status: int, data: dict) -> None:
        body = json.dumps(data).encode()
        self.send_response(status)
        self.send_header("Content-Type", "application/json")
        self.send_header("Content-Length", str(len(body)))
        self.send_cors_headers()
        self.end_headers()
        self.wfile.write(body)

    def do_OPTIONS(self) -> None:
        """Handle CORS preflight requests."""
        self.send_response(200)
        self.send_cors_headers()
        self.end_headers()

    def do_GET(self) -> None:
        if self.path == "/healthz":
            self.send_json(200, {"status": "ok"})

        elif self.path == "/readyz":
            self.send_json(200, {"status": "ready"})

        elif self.path == "/messages":
            with lock:
                self.send_json(200, {"count": len(messages), "messages": messages.copy()})

        elif self.path == "/stats":
            with lock:
                self.send_json(200, {"total_received": len(messages)})

        elif self.path == "/":
            self.send_json(200, {
                "name": "webhook-catcher",
                "version": "1.0.0",
                "endpoints": {
                    "POST /webhook": "Receive webhook payload",
                    "POST /receive": "Receive webhook payload (alias)",
                    "GET /messages": "List all received messages",
                    "DELETE /messages": "Clear all messages",
                    "GET /stats": "Get message count",
                    "GET /healthz": "Health check",
                    "GET /readyz": "Readiness check",
                },
            })
        else:
            self.send_json(404, {"error": "Not found"})

    def do_POST(self) -> None:
        if self.path in ("/webhook", "/receive"):
            global counter
            content_length = int(self.headers.get("Content-Length", 0))
            body = self.rfile.read(content_length) if content_length > 0 else b""

            try:
                payload = json.loads(body) if body else {}
            except json.JSONDecodeError:
                payload = {"raw": body.decode("utf-8", errors="replace")}

            with lock:
                counter += 1
                msg = {
                    "id": counter,
                    "received_at": datetime.now(timezone.utc).isoformat(),
                    "path": self.path,
                    "headers": dict(self.headers),
                    "body": payload,
                }
                messages.append(msg)
                # Keep only last 1000 messages
                if len(messages) > 1000:
                    messages.pop(0)

            print(f"Received webhook #{msg['id']} on {self.path}: {json.dumps(payload)[:200]}")
            self.send_json(200, {"received": True, "id": msg["id"]})
        else:
            self.send_json(404, {"error": "Not found"})

    def do_DELETE(self) -> None:
        if self.path == "/messages":
            with lock:
                count = len(messages)
                messages.clear()
            self.send_json(200, {"cleared": count})
        else:
            self.send_json(404, {"error": "Not found"})


def main() -> None:
    port = int(os.environ.get("PORT", "8080"))
    server = HTTPServer(("", port), WebhookHandler)
    print(f"webhook-catcher starting on :{port}")
    server.serve_forever()


if __name__ == "__main__":
    main()
