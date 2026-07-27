---
id: observability-docs-promise-checks
level: task
title: "Observability docs promise checks and states that don't exist (readyz hardcoded 200, diagnostics 'failed' unreachable, EventInfo schema mismatch, work-order success not monitored)"
short_code: "BROKKR-T-0291"
created_at: 2026-07-27T14:27:55.830337+00:00
updated_at: 2026-07-27T14:27:55.830337+00:00
parent: 
blocked_by: []
archived: false

tags:
  - "#task"
  - "#phase/backlog"
  - "#bug"


exit_criteria_met: false
initiative_id: NULL
---

# Observability docs promise checks and states that don't exist (readyz hardcoded 200, diagnostics 'failed' unreachable, EventInfo schema mismatch, work-order success not monitored)

## Objective

Fix observability documentation that describes fictional failure detection (2026-07-27 review; `docs/REVIEW-2026-07-27.md`):

1. **reference/health-endpoints.md + how-to/monitoring-setup.md**: broker `/readyz` is documented as detecting DB connectivity problems; it is hardcoded `200 "Ready"` with zero checks. A broker with a dead database keeps reporting Ready and receiving traffic — operators must not build alerting/routing on this. (Consider a code follow-up: real readiness check.)
2. **reference/diagnostics.md + how-to/diagnostics.md**: the `failed` status is unreachable — no code path calls the DAL `fail()`; agent collection errors submit an error result and the request completes. Operators watching for `failed` get a "successful" result whose events array is `[{"error": ...}]`.
3. **reference/diagnostics.md schema**: documents `involved_object_kind`/`involved_object_name`; the agent's `EventInfo` has a single `involved_object` (name only). Consumers parsing by the documented schema find fields that never exist.
4. **explanation/work-orders.md**: claims custom work orders "apply the YAML and monitor completion"; the agent server-side-applies and immediately reports success — a failing migration Job still logs a successful work order.
5. **explanation/data-flows.md retention table**: claims agent events are "Permanent / Soft delete only"; a background task hard-deletes them after `agent_events_retention_days` (default 30) hourly; the 6h k8s-events/pod-logs telemetry windows are also missing from the table.

## 2026-07-27 — verification: /readyz

Substance confirmed, **citation in this ticket is wrong**: the handler is `api/mod.rs:302-304` (`async fn readyz() -> impl IntoResponse { (StatusCode::OK, "Ready") }`), not `api/v1/health.rs` (that file is deployment-health reporting). `healthz()` at `:290-292` is identically hardcoded. Both sit on the root router outside the auth middleware — no `State`, no DB touch.

Chart wiring (`charts/brokkr-broker/templates/deployment.yaml`): liveness → `/healthz` (:39-45), readiness → `/readyz` (:47-53, delay 10s, period 5s, timeout 3s). Neither probe is configurable from `values.yaml`.

**The broker is the outlier — the agent already does this correctly**: `brokkr-agent/src/health.rs:98-110` has `/readyz` call `apiserver_version()` and return 503 on failure. A working reference pattern ships in the same repo.

**Concrete risk**: readiness gates Service endpoints, so a broker with a dead/unreachable Postgres or an exhausted pool stays in `Endpoints` and keeps taking traffic that 500s; rolling updates always report success (a new ReplicaSet with a bad `BROKKR__DATABASE__URL` reaches 100% and tears down the healthy pods, with no rollback signal); `kubectl rollout status` and CD gates that wait on readiness are decorative.

**RECOMMENDATION: implement it, tightly scoped.** `/readyz` takes `State(dal)` (trivial — root routes are already `Router<DAL>`), does a pool checkout + `SELECT 1`, with a short-TTL cached result (~2-3s) so a 5s-per-pod probe can't hammer or DoS the DB. Critical detail: the r2d2 pool is built with only `.max_size` (`db.rs:42-59`), so a plain `pool.get()` would wait the 30s default and blow the 3s probe timeout — the probe must use `try_get` or its own timeout. Leave `/healthz` process-only (a DB blip must never restart-loop the fleet), widen the readiness `failureThreshold`, and expect `tests/integration/api/health.rs:40` to need a DB-backed fixture. Do not check migrations per-probe — they run once at startup and abort on failure; a startup `AtomicBool` suffices if that signal is wanted.

## 2026-07-27 — verification: items 2, 3, 4

