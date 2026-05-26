---
id: 001-internal-websocket-channel-for
level: adr
title: "Internal WebSocket Channel for Broker↔Agent Communication and Short-Lived Log Streaming"
number: 1
short_code: "BROKKR-A-0008"
created_at: 2026-05-23T02:08:51.873643+00:00
updated_at: 2026-05-23T02:12:12.439957+00:00
decision_date: 
decision_maker: 
parent: 
archived: false

tags:
  - "#adr"
  - "#phase/decided"


exit_criteria_met: false
initiative_id: NULL
---

# ADR-1: Internal WebSocket Channel for Broker↔Agent Communication and Short-Lived Log Streaming

**Related**: [[BROKKR-I-0019]] · [[project_release_versioning]] · [[project_log_retention_stance]] · [[feedback_c4_architecture]]

## Context

The `brokkr-agent` currently communicates with `brokkr-broker` exclusively over the generated REST SDK (`crates/brokkr-agent/src/broker_sdk.rs`, BROKKR-T-0140). Each agent polls on a tick for work orders, target/stack changes, and posts events/health back. The REST surface is also the public contract consumed by the Rust / Python / TypeScript generated SDKs ([[BROKKR-I-0017]], [[BROKKR-I-0018]]), released in lockstep with the broker per [[project_release_versioning]].

Polling has two structural costs:

1. **Latency floor** — new work waits for the next agent tick before discovery; perceived reconcile time is dominated by poll interval.
2. **Steady-state load** — every agent pays a per-tick auth + DB roundtrip on the broker even when idle (the same pressure driving the auth-cache work in BROKKR-T-0128).

Separately, operators need **immediate** visibility into deployments the agent is managing: kube `Events` for the rollout itself, and pod logs from the workloads being deployed. There is no acceptable REST shape for live tailing, and Brokkr explicitly does **not** want to become a log warehouse — long-term log centralization belongs in Datadog or an equivalent product ([[project_log_retention_stance]]).

This ADR resolves how the broker and agent should talk for these internal needs while keeping the public REST/SDK contract authoritative.

## Decision

Introduce a **single bidirectional WebSocket** between each `brokkr-agent` and the `brokkr-broker`, served by Axum, authenticated with the agent's existing PAK. This channel is **internal-only** — it is not part of the OpenAPI surface, not exposed to external SDK consumers for control-plane operations, and not a replacement for REST.

Concretely:

1. **Transport**: native WebSocket (`axum::extract::ws`), bidirectional, one connection per agent.
2. **Message model**: typed enum on a single multiplexed connection, with the broker applying **priority** so control-plane messages (work orders, target/stack changes, heartbeat) are never starved by log/event traffic.
3. **Payloads**: **inline** — WS messages carry the work order / target / stack body directly using the same Rust types the SDK exposes. The WS message schema is derived from the SDK types, so there is exactly one definition.
4. **Agent uplink scope (v1)**: heartbeat + agent events + health + kube Events + pod logs.
5. **Default behavior**: WS **on by default**, REST polling as automatic fallback. If the WS connection cannot be established or drops, the agent reverts to REST polling with backoff and reconnects opportunistically. A config flag forces REST-only for restricted environments.
6. **Versioning**: WS message types piggyback on the SDK / OpenAPI version ([[project_release_versioning]]) — no separate protocol-version handshake in v1.
7. **HA / multi-replica broker**: explicitly **out of scope for v1**. v1 assumes a single broker replica; this is documented. With inline payloads, a multi-replica deploy would need a fan-out bus (Postgres `LISTEN/NOTIFY` is the default candidate) to push to agents connected to other replicas — deferred to a follow-up initiative. Multi-replica still functions correctly: cross-replica events surface to the agent on the next REST polling tick.
8. **Log/event streaming**: bounded by a **hard 6-hour retention ceiling** ([[project_log_retention_stance]]). Per-stack configuration can shorten retention but never extend it. Persisted in new `agent_k8s_events` and `agent_pod_logs` Postgres tables. The broker fans out live streams to subscribed UI / SDK clients over a read-only WS/SSE subscription, PAK-scoped to stacks the caller can read. Agent applies backpressure / sampling on log streams; gaps are signalled to subscribers.
9. **External boundary**: the public OpenAPI spec is unchanged for existing endpoints. New REST endpoints for paginated event/log *history* (within the 6h window) are added and generated into the SDKs. The live WS subscription for tail is documented separately as a non-OpenAPI integration point, since SSE/WS subscriptions don't model cleanly in OpenAPI 3.0.

## Alternatives Analysis

