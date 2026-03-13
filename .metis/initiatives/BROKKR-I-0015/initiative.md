---
id: documentation-validation-against
level: initiative
title: "Documentation Validation Against Implementation"
short_code: "BROKKR-I-0015"
created_at: 2026-03-13T13:52:00.558381+00:00
updated_at: 2026-03-13T14:06:23.749111+00:00
parent: BROKKR-V-0001
blocked_by: []
archived: false

tags:
  - "#initiative"
  - "#phase/active"


exit_criteria_met: false
estimated_complexity: XL
initiative_id: documentation-validation-against
---

# Documentation Validation Against Implementation

## Context

Brokkr's documentation (~7,900 lines across 25 content files) was substantially overhauled in BROKKR-I-0014, which converted bullet-heavy docs to prose, fixed known terminology issues, and created several new guides. However, that initiative focused primarily on style and coverage — it did not systematically validate every technical claim, code example, API response shape, configuration snippet, Helm value, Mermaid diagram, or behavioral description against the actual implementation.

Documentation that is wrong is worse than documentation that is missing. A user who follows incorrect instructions will waste hours debugging phantom problems. A user who finds no documentation will at least know they need to explore on their own. This initiative treats every factual claim in the documentation as an assertion that must be verified against the source of truth: the code.

The documentation spans multiple verification domains: Rust source code behavior, REST API contracts (utoipa/OpenAPI), database schema (Diesel migrations), Kubernetes RBAC and resource patterns, Helm chart values, Prometheus metrics, health endpoint responses, Mermaid diagrams depicting architecture and data flows, configuration environment variables, and inter-component interaction patterns.

BROKKR-I-0014 is archived-completed. This is not a redo — it is the adversarial verification pass that follows a writing pass.

## Goals & Non-Goals

**Goals:**
- Every factual claim in the documentation is verified against the current implementation or flagged as incorrect
- Every code example, API request/response, configuration snippet, and YAML block is tested or traced to source
- Every Mermaid diagram accurately reflects the current architecture, data model, and interaction flows
- Every cross-reference and internal link resolves to an existing page
- Every environment variable, CLI flag, and configuration option documented actually exists and behaves as described
- Every Helm chart value documented matches the actual `values.yaml` and templates
- Every Prometheus metric name, label, and type documented matches the instrumentation code
- Every health endpoint response shape documented matches the actual handler output
- Dead documentation (describing removed or never-implemented features) is identified and removed
- Documentation gaps where implementation exists but documentation does not are catalogued (but writing new docs is a separate initiative)

