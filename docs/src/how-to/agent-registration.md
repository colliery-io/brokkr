# How-To: Register Agents with Generators

An agent must be **registered** with a generator before any stack that generator owns can be targeted at it — registration is the agent's opt-in consent boundary. For the reasoning behind this application-level tenancy model, see [Generator registration and application scopes](../explanation/security-model.md#generator-registration-and-application-scopes). This guide is the operational reference for granting, revoking, and inspecting those registrations.

## Prerequisites

- Admin PAK for cross-entity registration (registering one entity on behalf of another)
- Access to the Brokkr broker API, or the `brokkr` CLI configured with a broker URL and PAK
- The UUIDs of the agents and generators you intend to link

Examples below assume an admin PAK in `$ADMIN_PAK` and a broker reachable at `http://localhost:3000`. The `brokkr` CLI resolves its broker URL and PAK from `--broker-url`/`BROKKR_BROKER_URL` and `--pak`/`BROKKR_PAK` (see the [CLI Reference](../reference/cli.md)).

## Register an Agent at Deploy Time

The most common path is to have the agent register itself at startup. Supply the generator UUIDs the agent should opt into through any one of the following, in precedence order (highest first):

1. `--generator-ids <UUID,UUID,...>` CLI flag on `brokkr-agent start`
2. `BROKKR__AGENT__GENERATOR_IDS` environment variable (config key `agent.generator_ids`)
3. `BROKKR_GENERATOR_IDS` — **deprecated** bare variable, still honored but logs a warning

