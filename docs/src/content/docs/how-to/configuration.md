---
title: Configuration Guide
---

This page covers how Brokkr's configuration system works and the handful of values every installation must set. The complete catalog of every setting, type, and default lives in the [Environment Variables Reference](../../reference/environment-variables/) — this page deliberately does not duplicate it.

## Configuration Sources

The `brokkr-broker` and `brokkr-agent` binaries load configuration from three layers, with later layers taking precedence:

1. **Default values** embedded in the application from `default.toml`
2. **Configuration file** (optional) — set `BROKKR_CONFIG_FILE` to a TOML file path
3. **Environment variables** prefixed with `BROKKR__`

Environment variables always win: defaults work out of the box, a file can carry environment-specific settings, and `BROKKR__*` variables override both. In Kubernetes, the Helm charts render most chart values into a ConfigMap of `BROKKR__*` variables and inject it into the pod as environment. The broker chart also mounts a small TOML file for the settings that reload without a restart (see [Hot-Reload Configuration](#hot-reload-configuration)). The agent chart mounts no file.

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

# Admin credential. If you do not set it, the broker uses the embedded
# default, which is a PUBLICLY KNOWN development hash. Set your own hash.
# Only an explicit empty string (BROKKR__BROKER__PAK_HASH="") makes the broker
# generate a fresh PAK at first startup and write it to /tmp/brokkr-keys/key.txt.
BROKKR__BROKER__PAK_HASH=<your-pak-hash>

# Webhook secrets are encrypted at rest under this key. If unset, a random key
# is generated per boot and previously encrypted webhook URLs become unreadable
# after a restart. 64 hex characters (32 bytes).
BROKKR__BROKER__WEBHOOK_ENCRYPTION_KEY=<64-hex-chars>

# The embedded default level is `debug`
BROKKR__LOG__LEVEL=info
```

To mint an admin credential before the broker ever starts, run `brokkr-broker generate-pak`. It produces a PAK and its SHA-256 hash entirely offline — no database, no keyfile — so you can set `BROKKR__BROKER__PAK_HASH` to that hash for a day-zero bootstrap. If you do not set the hash, the broker uses the hash in its embedded default configuration. The matching PAK is public. Only an explicit empty string, `BROKKR__BROKER__PAK_HASH=""`, makes the broker generate a fresh PAK on first startup and write it to `/tmp/brokkr-keys/key.txt`. See the [CLI Reference](../../reference/cli/) for the command's full output.

**Agent:**

```bash
BROKKR__AGENT__BROKER_URL=https://broker.example.com:3000
BROKKR__AGENT__PAK=<agent-pak-from-registration>
BROKKR__AGENT__AGENT_NAME=production-east
BROKKR__AGENT__CLUSTER_NAME=prod-us-east-1
```

These four agent values have placeholder defaults (`DEFAULT` names, localhost broker) that only suit the dev compose stack; `agent_name` and `cluster_name` must match the agent's broker registration. Everything else — polling intervals, health checking, the WebSocket channel, CORS, telemetry, PAK generation parameters — has working defaults; see the [Environment Variables Reference](../../reference/environment-variables/) for the full catalog.

One optional value belongs to the generator-registration authorization model. To let an agent receive stacks owned by application-scoped generators, register it with those generators:

```bash
# Comma-separated generator UUIDs the agent opts into at startup
BROKKR__AGENT__GENERATOR_IDS=<uuid>,<uuid>
```

The same list can be supplied by the `--generator-ids` CLI flag (highest precedence) or the `agent.generator_ids` config-file key. When unset, the agent runs in system/fleet scope only — it is still auto-registered with the system generator and serves any fleet-wide stacks. Registration is an agent's consent boundary: a stack owned by a generator cannot be targeted at the agent until the agent is registered with that generator. For the operational walkthrough see [Registering an agent with a generator](../agent-registration/); for the why, see [the security model](../../explanation/security-model/#generator-registration-and-application-scopes).

## Hot-Reload Configuration

The broker can apply some settings without a restart. The agent cannot: it reads its configuration once, at startup.

### Settings that reload

| Setting | Chart value | Effect of a reload |
|---------|-------------|--------------------|
| `log.level` | `broker.logLevel` | The log filter changes. |
| `broker.diagnostic_cleanup_interval_seconds` | `broker.diagnosticCleanupIntervalSeconds` | The cleanup task uses the new interval. |
| `broker.diagnostic_max_age_hours` | `broker.diagnosticMaxAgeHours` | The next cleanup pass uses the new age. |
| `broker.webhook_delivery_interval_seconds` | `broker.webhookDeliveryIntervalSeconds` | The delivery worker uses the new interval. |
| `broker.webhook_delivery_batch_size` | `broker.webhookDeliveryBatchSize` | The next delivery pass uses the new batch size. |
| `broker.webhook_cleanup_retention_days` | `broker.webhookCleanupRetentionDays` | The next webhook cleanup pass uses the new retention. |

The background tasks read these values again on each pass. When an interval changes, the task starts a new timer with the new interval.

The `log.level` reload has one limit. If `RUST_LOG` is set when the broker starts, `RUST_LOG` sets the log filter, and a reload does not change it. The broker prints a message that tells you this. To make `log.level` reload, do not set `RUST_LOG` (in a chart install, do not put it in `extraEnv`).

### Settings that need a restart

All other settings need a broker restart. These include:

- `database.*` and the database URL
- `cors.*`: the broker builds the CORS layer one time, at startup. A reload reports a change to `cors.allowed_origins` or `cors.max_age_seconds`, but the broker does not apply it.
- `broker.webhook_encryption_key` and `broker.pak_hash`
- `telemetry.*` and `log.format`
- The other background task settings, for example `broker.audit_log_retention_days`

### How a reload starts

A reload starts in one of two ways:

- **The config-file watcher.** When `BROKKR_CONFIG_FILE` names a file that exists at startup, the broker watches that file. When the file changes, the broker waits for the debounce time (default 5 seconds, set with `BROKKR_CONFIG_WATCHER_DEBOUNCE_SECONDS`), then reloads. The log shows `Config file watcher triggered configuration reload`. Set `BROKKR_CONFIG_WATCHER_ENABLED=false` to stop the watcher.
- **The admin endpoint.** Send this request with an admin PAK (Prefixed API Key):

```bash
curl -X POST https://broker.example.com/api/v1/admin/config/reload \
  -H "Authorization: Bearer $ADMIN_PAK"
```

A reload reads the defaults, the file and the environment again. It applies the settings in the table above and records the changes in the audit log as `config.reloaded`. The process environment does not change while the broker runs. Thus a reload finds changes only in the file. A key that is also set as a `BROKKR__*` variable keeps the value of the variable, because environment variables override the file.

### Hot reload in a chart install

With `configReload.enabled: true` (the default), the broker chart does these steps:

1. It renders the settings in the table above as `brokkr.toml` in a second ConfigMap, `<release>-brokkr-broker-dynamic`. These settings are not in the environment ConfigMap, because an environment variable overrides the file.
2. It mounts that ConfigMap at `/etc/brokkr` as a directory and sets `BROKKR_CONFIG_FILE=/etc/brokkr/brokkr.toml`.

To change a setting, run `helm upgrade` with the new value, or edit the `<release>-brokkr-broker-dynamic` ConfigMap. The kubelet updates the mounted file, usually in about one minute. Then the watcher reloads the broker. The broker does not call the Kubernetes API for this, so the chart creates no Role or RoleBinding.

With `configReload.enabled: false`, the chart mounts no file and sets no `BROKKR_CONFIG_FILE`. The settings go into the environment ConfigMap, and each one needs a restart.

`helm upgrade` does not restart the broker pod: the pod template has no checksum annotation. After you change a setting that needs a restart, run:

```bash
kubectl rollout restart deploy/<release>-brokkr-broker
```

For the chart side, see the "Configuration hot reload" section of the broker chart README (`charts/brokkr-broker/README.md`).

## Next Steps

- [Environment Variables Reference](../../reference/environment-variables/) — every setting, type, and default
- [Installation Guide](../installation/) — chart values that render these variables
- [Local Development Environment](../development/) — the dev stack's pre-wired configuration and troubleshooting
