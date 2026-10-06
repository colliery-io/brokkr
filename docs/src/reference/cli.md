# CLI Reference

Brokkr provides three command-line binaries: `brokkr-broker` for the central management server, `brokkr-agent` for the cluster-side agent, and `brokkr`, the control-plane client (documented in the [`brokkr`](#brokkr) section below). The two server binaries are configured through embedded defaults, an optional `BROKKR_CONFIG_FILE` TOML layer, and environment variables; `brokkr` uses its own flags / `~/.brokkr/config` (see its section).

## brokkr-broker

The broker binary runs the central API server and provides administrative commands for managing agents, generators, and keys.

### Commands

#### `brokkr-broker serve`

Starts the broker HTTP server on `0.0.0.0:3000`.

```bash
brokkr-broker serve
```

**Endpoints exposed:**

| Path | Purpose |
|------|---------|
| `/api/v1/*` | REST API (see [API Reference](./api/README.md)) |
| `/healthz` | Liveness probe |
| `/readyz` | Readiness probe |
| `/metrics` | Prometheus metrics |
| `/swagger-ui` | Interactive API documentation |
| `/docs/openapi.json` | OpenAPI 3 specification |
| `/` | Operator Console — the read-only web view of fleet, deployments, and telemetry. Any path not owned by `/api` or `/internal` falls back to the console shell |

The console is served only by builds that include the `embed-ui` feature. The published broker container image is built that way, so a stock `brokkr-broker` container serves the console on the same port as the API; a binary you build yourself without the feature serves a placeholder instead.

---

#### `brokkr-broker create agent`

Creates a new agent record, generates its initial PAK, and registers it with the system generator plus any generators you name.

```bash
brokkr-broker create agent --name <name> --cluster-name <cluster> [--generator-ids <uuid,...>]
```

**Flags:**

| Flag | Required | Description |
|------|----------|-------------|
| `--name` | Yes | Human-readable agent name |
| `--cluster-name` | Yes | Name of the Kubernetes cluster this agent represents |
| `--generator-ids` | No | Additional generators to register the agent with. Accepts a comma-separated list, and may be repeated. |

Every agent is registered with the system generator (the fleet scope) whether or not you pass `--generator-ids`, which matches what `POST /api/v1/agents` does. Each ID you pass is checked before anything is created, so a typo fails the whole command rather than leaving a half-registered agent behind.

**Output:**

```
Agent created successfully:
ID: a1b2c3d4-e5f6-7890-abcd-ef1234567890
Name: production-us-east
Cluster: us-east-1-prod
Initial PAK: brokkr_BRx9y2Kq_A1B2C3D4E5F6G7H8I9J0K1L2
Registered with: system (fleet scope), f8e7d6c5-b4a3-2109-8765-432109876543
```

> **Important:** The PAK is only displayed once. Store it securely.

If the broker has never been started against this database, the system generator does not exist yet. The agent is still created, but the command logs a warning and the `Registered with:` line omits the fleet scope — start the broker once to provision the system generator, then register the agent with [`brokkr register`](#brokkr-register).

---

#### `brokkr-broker create generator`

Creates a new generator for CI/CD integration.

```bash
brokkr-broker create generator --name <name> [--description <desc>]
```

**Flags:**

| Flag | Required | Description |
|------|----------|-------------|
| `--name` | Yes | Generator name (1-255 characters) |
| `--description` | No | Optional description |

**Output:**

```
Generator created successfully:
ID: f8e7d6c5-b4a3-2109-8765-432109876543
Name: github-actions
Initial PAK: brokkr_BRy8z3Lp_M1N2O3P4Q5R6S7T8U9V0W1X2
```

---

#### `brokkr-broker rotate admin`

Re-runs the admin-key upsert.

```bash
brokkr-broker rotate admin
```

Behavior depends on `broker.pak_hash`, and the command reports which branch it took:

- If `broker.pak_hash` is set and non-empty, the configured hash is validated and stored. An unset value takes this branch too, because the embedded default configuration supplies the publicly known development hash. In this branch **no new PAK is generated and nothing is revoked**. Any PAK matching that hash keeps working. The output says so and lists the two ways to actually replace the credential.
- If `broker.pak_hash` is an explicit empty string (for example `BROKKR__BROKER__PAK_HASH=""`), a new admin PAK is generated and its hash stored. **Both the PAK and its hash are printed**; the PAK is shown once and cannot be recovered from the hash. It is also written to `/tmp/brokkr-keys/key.txt`, which is deleted on graceful shutdown.

The old admin PAK stops working only on the second branch, where the stored hash actually changes. Take the printed hash as well as the PAK — it belongs in `BROKKR__BROKER__PAK_HASH` (or the chart's `broker.pakHash` / `broker.pakHashExistingSecret`), and it cannot be derived from the PAK with `sha256sum` because only the long-token component is hashed.

See [Managing PAKs](../how-to/pak-management.md#rotating-the-admin-pak) for the full flow, including the Kubernetes cold start.

---

#### `brokkr-broker rotate agent`

Rotates an agent's PAK.

```bash
brokkr-broker rotate agent --uuid <uuid>
```

**Flags:**

| Flag | Required | Description |
|------|----------|-------------|
| `--uuid` | Yes | The agent's UUID |

Prints the new PAK to stdout (shown once). The REST endpoint `POST /api/v1/agents/{id}/rotate-pak` is equivalent and additionally invalidates the broker's auth cache immediately; after CLI rotation the old PAK may continue to authenticate for up to `broker.auth_cache_ttl_seconds` (default 60).

---

#### `brokkr-broker rotate generator`

Rotates a generator's PAK.

```bash
brokkr-broker rotate generator --uuid <uuid>
```

**Flags:**

| Flag | Required | Description |
|------|----------|-------------|
| `--uuid` | Yes | The generator's UUID |

Prints the new PAK to stdout (shown once). The REST endpoint `POST /api/v1/generators/{id}/rotate-pak` is equivalent and additionally invalidates the broker's auth cache immediately; after CLI rotation the old PAK may continue to authenticate for up to `broker.auth_cache_ttl_seconds` (default 60).

---

#### `brokkr-broker generate-pak`

Mints an admin PAK and its SHA-256 hash offline, for day-zero bootstrap. Contacts neither the database nor a keyfile.

```bash
brokkr-broker generate-pak
```

Prints the PAK (the admin credential — store securely) and its hash. Set the hash as `BROKKR__BROKER__PAK_HASH` before the broker's first startup; the broker stores it on the admin role at boot.

**Output:**

```
Minted admin PAK (offline — nothing was written to the database):

  PAK (secret — send as `Authorization: Bearer <PAK>`; store securely):
    brokkr_BRx9y2Kq_A1B2C3D4E5F6G7H8I9J0K1L2

  PAK hash (set as BROKKR__BROKER__PAK_HASH before first startup):
    9f86d081884c7d659a2feaa0c55ad015a3bf4f1b2b0b822cd15d6c15b0f00a08

Day-zero flow:
  1. export BROKKR__BROKER__PAK_HASH=9f86d081884c7d659a2feaa0c55ad015a3bf4f1b2b0b822cd15d6c15b0f00a08
  2. Start the broker. First startup stores this hash on the admin role
     and the admin generator; no key file is written to /tmp/brokkr-keys/.
  3. Authenticate with the PAK above. It cannot be recovered from the hash,
     so the hash is safe to keep in config while the PAK stays secret.
```

The hash is printed as bare hexadecimal — exactly 64 characters, with no `sha256:` or other prefix. Copy it verbatim; the broker validates the configured hash at startup and refuses to boot with "Invalid PAK hash provided in configuration" if it is anything other than 64 hex characters.

See the [Environment Variables Reference](./environment-variables.md) for `BROKKR__BROKER__PAK_HASH`.

---

## brokkr-agent

The agent binary runs in each target Kubernetes cluster and polls the broker for deployment objects to apply.

### Commands

#### `brokkr-agent start`

Starts the agent process.

```bash
brokkr-agent start
```

**Health endpoints exposed on `agent.health_port` (default: 8080):**

| Path | Purpose |
|------|---------|
| `/healthz` | Liveness probe (always 200 OK) |
| `/readyz` | Readiness probe (checks Kubernetes API connectivity only) |
| `/health` | Detailed health status (JSON) |
| `/metrics` | Prometheus metrics |

**A new agent is inactive.** A new agent starts `INACTIVE` and applies nothing. On each poll while it is not `ACTIVE`, the agent logs one line at info level that names the fix:

```
Agent 'prod-1' (id: a1b2c3d4-e5f6-7890-abcd-ef1234567890) is INACTIVE. It applies nothing until an admin activates it. To activate it, run: brokkr agent activate prod-1
```

An admin activates it with [`brokkr agent activate`](#brokkr-agent-activate).

**Generator scope self-registration (optional):**

On startup the agent registers itself with the generator scopes it resolves, in precedence order:

| Precedence | Source | Notes |
|------------|--------|-------|
| 1 | `--generator-ids <csv>` flag | Comma-separated UUIDs. |
| 2 | `BROKKR__AGENT__GENERATOR_IDS` (config key `agent.generator_ids`) | Comma-separated UUIDs, or a YAML list in the config file. |
| 3 | `BROKKR_GENERATOR_IDS` | Deprecated legacy bare variable; still honored, logs a warning. |

Malformed UUIDs are skipped with a warning. An agent must be registered with a generator before any of that generator's stacks can be targeted at it.

Agents are registered with the system generator (the fleet scope) at creation time, by both `POST /api/v1/agents` and [`brokkr-broker create agent`](#brokkr-broker-create-agent) — this setting only adds application scopes on top. An agent with no scopes set therefore has the system/fleet scope only. See [Generator Registration](../explanation/security-model.md#generator-registration-and-application-scopes) and [`BROKKR__AGENT__GENERATOR_IDS`](./environment-variables.md).

---

## Configuration

Both **server** binaries (`brokkr-broker` and `brokkr-agent`) read configuration from the same layered system (the `brokkr` client is configured separately — see the [`brokkr`](#brokkr) section):

1. **Embedded defaults** (`default.toml` compiled into the binary)
2. **Configuration file** (optional; path from `BROKKR_CONFIG_FILE`)
3. **Environment variables** (prefix: `BROKKR__`, separator: `__`)

For the server binaries there is no command-line flag for loading a configuration file; set the `BROKKR_CONFIG_FILE` environment variable instead (the `brokkr` client does take a `--config` flag). It loads the TOML file as a layer between embedded defaults and `BROKKR__*` environment variables, and additionally enables the broker's file-change watcher used for hot-reload in Kubernetes (ConfigMap-mounted files).

See the [Configuration Guide](../getting-started/configuration.md) for all available settings and the [Environment Variables Reference](./environment-variables.md) for the complete variable listing.

---

## Exit Codes

| Code | Meaning |
|------|---------|
| 0 | Clean shutdown, including graceful shutdown on SIGINT (Ctrl+C) |
| 1 | Command returned an error (e.g. server failed to bind, database error during a CLI command) |
| 101 | Startup panic (configuration, telemetry, or PAK-controller initialization failure) |

---

## brokkr

`brokkr` is the control-plane client. It submits a folder of Kubernetes manifests as a stack's desired state. It wraps the Rust SDK's `apply` operation. It also does the day-zero steps: it activates, labels and lists agents, and labels, targets and lists stacks (see [Day-zero commands](#day-zero-commands-brokkr-agent-and-brokkr-stack)).

### Connection settings

Every command resolves a broker URL and a PAK from three sources, in precedence order: **command-line flag → environment variable → config file**. A blank value in one source is treated as unset and falls through to the next.

| Setting | Flag | Environment variable | Config-file key |
|---------|------|----------------------|-----------------|
| Broker URL | `--broker-url <URL>` | `BROKKR_BROKER_URL` | `broker_url` |
| PAK | `--pak <PAK>` | `BROKKR_PAK` | `pak` |
| Config-file path | `--config <PATH>` | — | — |

The config file is TOML at `~/.brokkr/config` by default (override with `--config`). A missing file is not an error; a present-but-malformed file is. Example:

```toml
broker_url = "https://broker.example.com"
pak = "brokkr_BRabcd1234_GeneratorTokenExample0001"
```

The broker URL may be given with or without the `/api/v1` suffix — it is appended when absent and never doubled. The flags are global and may appear before or after the subcommand.

### `brokkr apply`

Makes a folder of manifests the desired state of a stack, creating the stack if it does not exist. Idempotent: a re-run with an unchanged bundle submits no new revision and reports `unchanged`.

```bash
brokkr apply -f ./manifests --stack payments --target-label env:prod
```

**Flags:**

| Flag | Required | Description |
|------|----------|-------------|
| `-f`, `--filename <PATH>` | yes | Folder of manifests (top-level `*.yaml`/`*.yml`, sorted) or a single file. |
| `--stack <NAME>` | yes | Stack name; created if absent. |
| `--target-label <LABEL>` | no | Targeting label for agent fan-out (e.g. `env:prod`). Repeatable. |
| `--generator <NAME_OR_ID>` | with an admin PAK | The tenant that owns the stack, by name or id (the API calls a tenant a generator). Required with an admin PAK. Optional with a tenant PAK, where it must name the tenant of that PAK. |

With a **tenant** PAK, the tenant of that PAK owns the stack. With an **admin** PAK, the tenant named in `--generator` owns the stack; without it, the command exits `1` with an error that names the flag. It prints one of three lines and exits `0`:

| Output | Meaning |
|--------|---------|
| `created stack "<name>": first revision (sequence <n>)` | Stack and its first deployment object were created. |
| `updated stack "<name>": new revision (sequence <n>)` | Bundle changed; a new deployment object was submitted. |
| `unchanged: stack "<name>" already current` | Latest deployment object already matches the bundle; nothing submitted. |

On any error (no connection settings, malformed config, unreadable bundle, broker rejection) the command prints `error: <message>` to stderr and exits `1`.

### `brokkr register`

Registers an agent with a tenant for the agent. An agent must be registered with a tenant before any of the stacks of that tenant can target it. Agents normally self-register on startup (see [`brokkr-agent start`](#brokkr-agent-start)); use this to register an agent before it is live, or to add a tenant. Requires an admin PAK. Re-registering an already-registered pair returns `409 already_registered` and exits `1` (only the agent's own startup self-registration treats that as success).

```bash
brokkr register --agent <agent-id> --generator <tenant-id>
```

**Flags:**

| Flag | Required | Description |
|------|----------|-------------|
| `--agent <UUID>` | yes | The agent to register. |
| `--generator <UUID>` | yes | The tenant to register it with (its generator id in the API). |

See [Generator Registration](../explanation/security-model.md#generator-registration-and-application-scopes) for the model and [Agent registration](../how-to/agent-registration.md) for the operational guide.

### `brokkr deregister`

Removes an agent's registration from a tenant. Requires an admin PAK.

```bash
brokkr deregister --agent <agent-id> --generator <tenant-id>
```

**Flags:**

| Flag | Required | Description |
|------|----------|-------------|
| `--agent <UUID>` | yes | The agent to deregister. |
| `--generator <UUID>` | yes | The tenant to remove the agent from. |

Destructive: the broker also removes the agent's `agent_targets` for the stacks of that tenant and pushes a target-changed frame to the agent, which prunes the corresponding Kubernetes resources on its next reconcile.

### `brokkr registrations`

Lists the tenants that one agent is registered with, or the agents registered with one tenant. Exactly one of `--agent` or `--generator` is required. Cross-entity queries require an admin PAK.

```bash
# Tenants an agent is registered with
brokkr registrations --agent <agent-id>

# Agents registered with a tenant
brokkr registrations --generator <tenant-id>
```

**Flags:**

| Flag | Required | Description |
|------|----------|-------------|
| `--agent <UUID>` | one of¹ | List the tenant registrations of the agent. |
| `--generator <UUID>` | one of¹ | List the registered agents of the tenant. |

¹ Exactly one of `--agent` or `--generator` must be given (mutually exclusive).

### Day-zero commands: `brokkr agent` and `brokkr stack`

These commands do the first steps for a new agent or stack: activate the agent, add labels, target a stack at an agent, and list what the broker has. Each command takes an agent or a stack by **name or id**. The CLI looks up the id for you, so you do not need `curl` and `jq` to find it. An id matches first. If two agents have the same name, the command stops and prints their ids; give the id in its place.

Every command that adds something is safe to run again. If the label or the target exists already, the command prints a line that starts with `unchanged:` and exits `0`. On an error, the command prints `error: <message>` to stderr and exits `1`. A label that does not have the form `key:value` is a usage error: the command exits `2` and sends nothing to the broker.

| Command | PAK | What it does |
|---------|-----|--------------|
| `brokkr agent activate <AGENT>` | admin | Sets the agent status to `ACTIVE`. |
| `brokkr agent pause <AGENT>` | admin | Sets the agent status to `INACTIVE`. |
| `brokkr agent label <AGENT> <LABEL>` | admin | Adds a label to the agent. |
| `brokkr agent list` | admin or generator | Lists the agents. |
| `brokkr stack label <STACK> <LABEL>` | admin or owning generator | Adds a label to the stack. |
| `brokkr stack target <STACK> <AGENT>` | admin or owning generator | Sends the stack to the agent. |
| `brokkr stack list` | admin or generator | Lists the stacks. |

#### `brokkr agent activate`

Lets an agent apply its stacks. A new agent starts `INACTIVE` and applies nothing. While it is `INACTIVE`, the agent logs this at info level on each poll and names this command. After you activate it, the agent applies its stacks on its next poll.

```bash
brokkr agent activate prod-1
```

```
agent "prod-1" (a1b2c3d4-e5f6-7890-abcd-ef1234567890) is ACTIVE. It applies its stacks on its next poll.
```

This is the same as `PUT /api/v1/agents/{id}` with `{"status": "ACTIVE"}`.

#### `brokkr agent pause`

Stops an agent from applying its stacks. The command sets the status to `INACTIVE`. The agent keeps the resources that it applied, but it applies no new changes until you activate it again.

```bash
brokkr agent pause prod-1
```

```
agent "prod-1" (a1b2c3d4-e5f6-7890-abcd-ef1234567890) is INACTIVE. It applies nothing until you run: brokkr agent activate prod-1
```

#### `brokkr agent label`

Adds a label to an agent. A stack that has the same label goes to the agent. The label has the form `key:value`, for example `env:prod`. It can have up to 64 characters and no spaces. See [The label shape](../how-to/managing-stacks.md#the-label-shape).

```bash
brokkr agent label prod-1 env:prod
```

```
added label "env:prod" to agent "prod-1"
```

The API form is `POST /api/v1/agents/{id}/labels` with `{"agent_id": "<id>", "label": "env:prod"}`. The body repeats the agent id.

#### `brokkr agent list`

Lists the agents with their name, id, status, cluster and labels. An admin PAK lists all agents. A generator PAK lists only the agents that are registered with its generator. The broker does not show agent labels to a generator PAK, so that list has no `LABELS` column.

```bash
brokkr agent list
```

```
NAME     ID                                    STATUS    CLUSTER    LABELS
prod-1   a1b2c3d4-e5f6-7890-abcd-ef1234567890  ACTIVE    us-east-1  env:prod,region:us-east
stage-1  0f9e8d7c-6b5a-4321-9876-543210fedcba  INACTIVE  us-west-2  -
```

#### `brokkr stack label`

Adds a label to a stack. The stack goes to each agent that has the same label. The label shape is the same as for agents.

```bash
brokkr stack label payments env:prod
```

```
added label "env:prod" to stack "payments"
```

`brokkr apply --target-label env:prod` adds the same label when it applies a folder.

#### `brokkr stack target`

Sends a stack to one agent, whatever the labels of the agent are. The first argument is the stack; the second is the agent.

```bash
brokkr stack target payments prod-1
```

```
targeted stack "payments" at agent "prod-1"
```

The agent must be registered with the generator that owns the stack. If it is not, the broker refuses the target, and the error gives the command that registers it:

```
error: invalid request: agent "prod-1" is not registered with the generator that owns stack "payments". Register it first: brokkr register --agent <agent-id> --generator <generator-id>
```

The API form is `POST /api/v1/agents/{id}/targets` with `{"agent_id": "<id>", "stack_id": "<id>"}`. The body repeats the agent id.

#### `brokkr stack list`

Lists the stacks with their name, id and labels. An admin PAK lists all stacks. A generator PAK lists the stacks of its generator.

```bash
brokkr stack list
```

```
NAME      ID                                    LABELS
payments  c4ba105f-4332-447b-8af5-63f8e69ee89c  env:prod
```

---

## Examples

```bash
# Start broker with environment overrides
BROKKR__DATABASE__URL=postgres://user:pass@db:5432/brokkr \
BROKKR__LOG__LEVEL=info \
BROKKR__LOG__FORMAT=json \
  brokkr-broker serve

# Create an agent and capture its PAK
brokkr-broker create agent --name prod-1 --cluster-name us-east-1 2>&1 | grep "Initial PAK"

# Create an agent already registered with two application scopes
brokkr-broker create agent --name prod-1 --cluster-name us-east-1 \
  --generator-ids f8e7d6c5-b4a3-2109-8765-432109876543,a1b2c3d4-e5f6-7890-abcd-ef1234567890

# Start agent with environment config
BROKKR__AGENT__BROKER_URL=https://broker.example.com \
BROKKR__AGENT__PAK=brokkr_BRx9y2Kq_A1B2C3D4E5F6G7H8I9J0K1L2 \
BROKKR__AGENT__AGENT_NAME=prod-1 \
BROKKR__AGENT__CLUSTER_NAME=us-east-1 \
  brokkr-agent start

# Start agent and self-register with a generator scope
BROKKR__AGENT__BROKER_URL=https://broker.example.com \
BROKKR__AGENT__PAK=brokkr_BRx9y2Kq_A1B2C3D4E5F6G7H8I9J0K1L2 \
BROKKR__AGENT__AGENT_NAME=prod-1 \
BROKKR__AGENT__CLUSTER_NAME=us-east-1 \
BROKKR__AGENT__GENERATOR_IDS=f8e7d6c5-b4a3-2109-8765-432109876543 \
  brokkr-agent start
```
