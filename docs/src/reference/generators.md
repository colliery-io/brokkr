# Generators API Reference

This reference documents the API endpoints for managing generators in Brokkr.

## Overview

Generators are identity principals that enable external systems (CI/CD pipelines, automation tools) and teams to authenticate with Brokkr and manage resources. Each generator has its own Pre-Authentication Key (PAK) and can only access resources it created.

**A generator is also Brokkr's tenant.** Onboarding a team or an application onto a shared broker means creating a generator and handing over its PAK: stacks carry the owning `generator_id`, templates are private to their generator unless they are system templates, and agents must register with a generator before its stacks reach them. Generators are what the operator console's tenant scope selector lists (`GET /api/v1/paks`) and what `?pak_id=` narrows a listing to. See [Multi-Tenancy](./multi-tenancy.md) for the full tenant model.

An agent must be registered with a generator before any stack owned by that generator is associated with the agent. Registration is the agent's opt-in consent boundary and applies to every association path — explicit targets, label matches, and annotation matches — and cannot be bypassed by admin. A singleton system generator (`is_system = true`, excluded from the `GET /generators` and `GET /paks` listings) is provisioned at broker startup and carries fleet/system stacks that reach all agents without per-agent registration. Agents are registered with it automatically: both agent-creation paths (`POST /api/v1/agents` and `brokkr-broker create agent`) register the new agent with it, and any agents that already existed when it was first provisioned are back-filled at that moment. For the concept, see [Generator Registration and Application Scopes](../explanation/security-model.md#generator-registration-and-application-scopes); for operational steps, see [Agent Registration](../how-to/agent-registration.md).

## Data Model

### Generator Object

| Field | Type | Description |
|-------|------|-------------|
| `id` | UUID | Unique identifier |
| `name` | string | Human-readable name (unique, non-null) |
| `description` | string | Optional description |
| `pak_hash` | string | Hashed PAK (never returned in API responses) |
| `created_at` | timestamp | Creation timestamp |
| `updated_at` | timestamp | Last update timestamp |
| `deleted_at` | timestamp | Soft-delete timestamp (null if active) |
| `last_active_at` | timestamp | Last activity timestamp (null if never active) |
| `is_active` | boolean | Whether the generator is currently active |
| `is_system` | boolean | `true` only for the singleton system generator provisioned at broker startup; all other generators are `false` |

### NewGenerator Object

Used when creating a generator:

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `name` | string | Yes | Unique name for the generator |
| `description` | string | No | Optional description |

## API Endpoints

### List Generators

List all generators. Requires admin access.

```
GET /api/v1/generators
Authorization: Bearer <admin_pak>
```

**Response: 200 OK**

```json
[
  {
    "id": "a1b2c3d4-e5f6-7890-abcd-ef1234567890",
    "name": "github-actions-prod",
    "description": "Production deployment pipeline",
    "created_at": "2025-01-02T10:00:00Z",
    "updated_at": "2025-01-02T10:00:00Z",
    "deleted_at": null,
    "last_active_at": null,
    "is_active": true,
    "is_system": false
  }
]
```

**Error Responses:**

| Status | Description |
|--------|-------------|
| 403 | Admin access required |
| 500 | Internal server error |

---

### List Tenants (Named PAKs)

List generators reduced to identity only, for a tenant scope selector. Requires admin access; the operator console's ephemeral read-only PAK is a read-only admin and qualifies.

The path is `/api/v1/paks`. It is grouped under the `auth` tag in the OpenAPI document but is **not** nested under `/api/v1/auth/`.

```
GET /api/v1/paks
Authorization: Bearer <admin_pak>
```

**Response: 200 OK**

```json
[
  { "id": "a1b2c3d4-e5f6-7890-abcd-ef1234567890", "name": "team-acme" },
  { "id": "b2c3d4e5-f6a7-8901-bcde-f12345678901", "name": "team-globex" }
]
```

Returns all non-deleted, non-system generators. The system generator is never included. For full generator metadata, use `GET /api/v1/generators`.

**Error Responses:**

| Status | Description |
|--------|-------------|
| 403 | `admin_required` — admin access required |
| 500 | Internal server error |

