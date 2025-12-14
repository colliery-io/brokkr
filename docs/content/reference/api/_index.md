---
title: "API Reference"
description: "Complete API documentation for Brokkr"
weight: 1
---

# API Reference

Brokkr provides a comprehensive REST API for managing deployments, agents, stacks, templates, and work orders across your Kubernetes clusters.

## Interactive API Documentation

The Brokkr broker includes an interactive Swagger UI that provides complete API documentation with:

- All available endpoints with request/response schemas
- Authentication requirements for each endpoint
- Try-it-out functionality for testing endpoints
- Example requests and responses

**Access Swagger UI at:** `http://<broker-url>/swagger-ui`

**OpenAPI spec available at:** `http://<broker-url>/docs/openapi.json`

## API Overview

All API endpoints are prefixed with `/api/v1/` and require authentication via PAK (Pre-Authenticated Key) in the `Authorization` header.

### Authentication

```bash
# All requests require a PAK in the Authorization header
curl -H "Authorization: Bearer <your-pak>" http://localhost:3000/api/v1/...
```

There are three types of PAKs:
- **Admin PAK**: Full access to all endpoints
- **Agent PAK**: Access to agent-specific endpoints (target state, events, heartbeat)
- **Generator PAK**: Access to create deployment objects for assigned stacks

### Core Resources

#### Stacks
Stacks are collections of Kubernetes resources managed as a unit.

| Method | Endpoint | Description |
|--------|----------|-------------|
| GET | `/stacks` | List all stacks |
| POST | `/stacks` | Create a new stack |
| GET | `/stacks/:id` | Get stack by ID |
| PUT | `/stacks/:id` | Update a stack |
| DELETE | `/stacks/:id` | Delete a stack |
| GET | `/stacks/:id/labels` | List stack labels |
| POST | `/stacks/:id/labels` | Add label to stack |
| DELETE | `/stacks/:id/labels/:label` | Remove label from stack |
| GET | `/stacks/:id/annotations` | List stack annotations |
| POST | `/stacks/:id/annotations` | Add annotation to stack |
| DELETE | `/stacks/:id/annotations/:key` | Remove annotation |
| POST | `/stacks/:id/deployment-objects` | Create deployment object |
| POST | `/stacks/:id/deployment-objects/from-template` | Instantiate template |

#### Agents
Agents run in Kubernetes clusters and apply deployment objects.

| Method | Endpoint | Description |
|--------|----------|-------------|
| GET | `/agents` | List all agents |
| POST | `/agents` | Register a new agent |
| GET | `/agents/:id` | Get agent by ID |
| PUT | `/agents/:id` | Update an agent |
| DELETE | `/agents/:id` | Delete an agent |
| GET | `/agents/:id/target-state` | Get agent's target state |
| POST | `/agents/:id/heartbeat` | Record agent heartbeat |
| GET | `/agents/:id/labels` | List agent labels |
| POST | `/agents/:id/labels` | Add label to agent |
| GET | `/agents/:id/annotations` | List agent annotations |
| POST | `/agents/:id/annotations` | Add annotation to agent |
| GET | `/agents/:id/targets` | List agent's stack targets |
| POST | `/agents/:id/targets` | Add stack target |
| DELETE | `/agents/:id/targets/:stack_id` | Remove stack target |
| POST | `/agents/:id/rotate-pak` | Rotate agent PAK |

#### Templates
Reusable stack templates with Tera templating and JSON Schema validation.

| Method | Endpoint | Description |
|--------|----------|-------------|
| GET | `/templates` | List all templates |
| POST | `/templates` | Create a new template |
| GET | `/templates/:id` | Get template by ID |
| PUT | `/templates/:id` | Update a template |
| DELETE | `/templates/:id` | Delete a template |
| GET | `/templates/:id/labels` | List template labels |
| POST | `/templates/:id/labels` | Add label to template |
| GET | `/templates/:id/annotations` | List template annotations |
| POST | `/templates/:id/annotations` | Add annotation to template |

#### Work Orders
Transient operations like container builds routed to agents.

| Method | Endpoint | Description |
|--------|----------|-------------|
| GET | `/work-orders` | List all work orders |
| POST | `/work-orders` | Create a new work order |
| GET | `/work-orders/:id` | Get work order by ID |
| DELETE | `/work-orders/:id` | Cancel a work order |
| POST | `/work-orders/:id/claim` | Claim a work order (agent) |
| POST | `/work-orders/:id/complete` | Complete a work order (agent) |
| GET | `/agents/:id/work-orders/pending` | Get pending work orders for agent |
| GET | `/work-order-log` | List completed work orders |
| GET | `/work-order-log/:id` | Get completed work order details |

#### Generators
External systems that create deployment objects.

| Method | Endpoint | Description |
|--------|----------|-------------|
| GET | `/generators` | List all generators |
| POST | `/generators` | Create a new generator |
| GET | `/generators/:id` | Get generator by ID |
| PUT | `/generators/:id` | Update a generator |
| DELETE | `/generators/:id` | Delete a generator |
| POST | `/generators/:id/rotate-pak` | Rotate generator PAK |

#### Other Endpoints

| Method | Endpoint | Description |
|--------|----------|-------------|
| GET | `/agent-events` | List agent events |
| GET | `/agent-events/:id` | Get agent event by ID |
| GET | `/deployment-objects/:id` | Get deployment object by ID |
| POST | `/auth/pak` | Verify a PAK |

## Health Endpoints

The broker exposes health endpoints (not under `/api/v1/`):

| Endpoint | Description |
|----------|-------------|
| `/healthz` | Basic health check |
| `/health/live` | Liveness probe |
| `/health/ready` | Readiness probe |

See [Health Endpoints](../health-endpoints) for details.

## Error Handling

All API errors return JSON with an `error` field:

```json
{
  "error": "Description of what went wrong"
}
```

Common HTTP status codes:
- `400` - Bad request (invalid input)
- `401` - Unauthorized (missing or invalid PAK)
- `403` - Forbidden (valid PAK but insufficient permissions)
- `404` - Not found
- `422` - Unprocessable entity (validation failed)
- `500` - Internal server error

## Rate Limiting

The API does not currently implement rate limiting. For production deployments, consider placing a reverse proxy in front of the broker.
