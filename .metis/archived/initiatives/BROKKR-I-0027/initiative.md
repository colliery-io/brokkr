---
id: agent-fleet-legibility
level: initiative
title: "Agent Fleet Legibility"
short_code: "BROKKR-I-0027"
created_at: 2026-06-12T21:20:39.756079+00:00
updated_at: 2026-06-13T13:59:19.142159+00:00
parent: brokkr-environment-aware
blocked_by: []
archived: true

tags:
  - "#initiative"
  - "#phase/completed"


exit_criteria_met: false
estimated_complexity: M
initiative_id: agent-fleet-legibility
---

# Agent Fleet Legibility Initiative

## Context

Brokkr is most often embedded as a substrate component inside someone else's
platform. The **platform builder who embeds Brokkr** needs to give *their* users
confidence in the agent fleet — but today that is hard work:

- Liveness is pull-stale: `brokkr_active_agents` / `brokkr_agent_heartbeat_age_seconds`
  only refresh when something calls `GET /api/v1/agents` (computed inside that
  handler; no background sweep), and `Agent.status` never auto-transitions when
  an agent goes dark.
- "Health" must be derived by hand from raw timestamps and metrics; there is no
  single surface that answers "is my fleet OK, and if not, who and why?"
- The failure mode "alive but stuck" (heartbeating fine, doing nothing) is
  invisible without manual correlation.

This supersedes the archived **[[BROKKR-I-0013]]** (Agent Fleet Operational
Data), which framed the problem as "expose operational data via the API" and
made the consumer compute fleet health themselves. This initiative reframes it
around legibility: the consumer wants the verdict-enabling **signals and the
outliers**, not a pile of fields — *"this is your fleet, it's in good shape,
except for Fred who's been on the fritz."*

What has shipped since I-0013 and reshapes the foundation:
- The internal **broker↔agent WebSocket channel** (I-0019/I-0020) — live
  connectivity is now an *event*, not a poll (`ws_connected_agents`,
  `/api/v1/admin/ws/connections` with connected_at + subscriptions).
- The **work-orders** system — per-agent queue/backlog is partly modeled.
- **Deployment-health** endpoints and **telemetry** (k8s events, pod logs over
  WS, 6h retention) — reconciliation/activity visibility partly exists.

## Goals & Non-Goals

**Goals:**
- Make a Brokkr fleet **trivially legible** to the platform embedding it: one
  programmatic surface that, per agent, exposes **connectivity** (who is live
  now), **recent activity** (what each agent has been doing), and a curated set
  of **health signals** as *measured values*.
- Design philosophy — **Brokkr surfaces signals; the consumer owns severity.**
  Indicators are returned as raw measured values; thresholds, coloring, and
  alerting belong to the consumer's platform. This keeps platform-specific
  policy out of Brokkr and keeps the surface on the API-data side of the OTEL
  boundary.
- The consumer should be able to render "this is your fleet" without writing
  PromQL or joining heartbeat timestamps — Brokkr does the measurement, they do
  the presentation and the judgment.

**Non-Goals:**
- Brokkr deciding health **verdicts**, thresholds, or severity for an agent.
- **Alerting / paging** built into Brokkr (the consumer's platform monitors on
  top of the signals).
