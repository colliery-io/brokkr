# Configuration Guide

This page covers how Brokkr's configuration system works and the handful of values every installation must set. The complete catalog of every setting, type, and default lives in the [Environment Variables Reference](../reference/environment-variables.md) — this page deliberately does not duplicate it.

## Configuration Sources

The `brokkr-broker` and `brokkr-agent` binaries load configuration from three layers, with later layers taking precedence:

1. **Default values** embedded in the application from `default.toml`
2. **Configuration file** (optional) — set `BROKKR_CONFIG_FILE` to a TOML file path
3. **Environment variables** prefixed with `BROKKR__`

Environment variables always win: defaults work out of the box, a file can carry environment-specific settings, and `BROKKR__*` variables override both. In Kubernetes, the Helm charts take the third route exclusively — they render your chart values into a ConfigMap of `BROKKR__*` variables and inject it into the pod as environment, never as a file. A chart install therefore has no configuration file at all, which has consequences for reloading (see [Hot-Reload Configuration](#hot-reload-configuration)).

## Environment Variable Naming

All variables use the `BROKKR__` prefix with double underscores (`__`) as separators for nested configuration paths, uppercased. For example, the setting `broker.webhook_delivery_interval_seconds` becomes:

```
BROKKR__BROKER__WEBHOOK_DELIVERY_INTERVAL_SECONDS
```

Configuration files use the same paths in TOML form:

```toml
[broker]
webhook_delivery_interval_seconds = 5
```

## What You Must Set

Most defaults are sensible for development; a real installation sets these:

**Broker:**

```bash
# Where PostgreSQL lives (the embedded default points at the dev compose stack on :5433)
BROKKR__DATABASE__URL=postgres://user:password@db.example.com:5432/brokkr

# Admin credential. The embedded default is a PUBLICLY KNOWN development hash —
# set your own hash, or set empty to have a fresh PAK generated at first
# startup and written to /tmp/brokkr-keys/key.txt
BROKKR__BROKER__PAK_HASH=<your-pak-hash>

# Webhook secrets are encrypted at rest under this key. If unset, a random key
# is generated per boot and previously encrypted webhook URLs become unreadable
# after a restart. 64 hex characters (32 bytes).
BROKKR__BROKER__WEBHOOK_ENCRYPTION_KEY=<64-hex-chars>

# The embedded default level is `debug`
BROKKR__LOG__LEVEL=info
```

To mint an admin credential before the broker ever starts, run `brokkr-broker generate-pak`. It produces a PAK and its SHA-256 hash entirely offline — no database, no keyfile — so you can set `BROKKR__BROKER__PAK_HASH` to that hash for a day-zero bootstrap. Leaving the hash unset/empty instead has the broker auto-generate a fresh PAK on first startup and write it to `/tmp/brokkr-keys/key.txt`. See the [CLI Reference](../reference/cli.md) for the command's full output.

**Agent:**

```bash
BROKKR__AGENT__BROKER_URL=https://broker.example.com:3000
BROKKR__AGENT__PAK=<agent-pak-from-registration>
BROKKR__AGENT__AGENT_NAME=production-east
BROKKR__AGENT__CLUSTER_NAME=prod-us-east-1
```

These four agent values have placeholder defaults (`DEFAULT` names, localhost broker) that only suit the dev compose stack; `agent_name` and `cluster_name` must match the agent's broker registration. Everything else — polling intervals, health checking, the WebSocket channel, CORS, telemetry, PAK generation parameters — has working defaults; see the [Environment Variables Reference](../reference/environment-variables.md) for the full catalog.

One optional value belongs to the generator-registration authorization model. To let an agent receive stacks owned by application-scoped generators, register it with those generators:

```bash
# Comma-separated generator UUIDs the agent opts into at startup
BROKKR__AGENT__GENERATOR_IDS=<uuid>,<uuid>
```

The same list can be supplied by the `--generator-ids` CLI flag (highest precedence) or the `agent.generator_ids` config-file key. When unset, the agent runs in system/fleet scope only — it is still auto-registered with the system generator and serves any fleet-wide stacks. Registration is an agent's consent boundary: a stack owned by a generator cannot be targeted at the agent until the agent is registered with that generator. For the operational walkthrough see [Registering an agent with a generator](../how-to/agent-registration.md); for the why, see [the security model](../explanation/security-model.md#generator-registration-and-application-scopes).

## Hot-Reload Configuration

> **Hot reload does not currently change broker behaviour.** Several settings are *designated* hot-reloadable, and the machinery to detect a change exists — but no part of the broker reads the reloaded values back. The log filter, the CORS layer and the background workers each capture their settings once at startup. **Treat every setting as restart-only.** This is tracked as BROKKR-T-0292; the rest of this section describes what the reload machinery does today, which is detection and reporting.

The settings designated hot-reloadable are the log level, CORS origins and preflight max-age, and the diagnostic/webhook background-task tunables. The authoritative list is in the [Environment Variables Reference](../reference/environment-variables.md#configuration-file-and-hot-reload).

Trigger a reload explicitly:

```bash
curl -X POST https://broker.example.com/api/v1/admin/config/reload \
  -H "Authorization: Bearer $ADMIN_PAK"
```

A reload re-reads all three layers and records the change set in the audit log as `config.reloaded`. That audit entry is the whole observable effect: the recomputed values are stored, but nothing consults them afterwards, so the broker keeps behaving as it did before. In a chart install the endpoint additionally has nothing new to find — the process environment is fixed for the pod's lifetime — so it consistently reports no changes.

There is also an automatic path, but it depends on how the broker was deployed:

- **Deployed with a configuration file** — when `BROKKR_CONFIG_FILE` points at an existing file, the broker watches that file and reloads on its own whenever the contents change (5-second debounce, tunable via `BROKKR_CONFIG_WATCHER_DEBOUNCE_SECONDS`; turn it off with `BROKKR_CONFIG_WATCHER_ENABLED=false`).
- **Deployed with the Helm chart** — the watcher never starts. The chart delivers settings as environment variables and sets no `BROKKR_CONFIG_FILE`, so there is no file to watch. Editing chart values and upgrading changes the ConfigMap, but the running process keeps the environment it started with; **restart the broker pods to pick the new values up**. The explicit reload endpoint above still works, though with no file layer to re-read it only reflects what the process can already see.

To get automatic reload under Kubernetes today you would have to depart from the chart: mount a ConfigMap as a file and set `BROKKR_CONFIG_FILE` to its path yourself. Note that a mounted ConfigMap's contents propagate on the kubelet's own sync schedule, so the change is not instantaneous even then. Making the chart-installed broker reload without a restart is an open design question for Brokkr, so expect this area to improve.

## Next Steps

- [Environment Variables Reference](../reference/environment-variables.md) — every setting, type, and default
- [Installation Guide](./installation.md) — chart values that render these variables
- [Local Development Environment](./development.md) — the dev stack's pre-wired configuration and troubleshooting
