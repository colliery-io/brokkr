---
title: "Configuration Guide"
weight: 3
---

# Configuration Guide

Brokkr uses a layered configuration system that allows settings to be defined through default values, configuration files, and environment variables. This guide provides a comprehensive reference for all configuration options and explains how to configure both the broker and agent components.

## Configuration Sources

Configuration values are loaded from multiple sources, with later sources taking precedence over earlier ones:

1. **Default values** embedded in the application from `default.toml`
2. **Configuration file** (optional) specified at startup
3. **Environment variables** prefixed with `BROKKR__`

This layering enables a flexible deployment model where defaults work out of the box, configuration files provide environment-specific settings, and environment variables allow runtime overrides without modifying files.

## Environment Variable Naming

All environment variables use the `BROKKR__` prefix with double underscores (`__`) as separators for nested configuration. The naming convention converts configuration paths to uppercase with underscores.

For example, the configuration path `broker.webhook_delivery_interval_seconds` becomes the environment variable `BROKKR__BROKER__WEBHOOK_DELIVERY_INTERVAL_SECONDS`.

## Database Configuration

The database configuration controls the connection to PostgreSQL.

| Variable | Type | Default | Description |
|----------|------|---------|-------------|
| `BROKKR__DATABASE__URL` | string | `postgres://brokkr:brokkr@localhost:5432/brokkr` | PostgreSQL connection URL |
| `BROKKR__DATABASE__SCHEMA` | string | None | Schema name for multi-tenant isolation |

The schema setting enables multi-tenant deployments where each tenant's data is isolated in a separate PostgreSQL schema. When configured, all queries automatically set `search_path` to the specified schema.

```bash
# Standard single-tenant configuration
BROKKR__DATABASE__URL=postgres://user:password@db.example.com:5432/brokkr

# Multi-tenant configuration with schema isolation
BROKKR__DATABASE__URL=postgres://user:password@db.example.com:5432/brokkr
BROKKR__DATABASE__SCHEMA=tenant_acme
```

## Logging Configuration

Logging settings control the verbosity and format of application logs.

| Variable | Type | Default | Description |
|----------|------|---------|-------------|
| `BROKKR__LOG__LEVEL` | string | `debug` | Log level: trace, debug, info, warn, error |
| `BROKKR__LOG__FORMAT` | string | `text` | Log format: text (human-readable) or json (structured) |

The log level is hot-reloadable—changes take effect without restarting the application. Use `json` format in production environments for easier log aggregation and parsing.

```bash
# Production logging configuration
BROKKR__LOG__LEVEL=info
BROKKR__LOG__FORMAT=json
```

## Broker Configuration

The broker configuration controls the central management service's behavior.

### Core Settings

| Variable | Type | Default | Description |
|----------|------|---------|-------------|
| `BROKKR__BROKER__PAK_HASH` | string | None | Pre-computed PAK hash for admin authentication |

### Webhook Settings

Webhooks deliver event notifications to external systems. These settings control the delivery worker's behavior.

| Variable | Type | Default | Description |
|----------|------|---------|-------------|
| `BROKKR__BROKER__WEBHOOK_ENCRYPTION_KEY` | string | Random | 64-character hex string for AES-256 encryption |
| `BROKKR__BROKER__WEBHOOK_DELIVERY_INTERVAL_SECONDS` | integer | `5` | Polling interval for pending webhook deliveries |
| `BROKKR__BROKER__WEBHOOK_DELIVERY_BATCH_SIZE` | integer | `50` | Maximum deliveries processed per batch |
| `BROKKR__BROKER__WEBHOOK_CLEANUP_RETENTION_DAYS` | integer | `7` | Days to retain completed webhook deliveries |

The encryption key protects webhook URLs and authentication headers at rest. If not configured, the broker generates a random key at startup and logs a warning. This means encrypted data will become unreadable if the broker restarts. For production deployments, always configure an explicit encryption key.