| Option | Pros | Cons | Risk | Cost |
|---|---|---|---|---|
| **WebSocket (bidi, single channel, inline payloads)** — chosen | Full bidi from one connection; no new RPC stack; Axum-native; reuses SDK types | WS schema must be kept in lockstep with REST types; needs multiplexing/priority to keep logs from starving control plane | Med | Med |
| SSE (broker→agent only) | Simplest server; trivially scoped | No agent uplink; agent→broker traffic stays on REST, losing half the benefit; also can't carry log/event streams from agent | Low | Low |
| gRPC bidi streaming | Schema'd, generated clients, mature streaming | Introduces a second RPC stack alongside REST/OpenAPI; duplicates contract surface; awkward auth integration with PAKs | Med | High |
| External message bus (NATS / Redis Streams) | Solves multi-replica fan-out natively | New required dependency in the control plane; operational burden; doesn't help v1 (single-replica) | High | High |
| Long-polling | Incremental over short polling; no new transport | Doesn't change the architecture; no real win on latency or load | Low | Low |
| Keep polling + auth caching (BROKKR-T-0128) only | Smallest change | Doesn't address the latency floor; doesn't enable live log/event streaming | Low | Low |

**Sub-decision: log retention** — a longer retention window (24h, 7d) was explicitly rejected. The 6h ceiling encodes the product stance ([[project_log_retention_stance]]): Brokkr is a control plane with an *immediate-ops* log buffer, not a log warehouse. Users needing long-term retention ship to Datadog.

**Sub-decision: payload model** — "notification-only" (WS says "go fetch via REST") was rejected because it leaves the latency win half-realized for work orders and is a non-starter for log/event streams (the whole point is the live byte flow).

**Sub-decision: channel multiplexing** — one connection with typed messages + server-side priority was chosen over two parallel WS connections (control-plane + log-plane). Single connection = simpler auth, simpler reconnect, simpler diagnostics; priority handling is a known pattern.

## Rationale

- WebSocket is the lowest-impedance choice: it lives inside the existing Axum stack, integrates with PAK middleware the same way HTTP routes do, and the agent already pulls in WS-capable Rust deps via tonic-free paths.
- Inline payloads + shared SDK types means there is **one** schema for "what is a work order on the wire", not two. The cost (contract drift risk) is mitigated by the SDK contract tests already in place (BROKKR-T-0154).
- WS-default with REST fallback honors the user's pre-existing satisfaction with polling: if anything goes wrong with the new channel, the system degrades to the well-known behavior automatically, not to an outage.
- The 6h log retention ceiling is the constraint that makes the storage shape *tractable* — bounded storage at `(fleet log rate) × 6h` means we can use Postgres directly without needing an object store or a log-warehouse migration path.
- Priority-multiplexed single channel keeps the operational surface small (one socket per agent to monitor, one auth path, one reconnect path) while solving the starvation concern.
- Deferring HA fan-out keeps v1 small. Multi-replica is a real future need, but the polling fallback means a multi-replica deploy today is *suboptimal*, not *broken* — that's a fine v1 posture.

## Consequences

### Positive
- Time-to-reconcile drops from "≤ poll interval" to "≤ network RTT" for the common case.
- Idle agents stop generating per-tick auth + DB load on the broker.
- Operators get live kube Events + pod logs for managed workloads without standing up additional infra.
- Reduces motivation for some scope of BROKKR-T-0128 (auth cache still useful for REST callers, less critical for the agent hot path).
- C4 container diagram gains a clearly-labeled *internal* edge that distinguishes control-plane traffic from the public REST surface ([[feedback_c4_architecture]]).

### Negative
- More code surface to maintain in `brokkr-broker` (WS handler, per-agent connection registry, priority queue, eviction worker for logs) and `brokkr-agent` (WS client, multiplexer, reconnect, log/event tailers, backpressure).
- WS message types must stay aligned with SDK types; a missed update would silently diverge — mitigated by contract tests, but adds review surface.
- Persisted logs add write volume to Postgres bounded by `(fleet log rate) × 6h`; requires a continuous eviction worker and monitoring on table growth.
- Multi-replica broker deployments will see suboptimal latency for cross-replica events until the HA follow-up lands.
- The fallback path is load-bearing — it must be implemented and tested *before* the default flips to WS-on, or rollouts can regress to no-comms instead of polling.

### Neutral
- The public OpenAPI surface is unchanged for existing endpoints; SDKs only grow (new history endpoints).
- Webhook delivery from agents (BROKKR-T-0091) is unaffected and remains the outbound integration mechanism.
- Diagnostics gain a new datapoint: "is agent X currently WS-connected" — useful and worth surfacing in the UI / admin endpoints.

## Status

**Draft** — pending approval. Once accepted, this ADR's decisions are inputs to the BROKKR-I-0019 design doc and the task decomposition that follows.

## Amendments

### 2026-05-26 — WS endpoint may live on a separate ingress (BROKKR-T-0171 / I-0020)

The original decision implicitly assumed WS and REST share one ingress (the agent derived its WS URL from `broker_url`). The I-0020 chaos-test work ([[BROKKR-T-0171]]) added an optional `ws_url` agent setting: when set, the agent uses it for the WebSocket endpoint only; when unset, behavior is unchanged (derive from `broker_url`). This relaxes one implicit assumption — **the WS endpoint MAY terminate on a different ingress / load balancer than REST** — without reopening any decision above. The "WS is a hint, REST is the source of truth" invariant is untouched: REST still follows `broker_url`, and the REST fallback remains load-bearing. See `docs/explanation/internal-ws-channel` ("When to use ws_url") for operator guidance.