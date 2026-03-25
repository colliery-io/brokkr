# CLI Reference

Brokkr provides two command-line binaries: `brokkr-broker` for the central management server and `brokkr-agent` for the cluster-side agent. Both accept configuration via files, environment variables, and command-line flags.

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

Generates a new admin PAK, replacing the existing one.

```bash
brokkr-broker rotate admin
```

The new PAK is printed to stdout. The old PAK immediately stops working.

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
| `/readyz` | Readiness probe (checks K8s + broker connectivity) |
| `/health` | Detailed health status (JSON) |
| `/metrics` | Prometheus metrics |

---

## Configuration

Both binaries read configuration from the same layered system:

1. **Embedded defaults** (`default.toml` compiled into the binary)
2. **Configuration file** (optional, path passed at startup or via `BROKKR_CONFIG_FILE`)
3. **Environment variables** (prefix: `BROKKR__`, separator: `__`)

See the [Configuration Guide](../getting-started/configuration.md) for all available settings and the [Environment Variables Reference](./environment-variables.md) for the complete variable listing.

---

## Exit Codes

| Code | Meaning |
|------|---------|
| 0 | Clean shutdown |
| 1 | Startup failure (database, config, migration error) |
| 130 | SIGINT (Ctrl+C) received |
| 143 | SIGTERM received |

---

## Examples

```bash
# Start broker with custom config file
BROKKR_CONFIG_FILE=/etc/brokkr/broker.toml brokkr-broker serve

# Start broker with environment overrides
BROKKR__DATABASE__URL=postgres://user:pass@db:5432/brokkr \
BROKKR__LOG__LEVEL=info \
BROKKR__LOG__FORMAT=json \
  brokkr-broker serve

# Create an agent and capture its PAK
brokkr-broker create agent --name prod-1 --cluster_name us-east-1 2>&1 | grep "Initial PAK"

# Start agent with environment config
BROKKR__AGENT__BROKER_URL=https://broker.example.com \
BROKKR__AGENT__PAK=brokkr_BRx9y2Kq_A1B2C3D4E5F6G7H8I9J0K1L2 \
BROKKR__AGENT__AGENT_NAME=prod-1 \
BROKKR__AGENT__CLUSTER_NAME=us-east-1 \
  brokkr-agent start
```
