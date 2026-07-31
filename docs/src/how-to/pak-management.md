# How-To: Managing PAKs (Key Rotation)

Prefixed API Keys (PAKs) are the authentication credentials for all Brokkr entities — admins, agents, and generators. This guide covers creating, rotating, and managing PAKs.

## Overview

PAKs look like `brokkr_BR3rVsDa_GK3QN7CDUzYc6iKgMkJ98M2WSimM5t6U8`. Brokkr stores only the **hash** — once a PAK is displayed at creation, it cannot be recovered, only rotated. See the [Environment Variables Reference](../reference/environment-variables.md#pak-pre-authentication-key-generation) for PAK configuration details.

## Day-Zero Bootstrap

Before the broker has ever started, there is no admin PAK and no database to read one from. Use `brokkr-broker generate-pak` to mint the first admin PAK **offline** — it derives a PAK and its SHA-256 hash from the embedded defaults without touching a database, a running broker, or a key file. You supply the hash to the broker through configuration so it can authenticate the very first request. This is distinct from [Rotating the Admin PAK](#rotating-the-admin-pak) below, which changes a PAK that already exists.

1. Mint the PAK/hash pair offline (no database or running broker required):

   ```bash
   brokkr-broker generate-pak
   ```

   The command prints both the secret PAK and its SHA-256 hash:

   ```text
   PAK:  brokkr_BR3rVsDa_GK3QN7CDUzYc6iKgMkJ98M2WSimM5t6U8
   Hash: 9f2c4e1b8a7d6c5f4e3b2a1908d7c6b5a4f3e2d1c0b9a8f7e6d5c4b3a2f1e0d9
   ```

2. Store the PAK securely — a Kubernetes secret, CI/CD vault, or password manager. It is shown only once and cannot be recovered.

3. Set the hash in the broker configuration **before** first startup:

   ```bash
   export BROKKR__BROKER__PAK_HASH="9f2c4e1b8a7d6c5f4e3b2a1908d7c6b5a4f3e2d1c0b9a8f7e6d5c4b3a2f1e0d9"
   ```

4. Start the broker:

   ```bash
   brokkr-broker serve
   ```

5. Verify admin access with the PAK you stored:

   ```bash
   curl -s "http://localhost:3000/api/v1/admin/audit-logs" \
     -H "Authorization: brokkr_BR3rVsDa_GK3QN7CDUzYc6iKgMkJ98M2WSimM5t6U8" | jq .
   ```

> **Note:** On first startup the broker stores this hash on the admin role and admin generator — no key file is written to `/tmp/brokkr-keys/`. `generate-pak` takes no arguments; see the [CLI Reference](../reference/cli.md) for the exact output format.

Presetting the hash this way is also the safest bootstrap: you hold the admin PAK before the broker ever runs, so there is no window in which the credential exists only inside a container. If you instead let the broker mint its own admin PAK (see below), you must capture it from the key file on the very first startup or lose it.

## Rotating the Admin PAK

### Via CLI

`brokkr-broker rotate admin` is the only way to change the admin PAK. Restarting the broker does **not** rotate it: the admin bootstrap runs only on a database that has never been initialized, so on an existing database a restart re-reads the stored hash and changes nothing.

What `rotate admin` does depends on `broker.pak_hash`:

- **If `broker.pak_hash` is set** (it is by default, to a publicly-known development hash), the command validates and re-applies that hash — **nothing rotates**, and it says so rather than reporting success. This is the path to use when you manage the hash yourself: mint a new pair with `brokkr-broker generate-pak`, update the hash everywhere the broker config defines it (`BROKKR__BROKER__PAK_HASH`, or the chart's `broker.pakHash` / `broker.pakHashExistingSecret`), then run `rotate admin` to store it.
- **If `broker.pak_hash` is unset or empty**, the command mints a fresh PAK and **prints both the PAK and its hash**. The PAK is shown once and cannot be recovered from the hash. It is also written to `/tmp/brokkr-keys/key.txt` as a fallback, which the broker deletes on graceful shutdown.

> **Take both values.** The PAK is the secret — store it in a vault. The hash is not secret and belongs in configuration. If you keep only the PAK you cannot set the configured hash to match: it is a SHA-256 of the PAK's long-token component, not of the whole string, so `sha256sum` will not reproduce it. Configuration and database would then disagree, and a later `rotate admin` would overwrite your new credential with whatever the config still holds.

Rotating with a hash you minted yourself:

```bash
# 1. Mint the replacement pair offline
brokkr-broker generate-pak

# 2. Set the new hash in the broker's configuration
export BROKKR__BROKER__PAK_HASH="<new-hash>"

# 3. Store it on the admin role
brokkr-broker rotate admin
```

Letting the broker mint the replacement instead:

```bash
# 1. Clear the configured hash so a fresh PAK is generated
export BROKKR__BROKER__PAK_HASH=""

# 2. Rotate
brokkr-broker rotate admin

# 3. Read the new PAK from the key file, immediately
cat /tmp/brokkr-keys/key.txt
```

Either way the old admin PAK stops working once the new hash is in the database, subject to the auth cache — a `rotate admin` run from the CLI cannot reach a running broker's cache, so the previous PAK may keep authenticating for up to `broker.auth_cache_ttl_seconds` (default 60). See [Cache Considerations After CLI Rotation](#cache-considerations-after-cli-rotation).

### In Kubernetes (cold start from inside the pod)

`rotate admin` is CLI-only, so on a chart install you run it in the broker pod. The broker image ships the same binary that serves the API.

**Use `generate-pak`, not `rotate admin`, as the first step.** The chart always sets `broker.pakHash`, so an exec'd shell inherits `BROKKR__BROKER__PAK_HASH` and `rotate admin` takes the re-apply branch — it will tell you it minted nothing. `generate-pak` needs no database and prints both values you need:

```bash
kubectl exec -n brokkr-system deploy/brokkr-broker -- brokkr-broker generate-pak
```

Store the PAK in your secret manager, then put the hash where the chart reads it — preferably a Secret rather than a values file:

```bash
kubectl create secret generic brokkr-admin-pak \
  --from-literal=BROKKR__BROKER__PAK_HASH='<hash>' \
  --dry-run=client -o yaml | kubectl apply -f -

helm upgrade brokkr-broker ... --set broker.pakHashExistingSecret=brokkr-admin-pak
```

Once the pod is running with the new hash configured, commit it to the database:

```bash
kubectl exec -n brokkr-system deploy/brokkr-broker -- brokkr-broker rotate admin
```

That run takes the re-apply branch on purpose — configuration and `admin_role` now agree, and the old admin PAK stops working.

> Do not skip the last step. Authentication reads `admin_role.pak_hash` from the database, and `upsert_admin` runs only on a database that has never been initialized. Changing the chart value alone leaves the *old* hash stored and still accepted — the case the startup warning calls out as a stale **stored** hash.

The alternative — clearing the hash so the broker mints one for you — works but leaves configuration and database disagreeing until you set the printed hash back into the chart, so it is the longer road:

```bash
kubectl exec -n brokkr-system deploy/brokkr-broker -- \
  sh -c 'BROKKR__BROKER__PAK_HASH="" brokkr-broker rotate admin'
```

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
kubectl create secret generic brokkr-agent-credentials \
  --from-literal=BROKKR__AGENT__PAK="brokkr_BRnewKey_NewLongTokenValue1234567890" \
  --dry-run=client -o yaml | kubectl apply -f -
```

The key must be `BROKKR__AGENT__PAK` — that is the agent chart's default `broker.existingSecretKey`, and the chart injects it into the pod by that name. Point the release at the Secret with `--set broker.existingSecret=brokkr-agent-credentials`; a mismatched key leaves the pod in `CreateContainerConfigError`. Because the value is read once at container start, updating the Secret does not restart the agent — follow it with `kubectl rollout restart deploy/brokkr-agent`. See [Installing Brokkr](../getting-started/installation.md) for the full Secret-based install.

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
