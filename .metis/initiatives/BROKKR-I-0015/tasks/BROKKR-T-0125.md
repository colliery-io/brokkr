---
id: domain-6-reconciliation-templates
level: task
title: "Domain 6: Reconciliation, Templates and Stack Management Verification"
short_code: "BROKKR-T-0125"
created_at: 2026-03-13T14:01:19.383173+00:00
updated_at: 2026-03-13T14:11:36.783070+00:00
parent: BROKKR-I-0015
blocked_by: []
archived: false

tags:
  - "#task"
  - "#phase/completed"


exit_criteria_met: false
initiative_id: BROKKR-I-0015
---

# Domain 6: Reconciliation, Templates and Stack Management Verification

## Parent Initiative

[[BROKKR-I-0015]] — Documentation Validation Against Implementation

## Objective

Verify every claim about template rendering, JSON Schema validation, stack lifecycle operations, and the agent reconciliation loop against the actual Tera integration code, schema validation logic, stack CRUD handlers, and agent reconciliation implementation. These are core user-facing workflows — errors here directly mislead users trying to deploy workloads.

## Documentation Files in Scope

- `docs/content/how-to/templates.md` (~380 lines) — Tera syntax, JSON Schema validation, PostgreSQL StatefulSet walkthrough
- `docs/content/how-to/managing-stacks.md` (~284 lines) — stack creation, updates, lifecycle
- `docs/content/how-to/understanding-reconciliation.md` (~164 lines) — agent reconciliation behavior

## Source of Truth

- Template rendering code — Tera engine initialization, custom filters/functions, context variable injection
- JSON Schema validation code — how schemas are loaded, validated, error reporting
- Stack CRUD handlers in the broker — create, update, delete, list operations
- Agent reconciliation loop — polling interval, diff logic, apply/delete behavior, error handling, retry logic
- Deployment object state machine — how objects move through states during reconciliation

## Verification Checklist

### Template Rendering
For each documented Tera syntax feature:
- [ ] The syntax is supported by the version of Tera in use
- [ ] Custom filters (if documented) exist in the codebase
- [ ] Custom functions (if documented) exist in the codebase
- [ ] Context variables available during rendering match documentation
- [ ] Error messages shown in documentation match actual Tera error output

For each template example:
- [ ] The example would actually render correctly given the documented context
- [ ] The rendered output shown matches what Tera would produce
- [ ] Any JSON Schema validation shown actually works with the described schema

### JSON Schema Validation
- [ ] Schema format described matches what the code expects
- [ ] Validation timing (when in the pipeline validation occurs) matches code
- [ ] Error response format for validation failures matches implementation
- [ ] Any documented schema constraints (required fields, types, patterns) are enforced by code

### Stack Management
For each documented stack operation:
- [ ] API endpoint for the operation matches the actual route
- [ ] Request/response shapes match (overlap with Domain 1, but focus here is on lifecycle semantics)
- [ ] Documented ordering/sequencing constraints match implementation
- [ ] Any documented cascade behaviors (e.g., deleting a stack deletes its deployment objects) match code
- [ ] Version/annotation behavior described matches implementation

### Reconciliation
- [ ] Default polling interval matches code constant
- [ ] Diff/comparison logic described matches actual implementation
- [ ] Apply behavior (create vs update detection) matches code
- [ ] Delete behavior (orphan cleanup) matches code
- [ ] Error handling and retry behavior matches code
- [ ] Event reporting during reconciliation matches actual event emission
- [ ] Any described ordering of operations during reconciliation matches code

## Verification Findings

### templates.md

