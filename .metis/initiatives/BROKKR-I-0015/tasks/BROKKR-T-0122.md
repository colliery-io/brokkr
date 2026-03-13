---
id: domain-3-architecture-and
level: task
title: "Domain 3: Architecture and Component Verification with C4 Migration"
short_code: "BROKKR-T-0122"
created_at: 2026-03-13T14:01:17.290581+00:00
updated_at: 2026-03-13T14:48:45.410948+00:00
parent: BROKKR-I-0015
blocked_by: []
archived: false

tags:
  - "#task"
  - "#phase/completed"


exit_criteria_met: false
initiative_id: BROKKR-I-0015
---

# Domain 3: Architecture and Component Verification with C4 Migration

## Parent Initiative

[[BROKKR-I-0015]] — Documentation Validation Against Implementation

## Objective

Verify every architectural and component claim in the documentation against the actual crate structure, module organization, inter-crate dependencies, and control flow. Then **migrate all architectural diagrams from ad-hoc Mermaid to the C4 model**, producing properly-leveled System Context, Container, and Component diagrams. ER diagrams and sequence diagrams retain their current format — C4 applies to structural/architectural views only.

## Documentation Files in Scope

- `docs/content/explanation/architecture.md` (~309 lines)
- `docs/content/explanation/components.md` (~466 lines)
- `docs/content/explanation/data-flows.md` (~506 lines)
- `docs/content/explanation/network-flows.md` (~298 lines)
- `docs/content/explanation/security-model.md` (~489 lines)

## Source of Truth

- Workspace `Cargo.toml` and per-crate `Cargo.toml` files (crate structure, dependencies)
- `main.rs` / entrypoint files for broker and agent (startup flow, component wiring)
- Module tree (`mod.rs` files, `lib.rs` re-exports)
- Axum router composition (how API layers are assembled)
- Agent polling/reconciliation loop code
- Authentication/authorization middleware and extractors
- Network communication code (HTTP clients, TLS config)

## Verification Checklist

For architectural claims:
- [ ] Every component/service named in the docs exists as a crate or module
- [ ] Communication protocols described (HTTP, gRPC, database connections) match actual code
- [ ] The pull-based model description matches the agent's actual polling implementation
- [ ] Broker-agent interaction described matches the actual API calls the agent makes
- [ ] Any described internal layering (API → service → DAL) matches module organization

For component descriptions:
- [ ] Each component's described responsibilities match its actual code
- [ ] Dependencies between components match `Cargo.toml` dependency declarations
- [ ] No components are described that don't exist, and no significant components are omitted

For data flow descriptions:
- [ ] Each flow (deployment creation, reconciliation, event reporting) traces correctly through the code
- [ ] Sequence of operations matches actual control flow
- [ ] Data transformations described actually occur

For network flow descriptions:
- [ ] Ports, protocols, and connection directions match actual code and configuration
- [ ] TLS/mTLS claims match implementation
- [ ] Firewall/network policy recommendations align with actual requirements

For security model:
- [ ] Authentication mechanisms described match actual middleware
- [ ] Authorization model (API keys, roles, scopes) matches implementation
- [ ] Token/key format and validation described matches code
- [ ] Security boundaries described match actual enforcement points

## C4 Migration Deliverables

All existing ad-hoc Mermaid architecture diagrams must be replaced with C4 diagrams using Mermaid's C4 syntax:

### Level 1: System Context Diagram
- Brokkr as the central system
- External actors: Platform Engineers, CI/CD Pipelines (Generators), Kubernetes Clusters, PostgreSQL
- Relationships showing what each actor does with Brokkr

### Level 2: Container Diagram
- Broker (Rust/Axum REST API)
- Agent (Rust Kubernetes operator)
- PostgreSQL database
- Communication protocols between containers (HTTP REST, SQL)
- External system connections

### Level 3: Component Diagrams (one per container)
**Broker components**: API layer, authentication middleware, DAL (data access layer), webhook dispatcher, health check subsystem, metrics/observability, OpenAPI generation
**Agent components**: Polling loop, Kubernetes client, reconciliation engine, event reporter, health check subsystem, sidecar operator interface (if applicable per ADR-2)

