---
id: websocket-channel-for-broker-agent
level: initiative
title: "WebSocket Channel for Broker ↔ Agent Communication"
short_code: "BROKKR-I-0019"
created_at: 2026-05-22T23:45:15.203134+00:00
updated_at: 2026-05-27T15:34:21.926421+00:00
parent: BROKKR-V-0001
blocked_by: []
archived: true

tags:
  - "#initiative"
  - "#phase/completed"


exit_criteria_met: false
estimated_complexity: XL
initiative_id: websocket-channel-for-broker-agent
---

# WebSocket Channel for Broker ↔ Agent Communication Initiative

## Context

Today the `brokkr-agent` talks to `brokkr-broker` via the generated REST SDK (`crates/brokkr-agent/src/broker_sdk.rs`, migrated in BROKKR-T-0140). Agents poll the broker on a tick to discover new work orders, fetch desired state, and post events/health back. The REST path is well-exercised and is the right contract for **external** integrators (operators, generated SDKs in Rust/Python/TS, ad-hoc tooling), and per the lockstep SDK release strategy ([[project_release_versioning]]) it must keep working.

Polling has two costs that show up as the fleet grows:

1. **Latency floor** — a new work order waits for the next agent tick before it is even noticed. Reconciliation perceived end-to-end time is dominated by poll interval, not by broker or agent work.
2. **Steady-state load** — every agent pays a per-tick auth + DB roundtrip on the broker even when there is nothing to do. This is the same pressure motivating BROKKR-T-0128 (auth middleware caching).

We want a persistent, push-capable channel between broker and agents so that:
- New work is delivered to the agent the moment the broker knows about it
- Health / heartbeat / event reporting can flow continuously without per-message handshake overhead
- The REST API remains the canonical, documented, OpenAPI-described surface for everything **external**

This is an **internal-only optimization channel**, not a replacement for the public API.

## Goals & Non-Goals

**Goals:**
- Add a WebSocket (or equivalent persistent) channel between `brokkr-agent` and `brokkr-broker` for low-latency control-plane messaging
- Reduce time-to-reconcile for new work orders from "≤ poll interval" to "≤ network RTT + processing"
- Reduce steady-state broker auth/DB load from idle agents
- **Stream Kubernetes Events and pod logs** for objects the agent deployed, upstream to the broker, where they are both persisted (for history) and fanned out to live subscribers (for tail / UI)
- Keep the full REST/OpenAPI surface intact and authoritative for external consumers and the generated SDKs
- Preserve PAK-based auth and tenant isolation on the new channel
- Define a clear fallback: if WS is unavailable or disabled, the agent must continue working via REST polling

**Non-Goals:**
- Exposing WebSockets to external SDK users for control-plane messages (Rust/Python/TypeScript generated clients keep the REST contract for work-order ops). *Note: a read-only WS/SSE subscription for log/event tail in the UI is in scope, since there is no sensible REST shape for it.*
- Streaming logs/events for objects the agent did **not** deploy (no general cluster log shipping; not a replacement for Loki/Fluent Bit)
- **Long-term log retention or log analytics.** Brokkr streams logs for *immediate operational support* (debugging a deploy that just happened, watching a rollout). Hard cap: **6 hours** of retained logs per stream; configurable *down* from there, never up. Teams that need long-term log centralization, search, or analytics should ship to Datadog (or equivalent) — Brokkr will not grow into that role.
- Replacing webhook delivery (BROKKR-T-0046 / BROKKR-T-0091) — webhooks remain the outbound integration mechanism
- Bidirectional streaming of large payloads (rendered deployment objects, build artifacts) — those stay on REST/object endpoints
- Changing the v1 OpenAPI spec for existing endpoints (new endpoints for event/log history are additive)
- Multi-broker / broker-to-broker comms

## Requirements (draft)

**Functional**
- REQ-001: Agent can open an authenticated persistent connection to the broker using its existing PAK
- REQ-002: Broker can push inline `work_order`, `target_changed`, and `stack_changed` messages to a specific agent
- REQ-003: Agent can send heartbeat / health / events over the channel without opening new HTTP connections
- REQ-004: If the WS channel drops, the agent transparently falls back to REST polling and reconnects with backoff
- REQ-005: Broker tracks connected agents and can answer "is agent X currently connected?" for diagnostics
- REQ-006: Agent tails Kubernetes `Events` for objects it deployed (scoped via owner reference / managed-by label) and streams them upstream over WS
- REQ-007: Agent streams pod logs for containers in managed workloads upstream over WS (per-stack opt-in; see design questions)
- REQ-008: Broker persists received events and logs in a short-lived store (new `agent_k8s_events` and `agent_pod_logs` tables) with a **hard 6-hour TTL**; per-stack config can shorten retention but cannot extend it. Paginated history exposed via REST/SDK for the retained window only.
- REQ-009: Broker fans out live events/logs to subscribed UI / SDK clients (read-only WS or SSE subscription, PAK-scoped to stacks the caller can read)
- REQ-010: Agent applies backpressure / rate-limit / sampling on log streams to keep the WS channel from being overwhelmed by a chatty workload

**Non-functional**
- NFR-001: Notification delivery latency p50 < 250 ms inside a cluster
- NFR-002: A disconnected agent must not miss work — on reconnect it reconciles via the existing REST endpoints (the WS channel is a *hint*, not the source of truth for work orders)
- NFR-003: Auth model and tenant isolation are unchanged; a PAK only sees its own agent's traffic, and log/event subscribers only see streams their PAK can read
- NFR-004: Feature flag / config switch to disable the WS channel and force REST-only behavior (for debugging, for restricted environments)
- NFR-005: Log/event ingestion does not block control-plane messages on the same WS connection (separate logical channels or message priority)
- NFR-006: Log retention is bounded by a hard ceiling (6h) enforced at the broker — eviction runs continuously, not opportunistically. Storage growth is predictable from `(fleet log rate) × 6h` regardless of caller config.
- NFR-007: Broker exposes a clear "this is not your log store" signal in docs, UI, and SDK (e.g. responses on the history endpoint indicate the retention ceiling and recommended sinks for long-term retention).

