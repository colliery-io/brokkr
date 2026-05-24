---
id: b4-ws-channel-load-test-baseline
level: task
title: "B4: WS channel load-test baseline for v0.5.0"
short_code: "BROKKR-T-0177"
created_at: 2026-05-24T12:56:49.000000+00:00
updated_at: 2026-05-24T12:56:49.000000+00:00
parent: BROKKR-I-0020
blocked_by:
  - BROKKR-T-0174
archived: false

tags:
  - "#task"
  - "#phase/todo"


exit_criteria_met: false
initiative_id: BROKKR-I-0020
---

# B4: WS channel load-test baseline for v0.5.0

## Parent Initiative

[[BROKKR-I-0020]]

## Objective

The broker now runs per-connection mpsc lanes, a `LiveBroadcaster`, a
continuous eviction worker, and a 6h-ceiling-bounded Postgres write
path — none of this has been load-tested. Establish a v0.5.0 baseline
data point so future regressions are detectable, and confirm the
current implementation survives a realistic fleet at non-trivial
message rate.

## Acceptance Criteria

- [ ] Synthetic agent generator (probably a small Rust binary under
      `tools/` or a script under `bench/`) that fans out N concurrent WS
      connections, each authenticated with a unique PAK
- [ ] Each synthetic agent sends `Heartbeat` every 5s and an `AgentEvent`
      every 10s (configurable)
- [ ] Tool runs target: **500 agents × 10 msg/sec sustained for 5 minutes**
- [ ] Concurrently runs 50 live subscribers (admin PAK, subscribing to
      different stacks) reading the broadcast
- [ ] Captures broker CPU %, RSS (MB), and Postgres write rate (rows/sec
      into `agent_k8s_events` + `agent_pod_logs`) — record from `/metrics`
      and `pg_stat_statements`
- [ ] Records results in this task's Status Updates section as the v0.5.0
      baseline (raw numbers + a sentence on whether anything pegged)
- [ ] Cross-references the result in the I-0020 initiative doc so it
      survives task archival
- [ ] **One-off** — not added to CI. The tool stays in-tree for re-runs

## Implementation Notes

### Technical Approach

- The synthetic agent doesn't need the real `brokkr-agent` runtime — a
  thin `tokio-tungstenite` client that knows the wire protocol is enough.
  Reuse `brokkr-wire` types for serialization correctness
- Run against the `angreal local up` stack on dev hardware. If dev hardware
  can't sustain 500 agents, document the actual achievable number as the
  baseline — the absolute target matters less than the recorded number
- The 50 live subscribers exercise the per-stack `broadcast::Sender` lag
  path; if any get lagged, that's interesting data

### Dependencies

Blocked by [[BROKKR-T-0174]] (B1 version bump) — the baseline is recorded
against 0.5.0 specifically

### Risk Considerations

- If the load test surfaces a real bottleneck (Postgres write contention,
  broadcaster lag storms, mpsc full→drop), that becomes a new task — not
  a reason to delay the recording of the baseline number
- Don't run this against a shared dev DB; use the local docker-compose
  Postgres so we measure the broker, not someone else's load

## Status Updates

*To be added during implementation. Record load-test results here.*
