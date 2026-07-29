---
id: generators-cannot-list-their-own
level: task
title: "Generators cannot list their own agents: GET /agents is admin-only, so tenant tooling needs an admin PAK for a tenant-scoped read"
short_code: "BROKKR-T-0315"
created_at: 2026-07-28T22:30:00+00:00
updated_at: 2026-07-29T04:08:47.855014+00:00
parent: 
blocked_by: []
archived: false

tags:
  - "#task"
  - "#feature"
  - "#phase/completed"


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

**There are two special generators, and only one of them is `is_system`** — worth stating because a guard phrased as "reject special generators" will silently cover only half:

| | `__system__` | `admin-generator` |
|---|---|---|
| `is_system` | `true` | **`false`** |
| Created by | `provision_system_generator()`, `dal/generators.rs:255` | admin PAK provisioning, `utils/mod.rs:157-166` |
| `pak_hash` | **never set** | **the admin PAK's hash** |
| Agents auto-registered? | **yes, all of them** | no |
| Purpose | fleet-wide scope reaching every agent | gives admin-created stacks an owner |

`admin-generator` needs no guard here, and the reason is non-obvious: in `verify_pak` (`api/v1/middleware.rs:234-259`) the **admin-role check runs before the generator lookup**, so the admin PAK resolves to `admin: true, generator: None` and never reaches the generator branch. Its `pak_hash` is a shadow owner record, not a usable generator identity — anyone presenting it is already admin and takes the admin path through this handler regardless.

So `__system__` is the only case needing a decision. That property holds by omission, so it is worth *enforcing* rather than relying on.

### DECIDED (Dylan, 2026-07-28): reject system generators — they are not "real"

**A system generator is not a tenant, so it does not get a tenant's view.** `__system__` is internal infrastructure — a delivery scope that reaches every agent — not an application scope owned by anyone. It is already excluded from `GET /generators` and from the console's tenant selector on exactly this reasoning; a generator-scoped agent listing is the same kind of surface and should be consistent with them.

Implement as an explicit guard: if the authenticated generator has `is_system = true`, return 403 rather than a fleet-wide listing. Do **not** rely on "it has no PAK, so this is unreachable" — that invariant lives in `provision_system_generator` and nothing near this handler would notice if it changed.

Consequences to carry through:

- **Put the guard in a shared helper, not inline.** The ticket's "secondary surfaces" section already anticipates generator-scoped `get_agent`, `search_agent`, and per-agent reads. Each would need the identical check, and a guard copied four times is a guard that will be forgotten on the fifth. Something like `require_tenant_generator(&auth_payload, &dal) -> Result<Uuid, ApiError>` that resolves the generator, rejects `is_system`, and hands back the id.
- **The guard must load the generator**, since `AuthPayload` carries only `generator: Option<Uuid>` and not the `is_system` flag. That is one extra read per scoped request; the auth cache stores `AuthPayload`, not the generator row, so it does not help here. If that read shows up in profiles, the fix is to widen the cached payload rather than to drop the check.
- **`admin-generator` is unaffected** and must stay that way — it is `is_system = false`, and an `is_system` guard correctly ignores it. It never reaches the generator branch of `verify_pak` anyway (admin is checked first), so nothing changes for admin callers.
- **The error should be distinguishable** from "you are not a generator at all". A caller holding a system-generator credential is in a genuinely different situation from an unauthenticated one, and a distinct code (e.g. `system_generator_not_a_tenant`) makes that legible instead of looking like a broken PAK.

### Open question: soft-deleted agents

`agents().list()` and `agents().list_all()` differ in soft-delete handling. The new method should match whatever `list()` does so the admin and generator paths agree; worth confirming rather than assuming, since a generator seeing deleted agents (or not seeing live ones) would be a quiet inconsistency between the two branches.

### Secondary surfaces

Whatever is decided here, these should follow or be explicitly deferred:
- **`GET /agents/` (search_agent)** and the per-agent reads (`get_agent`, `list_labels`, `get_associated_stacks`) are also admin-gated. A generator that can list agents but cannot read one of them by id has a listing it cannot follow up on. At minimum, `get_agent` should accept a generator registered with that agent.
- **CLI**: `brokkr agent list` under a generator PAK currently fails; it should work and show the scoped view.
- **SDKs**: no signature change (same endpoint, same response type), but the three clients' docs claim admin-only for this call.
- **`reference/` docs + OpenAPI**: `security(("admin_pak" = []))` on the utoipa annotation becomes admin-or-generator, which changes the generated spec and therefore all three SDKs' generated docs.

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] A generator PAK calling `GET /api/v1/agents` receives exactly the agents registered to that generator, and admin behavior (including the `scope.pak_id` tenant-view filter) is unchanged.
- [ ] A generator PAK does not see agents registered only to other generators — asserted by an integration test with two generators and at least one agent each.
- [ ] A generator PAK whose generator has `is_system = true` is rejected with 403 and a distinct error code, **not** given a fleet-wide listing — asserted by a test that provisions a PAK for the system generator directly at the DAL layer rather than assuming the API cannot mint one.
- [ ] The `is_system` rejection lives in a shared helper reusable by the secondary surfaces, not inline in `list_agents`.
- [ ] `admin-generator` is confirmed unaffected: an admin caller still takes the admin branch and sees every agent.
- [ ] Soft-delete semantics match the admin `list()` path.
- [ ] A caller with neither admin nor generator identity still gets 403, with an error code consistent with `stacks_list_denied`.
- [ ] OpenAPI security annotation updated, `angreal openapi check` passes, and the SDK docs no longer claim admin-only.
- [ ] Reference docs state the scoping rule and that it is registration-derived, so operators know the listing reflects agent consent rather than label matching.