- Replacing external monitoring or **OTEL** system telemetry (that is for
  monitoring Brokkr itself; this is platform data for Brokkr's clients).
- Using this data for **placement/targeting** decisions (labels handle that).
- Validating agent-reported data against reality (trust the agent).

## Health Signals (v1 set)

Per-agent indicators, returned as measured values for the consumer to interpret:

| Signal | What it tells the consumer | Data locality |
|--------|----------------------------|---------------|
| **Deployment-object backpressure** | objects assigned to the agent are piling up / not draining — he is falling behind | Broker-computed (knows pending objects/work per agent) |
| **Heartbeat staleness** | time since last heartbeat — gone dark or flaky | Broker-computed (`last_heartbeat`) |
| **K8s connectivity** | the agent cannot reach its own cluster API — an infra problem, not Brokkr | **Agent-reported** (only the agent knows) |
| **Activity silence** | heartbeating/polling normally but producing no events — *alive but stuck/idle* | Broker-computed (last-event-time vs last-poll/heartbeat-time) |

Note the natural seam: **three of four are broker-computable today with no agent
changes**; only K8s connectivity requires the agent to self-report. This implies
a fat first slice that needs zero agent rollout, plus a thinner agent-reported
addition — but sequencing is a design-phase concern.

## Use Cases

### UC-1: "Is my fleet OK?" at a glance
- **Actor:** Platform operator (via the embedding platform's console).
- **Scenario:** Opens the fleet view; the platform calls one Brokkr surface and
  renders every agent with its connectivity + signals. 39 agents nominal; Fred
  shows a stale heartbeat and rising backpressure.
- **Expected Outcome:** The operator sees "fleet healthy except Fred" without
  computing anything; the platform applied its own thresholds to the raw signals.

### UC-2: "Alive but stuck"
- **Actor:** Platform operator.
- **Scenario:** Fred's heartbeat is fresh and he's polling, but he has produced
  no events for an unusually long window. The activity-silence signal surfaces
  it even though every liveness check looks green.
- **Expected Outcome:** A failure that a pure liveness/heartbeat check would miss
  is visible.

### UC-3: "Is it Brokkr or the infra?"
- **Actor:** Platform operator debugging failed deployments to one agent.
- **Scenario:** Deployments to Fred are failing; the operator checks his signals
  and sees `k8s_reachable: false`.
- **Expected Outcome:** Root cause attributed to the agent's cluster, not Brokkr
  or the application.

## Alternatives Considered

- **Brokkr computes a health verdict (`healthy`/`degraded`/`down`).** Rejected:
  what counts as unhealthy is platform-specific; baking thresholds into Brokkr
  makes it wrong for someone. Surface signals, let the consumer judge.
- **Push all of this to OTEL / external monitoring.** Rejected: this is platform
  data consumed by Brokkr's API clients to render *their* fleet view, not system
  telemetry for monitoring Brokkr. Different audience and access pattern.
- **Patch the archived I-0013 in place.** Rejected: it predates the WS channel
  and work-orders and assumes poll-only with "no real-time streaming"; cleaner to
  re-discover than retrofit.

## Resolved Decisions (Discovery)

Grounded against the existing data model. Key finding: **every v1 signal except
K8s connectivity is computable from data the broker already has, as measured
values** — so Slice 1 needs no migrations and no agent changes.

- **Surface (Q1):** Pull-first `GET /api/v1/fleet` returning a per-agent record
  array (the "this is your fleet" table) plus a per-agent detail view with the
  recent-events feed. WS *push* of fleet-state deltas is a later, optional
  enhancement — not a v1 dependency.
- **Liveness (Q2):** Surface BOTH as distinct fields — never a single `is_live`
  verdict: `ws_connected` + `connected_since` (real-time, WS-only) AND
  `heartbeat_age_seconds` (any agent). The consumer decides which matters.
- **Backpressure (Q4):** Surface BOTH as separate feeds — `pending_object_count`
  (target-state objects with no agent_event yet) and the work-order backlog
  (`pending_work_orders` / `claimed_work_orders` per agent). A per-agent
  applied-sequence lag is deferred (needs new tracking).
- **Activity silence (Q3):** Return `last_event_at` + `seconds_since_last_event`
  as measured values; the consumer derives "alive but silent." Brokkr computes
  no "silent" boolean.
- **Staleness freshness (Q7):** The `/fleet` surface computes `heartbeat_age` on
  read (always fresh). The pre-existing Prometheus gauge-staleness bug
  (`active_agents` / `agent_heartbeat_age_seconds` only refresh on
  `GET /agents`) is **folded into this initiative** — fixed via a small
  background refresh task in Slice 1.
- **K8s connectivity (Q5):** Kept as **Slice 2** — a standalone agent-reported
  signal (heartbeat-payload extension + a stored column), surfaced in `/fleet`.
  Self-contained; sequenced after Slice 1 proves the surface.
- **Recent window / retention (Q6):** Activity feed = latest N agent_events per
  agent. NOTE: `agent_events` currently has **no eviction** (grows unbounded) —
  tracked as an orthogonal follow-up task, not a legibility blocker.

## v1 Fleet Record (per agent)

`GET /api/v1/fleet` returns an array of these — all measured values, no verdicts:

| Field | Meaning | Source |
|-------|---------|--------|
| `agent_id`, `name` | identity | `agents` |
| `status` | the agent's self-set status string | `agents.status` |
| `ws_connected`, `connected_since` | live WS connectivity | WS registry (`is_connected`) |
| `last_heartbeat`, `heartbeat_age_seconds` | heartbeat liveness | `agents.last_heartbeat` (computed on read) |
| `pending_object_count` | deployment objects targeted but not yet acted on | target-state minus `agent_events` |
| `pending_work_orders`, `claimed_work_orders` | work-order backlog | `work_orders` by agent |
| `last_event_at`, `seconds_since_last_event` | activity recency | `max(agent_events.created_at)` |
| `health_failing`, `health_degraded` | reconciliation trouble counts | `deployment_health` by status |
| `k8s_reachable` *(Slice 2)* | agent can reach its cluster API | agent-reported |

## Implementation Plan

- **Slice 1 — Broker-computed fleet surface** *(no migrations, no agent changes)*:
  `GET /api/v1/fleet` (rollup) + per-agent detail with the recent-events feed,
  surfacing every field in the v1 record above except `k8s_reachable`. Fold in
  the Prometheus gauge-staleness fix (a background refresh of `active_agents` /
  `agent_heartbeat_age_seconds` so they're correct independent of who polls
  `GET /agents`).
- **Slice 2 — K8s connectivity** *(standalone, agent-reported)*: extend the
  heartbeat payload with `k8s_reachable` (+ optional API latency), store the
  latest per agent (migration), surface it in `/fleet`.
- **Later / optional — WS push:** push fleet-state-change deltas over the
  existing WS channel for consumers that want live updates instead of polling.
- **Orthogonal follow-up (separate task):** an `agent_events` retention/eviction
  policy (the table currently grows unbounded).

## Status Updates

- 2026-06-12: Created in discovery from the re-scope of archived [[BROKKR-I-0013]].
  Goals/Non-Goals and the v1 signal set agreed with the maintainer (surface
  signals, consumer owns severity). Design phase deferred pending the open
  questions above.

- 2026-06-12: Open questions worked through and resolved with the maintainer
  (see Resolved Decisions). Grounded against the data model — Slice 1 is a pure
  read/aggregation feature (no migrations, no agent changes). Decisions: fold in
  the Prometheus gauge-staleness fix; surface object-pending AND work-order
  backlog as separate feeds; K8s connectivity is Slice 2 (standalone); pull-first
  `GET /fleet` with WS push deferred. Ready to consider discovery → design and
  decomposition into Slice 1 / Slice 2 / follow-up tasks.
## Closeout 2026-06-13

Completed. All decomposed slices shipped (merged in PR #64):
- T-0226 — GET /api/v1/fleet broker-computed fleet surface (+ Prometheus gauge-staleness fix).
- T-0227 — agent-reported K8s connectivity signal (migration 20, WS heartbeat wire extension, surfaced in /fleet).
- T-0228 — agent_events retention/eviction.

The goal is met: a platform embedding Brokkr can hit one endpoint and get the
whole fleet's connectivity, backpressure, heartbeat staleness, activity recency,
health-status counts, and k8s reachability as measured values — severity left to
the consumer.

DEFERRED (not pursued in this initiative): the optional **WS push** of
fleet-state-change deltas. Pull-first GET /fleet was the v1 surface and is
sufficient. If real-time push demand appears, open it as a fresh task/initiative
rather than reviving this one.