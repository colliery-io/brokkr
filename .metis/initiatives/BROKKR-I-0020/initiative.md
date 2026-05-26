---
id: ws-channel-operational-hardening
level: initiative
title: "WS Channel Operational Hardening & Functional Testing"
short_code: "BROKKR-I-0020"
created_at: 2026-05-24T12:52:58.560174+00:00
updated_at: 2026-05-24T13:02:12.168794+00:00
parent: BROKKR-V-0001
blocked_by: []
archived: false

tags:
  - "#initiative"
  - "#phase/active"


exit_criteria_met: false
estimated_complexity: L
initiative_id: ws-channel-operational-hardening
---

# WS Channel Operational Hardening & Functional Testing Initiative

## Context

BROKKR-I-0019 landed the internal WS channel between broker and agent end-to-end:
wire protocol, registry, priority writer, telemetry persistence with 6h hard ceiling,
live fan-out, REST history endpoints, and ergonomic wrappers in all three SDKs.
14/14 tasks completed; all three SDK contract suites pass against a real broker.

The honest review surfaced that I-0019 is **structurally complete but light on
operational hardening**. Specifically:

- The agent process with the new spawned tasks (`broker_ws::spawn`,
  `kube_events::spawn`, `pod_logs::spawn`) has **never been smoke-tested in
  the full deployment** — only in unit/integration harnesses.
