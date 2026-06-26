---
id: 001-generator-registration-agent-scope
level: adr
title: "Generator Registration: Agent Scope Subscription Model"
number: 1
short_code: "BROKKR-A-0009"
created_at: 2026-06-26T13:29:19.574075+00:00
updated_at: 2026-06-26T18:11:49.498208+00:00
decision_date: 
decision_maker: 
parent: 
archived: false

tags:
  - "#adr"
  - "#phase/decided"


exit_criteria_met: false
initiative_id: NULL
---

# ADR-009: Generator Registration — Agent Scope Subscription Model

## Context

Brokkr operates as a single-tenant control plane: one admin PAK, a shared pool of agents, and one set of generators each representing an application scope. As adoption grows, multiple independent applications need to share a single Brokkr deployment safely.

The current model has a structural gap: **agents have no concept of which generator scopes they serve**. The `agent_targets` join table binds agents to specific stacks, but nothing enforces that an agent has consented to receive direction from the generator that owns a given stack. An operator with admin access can silently route any agent to any generator's stacks — there is no system-enforced boundary. The result is that Application A's production-cluster agent can end up applying Application B's deployment objects due to a single operator error.

A secondary consequence is that agents cannot distinguish between two conceptually different classes of stack:

1. **Fleet-management stacks** — monitoring agents, security tooling, compliance controls that should run on every cluster in the fleet. Today, targeting these requires an explicit `agent_targets` row per agent, which does not scale.
2. **Application stacks** — workloads owned by a specific application (generator). An agent should only receive these if it has explicitly opted in to serving that application.

## Decision

Introduce a **generator registration** model: an explicit subscription table that records which generator scopes each agent has opted in to serve. Registration becomes a hard precondition for stack targeting. An agent that has not registered with a generator cannot have that generator's stacks targeted at it, regardless of who is making the request.

### Data model

**New table: `agent_generator_registrations`**

```sql
CREATE TABLE agent_generator_registrations (
    id            UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    agent_id      UUID NOT NULL REFERENCES agents(id) ON DELETE CASCADE,
    generator_id  UUID NOT NULL REFERENCES generators(id) ON DELETE CASCADE,
    registered_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE (agent_id, generator_id)
);

CREATE INDEX idx_agr_agent_id     ON agent_generator_registrations(agent_id);
CREATE INDEX idx_agr_generator_id ON agent_generator_registrations(generator_id);
```

No changes to `agents`, `stacks`, `generators`, or `agent_targets`. The registration check is additive to the existing `authorize_target_mutation` logic; all existing per-stack targeting granularity is preserved.

### System generator

A reserved **system generator** is provisioned idempotently at broker startup. Its UUID is generated on first startup and stored persistently (in a `system_config` table or equivalent). It cannot be deleted via the API.

All agents are automatically registered with the system generator at creation time. Stacks created under the system generator (admin-tier: fleet monitoring, security tooling, policy enforcement) are automatically eligible for targeting at any agent in the fleet without any per-agent configuration.

This keeps the data model uniform — admin-tier stacks are normal stacks with a `generator_id`, not a special flag or a nullable relaxation.

### Registration flow

**Agent self-registers.** The primary registration path is the agent calling `POST /generators/:id/register` with its own PAK. This allows a cluster operator to declare the agent's application scope at deploy time. Admin may also register an agent on its behalf (for bootstrapping before the agent is live).

**Environment-variable bootstrap.** The agent process reads `BROKKR_GENERATOR_IDS` (comma-separated generator UUIDs) on startup and self-registers with each. This enables fully declarative agent configuration via environment without requiring a post-startup API call.

**Agent creation with initial registrations.** `POST /agents` accepts an optional `generator_ids: []` field. Any supplied IDs are registered alongside the automatic system-generator registration, enabling one-shot provisioning by an admin.

### Targeting enforcement

`authorize_target_mutation` is extended with a second check after the existing stack-ownership check:

```
1. Is caller admin or the generator that owns the stack? (existing)
2. Is the agent registered with the stack's generator?          (new)
   → No  → 403 agent_not_registered
   → Yes → proceed
```

This check applies unconditionally — including to admin. Admin cannot silently bypass it; the escape hatch is to explicitly register the agent first, which is one audited API call.

### Deregistration and reconciliation

Agents may deregister from any generator, including the system generator. Deregistration is a clean-cut mechanism: if a generator's stacks are creating risk for an agent's cluster, the agent must be able to exit quickly.

Deregistration triggers **reconciliation-to-empty for the departing agent**, not a freeze:

- The agent's `agent_targets` rows for that generator's stacks are removed.
- The stacks themselves are unaffected — other registered agents continue applying them.
- The departing agent must reconcile its cluster state to remove the resources it was applying for those stacks.

**Reconciliation mechanism (implementation-deferred):** Work orders are inappropriate here — they are imperative one-shot commands, not a state reconciliation signal. The correct model is a desired-state change: the agent's target list has shrunk, and its reconciliation loop should converge to that new desired state by applying deletion logic for resources it applied from stacks it no longer targets.

Whether this is driven by:
- The agent's existing reconciliation loop detecting "stack no longer in my target list → apply deletion markers"
- A new agent-targeted event pushed via WebSocket on deregistration
- A short-lived scoped deletion record the agent checks on its next reconciliation tick

...is an implementation decision for the task that covers deregistration. The invariant is: deregistration must not leave orphaned resources in the cluster, and must not affect other agents or the stack's global state.

### New API endpoints

| Method | Path | Auth | Description |
|--------|------|------|-------------|
| `POST` | `/generators/:id/register` | agent or admin PAK | Register agent with generator |
| `DELETE` | `/generators/:id/register` | agent or admin PAK | Deregister; cascades `agent_targets` |
| `GET` | `/agents/:id/registrations` | own agent PAK or admin | List generators this agent serves |
| `GET` | `/generators/:id/registered-agents` | generator PAK or admin | List agents registered with this generator |

## Alternatives Analysis

| Option | Pros | Cons | Risk |
|--------|------|------|------|
| **Generator registration (this ADR)** | Structurally enforced; agent controls its scope; multi-scope agents natural; uniform data model via system generator | New table and lifecycle; deregistration reconciliation needs careful design | Low |
| **Agent `generator_id` FK (ownership)** | Simple; one FK on agents table | Agent can only serve one generator; ops agents serving multiple apps impossible | Low impl, High design risk |
| **Label-based soft isolation** | No schema changes | Admin can bypass; generators don't control labels; not structurally enforced | High |
| **PostgreSQL schema-per-application** | Strongest isolation; already proven in integration tests | Requires broker-per-tenant or PAK-to-schema routing layer; operational complexity | Medium impl, over-engineered for this risk |
| **No change** | Zero cost | Cross-targeting risk remains; does not scale to multi-app | High ongoing risk |

## Rationale

The registration model was chosen because it matches the actual trust boundary: **the agent is the entity that knows which applications it is responsible for, not the operator**. An agent deployed into a production cluster by one team should not silently start executing workloads from another team's generator because of an admin error elsewhere. By making scope opt-in at the agent level, accidental cross-contamination becomes structurally impossible rather than dependent on operator discipline.

The system generator pattern avoids the need to special-case fleet management stacks anywhere in the data model or delivery logic. Fleet stacks are just stacks; every agent is always registered with their generator. The uniformity reduces cognitive load and keeps the codebase clean.

Preserving `agent_targets` maintains per-stack targeting granularity. Registration gates eligibility; targeting controls specificity. These are different concerns and should remain separate.

Rejecting an admin-bypass on the registration check (`force=true`) is intentional. The value of the invariant comes from it being unconditional. Admin's legitimate need to target an unregistered agent is served by registering first — one explicit, audited action — rather than a silent override that leaves no trace of intent.

## Consequences

### Positive
- Accidental cross-generator targeting is structurally prevented at the API layer regardless of caller identity
- Agents are self-describing about their scope; `GET /agents/:id/registrations` gives operators an accurate view of what an agent serves
- Generators gain first-class visibility into which agents are registered with them
- Fleet-wide management stacks reach all agents without per-agent configuration
- Multi-scope agents (an agent legitimately serving two applications) are naturally supported
- `BROKKR_GENERATOR_IDS` enables fully declarative agent configuration at deploy time

### Negative
- A new lifecycle concern: agents must register before stacks can be targeted at them. Existing deployments need a one-time back-fill to register existing agents with existing application generators.
- Deregistration reconciliation requires careful design to avoid orphaning cluster resources without affecting other agents — the implementation is non-trivial and deferred.
- Admin creating an agent and immediately targeting stacks now requires two steps (create → register) unless `generator_ids` is supplied at creation.

### Neutral
- No changes to `agent_targets`, `stacks`, `agents`, or `generators` tables. The existing targeting API surface is unchanged except for the new precondition check.
- The system generator is a permanent broker-level entity with no user-facing identity beyond its UUID. Operators should treat it as infrastructure, not an application generator.
- The auth cache (`dal.auth_cache`) is unaffected — registration checks hit the `agent_generator_registrations` table via the agent's ID (already in `AuthPayload`), not via PAK re-verification.