| # | Claim | File/Line | Verdict | Details |
|---|-------|-----------|---------|---------|
| 1 | Tera syntax features (variables, defaults, conditionals, loops) | templating.rs tests | CORRECT | All syntax confirmed working in unit tests |
| 2 | `sha256` filter shown in filter example (line 120) | Tera 1.20 docs | **INCORRECT** | Tera 1.20 has NO `sha256` filter. Should use a real Tera built-in like `slugify`. **FIX: Replace `hash: {{ content \| sha256 }}` with `slug: {{ name \| slugify }}`** |
| 3 | Custom filters exist | templating.rs | CORRECT | No custom filters registered; uses stock Tera. Doc doesn't claim custom filters, just "built-in filters" |
| 4 | Context variables flattened from JSON params | templating.rs:110-117 | CORRECT | Object keys are iterated and inserted into Tera Context |
| 5 | JSON Schema validation at creation time | templates.rs:202,211 | CORRECT | Both Tera syntax and JSON Schema validated on create |
| 6 | Template instantiation workflow (labels->schema->render->create) | stacks.rs:964-1069 | CORRECT | Steps 1-4 match code exactly |
| 7 | 422 error for label mismatch with missing_labels/missing_annotations | stacks.rs:972-984 | CORRECT | Exact JSON shape matches |
| 8 | Parameter validation error format `{"error": "Invalid parameters", "validation_errors": [...]}` | stacks.rs:988-998 | CORRECT | Matches code |
| 9 | Template rendering error `"Template rendering failed: ..."` | templating.rs:119-122 | CORRECT | Error message is `"Template rendering failed: <details>"` |
| 10 | Tera syntax error `"Invalid Tera syntax: ..."` | templating.rs:67-70 | CORRECT | Matches code |
| 11 | JSON Schema error `"Invalid JSON Schema: ..."` | templating.rs:155-158 | CORRECT | Matches code |
| 12 | Generator access control (view system + own, modify own only) | templates.rs:78-86, 107-158 | CORRECT | Code confirms |
| 13 | Template versioning via update creating new version | templates.rs:392-413 | CORRECT | `create_new_version` called on update |
| 14 | API endpoints (POST/GET templates, PUT/DELETE templates/:id, labels, annotations) | templates.rs:54-72 | CORRECT | Routes match documented URLs |

### managing-stacks.md

| # | Claim | File/Line | Verdict | Details |
|---|-------|-----------|---------|---------|
| 15 | Stack creation via POST /api/v1/stacks | stacks.rs:37,125-178 | CORRECT | Endpoint and behavior match |
| 16 | Stack response format with id, name, description, timestamps, generator_id | Stack model | CORRECT | Fields present |
| 17 | Labels: POST/GET/DELETE endpoints, plain string body | stacks.rs:50-51, 539-638 | CORRECT | Routes and body format match |
| 18 | Annotations: POST/GET/DELETE endpoints, key-value body with stack_id | stacks.rs:52-57, 688-807 | CORRECT | Routes match. Note: annotation creation requires `stack_id` in body matching path |
| 19 | Direct targeting via POST /api/v1/agents/$AGENT_ID/targets | stacks.rs:148-154 (doc ref) | CORRECT | Referenced endpoint exists in agent routes |
| 20 | Stack update via PUT /api/v1/stacks/:id with id in body | stacks.rs:259-328 | CORRECT | Code validates id match |
| 21 | Soft deletion sets deleted_at | stacks.rs dal:149-165 | CORRECT | `soft_delete` sets `deleted_at` |
| 22 | Soft deletion cascades: "All deployment objects soft-deleted" and "deletion marker created" (line 251-253) | stacks.rs dal:149-165 | **INCORRECT** | `soft_delete` ONLY sets `deleted_at` on the stack itself. It does NOT cascade soft-delete to deployment objects and does NOT create a deletion marker. The cascade behavior described is aspirational, not implemented. **FIX: Remove claims about cascading soft-deletion and deletion marker creation on stack delete, or clarify this is expected future behavior** |
| 23 | Deployment object creation via POST with yaml_content and is_deletion_marker | stacks.rs:455-514 | CORRECT | Code extracts these fields |
| 24 | Template instantiation via POST .../from-template | stacks.rs:47-49, 845-1070 | CORRECT | Route and handler match |

### understanding-reconciliation.md

