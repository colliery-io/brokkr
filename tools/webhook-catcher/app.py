#!/usr/bin/env python3
"""
Simple webhook catcher for demo/testing purposes.
Stores all received webhooks in memory and exposes them via /stats.
"""

from flask import Flask, request, jsonify
from flask_cors import CORS
from datetime import datetime
import json

app = Flask(__name__)
CORS(app)  # Allow cross-origin requests from UI

# In-memory storage for received webhooks
received_messages = []
MAX_MESSAGES = 100


@app.route('/receive', methods=['POST'])
def receive():
    """Receive and store a webhook payload."""
    try:
        payload = request.get_json(force=True, silent=True) or {}
    except:
        payload = {"raw": request.data.decode('utf-8', errors='replace')}

    message = {
        "id": len(received_messages) + 1,
        "received_at": datetime.utcnow().isoformat() + "Z",
        "event_type": payload.get("event_type", "unknown"),
        "payload": payload
    }

    received_messages.insert(0, message)

    # Keep only recent messages
    if len(received_messages) > MAX_MESSAGES:
        received_messages.pop()

    print(f"[WEBHOOK] Received: {message['event_type']}")

    return jsonify({"status": "received", "id": message["id"]})


@app.route('/stats', methods=['GET'])
def stats():
    """Return all received messages."""
    return jsonify({
        "count": len(received_messages),
        "messages": received_messages
    })


@app.route('/clear', methods=['POST'])
def clear():
    """Clear all stored messages."""
    received_messages.clear()
    print("[WEBHOOK] Cleared all messages")
    return jsonify({"status": "cleared"})


@app.route('/health', methods=['GET'])
def health():
    """Health check endpoint."""
    return jsonify({"status": "ok"})


if __name__ == '__main__':
    print("Webhook Catcher starting on port 8080...")
    app.run(host='0.0.0.0', port=8080)
