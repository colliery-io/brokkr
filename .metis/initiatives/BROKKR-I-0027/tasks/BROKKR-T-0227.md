---
id: slice-2-agent-reported-k8s
level: task
title: "Slice 2: agent-reported K8s connectivity signal in /fleet"
short_code: "BROKKR-T-0227"
created_at: 2026-06-12T21:39:43.724081+00:00
updated_at: 2026-06-13T11:39:42.934491+00:00
parent: agent-fleet-legibility
blocked_by: []
archived: false

tags:
  - "#task"
  - "#task"
  - "#phase/active"


exit_criteria_met: false
initiative_id: BROKKR-I-0027
---

# Slice 2: agent-reported K8s connectivity signal in /fleet

## Parent Initiative

[[BROKKR-I-0027]]

## Objective

Add the one fleet signal the broker cannot compute on its own: whether each agent
can reach its own Kubernetes API. The agent self-reports it; the broker stores the
latest per agent and surfaces it in the fleet record. Depends on Slice 1
([[BROKKR-T-0226]]) for the surface.

## Acceptance Criteria

## Acceptance Criteria

- [ ] Agent heartbeat payload (or a dedicated report) carries `k8s_reachable: bool`
      and optional `k8s_api_latency_ms: int`. Both optional — agents that cannot
      determine it omit them (graceful degradation).
- [ ] Broker stores the latest snapshot per agent. Decide storage: nullable
      columns on `agents` vs a small `agent_operational_status` table (1 row/agent).
      Migration with verified up/down (`angreal models migrations`).
- [ ] Agent side: collect reachability (e.g. a lightweight K8s API healthz/version
      probe on the heartbeat cycle) and include it in the heartbeat.
- [ ] `GET /api/v1/fleet` and the per-agent detail include `k8s_reachable`
      (+ latency if present); `null`/absent when never reported.
- [ ] OpenAPI + SDKs regenerated; drift checks pass. Reference docs updated.
- [ ] Tests: integration test — an agent reporting `k8s_reachable=false` surfaces
      in `/fleet`; an agent that never reports shows null without breaking the
      rollup.

## Implementation Notes

- Keep "trust the agent" (I-0027 non-goal: don't validate agent-reported data).
- This is deliberately standalone/small — it should not block Slice 1 shipping.

## Status Updates

*To be added during implementation*
## Status Updates

- 2026-06-13: IMPLEMENTED + verified (folded into PR #64 with T-0226). Build (broker+agent+models) + clippy (workspace, warning-free) + all 3 OpenAPI/SDK drift checks pass; integration test passes against a real DB. Migration #20 (T-0227) is additive nullable columns (trivially reversible). NOTE: the implementation agent hung on `angreal models migrations` (full-stack --wait); verification was completed by hand via build/clippy/drift + targeted integration tests (which apply the migration).
