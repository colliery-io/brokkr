# How-To: Managing PAKs (Key Rotation)

Pre-Authentication Keys (PAKs) are the authentication credentials for all Brokkr entities — admins, agents, and generators. This guide covers creating, rotating, and managing PAKs.

## Overview

PAKs look like `brokkr_BR3rVsDa_GK3QN7CDUzYc6iKgMkJ98M2WSimM5t6U8`. Brokkr stores only the **hash** — once a PAK is displayed at creation, it cannot be recovered, only rotated. See the [Environment Variables Reference](../reference/environment-variables.md#pak-pre-authentication-key-generation) for PAK configuration details.

## Rotating the Admin PAK

### Via CLI (recommended)

Run on the broker host:

```bash
brokkr-broker rotate admin
```

The new PAK is printed to stdout. The old PAK immediately stops working.

### Via API

Not available — admin PAK rotation requires CLI access to prevent an attacker with a compromised admin PAK from locking out the real admin.

### When to Rotate

- After personnel changes (someone with admin access leaves)
- If the PAK may have been exposed in logs, version control, or screenshots
- As part of a regular rotation schedule (e.g., quarterly)

## Rotating Agent PAKs

### Via CLI

```bash
brokkr-broker rotate agent --uuid <agent-uuid>
```

### Via API

An agent can rotate its own PAK, or an admin can rotate any agent's PAK:

```bash
# As admin
curl -s -X POST "http://localhost:3000/api/v1/agents/${AGENT_ID}/rotate-pak" \
  -H "Authorization: <admin-pak>" | jq .

# As the agent itself
curl -s -X POST "http://localhost:3000/api/v1/agents/${AGENT_ID}/rotate-pak" \
  -H "Authorization: <agent-pak>" | jq .
```

**Response:**

```json
{
  "agent": { "id": "...", "name": "prod-1", ... },
  "pak": "brokkr_BRnewKey_NewLongTokenValue1234567890"
}
```

### Updating the Agent After Rotation

After rotating, update the agent's configuration with the new PAK:

**Helm deployment:**
```bash
helm upgrade brokkr-agent oci://ghcr.io/colliery-io/charts/brokkr-agent \
  --set broker.pak="brokkr_BRnewKey_NewLongTokenValue1234567890"
```

**Environment variable:**
```bash
BROKKR__AGENT__PAK=brokkr_BRnewKey_NewLongTokenValue1234567890
```

**Kubernetes secret:**
```bash
kubectl create secret generic brokkr-agent-pak \
  --from-literal=pak="brokkr_BRnewKey_NewLongTokenValue1234567890" \
  --dry-run=client -o yaml | kubectl apply -f -
```

> **Warning:** The agent will fail to authenticate with the old PAK immediately after rotation. Ensure you update the agent configuration before the next poll cycle, or the agent will lose connectivity until updated.

## Rotating Generator PAKs

### Via CLI

```bash
brokkr-broker rotate generator --uuid <generator-uuid>
```

### Via API

```bash
# As admin
curl -s -X POST "http://localhost:3000/api/v1/generators/${GEN_ID}/rotate-pak" \
  -H "Authorization: <admin-pak>" | jq .

# As the generator itself
curl -s -X POST "http://localhost:3000/api/v1/generators/${GEN_ID}/rotate-pak" \
  -H "Authorization: <generator-pak>" | jq .
```

### Updating CI/CD After Rotation

Update the stored secret in your CI/CD system:

**GitHub Actions:**
1. Go to Settings → Secrets and variables → Actions
2. Update the `BROKKR_GENERATOR_PAK` secret with the new value

**GitLab CI:**
1. Go to Settings → CI/CD → Variables
2. Update the `BROKKR_GENERATOR_PAK` variable

## Cache Considerations After CLI Rotation

API-based rotation automatically invalidates the auth cache. CLI-based rotation operates directly on the database, so the old PAK may still work for up to 60 seconds (the default `broker.auth_cache_ttl_seconds`). To force immediate invalidation after a CLI rotation:

```bash
curl -s -X POST "http://localhost:3000/api/v1/admin/config/reload" \
  -H "Authorization: <admin-pak>"
```

## Verifying Rotation via Audit Logs

All PAK operations are recorded. Query PAK-related audit events:

```bash
curl -s "http://localhost:3000/api/v1/admin/audit-logs?action=pak.*" \
  -H "Authorization: <admin-pak>" | jq .
```

See the [Audit Logs Reference](../reference/audit-logs.md) for the full list of audit event types.

## Related Documentation

- [Security Model](../explanation/security-model.md) — authentication and authorization architecture
- [Audit Logs Reference](../reference/audit-logs.md) — audit event format and querying
- [Configuration Guide](../getting-started/configuration.md) — PAK and auth cache settings
