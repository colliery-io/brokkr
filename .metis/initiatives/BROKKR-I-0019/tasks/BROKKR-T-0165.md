---
id: ws-10-rest-history-endpoints-for
level: task
title: "WS-10: REST history endpoints for events/logs (6h window) + OpenAPI + SDK regen"
short_code: "BROKKR-T-0165"
created_at: 2026-05-23T02:12:43.666166+00:00
updated_at: 2026-05-23T10:57:13.758099+00:00
parent: BROKKR-I-0019
blocked_by: []
archived: false

tags:
  - "#task"
  - "#phase/completed"


exit_criteria_met: false
initiative_id: BROKKR-I-0019
---

# WS-10: REST history endpoints for events/logs (6h window) + OpenAPI + SDK regen

**Parent**: [[BROKKR-I-0019]] · **ADR**: [[BROKKR-A-0008]] · **Stance**: [[project_log_retention_stance]]

## Objective

Add paginated REST history endpoints for events and logs, scoped to the 6-hour retention window. Update the OpenAPI spec, regenerate all three SDKs (per [[project_release_versioning]]), and surface the retention ceiling explicitly in response metadata.

## Acceptance Criteria

## Acceptance Criteria

- [x] `GET /v1/stacks/{id}/events` and `GET /v1/stacks/{id}/logs` with `since` + `limit` query params, PAK-scoped via the existing `fetch_owned_stack` helper.
- [x] Responses include retention metadata: `retention_ceiling_seconds: 21600`, `effective_retention_seconds: 21600`, `oldest_available_ts: <iso8601|null>` — surfaced as a top-level `retention` object on both endpoints.
- [x] Response includes an explicit `long_term_sink_hint: "Brokkr retains telemetry for at most 6 hours. For long-term log centralisation, ship to Datadog or equivalent."` per ADR NFR-007.
- [x] `angreal openapi export` regenerates `openapi/brokkr-v1.json` (238297 bytes).
- [x] `angreal openapi check` passes (spec is fresh).
- [x] `angreal openapi gen-python` and `gen-typescript` regenerate SDKs; `check-python` / `check-typescript` both green. Rust SDK auto-regenerates from the in-crate spec mirror via `progenitor::generate_api!`.
- [ ] Dedicated SDK contract tests for the new endpoints deferred — the BROKKR-T-0154 harness covers the general pattern; the next contract-test pass should add fixtures for the new history shape.

## Implementation Notes

- **Approach**: standard Axum GET handlers in `api/v1/stacks.rs`, backed by `dal.agent_k8s_events().list_for_stack(...)` and `dal.agent_pod_logs().list_for_stack(...)` (added in WS-09). Reuses `fetch_owned_stack` for PAK scoping.
- **Dependencies**: WS-09 (DAL + tables).
- **Risk**: pagination across a constantly-evicting table. Mitigated by clamping `since` to the retention ceiling and returning `oldest_available_ts` so callers can detect when a page tail was evicted between requests. Cursor-style pagination (since-token) is left as a follow-up; today's `since + limit + DESC by created_at` is sufficient for the 6h window.

## Status Updates

**2026-05-23** — Done on branch `feat/i-0019-ws-broker-agent-channel`.

- New endpoints in `api/v1/stacks.rs`:
  - `GET /api/v1/stacks/{id}/events` → `K8sEventHistoryResponse { retention, events }`
  - `GET /api/v1/stacks/{id}/logs`   → `PodLogHistoryResponse  { retention, lines }`
  - Both accept `?since=<iso8601>` (clamped to 6h ago) and `?limit=<n>` (defaults 500, capped 5000).
  - Tag `stack-telemetry` in the OpenAPI spec.
- `RetentionInfo` shape is identical across both endpoints — single source of truth for the retention messaging that the UI (WS-12) and SDK callers will render. `long_term_sink_hint` is a `&'static str` baked in at compile time so it can never silently drift.
- `AgentK8sEvent` and `AgentPodLog` model structs got `derive(ToSchema)` so they serialise into the OpenAPI components correctly. No semantic change to the models.
- OpenAPI components and `paths(...)` updated; `angreal openapi {export,check,gen-python,gen-typescript,check-python,check-typescript}` all green.
- Tests (`tests/integration/api/ws.rs`):
  - `rest_history_endpoints_return_retained_telemetry_with_retention_metadata` — seeds one event + one log line via DAL, GETs both endpoints, asserts payload shape + retention metadata + that the `Datadog` sink hint is present.
  - `rest_history_endpoints_403_for_unauthorized_callers` — foreign-generator PAK is rejected by the same scoping path the rest of the stack reads use.
- 11/11 `api::ws` integration tests now green.

**Deferred**:
- Cursor-style pagination — today's `since` + DESC + limit is enough for the 6h window. Add when a real consumer needs deep paging.
- Dedicated SDK contract tests for the two new endpoints — out of scope for this task; extend BROKKR-T-0154 fixtures in a follow-up.