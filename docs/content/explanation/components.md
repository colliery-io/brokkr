---
title: "Component Implementation"
weight: 2
---

# Component Implementation Details

This document provides detailed technical implementation information about each component in the Brokkr system. Understanding these implementation details helps operators debug issues, extend functionality, and optimize performance.

## Broker Components

The broker is implemented in Rust using the Axum web framework with Tokio as the async runtime. It uses Diesel ORM with r2d2 connection pooling for PostgreSQL database access.

### API Module

The API module implements the broker's RESTful interface using Axum's router and middleware patterns. Routes are organized hierarchically with authentication applied uniformly through middleware.

#### Route Organization

Routes are defined in submodules and merged into a unified router with authentication middleware applied:

```rust
pub fn routes(dal: DAL, cors_config: &Cors, reloadable_config: Option<ReloadableConfig>) -> Router<DAL> {
    let cors = build_cors_layer(cors_config);
    let api_routes = Router::new()
        .merge(agent_events::routes())
        .merge(agents::routes())
        .merge(stacks::routes())
        .merge(webhooks::routes())
        .merge(work_orders::routes())
        // Additional route modules...
        .layer(from_fn_with_state(dal.clone(), middleware::auth_middleware))
        .layer(cors);

    Router::new().nest("/api/v1", api_routes)
}
```

The authentication middleware intercepts every request, extracts the PAK from the Authorization header, verifies it against the database, and attaches an `AuthPayload` to the request extensions. Handlers can then access the authenticated identity to make authorization decisions.

#### CORS Configuration

CORS is configured dynamically based on settings, supporting three modes: allow all origins when `"*"` is specified, restrict to specific origins otherwise, with configurable methods, headers, and preflight cache duration.

```rust
pub struct Cors {
    pub allowed_origins: Vec<String>,
    pub allowed_methods: Vec<String>,
    pub allowed_headers: Vec<String>,
    pub max_age_seconds: u64,
}
```

#### Debugging

Enable debug logging with environment variables:
- `RUST_LOG=debug` - General debug output
- `RUST_LOG=brokkr_broker=trace` - Detailed broker tracing
- `RUST_LOG=tower_http=debug` - HTTP layer debugging

### DAL (Data Access Layer) Module

The Data Access Layer provides structured access to the PostgreSQL database using Diesel ORM with r2d2 connection pooling. Each entity type has a dedicated accessor class that encapsulates all database operations.

#### Implementation Architecture

The DAL uses Diesel's compile-time query checking and type-safe schema definitions. Connection management uses r2d2's pooling with automatic connection recycling:

```rust
pub struct DAL {
    pool: Pool<ConnectionManager<PgConnection>>,
    schema: Option<String>,
}

impl DAL {
    pub fn agents(&self) -> AgentsAccessor {
        AgentsAccessor::new(self.pool.clone(), self.schema.clone())
    }

    pub fn stacks(&self) -> StacksAccessor {
        StacksAccessor::new(self.pool.clone(), self.schema.clone())
    }
    // Additional accessors...
}
```

Each accessor obtains a connection from the pool, optionally sets the PostgreSQL search path for multi-tenant schema isolation, executes the query, and returns the connection to the pool.

#### Multi-Tenant Schema Support

The DAL supports PostgreSQL schema isolation for multi-tenant deployments. When a schema is configured, every connection sets `search_path` before executing queries:

```rust
if let Some(schema) = &self.schema {
    diesel::sql_query(format!("SET search_path TO {}", schema))
        .execute(&mut conn)?;
}
```

Schema names are validated to prevent SQL injection before use.

#### Error Handling

Database errors are wrapped in a unified `DalError` type:

```rust
pub enum DalError {
    ConnectionPool(r2d2::Error),
    Query(diesel::result::Error),
    NotFound,
}
```

This provides consistent error handling across all database operations while preserving the underlying error details for debugging.

### CLI Module

The CLI module handles command-line argument parsing, configuration loading, and service initialization. It supports running database migrations, starting the broker server, and administrative operations.

Configuration is loaded from environment variables using the `BROKKR__` prefix with double underscore separators for nesting:

```bash
BROKKR__DATABASE__URL=postgres://user:pass@localhost/brokkr
BROKKR__DATABASE__SCHEMA=tenant_a
BROKKR__LOG__LEVEL=info
BROKKR__BROKER__WEBHOOK_DELIVERY_INTERVAL_SECONDS=5
```

### Background Tasks Module

The broker runs several background tasks for maintenance operations:

**Diagnostic Cleanup** runs every 15 minutes (configurable) to remove diagnostic results older than 1 hour (configurable).

**Work Order Maintenance** runs every 10 seconds to process retry scheduling and detect stale claims.

**Webhook Delivery** runs every 5 seconds (configurable) to process pending webhook deliveries in batches of 50 (configurable).

**Webhook Cleanup** runs hourly to remove delivery records older than 7 days (configurable).

**Audit Log Cleanup** runs daily to remove audit entries older than 90 days (configurable).

### Utils Module

The utils module provides shared functionality:

**Event Bus** implements pub/sub for internal event propagation using Tokio mpsc channels with a 1000-entry buffer.

**Audit Logger** provides non-blocking audit logging with batched database writes (100 entries or 1 second flush).

**Encryption** implements AES-256-GCM encryption for webhook secrets with versioned format for algorithm upgrades.

**PAK Controller** generates and verifies Prefixed API Keys using SHA-256 hashing with indexed lookups.

## Agent Components

The agent is implemented in Rust using Tokio for async operations and kube-rs for Kubernetes API interaction. It runs a continuous control loop that polls the broker and reconciles cluster state.

### Broker Communication Module

The broker module handles all communication with the Brokkr Broker service using REST API calls with PAK authentication. The agent polls the broker at configurable intervals rather than maintaining persistent connections.

#### Communication Pattern

All broker communication uses HTTP requests with Bearer token authentication:

```rust
pub async fn fetch_deployment_objects(
    config: &Settings,
    client: &Client,
    agent: &Agent,
) -> Result<Vec<DeploymentObject>, Error> {
    let url = format!(
        "{}/api/v1/agents/{}/target-state",
        config.agent.broker_url, agent.id
    );

    let response = client
        .get(&url)
        .header("Authorization", format!("Bearer {}", config.agent.pak))
        .send()
        .await?;

    response.json().await
}
```

#### Key Endpoints

The agent communicates with these broker endpoints:

| Endpoint | Method | Purpose |
|----------|--------|---------|
| `/api/v1/auth/pak` | POST | Verify PAK and retrieve agent identity |
| `/api/v1/agents/{id}/target-state` | GET | Fetch deployment objects to apply |
| `/api/v1/agents/{id}/events` | POST | Report deployment outcomes |
| `/api/v1/agents/{id}/heartbeat` | POST | Send periodic heartbeat |
| `/api/v1/agents/{id}/health-status` | PATCH | Report deployment health |
| `/api/v1/agents/{id}/work-orders/pending` | GET | Fetch claimable work orders |
| `/api/v1/agents/{id}/diagnostics/pending` | GET | Fetch diagnostic requests |

#### Retry Logic

Failed broker requests use exponential backoff with configurable parameters:

```rust
pub struct Agent {
    pub max_retries: u32,
    pub event_message_retry_delay: u64,
    // ...
}
```

### Kubernetes Module

The Kubernetes module manages all interactions with the Kubernetes API using the kube-rs client library. It implements server-side apply for resource management with intelligent ordering and ownership tracking.

#### Server-Side Apply

Resources are applied using Kubernetes server-side apply, which provides declarative management with conflict detection:

```rust
pub async fn apply_k8s_objects(
    k8s_objects: &[DynamicObject],
    k8s_client: Client,
    patch_params: PatchParams,
) -> Result<(), Error> {
    for obj in k8s_objects {
        let api = get_api_for_object(&k8s_client, obj)?;
        let patch = Patch::Apply(obj);
        api.patch(&obj.name_any(), &patch_params, &patch).await?;
    }
    Ok(())
}
```

#### Resource Ordering

Resources are applied in priority order to respect dependencies:

1. **Namespaces** are applied first as other resources may depend on them
2. **CustomResourceDefinitions** are applied second as custom resources require their definitions
3. **All other resources** are applied after dependencies exist

This ordering prevents failures from missing dependencies during initial deployment.

#### Ownership Tracking

The agent tracks resource ownership using annotations:

```rust
const STACK_ID_ANNOTATION: &str = "brokkr.io/stack-id";
const CHECKSUM_ANNOTATION: &str = "brokkr.io/checksum";
```

Before deleting resources, the agent verifies ownership to prevent removing resources managed by other systems. The checksum annotation enables detection of configuration drift.

#### Reconciliation

Full reconciliation applies the desired state and prunes resources that no longer belong:

```rust
pub async fn reconcile_target_state(
    objects: &[DynamicObject],
    client: Client,
    stack_id: &str,
    checksum: &str,
) -> Result<(), Error> {
    // Apply priority objects first
    apply_priority_objects(&objects, &client).await?;

    // Validate remaining objects
    validate_objects(&objects)?;

    // Apply all resources
    apply_all_objects(&objects, &client).await?;

    // Prune old resources with mismatched checksums
    prune_old_resources(&client, stack_id, checksum).await?;

    Ok(())
}
```

#### Error Handling

Kubernetes operations use retry logic for transient failures:

```rust
// Retryable HTTP status codes
const RETRYABLE_CODES: [u16; 4] = [429, 500, 503, 504];

// Retryable error reasons
const RETRYABLE_REASONS: [&str; 3] = [
    "ServiceUnavailable",
    "InternalError",
    "Timeout",
];
```

Exponential backoff prevents overwhelming a recovering API server.

### Health Module

The health module exposes HTTP endpoints for Kubernetes probes and Prometheus metrics:

| Endpoint | Purpose |
|----------|---------|
| `/healthz` | Liveness probe - returns 200 if process is alive |
| `/readyz` | Readiness probe - returns 200 if agent can serve traffic |
| `/health` | Detailed health status with JSON response |
| `/metrics` | Prometheus metrics in text exposition format |

