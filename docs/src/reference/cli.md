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

---

#### `brokkr-broker create agent`

Creates a new agent record and generates its initial PAK.

```bash
brokkr-broker create agent --name <name> --cluster-name <cluster>
```

**Flags:**

| Flag | Required | Description |
|------|----------|-------------|
| `--name` | Yes | Human-readable agent name |
| `--cluster-name` | Yes | Name of the Kubernetes cluster this agent represents |

**Output:**

```
Agent created successfully:
ID: a1b2c3d4-e5f6-7890-abcd-ef1234567890
Name: production-us-east
Cluster: us-east-1-prod
Initial PAK: brokkr_BRx9y2Kq_A1B2C3D4E5F6G7H8I9J0K1L2
```

> **Important:** The PAK is only displayed once. Store it securely.

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

Behavior depends on `broker.pak_hash`:

- If `broker.pak_hash` is set and non-empty, the configured hash is validated and stored; no new PAK is generated.
- If `broker.pak_hash` is unset or empty, a new admin PAK is generated and its hash stored. The PAK is written to `/tmp/brokkr-keys/key.txt`; it is never printed to stdout.

The previously stored hash is replaced, so the old admin PAK stops working.

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

  PAK (secret — send as `Authorization: Bearer <PAK>`):
    brokkr_BRx9y2Kq_A1B2C3D4E5F6G7H8I9J0K1L2

  PAK hash (set as BROKKR__BROKER__PAK_HASH before first startup):
    sha256:9f86d081884c7d659a2feaa0c55ad015a3bf4f1b2b0b822cd15d6c15b0f00a08
```

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

**Generator scope self-registration (optional):**

On startup the agent registers itself with the generator scopes it resolves, in precedence order:

| Precedence | Source | Notes |
|------------|--------|-------|
| 1 | `--generator-ids <csv>` flag | Comma-separated UUIDs. |
| 2 | `BROKKR__AGENT__GENERATOR_IDS` (config key `agent.generator_ids`) | Comma-separated UUIDs, or a YAML list in the config file. |
| 3 | `BROKKR_GENERATOR_IDS` | Deprecated legacy bare variable; still honored, logs a warning. |

Malformed UUIDs are skipped with a warning. An agent must be registered with a generator before any of that generator's stacks can be targeted at it. Every agent is auto-registered with the system generator regardless of this setting; if no scopes are set the agent has the system/fleet scope only. See [Generator Registration](../explanation/security-model.md#generator-registration-and-application-scopes) and [`BROKKR__AGENT__GENERATOR_IDS`](./environment-variables.md).

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

`brokkr` is the control-plane client. It submits a folder of Kubernetes manifests as a stack's desired state. It is built from the `brokkr-cli` crate (`crates/brokkr-cli`) and wraps the Rust SDK's `BrokkrClient::apply`.

### Connection settings

Every command resolves a broker URL and a PAK from three sources, in precedence order: **command-line flag → environment variable → config file**. A blank value in one source is treated as unset and falls through to the next (`crates/brokkr-cli/src/config.rs`, `resolve`).

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

The broker URL may be given with or without the `/api/v1` suffix — it is appended when absent and never doubled (`normalize_base_url`). The flags are global and may appear before or after the subcommand.

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

`apply` requires a **generator** PAK — the stack is owned by the generator the PAK resolves to. It prints one of three lines and exits `0`:

| Output | Meaning |
|--------|---------|
| `created stack "<name>": first revision (sequence <n>)` | Stack and its first deployment object were created. |
| `updated stack "<name>": new revision (sequence <n>)` | Bundle changed; a new deployment object was submitted. |
| `unchanged: stack "<name>" already current` | Latest deployment object already matches the bundle; nothing submitted. |

On any error (no connection settings, malformed config, unreadable bundle, broker rejection) the command prints `error: <message>` to stderr and exits `1`.

### `brokkr register`

Registers an agent with a generator scope on the agent's behalf. An agent must be registered with a generator before any of that generator's stacks can be targeted at it. Agents normally self-register on startup (see [`brokkr-agent start`](#brokkr-agent-start)); use this to register an agent before it is live, or to add a scope. Requires an admin PAK. Re-registering an already-registered pair returns `409 already_registered` and exits `1` (only the agent's own startup self-registration treats that as success).

```bash
brokkr register --agent <agent-id> --generator <generator-id>
```

**Flags:**

| Flag | Required | Description |
|------|----------|-------------|
| `--agent <UUID>` | yes | The agent to register. |
| `--generator <UUID>` | yes | The generator scope to register it with. |

See [Generator Registration](../explanation/security-model.md#generator-registration-and-application-scopes) for the model and [Agent registration](../how-to/agent-registration.md) for the operational guide.

### `brokkr deregister`

Removes an agent's registration from a generator scope. Requires an admin PAK.

```bash
brokkr deregister --agent <agent-id> --generator <generator-id>
```

**Flags:**

| Flag | Required | Description |
|------|----------|-------------|
| `--agent <UUID>` | yes | The agent to deregister. |
| `--generator <UUID>` | yes | The generator scope to remove. |

Destructive: the broker also removes the agent's `agent_targets` for that generator's stacks and pushes a target-changed frame to the agent, which prunes the corresponding Kubernetes resources on its next reconcile.

### `brokkr registrations`

Lists the generator scopes one agent is registered with, or the agents registered with one generator. Exactly one of `--agent` or `--generator` is required. Cross-entity queries require an admin PAK.

```bash
# Generator scopes an agent is registered with
brokkr registrations --agent <agent-id>

# Agents registered with a generator
brokkr registrations --generator <generator-id>
```

**Flags:**

| Flag | Required | Description |
|------|----------|-------------|
| `--agent <UUID>` | one of¹ | List the agent's generator registrations. |
| `--generator <UUID>` | one of¹ | List the generator's registered agents. |

¹ Exactly one of `--agent` or `--generator` must be given (mutually exclusive).

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
