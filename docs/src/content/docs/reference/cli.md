---
title: CLI Reference
---

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
| `/api/v1/*` | REST API (see [API Reference](../rest-api/)) |
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

See [Managing PAKs](../../how-to/pak-management/#rotating-the-admin-pak) for the full flow, including the Kubernetes cold start.

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

See the [Environment Variables Reference](../environment-variables/) for `BROKKR__BROKER__PAK_HASH`.

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

Agents are registered with the system generator (the fleet scope) at creation time, by both `POST /api/v1/agents` and [`brokkr-broker create agent`](#brokkr-broker-create-agent) — this setting only adds application scopes on top. An agent with no scopes set therefore has the system/fleet scope only. See [Generator Registration](../../explanation/security-model/#generator-registration-and-application-scopes) and [`BROKKR__AGENT__GENERATOR_IDS`](../environment-variables/).

---

## Configuration

Both **server** binaries (`brokkr-broker` and `brokkr-agent`) read configuration from the same layered system (the `brokkr` client is configured separately — see the [`brokkr`](#brokkr) section):

1. **Embedded defaults** (`default.toml` compiled into the binary)
2. **Configuration file** (optional; path from `BROKKR_CONFIG_FILE`)
3. **Environment variables** (prefix: `BROKKR__`, separator: `__`)

For the server binaries there is no command-line flag for loading a configuration file; set the `BROKKR_CONFIG_FILE` environment variable instead (the `brokkr` client does take a `--config` flag). It loads the TOML file as a layer between embedded defaults and `BROKKR__*` environment variables, and additionally enables the broker's file-change watcher used for hot-reload in Kubernetes (ConfigMap-mounted files).

See the [Configuration Guide](../../how-to/configuration/) for all available settings and the [Environment Variables Reference](../environment-variables/) for the complete variable listing.

---

## Exit Codes

| Code | Meaning |
|------|---------|
| 0 | Clean shutdown, including graceful shutdown on SIGINT (Ctrl+C) |
| 1 | Command returned an error (e.g. server failed to bind, database error during a CLI command) |
| 101 | Startup panic (configuration, telemetry, or PAK-controller initialization failure) |

---

## brokkr

`brokkr` is the control-plane client. It submits a folder of Kubernetes manifests as a stack's desired state. It wraps the Rust SDK's `apply` operation. It also does the admin's one-time steps: it creates and lists generators (see [`brokkr generator`](#brokkr-generator)), registers agents with them, and does the day-zero steps: it activates, labels and lists agents, and labels, targets and lists stacks (see [Day-zero commands](#day-zero-commands-brokkr-agent-and-brokkr-stack)).

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
| `--generator <NAME_OR_ID>` | with an admin PAK | The generator that owns the stack, by name or id. Required with an admin PAK. Optional with a generator PAK, where it must name the generator of that PAK. |

With a **generator** PAK, the generator of that PAK owns the stack. With an **admin** PAK, the generator named in `--generator` owns the stack; without it, the command exits `1` with an error that names the flag. It prints one of three lines and exits `0`:

| Output | Meaning |
|--------|---------|
| `created stack "<name>": first revision (sequence <n>)` | Stack and its first deployment object were created. |
| `updated stack "<name>": new revision (sequence <n>)` | Bundle changed; a new deployment object was submitted. |
| `unchanged: stack "<name>" already current` | Latest deployment object already matches the bundle; nothing submitted. |

On any error (no connection settings, malformed config, unreadable bundle, broker rejection) the command prints `error: <message>` to stderr and exits `1`.

### `brokkr generator`

Creates and lists generators. With `brokkr register` and `brokkr agent activate`, the admin's one-time steps for a new generator are one tool. Both commands require an admin PAK. For one release, `brokkr tenant` is a hidden alias of `brokkr generator`, and `--tenant` of `brokkr agent create` is a hidden alias of `--generator`.

#### `brokkr generator create`

```bash
brokkr generator create <NAME> [--description <TEXT>]
```

```
created generator "team-acme" (a1b2c3d4-e5f6-7890-abcd-ef1234567890)
PAK: brokkr_BRgen12ab_GeneratorLongTokenExample01
The PAK is shown once. Store it where the generator's CI reads it; the only way back is a rotation. Next: brokkr register --agent <AGENT> --generator team-acme
```

The PAK is printed once: the broker keeps only a hash. A name that exists already is a `409`; the command prints `error: <message>` to stderr and exits `1`. This is the same as `POST /api/v1/generators`.

#### `brokkr generator rotate`

Rotates a generator's PAK and prints the new one once. With a generator PAK, it rotates its own: give no generator. With an admin PAK, name the generator. The current PAK stops at once.

```bash
brokkr generator rotate            # as the generator
brokkr generator rotate team-acme  # as an admin
```

```
rotated the PAK of generator "team-acme" (a1b2c3d4-...)
PAK: brokkr_BRnew34cd_GeneratorLongTokenExample02
The PAK is shown once. The previous PAK stopped: update every pipeline that uses it.
```

This is `POST /api/v1/generators/{id}/rotate-pak`.

#### `brokkr generator list`

```bash
brokkr generator list
```

Prints a table with `NAME`, `ID`, `STATUS`, `LAST ACTIVE` and `DESCRIPTION`. The system generator is not listed. This is the same as `GET /api/v1/generators`.

### `brokkr whoami`

Says what the PAK is and which broker the CLI talks to.

```bash
brokkr whoami
```

```
broker: https://broker.example.com
PAK: generator "team-acme" (a1b2c3d4-...)
```

An admin PAK prints `PAK: admin`; an agent PAK prints the agent's name, id and cluster. This is `POST /api/v1/auth/pak`.

### `brokkr register`

Registers an agent with a generator for the agent. An agent must be registered with a generator before any of the stacks of that generator can target it. Agents normally self-register on startup (see [`brokkr-agent start`](#brokkr-agent-start)); use this to register an agent before it is live, or to add a generator. Requires an admin PAK. You can run this command again: if the agent is registered with the generator already, the command changes nothing and exits `0`, the same as [`brokkr agent label`](#day-zero-commands-brokkr-agent-and-brokkr-stack). The API still returns `409 already_registered` for a repeat; the CLI shows it as `unchanged:`.

```bash
brokkr register --agent <NAME_OR_ID> --generator <NAME_OR_ID>
```

**Flags:**

| Flag | Required | Description |
|------|----------|-------------|
| `--agent <NAME_OR_ID>` | yes | The agent to register, by name or id. |
| `--generator <NAME_OR_ID>` | yes | The generator to register it with, by name or id. |

The CLI finds each name or id in the same way as `brokkr agent` and `brokkr stack`. An id is used as it is. A name is looked up in the agent list or the generator list. If two agents have the same name (in different clusters), the command stops and lists their ids; give the id instead. For example:

```bash
brokkr register --agent prod-1 --generator payments-team
```

```
registered agent "prod-1" (6b1f...) with generator "payments-team" (e5fe...) (registration 0c7d...)
```

It prints one of two lines and exits `0`:

| Output | Meaning |
|--------|---------|
| `registered agent <agent> with generator <generator> (registration <id>)` | The broker added the registration. |
| `unchanged: agent <agent> is registered with generator <generator> already` | The registration exists already. The broker changed nothing. |

On an error, the command prints `error: <message>` to stderr and exits `1`.

See [Generator Registration](../../explanation/security-model/#generator-registration-and-application-scopes) for the model and [Agent registration](../../how-to/agent-registration/) for the operational guide.

### `brokkr deregister`

Removes an agent's registration from a generator. Requires an admin PAK.

```bash
brokkr deregister --agent <NAME_OR_ID> --generator <NAME_OR_ID>
```

**Flags:**

| Flag | Required | Description |
|------|----------|-------------|
| `--agent <NAME_OR_ID>` | yes | The agent to deregister, by name or id. |
| `--generator <NAME_OR_ID>` | yes | The generator to remove the agent from, by name or id. |

The lookup is the same as for [`brokkr register`](#brokkr-register).

Destructive: the broker also removes the agent's `agent_targets` for the stacks of that generator and pushes a target-changed frame to the agent, which prunes the corresponding Kubernetes resources on its next reconcile.

If the agent is not registered with the generator, the broker removes nothing and returns success. The command prints the same `deregistered` line and exits `0`.

### `brokkr registrations`

Lists the generators that one agent is registered with, or the agents registered with one generator. Exactly one of `--agent` or `--generator` is required. Cross-entity queries require an admin PAK.

```bash
# Generators an agent is registered with
brokkr registrations --agent <NAME_OR_ID>

# Agents registered with a generator
brokkr registrations --generator <NAME_OR_ID>
```

**Flags:**

| Flag | Required | Description |
|------|----------|-------------|
| `--agent <NAME_OR_ID>` | one of¹ | List the generator registrations of the agent, given by name or id. |
| `--generator <NAME_OR_ID>` | one of¹ | List the registered agents of the generator, given by name or id. |

¹ Exactly one of `--agent` or `--generator` must be given (mutually exclusive).

A name lookup reads the agent list or the generator list. Only an admin PAK can read the generator list. With an agent PAK or a generator PAK, give the id. An id is used as it is, with no lookup.

### Day-zero commands: `brokkr agent` and `brokkr stack`

These commands do the first steps for a new agent or stack: activate the agent, add labels, target a stack at an agent, and list what the broker has. Each command takes an agent or a stack by **name or id**. The CLI looks up the id for you, so you do not need `curl` and `jq` to find it. An id matches first. If two agents have the same name, the command stops and prints their ids; give the id in its place.

Every command that adds something is safe to run again. If the label or the target exists already, the command prints a line that starts with `unchanged:` and exits `0`. On an error, the command prints `error: <message>` to stderr and exits `1`. A label that does not have the form `key:value` is a usage error: the command exits `2` and sends nothing to the broker.

| Command | PAK | What it does |
|---------|-----|--------------|
| `brokkr agent create <NAME> --cluster <CLUSTER> [--generator <NAME_OR_ID>]...` | admin | Creates the agent record and prints its PAK once, with the Helm command. |
| `brokkr agent activate <AGENT>` | admin | Sets the agent status to `ACTIVE`. |
| `brokkr agent pause <AGENT>` | admin | Sets the agent status to `INACTIVE`. |
| `brokkr agent label <AGENT> <LABEL>` | admin | Adds a label to the agent. |
| `brokkr agent list` | admin or generator | Lists the agents. |
| `brokkr stack label <STACK> <LABEL>` | admin or owning generator | Adds a label to the stack. |
| `brokkr stack target <STACK> <AGENT>` | admin or owning generator | Sends the stack to the agent. |
| `brokkr stack list` | admin or generator | Lists the stacks. |
| `brokkr stack status <STACK>` | admin or owning generator | The latest revision, the agents that have the stack with their health, and the stack's health. |
| `brokkr stack events <STACK> [--limit N]` | admin or owning generator | The retained Kubernetes events of the stack. |
| `brokkr stack logs <STACK> [--limit N]` | admin or owning generator | The retained pod log lines of the stack. |
| `brokkr stack delete <STACK>` | admin or owning generator | Deletes the stack; each agent that has it removes its resources on its next poll. |

#### `brokkr agent create`

Creates the agent record and prints its PAK once, the Helm command that installs the agent with these values, and the next step. Each `--generator` registers the agent with that generator; every agent is also registered with the system generator.

```bash
brokkr agent create prod-1 --cluster us-east-1 --generator team-acme
```

```
created agent "prod-1" (6b1f...) in cluster "us-east-1"
PAK: brokkr_BRagt56ef_AgentLongTokenExample01
The PAK is shown once.

Install the agent:
helm install brokkr-agent oci://ghcr.io/colliery-io/charts/brokkr-agent \
  --set broker.url=http://broker:3000 \
  --set broker.pak=brokkr_BRagt56ef_AgentLongTokenExample01 \
  --set broker.agentName=prod-1 \
  --set broker.clusterName=us-east-1 \
  --set broker.generatorIds="{e5fe...}"

The agent is INACTIVE. After it checks in, run: brokkr agent activate prod-1
```

The broker URL in the command is the one the CLI uses; change it if the cluster reaches the broker by another address. This is the same as `POST /api/v1/agents` with `generator_ids`.

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

**The agent stops its own work.** The agent reads its status from the broker on each heartbeat. While the status is not `ACTIVE`, the agent does not fetch deployment objects, work orders, health checks, diagnostics or webhook deliveries. It continues to send heartbeats. The broker does not check the status: it serves deployment objects to an `INACTIVE` agent that asks for them. Thus `INACTIVE` is not an access control. To stop an agent that you do not trust, revoke or rotate its PAK (Prefixed API Key). See [Agent Status Is Not an Access Control](../../explanation/security-model/#agent-status-is-not-an-access-control).

```bash
brokkr agent pause prod-1
```

```
agent "prod-1" (a1b2c3d4-e5f6-7890-abcd-ef1234567890) is INACTIVE. It applies nothing until you run: brokkr agent activate prod-1
```

#### `brokkr agent label`

Adds a label to an agent. A stack that has the same label goes to the agent. The label has the form `key:value`, for example `env:prod`. It can have up to 64 characters and no spaces. See [The label shape](../../how-to/managing-stacks/#the-label-shape).

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

After the label line, `brokkr stack label` says which agents the stack's labels reach now: the agents the PAK can see (every agent for an admin; the registered agents for a generator) that carry one of the stack's labels. When none does, it says so, and that the stack reaches no agent until one that is registered with its generator has one of its labels.

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

#### `brokkr stack status`

Where a stack is and whether it is healthy. The latest revision, then one row per agent and deployment object with the agent's latest health report.

```bash
brokkr stack status payments
```

```
stack "payments" (a1b2c3d4-...)
latest revision: sequence 7, submitted 2026-11-07T15:00:00+00:00
health: healthy
AGENT   DEPLOYMENT OBJECT  STATUS   CHECKED
prod-1  d1a2b3c4-...       healthy  2026-11-07T15:00:30+00:00
```

Before any agent has reported, the table is replaced by one line that says so. This is `GET /api/v1/stacks/{id}/deployment-objects` and `GET /api/v1/stacks/{id}/health`; the generator that owns the stack may read both.

#### `brokkr stack events` and `brokkr stack logs`

The retained Kubernetes events of the stack, and the retained pod log lines. The broker keeps a short window (at most six hours). Only pods whose template carries the `brokkr.io/stream-logs` annotation are streamed. `--limit N` caps the rows, newest first; the broker's default is 500.

```bash
brokkr stack events payments --limit 20
brokkr stack logs payments
```

These are `GET /api/v1/stacks/{id}/events` and `GET /api/v1/stacks/{id}/logs`.

#### `brokkr stack delete`

Deletes a stack. The broker records a deletion marker, and each agent that has the stack removes its resources on its next poll. Requires an admin PAK or the PAK of the generator that owns the stack.

```bash
brokkr stack delete payments
```

#### `brokkr stack list`

Lists the stacks with their name, id and labels. An admin PAK lists all stacks. A generator PAK lists the stacks of its generator.

```bash
brokkr stack list
```

```
NAME      ID                                    LABELS
payments  c4ba105f-4332-447b-8af5-63f8e69ee89c  env:prod
```

### Work orders: `brokkr work-order`

A work order runs one time: an agent applies its YAML and watches it to its end. Use a stack for state that an agent keeps. An admin PAK or a generator PAK creates a work order, and the PAK that creates it owns it. The owner and an admin read and delete it. A work order has no name: each command takes its id.

| Command | What it does |
|---------|--------------|
| `brokkr work-order create` | Creates a work order from a file of YAML. |
| `brokkr work-order list` | Lists the work orders that wait or run. |
| `brokkr work-order status <ID>` | Shows the status of one work order, and its result when it has ended. |
| `brokkr work-order delete <ID>` | Deletes a work order that has not ended. |
| `brokkr work-order logs <ID>` | Shows the retained pod log lines of a work order. |
| `brokkr work-order events <ID>` | Shows the retained Kubernetes events of a work order. |

#### `brokkr work-order create`

| Flag | Meaning |
|------|---------|
| `-f`, `--filename <PATH>` | The file of YAML that the agent applies, for example a Job. Required. |
| `--type <TYPE>` | The type of work: `custom` (the default) or `build`. |
| `--agent <NAME_OR_ID>` | An agent that can have the work order. Repeatable. |
| `--label <LABEL>` | An agent with this label can have the work order. The form is `key:value`. Repeatable. |
| `--annotation <KEY=VALUE>` | An agent with this annotation can have the work order. The form is `key=value`. Repeatable. |
| `--claim-timeout <SECONDS>` | How long an agent can hold the work order. After this time the broker gives it to the queue again. The broker's default is 3600. |
| `--max-retries <N>` | How many times the broker tries again after a failure. The broker's default is 3. |
| `--backoff <SECONDS>` | The wait before the first new try. The wait doubles for each new try. The broker's default is 60. |

Give one target at least: `--agent`, `--label` or `--annotation`. An agent that matches one of the targets can claim the work order.

```bash
brokkr work-order create -f backup-job.yaml --agent prod-1 --claim-timeout 600
```

```
created work order 7c1f4a52-9d3e-4b8a-a6f0-2e5d8c9b1a34 (type custom)
targets: agent "prod-1"
An agent that matches a target claims the work order on its next poll.
To follow it, run: brokkr work-order status 7c1f4a52-9d3e-4b8a-a6f0-2e5d8c9b1a34
```

With a generator PAK, each agent of the work order must be registered with the generator. A label or an annotation matches only the agents that are registered with the generator. If an agent given with `--agent` is not registered, the broker creates nothing, and the error gives the command that registers it:

```
error: agent "prod-2" (<agent-id>) is not registered with your generator, so the generator cannot send it a work order. The broker created nothing. An admin registers it: brokkr register --agent <agent-id> --generator <generator-id>
```

This is `POST /api/v1/work-orders`.

#### `brokkr work-order list`

Lists the work orders that wait or run, with their id, type, status, owner and age. An admin PAK lists all of them. A generator PAK lists the work orders of its generator. The owner is the name of a generator, or `admin`.

```bash
brokkr work-order list
```

```
ID                                    TYPE    STATUS   OWNER       AGE
7c1f4a52-9d3e-4b8a-a6f0-2e5d8c9b1a34  custom  CLAIMED  admin       2m
1e0d9c8b-7a6f-4e5d-8c3b-2a1f0e9d8c7b  build   PENDING  team-robin  45s
```

A work order that ended is not in this list. Use `brokkr work-order status` with its id.

#### `brokkr work-order status`

Shows one work order. The statuses are:

| Status | Meaning |
|--------|---------|
| `PENDING` | No agent has claimed it yet. |
| `CLAIMED` | An agent runs it. |
| `RETRY_PENDING` | A try failed, and the broker waits before it gives the work order to the queue again. |
| `COMPLETED` | It ended with success. |
| `FAILED` | It ended with a failure. |

For a work order that ended, the command reads the work order log and shows the result and its message.

```bash
brokkr work-order status 7c1f4a52-9d3e-4b8a-a6f0-2e5d8c9b1a34
```

```
work order 7c1f4a52-9d3e-4b8a-a6f0-2e5d8c9b1a34
type: custom
owner: admin
created: 2026-11-13T10:00:00+00:00
status: COMPLETED at 2026-11-13T10:02:10+00:00. agent "prod-1" ran it. Tries after the first: 0.
result: success
message: none
To see the lines of its pods, run: brokkr work-order logs 7c1f4a52-9d3e-4b8a-a6f0-2e5d8c9b1a34
```

These are `GET /api/v1/work-orders/{id}` and, for a work order that ended, `GET /api/v1/work-order-log/{id}`.

#### `brokkr work-order delete`

Deletes a work order that has not ended. A work order that ended stays in the work order log and cannot be deleted.

```bash
brokkr work-order delete 7c1f4a52-9d3e-4b8a-a6f0-2e5d8c9b1a34
```

```
deleted work order 7c1f4a52-9d3e-4b8a-a6f0-2e5d8c9b1a34
```

#### `brokkr work-order logs` and `brokkr work-order events`

The retained pod log lines of the job of a work order, and its retained Kubernetes events. The agent that runs the work order sends them. The broker keeps a short window (at most six hours), also after the work order ended. `--limit N` caps the rows, newest first; the broker's default is 500. The output has the shape of `brokkr stack logs` and `brokkr stack events`.

```bash
brokkr work-order logs 7c1f4a52-9d3e-4b8a-a6f0-2e5d8c9b1a34 --limit 50
brokkr work-order events 7c1f4a52-9d3e-4b8a-a6f0-2e5d8c9b1a34
```

These are `GET /api/v1/work-orders/{id}/logs` and `GET /api/v1/work-orders/{id}/events`.

#### Refusals

| The broker answers | The command says |
|--------------------|------------------|
| `404 work_order_not_found` | No work order has the id. The command names `brokkr work-order list`. |
| `403 work_order_not_owned` | The work order belongs to another owner. Use the PAK of the generator that owns it, or an admin PAK. |
| `403 work_order_access_denied` | An agent PAK cannot use the work order commands. Use a generator PAK or an admin PAK. |
| `403 agent_not_registered` | An agent of the work order is not registered with the generator. The command names `brokkr register`. |

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
