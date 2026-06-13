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
| GET | `/stacks/:id/deployment-objects` | List deployment objects |
| POST | `/stacks/:id/deployment-objects` | Create deployment object (`application/json` envelope, or raw `application/yaml` body with `?deletion_marker=`) |
| POST | `/stacks/:id/deployment-objects/from-template` | Instantiate template |
| GET | `/stacks/:id/health` | Aggregated stack health (computed on read) |
| GET | `/stacks/:id/events` | Retained Kubernetes events (6h window; `since`, `limit`) |
| GET | `/stacks/:id/logs` | Retained pod log lines (6h window; `since`, `limit`) |

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
| POST | `/agents/:id/heartbeat` | Record agent heartbeat (agent PAK only; admin PAK rejected) |
| GET | `/agents/:id/labels` | List agent labels |
| POST | `/agents/:id/labels` | Add label to agent |
| DELETE | `/agents/:id/labels/:label` | Remove label from agent |
| GET | `/agents/:id/annotations` | List agent annotations |
| POST | `/agents/:id/annotations` | Add annotation to agent |
| DELETE | `/agents/:id/annotations/:key` | Remove annotation from agent |
| GET | `/agents/:id/stacks` | List agent's associated stacks |
| GET | `/agents/:id/targets` | List agent's stack targets |
| POST | `/agents/:id/targets` | Add stack target |
| DELETE | `/agents/:id/targets/:stack_id` | Remove stack target |
| POST | `/agents/:id/rotate-pak` | Rotate agent PAK |
| PATCH | `/agents/:id/health-status` | Agent reports deployment health |
| GET | `/agents/:id/events` | List agent events |
| POST | `/agents/:id/events` | Record agent event |
| GET | `/agents/:id/diagnostics/pending` | Pending diagnostics for agent |
| GET | `/agents/:agent_id/webhooks/pending` | Pending webhook deliveries (auto-claims) |
| GET | `/agents/` | Search agents by query string |

#### Fleet Legibility

Broker-computed, read-only fleet surface returning **measured signals only**
(no health verdicts). Admin PAK only — agent and generator PAKs cannot read the
fleet. Every field is computed on read from data the broker already holds; the
rollup is assembled with bounded grouped queries (no per-agent fan-out).

| Method | Endpoint | Description |
|--------|----------|-------------|
| GET | `/fleet` | Per-agent fleet records for all agents (rollup) |
| GET | `/agents/:id/fleet-status` | One agent's fleet record plus its 20 most recent events |

Each fleet record carries: `agent_id`, `name`, `status`, `ws_connected`,
`connected_since`, `last_heartbeat`, `heartbeat_age_seconds`,
`pending_object_count`, `pending_work_orders`, `claimed_work_orders`,
`last_event_at`, `seconds_since_last_event`, `health_failing`, and
`health_degraded`. The `*_age_seconds` / `seconds_since_*` fields are
`now - timestamp`, clamped to be non-negative.

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
| DELETE | `/templates/:id/labels/:label` | Remove label from template |
| GET | `/templates/:id/annotations` | List template annotations |
| POST | `/templates/:id/annotations` | Add annotation to template |
| DELETE | `/templates/:id/annotations/:key` | Remove annotation from template |

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

#### Webhooks

| Method | Endpoint | Description |
|--------|----------|-------------|
| GET | `/webhooks` | List webhook subscriptions |
| POST | `/webhooks` | Create webhook subscription |
| GET | `/webhooks/event-types` | List available event types |
| GET | `/webhooks/:id` | Get webhook subscription |
| PUT | `/webhooks/:id` | Update webhook subscription |
| DELETE | `/webhooks/:id` | Delete webhook subscription |
| POST | `/webhooks/:id/test` | Test webhook delivery |
| GET | `/webhooks/:id/deliveries` | List webhook deliveries |
| POST | `/webhook-deliveries/:id/result` | Report delivery result (agent) |

#### Admin

| Method | Endpoint | Description |
|--------|----------|-------------|
| GET | `/admin/audit-logs` | Query audit logs (`limit` default 100, max 1000; `offset`; actor/action/resource/time filters) |
| POST | `/admin/config/reload` | Reload broker configuration |
| GET | `/admin/ws/connections` | Snapshot of active WebSocket connections |

#### Health Monitoring & Diagnostics

| Method | Endpoint | Description |
|--------|----------|-------------|
| GET | `/deployment-objects/:id/health` | Get deployment health |
| POST | `/deployment-objects/:id/diagnostics` | Request a diagnostic collection |
| GET | `/diagnostics/:id` | Get diagnostic request and result |
| POST | `/diagnostics/:id/claim` | Claim a diagnostic request (agent) |
| POST | `/diagnostics/:id/result` | Submit diagnostic result (agent) |

### Pagination

Most list endpoints return the full collection without pagination. The exceptions take query parameters: `/admin/audit-logs` (`limit`/`offset`), the stack telemetry endpoints (`since`/`limit`, clamped to the 6-hour retention window), `/work-order-log` (`work_type`, `success`, `agent_id`, `limit`), and `/webhooks/:id/deliveries` (`status` filter).

### WebSocket Endpoints

The broker also serves two WebSocket upgrade endpoints — the internal agent channel (`/internal/ws/agent`) and the per-stack live tail (`/api/v1/stacks/:id/live`). These are not part of the OpenAPI spec; see the [WebSocket Protocol reference](../ws-protocol.md).

## Health Endpoints

The broker exposes health endpoints (not under `/api/v1/`):

| Endpoint | Description |
|----------|-------------|
| `/healthz` | Basic health check |
| `/readyz` | Readiness probe |
| `/metrics` | Prometheus metrics |

See [Health Endpoints](../health-endpoints.md) for details.

## Error Handling

All API errors return a JSON body with a stable machine-readable `code`, a human-readable `message` (wording may change between releases), and optional structured `details` (`crates/brokkr-broker/src/api/v1/error.rs`):

```json
{
  "code": "agent_not_found",
  "message": "No agent with id 550e8400-e29b-41d4-a716-446655440000",
  "details": null
}
```

Match on `code`, not `message` — the codes are part of the SDK contract. See [Stable Error Codes](../error-codes.md) for the catalog.

Common HTTP status codes:
- `400` - Bad request (invalid input)
- `401` - Unauthorized (missing or invalid PAK)
- `403` - Forbidden (valid PAK but insufficient permissions)
- `404` - Not found
- `409` - Conflict (unique constraint violations)
- `422` - Unprocessable entity (validation, foreign-key, check, or not-null violations)
- `500` - Internal server error

## Rate Limiting

The API does not currently implement rate limiting. For production deployments, consider placing a reverse proxy in front of the broker.
