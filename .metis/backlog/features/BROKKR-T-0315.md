---
id: generator-pak-list-registered-agents
level: task
title: "Generators cannot list their own agents: GET /agents is admin-only, so tenant tooling needs an admin PAK to answer \"which agents serve me?\""
short_code: "BROKKR-T-0315"
created_at: 2026-07-28T22:30:00.000000+00:00
updated_at: 2026-07-28T22:30:00.000000+00:00
parent: 
blocked_by: []
archived: false

tags:
  - "#task"
  - "#feature"
  - "#phase/backlog"


exit_criteria_met: false
initiative_id: NULL
---

# Generators cannot list their own agents: GET /agents is admin-only, so tenant tooling needs an admin PAK to answer "which agents serve me?"

## Objective

Let a generator PAK call `GET /api/v1/agents` and receive the agents registered to that generator, so a tenant can answer "which agents serve me?" without holding an admin credential.

Today `list_agents` (`crates/brokkr-broker/src/api/v1/agents.rs:106`) opens with `require_admin(&auth_payload)?`. A generator PAK gets 403. Since tenants **are** generators (BROKKR-A-0009), the only way for a tenant to enumerate the agents that will pick up its stacks is to be handed an admin PAK — which grants the entire fleet, every other tenant's agents, and full write access. That is a real pressure toward over-provisioning the strongest credential in the system for a read that is intrinsically tenant-scoped.

This is the missing half of a pairing that already exists in the other direction: an agent can ask which generators it is registered with (`GET /agents/:id/registrations`, admin-or-that-agent), but a generator cannot ask which agents are registered to it.

## Backlog Item Details

### Type
- [x] Feature - New functionality

### Priority
- [x] P2 - Medium (no live defect; it removes a standing reason to issue admin PAKs to tenants, which is a security posture improvement rather than a fix)

## 2026-07-28 — grounding: most of this already exists

Checked against the code before filing. The work is smaller than the title suggests, because three of the four pieces are already in place:

| Piece | State |
|---|---|
| `AuthPayload.generator: Option<Uuid>` | **Exists** — `api/v1/middleware.rs:40` |
| `agent_generator_registrations().list_for_generator(id)` | **Exists** — `dal/agent_generator_registrations.rs:65`, returns `Vec<AgentGeneratorRegistration>` |
| Handler pattern for admin-or-generator listing | **Exists** — `list_stacks`, `api/v1/stacks.rs:111-135` |
| `agents().list_for_generator(id)` | **Missing** — the only new DAL method needed |

**`list_stacks` is the precedent to copy, almost literally.** It branches: admin → `stacks().list()` (with the `scope.pak_id` tenant-view filter applied on the admin path only), generator → `stacks().list_for_generator(generator_id)`, neither → 403 `stacks_list_denied`. Mirroring that shape in `list_agents` keeps the two collection endpoints consistent, and a reviewer who knows one will recognize the other.

**No credential material is at risk.** `Agent.pak_hash` is `#[serde(skip_serializing, skip_deserializing)]` (`brokkr-models/src/models/agents.rs:80-82`), so it never crosses the wire on any path. Broadening the caller set does not expose it. What a generator would newly see is agent id, name, cluster name, status, heartbeat, and the k8s reachability fields — for agents that have explicitly registered with that generator, which is a consent relationship the agent opted into (BROKKR-T-0287).

### The one real design question: the system generator

**Every agent is always registered with the system generator** — `create_agent` does it unconditionally (`agents.rs:204`), the CLI path does the same (`cli/commands.rs:488`), and `provision_system_generator` back-registers every pre-existing agent on first run (`dal/generators.rs:281-285`). So `list_for_generator(system_generator_id)` returns **the entire fleet**.

If a system-generator PAK were ever issuable, this feature would turn it into an admin-equivalent fleet read through a non-admin credential. **Today it is not**: `provision_system_generator` inserts only `name`/`description`/`is_system` and never sets a `pak_hash`, so nothing can authenticate as `__system__`.

That is a property worth *enforcing* rather than relying on, since it holds by omission. Decide one of:
- **(a)** Explicitly reject `is_system` generators in this handler, so the fleet-wide read cannot be reached this way even if a PAK is later mintable for one. Cheap, and documents the invariant in the place that depends on it.
- **(b)** Treat it as already covered by "the system generator has no PAK" and add a test asserting that instead.

**(a) plus the test is the recommendation** — the guard is a few lines and the invariant currently lives nowhere near the code that would break if it changed.

### Open question: soft-deleted agents

`agents().list()` and `agents().list_all()` differ in soft-delete handling. The new method should match whatever `list()` does so the admin and generator paths agree; worth confirming rather than assuming, since a generator seeing deleted agents (or not seeing live ones) would be a quiet inconsistency between the two branches.

### Secondary surfaces

Whatever is decided here, these should follow or be explicitly deferred:
- **`GET /agents/` (search_agent)** and the per-agent reads (`get_agent`, `list_labels`, `get_associated_stacks`) are also admin-gated. A generator that can list agents but cannot read one of them by id has a listing it cannot follow up on. At minimum, `get_agent` should accept a generator registered with that agent.
- **CLI**: `brokkr agent list` under a generator PAK currently fails; it should work and show the scoped view.
- **SDKs**: no signature change (same endpoint, same response type), but the three clients' docs claim admin-only for this call.
- **`reference/` docs + OpenAPI**: `security(("admin_pak" = []))` on the utoipa annotation becomes admin-or-generator, which changes the generated spec and therefore all three SDKs' generated docs.

## Acceptance Criteria

- [ ] A generator PAK calling `GET /api/v1/agents` receives exactly the agents registered to that generator, and admin behavior (including the `scope.pak_id` tenant-view filter) is unchanged.
- [ ] A generator PAK does not see agents registered only to other generators — asserted by an integration test with two generators and at least one agent each.
- [ ] The system-generator case is resolved per (a) or (b) above, with a test pinning whichever invariant is chosen.
- [ ] Soft-delete semantics match the admin `list()` path.
- [ ] A caller with neither admin nor generator identity still gets 403, with an error code consistent with `stacks_list_denied`.
- [ ] OpenAPI security annotation updated, `angreal openapi check` passes, and the SDK docs no longer claim admin-only.
- [ ] Reference docs state the scoping rule and that it is registration-derived, so operators know the listing reflects agent consent rather than label matching.

## Status Updates

*To be added during implementation*
