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

#### Target Authorization and Registration Gates

Authentication establishes *who* is calling; a second layer establishes *which generators' stacks may be targeted at a given agent*. Generator registration is the agent's opt-in consent boundary: an agent must be registered with a generator (an application scope) before any stack owned by that generator can be explicitly targeted at it. The conceptual model lives in [the security model](./security-model.md#generator-registration-and-application-scopes).

The enforcement point is `authorize_target_mutation` in the agents handler. It gates *both* directions of an explicit target — adding one (`POST /agents/{id}/targets`) and removing one (`DELETE /agents/{id}/targets/{stack_id}`). If the calling context is not registered with the stack's owning generator, the broker rejects the mutation with HTTP 403 and the error code `agent_not_registered`. This gate is absolute: there is no admin override or force flag, so even an admin PAK cannot create a target that crosses an agent's registration boundary.

Registration gates only the *creation* of explicit targets. The read path (`GET /agents/{id}/target-state`) is unchanged: the served-stack set remains the union of explicit `agent_targets`, label matches, and annotation matches. Existing targets stay valid (a migration back-fills registrations from any `agent_targets` that predate this model), so registration controls what can be wired up, not what is read back at reconcile time.

This application-level isolation is complementary to — and separate from — the deployment-level [schema-per-tenant](#multi-tenant-schema-support) isolation described below; one partitions data across PostgreSQL schemas, the other scopes targeting within a single broker.

##### The System Generator

One generator is special. At broker startup the broker idempotently provisions a system generator (`__system__`, `is_system=true`), and every agent is automatically registered with it at creation (`POST /agents`). The system generator carries fleet- and system-wide stacks that must reach all agents, so fleet-scoped targeting works without any per-application opt-in. It is excluded from the public `GET /generators` listing. The system generator is distinct from the admin generator, which is tied to the admin role/PAK; agents are *not* auto-registered with the admin generator.

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

For debugging, the `RUST_LOG` environment variable controls log verbosity (e.g., `RUST_LOG=debug`, `RUST_LOG=brokkr_broker=trace`, or `RUST_LOG=tower_http=debug` for the HTTP layer).

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
    pub fn agents(&self) -> AgentsDAL {
        AgentsDAL::new(self.pool.clone(), self.schema.clone())
    }

    pub fn stacks(&self) -> StacksDAL {
        StacksDAL::new(self.pool.clone(), self.schema.clone())
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

For day-zero bootstrap, the `brokkr-broker generate-pak` subcommand mints an admin PAK and prints its SHA-256 hash entirely offline — no database connection and no keyfile are involved. The operator sets `BROKKR__BROKER__PAK_HASH` to that hash before the first broker startup; on startup the broker stores the hash on the admin role so the corresponding PAK authenticates as admin. The PAK itself is never written to disk by the broker. See the [CLI reference](../reference/cli.md) for the command and the [Environment Variables reference](../reference/environment-variables.md) for `BROKKR__BROKER__PAK_HASH`.

### Background Tasks Module

The broker runs a set of background tasks for maintenance operations:

**Diagnostic Cleanup** runs every 15 minutes (configurable) to remove diagnostic results older than 1 hour (configurable).

**Work Order Maintenance** runs every 10 seconds to process retry scheduling and detect stale claims.

**Webhook Delivery** runs every 5 seconds (configurable) to process pending webhook deliveries in batches of 50 (configurable).

**Webhook Cleanup** runs hourly to remove delivery records older than 7 days (configurable).

**Audit Log Cleanup** runs daily to remove audit entries older than 90 days (configurable).

**Agent Metrics Refresh** and **Agent Events Cleanup** keep agent telemetry current and prune old event history on their own intervals.

**Fleet Sweep** and **WebSocket Eviction** maintain fleet liveness and reap stale internal WebSocket connections.

### Utils Module

The utils module provides shared functionality:

**Event Emission** provides database-centric webhook dispatch by matching events against subscriptions and inserting delivery records directly.

**Audit Logger** provides non-blocking audit logging with batched database writes (100 entries or 1 second flush).

**Encryption** implements AES-256-GCM encryption for webhook secrets with versioned format for algorithm upgrades.

**PAK Controller** generates and verifies Prefixed API Keys using SHA-256 hashing with indexed lookups.

## Agent Components

The agent is implemented in Rust using Tokio for async operations and kube-rs for Kubernetes API interaction. It runs a continuous control loop that polls the broker and reconciles cluster state.

### Broker Communication Module

The broker module handles all communication with the Brokkr Broker service using REST API calls with PAK authentication. The agent polls the broker at configurable intervals; in addition, the `broker_ws` module maintains a persistent internal WebSocket connection used as a latency optimization for control-plane pushes and for streaming telemetry. REST polling remains the load-bearing path — when the WebSocket is down or its send lanes are full, every operation falls back to REST transparently (see [Internal Broker↔Agent WS Channel](./internal-ws-channel.md)).

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
| `/api/v1/agents/{id}/registrations` | GET | List generator scopes the agent is registered with |
| `/api/v1/generators/{id}/register` | POST | Register an agent with a generator (self or admin) |
| `/api/v1/generators/{id}/register` | DELETE | Deregister an agent from a generator |
| `/api/v1/generators/{id}/registered-agents` | GET | List agents registered with a generator |

On startup the agent self-registers with its configured generator scopes via `POST /generators/{id}/register` (see [Generator Self-Registration](#agent-startup-sequence) below). For the full request/response shapes see the [API reference](../reference/api/README.md).

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

The agent stamps tracking metadata onto every object it applies. Despite the `*_LABEL` constant names, all five keys are written as **annotations** on the applied object:

```rust
// All five are stamped as annotations by create_k8s_objects()
pub static STACK_LABEL: &str = "k8s.brokkr.io/stack";
pub static DEPLOYMENT_OBJECT_ID_LABEL: &str = "brokkr.io/deployment-object-id";
pub static CHECKSUM_ANNOTATION: &str = "k8s.brokkr.io/deployment-checksum";
pub static LAST_CONFIG_ANNOTATION: &str = "k8s.brokkr.io/last-config-applied";
pub static BROKKR_AGENT_OWNER_ANNOTATION: &str = "brokkr.io/owner-id";
```

Before deleting resources, the agent verifies ownership by checking the owner annotation to prevent removing resources managed by other systems. The checksum annotation enables detection of configuration drift. See the [Agent Annotations and Labels reference](../reference/agent-annotations.md) for the full catalog, including the keys the agent *looks for* (some of which are labels) for health checks and log streaming.

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
BROKKR__AGENT__POLLING_INTERVAL=10
BROKKR__AGENT__HEALTH_PORT=8080
BROKKR__AGENT__GENERATOR_IDS=<uuid>,<uuid>
```

`BROKKR__AGENT__GENERATOR_IDS` is a comma-separated list of generator UUIDs the agent self-registers with at startup. The agent resolves its generator scopes by precedence: the `--generator-ids` CLI flag wins, then `BROKKR__AGENT__GENERATOR_IDS` (config key `agent.generator_ids`, settable via env var or config file), then the legacy bare `BROKKR_GENERATOR_IDS` variable. `BROKKR_GENERATOR_IDS` is **deprecated** — it is still honored but logs a warning; prefer the namespaced form. Malformed UUIDs are skipped with a warning, and an empty/unset value means the agent serves only the system/fleet scope it is auto-registered with. The full catalog lives in the [Environment Variables reference](../reference/environment-variables.md).

## Configuration

The full configuration catalogs for both components live in the [Configuration guide](../getting-started/configuration.md) and the [Environment Variables reference](../reference/environment-variables.md).

The broker supports hot-reloading a limited set of values without a restart: the log level, diagnostic cleanup settings, webhook delivery interval/batch size/retention, and — of the CORS settings — only the allowed origins and the preflight `max_age_seconds` (allowed methods and headers require a restart). A manual reload can be triggered via `POST /api/v1/admin/config/reload`; when a configuration file is specified, the broker also watches it for filesystem changes with a 5-second debounce.

## Component Lifecycle

### Broker Startup Sequence

1. **Configuration Loading** - Parse environment variables and configuration files
2. **Database Connection** - Establish r2d2 connection pool to PostgreSQL
3. **Migration Check** - Verify database schema is current
4. **System Generator Provisioning** - Idempotently provision the `__system__` generator and ensure every existing agent is registered with it
5. **Encryption Initialization** - Load or generate webhook encryption key
6. **Event Emission Setup** - Database-centric: events are matched against subscriptions and inserted directly into `webhook_deliveries`; no in-memory event bus exists
7. **Audit Logger Initialization** - Start background writer with batching
8. **Background Tasks** - Spawn the maintenance task set (diagnostic, work order, webhook delivery/cleanup, audit, agent-metrics-refresh, agent-events-cleanup, fleet-sweep, and WebSocket-eviction)
9. **API Server** - Bind to configured port and start accepting requests

### Agent Startup Sequence

1. **Configuration Loading** - Parse environment variables
2. **PAK Verification** - Authenticate with broker and retrieve agent identity
3. **Generator Self-Registration** - Resolve generator scopes (`--generator-ids` flag > `BROKKR__AGENT__GENERATOR_IDS` > deprecated `BROKKR_GENERATOR_IDS`, which logs a warning) and self-register with each via `POST /generators/{id}/register`; registration with the system generator is implicit and needs no configuration. See [agent registration](../how-to/agent-registration.md) for the operational walkthrough
4. **Kubernetes Client** - Initialize kube-rs client with in-cluster or kubeconfig credentials
5. **Health Server** - Start HTTP server for probes and metrics
6. **Control Loop** - Enter main loop with polling, health checks, and work order processing

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

**Incremental Fetching** - The broker's target-state endpoint defaults to incremental mode, filtering out objects the agent has already reported events for, so polls return only new work.

**Parallel Apply** - Independent resources can be applied concurrently within priority groups.

**Connection Reuse** - HTTP client maintains connection pools to the broker and Kubernetes API.

**Efficient Diffing** - Checksum-based change detection avoids unnecessary applies for unchanged resources.