The IDs returned here are the values accepted by the `?pak_id=` query parameter on `GET /fleet`, `GET /stacks`, and `GET /agent-events`. That parameter is a **view filter, not an authorization boundary** — see [Multi-Tenancy](./multi-tenancy.md#scoped-views-the-pak_id-query-parameter).

---

### Create Generator

Create a new generator and receive its PAK. Requires admin access.

```
POST /api/v1/generators
Authorization: Bearer <admin_pak>
Content-Type: application/json
```

**Request Body:**

```json
{
  "name": "github-actions-prod",
  "description": "Production deployment pipeline"
}
```

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `name` | string | Yes | Unique name (max 255 characters) |
| `description` | string | No | Optional description |

**Response: 201 Created**

```json
{
  "generator": {
    "id": "a1b2c3d4-e5f6-7890-abcd-ef1234567890",
    "name": "github-actions-prod",
    "description": "Production deployment pipeline",
    "created_at": "2025-01-02T10:00:00Z",
    "updated_at": "2025-01-02T10:00:00Z",
    "deleted_at": null,
    "last_active_at": null,
    "is_active": true,
    "is_system": false
  },
  "pak": "brokkr_BRgen12ab_GeneratorLongTokenExample01"
}
```

The `pak` field is only returned once at creation time. Store it securely immediately.

**Error Responses:**

| Status | Description |
|--------|-------------|
| 400 | Invalid generator data |
| 409 | Duplicate generator name (`unique_violation`) |
| 403 | Admin access required |
| 500 | Internal server error |

---

### Get Generator

Retrieve a specific generator by ID. Accessible by admin or the generator itself.

```
GET /api/v1/generators/{id}
Authorization: Bearer <admin_pak | generator_pak>
```

**Path Parameters:**

| Parameter | Type | Description |
|-----------|------|-------------|
| `id` | UUID | Generator ID |

**Response: 200 OK**

```json
{
  "id": "a1b2c3d4-e5f6-7890-abcd-ef1234567890",
  "name": "github-actions-prod",
  "description": "Production deployment pipeline",
  "created_at": "2025-01-02T10:00:00Z",
  "updated_at": "2025-01-02T10:00:00Z",
  "deleted_at": null,
  "last_active_at": null,
  "is_active": true,
  "is_system": false
}
```

**Error Responses:**

| Status | Description |
|--------|-------------|
| 403 | Unauthorized access (not admin and not the generator) |
| 404 | Generator not found |
| 500 | Internal server error |

---

### Update Generator

Update a generator's metadata. Accessible by admin or the generator itself.

```
PUT /api/v1/generators/{id}
Authorization: Bearer <admin_pak | generator_pak>
Content-Type: application/json
```

**Path Parameters:**

| Parameter | Type | Description |
|-----------|------|-------------|
| `id` | UUID | Generator ID |

**Request Body:** a complete Generator object.

This is a full replacement, not a patch. The body is deserialized as a whole Generator, so a partial body such as `{"name": "...", "description": "..."}` is rejected with `422` before the handler runs. The supported pattern is fetch-modify-PUT: `GET /api/v1/generators/{id}`, change the fields you want, and send the result back.

```json
{
  "id": "a1b2c3d4-e5f6-7890-abcd-ef1234567890",
  "name": "github-actions-prod",
  "description": "Updated description",
  "created_at": "2025-01-02T10:00:00Z",
  "updated_at": "2025-01-02T10:00:00Z",
  "deleted_at": null,
  "last_active_at": "2025-01-02T10:45:00Z",
  "is_active": true,
  "is_system": false
}
```

Field handling:

| Field | Behavior on update |
|-------|--------------------|
| `id` | Ignored — the path parameter identifies the row |
| `pak_hash` | Ignored — never accepted on input; rotate the PAK with the rotate endpoint |
| `updated_at` | Ignored — a database trigger sets it to the current time on every update |
| `created_at` | **Written as supplied.** Send back the value returned by `GET`, or the generator's creation timestamp is overwritten |
| `name`, `description`, `is_active`, `is_system` | Written as supplied |
| `deleted_at`, `last_active_at` | Written when non-null; a `null` leaves the stored value unchanged |

**Response: 200 OK**

```json
{
  "id": "a1b2c3d4-e5f6-7890-abcd-ef1234567890",
  "name": "github-actions-prod",
  "description": "Updated description",
  "created_at": "2025-01-02T10:00:00Z",
  "updated_at": "2025-01-02T11:00:00Z",
  "deleted_at": null,
  "last_active_at": "2025-01-02T10:45:00Z",
  "is_active": true,
  "is_system": false
}
```

**Error Responses:**

| Status | Description |
|--------|-------------|
| 403 | Unauthorized access |
| 404 | Generator not found |
| 422 | Request body is not a complete Generator object (plain-text body, no error envelope) |
| 500 | Internal server error |

---

### Delete Generator

Soft-delete a generator. Accessible by admin or the generator itself.

```
DELETE /api/v1/generators/{id}
Authorization: Bearer <admin_pak | generator_pak>
```

**Path Parameters:**

| Parameter | Type | Description |
|-----------|------|-------------|
| `id` | UUID | Generator ID |

**Response: 204 No Content**

The generator is soft-deleted (marked with `deleted_at` timestamp). A database trigger cascades the soft-delete to all stacks owned by this generator and their deployment objects.

**Error Responses:**

| Status | Description |
|--------|-------------|
| 403 | Unauthorized access |
| 404 | Generator not found |
| 500 | Internal server error |

---

### Rotate Generator PAK

Generate a new PAK for the generator, invalidating the previous one. Accessible by admin or the generator itself.

```
POST /api/v1/generators/{id}/rotate-pak
Authorization: Bearer <admin_pak | generator_pak>
```

**Path Parameters:**

| Parameter | Type | Description |
|-----------|------|-------------|
| `id` | UUID | Generator ID |

**Response: 201 Created**

```json
{
  "generator": {
    "id": "a1b2c3d4-e5f6-7890-abcd-ef1234567890",
    "name": "github-actions-prod",
    "description": "Production deployment pipeline",
    "created_at": "2025-01-02T10:00:00Z",
    "updated_at": "2025-01-02T12:00:00Z",
    "deleted_at": null,
    "last_active_at": "2025-01-02T11:30:00Z",
    "is_active": true,
    "is_system": false
  },
  "pak": "brokkr_BRnew34cd_GeneratorLongTokenExample02"
}
```

The old PAK is immediately invalidated. Store the new PAK securely and update all systems using the old PAK.

**Error Responses:**

| Status | Description |
|--------|-------------|
| 403 | Unauthorized access |
| 404 | Generator not found |
| 500 | Internal server error |

---

### Register Agent

Register an agent with the generator, permitting the generator's stacks to be targeted at that agent. Accessible by admin, or by the agent acting on itself (a generator PAK is rejected with `403 forbidden`). Not idempotent: re-registering an already-registered pair returns `409 already_registered`.

```
POST /api/v1/generators/{id}/register
Authorization: Bearer <admin_pak | agent_pak>
Content-Type: application/json
```

**Path Parameters:**

| Parameter | Type | Description |
|-----------|------|-------------|
| `id` | UUID | Generator ID |

**Request Body (`AgentRegistrationBody`):**

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `agent_id` | UUID | No | Agent to register. Omit when an agent self-registers; admin supplies it to register another agent. |

**Response: 201 Created (`AgentGeneratorRegistration`)**

```json
{
  "id": "f0e1d2c3-b4a5-6789-abcd-ef0123456789",
  "agent_id": "11112222-3333-4444-5555-666677778888",
  "generator_id": "a1b2c3d4-e5f6-7890-abcd-ef1234567890",
  "registered_at": "2025-01-02T12:30:00Z"
}
```

**Error Responses:**

| Status | Description |
|--------|-------------|
| 400 | `missing_agent_id` — an admin caller omitted `agent_id` from the body |
| 403 | `forbidden` — caller is a generator (only an agent self or an admin may register) |
| 404 | `generator_not_found` |
| 409 | `already_registered` — the agent is already registered with this generator |
| 500 | Internal server error |

---

### Deregister Agent

Remove an agent's registration with the generator. Accessible by admin, or by the agent acting on itself (a generator PAK is rejected with `403 forbidden`). Destructive: this cascades, removing the agent's `agent_targets` for that generator's stacks and pushing a `TargetChanged` frame to the agent over its WebSocket connection (the agent prunes those Kubernetes resources on its next reconcile). Deregistering an agent that was never registered is a no-op that still returns `204`.

```
DELETE /api/v1/generators/{id}/register
Authorization: Bearer <admin_pak | agent_pak>
Content-Type: application/json
```

**Path Parameters:**

| Parameter | Type | Description |
|-----------|------|-------------|
| `id` | UUID | Generator ID |

**Request Body (`AgentRegistrationBody`):**

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `agent_id` | UUID | No | Agent to deregister. Omit when an agent self-deregisters; admin supplies it to deregister another agent. |

**Response: 204 No Content**

**Error Responses:**

| Status | Description |
|--------|-------------|
| 400 | `missing_agent_id` — an admin caller omitted `agent_id` from the body |
| 403 | `forbidden` — caller is a generator (only an agent self or an admin may deregister) |
| 404 | `generator_not_found` |
| 500 | Internal server error |

---

### List Registered Agents

List the agents registered with the generator. Accessible by admin or the generator itself.

Returns **registration records** — `agent_id` and `registered_at` — not agent detail. To resolve those ids into names, clusters, status, and heartbeats, use `GET /api/v1/agents`, which a generator PAK may call for exactly the agents registered with it (see [Agents](api/README.md)).

```
GET /api/v1/generators/{id}/registered-agents
Authorization: Bearer <admin_pak | generator_pak>
```

**Path Parameters:**

| Parameter | Type | Description |
|-----------|------|-------------|
| `id` | UUID | Generator ID |

**Response: 200 OK** — a list of `AgentGeneratorRegistration` objects.

**Error Responses:**

| Status | Description |
|--------|-------------|
| 403 | Unauthorized access, or `system_generator_not_a_tenant` when a non-admin caller scopes to the system generator |
| 404 | Generator not found |
| 500 | Internal server error |

> The system generator is excluded from the generator-PAK path. Every agent is auto-registered with it, so scoping this read to it would enumerate the whole fleet through a non-admin credential. Admin callers are unaffected.

---

### List Agent Registrations

List the generator registrations held by a given agent. Accessible by admin or the agent itself.

```
GET /api/v1/agents/{id}/registrations
Authorization: Bearer <admin_pak | agent_pak>
```

**Path Parameters:**

| Parameter | Type | Description |
|-----------|------|-------------|
| `id` | UUID | Agent ID |

**Response: 200 OK** — a list of `AgentGeneratorRegistration` objects.

**Error Responses:**

| Status | Description |
|--------|-------------|
| 403 | Unauthorized access |
| 404 | Agent not found |
| 500 | Internal server error |

## Authentication

### PAK Format

All PAKs — admin, agent, generator, and the console's ephemeral read-only PAK — share the same format: `<prefix>_<short-token>_<long-token>`, by default `brokkr_BR<8 chars>_<24 chars>` (configured by the `pak.*` settings; see [Environment Variables](./environment-variables.md)). The token itself does not encode the role: the broker determines which identity a PAK belongs to by hash lookup. Use `POST /api/v1/auth/pak` to discover which identity a PAK resolves to; its `generator` field is the caller's tenant ID, and `readonly` is `true` only for the console PAK.

### Authorization Header

Include the PAK in the Authorization header:

```
Authorization: Bearer brokkr_BRgen12ab_GeneratorLongTokenExample01
```

### Permission Model

| Operation | Admin PAK | Generator PAK (own) | Generator PAK (other) |
|-----------|-----------|---------------------|----------------------|
| List generators | Yes | No | No |
| List tenants (`GET /paks`) | Yes | No | No |
| Create generator | Yes | No | No |
| Get generator | Yes | Yes | No |
| Update generator | Yes | Yes | No |
| Delete generator | Yes | Yes | No |
| Rotate PAK | Yes | Yes | No |
| Register / deregister an agent | Yes | No (`403 forbidden`) | No |
| List registered agents | Yes | Yes | No |

## Resource Scoping

Resources created by a generator are scoped to that generator:

### Stacks

When a generator creates a stack, the stack's `generator_id` is set to the generator's ID. The generator can only view and modify its own stacks.

### Agent Registration

Before any of a generator's stacks are associated with an agent, the agent must be registered with that generator. Registration is explicit (an opt-in boundary) and applies to all three association paths:

| Path | Enforcement |
|------|-------------|
| Explicit target | Gated at creation and removal (`POST /agents/{id}/targets`, `DELETE /agents/{id}/targets/{stack_id}`). An unregistered agent yields error code [`agent_not_registered`](./error-codes.md) (HTTP 403), and admin cannot bypass it |
| Shared label | Filtered at read time — a label match associates the stack only when the agent is registered with the stack's owning generator |
| Shared annotation | Filtered at read time, on the same rule |

An agent with no registrations therefore receives nothing from label or annotation matching, and two generators can safely reuse the same label and annotation vocabulary. The served set returned by `GET /agents/{id}/target-state` reflects this rule.

Registration with the system generator is automatic, so system/fleet stacks reach all agents without per-agent registration: both `POST /api/v1/agents` and `brokkr-broker create agent` register the new agent with it, and pre-existing agents are back-filled when it is first provisioned. An agent created by the CLI while the system generator does not yet exist (a broker whose `serve` has never run) is the one case that is left unregistered; the CLI logs a warning, and the agent must then be registered explicitly. Every other generator is registered explicitly, via the registration endpoints above or at agent startup. See [Agent Registration](../how-to/agent-registration.md) for operations and [Multi-Tenancy](./multi-tenancy.md) for the tenant model this enforces.

### Templates

Templates can be:
- **Generator-scoped**: created by a generator, and readable, modifiable, and instantiable only by that generator
- **System templates**: created by admin (`generator_id` is null), readable and instantiable by every generator — the sanctioned cross-tenant sharing mechanism

A generator that reads or instantiates another generator's template receives `403` with error code `template_not_accessible`. This applies to `GET /templates/{id}`, the template label and annotation reads, and `POST /stacks/{id}/deployment-objects/from-template`. Admin may access any template.

### Deployment Objects

Deployment objects inherit the `generator_id` from their parent stack.

## Database Schema

### generators Table

| Column | Type | Constraints |
|--------|------|-------------|
| `id` | UUID | PRIMARY KEY, DEFAULT uuid_generate_v4() |
| `name` | VARCHAR(255) | NOT NULL (uniqueness comes from the partial index below, not a column constraint) |
| `description` | TEXT | |
| `pak_hash` | TEXT | |
| `created_at` | TIMESTAMP | NOT NULL, DEFAULT NOW() |
| `updated_at` | TIMESTAMP | NOT NULL, DEFAULT NOW() |
| `deleted_at` | TIMESTAMP | NULL (soft delete) |
| `last_active_at` | TIMESTAMP | NULL |
| `is_active` | BOOLEAN | NOT NULL, DEFAULT true |
| `is_system` | BOOLEAN | NOT NULL, DEFAULT false |

### Unique Constraint

The `name` column has a partial unique index excluding soft-deleted rows:

```sql
CREATE UNIQUE INDEX unique_generator_name
ON generators (name)
WHERE deleted_at IS NULL;
```

This allows reusing names after a generator is deleted.

## Related Documentation

- [Working with Generators](../how-to/generators.md) - How-to guide
- [Agent Registration](../how-to/agent-registration.md) - Registering agents with generators for targeting
- [Stack Templates](../how-to/templates.md) - Using templates with generators
- [Security Model](../explanation/security-model.md#generator-registration-and-application-scopes) - Generator registration and the targeting authorization gate
- [Multi-Tenancy](./multi-tenancy.md) - The tenant model, tenant listing, and `pak_id` scoped views
- [Multi-Tenant Setup](../how-to/multi-tenant-setup.md) - Onboarding teams onto a shared broker
