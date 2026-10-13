---
title: Environment Variables Reference
---

Complete listing of all environment variables supported by Brokkr. All variables use the `BROKKR__` prefix with double underscores (`__`) as nested separators.

Configuration precedence (highest wins): Environment variables > Config file > Embedded defaults.

## Database

| Variable | Type | Default | Description |
|----------|------|---------|-------------|
| `BROKKR__DATABASE__URL` | String | `postgres://brokkr:brokkr@localhost:5433/brokkr` | PostgreSQL connection URL |
| `BROKKR__DATABASE__SCHEMA` | String | *(none)* | Schema name for multi-tenant isolation. When set, all queries use this schema. |

## Logging

| Variable | Type | Default | Description |
|----------|------|---------|-------------|
| `BROKKR__LOG__LEVEL` | String | `debug` | Log level: `trace`, `debug`, `info`, `warn`, `error` |
| `BROKKR__LOG__FORMAT` | String | `text` | Log format: `text` (human-readable) or `json` (structured) |

The broker reads `BROKKR__LOG__FORMAT` once, at startup. A configuration reload can change the broker's `log.level`, but not when `RUST_LOG` was set at startup. The agent reads both once, at startup. See [Configuration File and Hot-Reload](#configuration-file-and-hot-reload).

## Broker

