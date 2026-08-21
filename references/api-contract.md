# Synapt — SynaptClip API Contract

---

## Overview

Synapt exposes a local HTTP API when running. SynaptClip consumes this API to enable cross-device clipboard features. Neither app hard-depends on the other. SynaptClip uses this API at runtime only, with graceful fallback when Synapt is not present.

This document is the authoritative reference for SynaptClip developers. The Synapt team owns and implements these endpoints. The SynaptClip team codes against this contract.

---

## Conventions

- Base URL: `http://127.0.0.1:57321`
- All request and response bodies are JSON
- All timestamps are ISO 8601 UTC strings
- API version is included in every response as `"api_version": "1"`
- Endpoints are versioned under `/v1/`
- The server is only available when Synapt is running. SynaptClip must handle connection errors gracefully.

---

## Port

Synapt listens on `127.0.0.1:57321` (loopback only, never exposed to the network).

The port `57321` is fixed. If it is in use, Synapt should log an error and disable the API — not pick a random port. SynaptClip always connects to `57321` and does not need to discover the port.

---

## Authentication

No authentication is required. The API is loopback-only and accessible only to processes on the same machine. If the user is running Synapt, they are authorized to call the API.

---

## Versioning

The API follows a simple integer version. This document describes v1. Breaking changes increment the version. SynaptClip should check the `api_version` field on `/v1/health` before using other endpoints and display a warning if the version is higher than expected.

---

## Endpoints

---

### GET /v1/health

Check whether Synapt is running and retrieve API metadata.

SynaptClip polls this endpoint on startup and periodically (every 10 seconds) to detect whether Synapt is running. If this request fails or times out, Synapt is considered offline and integration features are disabled.

#### Request

No body.

#### Response

```json
{
  "api_version": "1",
  "synapt_version": "0.5.0",
  "status": "ok"
}
```

#### Status codes

| Code | Meaning |
|---|---|
| 200 | Synapt is running |
| Any error or timeout | Synapt is not running or API is disabled |

---

### GET /v1/peers

Return the list of currently discovered peers on the LAN.

SynaptClip uses this to populate the "Send to device" list in the panel.

#### Request

No body.

#### Response

```json
{
  "api_version": "1",
  "peers": [
    {
      "id": "peer-uuid-string",
      "name": "aatish-laptop",
      "ip": "192.168.1.42",
      "port": 54321,
      "online": true,
      "last_seen": "2025-03-14T10:22:00Z"
    }
  ]
}
```

#### Peer object fields

| Field | Type | Description |
|---|---|---|
| id | string | Stable UUID for this peer, persisted across sessions |
| name | string | Hostname or user-configured display name |
| ip | string | IPv4 address |
| port | integer | Synapt transfer port on the peer device |
| online | boolean | Whether the peer was seen in the last 30 seconds |
| last_seen | string | ISO 8601 UTC timestamp of last discovery response |

#### Status codes

| Code | Meaning |
|---|---|
| 200 | Success, peers list returned (may be empty) |
| 503 | Synapt discovery service is not running |

---

### POST /v1/clips/send

Send text content to a peer device via Synapt's P2P transfer layer. Synapt wraps the content as a temporary file and sends it using its existing transfer mechanism.

SynaptClip calls this when the user selects "Send to device" from a clip's context menu.

#### Request

```json
{
  "peer_id": "peer-uuid-string",
  "content": "the text content of the clipboard entry",
  "content_type": "text"
}
```

#### Request fields

| Field | Type | Required | Description |
|---|---|---|---|
| peer_id | string | Yes | The `id` field from the peers list |
| content | string | Yes | The clipboard content to send |
| content_type | string | Yes | Always `"text"` in v1 |

#### Response

```json
{
  "api_version": "1",
  "transfer_id": "transfer-uuid-string",
  "status": "queued"
}
```

The transfer is asynchronous. `status: "queued"` means Synapt has accepted the request and will attempt the transfer. SynaptClip does not need to poll for completion in v1 — it is fire-and-forget from SynaptClip's perspective.

#### Status codes

| Code | Meaning |
|---|---|
| 202 | Transfer queued |
| 404 | Peer not found or no longer online |
| 422 | Invalid request body |
| 503 | Transfer service not available |

---

### POST /v1/clips/receive (Webhook — Synapt calls SynaptClip)

This endpoint is implemented by SynaptClip, not Synapt. When Synapt receives content sent from a remote peer's SynaptClip instance, it forwards the content to the local SynaptClip by POSTing to this endpoint.

SynaptClip listens on `http://127.0.0.1:57322` for incoming content from Synapt.

#### SynaptClip's listener port: `57322`

Synapt always posts to `127.0.0.1:57322` when delivering received content.

#### Request (Synapt sends this to SynaptClip)

