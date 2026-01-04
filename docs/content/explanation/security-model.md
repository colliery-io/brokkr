---
title: "Security Model"
weight: 7
---

# Security Model

Security in Brokkr follows a defense-in-depth approach, implementing multiple layers of protection from network boundaries through application-level access controls. This document describes the trust boundaries, authentication mechanisms, authorization model, and operational security practices that protect Brokkr deployments.

## Trust Boundaries

Brokkr defines four distinct security zones, each with different trust levels and access controls. Understanding these boundaries is essential for secure deployment architecture and incident response.

{{< mermaid >}}
flowchart TB
    subgraph Untrusted["Untrusted Zone"]
        Internet[Internet/External]
        Admin[Admin Users]
        Generator[Generators/CI]
    end

    subgraph DMZ["DMZ / Edge"]
        Ingress[Ingress Controller]
        TLS[TLS Termination]
    end

    subgraph Trusted["Trusted Zone"]
        Broker[Broker Service]
        DB[(PostgreSQL)]
    end

    subgraph SemiTrusted["Semi-Trusted Zone (per cluster)"]
        Agent[Agent]
        K8s[Kubernetes API]
    end

    Internet --> Ingress
    Admin --> Ingress
    Generator --> Ingress
    Ingress --> TLS
    TLS --> Broker
    Broker <--> DB
    Agent --> Broker
    Agent --> K8s
{{< /mermaid >}}

The **Untrusted Zone** encompasses all external entities: internet traffic, administrator clients, and CI/CD generators. Nothing in this zone receives implicit trust—every request must authenticate before accessing protected resources.

The **DMZ** provides the transition layer where external traffic enters the system. The ingress controller terminates TLS connections, validating certificates and establishing encrypted channels. This layer handles the initial security negotiation before traffic reaches application components.

The **Trusted Zone** contains the broker service and its PostgreSQL database. Components in this zone communicate over internal networks with mutual trust. The database accepts connections only from the broker, and the broker applies authentication and authorization before exposing data to external zones.

The **Semi-Trusted Zone** exists in each target cluster where agents operate. Agents receive scoped trust: they can access resources targeted specifically to them but cannot see resources belonging to other agents. This isolation prevents a compromised agent from affecting deployments on other clusters.

### Security Principles

Four principles guide Brokkr's security architecture:

**Zero Trust by Default** requires all external requests to authenticate. The broker's API middleware rejects any request without valid credentials before route handlers execute. There are no anonymous endpoints except health checks.

**Least Privilege** restricts each identity to the minimum permissions necessary. Agents can only access deployment objects targeted to them through the agent_targets association. Generators can only manage stacks they created. This scoping limits the blast radius of credential compromise.

**Defense in Depth** implements multiple overlapping security controls. Even if an attacker bypasses network security, they face application-level authentication. Even with valid credentials, authorization limits accessible resources. Even with resource access, audit logging records all actions.

**Immutable Audit Trail** records every significant action in the system. Audit logs support only create and read operations—no updates or deletions are possible. This immutability ensures forensic evidence remains intact regardless of what an attacker does after gaining access.

## Authentication Mechanisms

Brokkr implements three authentication mechanisms, each designed for different actor types and usage patterns.

### Prefixed API Keys (PAKs)

PAKs serve as the primary authentication mechanism for agents and can also authenticate administrators and generators. The PAK design balances security with operational simplicity, enabling stateless authentication without storing plaintext secrets.

#### PAK Structure

Every PAK follows a structured format that embeds both an identifier and a secret component:

```
brokkr_BR{short_token}_{long_token}
       ^  ^            ^
       |  |            |
       |  |            +-- Long token (secret, used for verification)
       |  +--------------- Short token (identifier, safe to log)
       +------------------ Prefix (identifies key type)
```

The prefix `brokkr_BR` identifies this as a Brokkr PAK, distinguishing it from other credentials. The short token serves as an identifier that can appear in logs and error messages without compromising security. The long token is the secret component—it proves the holder possesses the original PAK.

A typical PAK looks like `brokkr_BRabc123_xyzSecretTokenHere...`. In this example, `abc123` is the short token (identifier) and `xyzSecretTokenHere...` is the long token (secret).

#### Generation Process

PAK generation occurs when creating agents, generators, or admin credentials. The process uses cryptographically secure randomness from the operating system's entropy source:

