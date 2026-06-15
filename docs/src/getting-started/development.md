# Local Development Environment

This guide covers running Brokkr from source on your own machine — for contributors, or for anyone who wants the full broker/agent/k3s stack running locally without a real cluster.

## Prerequisites

- **Rust 1.90 or later** (the workspace uses edition 2024)
- **PostgreSQL** client tooling (the database itself runs in Docker)
- **Docker** with Docker Compose
- **Angreal**, the project's task runner: `pip install angreal`

## Building from Source

```bash
# Clone the repository
git clone https://github.com/colliery-io/brokkr.git
cd brokkr

# Build using Cargo
cargo build --release

# The binaries will be available in target/release/
# - brokkr-broker: The central management service
# - brokkr-agent: The Kubernetes cluster agent
```

## Running Locally (Bare Binaries)

You can run the binaries directly against a PostgreSQL database:

```bash
# Set up database
export BROKKR__DATABASE__URL="postgres://brokkr:brokkr@localhost:5432/brokkr"

# Run broker
./target/release/brokkr-broker serve

# Run agent (in another terminal)
export BROKKR__AGENT__PAK="<your-pak>"
export BROKKR__AGENT__BROKER_URL="http://localhost:3000"
./target/release/brokkr-agent start
```

Both binaries are configured through defaults, an optional `BROKKR_CONFIG_FILE` TOML layer, and `BROKKR__*` environment variables — see the [Configuration Guide](./configuration.md).

## The Full Development Environment

For day-to-day development, the docker-compose environment gives you the whole system in one command:

```bash
# Start the development environment
angreal local up

# Rebuild specific services after code changes
angreal local rebuild broker
angreal local rebuild agent
```

`angreal local up` starts the `brokkr-dev` compose project with:

| Service | Host port | Notes |
|---------|-----------|-------|
| PostgreSQL | `5433` | Mapped from container port 5432 to avoid conflicts |
| Local container registry | `5050` | For Shipwright builds |
| Broker | `3000` | Built from your working tree |
| k3s | `6443` | Kubernetes cluster with Tekton + Shipwright installed |
| Demo admin UI | `3001` | `examples/ui-slim` |
| Webhook catcher | `8090` | `examples/webhook-catcher` |

It also pre-creates an agent named `brokkr-integration-test-agent` (cluster `brokkr-dev-integration-cluster`) and starts an agent container with it. The broker runs with the default configuration, so the publicly known dev admin PAK (Prefixed API Key) `brokkr_BR3rVsDa_GK3QN7CDUzYc6iKgMkJ98M2WSimM5t6U8` works against it.

### Where Keys Land

The environment copies credentials to `/tmp/brokkr-keys/` on your host:

- `agent.pak` — the pre-created agent's PAK
- `kubeconfig.local.yaml` — kubeconfig for reaching the k3s cluster from your host (`https://localhost:6443`)
- `kubeconfig.docker.yaml` — kubeconfig used by containers inside the compose network

(When a broker runs with `broker.pak_hash` unset or empty, it writes a freshly generated admin PAK to `/tmp/brokkr-keys/key.txt` inside its container. The dev environment uses the default hash instead, so use the dev PAK above.)

### Bundled Examples

- **`examples/ui-slim`** — a React admin UI served on port `3001`, pre-wired to the broker with the dev admin PAK. This is why the broker's default CORS configuration allows `http://localhost:3001`.
- **`examples/webhook-catcher`** — a small receiver on port `8090` for exercising [webhooks](../how-to/webhooks.md) end to end.

## Gotchas

- **Don't run two angreal docker suites in parallel.** All angreal suites (local environment, integration, e2e) share the same `brokkr-dev` compose project; running two at once tears each other down.
- **`angreal docs serve` collides with the broker.** mdBook also defaults to port 3000, so the docs server and the broker can't run at the same time without changing one of the ports.
- **Postgres port mismatch.** The compose environment exposes PostgreSQL on host port `5433`, but the `angreal models` tasks connect to `5432` — they expect their own database, not the compose one.

## Troubleshooting Configuration

**Database connection failures** typically indicate incorrect credentials or network issues. Verify the database URL is correct, the database server is running, and network connectivity exists between the broker and database.

```bash
# Test database connectivity (note: the dev compose environment exposes 5433)
psql "postgres://brokkr:brokkr@localhost:5433/brokkr" -c "SELECT 1"
```

**Agent authentication failures** usually result from an invalid PAK. Verify the PAK was copied correctly without extra whitespace and that the agent record hasn't been deleted from the broker.

**Kubernetes access issues** in agents may indicate missing or invalid credentials. When running outside a cluster, ensure `BROKKR__AGENT__KUBECONFIG_PATH` points to a valid kubeconfig file. When running inside a cluster, verify the service account has appropriate RBAC permissions.

### Debugging Configuration

Enable trace-level logging to see configuration loading details:

```bash
BROKKR__LOG__LEVEL=trace brokkr-broker serve
```

The broker logs configuration values at startup (with sensitive values redacted), making it easy to verify which settings were applied.

### Getting Help

If you encounter configuration issues:

1. Check the logs for detailed error messages
2. Verify all required configuration values are set
3. Test connectivity to external dependencies (database, Kubernetes API)
4. Consult the [GitHub Issues](https://github.com/colliery-io/brokkr/issues) for known issues

For more details on contributing, see the [project README](https://github.com/colliery-io/brokkr).
