---
title: "Audit Logs"
weight: 11
---

# Audit Logs

Brokkr maintains an immutable audit trail of administrative and security-sensitive operations. Every PAK creation, resource modification, authentication attempt, and significant system event is recorded with details about who performed the action, what was affected, and when it occurred. This documentation covers the audit log schema, available actions, query patterns, and retention policies.

## Schema

Each audit log entry captures comprehensive information about an event:

| Field | Type | Description |
|-------|------|-------------|
| `id` | UUID | Unique identifier for the log entry |
| `timestamp` | timestamp | When the event occurred |
| `actor_type` | string | Type of actor: `admin`, `agent`, `generator`, `system` |
| `actor_id` | UUID | ID of the actor (null for system or unauthenticated) |
| `action` | string | The action performed (e.g., `agent.created`) |
| `resource_type` | string | Type of resource affected (e.g., `agent`, `stack`) |
| `resource_id` | UUID | ID of the affected resource (null if not applicable) |
| `details` | JSON | Structured details about the action |
| `ip_address` | string | Client IP address |
| `user_agent` | string | Client user agent string |
| `created_at` | timestamp | When the record was created |

## Actor Types

The `actor_type` field identifies what kind of entity performed the action:

| Type | Description |
|------|-------------|
| `admin` | Administrator using an admin PAK |
| `agent` | An agent performing its own operations |
| `generator` | A generator creating or managing resources |
| `system` | System-initiated operations (background tasks, scheduled jobs) |

## Actions

Actions follow a `resource.verb` naming convention. The following actions are currently logged:

### Authentication

| Action | Description |
|--------|-------------|
| `pak.created` | A new PAK was generated |
| `pak.rotated` | An existing PAK was rotated |
| `pak.deleted` | A PAK was invalidated |
| `auth.failed` | Authentication attempt failed |
| `auth.success` | Authentication succeeded |

### Resource Management

| Action | Description |
|--------|-------------|
| `agent.created` | New agent registered |
| `agent.updated` | Agent details modified |
| `agent.deleted` | Agent removed |
| `stack.created` | New stack created |
| `stack.updated` | Stack details modified |
| `stack.deleted` | Stack removed |
| `generator.created` | New generator created |
| `generator.updated` | Generator details modified |
| `generator.deleted` | Generator removed |
| `template.created` | New template created |
| `template.updated` | Template modified |
| `template.deleted` | Template removed |

### Webhooks

| Action | Description |
|--------|-------------|
| `webhook.created` | New webhook subscription created |
| `webhook.updated` | Webhook subscription modified |
| `webhook.deleted` | Webhook subscription removed |
| `webhook.delivery_failed` | Webhook delivery failed after retries |

### Work Orders

| Action | Description |
|--------|-------------|
| `workorder.created` | New work order created |
| `workorder.claimed` | Work order claimed by an agent |
| `workorder.completed` | Work order completed successfully |
| `workorder.failed` | Work order failed |
| `workorder.retry` | Work order returned for retry |

### Administration

| Action | Description |
|--------|-------------|
| `config.reloaded` | Configuration hot-reload performed |

## Resource Types

The `resource_type` field identifies what kind of resource was affected:

| Type | Description |
|------|-------------|
| `agent` | An agent resource |
| `stack` | A stack resource |
| `generator` | A generator resource |
| `template` | A stack template |
| `webhook_subscription` | A webhook subscription |
| `work_order` | A work order |
| `pak` | A PAK (authentication key) |
| `config` | System configuration |
| `system` | System-level resource |

## Querying Audit Logs

### API Endpoint

Query audit logs through the admin API:

```
GET /api/v1/admin/audit-logs
Authorization: Bearer <admin_pak>
```

### Query Parameters

| Parameter | Type | Description |
|-----------|------|-------------|
| `actor_type` | string | Filter by actor type |
| `actor_id` | UUID | Filter by actor ID |
| `action` | string | Filter by action (exact match or prefix with `*`) |
| `resource_type` | string | Filter by resource type |
| `resource_id` | UUID | Filter by resource ID |
| `from` | timestamp | Start time (inclusive, ISO 8601) |
| `to` | timestamp | End time (exclusive, ISO 8601) |
| `limit` | integer | Maximum results (default 100, max 1000) |
| `offset` | integer | Results to skip (for pagination) |

### Response Format

```json
{
  "logs": [
    {
      "id": "a1b2c3d4-...",
      "timestamp": "2025-01-02T10:00:00Z",
      "actor_type": "admin",
      "actor_id": null,
      "action": "agent.created",
      "resource_type": "agent",
      "resource_id": "e5f6g7h8-...",
      "details": {
        "agent_name": "production-cluster",
        "cluster_name": "prod-us-east"
      },
      "ip_address": "192.168.1.100",
      "user_agent": "curl/8.0.0",
      "created_at": "2025-01-02T10:00:00Z"
    }
  ],
  "total": 150,
  "count": 100,
  "limit": 100,
  "offset": 0
}
```