1. The system generates a random short token of configurable length. This token uses URL-safe characters and serves as the lookup key in the database.

2. A separate random long token is generated with sufficient entropy for cryptographic security. This token never leaves the generation response.

3. The long token is hashed using SHA-256, producing a fixed-size digest that represents the secret without revealing it.

4. The database stores only the hash. The original long token exists only in the complete PAK string returned to the caller.

5. The complete PAK is returned exactly once. If the caller loses it, a new PAK must be generated—the original cannot be recovered.

This design means the broker never stores information sufficient to reconstruct a PAK. A database breach reveals only hashes, which cannot authenticate without the original long tokens.

#### Verification Process

When a request arrives with a PAK, the authentication middleware executes a verification sequence:

{{< mermaid >}}
sequenceDiagram
    participant Client
    participant Middleware as Auth Middleware
    participant DB as PostgreSQL

    Client->>Middleware: Request with PAK header
    Middleware->>Middleware: Parse PAK structure
    Middleware->>Middleware: Extract short token
    Middleware->>DB: Lookup by pak_hash index
    DB-->>Middleware: Record with stored hash

    Middleware->>Middleware: Hash long token from request
    Middleware->>Middleware: Constant-time hash comparison

    alt Hashes match
        Middleware-->>Client: Authenticated (continue to handler)
    else Hashes don't match
        Middleware-->>Client: 401 Unauthorized
    end
{{< /mermaid >}}

The middleware first parses the PAK to extract its components. Using the short token as an identifier, it performs an indexed database lookup to find the associated record. This lookup uses the `pak_hash` column index, ensuring O(1) performance regardless of how many credentials exist.

The middleware then hashes the long token from the incoming request using the same SHA-256 algorithm used during generation. Finally, it compares this computed hash with the stored hash. The comparison uses constant-time algorithms to prevent timing attacks that could reveal information about valid hashes.

If verification succeeds, the middleware populates an `AuthPayload` structure identifying the authenticated entity (agent, generator, or admin) and attaches it to the request for downstream handlers. If verification fails, the request is rejected with a 401 status before reaching any route handler.

#### PAK Security Properties

| Property | Implementation |
|----------|----------------|
| **Secrecy** | Long token never stored; only SHA-256 hash persisted |
| **Non-repudiation** | PAK uniquely identifies the acting entity |
| **Revocation** | Entity can be disabled; PAK immediately invalid |
| **Rotation** | New PAK generated via rotate endpoint; old one invalidated |
| **Performance** | Indexed lookup prevents timing-based enumeration |

### Admin Authentication

Administrative users authenticate using PAKs stored in the `admin_role` table. Admin PAKs grant access to sensitive management operations that regular agents and generators cannot perform.

Admin authentication follows the same verification process as agent authentication, but the resulting `AuthPayload` sets the `admin` flag to true. Route handlers check this flag to authorize access to admin-only endpoints.

```bash
# Example admin API call
curl -X POST https://broker.example.com/api/v1/admin/config/reload \
  -H "Authorization: Bearer brokkr_BR..."
```

Admin credentials should be treated with extreme care. A compromised admin PAK grants access to all system data, configuration changes, and audit logs. Organizations should implement additional controls around admin credential storage and usage, such as hardware security modules or secrets management systems.

### Generator Authentication

Generators authenticate using PAKs stored in the `generators` table. Generator credentials enable CI/CD systems to create and manage deployments programmatically.

Generator permissions are scoped to resources they create. When a generator creates a stack, the broker records the generator's ID with that stack. Future operations on the stack verify the requesting generator matches the owner. This scoping prevents one generator from modifying another's deployments.

```bash
# Example generator API call
curl -X POST https://broker.example.com/api/v1/stacks \
  -H "Authorization: Bearer brokkr_BR..." \
  -H "Content-Type: application/json" \
  -d '{"name": "my-stack"}'
```

Generators cannot access admin endpoints regardless of their PAK. The authorization layer checks identity type before granting access to protected routes.

## Authorization Model

Brokkr implements implicit role-based access control (RBAC) where roles are determined by authentication type rather than explicit role assignments.

### Role Definitions

| Role | Authentication | Capabilities |
|------|----------------|--------------|
| **Agent** | PAK via agents table | Read targeted deployments, report events, claim work orders |
| **Generator** | PAK via generators table | Manage own stacks and deployment objects |
| **Admin** | PAK via admin_role table | Full system access including configuration and audit logs |
| **System** | Internal only | Background tasks, automated cleanup |

