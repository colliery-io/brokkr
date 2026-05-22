---
id: get-stacks-returns-403-to
level: task
title: "GET /stacks returns 403 to generator PAKs that own stacks"
short_code: "BROKKR-T-0155"
created_at: 2026-05-22T02:16:12.158209+00:00
updated_at: 2026-05-22T02:39:26.733772+00:00
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

# GET /stacks returns 403 to generator PAKs that own stacks

## Objective

A generator PAK that has created stacks cannot list them. `GET /api/v1/stacks` is currently hard-coded admin-only in the broker handler — any non-admin caller receives `403 admin_required`, including the generator that owns the stacks being listed. The deploy workflow ends up half-broken from the consumer's perspective: a generator can create, get-by-id, label, annotate, add deployment objects, and target its own stacks, but cannot enumerate them.

Fix: filter on the server side instead of denying. Admin sees all stacks; a generator PAK sees only stacks where `generator_id == auth.generator`; everyone else still gets 403.

## Reproduction

1. Create a generator (admin PAK), capture its PAK
2. As that generator: `POST /api/v1/stacks` → 201
3. As the same generator: `GET /api/v1/stacks` → 403 `{ "code": "admin_required" }`

## Affected Code

- `crates/brokkr-broker/src/api/v1/stacks.rs::list_stacks` — currently `if !auth_payload.admin { return Err(...forbidden(...)); }` then `dal.stacks().list()` (returns everything regardless).
- DAL likely already has a `list_for_generator(generator_id)` (used by `fetch_owned_stack` and friends) — verify before reaching for a new method.

## Backlog Item Details

- **Type**: Bug
- **Priority**: P1 — breaks the natural enumerate-my-resources flow for any generator-PAK consumer
- **Discovered**: 2026-05-22, by the maintainer using a generator PAK against the live broker

## Acceptance Criteria

## Acceptance Criteria

- [x] Admin PAK on `GET /api/v1/stacks` → 200 with all stacks (unchanged path)
- [x] Generator PAK on `GET /api/v1/stacks` → 200 with only stacks where `generator_id == auth.generator` (via new `dal.stacks().list_for_generator()`)
- [x] Any other caller (agent PAK, no scope) → 403 `ErrorResponse { code: "stacks_list_denied" }`
- [x] utoipa `security` updated to `(admin_pak, generator_pak)`; spec regen'd
- [x] Integration tests added: `test_list_stacks_with_generator_pak_filters_to_own` + `test_list_stacks_without_pak_forbidden`
- [x] SDK contract suites (rust/python/typescript) all extended with a `list_stacks` step asserting the generator sees own stack and nothing leaks from other generators

## Implementation Notes

This is the natural extension of the auth model landed in BROKKR-T-0153: a generator that can create/get/modify its own stacks should be able to enumerate them too. The shape mirrors `fetch_owned_stack`'s ownership gate — just at the collection level. Land alongside a follow-on review of other list endpoints (templates, generators, deployment-objects) to see if any have the same admin-only-by-default footgun.

## Test Cases **[CONDITIONAL: Testing Task]**

{Delete unless this is a testing task}

### Test Case 1: {Test Case Name}
- **Test ID**: TC-001
- **Preconditions**: {What must be true before testing}
- **Steps**: 
  1. {Step 1}
  2. {Step 2}
  3. {Step 3}
- **Expected Results**: {What should happen}
- **Actual Results**: {To be filled during execution}
- **Status**: {Pass/Fail/Blocked}

### Test Case 2: {Test Case Name}
- **Test ID**: TC-002
- **Preconditions**: {What must be true before testing}
- **Steps**: 
  1. {Step 1}
  2. {Step 2}
- **Expected Results**: {What should happen}
- **Actual Results**: {To be filled during execution}
- **Status**: {Pass/Fail/Blocked}

## Documentation Sections **[CONDITIONAL: Documentation Task]**

{Delete unless this is a documentation task}

### User Guide Content
- **Feature Description**: {What this feature does and why it's useful}
- **Prerequisites**: {What users need before using this feature}
- **Step-by-Step Instructions**:
  1. {Step 1 with screenshots/examples}
  2. {Step 2 with screenshots/examples}
  3. {Step 3 with screenshots/examples}

### Troubleshooting Guide
- **Common Issue 1**: {Problem description and solution}
- **Common Issue 2**: {Problem description and solution}
- **Error Messages**: {List of error messages and what they mean}

### API Documentation **[CONDITIONAL: API Documentation]**
- **Endpoint**: {API endpoint description}
- **Parameters**: {Required and optional parameters}
- **Example Request**: {Code example}
- **Example Response**: {Expected response format}

## Implementation Notes **[CONDITIONAL: Technical Task]**

{Keep for technical tasks, delete for non-technical. Technical details, approach, or important considerations}

### Technical Approach
{How this will be implemented}

### Dependencies
{Other tasks or systems this depends on}

### Risk Considerations
{Technical risks and mitigation strategies}

## Status Updates

- 2026-05-22: Filed after maintainer hit the 403 against the live v0.4.1 broker.
- 2026-05-22: Fixed. Added `dal.stacks().list_for_generator(generator_id)`; handler now mirrors `list_templates`: admin sees all, generator sees own, else 403 `stacks_list_denied`. utoipa `security` now `(admin_pak, generator_pak)`. Spec + Python + TS SDKs regen'd; drift checks clean.
- 2026-05-22: **Test lifecycle audit** — confirmed other list endpoints are correctly scoped:
  - `list_generators` admin-only (correct — generators must not enumerate each other)
  - `list_agents` admin-only (correct — generators target by ID, not by enumeration)
  - `list_templates` already filtered (admin all / generator own + system)
  - `list_stacks` was the broken one; now matches the templates shape.
- 2026-05-22: SDK contract suites (rust/python/typescript) extended with a generator-PAK `list_stacks` step (asserts own stack appears, no leaks). Broker integration tests added: filtered-list + agent-PAK-denied.