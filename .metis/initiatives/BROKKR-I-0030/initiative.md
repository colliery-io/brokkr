---
id: multi-application-isolation-safe
level: initiative
title: "Multi-Application Isolation: Safe Shared Control Plane"
short_code: "BROKKR-I-0030"
created_at: 2026-06-26T12:45:02.369710+00:00
updated_at: 2026-06-26T12:45:02.369710+00:00
parent: 
blocked_by: []
archived: false

tags:
  - "#initiative"
  - "#phase/discovery"


exit_criteria_met: false
estimated_complexity: L
initiative_id: multi-application-isolation-safe
---

# Multi-Application Isolation: Safe Shared Control Plane

## Context

Brokkr is highly efficient as a single-tenant control plane: one admin PAK, one set of generators and agents, one shared DB. That efficiency becomes a liability when multiple independent applications want to share it. The core risk is **accidental cross-targeting**: because agent-to-stack binding is controlled only at the admin level, any operator error can route a deployment agent to a stack it was never meant to touch. Application A's production-cluster agent silently starts applying Application B's deployment objects.

The revised design solves this with a **subscription model**: agents opt in to the scopes they serve rather than having scopes pushed onto them. A generator IS an application scope. An agent cannot receive stacks from a generator it has not registered with.

### Current auth/tenancy model (as-coded)

**Auth principals:**
- `admin` — singleton PAK, god mode
- `generator` — application-level identity; owns stacks (`stacks.generator_id` FK), can only CRUD its own stacks and assign agents to its own stacks
- `agent` — cluster executor; can only act on its own record and read stacks it is targeted against

**What isolation exists today:**
- `fetch_owned_stack` enforces generator ownership on all stack CRUD
- `authorize_target_mutation` prevents a generator from assigning an agent to another generator's stack
- DB has PostgreSQL schema-per-tenant infrastructure (`create_shared_connection_pool(schema=...)`) — fully proven in integration tests but deployment-level only, not API-surfaced

**The isolation gap:**
Agents have no concept of which generator scopes they serve. Admin can bind any agent to any stack from any generator — accidentally or intentionally. There is no system-enforced guard. The `agent_targets` join says "this agent applies this stack" but nothing says "this agent is allowed to receive direction from this generator."

## Goals

- Multiple independent applications can safely share a single Brokkr control plane without accidental cross-targeting
- Agents explicitly declare which generator scopes (applications) they serve; cross-scope delivery is structurally impossible without a prior registration
- All agents automatically receive admin-tier stacks for fleet-wide management (monitoring, security, etc.) without additional configuration
- `agent_targets` (per-stack targeting granularity) is preserved — registration gates eligibility, targeting still controls specificity
- Backward compatibility: existing single-generator deployments continue to work without changes

**Non-Goals:**
- Physical schema-per-application isolation (available as an escape hatch via the existing PG schema mechanism, not the primary path here)
- Multi-region or multi-cluster broker federation
- UI/dashboard work — API and data model only

## Threat Model

| Threat | Current posture | Target posture |
|--------|----------------|----------------|
| Admin accidentally targets an agent at a generator's stack it doesn't serve | No guard — succeeds silently | Hard error: `agent_not_registered` — agent must register with generator first |
| Generator A targets an agent against Generator B's stacks | Blocked (`authorize_target_mutation` checks stack ownership) | No change — still blocked |
| Generator A reads Generator B's stacks | Blocked (`fetch_owned_stack`) | No change |
| Agent receives stacks from a generator it never opted in to | Not blocked | Blocked: stack delivery requires registration |
| Fleet-wide update reaches all agents | Requires targeting each agent individually | Admin-tier stacks via system generator reach all agents automatically |

## Architecture

### Core model: generator registration as scope gate

Introduce a new join table `agent_generator_registrations (agent_id, generator_id)`. This is the agent's explicit declaration of which application scopes it serves.

**The invariant added to `authorize_target_mutation`:**
Before a stack can be targeted at an agent, the agent must be registered with the stack's generator. This check applies unconditionally — admin included. Admin can register an agent with a generator (as an operational escape hatch) but cannot silently bypass the registration requirement.

**`agent_targets` is preserved.** Registration gates eligibility; `agent_targets` still controls which specific stacks within an eligible generator the agent actually applies. This keeps the existing label/annotation-based matching and the per-stack granularity intact.

### Admin-tier stacks: the system generator

A reserved **system generator** is provisioned at broker initialization (deterministic, cannot be deleted). All agents are auto-registered with the system generator at creation time. Admin-tier stacks (fleet monitoring, security tooling, etc.) are created under this system generator and reach every agent without any additional targeting configuration.