### Endpoint Authorization

The following table summarizes which roles can access each API endpoint category:

| Endpoint Pattern | Agent | Generator | Admin |
|------------------|-------|-----------|-------|
| `/api/v1/agents/{id}/target-state` | Own ID only | No | Yes |
| `/api/v1/agents/{id}/events` | Own ID only | No | Yes |
| `/api/v1/agents/{id}/work-orders/*` | Own ID only | No | Yes |
| `/api/v1/stacks/*` | No | Own stacks | Yes |
| `/api/v1/agents/*` (management) | No | No | Yes |
| `/api/v1/admin/*` | No | No | Yes |
| `/api/v1/webhooks/*` | No | No | Yes |
| `/healthz`, `/readyz` | Yes | Yes | Yes |
| `/metrics` | No | No | Yes |

### Resource-Level Access Control

Beyond endpoint-level authorization, Brokkr enforces resource-level access control through database queries.

**Agent Scope** limits agents to resources explicitly targeted to them. When an agent requests deployment objects, the query joins through the `agent_targets` table:

```sql
SELECT do.* FROM deployment_objects do
JOIN agent_targets at ON at.stack_id = do.stack_id
WHERE at.agent_id = :requesting_agent_id
  AND at.deleted_at IS NULL
  AND do.deleted_at IS NULL;
```

This query structure ensures agents can never see deployment objects from stacks not targeted to them, regardless of what parameters they provide in API requests.

**Generator Scope** restricts generators to stacks they created:

```sql
SELECT * FROM stacks
WHERE generator_id = :requesting_generator_id
  AND deleted_at IS NULL;
```

Generators cannot list, read, or modify stacks created by other generators or through admin operations.

## Credential Management

### Storage

Brokkr stores credentials using appropriate protection levels based on sensitivity:

| Credential Type | Storage Location | Protection |
|-----------------|------------------|------------|
| PAK hashes | PostgreSQL | SHA-256 hash (plaintext never stored) |
| Webhook URLs | PostgreSQL | AES-256-GCM encryption |
| Webhook auth headers | PostgreSQL | AES-256-GCM encryption |
| Database password | Kubernetes Secret | Base64 encoding (use sealed-secrets in production) |
| Webhook encryption key | Environment variable | Should use Kubernetes Secret |

### Webhook Secret Encryption

Webhook URLs and authentication headers may contain sensitive information like API keys or tokens. Brokkr encrypts these values at rest using AES-256-GCM, a modern authenticated encryption algorithm.

The encryption format includes version information for future algorithm upgrades:

```
version (1 byte) || nonce (12 bytes) || ciphertext || tag (16 bytes)
```

The current version byte (`0x01`) indicates AES-256-GCM encryption. The 12-byte nonce ensures each encryption produces unique ciphertext even for identical plaintexts. The 16-byte authentication tag detects any tampering with the encrypted data.

The encryption key is configured via the `BROKKR__BROKER__WEBHOOK_ENCRYPTION_KEY` environment variable as a 64-character hexadecimal string (representing 32 bytes). If no key is configured, the broker generates a random key at startup and logs a warning. Production deployments should always configure an explicit key to ensure encrypted data survives broker restarts.

### PAK Rotation

PAK rotation replaces an entity's authentication credential without disrupting its identity or permissions. The `POST /api/v1/agents/{id}/rotate-pak` endpoint (and similar endpoints for generators) generates a new PAK and invalidates the previous one atomically.

```bash
# Rotate an agent's PAK
curl -X POST https://broker/api/v1/agents/{id}/rotate-pak \
  -H "Authorization: Bearer <admin-pak>"
# Returns: new PAK (store immediately; cannot be retrieved again)

# Update agent configuration with new PAK
kubectl set env deployment/brokkr-agent BROKKR__BROKER__PAK=<new-pak>
```

After rotation, the old PAK becomes invalid immediately. Any requests using the old PAK receive 401 Unauthorized responses. The agent must be updated with the new PAK before its next broker communication.

### Credential Revocation

Revoking access involves soft-deleting the associated entity. When an agent is deleted, its record remains in the database with a `deleted_at` timestamp, but authentication queries filter out deleted records:

