---
id: documentation-audit-onboarding
level: initiative
title: "Documentation Audit & Onboarding Restructure (Diátaxis)"
short_code: "BROKKR-I-0029"
created_at: 2026-06-14T16:01:38.094833+00:00
updated_at: 2026-06-14T16:08:50.701805+00:00
parent: BROKKR-V-0001
blocked_by: []
archived: false

tags:
  - "#initiative"
  - "#phase/active"


exit_criteria_met: false
estimated_complexity: L
initiative_id: documentation-audit-onboarding
---

# Documentation Audit & Onboarding Restructure (Diátaxis) Initiative

Full Diátaxis documentation audit of Brokkr, driven by the `/docs-diataxis` skill (four phases: Discovery → Plan → Write → gated Review).

## Context **[REQUIRED]**

Brokkr just shipped v0.8.0. The repo already has a mature mdBook docs tree (~150 files across getting-started, tutorials, how-to, explanation, reference, plus auto-generated rustdoc under `docs/src/api/rust/**`). This is therefore an **audit-and-extend**, not greenfield.

The user requested a **full-codebase audit** with two explicit priorities beyond raw coverage:
1. **Correctness** — every reference claim and how-to step must trace to a specific code location; no stale/invented claims.
2. **Organization for onboarding & usage** — the docs should be structured to get a newcomer productive as fast as possible.

Two article families are explicitly wanted:
- **Fast trial/consumption**: "how do I get Brokkr into my codebase ASAP to see if it's right for me" (kick-the-tires / evaluation path).
- **Fast common integration patterns**: the "safe consumption" patterns people actually need — **templates**, **push/apply (CLI)**, and **monitoring**.

Known concrete gap at kickoff: the v0.8.0 **fleet observability surface** (`GET /api/v1/fleet`, `/api/v1/fleet/{id}`, `/api/v1/fleet/live` WS stream, `WsMessage::FleetUpdate`, agent k8s-connectivity fields, fleet-sweep task) has **no documentation** in the nav.

## Goals & Non-Goals **[REQUIRED]**

**Goals:**
- Audit the entire docs tree against the actual code for **correctness** (reference claims + how-to steps traceable to file:line).
- Audit Diátaxis **quadrant compliance** (no tutorial that explains, no reference that instructs, etc.).
- **Reorganize** the docs (incl. `SUMMARY.md` nav) to lead with a fast trial path, then common integration patterns.
- Add the missing **fleet observability** docs across the right quadrants.
- Add/strengthen the two requested article families (fast-trial, common-integration-patterns).
- Pass the gated four-reviewer loop (accuracy, completeness, clarity, diataxis-compliance) with zero blockers/majors on accuracy/completeness/compliance and no clarity blockers.

**Non-Goals:**
- Re-writing the auto-generated rustdoc under `docs/src/api/rust/**` (regenerated from source; treat as out of scope except where nav/linking matters).
- Documenting the web UI (`examples/ui-slim`) as a product. **HARD CONSTRAINT: the UI is a DEMO of what a consumer would build, NOT a supported consumption interface.** Any doc that treats the UI as the product is a defect to fix, not a surface to document.
- Shipping the docs themselves as a code/feature change beyond the docs tree + nav.

## Requirements **[CONDITIONAL: Requirements-Heavy Initiative]**

{Delete if not a requirements-focused initiative}

### User Requirements
- **User Characteristics**: {Technical background, experience level, etc.}
- **System Functionality**: {What users expect the system to do}
- **User Interfaces**: {How users will interact with the system}

### System Requirements
- **Functional Requirements**: {What the system should do - use unique identifiers}
  - REQ-001: {Functional requirement 1}
  - REQ-002: {Functional requirement 2}
- **Non-Functional Requirements**: {How the system should behave}
  - NFR-001: {Performance requirement}
  - NFR-002: {Security requirement}

## Use Cases **[CONDITIONAL: User-Facing Initiative]**

{Delete if not user-facing}

### Use Case 1: {Use Case Name}
- **Actor**: {Who performs this action}
- **Scenario**: {Step-by-step interaction}
- **Expected Outcome**: {What should happen}

### Use Case 2: {Use Case Name}
- **Actor**: {Who performs this action}
- **Scenario**: {Step-by-step interaction}
- **Expected Outcome**: {What should happen}

## Architecture **[CONDITIONAL: Technically Complex Initiative]**

{Delete if not technically complex}

### Overview
{High-level architectural approach}

### Component Diagrams
{Describe or link to component diagrams}

### Class Diagrams
{Describe or link to class diagrams - for OOP systems}

### Sequence Diagrams
{Describe or link to sequence diagrams - for interaction flows}

### Deployment Diagrams
{Describe or link to deployment diagrams - for infrastructure}

## Detailed Design **[REQUIRED]**

{Technical approach and implementation details}

## UI/UX Design **[CONDITIONAL: Frontend Initiative]**

{Delete if no UI components}

### User Interface Mockups
{Describe or link to UI mockups}