```bash
# Production webhook configuration
BROKKR__BROKER__WEBHOOK_ENCRYPTION_KEY=0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef
BROKKR__BROKER__WEBHOOK_DELIVERY_INTERVAL_SECONDS=5
BROKKR__BROKER__WEBHOOK_DELIVERY_BATCH_SIZE=100
BROKKR__BROKER__WEBHOOK_CLEANUP_RETENTION_DAYS=30
```

### Diagnostic Settings

Diagnostics are temporary operations that agents execute for debugging purposes.

| Variable | Type | Default | Description |
|----------|------|---------|-------------|
| `BROKKR__BROKER__DIAGNOSTIC_CLEANUP_INTERVAL_SECONDS` | integer | `900` | Cleanup task interval (15 minutes) |
| `BROKKR__BROKER__DIAGNOSTIC_MAX_AGE_HOURS` | integer | `1` | Maximum age for diagnostic results |

### Audit Log Settings

Audit logs record all significant actions for security and compliance.

| Variable | Type | Default | Description |
|----------|------|---------|-------------|
| `BROKKR__BROKER__AUDIT_LOG_RETENTION_DAYS` | integer | `90` | Days to retain audit log entries |

```bash
# Extended audit retention for compliance
BROKKR__BROKER__AUDIT_LOG_RETENTION_DAYS=365
```

## CORS Configuration

Cross-Origin Resource Sharing (CORS) settings control which origins can access the broker API.

| Variable | Type | Default | Description |
|----------|------|---------|-------------|
| `BROKKR__CORS__ALLOWED_ORIGINS` | list | `["*"]` | Allowed origins (use `*` for all) |
| `BROKKR__CORS__ALLOWED_METHODS` | list | `["GET", "POST", "PUT", "DELETE"]` | Allowed HTTP methods |
| `BROKKR__CORS__ALLOWED_HEADERS` | list | `["Content-Type", "Authorization"]` | Allowed request headers |
| `BROKKR__CORS__MAX_AGE_SECONDS` | integer | `3600` | Preflight cache duration |

CORS settings are hot-reloadable. In production, restrict `allowed_origins` to specific domains rather than using `*`.

## Agent Configuration

The agent configuration controls the Kubernetes cluster agent's behavior.

### Required Settings

| Variable | Type | Required | Description |
|----------|------|----------|-------------|
| `BROKKR__AGENT__BROKER_URL` | string | Yes | Broker API URL |
| `BROKKR__AGENT__PAK` | string | Yes | Prefixed API Key for broker communication |
| `BROKKR__AGENT__AGENT_NAME` | string | Yes | Human-readable agent name |
| `BROKKR__AGENT__CLUSTER_NAME` | string | Yes | Name of the managed Kubernetes cluster |

### Polling Settings

| Variable | Type | Default | Description |
|----------|------|---------|-------------|
| `BROKKR__AGENT__POLLING_INTERVAL` | integer | `30` | Seconds between broker polls |
| `BROKKR__AGENT__MAX_RETRIES` | integer | `3` | Maximum operation retry attempts |
| `BROKKR__AGENT__MAX_EVENT_MESSAGE_RETRIES` | integer | `3` | Maximum event reporting retry attempts |
| `BROKKR__AGENT__EVENT_MESSAGE_RETRY_DELAY` | integer | `5` | Seconds between event retry attempts |

### Health and Monitoring

| Variable | Type | Default | Description |
|----------|------|---------|-------------|
| `BROKKR__AGENT__HEALTH_PORT` | integer | `8080` | HTTP port for health endpoints |
| `BROKKR__AGENT__DEPLOYMENT_HEALTH_ENABLED` | boolean | `true` | Enable deployment health monitoring |
| `BROKKR__AGENT__DEPLOYMENT_HEALTH_INTERVAL` | integer | `60` | Seconds between health checks |

### Kubernetes Settings

| Variable | Type | Default | Description |
|----------|------|---------|-------------|
| `BROKKR__AGENT__KUBECONFIG_PATH` | string | None | Path to kubeconfig file (uses in-cluster config if not set) |

