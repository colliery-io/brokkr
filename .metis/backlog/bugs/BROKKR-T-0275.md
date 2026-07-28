---
id: console-run-diagnostic-posts-to
level: task
title: "Console 'Run diagnostic' posts to nonexistent POST /api/v1/diagnostics and always 404s"
short_code: "BROKKR-T-0275"
created_at: 2026-07-27T14:13:03.788286+00:00
updated_at: 2026-07-28T00:30:51.415512+00:00
parent: 
blocked_by: []
archived: false

tags:
  - "#task"
  - "#bug"
  - "#phase/completed"


exit_criteria_met: false
initiative_id: NULL
---

# Console 'Run diagnostic' posts to nonexistent POST /api/v1/diagnostics and always 404s

## Objective

Fix the Operator Console's only write action: the "Run diagnostic" button in the Fleet agent-detail modal can never succeed because the client and broker disagree on the route.

## Backlog Item Details

### Type
- [x] Bug - Production issue that needs fixing

### Priority
- [ ] P0 - Critical (blocks users/revenue)
- [x] P1 - High (important for user experience)

### Impact Assessment
- **Affected Users**: Every console user who tries to trigger a diagnostic (the console's single mutating feature, allowlisted for the read-only UI PAK precisely so it would work).
- **Reproduction Steps**:
  1. Open the console at `http://<broker>:3000/`, go to Fleet, open an agent's detail modal.
  2. Click "Run diagnostic".
  3. Observe the "diagnostic failed" toast; network tab shows `POST /api/v1/diagnostics` → 404.
- **Expected vs Actual**: Expected a diagnostic request to be created for the agent. Actual: the broker has no bare `POST /diagnostics` route — diagnostics are created via `POST /api/v1/deployment-objects/:id/diagnostics` only — so the request 404s unconditionally.

### Root Cause
- Client: `crates/brokkr-web/src/api.rs:191-194` posts `/api/v1/diagnostics` with body `{agent_id}`.
- Broker: `crates/brokkr-broker/src/api/v1/diagnostics.rs:30-44` routes only `POST /deployment-objects/:id/diagnostics`; the readonly-PAK POST allowlist (`api/v1/middleware.rs:160-177`, `readonly_request_allowed`) also only admits the deployment-object-scoped path, so even adding a bare route requires an allowlist decision.

Discovered during the 2026-07-27 full-docs review (workflow `wf_52547a72-dbf` ground-truth extraction); confirmed against code, not just docs.

## 2026-07-27 — verification and recommended shape

**Framing correction:** this ticket implies the readonly-PAK allowlist is an obstacle. It is not. The allowlist (`middleware.rs:160-181`) and the broker route are coherent, and `tests/integration/api/ui_pak.rs:140` already proves the UI PAK can create a diagnostic via `POST /deployment-objects/{id}/diagnostics`. **Only the console client is wrong.** The allowlist becomes a decision point only if an agent-scoped route is added.

**Structural fact:** a diagnostic request is inherently deployment-object-scoped — `deployment_object_id` is NOT NULL and `NewDiagnosticRequest::new` rejects a nil UUID (`brokkr-models/src/models/diagnostic_requests.rs:74-96`). An agent-scoped diagnostic requires a migration.

**Console data gap:** the Fleet modal holds only `FleetAgentRecord` (agent id, name, cluster, status, heartbeat, counts — `brokkr-web/src/models.rs:8-34`); there is no deployment-object id in that view.

**RECOMMENDATION — point the console at the existing route, with a picker.** Fetch `GET /agents/:id/target-state?mode=full` (admin-readable, passes for the UI PAK, no allowlist change) to populate a deployment-object picker in the modal, then POST to the existing DO-scoped route. Rejected alternative: adding an agent-scoped `POST /diagnostics` would require a migration plus a new allowlist entry that lets a *read-only, network-reachable* credential trigger unbounded cluster-wide pod-log collection — a materially worse security posture than the narrow entry deliberately scoped in BROKKR-I-0032, for the sake of one button.

**Two riders filed separately** (do not fold in): BROKKR-T-0299 (pod attribution bug — without it this button reliably returns empty pod data and looks broken) and BROKKR-T-0301 (console discards the response, so results are unreadable). Related: BROKKR-T-0300 (requests stuck in `claimed` forever).

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] Clicking "Run diagnostic" from the Fleet agent modal creates a diagnostic request and surfaces success in the UI.
- [ ] The chosen fix reconciles route, request body, and the readonly-PAK POST allowlist coherently (either the console targets an existing deployment-object-scoped route, or a deliberate agent-scoped diagnostics route is added and allowlisted).
- [ ] An integration test covers the console's diagnostic path with the UI PAK credential (extend `tests/integration/api/ui_pak.rs`).
- [ ] Docs describing the console's diagnostics capability (BROKKR-T-0276) match the shipped behavior.

## Status Updates

**2026-07-28 — FIXED** on branch `docs/tenancy-review-2026-07`, using the decided shape (console targets the existing DO-scoped route; no broker or allowlist change).

`api.rs` replaces the phantom `POST /api/v1/diagnostics` with `create_diagnostic(deployment_object_id, agent_id)` against `POST /deployment-objects/{id}/diagnostics`, and adds `agent_target_state(agent_id)` for the picker. `fleet.rs` renders a deployment-object picker in the agent modal driven by a selection-scoped `LocalResource` matching the `deployments.rs`/`webhooks.rs` idiom, with an honest empty state ("No deployment objects target this agent — nothing to diagnose") instead of a dead button. `models.rs` gains a `TargetStateObject` mirror whose label includes the globally-unique `sequence_id`, because `aurora_leptos::Select` uses one string as both option text and value and maps back by exact match.

**Toast deliberately de-overclaimed** to "diagnostic request created", with a caption stating results are not shown in the console yet — accurate until BROKKR-T-0301 lands.

**Latent trap found and closed for all callers:** gloo-net 0.6's `RequestBuilder::build` concatenates with `format!("{}&{}", url.search(), value.query)` when the URL already carries a query, so a hand-written `?mode=full` in a path would emit `?mode=full&`. GET calls now route query params through the builder via a shared `get_query` helper rather than embedding them in the path.

Test: `test_ui_pak_walks_console_diagnostic_path` in `ui_pak.rs`. The existing `test_ui_pak_can_create_diagnostics` already covered the POST leg, so that was not duplicated; what was uncovered — and is the genuinely new dependency this fix introduces — is the *read* the picker needs (`GET /agents/:id/target-state?mode=full` under the UI PAK), plus end-to-end chaining of the id from the read into the write. The test also asserts the DTO field shapes, guarding the console mirror.

**Build knowledge worth keeping:** `brokkr-web` is `exclude`d from the root workspace and has its own lockfile, so `cargo clippy -p brokkr-web` from the root does **not** work. Use `cd crates/brokkr-web && cargo clippy --all-targets --target wasm32-unknown-unknown`, and `trunk build` for the real artifact (what `docker/Dockerfile.broker` runs before building with `--features embed-ui`). There is no angreal task for either.

Verified: clippy warning count byte-identical to baseline (7 pre-existing `redundant_closure`), `trunk build` succeeds, integration target compiles.

**Acceptance criterion 4** (docs match shipped behavior) is deliberately deferred to BROKKR-T-0276, which owns the console page — the capability to document is now "pick a deployment object from the agent's target state, then POST to the DO-scoped route", including the empty-state behavior.