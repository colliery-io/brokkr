# Environment Variables Reference

Complete listing of all environment variables supported by Brokkr. All variables use the `BROKKR__` prefix with double underscores (`__`) as nested separators.

Configuration precedence (highest wins): Environment variables > Config file > Embedded defaults.

## Database

| Variable | Type | Default | Description |
|----------|------|---------|-------------|
| `BROKKR__DATABASE__URL` | String | `postgres://brokkr:brokkr@localhost:5432/brokkr` | PostgreSQL connection URL |
| `BROKKR__DATABASE__SCHEMA` | String | *(none)* | Schema name for multi-tenant isolation. When set, all queries use this schema. |

## Logging

| Variable | Type | Default | Description |
|----------|------|---------|-------------|
| `BROKKR__LOG__LEVEL` | String | `debug` | Log level: `trace`, `debug`, `info`, `warn`, `error` |
| `BROKKR__LOG__FORMAT` | String | `text` | Log format: `text` (human-readable) or `json` (structured) |

The log level is **hot-reloadable** — changes take effect without restarting.

## Broker

| Variable | Type | Default | Description |
|----------|------|---------|-------------|
| `BROKKR__BROKER__PAK_HASH` | String | *(generated)* | Admin PAK hash (set during first startup) |
| `BROKKR__BROKER__DIAGNOSTIC_CLEANUP_INTERVAL_SECONDS` | Integer | `900` | Interval for diagnostic cleanup task (seconds) |
| `BROKKR__BROKER__DIAGNOSTIC_MAX_AGE_HOURS` | Integer | `1` | Max age for completed diagnostics before deletion (hours) |
| `BROKKR__BROKER__WEBHOOK_ENCRYPTION_KEY` | String | *(random)* | Hex-encoded 32-byte AES-256 key for encrypting webhook URLs and auth headers. If empty, a random key is generated on startup (not recommended for production — webhooks won't decrypt after restart). |
| `BROKKR__BROKER__WEBHOOK_DELIVERY_INTERVAL_SECONDS` | Integer | `5` | Webhook delivery worker poll interval (seconds) |
| `BROKKR__BROKER__WEBHOOK_DELIVERY_BATCH_SIZE` | Integer | `50` | Max webhook deliveries processed per batch |
| `BROKKR__BROKER__WEBHOOK_CLEANUP_RETENTION_DAYS` | Integer | `7` | How long to keep completed/dead webhook deliveries (days) |
| `BROKKR__BROKER__AUDIT_LOG_RETENTION_DAYS` | Integer | `90` | How long to keep audit log entries (days) |
| `BROKKR__BROKER__AUTH_CACHE_TTL_SECONDS` | Integer | `60` | TTL for PAK authentication cache (seconds). Set to `0` to disable caching. |

## Agent

| Variable | Type | Default | Description |
|----------|------|---------|-------------|
| `BROKKR__AGENT__BROKER_URL` | String | `http://localhost:3000` | Broker API base URL |
| `BROKKR__AGENT__POLLING_INTERVAL` | Integer | `10` | How often to poll broker for updates (seconds) |
| `BROKKR__AGENT__KUBECONFIG_PATH` | String | *(in-cluster)* | Path to kubeconfig file. If unset, uses in-cluster configuration. |
| `BROKKR__AGENT__MAX_RETRIES` | Integer | `60` | Max retries when waiting for broker on startup |
| `BROKKR__AGENT__PAK` | String | *(required)* | Agent's PAK for broker authentication |
| `BROKKR__AGENT__AGENT_NAME` | String | `DEFAULT` | Agent name (must match broker registration) |
| `BROKKR__AGENT__CLUSTER_NAME` | String | `DEFAULT` | Cluster name (must match broker registration) |
| `BROKKR__AGENT__MAX_EVENT_MESSAGE_RETRIES` | Integer | `2` | Max retries for event message delivery |
| `BROKKR__AGENT__EVENT_MESSAGE_RETRY_DELAY` | Integer | `5` | Delay between event message retries (seconds) |
| `BROKKR__AGENT__HEALTH_PORT` | Integer | `8080` | Port for agent health check HTTP server |
| `BROKKR__AGENT__DEPLOYMENT_HEALTH_ENABLED` | Boolean | `true` | Enable deployment health checking |
| `BROKKR__AGENT__DEPLOYMENT_HEALTH_INTERVAL` | Integer | `60` | Interval for deployment health checks (seconds) |

## PAK (Pre-Authentication Key) Generation

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
| `BROKKR_CONFIG_FILE` | String | *(none)* | Path to TOML configuration file |
| `BROKKR_CONFIG_WATCHER_ENABLED` | Boolean | *(auto)* | Enable/disable ConfigMap hot-reload watcher |
| `BROKKR_CONFIG_WATCHER_DEBOUNCE_SECONDS` | Integer | `5` | Debounce window for config file changes |

### Hot-Reloadable Settings

The following settings can be changed at runtime without restarting the broker (via config file change or admin API):

- `log.level`
- `broker.diagnostic_cleanup_interval_seconds`
- `broker.diagnostic_max_age_hours`
- `broker.webhook_delivery_interval_seconds`
- `broker.webhook_delivery_batch_size`
- `broker.webhook_cleanup_retention_days`
- `cors.allowed_origins`
- `cors.max_age_seconds`

## Related Documentation

- [Configuration Guide](../getting-started/configuration.md) — configuration system overview
- [CLI Reference](./cli.md) — command-line usage
- [Multi-Tenancy](./multi-tenancy.md) — schema-based isolation
- [Monitoring & Observability](./monitoring.md) — telemetry and metrics setup
