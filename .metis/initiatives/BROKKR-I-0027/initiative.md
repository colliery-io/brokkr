---
id: agent-fleet-legibility
level: initiative
title: "Agent Fleet Legibility"
short_code: "BROKKR-I-0027"
created_at: 2026-06-12T21:20:39.756079+00:00
updated_at: 2026-06-12T21:20:39.756079+00:00
parent: brokkr-environment-aware
blocked_by: []
archived: false

tags:
  - "#initiative"
  - "#phase/discovery"


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

## Open Questions (Discovery → Design)

To be resolved before/within the design phase:
1. **Surface shape** — per-agent endpoint + a fleet rollup (one call returns all
   agents + signals)? Pull-only, or push fleet-state changes over the existing WS
   channel?
2. **Liveness source of truth** — reconcile WS connection state (precise, but
   only for WS-connected agents) with heartbeat-age (works for any agent). Which
   wins, and how are they combined?
3. **"Activity silence" definition** — what window counts as silent, and what
   counts as "activity" (events only? events + successful reconciles?).
4. **Backpressure definition** — pending deployment objects, work-order backlog,
   or both; how "draining vs piling up" is measured.
5. **K8s-connectivity reporting** — agent self-report mechanism and heartbeat
   payload extension; sequencing relative to the broker-computed signals.
6. **"Recent" window/retention** for the activity feed (ties into the existing
   6h telemetry ceiling and agent-events retention).
7. **Staleness freshness** — fix the existing pull-triggered gauge staleness
   (`active_agents` / `heartbeat_age` only refresh on `GET /agents`) so the
   signals are accurate independent of who is polling.

## Implementation Plan

*Provisional — to be refined during the design phase once the open questions
above are settled.* The likely shape, given the broker/agent data seam:

- **Slice 1 (broker-computed, no agent changes):** backpressure, heartbeat
  staleness, and activity-silence signals + the per-agent/fleet read surface;
  also address the pull-triggered staleness gauge.
- **Slice 2 (agent-reported):** K8s-connectivity signal via a heartbeat-payload
  extension.
- **Slice 3 (activity feed):** consolidate "recent events per agent" into the
  surface, reconciled with existing agent-events + telemetry.

## Status Updates

- 2026-06-12: Created in discovery from the re-scope of archived [[BROKKR-I-0013]].
  Goals/Non-Goals and the v1 signal set agreed with the maintainer (surface
  signals, consumer owns severity). Design phase deferred pending the open
  questions above.
