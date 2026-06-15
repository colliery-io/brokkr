# Audit Logs

Brokkr maintains an immutable audit trail of administrative and security-sensitive operations. Every PAK creation, resource modification, authentication attempt, and significant system event is recorded with details about who performed the action, what was affected, and when it occurred.

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

Actions follow a `resource.verb` naming convention.

### Currently Emitted

These actions are recorded by the broker today (emit sites in `crates/brokkr-broker/src/api/v1/{agents,stacks,webhooks,middleware,admin}.rs`):

| Action | Description |
|--------|-------------|
| `agent.created` | New agent registered |
| `agent.updated` | Agent details modified |
| `agent.deleted` | Agent removed |
| `pak.rotated` | An agent or generator PAK was rotated (REST endpoint or CLI; CLI entries carry `details.via = "cli"`) |
| `stack.created` | New stack created |
| `stack.updated` | Stack details modified |
| `stack.deleted` | Stack removed |
| `webhook.created` | New webhook subscription created |
| `webhook.updated` | Webhook subscription modified |
| `webhook.deleted` | Webhook subscription removed |
| `generator.created` | New generator created (paired with a `pak.created` entry) |
| `generator.updated` | Generator details modified |
| `generator.deleted` | Generator removed |
| `template.created` | New template created |
| `template.updated` | Template updated (new version row) |
| `template.deleted` | Template removed |
| `workorder.created` | New work order created |
| `workorder.claimed` | Work order claimed by an agent |
| `workorder.completed` | Work order completed successfully |
| `workorder.failed` | Work order failed terminally (max retries or non-retryable) |
| `workorder.retry` | Work order failed and was scheduled for retry |
| `webhook.delivery_failed` | A webhook delivery exhausted retries and went `dead` |
| `pak.created` | A PAK was issued at agent/generator creation (REST or CLI) |
| `auth.failed` | A request presented an invalid PAK (recorded with source IP and path) |
| `config.reloaded` | Configuration hot-reload performed (recorded with the change set) |

### Defined but Not Yet Emitted

Two action constants exist in the data model (`crates/brokkr-models/src/models/audit_logs.rs`) but are intentionally not recorded: `auth.success` (one row per uncached authenticated request would dwarf the rest of the log; successful access is observable via the resource-level events) and `pak.deleted` (PAKs have no standalone deletion — they are replaced on rotation or die with their entity, both of which are audited).

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

**All webhook-related actions (using prefix matching):**

```bash
curl "http://localhost:3000/api/v1/admin/audit-logs?action=webhook.*" \
  -H "Authorization: Bearer $ADMIN_PAK"
```

For more worked query patterns, see [Working with Audit Logs](../how-to/audit-logs.md).

## Details Field

The `details` field contains structured JSON with context specific to each action type. Common patterns include:

**Resource creation:**
```json
{
  "name": "my-stack",
  "generator_id": "abc123-..."
}
```

**Agent update** (`agent.updated`, from `crates/brokkr-broker/src/api/v1/agents.rs`):
```json
{
  "name": "my-agent",
  "cluster_name": "production",
  "status": "ACTIVE"
}
```

**PAK rotation** (`pak.rotated`):
```json
{
  "agent_name": "my-agent"
}
```

## Retention Policy

Audit logs are subject to a retention policy that automatically removes old entries:

- **Retention period**: Configurable via `broker.audit_log_retention_days` (default 90 days)
- **Cleanup frequency**: Background task runs daily (86400-second interval)
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

## Related Documentation

- [Working with Audit Logs](../how-to/audit-logs.md) - Security monitoring, compliance reporting, and change-tracking queries
- [Security Model](../explanation/security-model.md) - Authentication and authorization
- [Soft Deletion](./soft-deletion.md) - Resource deletion patterns
- [Webhooks](../how-to/webhooks.md) - Event notification system
