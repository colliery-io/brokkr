# Audit Logs

Brokkr records administrative and security-sensitive operations to an append-only audit trail. PAK creations, resource modifications, failed authentication attempts, and significant system events are captured with details about who performed the action, what was affected, and when it occurred.

Audit writes are asynchronous and best-effort — see [Delivery Guarantees](#delivery-guarantees) before treating the trail as a system of record.

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
| `ip_address` | string | Client IP address. Omitted from the JSON entirely when not recorded |
| `user_agent` | string | Client user agent string. Omitted from the JSON entirely when not recorded |
| `created_at` | timestamp | When the record was created |
| `actor_name` | string \| null | Human-readable actor name, added by the query API (see below) |

`actor_name` is not a stored column: the query API resolves it when the entry is read. For `admin` and `system` actors it is the literal `"admin"` or `"system"`; for agents and generators it is that entity's current name, so a rename is reflected across the whole history. It is `null` when the actor ID no longer resolves to a live entity.

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

These actions are recorded by the broker today:

| Action | Description |
|--------|-------------|
| `agent.created` | New agent created |
| `agent.updated` | Agent details modified |
| `agent.deleted` | Agent removed |
| `agent.registered` | An agent was registered with a generator |
| `agent.deregistered` | An agent's registration with a generator was removed |
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

Two action constants exist in the data model but are intentionally not recorded: `auth.success` (one row per uncached authenticated request would dwarf the rest of the log; successful access is observable via the resource-level events) and `pak.deleted` (PAKs have no standalone deletion — they are replaced on rotation or die with their entity, both of which are audited).

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
      "actor_name": "admin",
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

**Agent update** (`agent.updated`):
```json
{
  "name": "my-agent",
  "cluster_name": "production",
  "status": "ACTIVE"
}
```

**Agent registration** (`agent.registered`, `agent.deregistered`) — the resource is the agent; the generator it was registered with is in the details:
```json
{
  "generator_id": "abc123-..."
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

## Delivery Guarantees

Audit logging is asynchronous and best-effort. It is deliberately kept off the request hot path: a handler hands the entry to an in-memory queue (bounded at 10,000 entries) and returns immediately, and a background writer drains the queue into the database in batches of up to 100, at least once per second.

The consequences of that design are:

- **Entries are not written transactionally with the operation they describe.** An operation can succeed and its entry can still be missing. Ordering within a batch is preserved, but an entry may land in the database slightly after the response is sent.
- **A failed batch insert is discarded, not retried.** If the database is unavailable when the writer flushes, the entries in that batch are logged to the broker's error log (`Lost audit entry: ...`) and dropped. They are never re-attempted.
- **Queued entries do not survive an abrupt stop.** Anything still in memory when the process is killed is lost.
- **Sustained write pressure slows producers rather than losing entries.** When the queue is full, the enqueue waits for space instead of discarding, so a backlog shows up as delayed entries.

Treat the trail as a high-fidelity operational and security record, not as a guaranteed-complete ledger. If your compliance posture requires guaranteed capture, export the trail on a schedule (see [Working with Audit Logs](../how-to/audit-logs.md)) and alert on `Lost audit entry` in the broker logs.

## Immutability

Audit log records are immutable after creation. The database schema enforces this by:

- No `updated_at` column exists
- No update operations are exposed through the API or DAL
- Records can only be deleted by the retention policy

Nothing that reaches the table can be altered after the fact. Note that immutability is a separate property from completeness — see [Delivery Guarantees](#delivery-guarantees).

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

**Access control**: The audit log API is gated on admin rights. Two credentials qualify: an admin PAK, and the broker's zero-config read-only console token. Agent and generator PAKs are rejected.

**Console exposure**: The read-only console token is handed to any browser that can load the Operator Console from the broker's HTTP port. The console itself has no audit-log screen, but the token is not limited to the screens the console draws — it is a read-only admin credential, so whoever holds it can query this API directly and read the whole trail, IP addresses and user agents included. Network reachability of the broker port is therefore the boundary protecting the audit trail. Restrict access to it accordingly.

**Sensitive data**: The `details` field may contain resource names and identifiers but should not contain secrets. PAK values are never logged—only the action of creation or rotation is recorded.

**IP address logging**: Client IP addresses are captured for security investigation. Consider privacy implications for your deployment.

**Failed auth tracking**: Failed authentication attempts are logged with IP addresses, enabling detection of brute force attacks or credential stuffing.

## Related Documentation

- [Working with Audit Logs](../how-to/audit-logs.md) - Security monitoring, compliance reporting, and change-tracking queries
- [Security Model](../explanation/security-model.md) - Authentication and authorization
- [Soft Deletion](./soft-deletion.md) - Resource deletion patterns
- [Webhooks](../how-to/webhooks.md) - Event notification system
