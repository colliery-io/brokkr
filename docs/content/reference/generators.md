---
title: "Generators API Reference"
weight: 7
description: "Complete API reference for Brokkr generators"
---

# Generators API Reference

This reference documents the API endpoints for managing generators in Brokkr.

## Overview

Generators are identity principals that enable external systems (CI/CD pipelines, automation tools) to authenticate with Brokkr and manage resources. Each generator has its own Pre-Authentication Key (PAK) and can only access resources it created.

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
    "deleted_at": null
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
    "deleted_at": null
  },
  "pak": "brk_gen_abc123...xyz789"
}
```

The `pak` field is only returned once at creation time. Store it securely immediately.

**Error Responses:**

| Status | Description |
|--------|-------------|
| 400 | Invalid generator data (e.g., duplicate name) |
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
  "deleted_at": null
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
  "deleted_at": null
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

The generator is soft-deleted (marked with `deleted_at` timestamp). Its resources (stacks, templates, deployment objects) are not affected.

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

**Response: 200 OK**

```json
{
  "generator": {
    "id": "a1b2c3d4-e5f6-7890-abcd-ef1234567890",
    "name": "github-actions-prod",
    "description": "Production deployment pipeline",
    "created_at": "2025-01-02T10:00:00Z",
    "updated_at": "2025-01-02T12:00:00Z",
    "deleted_at": null
  },
  "pak": "brk_gen_new123...newxyz"
}
```

The old PAK is immediately invalidated. Store the new PAK securely and update all systems using the old PAK.

**Error Responses:**

| Status | Description |
|--------|-------------|
| 403 | Unauthorized access |
| 404 | Generator not found |
| 500 | Internal server error |

## Authentication

### PAK Format

Generator PAKs follow the format: `brk_gen_<random_string>`

The prefix identifies this as a generator PAK (as opposed to admin or agent PAKs).

### Authorization Header

Include the PAK in the Authorization header:

```
Authorization: Bearer brk_gen_abc123...xyz789
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

### Unique Constraint

The `name` column has a partial unique index excluding soft-deleted rows:

```sql
CREATE UNIQUE INDEX generators_name_unique
ON generators (name)
WHERE deleted_at IS NULL;
```

This allows reusing names after a generator is deleted.

## Related Documentation

- [Working with Generators](/how-to/generators) - How-to guide
- [Stack Templates](/how-to/templates) - Using templates with generators
- [Authentication](/explanation/security-model) - Security model overview
