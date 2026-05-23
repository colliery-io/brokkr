---
id: ws-10-rest-history-endpoints-for
level: task
title: "WS-10: REST history endpoints for events/logs (6h window) + OpenAPI + SDK regen"
short_code: "BROKKR-T-0165"
created_at: 2026-05-23T02:12:43.666166+00:00
updated_at: 2026-05-23T02:12:43.666166+00:00
parent: BROKKR-I-0019
blocked_by: []
archived: false

tags:
  - "#task"
  - "#phase/todo"


exit_criteria_met: false
initiative_id: BROKKR-I-0019
---

# WS-10: REST history endpoints for events/logs (6h window) + OpenAPI + SDK regen

**Parent**: [[BROKKR-I-0019]] · **ADR**: [[BROKKR-A-0008]] · **Stance**: [[project_log_retention_stance]]

## Objective

Add paginated REST history endpoints for events and logs, scoped to the 6-hour retention window. Update the OpenAPI spec, regenerate all three SDKs (per [[project_release_versioning]]), and surface the retention ceiling explicitly in response metadata.

## Acceptance Criteria

- [ ] `GET /v1/stacks/{id}/events` and `GET /v1/stacks/{id}/logs` with pagination, time-range filter, PAK-scoped to readable stacks
- [ ] Responses include retention metadata: `retention_ceiling_seconds: 21600`, `effective_retention_seconds: <stack-config>`, `oldest_available_ts`
- [ ] Response examples / docs include explicit "for long-term retention, use Datadog" guidance
- [ ] `angreal openapi export` regenerates `openapi/brokkr-v1.json`
- [ ] `angreal openapi check` passes (spec is fresh)
- [ ] `angreal openapi gen-python`, `gen-typescript` regenerate SDKs; Rust SDK contract tests updated
- [ ] SDK contract tests cover the new endpoints (extend BROKKR-T-0154 work)

## Implementation Notes

- **Approach**: standard Axum handlers backed by the DAL from WS-09. Use existing pagination helpers; reuse PAK scoping middleware.
- **Dependencies**: WS-09.
- **Risk**: pagination across a constantly-evicting table. Use timestamp-based cursors, not offsets, and accept that pages may shrink between requests as old rows evict.