```bash
# Revoke agent access
curl -X DELETE https://broker/api/v1/agents/{id} \
  -H "Authorization: Bearer <admin-pak>"
```

After revocation, the agent's PAK becomes invalid immediately. Existing deployments remain in the target cluster (Brokkr doesn't forcibly remove resources), but the agent can no longer fetch new deployments or report events.

## Audit Logging

The audit logging system records significant actions for security monitoring, compliance, and forensic analysis. The system prioritizes completeness and immutability while minimizing performance impact on normal operations.

### Architecture

The audit logger uses an asynchronous design to avoid blocking API request handlers:

```
API Handler → mpsc channel (10,000 buffer) → Background Writer → PostgreSQL
```

When an action occurs, the handler sends an audit entry to a bounded channel. A background task collects entries from this channel and writes them to PostgreSQL in batches. This design ensures audit logging never blocks request processing, even under high load.

The background writer batches entries for efficiency, flushing when the batch reaches 100 entries or after 1 second, whichever comes first. This batching reduces database round trips while ensuring entries are persisted within a predictable time window.

### Audit Log Contents

Each audit log entry captures comprehensive context about the action:

| Field | Description |
|-------|-------------|
| `timestamp` | When the action occurred (UTC) |
| `actor_type` | Identity type: admin, agent, generator, or system |
| `actor_id` | UUID of the acting entity (if applicable) |
| `action` | What happened (e.g., `agent.created`, `pak.rotated`) |
| `resource_type` | Type of affected resource |
| `resource_id` | UUID of affected resource (if applicable) |
| `details` | Structured JSON with action-specific data |
| `ip_address` | Client IP address (when available) |
| `user_agent` | Client user agent string (when available) |

### Recorded Actions

The audit system records actions across several categories:

**Authentication Events** track credential usage and failures:
- `auth.success` - Successful authentication
- `auth.failed` - Failed authentication attempt
- `pak.created` - New PAK generated
- `pak.rotated` - PAK rotated
- `pak.deleted` - PAK revoked

**Resource Lifecycle** tracks creation, modification, and deletion:
- `agent.created`, `agent.updated`, `agent.deleted`
- `stack.created`, `stack.updated`, `stack.deleted`
- `generator.created`, `generator.updated`, `generator.deleted`
- `webhook.created`, `webhook.updated`, `webhook.deleted`

**Operational Events** record system activities:
- `workorder.created`, `workorder.claimed`, `workorder.completed`, `workorder.failed`
- `config.reloaded` - Configuration hot-reload triggered
- `webhook.delivery_failed` - Webhook delivery failure

### Query Capabilities

The audit log API supports filtering to find relevant entries:

```bash
# Recent authentication failures
curl "https://broker/api/v1/admin/audit-logs?action=auth.failed&limit=100" \
  -H "Authorization: Bearer <admin-pak>"

# Actions by specific agent
curl "https://broker/api/v1/admin/audit-logs?actor_type=agent&actor_id=<uuid>" \
  -H "Authorization: Bearer <admin-pak>"

# All admin actions in a time range
curl "https://broker/api/v1/admin/audit-logs?actor_type=admin&from=2024-01-01T00:00:00Z" \
  -H "Authorization: Bearer <admin-pak>"
```

Filters support actor type and ID, action (with wildcard prefix matching), resource type and ID, and time ranges. Pagination limits responses to manageable sizes, with a maximum of 1000 entries per request.

### Retention and Cleanup

Audit logs are retained for a configurable period (default: 90 days). A background task runs daily to remove entries older than the retention period. This cleanup prevents unbounded database growth while maintaining sufficient history for security investigations.

The retention period is configurable via broker settings, allowing organizations to meet their specific compliance requirements.

## Kubernetes Security

### Pod Security

Both broker and agent deployments configure security contexts that follow Kubernetes security best practices:

```yaml
podSecurityContext:
  runAsNonRoot: true
  runAsUser: 10001
  runAsGroup: 10001
  fsGroup: 10001

containerSecurityContext:
  allowPrivilegeEscalation: false
  readOnlyRootFilesystem: false  # Configurable
  capabilities:
    drop:
      - ALL
```

**Non-root execution** prevents containers from running as the root user, limiting the impact of container escape vulnerabilities. The UID 10001 is arbitrary but consistent across components.

**Privilege escalation prevention** blocks processes from gaining additional capabilities after container startup, closing a common attack vector.