```bash
# Complete agent configuration
BROKKR__AGENT__BROKER_URL=https://broker.example.com:3000
BROKKR__AGENT__PAK=brokkr_BRabc123_xyzSecretTokenHere
BROKKR__AGENT__AGENT_NAME=production-east
BROKKR__AGENT__CLUSTER_NAME=prod-us-east-1
BROKKR__AGENT__POLLING_INTERVAL=30
BROKKR__AGENT__HEALTH_PORT=8080
BROKKR__AGENT__DEPLOYMENT_HEALTH_ENABLED=true
BROKKR__AGENT__DEPLOYMENT_HEALTH_INTERVAL=60
```

## Telemetry Configuration

Telemetry settings control OpenTelemetry trace export for distributed tracing.

### Base Settings

| Variable | Type | Default | Description |
|----------|------|---------|-------------|
| `BROKKR__TELEMETRY__ENABLED` | boolean | `false` | Enable telemetry export |
| `BROKKR__TELEMETRY__OTLP_ENDPOINT` | string | `http://localhost:4317` | OTLP collector endpoint (gRPC) |
| `BROKKR__TELEMETRY__SERVICE_NAME` | string | `brokkr` | Service name for traces |
| `BROKKR__TELEMETRY__SAMPLING_RATE` | float | `0.1` | Sampling rate (0.0 to 1.0) |

### Component Overrides

The broker and agent can have independent telemetry configurations that override the base settings.

| Variable | Description |
|----------|-------------|
| `BROKKR__TELEMETRY__BROKER__ENABLED` | Override enabled for broker |
| `BROKKR__TELEMETRY__BROKER__OTLP_ENDPOINT` | Override endpoint for broker |
| `BROKKR__TELEMETRY__BROKER__SERVICE_NAME` | Override service name for broker |
| `BROKKR__TELEMETRY__BROKER__SAMPLING_RATE` | Override sampling rate for broker |
| `BROKKR__TELEMETRY__AGENT__ENABLED` | Override enabled for agent |
| `BROKKR__TELEMETRY__AGENT__OTLP_ENDPOINT` | Override endpoint for agent |
| `BROKKR__TELEMETRY__AGENT__SERVICE_NAME` | Override service name for agent |
| `BROKKR__TELEMETRY__AGENT__SAMPLING_RATE` | Override sampling rate for agent |

```bash
# Enable telemetry with different sampling for broker and agent
BROKKR__TELEMETRY__ENABLED=true
BROKKR__TELEMETRY__OTLP_ENDPOINT=http://otel-collector:4317
BROKKR__TELEMETRY__SERVICE_NAME=brokkr
BROKKR__TELEMETRY__SAMPLING_RATE=0.1
BROKKR__TELEMETRY__BROKER__SERVICE_NAME=brokkr-broker
BROKKR__TELEMETRY__BROKER__SAMPLING_RATE=0.5
BROKKR__TELEMETRY__AGENT__SERVICE_NAME=brokkr-agent
BROKKR__TELEMETRY__AGENT__SAMPLING_RATE=0.1
```

## PAK Configuration

Prefixed API Key (PAK) settings control token generation characteristics.

| Variable | Type | Default | Description |
|----------|------|---------|-------------|
| `BROKKR__PAK__PREFIX` | string | `brokkr` | PAK string prefix |
| `BROKKR__PAK__SHORT_TOKEN_PREFIX` | string | `BR` | Short token prefix |
| `BROKKR__PAK__SHORT_TOKEN_LENGTH` | integer | `8` | Short token character count |
| `BROKKR__PAK__LONG_TOKEN_LENGTH` | integer | `24` | Long token character count |
| `BROKKR__PAK__RNG` | string | `osrng` | Random number generator type |
| `BROKKR__PAK__DIGEST` | integer | `8` | Digest algorithm identifier |

These settings are typically left at their defaults. Changing them affects only newly generated PAKs—existing PAKs remain valid.

## Configuration File Format

Configuration files use TOML format. All settings can be specified in a configuration file as an alternative to environment variables.

