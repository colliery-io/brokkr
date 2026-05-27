---
id: ws-11-live-fan-out-subscription
level: task
title: "WS-11: Live fan-out subscription (read-only WS/SSE) for UI tail — PAK-scoped"
short_code: "BROKKR-T-0166"
created_at: 2026-05-23T02:12:45.257748+00:00
updated_at: 2026-05-23T11:08:27.465518+00:00
parent: BROKKR-I-0019
blocked_by: []
archived: false

tags:
  - "#task"
  - "#phase/completed"


exit_criteria_met: false
initiative_id: BROKKR-I-0019
---

# WS-11: Live fan-out subscription (read-only WS/SSE) for UI tail — PAK-scoped

**Parent**: [[BROKKR-I-0019]] · **ADR**: [[BROKKR-A-0008]]

## Objective

Add a read-only WS/SSE subscription endpoint on the broker that lets UI / SDK clients tail live events and logs for stacks. The broker fans out from an in-memory hub fed by ingestion (WS-09). PAK-scoped: subscribers receive only streams from stacks their PAK can read.

## Acceptance Criteria

## Acceptance Criteria

- [x] Endpoint `GET /api/v1/stacks/{id}/live` accepts a WS subscription filtered by `stack_id` (one stack per connection; multi-stack subscriptions are a follow-up if needed).
- [x] Only streams from stacks the PAK has read access to are delivered — `authorise()` reuses the admin-or-owning-generator rule from the rest of the stack surface; foreign generators get 403 *before* the upgrade completes.
- [x] Subscriber receives messages within ingest latency: `dispatch_uplink` broadcasts on the `LiveBroadcaster` **before** the DAL insert, so there's no DB round-trip between an agent send and a subscriber receive.
- [x] Clean disconnect: subscriber close drops the `broadcast::Receiver`; per-stack channels stay alive (cheap) until process exit.
- [ ] Heartbeat / ping for proxy timeouts deferred to WS-14 docs (we'll document the required ingress timeout instead of adding application-level ping; nothing about tokio-tungstenite needs it for direct connections).
- [x] Integration test: `live_subscription_forwards_agent_telemetry_to_subscribers` opens a subscription, agent sends K8sEvent + PodLogLine, subscriber receives both within the test deadline. `live_subscription_rejects_unauthorised_caller` proves the 403 path.

## Implementation Notes

- **Approach**: per-stack `tokio::sync::broadcast` channel inside a `LiveBroadcaster` (lazy-create on first subscribe). `dispatch_uplink` broadcasts each telemetry frame in addition to (and before) persisting it. `RecvError::Lagged(n)` is converted to a synthetic `LogGap{reason: BufferFull}` so the UI renders a visible gap — per ADR-0008's "a slow subscriber must not slow ingestion".
- **Dependencies**: WS-09 (ingestion path is the producer).
- **Risk**: slow subscriber → handled by bounded channels (capacity 1024 per stack) + lag-to-gap conversion; ingestion never blocks on subscribers.

## Status Updates

**2026-05-23** — Done on branch `feat/i-0019-ws-broker-agent-channel`.

- New module `ws/broadcaster.rs`: `LiveBroadcaster` with per-stack `broadcast::Sender`, lazy-creation on subscribe, diagnostic counters (`stack_count`, `subscriber_count`) for WS-13.
- New module `ws/subscribe.rs`: `GET /api/v1/stacks/{id}/live` upgrade handler. Auth via the existing PAK middleware + an additional `authorise()` check that admin-or-owning-generator can read the stack. Lag conversion to `LogGap` for slow subscribers.
- `ws/handler.rs::dispatch_uplink` extended with a `broadcaster` parameter:
  - K8sEvent → `broadcast` then `agent_k8s_events.create`
  - PodLogLine → `broadcast` then `agent_pod_logs.create`
  - LogGap → `broadcast` only (not persisted, per WS-09)
  - Broadcast is *before* DAL insert so subscribers see frames at ingest latency.
- `internal_routes` now takes a `broadcaster: Arc<LiveBroadcaster>` (passed down to the connection task).
- `configure_api_routes` creates one `LiveBroadcaster::new()`, threads it into both `internal_routes` (producer) and `subscribe_routes` (consumer), and exposes it as a `.layer(Extension(...))` for any future code that needs to introspect it.
- Tests:
  - 4 unit tests in `ws::broadcaster::tests` (no-subscriber no-op, per-stack filtering, counter accuracy, type-agnostic delivery)
  - 2 new integration tests (`live_subscription_forwards_agent_telemetry_to_subscribers`, `live_subscription_rejects_unauthorised_caller`)
  - 13/13 `api::ws` integration tests now green; 4/4 broadcaster unit tests green.

**Deferred**:
- Multi-stack subscriptions (one WS upgrade subscribing to N stacks) — single-stack is enough for the UI live-tail view in WS-12.
- Server-sent ping/pong for ingress timeout protection — moved to WS-14 docs (recommend a 5-minute idle timeout on ingress).
- Agent-side subscription support (agents subscribing to their own targets) — not in scope; agents already have the bidi internal channel.