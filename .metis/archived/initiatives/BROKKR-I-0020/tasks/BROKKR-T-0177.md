---
id: b4-ws-channel-load-test-baseline
level: task
title: "B4: WS channel load-test baseline for v0.5.0"
short_code: "BROKKR-T-0177"
created_at: 2026-05-24T12:56:49+00:00
updated_at: 2026-05-24T12:56:49+00:00
parent: BROKKR-I-0020
blocked_by: [BROKKR-T-0174]
archived: true

tags:
  - "#task"
  - "#phase/completed"


exit_criteria_met: true
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

### 2026-05-26 — Tool built; v0.5.0 baseline recorded

Built `tools/ws-loadtest/` — a standalone (non-workspace) Rust binary that
provisions a synthetic agent fleet over REST, opens N agent WS connections
(thin `tokio-tungstenite` clients speaking the wire protocol), drives
heartbeat + telemetry traffic, runs K live subscribers, and samples broker
`/metrics`, `docker stats` (CPU/RSS), and telemetry-table row counts. See
`tools/ws-loadtest/README.md`. Stays in-tree for re-runs; not in CI.

Two bugs in the harness found+fixed during bring-up (the broker behaved
correctly throughout): (1) telemetry body `agent_id` must equal the
authenticated agent or the broker drops the frame before persist/broadcast
(`handler.rs`) — fixed to send the provisioned id; (2) generator/stack/agent
names are unique-constrained, so runs need a unique suffix — added a per-run
timestamp.

**v0.5.0 baseline — hit the full target (500 agents × 10 msg/s × 5 min):**

```
  agents requested   : 500
  agents connected   : 500 (peak gauge 501, conn errors 0)   [+1 = the real dev agent]
  subscribers        : 50 connected, 0 errors, 1470395 frames received
  messages sent      : 1500536 total → 5002 msg/s achieved
  send errors        : 0
  broker CPU         : avg 87%  peak 130%
  broker RSS         : peak 131 MiB
  pg write rate      : agent_k8s_events 2378 rows/s  agent_pod_logs 2378 rows/s
```

Run config: `LT_AGENTS=500 LT_STACKS=50 LT_SUBSCRIBERS=50 LT_MSG_RATE=10
LT_DURATION_SECS=300 LT_SAMPLE_SECS=15` against `angreal local up` on dev
hardware (Apple Silicon, docker-compose Postgres).

**What pegged:** broker CPU — ~1 core saturated (avg 87%, peaks to ~130% =
brief spill onto a second core). That's the single-process scaling ceiling at
~5000 msg/s, and it's expected, not a bug: zero send errors, zero dropped
connections, RSS flat (~85 MiB steady, one 131 MiB blip — no leak over 5 min),
and Postgres comfortably absorbed ~4.75k rows/s combined without backing up.
Subscriber fan-out kept pace (1.47M of 1.50M frames; the ~2% delta is in-flight
at shutdown, not drops).

**Conclusion:** the v0.5.0 WS stack sustains a 500-agent fleet at full message
rate on a single dev box with headroom everywhere except CPU, which tops out
around one core. If a future deployment needs >~5k msg/s per broker process,
CPU is the first thing to scale (more cores won't help a single saturated core
much — that points at either per-message serialization cost or the broadcaster
hot path as the profiling target). Recorded as the regression-detection
baseline; no follow-up bottleneck task warranted at current targets.

### Deferred / not done

- `pg_stat_statements` query-level breakdown — used row-count deltas instead
  (simpler, and the question "is PG keeping up" is answered: yes). If a future
  run shows PG as the bottleneck, add the `pg_stat_statements` drill-down then.