**Item 2 (`failed` unreachable) — CONFIRMED.** `DiagnosticRequestsDAL::fail()` exists (`dal/diagnostic_requests.rs:139-152`) with exactly one repo-wide caller: a DAL test. On a collection error the agent submits a *synthetic success* — `pod_statuses: "[]"`, `events: [{"error": "<msg>"}]` (`brokkr-agent/src/cli/commands.rs:665-687`) — and the broker unconditionally calls `.complete()`. An operator sees `status: "completed"` with the error text buried in a JSON-encoded string, indistinguishable from a healthy-but-empty result — which, per BROKKR-T-0299, is the *common* case. Alerting on `status == "failed"` gets zero hits, ever.
**Recommendation: do the doc correction now, but do NOT delete `failed` from the model.** BROKKR-T-0300 (requests stuck in `claimed` forever) proves the state machine needs a terminal failure path regardless. Wire `fail()` as a follow-up; deleting the state would be optimizing the docs for a bug.

**Item 3 (EventInfo schema) — CONFIRMED.** Docs document `involved_object_kind` and `involved_object_name` (`reference/diagnostics.md:211-212`, sample at :169); the struct has a single `involved_object: String` (`brokkr-agent/src/diagnostics.rs:117`) populated with the name only, defaulting to the literal `"unknown"`. Near-miss explanation: `kube_events.rs:226-231` builds a full `ObjectRef {api_version, kind, namespace, name, uid}` for the *streaming* telemetry path — the docs appear to describe that shape applied to the wrong struct.
**Recommendation: widen the struct rather than narrow the docs.** `event.involved_object.kind` is already available; it is a one-line change and kind-less event references are meaningfully less useful.

**Item 4 (work orders) — CONFIRMED, but only for `custom`.** `execute_custom_work_order` (`brokkr-agent/src/work_orders/mod.rs:269-331`) applies and returns success at apply time; `process_single_work_order` immediately completes with `success = true`. A Job in `ImagePullBackOff` logs "Successfully applied 1 resource(s)". **However `build` work orders genuinely do monitor completion** — `build::execute_build` watches a BuildRun through a 15-minute bounded poll (`work_orders/build.rs:103-195, 260-342`), so `explanation/work-orders.md:102` is accurate and only the blanket claim at :139 is false. **Any doc fix must preserve that asymmetry rather than flattening both.**
**Recommendation: document apply-only semantics for `custom` now; implement Job-completion monitoring as a scoped follow-up.** The build watcher is a working template, and the agent's spawned-task structure already tolerates blocking waits. Two constraints for that follow-up: the wait must be bounded *below* `claim_timeout_seconds` (default 3600, `brokkr-models/src/models/work_orders.rs:158-160`) or the broker's stale-claim reaper will re-dispatch a still-running order to another agent; and `deployment_health.rs` is NOT reusable here because custom work orders stamp no Brokkr annotations at all — it is a pattern reference, not a drop-in. Scope to `batch/v1 Job` (the documented example and the only kind with unambiguous terminal semantics); non-Job kinds stay apply-only and say so.

**Related tickets filed from this investigation:** BROKKR-T-0299 (pod attribution), BROKKR-T-0300 (claimed-forever leak).

**Citation correction:** `/readyz` is `api/mod.rs:302-304`, not `api/v1/health.rs` — see the readyz section above.

## DECISIONS (Dylan, 2026-07-27)

- **`/readyz`** — implement a real check (cached `SELECT 1` with `try_get`/short timeout; `/healthz` stays process-only). Ship the doc correction in the same release.
- **Diagnostics `failed`** — correct the docs now to describe the shipped completed-with-error-events behavior, but **keep `failed` in the model**; BROKKR-T-0300 (requests stuck in `claimed`) proves the state machine needs a terminal failure path. Wiring `fail()` is a follow-up.
- **`EventInfo`** — widen the struct to carry `kind` (available on the source `Event`, one line) rather than narrowing the docs; kind-less event references are meaningfully less useful.
- **Work orders (item 4)** — **implement Job completion monitoring FIRST, then write the docs once against final behavior** (Dylan chose code-first over doc-first). Split out to **BROKKR-T-0303** so the rest of this ticket can close without waiting on agent work. This ticket keeps only the doc correction that must accompany that change, and must preserve the build/custom asymmetry.

## Backlog Item Details

### Type
- [x] Bug - Production issue that needs fixing (operators will build alerting on fictional signals)

### Priority
- [x] P1 - High

## Acceptance Criteria

- [ ] Each of the five claims corrected to code truth (or the code upgraded and docs kept — record the per-item decision here).
- [ ] readyz limitations stated wherever readiness probes are configured (charts reference it).
- [ ] Diagnostics error-surfacing pattern (completed + error events payload) documented with an example.

## Status Updates

*To be added during implementation*