**Non-Goals:**
- Writing new documentation for uncovered features (this initiative produces a gap inventory; a follow-up initiative would address it)
- Style, grammar, or prose quality improvements (BROKKR-I-0014 already addressed this)
- Rewriting the documentation site infrastructure or theme
- Updating the ROADMAP.md (that's a separate concern)
- Validating tutorial/getting-started content (those sections are stubs — nothing to validate)

## Detailed Design

The core methodology is **adversarial verification**: treat every sentence as a claim, find the code that would prove or disprove it, and record the verdict. Each documentation file gets its own validation pass, and each pass produces a structured findings report embedded in the task's progress notes.

### Verification Taxonomy

Each claim falls into one of these verification categories, ordered by rigor:

1. **Code-Traceable** — The claim can be verified by reading source code. Example: "The broker exposes a `/healthz` endpoint" → find the route registration in the axum router.

2. **Schema-Traceable** — The claim describes a data structure that can be verified against Diesel schema, OpenAPI spec, or Rust struct definitions. Example: "The webhook payload includes a `event_type` field" → find the serialization struct.

3. **Behavior-Traceable** — The claim describes runtime behavior that can be verified by reading control flow logic or test assertions. Example: "The agent polls every 30 seconds by default" → find the default interval constant and the polling loop.

4. **Configuration-Traceable** — The claim describes a configuration option that can be verified against environment variable parsing, CLI arg definitions, or Helm values. Example: "Set `BROKER_DATABASE_URL` to configure the database" → find the env var read.

5. **Diagram-Traceable** — The claim is a visual representation (Mermaid) that can be verified by cross-referencing the depicted components, relationships, and flows against the code. Architectural diagrams should be migrated to the C4 model (System Context, Container, Component, Code) during verification — ad-hoc Mermaid architecture diagrams are replaced with properly-leveled C4 diagrams.

6. **Cross-Reference-Traceable** — The claim is a link to another page or external resource that can be verified by resolution.

### Validation Process Per Document

For each documentation file:

1. **Extract claims** — Read the document and identify every verifiable assertion (factual statements, code examples, configuration references, behavioral descriptions, diagram elements).

2. **Locate source of truth** — For each claim, identify the specific source file(s), struct(s), function(s), migration(s), or config(s) that would confirm or deny it.

3. **Verify** — Compare the documented claim against the source of truth. Classify each as:
   - **CORRECT** — Matches implementation exactly
   - **STALE** — Was once correct but implementation has since changed
   - **INACCURATE** — Never matched implementation or contains meaningful errors
   - **UNVERIFIABLE** — Cannot be confirmed or denied from the codebase (e.g., performance claims, external system behavior)
   - **MISSING CONTEXT** — Correct but omits important caveats, edge cases, or prerequisites

4. **Fix** — For each non-CORRECT finding, produce the corrected documentation text and apply the edit.

5. **Record** — Update the task document with a structured findings summary.

### Decomposition Strategy

The work is decomposed by **verification domain**, not by documentation file. This is intentional: a domain-oriented decomposition ensures that the person (or agent) doing the verification builds up deep expertise in one area of the codebase and can cross-reference related claims across multiple documents. File-oriented decomposition would require constant context-switching between unrelated code areas.

**Domain 1: API Contract Verification**
Covers: `reference/webhooks.md`, `reference/generators.md`, `reference/work-orders.md`, `reference/audit-logs.md`, `reference/soft-deletion.md`, `reference/api/_index.md`, `how-to/generators.md`, `how-to/webhooks.md`, `how-to/shipwright-builds.md`
Source of truth: utoipa annotations, axum route handlers, request/response structs, OpenAPI spec generation
Method: For every documented endpoint, request body, response shape, status code, and query parameter — find the handler, verify the struct fields and types, verify the route path and HTTP method, verify authentication requirements.

**Domain 2: Data Model & Schema Verification**
Covers: `explanation/data-model.md`, `explanation/core-concepts.md`, `reference/soft-deletion.md`, `reference/work-orders.md`
Source of truth: Diesel schema.rs, migration SQL files, model structs in brokkr-models
Method: For every entity, relationship, field, and constraint described — find the corresponding table definition, verify column names and types, verify foreign key relationships, verify the Mermaid ER diagrams match the actual schema.

**Domain 3: Architecture & Component Verification (with C4 Migration)**
Covers: `explanation/architecture.md`, `explanation/components.md`, `explanation/data-flows.md`, `explanation/network-flows.md`, `explanation/security-model.md`
Source of truth: Crate structure, module organization, inter-crate dependencies in Cargo.toml, actual control flow in main entrypoints
Method: For every component described, communication pattern, data flow arrow, network connection, and security boundary — trace through the code to verify the described topology matches reality. All architectural diagrams in this domain must be migrated from ad-hoc Mermaid to the **C4 model** during verification. This means producing properly-leveled diagrams: a System Context diagram showing Brokkr's relationship to external actors (users, Kubernetes clusters, CI/CD systems, PostgreSQL), Container diagrams showing the broker, agent, and database as containers with their communication protocols, and Component diagrams for each container showing internal modules (e.g., the broker's API layer, DAL, reconciliation engine, webhook dispatcher). Mermaid's C4 diagram syntax (`C4Context`, `C4Container`, `C4Component`) should be used so diagrams remain renderable in Hugo. ER diagrams and sequence diagrams that depict data models or interaction flows (not architecture) retain their current Mermaid format — C4 applies to structural/architectural views only.

**Domain 4: Observability & Operations Verification**
Covers: `reference/health-endpoints.md`, `reference/monitoring.md`, `reference/container-images.md`, `how-to/deployment-health.md`
Source of truth: Health check handlers, Prometheus metric registrations, metric recording call sites, Dockerfile/container build configs
Method: For every health endpoint path, response field, and status logic — find the handler code. For every Prometheus metric name, type, label set, and description — find the registration macro and recording call sites. For every container image name, tag format, and registry — verify against CI/build configs.

**Domain 5: Configuration & Deployment Verification**
Covers: `getting-started/configuration.md`, `getting-started/installation.md`, `getting-started/quick-start.md`, Helm chart READMEs, `charts/brokkr-broker/README.md`, `charts/brokkr-agent/README.md`, `charts/brokkr-agent/RBAC.md`
Source of truth: Environment variable parsing code, Helm `values.yaml` files, Helm templates, RBAC templates, CLI argument parsing
Method: For every documented env var — find where it's read and verify the name, default value, and description. For every Helm value — verify it exists in values.yaml and is used in templates. For every RBAC rule — verify the ClusterRole template matches.