### User Flows
{Describe key user interaction flows}

### Design System Integration
{How this fits with existing design patterns}

## Testing Strategy **[CONDITIONAL: Separate Testing Initiative]**

{Delete if covered by separate testing initiative}

### Unit Testing
- **Strategy**: {Approach to unit testing}
- **Coverage Target**: {Expected coverage percentage}
- **Tools**: {Testing frameworks and tools}

### Integration Testing
- **Strategy**: {Approach to integration testing}
- **Test Environment**: {Where integration tests run}
- **Data Management**: {Test data strategy}

### System Testing
- **Strategy**: {End-to-end testing approach}
- **User Acceptance**: {How UAT will be conducted}
- **Performance Testing**: {Load and stress testing}

### Test Selection
{Criteria for determining what to test}

### Bug Tracking
{How defects will be managed and prioritized}

## Alternatives Considered **[REQUIRED]**

{Alternative approaches and why they were rejected}

## Implementation Plan **[REQUIRED]**

Driven by `/docs-diataxis` (Discovery ✔ → Plan → Write → gated Review). Full discovery inventory: `discovery-inventory.md` (sibling file).

### Workstream A — Onboarding restructure (user priority #1)
- **A1 (new, getting-started/tutorial): "Evaluate Brokkr locally"** — a fast, one-path trial that gets an evaluator to a working deploy ASAP. Replaces the misnamed `quick-start.md`. Vehicle TBD (see Open Decisions).
- **A2 (reorg): SUMMARY.md** — lead with the trial path; add a "Common integration patterns — start here" signpost; move contributor-facing pages (build-and-publish-images, sdks/regeneration) out of the consumption flow; relabel misfiled getting-started pages (`development.md`=how-to, `quick-start.md` removed/folded).
- **A3 (de-dup):** reconcile `quick-start.md` vs `tutorials/first-deployment.md` (near-identical) — one trial eval, one deeper tutorial.

### Workstream B — Fleet observability (v0.8.0 gap)
- **B1 (new how-to): "Monitor your agent fleet"** — GET /api/v1/fleet, /agents/:id/fleet-status, consuming /fleet/live WS; interpreting signals (heartbeat staleness, backpressure: pending_object_count/pending_work_orders/claimed_work_orders, health_failing/degraded, k8s_reachable). Philosophy: broker surfaces signals, consumer decides severity (I-0027).
- **B2 (new reference): reference/fleet.md** — full FleetAgentRecord schema, the 2 REST endpoints (admin-gated), /fleet/live + FleetUpdate frame, 20s sweep + computed-signal trigger. Cross-link from reference/api/README.md + ws-protocol.md.
- **B3 (explanation):** fleet legibility rationale (pull vs push, broker-computed, hybrid trigger, "surface not decide") — fold into monitoring explanation or new page.
- **B4 (nav):** add fleet to How-To "Observe & debug" + Reference.

### Workstream C — Correctness sweep (user priority on correctness)
- **C1 UI-as-demo fixes:** `deployment-health.md:7` (clear violation), `internal-ws-channel.md` (qualify "the UI" language), `container-images.md` (resolve build/size self-contradiction). Model on the exemplary `container-images.md:15` / `development.md` framing.
- **C2 WS count fix:** `ws-protocol.md` intro "two"→"three" WebSocket surfaces.
- **C3 Config correctness:** `pak.digest` type, `audit_log_retention_days` default-90 location, `kubeconfig_path` `${USER}` footgun, document hardcoded cleanup intervals as fixed (not tunable).
- **C4 Metrics correctness:** purge removed `brokkr_database_*` metrics from monitoring.md/dashboards if referenced.
- **C5 C4-model check:** verify `architecture.md` diagrams are C4-form (per project convention).
- **C6 Found code/doc mismatches (flag, don't paper over):** OpenAPI auth annotations vs enforcement (agent_events admin-only; search_agent; add/remove_target owning-generator; config/reload 500 path; complete_work_order 202; stale agents.rs module header). Document **actual enforced behavior**; recommend separate code tickets for the annotation bugs.

### Workstream D — Gated review (Phase 4)
- Dispatch accuracy-, completeness-, clarity-, diataxis-compliance-reviewers in parallel; loop until accuracy/completeness/compliance = 0 blockers/majors and clarity = 0 blockers. Minors addressed or waived-with-reason.

### Decisions (user sign-off received 2026-06-14)
1. **Trial-fast vehicle (A1): BOTH** — a "fastest look" page on `angreal local up` (clone + one command, full local stack), AND a "realistic evaluation" page on Helm into local kind/k3d using the published 0.8.0 images (no source build).
2. **Delivery: SINGLE docs PR** for the whole audit.
3. Plan scope/grouping approved → proceed to design → decompose.

### Sequencing
Plan sign-off → transition to design → decompose into tasks (one per workstream item, fleet docs + onboarding restructure first as highest value) → write → gated review loop → single docs PR (or grouped PRs per workstream).