### Syntax Requirements
- Use `C4Context`, `C4Container`, `C4Component` Mermaid diagram types
- Ensure all diagrams render correctly in Hugo with the geekdoc theme
- Include proper `Person`, `System`, `Container`, `Component` elements with descriptions
- Use `Rel` for relationships with protocol/purpose labels

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] Every architectural claim in the 5 files has been verified against source code
- [ ] Every existing Mermaid architecture diagram has been migrated to C4
- [ ] System Context (L1), Container (L2), and Component (L3) diagrams are produced
- [ ] C4 diagrams render correctly in Mermaid syntax
- [ ] ER and sequence diagrams are left in their original format (not converted to C4)
- [ ] All prose descriptions of architecture match the verified reality
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

### Fixes Applied (Session 2)

**architecture.md** (8 fixes + 3 C4 diagrams):
- Fixed connection pool default: 5 → 50
- Rewrote event bus section: was described as in-memory OnceCell/mpsc pub/sub with 1000 buffer; actually database-centric `emit_event()` that directly inserts webhook delivery records
- Fixed config watcher: was "Kubernetes API ConfigMap watch"; actually filesystem-based using `notify` crate with `BROKKR_CONFIG_FILE` env var
- Removed WebSocket reference (not implemented)
- Fixed heartbeat timer description (was hardcoded "30 seconds")
- Fixed event/webhook flow description to match database-centric model
- Added C4 Level 1 System Context diagram
- Added C4 Level 2 Container diagram
- Converted request processing flowchart → C4 Component diagram (L3)
- Converted agent control loop flowchart → C4 Component diagram (L3)
- Converted horizontal scaling graph → C4 Container diagram (L2)

**components.md** (6 fixes):
- Fixed DAL accessor types: `AgentsAccessor`/`StacksAccessor` → `AgentsDAL`/`StacksDAL`
- Fixed annotation constants: wrong keys and wrong label/annotation distinction. Now shows actual `STACK_LABEL`, `DEPLOYMENT_OBJECT_ID_LABEL`, `CHECKSUM_ANNOTATION`, `LAST_CONFIG_ANNOTATION`, `BROKKR_AGENT_OWNER_ANNOTATION` with correct `k8s.brokkr.io/` and `brokkr.io/` prefixes
- Fixed polling interval default: 30 → 10
- Fixed event bus description to "database-centric webhook dispatch"
- Fixed config watcher: "ConfigMap" → "filesystem"

**data-flows.md** (5 fixes):
- Fixed polling interval: 30s → 10s (prose and diagram)
- Rewrote event architecture section: removed in-memory bus description, replaced with database-centric emit_event() model, correct audit logger buffer (10,000)
- Fixed generator auth: `X-Generator-Key` → `Authorization: Bearer {PAK}` with full PAK verification flow
- Fixed admin auth: "bearer tokens" → PAKs with same verification mechanism
- Fixed auth overview: was "agents use PAKs, admins use bearer tokens, generators use API keys" → "all actors use PAKs via Authorization: Bearer header"

**network-flows.md** (2 fixes + 1 C4 diagram):
- Fixed readiness endpoint: `/ready` → `/readyz`
- Fixed polling interval default: 30 → 10
- Converted network topology flowchart → C4 System Context diagram

**security-model.md** (3 fixes + 1 C4 diagram):
- Fixed PAK rotation env var: `BROKKR__BROKER__PAK` → `BROKKR__AGENT__PAK`
- Fixed RBAC description: "Read-only access" → "Resource management access" (agents need write permissions)
- Converted trust boundaries flowchart → C4 System Context diagram

**C4 Migration Summary**: 7 ad-hoc Mermaid diagrams converted to C4 syntax:
- 1x C4Context (L1) - System Context (new in architecture.md)
- 2x C4Container (L2) - Container diagram + scaling pattern
- 2x C4Component (L3) - Broker request processing + Agent internals
- 1x C4Context - Network topology (network-flows.md)
- 1x C4Context - Security trust boundaries (security-model.md)

All sequence diagrams, state diagrams, and ER diagrams left in original format per task spec.