| Variable | Type | Default | Description |
|----------|------|---------|-------------|
| `BROKKR__BROKER__PAK_HASH` | String | *(embedded development hash)* | Hash of the admin PAK. The compiled-in default is a publicly known development hash from `default.toml`, not a generated value. |
| `BROKKR__BROKER__DIAGNOSTIC_CLEANUP_INTERVAL_SECONDS` | Integer | `900` | Interval for diagnostic cleanup task (seconds) |
| `BROKKR__BROKER__DIAGNOSTIC_MAX_AGE_HOURS` | Integer | `1` | Max age for completed diagnostics before deletion (hours) |
| `BROKKR__BROKER__WEBHOOK_ENCRYPTION_KEY` | String | *(random)* | Hex-encoded 32-byte AES-256 key for encrypting webhook URLs and auth headers. If empty, a random key is generated on startup (not recommended for production — webhooks won't decrypt after restart). |
| `BROKKR__BROKER__WEBHOOK_DELIVERY_INTERVAL_SECONDS` | Integer | `5` | Webhook delivery worker poll interval (seconds) |
| `BROKKR__BROKER__WEBHOOK_DELIVERY_BATCH_SIZE` | Integer | `50` | Max webhook deliveries processed per batch |
| `BROKKR__BROKER__WEBHOOK_CLEANUP_RETENTION_DAYS` | Integer | `7` | How long to keep completed/dead webhook deliveries (days) |
| `BROKKR__BROKER__AUDIT_LOG_RETENTION_DAYS` | Integer | `90` | How long to keep audit log entries (days) |
| `BROKKR__BROKER__AUTH_CACHE_TTL_SECONDS` | Integer | `60` | TTL for PAK authentication cache (seconds). Set to `0` to disable caching. |
| `BROKKR__BROKER__AGENT_EVENTS_RETENTION_DAYS` | Integer | `30` | How long to keep agent events before hard-deletion (days). Set to `0` (or leave unset) to disable eviction and retain all agent events indefinitely. |

A fresh admin PAK is generated (and written to `/tmp/brokkr-keys/key.txt`) only when `BROKKR__BROKER__PAK_HASH` is explicitly set to an empty value; when left at the default, the embedded development hash — and its corresponding publicly known development PAK — remain in effect. The admin credential is written on first startup against an empty database and by an explicit `brokkr-broker rotate admin`; a later restart with a different value does not change it, and the broker reports the still-in-use default through the `brokkr_default_admin_pak_hash_in_use` metric (see [Monitoring](../monitoring/)).

## Agent

| Variable | Type | Default | Description |
|----------|------|---------|-------------|
| `BROKKR__AGENT__BROKER_URL` | String | `http://localhost:3000` | Broker API base URL |
| `BROKKR__AGENT__POLLING_INTERVAL` | Integer | `10` | How often to poll broker for updates (seconds) |
| `BROKKR__AGENT__KUBECONFIG_PATH` | String | `/home/${USER}/.kube/config` (literal, not shell-expanded) | Exported as `KUBECONFIG` before client creation; if the file cannot be loaded, kube's config inference falls back to in-cluster configuration |
| `BROKKR__AGENT__GENERATOR_IDS` | String (comma-separated) | *(none)* | Generator UUIDs to self-register with on startup. Resolution precedence (highest wins): `--generator-ids` CLI flag > `BROKKR__AGENT__GENERATOR_IDS` > `agent.generator_ids` (config file) > legacy `BROKKR_GENERATOR_IDS`. When unset or empty, the agent serves system/fleet scope only. See [Agent Registration](../../how-to/agent-registration/). |
| `BROKKR_GENERATOR_IDS` | String (comma-separated) | *(deprecated)* | **Deprecated** legacy form of `BROKKR__AGENT__GENERATOR_IDS`, outside the `BROKKR__` namespace. Still honored as a fallback when `BROKKR__AGENT__GENERATOR_IDS` is unset, but logs a deprecation warning on startup. Migrate to `BROKKR__AGENT__GENERATOR_IDS` or `agent.generator_ids` in the config file. |
| `BROKKR__AGENT__MAX_RETRIES` | Integer | `60` | Max retries when waiting for broker on startup |
| `BROKKR__AGENT__PAK` | String | *(embedded development PAK)* | Agent's PAK for broker authentication. Like `BROKKR__BROKER__PAK_HASH`, the compiled-in default is a publicly known development credential, so an unset value does not fail — the agent silently authenticates with it. Always set this explicitly outside local development. |
| `BROKKR__AGENT__AGENT_NAME` | String | `DEFAULT` | Agent name (must match broker registration) |
| `BROKKR__AGENT__CLUSTER_NAME` | String | `DEFAULT` | Cluster name (must match broker registration) |
| `BROKKR__AGENT__HEALTH_PORT` | Integer | `8080` | Port for agent health check HTTP server |
| `BROKKR__AGENT__DEPLOYMENT_HEALTH_ENABLED` | Boolean | `true` | Enable deployment health checking |
| `BROKKR__AGENT__DEPLOYMENT_HEALTH_INTERVAL` | Integer | `60` | Interval for deployment health checks (seconds) |
| `BROKKR__AGENT__WS_FORCE_REST` | Boolean | `false` | Disable the internal WebSocket channel; use REST polling only |
| `BROKKR__AGENT__WS_URL` | String | *(derived)* | Override the WebSocket URL; defaults to `broker_url` with `http`→`ws`/`https`→`wss` and `/internal/ws/agent` appended |
| `BROKKR__AGENT__KUBE_EVENT_UID_CACHE_CAP` | Integer | `10000` | LRU capacity for the kube-event UID→stack ownership cache |
| `BROKKR__AGENT__WATCH_NAMESPACE` | String | *(none — cluster-wide)* | Restrict pod-log/kube-event watchers and health discovery to one namespace (namespace-scoped RBAC). The agent Helm chart sets this from the downward API when `rbac.clusterWide: false` |

On startup the agent self-registers with each generator UUID resolved from `BROKKR__AGENT__GENERATOR_IDS` (or its higher-precedence equivalents). Malformed UUIDs are skipped with a warning, and registration failures do not block startup. Every agent is additionally auto-registered with the system generator (internal fleet scope) by the broker at agent creation, independent of this configuration. For operational steps and conceptual background, see [Agent Registration](../../how-to/agent-registration/) and the [security model](../../explanation/security-model/#generator-registration-and-application-scopes).

## PAK (Prefixed API Key) Generation

| Variable | Type | Default | Description |
|----------|------|---------|-------------|
| `BROKKR__PAK__PREFIX` | String | `brokkr` | Prefix for generated PAKs |
| `BROKKR__PAK__RNG` | String | `osrng` | Random number generator type |
| `BROKKR__PAK__DIGEST` | Integer | `8` | Digest algorithm identifier |
| `BROKKR__PAK__SHORT_TOKEN_LENGTH` | Integer | `8` | Length of the short token portion |
| `BROKKR__PAK__LONG_TOKEN_LENGTH` | Integer | `24` | Length of the long token portion |
| `BROKKR__PAK__SHORT_TOKEN_PREFIX` | String | `BR` | Prefix for the short token |

Generated PAK format: `{prefix}_{short_token_prefix}{short_token}_{long_token}`

Example: `brokkr_BR3rVsDa_GK3QN7CDUzYc6iKgMkJ98M2WSimM5t6U8`

## CORS

| Variable | Type | Default | Description |
|----------|------|---------|-------------|
| `BROKKR__CORS__ALLOWED_ORIGINS` | String (comma-separated) | `http://localhost:3001` | Allowed CORS origins. Use `*` to allow all (not recommended for production). |
| `BROKKR__CORS__ALLOWED_METHODS` | String (comma-separated) | `GET,POST,PUT,DELETE,OPTIONS` | Allowed HTTP methods |
| `BROKKR__CORS__ALLOWED_HEADERS` | String (comma-separated) | `Authorization,Content-Type` | Allowed request headers |
| `BROKKR__CORS__MAX_AGE_SECONDS` | Integer | `3600` | Preflight response cache duration (seconds) |

> **Note:** Array-type CORS settings accept comma-separated strings when set via environment variables (e.g., `BROKKR__CORS__ALLOWED_ORIGINS=http://a.com,http://b.com`).

## Telemetry (OpenTelemetry)

### Base Settings

| Variable | Type | Default | Description |
|----------|------|---------|-------------|
| `BROKKR__TELEMETRY__ENABLED` | Boolean | `false` | Enable OpenTelemetry tracing |
| `BROKKR__TELEMETRY__OTLP_ENDPOINT` | String | `http://localhost:4317` | OTLP gRPC endpoint for trace export |
| `BROKKR__TELEMETRY__SERVICE_NAME` | String | `brokkr` | Service name for traces |
| `BROKKR__TELEMETRY__SAMPLING_RATE` | Float | `0.1` | Sampling rate (0.0 to 1.0, where 1.0 = 100%) |

### Broker-Specific Overrides

These override the base telemetry settings for the broker component only. If unset, the base value is used.

| Variable | Type | Default | Description |
|----------|------|---------|-------------|
| `BROKKR__TELEMETRY__BROKER__ENABLED` | Boolean | *(inherits)* | Override enabled for broker |
| `BROKKR__TELEMETRY__BROKER__OTLP_ENDPOINT` | String | *(inherits)* | Override OTLP endpoint for broker |
| `BROKKR__TELEMETRY__BROKER__SERVICE_NAME` | String | `brokkr-broker` | Override service name for broker |
| `BROKKR__TELEMETRY__BROKER__SAMPLING_RATE` | Float | *(inherits)* | Override sampling rate for broker |

### Agent-Specific Overrides

| Variable | Type | Default | Description |
|----------|------|---------|-------------|
| `BROKKR__TELEMETRY__AGENT__ENABLED` | Boolean | *(inherits)* | Override enabled for agent |
| `BROKKR__TELEMETRY__AGENT__OTLP_ENDPOINT` | String | *(inherits)* | Override OTLP endpoint for agent |
| `BROKKR__TELEMETRY__AGENT__SERVICE_NAME` | String | `brokkr-agent` | Override service name for agent |
| `BROKKR__TELEMETRY__AGENT__SAMPLING_RATE` | Float | *(inherits)* | Override sampling rate for agent |

## Configuration File and Hot-Reload

These environment variables control the configuration system itself and are **not** part of the `BROKKR__` namespace:

| Variable | Type | Default | Description |
|----------|------|---------|-------------|
| `BROKKR_CONFIG_FILE` | String | *(none)* | Path to a TOML configuration file, loaded between embedded defaults and `BROKKR__*` variables; also arms the broker's config-file watcher |
| `BROKKR_CONFIG_WATCHER_ENABLED` | Boolean | `true` | Disable the config-file watcher by setting it to `false` or `0`. Has no effect unless `BROKKR_CONFIG_FILE` is set to an existing file |
| `BROKKR_CONFIG_WATCHER_DEBOUNCE_SECONDS` | Integer | `5` | Debounce window for config file changes |

The watcher is a **config-file** watcher, not a ConfigMap watcher. It arms only when `BROKKR_CONFIG_FILE` names a file that exists at startup; if the variable is unset or points at a missing path, the watcher is silently disabled regardless of `BROKKR_CONFIG_WATCHER_ENABLED`.

The broker chart uses this file. With `configReload.enabled: true` (the default), it mounts the `<release>-brokkr-broker-dynamic` ConfigMap at `/etc/brokkr` and sets `BROKKR_CONFIG_FILE=/etc/brokkr/brokkr.toml`. That file holds only the settings that reload. The chart sets `BROKKR_CONFIG_WATCHER_ENABLED` and `BROKKR_CONFIG_WATCHER_DEBOUNCE_SECONDS` from `configReload.*`. With `configReload.enabled: false`, the chart mounts no file. The agent has no watcher.

### What a Reload Does

A config-file change (when the watcher is armed) or `POST /api/v1/admin/config/reload` starts a reload. The broker reads the configuration sources again, compares them with the current values, and reports which keys changed, in the API response and in the broker log. It records the change set in the audit log as `config.reloaded`. The broker applies these keys:

| Key | Applied by |
|-----|------------|
| `log.level` | The log filter. Not applied when `RUST_LOG` was set at startup. |
| `broker.diagnostic_cleanup_interval_seconds` | The diagnostic cleanup task, on its next pass |
| `broker.diagnostic_max_age_hours` | The diagnostic cleanup task, on its next pass |
| `broker.webhook_delivery_interval_seconds` | The webhook delivery worker, on its next pass |
| `broker.webhook_delivery_batch_size` | The webhook delivery worker, on its next pass |
| `broker.webhook_cleanup_retention_days` | The webhook cleanup task, on its next pass |

A reload also reports changes to `cors.allowed_origins` and `cors.max_age_seconds`, but it does not apply them: the broker builds the CORS layer one time, at startup. All other settings need a restart.

The process environment does not change while the broker runs. Thus a reload finds changes only in the configuration file. A key that is also set as a `BROKKR__*` variable keeps the value of the variable.

## Related Documentation

- [Configuration Guide](../../how-to/configuration/) — configuration system overview
- [CLI Reference](../cli/) — command-line usage
- [Multi-Tenancy](../multi-tenancy/) — schema-based isolation
- [Monitoring & Observability](../monitoring/) — telemetry and metrics setup
