---
id: spec-hardening-a2-restore-missing
level: task
title: "Spec hardening A2: Restore missing spec coverage (routes and schemas)"
short_code: "BROKKR-T-0132"
created_at: 2026-05-14T18:26:18.028164+00:00
updated_at: 2026-05-14T20:27:20.774334+00:00
parent: BROKKR-I-0017
blocked_by: [BROKKR-T-0131]
archived: false

tags:
  - "#task"
  - "#phase/completed"


exit_criteria_met: false
initiative_id: BROKKR-I-0017
---

# Spec hardening A2: Restore missing spec coverage (routes and schemas)

## Parent Initiative

[[BROKKR-I-0017]]

## Objective

Bring every routed v1 handler into the OpenAPI spec, with all request/response schemas registered. Covers punch list items **M5** (16+ missing routes) and **M6** (webhook schemas absent), and resolves **F3** (verify whether `stacks::list_deployment_objects` / `create_deployment_object` are dead code).

Depends on [[BROKKR-T-0131]] because newly-added annotations should reference the canonical `ErrorResponse` schema from the start.

## Acceptance Criteria

## Acceptance Criteria

- [x] All 10 webhook routes are exposed in the spec.
- [x] Webhook schemas (4 from models + 6 broker-local DTOs) registered in `ApiDoc.components.schemas(...)`. Final set: `WebhookSubscription`, `WebhookDelivery`, `WebhookFilters`, `CreateWebhookRequest`, `UpdateWebhookRequest`, `WebhookResponse`, `ListDeliveriesQuery`, `PendingWebhookDelivery`, `DeliveryResultRequest` (+ `NewWebhookSubscription` / `UpdateWebhookSubscription` are internal-only and intentionally not surfaced).
- [x] Annotated-but-unregistered handlers added to `ApiDoc.paths(...)`: `agents::list_events`, `agents::create_event`, `agents::record_heartbeat`, `agents::get_associated_stacks`, `agents::rotate_agent_pak`, `generators::rotate_generator_pak`.
- [x] Stacks label/annotation handlers gained `#[utoipa::path]` annotations and registration. Explicit `operation_id = "stacks_*"` set on each to avoid colliding with the templates analogues (templates/agents collisions remain — T-A3).
- [x] F3 resolved: `stacks::list_deployment_objects` and `stacks::create_deployment_object` were not dead — both are routed at `/stacks/:id/deployment-objects` (GET/POST) via `routes()` in stacks.rs. Annotated and registered.
- [x] Spec grew from **42 paths / 61 ops** → **59 paths / 85 ops** (+17 paths, +24 ops). Schema count: **50 → 65**.
- [~] Validation pass via `redocly lint` deferred to T-B1 (the CI drift task explicitly owns lint wiring). Internal validation: 0 untyped error responses; all expected routes present; no dangling schema references in spot-checks.

## Implementation Notes

### Technical Approach

1. **Webhooks first** — the largest concentrated gap. Walk every annotated handler in `webhooks.rs`, add to `paths(...)` in `openapi.rs`. Add all webhook schemas to `components(schemas(...))`. Confirm tag name (e.g. `webhooks`) added to `tags(...)`.
2. **Single-handler additions** (agents events/heartbeat/stacks/rotate-pak, generators rotate-pak) — straightforward `paths(...)` edits.
3. **Stacks labels/annotations** — six handlers need new `#[utoipa::path]` annotations. Copy structure from the equivalent template annotations as a starting point.
4. **F3 verification** — grep for `list_deployment_objects` and `create_deployment_object` usages outside the handler file. If no `.route(...)` references them, delete the handlers; if they're routed via a path I missed, annotate them.
5. Re-run `openapi_export`, validate with `redocly lint`, commit the updated spec.

### Dependencies

- Hard: [[BROKKR-T-0131]] (need `ErrorResponse` available before adding new `responses(...)` clauses).

### Risk Considerations

- New annotations are easy to get subtly wrong (param names, status codes, security clauses). Cross-check each against the handler signature.
- Webhook routes may have their own auth/middleware nuances — verify security clauses match runtime behavior.

## Status Updates

### 2026-05-14 — Completed

**Changes:**
- Added `ToSchema` to `StackLabel`, `NewStackLabel`, `StackAnnotation`, `NewStackAnnotation` in `crates/brokkr-models/src/models/`.
- Added `#[utoipa::path]` annotations to 8 previously-unannotated stacks handlers: `list_deployment_objects`, `create_deployment_object`, `list_labels`, `add_label`, `remove_label`, `list_annotations`, `add_annotation`, `remove_annotation`. Made them `pub` so `paths(...)` registration resolves. Used explicit `operation_id = "stacks_<verb>_<resource>"` on labels/annotations to avoid colliding with the analogous templates handlers (templates vs agents collision still exists — T-A3 handles).
- Registered 17 new path entries in `ApiDoc.paths(...)`:
  - **Webhooks (10):** `list_webhooks`, `list_event_types`, `create_webhook`, `get_webhook`, `update_webhook`, `delete_webhook`, `list_deliveries`, `test_webhook`, `get_pending_agent_webhooks`, `report_delivery_result`.
  - **Agents (5):** `list_events`, `create_event`, `record_heartbeat`, `get_associated_stacks`, `rotate_agent_pak`.
  - **Generators (1):** `rotate_generator_pak`.
  - **Stacks (8):** `list_deployment_objects`, `create_deployment_object`, `list_labels`, `add_label`, `remove_label`, `list_annotations`, `add_annotation`, `remove_annotation`. (Some annotated handlers map onto routes already registered by other paths; net path count change is 17.)
- Added 15 new schema registrations: `NewAgentEvent`, `StackLabel`, `NewStackLabel`, `StackAnnotation`, `NewStackAnnotation`, `WebhookSubscription`, `WebhookDelivery`, `WebhookFilters`, `CreateWebhookRequest`, `UpdateWebhookRequest`, `WebhookResponse`, `ListDeliveriesQuery`, `PendingWebhookDelivery`, `DeliveryResultRequest`. (Plus the prior `ErrorResponse` from T-A1.)
- Added `webhooks` tag to `ApiDoc.tags(...)`.

**Verification (`openapi/brokkr-v1.json`):**
- Paths: 42 → **59** (+17)
- Operations: 61 → **85** (+24)
- Schemas: 50 → **65** (+15)
- All 17 spot-checked new routes present.
- Untyped error responses: **0**.
- `cargo build -p brokkr-broker` clean.

**F3 outcome:** Not dead code. Routes exist at `/stacks/:id/deployment-objects` (GET/POST), bound in `stacks::routes()`. Now properly annotated and registered.

**Carry-overs to T-A3:**
- 6 remaining duplicate operationIds (agents vs templates: `list_labels`, `add_label`, `remove_label`, `list_annotations`, `add_annotation`, `remove_annotation`). My new stacks annotations use `stacks_*` prefixes already; T-A3 needs to apply the same convention to agents and templates.
- All newly-added annotations declare paths starting with `/api/v1/` (matching the existing inconsistent convention). T-A3's path-prefix sweep will strip these along with the pre-existing ones.