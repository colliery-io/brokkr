---
id: docs-named-paks-get-api-v1-paks
level: task
title: "Docs: named PAKs, GET /api/v1/paks, and pak_id view-filter semantics missing from docs/src"
short_code: "BROKKR-T-0277"
created_at: 2026-07-27T14:13:06.593229+00:00
updated_at: 2026-07-27T14:13:06.593229+00:00
parent: 
blocked_by: []
archived: false

tags:
  - "#task"
  - "#phase/backlog"
  - "#tech-debt"


exit_criteria_met: false
initiative_id: NULL
---

# Docs: named PAKs, GET /api/v1/paks, and pak_id view-filter semantics missing from docs/src

## Objective

Document the tenant-listing/scoping surface added in PR #87 (BROKKR-I-0032). As of 2026-07-27, `grep -rn "pak_id\|/paks" docs/src` returns zero hits: the endpoint, the query parameter, and the "named PAK" concept are entirely undocumented.

## Backlog Item Details

### Type
- [x] Tech Debt - Code improvement or refactoring (documentation debt)

### Priority
- [x] P1 - High (important for user experience)

### Technical Debt Impact
- **Current Problems**: PR #87 added, with no doc sync:
  - `GET /api/v1/paks` (admin-gated; the read-only UI PAK qualifies) returning non-system generators as slim `[{id, name}]` — "named PAKs" ARE generators/tenants (ADR-0009; `crates/brokkr-broker/src/api/v1/paks.rs:7-13`). Note the route is `/paks`, only OpenAPI-*tagged* "auth" — easy for a doc author to mis-state as `/auth/paks`.
  - `?pak_id=<generator uuid>` accepted by `GET /fleet`, `GET /stacks`, `GET /agent-events` to narrow results to one tenant. Explicitly a **view filter, not an authorization boundary** (`paks.rs:29-37`, I-0032 non-goal); unknown id → empty 200, malformed id → 400.
  - `AuthResponse.readonly` field on `POST /auth/pak` introspection.
- **Benefits of Fixing**: reference/multi-tenancy.md can state precisely what tenant scoping does and does not guarantee; consumers wiring dashboards/SDKs get the endpoint and parameter; nobody mistakes view scoping for isolation.
- **Risk Assessment**: The overclaim hazard is the sharp edge — if consumers infer that `pak_id` scoping is security isolation, they will build multi-team setups on a guarantee that does not exist. Real isolation is the generator-PAK handler enforcement, and the docs must draw that line.

### Scope expansion (2026-07-27, per Dylan)
Schema-per-tenant is **not** the tenancy model — tenants live at the generator/agent level. But `how-to/multi-tenant-setup.md` is built entirely around broker-per-tenant + PostgreSQL schemas and explicitly states "Schema-per-tenant isolation does not require generators," and `reference/multi-tenancy.md` presents schema isolation as the headline mechanism. This task should therefore also **reframe both pages**: generator-level tenancy (create generator → PAK → register agents → scoped views via named PAKs/`pak_id`) becomes the primary multi-tenant onboarding story on one broker; schema isolation gets demoted to a "running fully separate instances" deployment note (it's a real config feature, just not the tenant model). Coordinate with BROKKR-T-0287 — the how-to must not promise isolation that label/annotation matching doesn't currently enforce.

### Content to write (suggested placement)
- reference/multi-tenancy.md: named-PAK/tenant model (tenant = generator), `GET /api/v1/paks` contract, `pak_id` semantics with the view-filter-not-authz warning, `readonly` introspection field.
- reference/api landing + regenerate anything derived from `openapi/brokkr-v1.json` (spec already gained the endpoint in #87).
- how-to/multi-tenant-setup.md: how the scope selector/tenant filtering fits an actual multi-team install.

## Acceptance Criteria

- [ ] `GET /api/v1/paks` documented with auth requirements and exact path.
- [ ] `pak_id` documented on all three list endpoints with an explicit "view filter, not an authorization boundary" statement.
- [ ] Tenant = generator (ADR-0009) stated in reference/multi-tenancy.md and consistent with how-to/multi-tenant-setup.md.
- [ ] `readonly` field of `POST /auth/pak` response documented.

## Status Updates

*To be added during implementation*