### Example Queries

**All agent creation events:**

```bash
curl "http://localhost:3000/api/v1/admin/audit-logs?action=agent.created" \
  -H "Authorization: Bearer $ADMIN_PAK"
```

**All actions by a specific generator:**

```bash
curl "http://localhost:3000/api/v1/admin/audit-logs?actor_type=generator&actor_id=$GENERATOR_ID" \
  -H "Authorization: Bearer $ADMIN_PAK"
```

**Failed authentication attempts in the last hour:**

```bash
curl "http://localhost:3000/api/v1/admin/audit-logs?action=auth.failed&from=$(date -u -d '1 hour ago' +%Y-%m-%dT%H:%M:%SZ)" \
  -H "Authorization: Bearer $ADMIN_PAK"
```

**All webhook-related actions (using prefix matching):**

```bash
curl "http://localhost:3000/api/v1/admin/audit-logs?action=webhook.*" \
  -H "Authorization: Bearer $ADMIN_PAK"
```

**History of a specific resource:**

```bash
curl "http://localhost:3000/api/v1/admin/audit-logs?resource_type=stack&resource_id=$STACK_ID" \
  -H "Authorization: Bearer $ADMIN_PAK"
```

## Details Field

The `details` field contains structured JSON with context specific to each action type. Common patterns include:

**Resource creation:**
```json
{
  "name": "my-stack",
  "generator_id": "abc123-..."
}
```

**Authentication failure:**
```json
{
  "reason": "invalid_pak",
  "pak_prefix": "brk_gen_"
}
```

**Configuration changes:**
```json
{
  "key": "webhook.timeout",
  "old_value": "30",
  "new_value": "60"
}
```

## Retention Policy

Audit logs are subject to a retention policy that automatically removes old entries:

- **Retention period**: Configurable (default varies by deployment)
- **Cleanup frequency**: Background task runs periodically
- **Deletion method**: Hard delete (permanent removal)

Configure retention through broker settings:

```yaml
broker:
  audit_log_retention_days: 90
```

The cleanup task uses the `created_at` index for efficient deletion of old records.

## Immutability

Audit log records are immutable after creation. The database schema enforces this by:

- No `updated_at` column exists
- No update operations are exposed through the API or DAL
- Records can only be deleted by the retention policy

This immutability is essential for compliance requirements—audit logs must accurately reflect what happened without possibility of after-the-fact modification.

## Database Indexes

For query performance, the following indexes exist:

| Index | Columns | Purpose |
|-------|---------|---------|
| `idx_audit_logs_timestamp` | `(timestamp DESC)` | Time-based queries |
| `idx_audit_logs_actor` | `(actor_type, actor_id, timestamp DESC)` | Actor queries |
| `idx_audit_logs_resource` | `(resource_type, resource_id, timestamp DESC)` | Resource history |
| `idx_audit_logs_action` | `(action, timestamp DESC)` | Action filtering |
| `idx_audit_logs_cleanup` | `(created_at)` | Retention cleanup |

## Security Considerations

**Access control**: Only admin PAKs can query audit logs. Agents and generators cannot access the audit log API.

**Sensitive data**: The `details` field may contain resource names and identifiers but should not contain secrets. PAK values are never logged—only the action of creation or rotation is recorded.

**IP address logging**: Client IP addresses are captured for security investigation. Consider privacy implications for your deployment.

**Failed auth tracking**: Failed authentication attempts are logged with IP addresses, enabling detection of brute force attacks or credential stuffing.

## Integration Patterns

### Security Monitoring

Query failed authentication attempts to detect attack patterns:

```bash
# Get failed auth count by IP in last 24 hours
curl "http://localhost:3000/api/v1/admin/audit-logs?action=auth.failed&from=$(date -u -d '24 hours ago' +%Y-%m-%dT%H:%M:%SZ)&limit=1000" \
  -H "Authorization: Bearer $ADMIN_PAK" | \
  jq '[.logs[].ip_address] | group_by(.) | map({ip: .[0], count: length}) | sort_by(-.count)'
```

### Compliance Reporting

Export audit logs for compliance audits:

```bash
# Export all actions for a time period
curl "http://localhost:3000/api/v1/admin/audit-logs?from=2025-01-01T00:00:00Z&to=2025-02-01T00:00:00Z&limit=1000" \
  -H "Authorization: Bearer $ADMIN_PAK" > audit-january-2025.json
```

### Change Tracking

Track changes to a specific resource over time:

```bash
# See complete history of an agent
curl "http://localhost:3000/api/v1/admin/audit-logs?resource_type=agent&resource_id=$AGENT_ID&limit=50" \
  -H "Authorization: Bearer $ADMIN_PAK"
```

## Related Documentation

- [Security Model](/explanation/security-model) - Authentication and authorization
- [Soft Deletion](/reference/soft-deletion) - Resource deletion patterns
- [Webhooks](/how-to/webhooks) - Event notification system