## Status Updates

**2026-07-29 — DONE** on branch `docs/tenancy-review-2026-07` (commit `ac958b5`). **529 integration tests pass (7 new), 147 unit, `angreal openapi check` + both SDK drift checks clean, `angreal docs build` passes.**

### Correction to this ticket's premise: `GET /generators/{id}/registered-agents` already exists

Found while updating the docs, not while writing the ticket — the endpoint is at `api/v1/generators.rs:601`, is already `security(("admin_pak"), ("generator_pak"))`, and already lets a generator ask which agents are registered with it. The Objective above says a generator "cannot ask which agents are registered to it." **That is wrong as written.**

What is true, and is the actual gap: that endpoint returns `Vec<AgentGeneratorRegistration>` — `{id, agent_id, generator_id, registered_at}`. A tenant gets a list of **opaque UUIDs** and no way to resolve them, because `GET /agents` *and* `GET /agents/{id}` are both admin-only. So the tenant can learn that four agents serve it and nothing else: not a name, not a cluster, not whether any of them is alive. Answering "which agents serve me?" in any useful sense still required an admin PAK.

So the feature stands and the fix is unchanged, but the framing was sloppy: this is **resolving ids into detail**, not discovering the ids. Recorded rather than silently rewritten because the mistake is instructive — the ticket asserted absence after checking `agents.rs` and not `generators.rs`.

### That correction surfaced a real hole

`list_generator_registered_agents` had **no system-generator guard**. Since every agent is auto-registered with `__system__`, a system-generator PAK could have enumerated the entire fleet's agent ids through it — the same hole this ticket closes for `GET /agents`, already present on a shipped endpoint. Guarded it there too, reusing the generator lookup that handler already performs (no extra query). Non-admin only: an admin scoping to the system generator is legitimate and unaffected.

Not reachable today — `provision_system_generator` sets no `pak_hash` — so this is defense in depth, consistent with the decision to guard rather than rely on that omission.

### Implemented

- `agents().list_for_generator()` (`dal/agents.rs`) — inner join on `agent_generator_registrations`, filtered `deleted_at.is_null()` to match `list()`. **Resolves the ticket's open question**: `list()` filters soft-deleted and `stacks::list_for_generator` does too, so the new method does as well; admin and generator branches cannot disagree about which agents exist.
- `require_tenant_generator()` (`api/v1/middleware.rs`) — shared, next to `AuthPayload` so `stacks.rs`/`generators.rs` can use it, not private to `agents.rs`. Resolves the generator, rejects `is_system`, returns the id. Three distinct 403s: `generator_required`, `system_generator_not_a_tenant`, `generator_not_found`.
- `list_agents` branches admin → `list()`, else → `require_tenant_generator` + `list_for_generator`, mirroring `list_stacks`.

**Caught while editing, not by the tests:** `list_agents` writes `brokkr_active_agents` and per-agent heartbeat-age gauges on every call. These are **fleet-wide**, so a tenant-scoped listing would have made the gauge report whichever tenant polled most recently — a monitoring corruption with no visible symptom. Metrics writes are now admin-path-only, matching the "only an unscoped admin listing reflects the true total" comment already in `list_stacks`.

**Deliberately not done:** no `?pak_id=` scope parameter on `GET /agents`. `list_stacks` has one, but `list_agents` never did, and adding it is a console feature rather than part of this. Noted as a follow-up, not silently absorbed.

### Tests — `tests/integration/api/generator_agent_listing.rs` (7)

Isolation between two tenants; the core listing; empty listing for a generator with no registrations; soft-delete parity; agent PAK rejected with `generator_required`; admin still sees the fleet.

The system-generator test **mints a PAK for `__system__` at the DAL layer** rather than asserting the API cannot mint one — testing the latter would exercise `provision_system_generator`, not this handler. It also asserts no agent id appears in the refusal body, so the test fails loudly if the guard is removed and the fleet leaks.

### Docs updated

`security-model.md` endpoint matrix (the `/api/v1/agents/*` row said "No / No / Yes" and is now split), `reference/multi-tenancy.md` (the "cannot manage agents" claim was stale), `reference/generators.md`, `reference/api/README.md`, and `reference/error-codes.md` (both new codes). OpenAPI annotation now `admin_pak` + `generator_pak`.

### Pre-existing spec drift found by regenerating

`angreal openapi check` was **already failing on this branch before this change** — verified by stashing the work and re-running (exit 1). Commit `39f2b0a` edited the `WebhookFilters` docstring (correcting which `deployment.*` payloads carry `stack_id`) without re-exporting the spec, so `openapi/brokkr-v1.json` and both generated SDKs were stale against the source. The committed spec and committed SDKs agreed with *each other*, which is why the SDK drift checks passed and nothing surfaced it.

Regenerating here corrected all of it: spec, its `crates/brokkr-client/spec/` mirror, the Python SDK (4 webhook models), and the TypeScript schema. Unrelated to this feature but folded in because the export could not be done partially.

**Worth a follow-up:** e2e runs only on release/nightly, and `angreal openapi check` evidently is not gating this branch either — a docstring edit silently invalidated the published spec and both SDKs for several commits.