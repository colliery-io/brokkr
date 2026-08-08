---
id: agent-ws-client-cannot-dial-wss
level: task
title: "Agent WS client cannot dial wss:// — tokio-tungstenite built without TLS feature"
short_code: "BROKKR-T-0325"
created_at: 2026-08-08T13:31:15.220414+00:00
updated_at: 2026-08-08T21:48:19.496004+00:00
parent: 
blocked_by: []
archived: false

tags:
  - "#task"
  - "#bug"
  - "#phase/completed"


exit_criteria_met: false
initiative_id: NULL
---

# Agent WS client cannot dial wss:// — tokio-tungstenite built without TLS feature

## Objective **[REQUIRED]**

Make the agent's broker WebSocket channel work across a TLS boundary. As shipped in 0.9.1, an agent pointed at an `https://` broker URL can never establish its WS connection, because TLS support was compiled out of the WebSocket client.

## Backlog Item Details

### Type
- [x] Bug - Production issue that needs fixing

### Priority
- [x] P1 - High (important for user experience)

### Impact Assessment
- **Affected Users**: Every deployment where agents reach the broker through a TLS endpoint (ingress/LB with an `https://` broker URL). In-cluster plain-HTTP deployments are unaffected.
- **Reproduction Steps**:
  1. Deploy the 0.9.1 broker behind a TLS-terminating endpoint.
  2. Configure an agent with `broker_url = "https://<host>"`.
  3. Agent registers and polls fine over HTTPS (reqwest has native-tls), but the WS channel never comes up.
- **Expected vs Actual**: Expected — `ws_url_from_broker_url` maps `https://` → `wss://` (`crates/brokkr-agent/src/broker_ws.rs:222`) and the agent connects. Actual — `tokio_tungstenite::connect_async` fails instantly with `UrlError::TlsFeatureNotEnabled` on every reconnect attempt; the handshake never reaches the network. `crates/brokkr-agent/Cargo.toml` declared `tokio-tungstenite = "0.24"` with no TLS feature (TLS is opt-in in tokio-tungstenite).

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria **[REQUIRED]**

- [x] Agent binary can complete a `wss://` handshake (TLS feature compiled in).
- [x] Plain `ws://` in-cluster behavior unchanged (`MaybeTlsStream` only engages TLS for `wss://`).
- [x] Workspace compiles and brokkr-agent unit tests pass (101/101).

## Implementation Notes

### Technical Approach
One-line dependency change: `tokio-tungstenite = { version = "0.24", features = ["native-tls"] }` in `crates/brokkr-agent/Cargo.toml`. Chose `native-tls` over rustls because the agent's reqwest 0.11 (default features) already links native-tls/OpenSSL — this adds no new TLS dependency tree and leaves image requirements unchanged. Switching both clients to rustls is a reasonable future cleanup that would drop the OpenSSL requirement from the agent image.

### Risk Considerations
- Unit tests cannot exercise a real TLS handshake, and e2e runs plain `ws://` in-cluster — definitive verification is an agent dialing an `https://` broker from the released image.
- This class of bug is invisible to PR CI (e2e only runs on release/nightly, and even there no TLS boundary exists). Related gap: [[four-ws-e2e-scenarios-never-run]] (BROKKR-T-0323).

## Status Updates **[REQUIRED]**

- 2026-08-08: Root-caused (missing tokio-tungstenite TLS feature), fix applied, `cargo check` clean, 101/101 agent unit tests pass. Shipping in the 0.9.2 patch release PR alongside the lockstep version bump.