**Capability dropping** removes all Linux capabilities by default. Most applications don't need capabilities, and removing them reduces attack surface.

**AppArmor support** is available when enabled via `apparmor.enabled: true`. AppArmor provides mandatory access control at the kernel level, further restricting what container processes can do.

### Agent RBAC

The agent requires Kubernetes permissions to manage resources in target clusters. The Helm chart creates RBAC resources that implement least-privilege access:

**Read-only access** to core resources (pods, services, configmaps, deployments) enables the agent to monitor cluster state without modification capabilities beyond what's necessary for deployment management.

**Shipwright access** (when enabled) grants create, update, and delete permissions on Build and BuildRun resources for container image builds.

**Optional secret access** is disabled by default. When enabled via `rbac.secretAccess.enabled`, the agent can list and watch secrets. The `rbac.secretAccess.readContents` flag controls whether the agent can read actual secret values.

The RBAC configuration supports both namespace-scoped (Role/RoleBinding) and cluster-wide (ClusterRole/ClusterRoleBinding) permissions, configured via `rbac.clusterWide`.

## Security Best Practices

### Production Deployment Checklist

Before deploying Brokkr to production, verify these security configurations:

- [ ] **TLS Everywhere**: Enable TLS for all external connections via ingress or direct TLS
- [ ] **Strong Secrets**: Use cryptographically secure random values for all PAKs and encryption keys
- [ ] **External Database**: Use managed PostgreSQL with encryption at rest
- [ ] **Secret Management**: Store credentials in Kubernetes Secrets or external vault
- [ ] **NetworkPolicy**: Enable and configure network policies to restrict traffic
- [ ] **RBAC**: Use minimal required permissions for service accounts
- [ ] **Pod Security**: Enable pod security standards (restricted profile)
- [ ] **Audit Logging**: Enable and monitor audit logs
- [ ] **Resource Limits**: Set CPU/memory limits to prevent resource exhaustion
- [ ] **Image Scanning**: Scan container images for vulnerabilities before deployment

### Monitoring for Security Events

Monitor these metrics and events for security-relevant activity:

| Indicator | Alert Threshold | Potential Issue |
|-----------|-----------------|-----------------|
| Failed authentication rate | > 10/minute | Brute force attack |
| Unexpected agent disconnections | Any | Possible compromise or network attack |
| Webhook delivery failure rate | > 50% | Network issues or endpoint compromise |
| Audit log volume spike | 10x normal | Unusual activity, possible attack |
| Admin action from unknown IP | Any | Credential theft |

### Incident Response

#### Suspected Agent Compromise

If you suspect an agent's credentials have been compromised:

1. **Revoke immediately**: Delete or disable the agent via the admin API
2. **Review audit logs**: Search for unusual actions by the agent's actor_id
3. **Inspect cluster**: Review resources the agent may have created or modified
4. **Rotate secrets**: Generate new PAK if re-enabling the agent
5. **Investigate**: Determine how the compromise occurred

#### Suspected Broker Compromise

If you suspect the broker itself has been compromised:

1. **Isolate**: Remove external network access to the broker
2. **Preserve evidence**: Capture logs, database state, and container images
3. **Rotate all credentials**: Generate new PAKs for all agents, generators, and admins
4. **Review webhooks**: Check for unauthorized webhook subscriptions
5. **Audit database**: Look for unauthorized modifications to stacks or agents
6. **Rebuild**: Consider deploying fresh broker instances rather than cleaning compromised ones

## Compliance Considerations

### Data Protection

Brokkr implements several controls relevant to data protection regulations:

**Access logging** records all API access with actor identification, supporting accountability requirements.

**Encryption at rest** protects sensitive webhook data using AES-256-GCM.

**Encryption in transit** via TLS protects all external communications.

**Data minimization** ensures PAK secrets are never stored, only their hashes.

**Retention controls** automatically remove old audit logs after the configured period.

### Regulatory Mapping

| Requirement | Brokkr Feature |
|-------------|----------------|
| Access control | PAK authentication + implicit RBAC |
| Audit trail | Immutable audit logs with comprehensive action recording |
| Data encryption | TLS in transit, AES-256-GCM for secrets at rest |
| Least privilege | Scoped agent and generator access |
| Monitoring | Metrics endpoint, audit log queries |
| Incident response | Credential revocation, audit log forensics |