This keeps the data model uniform: admin-tier stacks are normal stacks with a normal `generator_id` (the system generator's). No new flags or nullable relaxations needed.

```
system_config table OR generators.is_system BOOLEAN:
  - system_generator_id: UUID (generated at first startup, stored persistently)
  - Cannot be deleted (enforced in API)
  - All new agents auto-registered at creation
```

### New table: `agent_generator_registrations`

```sql
CREATE TABLE agent_generator_registrations (
    id           UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    agent_id     UUID NOT NULL REFERENCES agents(id) ON DELETE CASCADE,
    generator_id UUID NOT NULL REFERENCES generators(id) ON DELETE CASCADE,
    registered_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE (agent_id, generator_id)
);

CREATE INDEX idx_agr_agent_id     ON agent_generator_registrations(agent_id);
CREATE INDEX idx_agr_generator_id ON agent_generator_registrations(generator_id);
```

On agent deletion: cascade (FK). On generator deletion: cascade (removes all agent registrations for that generator, and those agents will fail the next `authorize_target_mutation` check for any remaining targets — cleanup should also cascade-delete the associated `agent_targets` rows for that generator's stacks).

### API surface

**Registration (agent self-registers):**
- `POST /generators/:id/register` — Agent PAK registers itself with the generator identified by `:id`. The agent's PAK must match an agent record. Returns the registration. Admin can also call this to bootstrap an agent before it comes online.
- `DELETE /generators/:id/register` — Agent PAK deregisters itself. When deregistering, all `agent_targets` rows for that generator's stacks are cascade-deleted (agent is no longer eligible, dangling targets serve no purpose).

**Visibility:**
- `GET /agents/:id/registrations` — Returns the list of generators this agent is registered with. Accessible by the agent itself (own PAK) or admin.
- `GET /generators/:id/registered-agents` — Returns agents registered with this generator. Accessible by the generator's own PAK or admin.

**Updated `authorize_target_mutation` logic:**
```
1. Is caller admin or owning generator? (existing check — unchanged)
2. Is the agent registered with the stack's generator? (NEW)
   → No → 403 agent_not_registered
   → Yes → proceed
```

**Updated `POST /agents` (create agent):**
Auto-insert a registration row for the system generator. No caller action required.

**No change to `POST /agents/:id/targets` or `DELETE /agents/:id/targets/:stack_id` signatures** — the authorization check is the only change.

### Delivery logic for agent target-state

No change needed to the agent's `GET /agents/:id/target-state` response. The registration enforcement lives entirely at write time (`authorize_target_mutation`). Existing targets that were created before this feature is deployed remain valid (the registration for the system generator covers admin-created targets; application-generator targets will need a one-time migration or a registration back-fill script).

## Requirements

**Functional:**
- REQ-001: `POST /agents/:id/targets` MUST be rejected with `403 agent_not_registered` if the agent has no registration for the stack's generator — this applies to admin callers too
- REQ-002: An agent PAK MUST be able to self-register with any non-system generator via `POST /generators/:id/register`
- REQ-003: All agents MUST be automatically registered with the system generator at creation time
- REQ-004: The system generator MUST be provisioned at broker first-startup and MUST NOT be deletable via any API
- REQ-005: Deregistering from a generator MUST cascade-delete all `agent_targets` rows for that generator's stacks for that agent
- REQ-006: `GET /agents/:id/registrations` MUST be accessible by the agent's own PAK and by admin
- REQ-007: `GET /generators/:id/registered-agents` MUST be accessible by that generator's own PAK and by admin
- REQ-008: All registration and deregistration events MUST be written to the audit log

**Non-Functional:**
- NFR-001: The registration check in `authorize_target_mutation` MUST use an indexed lookup (`agent_id + generator_id` unique index) — no table scans on the hot targeting path
- NFR-002: Existing `agent_targets` rows created before the feature ships remain valid — backward compatible for single-generator deployments where the agent is auto-registered with the system generator and the application generator is the only other one

## Alternatives Considered

**Agent `generator_id` FK (earlier draft):** An agent owns one generator. Simpler but doesn't allow an agent to serve multiple application scopes legitimately (e.g., an ops agent that serves both App A and App B's clusters). The subscription model handles this naturally.

**Label-based soft isolation:** Agent labels signal scope membership. Admin can still bypass; generators that don't control labels remain blind. Not structurally enforced.

**PostgreSQL schema-per-application (full physical isolation):** Strongest guarantee, already proven in integration tests. Over-engineered for the stated risk. Available as a deployment-level escape hatch; not needed as the primary path.

## Implementation Plan

**Phase 1 — Design finalization (this document)**
- ✅ Confirm design decisions with stakeholders
- ✅ Write ADR for generator registration model and system generator design → BROKKR-A-0009

**Phase 2 — System generator + registration table**
- Migration: `agent_generator_registrations` table + indexes
- Broker init: provision system generator (idempotent, stored in `system_config` or `generators.is_system`)
- DAL: `agent_generator_registrations()` CRUD, `is_registered(agent_id, generator_id)` lookup
- `POST /agents` auto-registration with system generator

**Phase 3 — Registration API endpoints**
- `POST /generators/:id/register` (agent self-registers, admin can also call)
- `DELETE /generators/:id/register` with cascade on `agent_targets`
- `GET /agents/:id/registrations`
- `GET /generators/:id/registered-agents`
- Auth: enforce correct PAK types, audit log all events

**Phase 4 — Enforce registration at targeting**
- Update `authorize_target_mutation` to check registration
- Integration tests: unregistered agent rejected; registered agent accepted; admin cannot bypass; system-generator stacks always work; deregistration cascades targets

**Phase 5 — Migration / back-fill**
- Script or migration to register existing agents with existing application generators (one-time, data-preserving)
- Verify no existing targets are orphaned

**Phase 6 — Operator surface (post-release audit, v0.8.2)**

A release audit of #79 found the registration model shipped end-to-end in the
broker/agent/API/SDK, but the operator-facing surfaces that the design intended
(declare scope "at deploy time", Decision 3) were not wired. Follow-on tasks:
- [[BROKKR-T-0249]] — Agent Helm chart: expose `BROKKR_GENERATOR_IDS` via a
  `generatorIds` value (chart was version-bumped only; the value is unreachable
  through Helm today). Empty default = system/fleet scope only.
- [[BROKKR-T-0250]] — Promote generator IDs into `AgentConfig`
  (`BROKKR__AGENT__GENERATOR_IDS` + `--generator-ids`); the agent currently reads
  a bare `std::env::var`, off the `BROKKR__AGENT__*` config convention.
- [[BROKKR-T-0251]] — `brokkr-cli` registration commands (register / deregister /
  list); the lifecycle is API/SDK-only, no admin CLI path.

Broker Helm chart correctly needed no change (system generator is auto-provisioned
at startup). The day-zero offline-PAK bootstrap QoL is handled separately on the
working branch and is not ticketed here.

## Resolved Design Decisions

1. **System generator UUID:** Generated at first startup, stored persistently (in `system_config` table, a text file, or broker config — implementation choice at task time). Avoids hardcoded constants; each deployment gets its own identity.

2. **Admin bypass on registration check:** No bypass. Admin's escape hatch is an explicit `POST /generators/:id/register` call on behalf of the agent — one additional step that is fully audited. There is no `force=true` override.

3. **Multi-registration on agent create:** `POST /agents` accepts an optional `generator_ids: []` list. Any IDs supplied are registered immediately alongside the automatic system-generator registration. This enables one-shot provisioning. Additionally, the agent process supports a `BROKKR_GENERATOR_IDS` environment variable: a comma-separated list of generator IDs the agent will self-register with on startup. This means a cluster operator can fully declare the agent's scope at deploy time via environment rather than requiring a post-startup API call.

4. **Deregistration from any generator including system:** Agents CAN deregister from any generator, including the system generator. The model is strictly opt-in — if a generator's stacks are putting an agent at risk, the agent must have a fast, clean cut mechanism. Deregistration triggers **active reconciliation, not a freeze**:
   - The broker synthesizes per-agent deletion signals for every stack the agent was actively targeting from that generator
   - The agent receives these signals and applies them locally (removing its cluster resources for those stacks)
   - Only this agent reconciles to empty — the stacks are unaffected and other registered agents continue applying them
   - Once the agent reports reconciliation or after a configurable timeout, the `agent_targets` rows for that generator's stacks are removed

   **Open implementation question (task-level):** The "deletion signal scoped to a single agent" does not map directly to the existing `deployment_objects.is_deletion_marker` (which is global — all agents targeting a stack see it). The implementation needs one of:
   - A nullable `agent_id` on a new deletion work order record so the broker can issue targeted cleanup
   - A new agent-event type (`DEREGISTER_RECONCILE`) that the agent receives via WS push and handles autonomously
   - An extension to the existing `work_orders` model to support agent-scoped deletion instructions
   
   The work orders system (`crates/brokkr-broker/src/api/v1/work_orders.rs`) is the most likely home for this — targeted work orders per agent are already close to this concept. The implementation approach should be decided in the task for Phase 4.