```json
{
  "sender_peer_id": "peer-uuid-string",
  "sender_name": "aatish-laptop",
  "content": "the text content received from the remote device",
  "content_type": "text",
  "received_at": "2025-03-14T10:22:00Z"
}
```

#### Response (SynaptClip returns this to Synapt)

```json
{
  "status": "accepted"
}
```

#### Status codes (SynaptClip returns)

| Code | Meaning |
|---|---|
| 200 | SynaptClip accepted and stored the clip |
| 503 | SynaptClip is running but cannot accept clips at this time |
| Any error or timeout | SynaptClip is not running, Synapt should discard |

---

## Port Summary

| App | Port | Direction | Purpose |
|---|---|---|---|
| Synapt | 57321 | SynaptClip calls Synapt | Main integration API |
| SynaptClip | 57322 | Synapt calls SynaptClip | Incoming clip delivery |

Both ports are loopback only (`127.0.0.1`). Neither is exposed to the LAN.

---

## Error Response Format

All error responses use this shape:

```json
{
  "api_version": "1",
  "error": "peer_not_found",
  "message": "No peer with the given ID was found in the current session."
}
```

| Field | Description |
|---|---|
| error | Machine-readable error code, snake_case |
| message | Human-readable description for logging |

---

## SynaptClip Integration Checklist

For SynaptClip developers implementing the Synapt bridge (`src-tauri/src/synapt/bridge.rs`):

**Startup**
- Poll `GET /v1/health` on app start with a 2-second timeout
- If successful, mark bridge as active and enable integration UI
- If failed, mark bridge as inactive, hide integration UI, no error shown to user

**Periodic health check**
- Poll `GET /v1/health` every 10 seconds
- If status changes from active to inactive, hide integration UI gracefully
- If status changes from inactive to active, enable integration UI without restart

**Fetching peers**
- Call `GET /v1/peers` when the panel opens and bridge is active
- Cache the result for the duration of the panel session
- Do not poll continuously — fetch on demand

**Sending a clip**
- Call `POST /v1/clips/send` with the peer ID and clip content
- Show a brief "Sent" confirmation in the UI on 202 response
- On 404, refresh peer list and show "Device no longer available"
- On other errors, show "Transfer failed" without crashing

**Receiving clips**
- Start the listener on `127.0.0.1:57322` when the app launches
- Always start the listener regardless of whether Synapt is detected — Synapt may start later
- On receiving a clip, store it in the local SQLite database as a normal clip with `source_app = "synapt"` and the sender name noted
- Optionally show a system notification: "Clip received from [sender name]"

---

## Mocking the API for Development

SynaptClip developers can develop and test the bridge without Synapt running by using a mock server.

A minimal mock can be run with any HTTP server. Example using Python:

```python
# mock_synapt.py
from http.server import HTTPServer, BaseHTTPRequestHandler
import json

class Handler(BaseHTTPRequestHandler):
    def do_GET(self):
        if self.path == "/v1/health":
            self.send_response(200)
            self.send_header("Content-Type", "application/json")
            self.end_headers()
            self.wfile.write(json.dumps({
                "api_version": "1",
                "synapt_version": "0.5.0",
                "status": "ok"
            }).encode())

        elif self.path == "/v1/peers":
            self.send_response(200)
            self.send_header("Content-Type", "application/json")
            self.end_headers()
            self.wfile.write(json.dumps({
                "api_version": "1",
                "peers": [
                    {
                        "id": "mock-peer-001",
                        "name": "mock-device",
                        "ip": "192.168.1.99",
                        "port": 54321,
                        "online": True,
                        "last_seen": "2025-03-14T10:00:00Z"
                    }
                ]
            }).encode())

    def do_POST(self):
        if self.path == "/v1/clips/send":
            length = int(self.headers["Content-Length"])
            body = json.loads(self.rfile.read(length))
            print(f"[mock] send clip to {body['peer_id']}: {body['content'][:60]}")
            self.send_response(202)
            self.send_header("Content-Type", "application/json")
            self.end_headers()
            self.wfile.write(json.dumps({
                "api_version": "1",
                "transfer_id": "mock-transfer-001",
                "status": "queued"
            }).encode())

    def log_message(self, format, *args):
        pass  # suppress default logging

HTTPServer(("127.0.0.1", 57321), Handler).serve_forever()
```

Run with `python mock_synapt.py` before launching SynaptClip during development.

---

## Future Versions (Post v1)

These are not part of the v1 contract. They are listed here so both teams are aware of planned direction.

| Feature | Notes |
|---|---|
| Image clip transfer | Requires binary transfer, not just JSON text |
| Transfer status polling | `GET /v1/transfers/{id}` for progress |
| Peer authentication | Shared secret or pairing flow |
| Multiple clip batch send | Send several clips at once |
| Clipboard sync mode | Automatic sync without manual send |