| # | Claim | File/Line | Verdict | Details |
|---|-------|-----------|---------|---------|
| 25 | Default polling interval: 30 seconds (line 14, 93-95) | default.toml:38 | **INCORRECT** | Default is `10` seconds, not 30. **FIX: Change "30 seconds" to "10 seconds" in both the prose (line 14) and the YAML example (line 93)** |
| 26 | Agent fetches via GET /api/v1/agents/{id}/target-state | broker.rs:199-201 | CORRECT | URL matches |
| 27 | Deployment objects ordered by sequence ID | deployment_objects.rs:256 | CORRECT | Sorted by sequence_id desc |
| 28 | Checksum used as version identifier in annotations | objects.rs:47,86-88 | CORRECT | `CHECKSUM_ANNOTATION` applied |
| 29 | Annotation names: `brokkr.io/stack-id` and `brokkr.io/checksum` (lines 48-49) | objects.rs:44,47 | **INCORRECT** | Actual annotation keys are `k8s.brokkr.io/stack` and `k8s.brokkr.io/deployment-checksum`. **FIX: Update to `k8s.brokkr.io/stack` and `k8s.brokkr.io/deployment-checksum`** |
| 30 | Priority resource application (Namespaces, CRDs first) | api.rs:688-729 | CORRECT | Partitioned and applied first |
| 31 | Dry-run validation of remaining objects | api.rs:732-743 | CORRECT | `validate_k8s_objects` uses dry-run apply |
| 32 | Server-side apply with force enabled | api.rs:777,594 | CORRECT | `PatchParams::apply("brokkr-controller")` with `force = true` |
| 33 | Pruning by checksum mismatch | api.rs:811-873 | CORRECT | Deletes objects where checksum != current |
| 34 | Owner references skip during pruning | api.rs:832-838 | CORRECT | Skips if owner_references non-empty |
| 35 | Namespace rollback on failure | api.rs:624-658,724-728,739,795 | CORRECT | `rollback_namespaces` called on any error |
| 36 | No resource rollback for individual resources | api.rs (general) | CORRECT | Only namespaces rolled back |
| 37 | Failure event sent to broker on error | commands.rs:255-265 | CORRECT | `send_failure_event` called |
| 38 | Retry config: initial=1s, max=60s, elapsed=5min, multiplier=2.0 | api.rs:74-83 | CORRECT | Exact values match |
| 39 | Retryable errors: 429, 500, 503, 504 | api.rs:86-97 | CORRECT | Plus reason-string matches |
| 40 | Deletion markers: agent identifies stack resources and deletes them | commands.rs:221-265 | PARTIALLY CORRECT | The agent processes deletion markers through the same reconcile path. When `is_deletion_marker` is true, the objects list would be empty, causing all existing stack resources to be pruned. The description is conceptually correct. |
| 41 | YAML example shows `polling_interval: 30` | line 93 | **INCORRECT** | Should be `polling_interval: 10` to match default. **FIX: Change value to 10** |
| 42 | Inactive agents skip deployment object requests | commands.rs:215-219 | CORRECT | Checks `agent.status != "ACTIVE"` |

### Summary of Issues Found

**4 issues requiring fixes:**

1. **templates.md line 120**: `sha256` is not a Tera built-in filter. Replace with a real filter like `slugify`.
2. **managing-stacks.md lines 251-253**: Stack soft-delete does NOT cascade to deployment objects and does NOT create deletion markers. The `soft_delete` method only sets `deleted_at` on the stack row.
3. **understanding-reconciliation.md lines 14, 93-95**: Default polling interval is 10 seconds (per `default.toml`), not 30.
4. **understanding-reconciliation.md lines 48-49**: Annotation keys are `k8s.brokkr.io/stack` and `k8s.brokkr.io/deployment-checksum`, not `brokkr.io/stack-id` and `brokkr.io/checksum`.

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] Every Tera syntax claim verified against the Tera version and custom extensions in use
- [ ] Every template example verified for correctness
- [ ] Every JSON Schema claim verified against validation code
- [ ] Every stack lifecycle operation verified against handlers
- [ ] Every reconciliation behavior claim traced through the agent's reconciliation loop
- [ ] All findings recorded using verdict taxonomy
- [ ] All non-CORRECT findings fixed in documentation

## Findings Report

*To be populated during verification. Use this format:*

```
### [filename.md]
| Line/Section | Claim | Verdict | Source File | Notes |
|---|---|---|---|---|
| ... | ... | ... | ... | ... |
```

## Status Updates

*To be added during implementation*