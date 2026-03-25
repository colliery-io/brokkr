# How-To: Querying Audit Logs

Brokkr maintains an immutable audit trail of all significant operations. This guide shows how to query audit logs to investigate events, track changes, and monitor security.

## Audit Log API

All audit log queries go through the admin API:

```
GET /api/v1/admin/audit-logs
```

**Auth:** Admin only.

## Basic Query

List the most recent audit events:

```bash
curl -s "http://localhost:3000/api/v1/admin/audit-logs" \
  -H "Authorization: <admin-pak>" | jq .
```

Default: returns the 100 most recent entries, ordered by timestamp (newest first). The response is a JSON object with a `logs` array, plus `total`, `count`, `limit`, and `offset` fields for pagination.

## Filtering

### By Actor Type

See all actions performed by agents:

```bash
curl -s "http://localhost:3000/api/v1/admin/audit-logs?actor_type=agent" \
  -H "Authorization: <admin-pak>" | jq '.logs[] | {action, resource_type, timestamp}'
```

Valid actor types: `admin`, `agent`, `generator`, `system`

### By Action

Find all agent creation events:

```bash
curl -s "http://localhost:3000/api/v1/admin/audit-logs?action=agent.created" \
  -H "Authorization: <admin-pak>" | jq .
```

Actions support **wildcard prefix matching**. To see all agent-related events:

```bash
curl -s "http://localhost:3000/api/v1/admin/audit-logs?action=agent.*" \
  -H "Authorization: <admin-pak>" | jq .
```

### By Resource

Track all changes to a specific agent:

```bash
curl -s "http://localhost:3000/api/v1/admin/audit-logs?resource_type=agent&resource_id=${AGENT_ID}" \
  -H "Authorization: <admin-pak>" | jq .
```

### By Time Range

Query events within a specific window:

```bash
curl -s "http://localhost:3000/api/v1/admin/audit-logs?from=2025-01-15T00:00:00Z&to=2025-01-16T00:00:00Z" \
  -H "Authorization: <admin-pak>" | jq .
```

### By Actor ID

See everything a specific generator has done:

```bash
curl -s "http://localhost:3000/api/v1/admin/audit-logs?actor_type=generator&actor_id=${GEN_ID}" \
  -H "Authorization: <admin-pak>" | jq .
```

### Pagination

Use `limit` and `offset` for large result sets:

```bash
# First page
curl -s "http://localhost:3000/api/v1/admin/audit-logs?limit=50&offset=0" \
  -H "Authorization: <admin-pak>" | jq .

# Second page
curl -s "http://localhost:3000/api/v1/admin/audit-logs?limit=50&offset=50" \
  -H "Authorization: <admin-pak>" | jq .
```

Maximum `limit` is 1000.

## Common Investigation Patterns

### Who Changed This Agent?

```bash
curl -s "http://localhost:3000/api/v1/admin/audit-logs?resource_type=agent&resource_id=${AGENT_ID}" \
  -H "Authorization: <admin-pak>" \
  | jq '.logs[] | {actor_type, actor_id, action, timestamp, details}'
```

### Failed Authentication Attempts

```bash
curl -s "http://localhost:3000/api/v1/admin/audit-logs?action=auth.failed" \
  -H "Authorization: <admin-pak>" \
  | jq '.logs[] | {timestamp, ip_address, user_agent, details}'
```

### Recent PAK Rotations

```bash
curl -s "http://localhost:3000/api/v1/admin/audit-logs?action=pak.rotated" \
  -H "Authorization: <admin-pak>" \
  | jq '.logs[] | {actor_type, resource_type, resource_id, timestamp}'
```

### All Admin Actions Today

```bash
TODAY=$(date -u +%Y-%m-%dT00:00:00Z)
curl -s "http://localhost:3000/api/v1/admin/audit-logs?actor_type=admin&from=${TODAY}" \
  -H "Authorization: <admin-pak>" | jq .
```

### Webhook Configuration Changes

```bash
curl -s "http://localhost:3000/api/v1/admin/audit-logs?action=webhook.*" \
  -H "Authorization: <admin-pak>" \
  | jq '.logs[] | {action, resource_id, timestamp, details}'
```

### Stack Deletion History

```bash
curl -s "http://localhost:3000/api/v1/admin/audit-logs?action=stack.deleted" \
  -H "Authorization: <admin-pak>" \
  | jq '.logs[] | {actor_type, actor_id, resource_id, timestamp}'
```

## Audit Event Types

Actions follow the pattern `resource.verb` (e.g., `agent.created`, `pak.rotated`, `auth.failed`). You can use wildcard queries like `action=agent.*` to match all events for a resource type.

For the complete list of all audit event types, see the [Audit Logs Reference](../reference/audit-logs.md).

## Retention

Audit logs are retained for 90 days by default (`broker.audit_log_retention_days`). The broker runs a daily cleanup task to purge older entries.

To change retention:

```bash
BROKKR__BROKER__AUDIT_LOG_RETENTION_DAYS=365
```

## Related Documentation

- [Audit Logs Reference](../reference/audit-logs.md) — schema and data model details
- [Security Model](../explanation/security-model.md) — authentication and authorization
- [Managing PAKs](./pak-management.md) — PAK rotation and security
