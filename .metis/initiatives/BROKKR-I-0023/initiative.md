---
id: agent-reconciler-hardening-crash
level: initiative
title: "Agent reconciler hardening: crash loops, prune safety, liveness"
short_code: "BROKKR-I-0023"
created_at: 2026-06-11T11:01:39.384531+00:00
updated_at: 2026-06-11T15:11:06.583906+00:00
parent: brokkr-environment-aware
blocked_by: []
archived: false

tags:
  - "#initiative"
  - "#initiative"
  - "#phase/completed"


exit_criteria_met: false
estimated_complexity: M
initiative_id: agent-reconciler-hardening-crash
---

# Agent reconciler hardening: crash loops, prune safety, liveness

## Context

Pre-0.7.0 quality sweep (2026-06-11) of `crates/brokkr-agent`. The agent runs unattended in customer clusters, so panics, silent error-swallowing, and unsafe pruning are the top risk classes. The sweep (all main-loop files read end-to-end) found: crash-loop paths (one malformed bundle kills the agent and restarts refetch it; `.expect` on API discovery on every reconcile tick), a prune path that can delete a live resource whose replacement was silently skipped, the WS push channel dead on the agent side (`take_inbound` has no callers — verified), and a single-threaded control loop where a 15-minute build work order starves heartbeats until the broker marks the agent offline.

## Goals & Non-Goals

**Goals:**
- No single bad input (malformed bundle, API-server blip, unresolvable GVK) can crash or wedge the agent.
- Prune never deletes a resource whose replacement failed to apply, never touches objects it doesn't own, and works under namespace-scoped RBAC.
- Broker→agent WS push frames actually trigger reconciliation.
- Heartbeats/health continue during long-running operations.

**Non-Goals:**
- New agent features or protocol changes; broker-side fixes (BROKKR-I-0024).
- Making every hardcoded tunable configurable (only the ones called out in T-0206).

## Detailed Design

Five tasks: crash-loop fixes (T-0202), prune safety (T-0203), WS inbound wiring (T-0204), control-loop liveness (T-0205), misc hardening checklist (T-0206). File:line specifics are in each task.

## Alternatives Considered

- Catch-all panic handler around the main loop instead of fixing individual sites — rejected: masks bugs and leaves wedged state; the individual fixes are small.
- Removing the WS channel instead of wiring inbound — rejected: broker-side push (`ws/push.rs`) already works and the uplink is in use; finishing the loop is cheap and removes a buffer-wedge hazard.

## Implementation Plan

T-0202 and T-0203 first (crash loop + data loss are the release-gating risks), then T-0204/T-0205, then T-0206. Tests: unit where pure, integration (`angreal tests integration brokkr-agent`) for loop behavior.