## Architecture (to be detailed in design phase)

C4 context is unchanged: external systems still talk to the broker over REST. The new edge is **inside** the control plane, between the `brokkr-broker` container and `brokkr-agent` containers. A C4 container/component update will accompany the design phase per [[feedback_c4_architecture]].

Key design questions to resolve before decompose (see Open Questions below):
- Transport: raw WebSocket framing vs a higher-level protocol (e.g. SSE, gRPC streaming, NATS, Redis pub/sub fronted by the broker)
- Message semantics: notification-only ("something changed, go fetch") vs payload-bearing ("here is the new work order")
- Connection ownership in the broker process (Axum WS handler, background broadcaster, per-agent mailbox)
- ADR: this initiative will produce an ADR capturing the chosen approach and the explicit boundary between "internal control channel" and "external REST/SDK contract"

## Alternatives Considered (to be expanded)

- **Keep polling, just tune interval / add caching** — BROKKR-T-0128 already addresses the auth-cache angle; doesn't fix the latency floor
- **Server-Sent Events (SSE)** — simpler than WS, one-directional (broker → agent); agent→broker traffic stays on REST. Possibly a good first cut
- **gRPC bidi streaming** — proper schema'd bidi, but introduces a second RPC stack alongside REST
- **External message bus (NATS / Redis Streams)** — operationally heavier; adds a new required dependency to the control plane
- **Long-polling** — incremental improvement over short polling, but doesn't change the architecture meaningfully

## Implementation Plan (sketch — to be finalized after design check-in)

1. Discovery + design check-in with human (this phase)
2. ADR for transport choice and internal-vs-external boundary
3. Spike: minimal WS endpoint on broker + agent client that receives a single notification type, behind a feature flag
4. Promote to first-class: full notification set, reconnect/backoff, fallback to polling, metrics
5. Cut over agent's default behavior; keep polling as configurable fallback
6. Documentation update (architecture C4, agent ops docs)

## Design Decisions (from discovery check-in 2026-05-22)

- **Transport**: Native **WebSocket (bidirectional)** served by Axum on the broker, consumed by the agent. Rejected SSE (insufficient for agent uplink) and gRPC (avoids introducing a second RPC stack alongside REST/OpenAPI).
- **Message semantics**: **Inline payloads**. WS messages carry the work order / target / stack body directly rather than acting as fetch hints. Implication: the WS message schema becomes a contract that must stay aligned with the REST/SDK types — addressed by sharing the same Rust types as the SDK and covering with contract tests.
- **Agent uplink**: Heartbeat **plus events and health** flow over the WS channel in v1. REST endpoints for these stay available (for external tooling and as fallback) but the agent's default path is WS.
- **Default behavior**: **WS default, REST polling as fallback** (opt-out). This raises the bar on the reconnect / fallback path — it must be implemented and tested before the default flips.

- **Multi-replica broker**: **Out of scope for v1.** v1 assumes a single broker replica; the limitation will be documented. When HA broker becomes a requirement, a follow-up initiative will add a fan-out mechanism (Postgres LISTEN/NOTIFY is the default candidate since it adds no new infra). Agents always have the polling fallback, so a multi-replica deploy degrades to "REST-tick latency for cross-replica events" rather than missing work.
- **Versioning**: **Piggyback on the existing SDK / OpenAPI version** ([[project_release_versioning]]). WS message types are shared with the generated SDK types, so a lockstep release bumps both. No separate WS protocol version in v1; revisit only if WS evolution needs to outpace the SDK.

## Design-Phase Questions (to resolve in ADR / design doc)

These were deferred from discovery because they're implementation-shape questions, not strategic ones:

1. **Log opt-in granularity**: per-stack flag? per-deployment-object annotation (e.g. `brokkr.io/stream-logs: "true"`)? agent-wide default?
2. **Log retention defaults** (ceiling already fixed at 6h, see [[project_log_retention_stance]]): pick a sensible *default* under the ceiling (1h? 6h?), per-stack override range, eviction cadence. Max bytes per stack as a secondary cap.
3. **Schema for `agent_pod_logs`**: row-per-line vs. chunked blobs in Postgres. **External object store (S3/MinIO) is likely overkill** given the 6h ceiling — re-evaluate only if Postgres write volume becomes a problem.
4. **Backpressure protocol**: how the agent signals "I'm dropping log lines" and how the broker surfaces that gap to subscribers.
5. **Channel multiplexing**: single WS with typed messages (priority queue server-side) vs. two WS connections (control-plane + log-plane) per agent.
6. **Live subscription auth**: confirm PAK scopes cover "subscribe to stack X's live tail" without inventing a new permission.

## Exit Criteria (discovery → design)

- [x] Problem and scope framed; internal-vs-external boundary explicit
- [x] Goals / non-goals agreed with human (including event/log streaming)
- [x] Transport, payload model, uplink scope, default behavior decided
- [x] HA and versioning posture decided
- [x] Event/log streaming scope decided (Events + logs, persist + fan-out)
- [ ] ADR drafted capturing the above (next phase)
- [ ] C4 container/component update sketched for the design phase ([[feedback_c4_architecture]])
- [ ] Log retention / storage shape decided (design phase)