Values are comma-separated UUIDs; malformed entries are skipped with a warning. An **empty** value is valid and leaves the agent in system/fleet scope only (it still receives system-generator stacks — see [System generator scope](#system-generator-scope)).

### Via Helm (brokkr-agent chart)

Set `broker.generatorIds` as either a YAML list or a comma string; the chart renders it as `BROKKR__AGENT__GENERATOR_IDS` in the agent ConfigMap:

```yaml
# values.yaml
broker:
  generatorIds:
    - a1b2c3d4-e5f6-7890-abcd-ef1234567890
    - b2c3d4e5-f6a7-8901-bcde-f12345678901
```

```bash
helm upgrade --install brokkr-agent oci://ghcr.io/colliery-io/charts/brokkr-agent \
  -f values.yaml
```

The default is empty, which leaves the agent in system/fleet scope only.

### Via environment variable

```bash
BROKKR__AGENT__BROKER_URL=https://broker.example.com \
BROKKR__AGENT__PAK=brokkr_BRx9y2Kq_A1B2C3D4E5F6G7H8I9J0K1L2 \
BROKKR__AGENT__GENERATOR_IDS=a1b2c3d4-e5f6-7890-abcd-ef1234567890,b2c3d4e5-f6a7-8901-bcde-f12345678901 \
  brokkr-agent start
```

### Via CLI flag

```bash
brokkr-agent start \
  --generator-ids a1b2c3d4-e5f6-7890-abcd-ef1234567890,b2c3d4e5-f6a7-8901-bcde-f12345678901
```

## Register or Deregister an Existing Agent (Admin)

Use this when an agent is already running and you need to grant or revoke a generator scope without restarting it.

### Register via the brokkr CLI

```bash
brokkr register \
  --agent a1b2c3d4-e5f6-7890-abcd-ef1234567890 \
  --generator b2c3d4e5-f6a7-8901-bcde-f12345678901 \
  --pak "$ADMIN_PAK"
```

Re-running `brokkr register` for an already-registered pair returns `409 already_registered` and exits non-zero; registering a new pair prints the registration record. (Agent *startup* self-registration, by contrast, treats `409` as success — see [Register an Agent at Deploy Time](#register-an-agent-at-deploy-time).)

### Register via the REST API

`POST /generators/{id}/register` with an `agent_id` in the body registers that agent (admin acting cross-entity). Omit `agent_id` and the calling agent registers itself. A generator PAK cannot call this endpoint — it returns `403 forbidden`:

```bash
curl -s -X POST "http://localhost:3000/api/v1/generators/${GENERATOR_ID}/register" \
  -H "Authorization: Bearer $ADMIN_PAK" \
  -H "Content-Type: application/json" \
  -d '{"agent_id": "'"$AGENT_ID"'"}' | jq .
```

Response (`201 Created`):

```json
{
  "id": "c3d4e5f6-a7b8-9012-cdef-123456789012",
  "agent_id": "a1b2c3d4-e5f6-7890-abcd-ef1234567890",
  "generator_id": "b2c3d4e5-f6a7-8901-bcde-f12345678901",
  "registered_at": "2026-06-27T10:00:00Z"
}
```

### Deregister (destructive)

> **Warning:** Deregistration cascades. It removes the agent's explicit `agent_targets` for every stack the generator owns **and** pushes a `TargetChanged` frame to the agent over its WebSocket, which prunes the corresponding Kubernetes resources on the next reconcile. There is no force flag and admin cannot bypass the consent boundary — deregistering is the supported way to revoke it.

Via the CLI:

```bash
brokkr deregister \
  --agent a1b2c3d4-e5f6-7890-abcd-ef1234567890 \
  --generator b2c3d4e5-f6a7-8901-bcde-f12345678901 \
  --pak "$ADMIN_PAK"
```

Via the REST API:

```bash
curl -s -X DELETE "http://localhost:3000/api/v1/generators/${GENERATOR_ID}/register" \
  -H "Authorization: Bearer $ADMIN_PAK" \
  -H "Content-Type: application/json" \
  -d '{"agent_id": "'"$AGENT_ID"'"}'
```

## Inspect Registrations

### List a generator's scopes for one agent

```bash
brokkr registrations --agent "$AGENT_ID" --pak "$ADMIN_PAK"
```

```bash
curl -s "http://localhost:3000/api/v1/agents/${AGENT_ID}/registrations" \
  -H "Authorization: Bearer $ADMIN_PAK" | jq .
```

### List the agents registered with one generator

```bash
brokkr registrations --generator "$GENERATOR_ID" --pak "$ADMIN_PAK"
```

```bash
curl -s "http://localhost:3000/api/v1/generators/${GENERATOR_ID}/registered-agents" \
  -H "Authorization: Bearer $ADMIN_PAK" | jq .
```

Both endpoints are authorized for admin, or for the agent/generator acting on itself.

## Pre-register at Agent Creation

`POST /agents` accepts an optional `generator_ids` array that pre-registers the new agent at creation time, alongside the automatic system-generator registration every agent receives:

```bash
curl -s -X POST "http://localhost:3000/api/v1/agents" \
  -H "Authorization: Bearer $ADMIN_PAK" \
  -H "Content-Type: application/json" \
  -d '{
    "name": "prod-1",
    "cluster_name": "us-east-1",
    "generator_ids": [
      "a1b2c3d4-e5f6-7890-abcd-ef1234567890",
      "b2c3d4e5-f6a7-8901-bcde-f12345678901"
    ]
  }' | jq .
```

An unknown generator UUID rejects the whole request with `400 invalid_generator_id`.

## System Generator Scope

The system generator (`__system__`) is provisioned at broker startup, and **every** agent is auto-registered with it at creation. It carries fleet- and system-wide stacks that should reach all agents, so an agent with an empty `generator_ids` still serves those stacks. The system generator is excluded from the public `GET /generators` listing.

The system generator is **not** the admin generator — the admin generator is a separate entity tied to the admin role/PAK, and agents are **not** auto-registered with it.

## Troubleshoot `403 agent_not_registered`

A `POST /agents/{id}/targets` (add a target) or `DELETE /agents/{id}/targets/{stack_id}` (remove a target) that fails with HTTP `403` and error code `agent_not_registered` means the agent is not registered with the generator that owns the stack. Both mutations are gated, and admin cannot bypass the check — there is no force flag.

To resolve:

1. Identify the stack's owning generator, then confirm the gap:
   ```bash
   brokkr registrations --agent "$AGENT_ID" --pak "$ADMIN_PAK"
   ```
2. Register the agent with that generator (see [Register or Deregister an Existing Agent](#register-or-deregister-an-existing-agent-admin)).
3. Retry the target mutation.

What registration does **not** change: the read path `GET /agents/{id}/target-state` is unchanged. An agent's served-stack set is still the **union** of explicit `agent_targets`, label matches, and annotation matches. Registration only gates whether an explicit target can be **created** — it does not alter the read-time union, and existing targets remain valid. (Migration 23 back-fills registrations from pre-existing `agent_targets`, so upgrades do not break current targeting.)

## A Note on the Two Isolation Layers

Generator registration is **application-level** isolation within a single broker. It is complementary to, and must not be conflated with, **schema-per-tenant** (`database.schema`) isolation, which is a separate deployment-level mechanism. See the application-level section of the [Multi-Tenancy Reference](../reference/multi-tenancy.md).

## Related

- [Security Model — Generator registration and application scopes](../explanation/security-model.md#generator-registration-and-application-scopes) — why registration exists
- [Core Concepts](../explanation/core-concepts.md) — generators, agents, and targeting
- [API Reference](../reference/api/README.md) — the registration endpoints, `agent_not_registered`, and `generator_ids`
- [CLI Reference](../reference/cli.md) — `register` / `deregister` / `registrations` and `generate-pak`
- [Environment Variables Reference](../reference/environment-variables.md) — `BROKKR__AGENT__GENERATOR_IDS` and the deprecated `BROKKR_GENERATOR_IDS`
- [Error Codes Reference](../reference/error-codes.md) — `agent_not_registered`
- [Generators Reference](../reference/generators.md) — generator entity and permission model
- [Multi-Tenancy Reference](../reference/multi-tenancy.md) — application-level vs. deployment-level isolation
