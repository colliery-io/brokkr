# Generators API Reference

This reference documents the API endpoints for managing generators in Brokkr.

## Overview

Generators are identity principals that enable external systems (CI/CD pipelines, automation tools) to authenticate with Brokkr and manage resources. Each generator has its own Pre-Authentication Key (PAK) and can only access resources it created.

A generator also defines an application-level tenant scope. An agent must be registered with a generator before any stack owned by that generator can be targeted at the agent; this registration is the agent's opt-in consent boundary and is enforced at target-creation time (it cannot be bypassed by admin). A singleton system generator (`is_system = true`, excluded from the `GET /generators` listing) is provisioned at broker startup and auto-registers every agent at creation, carrying fleet/system stacks that reach all agents without per-agent registration. For the concept, see [Generator Registration and Application Scopes](../explanation/security-model.md#generator-registration-and-application-scopes); for operational steps, see [Agent Registration](../how-to/agent-registration.md).

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
    "is_active": true
  }
]
```

**Error Responses:**

| Status | Description |
|--------|-------------|
| 403 | Admin access required |
| 500 | Internal server error |

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
    "is_active": true
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
  "is_active": true
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

**Request Body:**

```json
{
  "id": "a1b2c3d4-e5f6-7890-abcd-ef1234567890",
  "name": "github-actions-prod",
  "description": "Updated description"
}
```

All fields from the Generator object can be provided, though `id`, `created_at`, and `pak_hash` are ignored if included.

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
  "is_active": true
}
```

**Error Responses:**

| Status | Description |
|--------|-------------|
| 403 | Unauthorized access |
| 404 | Generator not found |
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
    "is_active": true
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
| 403 | `forbidden` — caller is a generator (only an agent self or an admin may deregister) |
| 404 | `generator_not_found` |
| 500 | Internal server error |

---

### List Registered Agents

List the agents registered with the generator. Accessible by admin or the generator itself.

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
| 403 | Unauthorized access |
| 404 | Generator not found |
| 500 | Internal server error |

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

All PAKs — admin, agent, and generator — share the same format: `<prefix>_<short-token>_<long-token>`, by default `brokkr_BR<8 chars>_<24 chars>` (configured by the `pak.*` settings; see [Environment Variables](./environment-variables.md)). The token itself does not encode the role: the broker determines whether a PAK belongs to the admin role, an agent, or a generator by hash lookup. Use `POST /api/v1/auth/pak` to discover which identity a PAK resolves to.

### Authorization Header

Include the PAK in the Authorization header:

```
Authorization: Bearer brokkr_BRgen12ab_GeneratorLongTokenExample01
```

### Permission Model

| Operation | Admin PAK | Generator PAK (own) | Generator PAK (other) |
|-----------|-----------|---------------------|----------------------|
| List generators | Yes | No | No |
| Create generator | Yes | No | No |
| Get generator | Yes | Yes | No |
| Update generator | Yes | Yes | No |
| Delete generator | Yes | Yes | No |
| Rotate PAK | Yes | Yes | No |

## Resource Scoping

Resources created by a generator are scoped to that generator:

### Stacks

When a generator creates a stack, the stack's `generator_id` is set to the generator's ID. The generator can only view and modify its own stacks.

### Agent Registration

Before any of a generator's stacks can be targeted at an agent, the agent must be registered with that generator. Registration is explicit (an opt-in boundary) and is enforced when a target is created or removed (`POST /agents/{id}/targets` and `DELETE /agents/{id}/targets/{stack_id}`); an unregistered agent yields error code [`agent_not_registered`](./error-codes.md) (HTTP 403), and admin cannot bypass it. The system generator auto-registers every agent at creation, so system/fleet stacks reach all agents without per-agent registration. Application-scoped generators are registered explicitly, via the registration endpoints above or at agent startup. Registration gates only the *creation* of explicit targets; the agent's read-time served-stack set (`GET /agents/{id}/target-state`) is unchanged. See [Agent Registration](../how-to/agent-registration.md) for operations and [Multi-Tenancy](./multi-tenancy.md) for how this application-level isolation relates to schema-per-tenant deployment isolation.

### Templates

Templates can be:
- **Generator-scoped**: Created by a generator, only visible to that generator
- **System templates**: Created by admin (no `generator_id`), visible to all generators

### Deployment Objects

Deployment objects inherit the `generator_id` from their parent stack.

## Database Schema

### generators Table

| Column | Type | Constraints |
|--------|------|-------------|
| `id` | UUID | PRIMARY KEY, DEFAULT uuid_generate_v4() |
| `name` | VARCHAR(255) | NOT NULL, UNIQUE |
| `description` | TEXT | |
| `pak_hash` | VARCHAR(255) | |
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
- [Multi-Tenancy](./multi-tenancy.md) - Application-level vs deployment-level isolation
