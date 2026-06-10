# How-To: Managing PAKs (Key Rotation)

Prefixed API Keys (PAKs) are the authentication credentials for all Brokkr entities — admins, agents, and generators. This guide covers creating, rotating, and managing PAKs.

## Overview

PAKs look like `brokkr_BR3rVsDa_GK3QN7CDUzYc6iKgMkJ98M2WSimM5t6U8`. Brokkr stores only the **hash** — once a PAK is displayed at creation, it cannot be recovered, only rotated. See the [Environment Variables Reference](../reference/environment-variables.md#pak-pre-authentication-key-generation) for PAK configuration details.

## Rotating the Admin PAK

### Via CLI

The `rotate admin` command re-runs the same admin upsert that runs at startup, and its behavior depends on `broker.pak_hash`:

- **If `broker.pak_hash` is set** (it is by default, to a publicly-known development hash), the command simply re-applies that hash — **nothing rotates**.
- **If `broker.pak_hash` is unset or empty**, a fresh PAK is generated and written to `/tmp/brokkr-keys/key.txt` on the broker host. It is never printed to stdout.

To actually rotate:

```bash
# 1. Clear the configured hash so a fresh PAK is generated
export BROKKR__BROKER__PAK_HASH=""

# 2. Rotate (or restart the broker — startup runs the same upsert)
brokkr-broker rotate admin

# 3. Read the new PAK from the key file
cat /tmp/brokkr-keys/key.txt
```

The old admin PAK stops working once the new hash is in the database (subject to the auth cache — see below). If you manage `broker.pak_hash` explicitly, instead generate a new PAK/hash pair yourself and set the new hash in every place the broker config defines it.

### Via API

Not available — admin PAK rotation requires CLI access to prevent an attacker with a compromised admin PAK from locking out the real admin.

### When to Rotate

- After personnel changes (someone with admin access leaves)
- If the PAK may have been exposed in logs, version control, or screenshots
- As part of a regular rotation schedule (e.g., quarterly)

## Rotating Agent PAKs

### Via CLI

The CLI prints the new PAK to stdout (`New PAK: ...`); it is shown once, like the REST endpoint's response.

```bash
# Revokes the agent's current PAK; the replacement is unrecoverable
brokkr-broker rotate agent --uuid <agent-uuid>
```

### Via API (recommended)

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

As with agents, the CLI prints the new generator PAK to stdout once.

```bash
# Revokes the generator's current PAK; the replacement is unrecoverable
brokkr-broker rotate generator --uuid <generator-uuid>
```

### Via API (recommended)

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

API-based rotation evicts the old PAK from the broker's auth cache immediately, so it stops working right away. CLI-based rotation (including `rotate admin`) operates directly on the database without touching the running broker's cache, so the old PAK may still authenticate for up to 60 seconds (the default `broker.auth_cache_ttl_seconds`). There is no endpoint to flush the cache; if a 60-second window is unacceptable, restart the broker after a CLI rotation.

## Verifying Rotation via Audit Logs

All PAK rotations — agent and generator, via the REST endpoints or the CLI — are recorded as `pak.rotated` audit events (CLI entries carry `details.via = "cli"`).

```bash
curl -s "http://localhost:3000/api/v1/admin/audit-logs?action=pak.*" \
  -H "Authorization: <admin-pak>" | jq .
```

See the [Audit Logs Reference](../reference/audit-logs.md) for the full list of audit event types.

## Related Documentation

- [Security Model](../explanation/security-model.md) — authentication and authorization architecture
- [Audit Logs Reference](../reference/audit-logs.md) — audit event format and querying
- [Configuration Guide](../getting-started/configuration.md) — PAK and auth cache settings