The health server runs on a configurable port (default: 8080) separately from the main control loop.

### CLI Module

The agent CLI handles configuration loading and service initialization. Configuration uses the same environment variable pattern as the broker:

```bash
BROKKR__AGENT__BROKER_URL=https://broker.example.com:3000
BROKKR__AGENT__PAK=brokkr_BR...
BROKKR__AGENT__AGENT_NAME=production-cluster
BROKKR__AGENT__CLUSTER_NAME=prod-us-east-1
BROKKR__AGENT__POLLING_INTERVAL=30
BROKKR__AGENT__HEALTH_PORT=8080
```

## Configuration Reference

### Broker Configuration

```rust
pub struct Settings {
    pub database: Database,
    pub log: Log,
    pub broker: Broker,
    pub cors: Cors,
    pub telemetry: Telemetry,
}

pub struct Database {
    pub url: String,
    pub schema: Option<String>,
}

pub struct Broker {
    pub diagnostic_cleanup_interval_seconds: Option<u64>,  // default: 900
    pub diagnostic_max_age_hours: Option<i64>,              // default: 1
    pub webhook_encryption_key: Option<String>,
    pub webhook_delivery_interval_seconds: Option<u64>,     // default: 5
    pub webhook_delivery_batch_size: Option<i64>,           // default: 50
    pub webhook_cleanup_retention_days: Option<i64>,        // default: 7
    pub audit_log_retention_days: Option<i64>,              // default: 90
}

pub struct Cors {
    pub allowed_origins: Vec<String>,
    pub allowed_methods: Vec<String>,
    pub allowed_headers: Vec<String>,
    pub max_age_seconds: u64,
}
```

### Agent Configuration

```rust
pub struct Settings {
    pub agent: Agent,
    pub log: Log,
    pub telemetry: Telemetry,
}

pub struct Agent {
    pub broker_url: String,
    pub pak: String,
    pub agent_name: String,
    pub cluster_name: String,
    pub polling_interval: u64,                    // default: 30
    pub kubeconfig_path: Option<String>,
    pub max_retries: u32,
    pub max_event_message_retries: usize,
    pub event_message_retry_delay: u64,
    pub health_port: Option<u16>,                 // default: 8080
    pub deployment_health_enabled: Option<bool>,  // default: true
    pub deployment_health_interval: Option<u64>,  // default: 60
}
```

### Hot-Reload Configuration

The broker supports dynamic configuration reloading for certain settings:

**Hot-reloadable** (apply without restart):
- Log level
- CORS settings (origins, methods, headers, max-age)
- Webhook delivery interval and batch size
- Diagnostic cleanup settings

**Static** (require restart):
- Database URL and schema
- Webhook encryption key
- PAK configuration
- Telemetry settings

Trigger a manual reload via the admin API:

```bash
curl -X POST https://broker/api/v1/admin/config/reload \
  -H "Authorization: Bearer <admin-pak>"
```

In Kubernetes, the broker automatically watches its ConfigMap for changes with a 5-second debounce.

## Component Lifecycle

### Broker Startup Sequence

1. **Configuration Loading** - Parse environment variables and configuration files
2. **Database Connection** - Establish r2d2 connection pool to PostgreSQL
3. **Migration Check** - Verify database schema is current
4. **Encryption Initialization** - Load or generate webhook encryption key
5. **Event Bus Initialization** - Start event dispatcher with mpsc channel
6. **Audit Logger Initialization** - Start background writer with batching
7. **Background Tasks** - Spawn diagnostic cleanup, work order, webhook, and audit tasks
8. **API Server** - Bind to configured port and start accepting requests

### Agent Startup Sequence

1. **Configuration Loading** - Parse environment variables
2. **PAK Verification** - Authenticate with broker and retrieve agent identity
3. **Kubernetes Client** - Initialize kube-rs client with in-cluster or kubeconfig credentials
4. **Health Server** - Start HTTP server for probes and metrics
5. **Control Loop** - Enter main loop with polling, health checks, and work order processing

### Graceful Shutdown

Both components handle SIGTERM and SIGINT for graceful shutdown:

1. Stop accepting new requests
2. Complete in-flight operations
3. Flush pending data (audit logs, events)
4. Close database connections
5. Exit cleanly

## Performance Considerations

### Broker Optimization

**Connection Pooling** - r2d2 maintains a pool of database connections, avoiding connection establishment overhead for each request.

**Batched Writes** - Audit logs and webhook deliveries are batched to reduce database round trips.

**Indexed Lookups** - PAK verification uses indexed columns for O(1) authentication performance.

**Async I/O** - Tokio provides non-blocking I/O for high concurrency without thread-per-request overhead.

### Agent Optimization

**Incremental Fetching** - Agents track processed sequence IDs to fetch only new deployment objects.

**Parallel Apply** - Independent resources can be applied concurrently within priority groups.

**Connection Reuse** - HTTP client maintains connection pools to the broker and Kubernetes API.

**Efficient Diffing** - Checksum-based change detection avoids unnecessary applies for unchanged resources.