- The **load-bearing claim** of the whole design ("WS is an additive optimization
  with automatic REST fallback that can never cause an outage") is not proven
  end-to-end. WS-06 covered the structural decision branch but no chaos test
  drives the full agent runtime against a kill-switched broker.
- The kube-events and pod-logs tailers have unit tests for their cache and
  rate-limiter pieces but no test that they actually emit when k3s does.
- Production posture work didn't get done: no SDK version bump, no Grafana
  dashboards for the 5 new metrics, no load test, PAK revocation doesn't kill
  open WS sockets.
- A few code smells were left in place (RateLimiter `Pass`/silent-drop naming;
  `annotation_lookup` doing one dynamic-API call per cache miss).

This initiative closes those gaps **on the same feature branch
(`feat/i-0019-ws-broker-agent-channel`)** so the WS channel ships as a single
bundled PR, then bump-and-merge.

## Goals & Non-Goals

**Goals:**
- Prove the REST-fallback claim with a real end-to-end chaos test, not a unit test.
- Smoke-test the agent runtime with all WS-spawned tasks against a real broker.
- Exercise the kube-events and pod-logs tailers against a real k3s cluster.
- Hit production posture before merge: SDK version bump, Grafana dashboards,
  PAK-revocation socket close, a load-test data point.
- Clean up the two known API smells (RateLimiter, annotation_lookup scaling).
- Land an answer for browser-side live tail (either broker-side or proxy-side).

**Non-Goals:**
- Multi-replica broker fan-out (HA) — deferred per ADR-0008, future initiative.
- Label/annotation-selector-targeted work-order push — future feature, not hardening.
- Per-stack retention overrides — current global 6h ceiling is the product stance.
- Cursor-style pagination on history endpoints — current limit/since is sufficient
  for the 6h window.

## Detailed Design

Three task clusters on the existing branch, sequenced A → B → C.

### Cluster A — Functional / chaos testing

**A1: Agent runtime smoke test.** New angreal scenario (probably
`angreal tests e2e --scenario ws-smoke`) that boots broker + agent via the
existing docker-compose, asserts:

- `brokkr_ws_connected_agents == 1` within 10s
- Kill broker → agent enters REST-fallback within `agent.poll_interval * 2`
- Seed a work order during the outage → agent reconciles it via REST poll
- Restart broker → WS reattaches, `brokkr_ws_connected_agents == 1` again
- Subsequent work order arrives via WS push, not poll

**A2: Full-loop chaos test for the fallback path.** Same harness as A1, driven
by a longer scenario:

- Seed N work orders with a network proxy (toxiproxy or `tc netem`) between
  agent and broker
- Sever the WS connection mid-flight at randomized intervals while leaving
  REST reachable
- Assert zero dropped reconciliations, all work orders reach `completed`,
  duplicate-reconcile detection holds
- Capture `brokkr_ws_messages_total{direction,type}` deltas and confirm the
  drop is observable in metrics

**A3: Real-kube test for kube-events + pod-logs tailers.** Extend the existing
`angreal helm test` k3s harness:

- Deploy a stack with `brokkr.io/stream-logs: "true"` and the
  `k8s.brokkr.io/stack` annotation
- Force a pod failure (bad image), assert a Warning event lands in
  `agent_k8s_events`
- Drive a pod that logs >100 lines/sec, assert a `LogGap{RateLimit}` row
  appears in `agent_pod_logs`
- Assert opt-out stack produces zero rows (annotation enforcement)

**A4: Post-commit push race test.** Broker integration test: concurrent REST
GET of an agent's targets + WS `target_changed` push for the same agent.
Assert the agent state machine doesn't double-reconcile and the push isn't
silently dropped because of a competing read lock.

### Cluster B — Operational / production posture

**B1: SDK lockstep version bump.** Per `project_release_versioning`: bump
`brokkr-wire`, `brokkr-client` (Rust), `brokkr-broker-client` (Python),
`@colliery-io/brokkr-client` (TS), and the broker/agent crates to the next
minor (proposal: 0.5.0). Update CHANGELOG entries for each. **Do not** publish —
the existing `release-sdks.yml` does that on tag.

**B2: Grafana dashboard for the 5 new metrics.** Add a panel set to the
existing dashboard JSON (or create `dashboards/ws-channel.json`):

- `brokkr_ws_connected_agents` (single-stat + sparkline)
- `brokkr_ws_messages_total` rate by `{direction, type}` (stacked area)
- `brokkr_ws_live_subscribers` (gauge)
- `brokkr_ws_log_eviction_runs_total` rate (sanity check: > 0 means worker alive)
- `brokkr_ws_telemetry_evicted_total{table}` rate (eviction throughput by table)

Alert rule: `brokkr_ws_log_eviction_runs_total` no-op for >2× tick interval
(eviction worker dead → 6h ceiling about to be violated).

**B3: PAK revocation kills open WS.** Wire the existing PAK invalidation
path (admin DELETE on a PAK) to call `ConnectionRegistry::close_for_agent(id)`.
Add an integration test: agent connects → admin revokes PAK → assert
socket closes within 1s and `brokkr_ws_connected_agents` decrements.

**B4: Load test data point.** Synthetic agent generator script that fans out
N connections, each sending heartbeat + occasional events at a configurable
rate. Target: 500 agents × 10 msg/sec sustained for 5 minutes, plus 50 live
subscribers reading the broadcast. Capture broker CPU, RSS, and Postgres write
rate; record results in this initiative as a v0.5.0 baseline.

### Cluster C — UX/cleanup

**C1: Browser-side live tail.** Pick one of:

- (a) Broker accepts PAK via `Sec-WebSocket-Protocol` header (RFC 6455 abuse
  but the standard browser workaround), OR
- (b) ui-slim adds an SSE proxy endpoint server-side that injects the PAK

Decision should be captured as an ADR amendment or a short ADR-0009. Once
wired, replace the ui-slim "live tail unavailable" stub with an actual tail
view that mirrors the WS-12 telemetry tabs.

**C2: `RateLimiter` API cleanup.** Rename `Allowance::Pass` → `Allowance::Drop`
and add `Allowance::DropAndGap` for the explicit gap-frame-emitted case.
Update `pod_logs.rs` consumer + tests; the on-the-wire `LogGap{RateLimit}`
frame stays exactly as-is.

**C3: `annotation_lookup` scaling.** Two-line fix in `kube_events.rs`:
add a bounded `NotOurs` LRU cache (lru crate, cap 10_000 entries) so the
miss path is O(1) for non-managed UIDs after first lookup. Add a unit test
that hammers the lookup with 50k unique non-managed UIDs and asserts API
call count stays bounded.

## Testing Strategy

The whole point of this initiative is functional + chaos testing, so testing
strategy is the work itself. Specifically:

- A1–A4 deliver e2e/integration tests that should run in CI on every push to
  this branch (gated on docker, k3s, or both)
- B3 and C3 deliver focused unit/integration tests against the new code paths
- B4 is a one-off baseline, not CI

Existing test suites (broker unit/integration, agent unit, SDK contract ×3,
helm test) must remain green throughout — no regressions, no skipped tests.

## Alternatives Considered

**Ship I-0019 as-is and do hardening in a follow-up PR after merge.**
Rejected: the load-bearing claim (REST fallback never causes outage) is
unproven end-to-end, and merging an untested fallback path to main is exactly
the kind of "ship now, hope later" we said the 6h-ceiling design was meant
to avoid. The branch is the right place to harden.

**Split hardening into separate initiatives by cluster.** Rejected by the
user — bundling keeps the scope of the single PR contained, and the SDK
version bump (B1) naturally caps the whole change at one merge.

**Address the deferred features (HA, label-selector push, per-stack retention)
as part of this initiative.** Rejected: those are features, not hardening,
and would balloon scope and time-to-merge for the WS channel work that's
otherwise ready.

## Implementation Plan

**Sequencing:** A → B → C, on `feat/i-0019-ws-broker-agent-channel`.
Each task is a separate Metis task document under this initiative; each task
commits to the branch when it's green.

**Order rationale:** A first because it retires the biggest unknown (the
fallback claim) and might surface issues that change B/C. B before C because
production posture gates merge — if B uncovers something nasty, C is wasted
polish. C last because it's pure cleanup that won't gate the decision to merge.

**Merge gate (exit criteria):**
- All Cluster A tests green in CI
- All Cluster B items shipped (SDK versions bumped, Grafana dashboard checked in,
  PAK revocation test green, load test results recorded in this doc)
- All Cluster C items either shipped or explicitly deferred with rationale
  recorded here
- Full test suite green on the branch (`angreal tests unit/integration/e2e/sdk-contract`)
- Manual approval before tag + publish + merge

## Exit Criteria

- [x] A1 agent runtime smoke test green in CI (BROKKR-T-0170, 2026-05-24)
- [x] A2 chaos test green in CI (BROKKR-T-0171, 2026-05-24) — narrowed scope: REST-heartbeat-fallback proven, work-order-completion lifecycle deferred (no harness for it yet; revisit with A3)
- [x] A3 real-kube tailer test green in CI (BROKKR-T-0172, 2026-05-26) — `angreal tests e2e --scenario ws-telemetry`; events + pod-logs both proven through real k3s. **Found & fixed a real agent bug**: pod-logs tailer lost the container-startup race and never streamed a freshly-created pod's logs (pod_logs.rs reopen loop). LogGap-rate-limit + opt-out negative tests deferred
- [ ] A4 push/poll race test green in broker integration suite
- [ ] B1 SDK lockstep version bump applied + CHANGELOGs updated
- [ ] B2 Grafana dashboard JSON checked in + eviction-worker dead-alert rule
- [ ] B3 PAK revocation closes WS socket; integration test green
- [ ] B4 load test executed, results recorded in this initiative as v0.5.0 baseline
- [ ] C1 browser live-tail decision recorded (ADR amendment or ADR-0009) and implemented
- [ ] C2 `RateLimiter` API renamed, tests updated, on-wire frame unchanged
- [ ] C3 `annotation_lookup` LRU added, scaling test asserts bounded API calls
- [ ] T-0181 docs: ws_url config — docs page, helm values, ADR-0008 amendment, C4 caption (follow-up from T-0171 introducing the new agent config)
- [ ] Existing test suites still green
- [ ] Initiative review with human before tag + merge