**Domain 6: Reconciliation, Templates & Stack Management Verification**
Covers: `how-to/templates.md`, `how-to/managing-stacks.md`, `how-to/understanding-reconciliation.md`
Source of truth: Template rendering code (Tera integration), JSON Schema validation, stack CRUD handlers, reconciliation loop in agent
Method: For every template syntax example — verify Tera integration supports it. For every reconciliation behavior described — trace the agent's reconciliation loop. For every stack lifecycle operation — verify the API and state machine.

**Domain 7: Cross-Reference & Structural Integrity**
Covers: All documentation files
Method: Verify every internal link resolves. Verify every external link is not broken. Verify Hugo frontmatter is correct (weight, title, geekdocCollapseSection). Verify the site navigation matches the actual file tree. Identify any orphaned pages not reachable from navigation. Identify any referenced-but-missing pages (known gaps: installation.md, quick-start.md, first-steps.md, tutorials/*).

**Domain 8: ADR Accuracy Verification**
Covers: `BROKKR-A-0001` through `BROKKR-A-0005`
Source of truth: The actual implementation state of each decision
Method: For each ADR, verify that the "decided" status accurately reflects implementation reality. Flag any ADRs where the decision was made but implementation diverged. Verify status annotations (e.g., ADR-3 noting "metrics implemented, tracing planned", ADR-4 noting "planned, not implemented") are still accurate.

## Alternatives Considered

**File-by-file decomposition**: Rejected because it forces constant context-switching between unrelated code areas. A reviewer checking `reference/monitoring.md` and then `how-to/templates.md` would need to understand both the Prometheus instrumentation layer and the Tera template engine in the same pass. Domain decomposition lets reviewers build depth.

**Sampling-based spot-check**: Rejected because the user explicitly requested maximum scrutiny. A spot-check would catch obvious errors but miss subtle drift (e.g., a field renamed in a struct but still documented under the old name, or a metric whose labels changed).

**Automated-only verification** (e.g., compiling code examples, hitting live endpoints): Rejected as the sole approach because much of the documentation describes architecture, design rationale, and behavioral patterns that can only be verified by reading code. Automated checks are valuable but insufficient alone. The domain tasks should use automated verification where possible (e.g., checking OpenAPI spec generation, running health endpoints) as a supplement to manual code tracing.

**Combined write+validate initiative**: Rejected because conflating "verify what exists" with "write what's missing" would dilute focus. Verification requires an adversarial mindset — you are trying to find errors. Writing requires a constructive mindset. Mixing them leads to less rigorous verification. The gap inventory produced by this initiative will feed a separate writing initiative.

## Implementation Plan

**Phase 1: Discovery (current)** — Define scope, methodology, and decomposition strategy. Align on approach.

**Phase 2: Design** — Finalize task breakdown. Each domain becomes one task. Establish the findings report template that each task will use.

**Phase 3: Decompose** — Create all 8 domain tasks with detailed instructions, specific file lists, and verification checklists.

**Phase 4: Active** — Execute all domain tasks. Each task produces a findings report and applies fixes directly to the documentation. Tasks can run in parallel since domains are independent.

**Phase 5: Completed** — All domains verified. Consolidate findings into a gap inventory document for future writing initiatives. Archive.

## Decomposition Summary

| Task | Domain | Files | Key Focus |
|---|---|---|---|
| BROKKR-T-0120 | API Contract Verification | 9 docs | Endpoints, request/response shapes, curl examples vs. utoipa/axum handlers |
| BROKKR-T-0121 | Data Model & Schema | 4 docs | Entities, fields, ER diagrams vs. Diesel schema.rs, migrations, model structs |
| BROKKR-T-0122 | Architecture & Components (C4 Migration) | 5 docs | Architecture prose + migrate all Mermaid arch diagrams to C4 (L1/L2/L3) |
| BROKKR-T-0123 | Observability & Operations | 4 docs | Health endpoints, Prometheus metrics catalog, container images vs. handler code |
| BROKKR-T-0124 | Configuration & Deployment | 7 docs | Env vars, Helm values, RBAC rules vs. parsing code, values.yaml, templates |
| BROKKR-T-0125 | Reconciliation, Templates & Stacks | 3 docs | Tera syntax, JSON Schema, reconciliation loop, stack lifecycle vs. code |
| BROKKR-T-0126 | Cross-Reference & Structural Integrity | All docs | Internal/external links, Hugo frontmatter, navigation, missing pages inventory |
| BROKKR-T-0127 | ADR Accuracy | 5 ADRs | Implementation state of each decision, status annotations, cross-ADR consistency |

All tasks are independent and can execute in parallel.