```toml
[database]
url = "postgres://user:password@localhost:5432/brokkr"
schema = "tenant_acme"

[log]
level = "info"
format = "json"

[broker]
webhook_encryption_key = "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef"
webhook_delivery_interval_seconds = 5
webhook_delivery_batch_size = 50
webhook_cleanup_retention_days = 7
diagnostic_cleanup_interval_seconds = 900
diagnostic_max_age_hours = 1
audit_log_retention_days = 90

[cors]
allowed_origins = ["https://admin.example.com"]
allowed_methods = ["GET", "POST", "PUT", "DELETE"]
allowed_headers = ["Content-Type", "Authorization"]
max_age_seconds = 3600

[agent]
broker_url = "https://broker.example.com:3000"
pak = "brokkr_BRabc123_xyzSecretTokenHere"
agent_name = "production-east"
cluster_name = "prod-us-east-1"
polling_interval = 30
max_retries = 3
health_port = 8080
deployment_health_enabled = true
deployment_health_interval = 60

[telemetry]
enabled = true
otlp_endpoint = "http://otel-collector:4317"
service_name = "brokkr"
sampling_rate = 0.1

[telemetry.broker]
service_name = "brokkr-broker"
sampling_rate = 0.5

[telemetry.agent]
service_name = "brokkr-agent"
sampling_rate = 0.1

[pak]
prefix = "brokkr"
short_token_prefix = "BR"
short_token_length = 8
long_token_length = 24
```

## Hot-Reload Configuration

The broker supports dynamic configuration reloading for certain settings without requiring a restart.

### Hot-Reloadable Settings

These settings can be changed at runtime:

- `log.level` - Log verbosity
- `cors.allowed_origins` - CORS origins
- `cors.max_age_seconds` - CORS preflight cache
- `broker.diagnostic_cleanup_interval_seconds` - Diagnostic cleanup interval
- `broker.diagnostic_max_age_hours` - Diagnostic retention
- `broker.webhook_delivery_interval_seconds` - Webhook delivery interval
- `broker.webhook_delivery_batch_size` - Webhook batch size
- `broker.webhook_cleanup_retention_days` - Webhook retention

### Static Settings (Require Restart)

These settings require an application restart to change:

- `database.url` - Database connection
- `database.schema` - Database schema
- `broker.webhook_encryption_key` - Encryption key
- `broker.pak_hash` - Admin PAK hash
- `telemetry.*` - All telemetry settings
- `pak.*` - All PAK generation settings

### Triggering a Reload

Reload configuration via the admin API:

```bash
curl -X POST https://broker.example.com/api/v1/admin/config/reload \
  -H "Authorization: Bearer $ADMIN_PAK"
```

In Kubernetes deployments, the broker automatically watches its ConfigMap for changes with a 5-second debounce period.

## Troubleshooting

### Common Configuration Issues

**Database connection failures** typically indicate incorrect credentials or network issues. Verify the database URL is correct, the database server is running, and network connectivity exists between the broker and database.

```bash
# Test database connectivity
psql "postgres://user:password@localhost:5432/brokkr" -c "SELECT 1"
```

**Agent authentication failures** usually result from an invalid PAK. Verify the PAK was copied correctly without extra whitespace and that the agent record hasn't been deleted from the broker.

**Kubernetes access issues** in agents may indicate missing or invalid credentials. When running outside a cluster, ensure `BROKKR__AGENT__KUBECONFIG_PATH` points to a valid kubeconfig file. When running inside a cluster, verify the service account has appropriate RBAC permissions.

### Debugging Configuration

Enable trace-level logging to see configuration loading details:

```bash
BROKKR__LOG__LEVEL=trace brokkr-broker
```

The broker logs configuration values at startup (with sensitive values redacted), making it easy to verify which settings were applied.

### Getting Help

If you encounter configuration issues:

1. Check the logs for detailed error messages
2. Verify all required configuration values are set
3. Test connectivity to external dependencies (database, Kubernetes API)
4. Consult the [GitHub Issues](https://github.com/colliery-io/brokkr/issues) for known issues
