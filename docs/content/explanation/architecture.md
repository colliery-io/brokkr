---
title: "Technical Architecture"
weight: 1
---

# Technical Architecture

Brokkr is a deployment orchestration platform designed to manage Kubernetes resources across multiple clusters from a central control plane. This document provides a comprehensive technical overview of Brokkr's architecture, explaining how its components are implemented, how they interact, and the design decisions that shape its behavior.

## System Overview

At its core, Brokkr follows a hub-and-spoke architecture where a central broker service coordinates deployments across multiple agent instances. The broker maintains the desired state of all deployments in a PostgreSQL database, while agents running in target Kubernetes clusters continuously poll the broker to fetch their assigned work and reconcile the actual cluster state to match the desired state.

This pull-based model was chosen deliberately over a push-based approach for several reasons. First, it simplifies network topology since agents only need outbound connectivity to the broker rather than requiring the broker to reach into potentially firewalled cluster networks. Second, it provides natural resilience since agents can continue operating with their cached state during temporary broker unavailability. Third, it allows agents to control their own reconciliation pace based on their cluster's capabilities and load.

## Broker Service Architecture

The broker is implemented as an asynchronous Rust service built on the Axum web framework with the Tokio runtime providing the async execution environment. When the broker starts, it initializes several interconnected subsystems that work together to provide the complete platform functionality.

### Initialization Sequence

The broker follows a carefully ordered startup sequence to ensure all dependencies are properly initialized before accepting requests. First, the configuration is loaded from environment variables and configuration files, establishing database connection parameters, authentication settings, and operational parameters like polling intervals.

Next, a PostgreSQL connection pool is established using the r2d2 connection manager with Diesel as the ORM layer. The pool is configured with a default of five connections, though this is adjustable for production workloads. If multi-tenant mode is enabled, the broker sets up the appropriate PostgreSQL schema to isolate tenant data.

After the database connection is established, Diesel migrations run automatically to ensure the schema is up to date. The broker then checks the `app_initialization` table to determine if this is a first-time startup. On first run, it creates the admin role and an associated admin generator, writing the initial admin PAK (Prefixed API Key) to a temporary file for retrieval.

With the database ready, the broker initializes its runtime subsystems in sequence: the Data Access Layer (DAL), the encryption subsystem for webhook secrets, the event bus for asynchronous event dispatch, the audit logger for compliance tracking, and finally the five background task workers. If running in Kubernetes, it also starts a ConfigMap watcher for hot-reload capability.

### API Layer

The API layer exposes a RESTful interface under the `/api/v1` prefix, with all endpoints documented via OpenAPI annotations that generate Swagger documentation. Every request to an API endpoint passes through authentication middleware that extracts and validates the PAK from the Authorization header.

The authentication middleware performs verification against three possible identity types in order: first checking the admin role table, then the agents table, and finally the generators table. This lookup is optimized with partial database indexes on the `pak_hash` column that exclude soft-deleted records, enabling O(1) lookup performance rather than full table scans.

Upon successful authentication, the middleware injects an `AuthPayload` structure into the request context containing the authenticated identity's type and ID. Individual endpoint handlers then perform authorization checks against this payload to determine if the caller has permission for the requested operation.

{{< mermaid >}}
flowchart TB
    subgraph Request["Incoming Request"]
        HTTP[HTTP Request]
    end

    subgraph Middleware["Authentication Layer"]
        Extract[Extract PAK from Header]
        Verify[Verify Against DB]
        Inject[Inject AuthPayload]
    end

    subgraph Handler["Endpoint Handler"]
        AuthZ[Authorization Check]
        Logic[Business Logic]
        DAL[Data Access Layer]
    end

    subgraph Database["PostgreSQL"]
        Pool[Connection Pool]
        Query[Execute Query]
    end

    HTTP --> Extract
    Extract --> Verify
    Verify --> Inject
    Inject --> AuthZ
    AuthZ --> Logic
    Logic --> DAL
    DAL --> Pool
    Pool --> Query
{{< /mermaid >}}

The API is organized into resource-specific modules: agents, generators, stacks, deployment objects, templates, work orders, webhooks, diagnostics, health status, and admin operations. Each module defines its routes, request/response structures, and authorization requirements.

### Data Access Layer

The DAL provides a clean abstraction over database operations, exposing specialized accessor types for each entity in the system. Rather than having a monolithic data access object, the DAL struct provides factory methods that return purpose-built accessor types: `dal.agents()` returns an `AgentsDAL`, `dal.stacks()` returns a `StacksDAL`, and so on for all twenty-two entity types in the system.

Each accessor type implements the standard CRUD operations appropriate for its entity, along with any specialized queries needed. For example, the `AgentsDAL` provides not only basic `create`, `get`, `update`, and `delete` operations, but also `get_by_pak_hash` for authentication lookups and filtered listing methods that support complex queries combining label and annotation filters.

The DAL uses Diesel's query builder for type-safe SQL generation, ensuring that queries are checked at compile time rather than failing at runtime with SQL syntax errors. All operations use the connection pool, automatically returning connections when operations complete.

Soft deletion is implemented throughout the system using a `deleted_at` timestamp column. When an entity is "deleted," this timestamp is set rather than removing the row, preserving the audit trail and enabling potential recovery. Most queries automatically filter out soft-deleted records.

### Event Bus

The event bus provides an asynchronous publish-subscribe mechanism for internal event dispatch, primarily used to trigger webhook deliveries. It is implemented as a singleton using Rust's `OnceCell` pattern, ensuring only one event bus instance exists throughout the application lifecycle.

When initialized, the event bus spawns a background task that continuously receives events from a Tokio mpsc channel. The channel is configured with a default buffer size of 1,000 events, providing backpressure when the system is overwhelmed. Events can be emitted in fire-and-forget mode using `emit()` or with await capability using `emit_async()`.

When an event arrives, the dispatcher queries the database for all webhook subscriptions whose event type pattern matches the event. Pattern matching supports exact matches like `workorder.completed`, wildcard suffixes like `health.*` that match any health-related event, and a catch-all `*` pattern that matches everything. For each matching subscription, a webhook delivery record is created in PENDING status for the webhook delivery worker to process.

### Background Tasks

The broker runs five concurrent background tasks that handle various maintenance and processing duties:

**Diagnostic Cleanup Task** runs on a configurable interval (default: every 15 minutes) to expire pending diagnostic requests that have exceeded their timeout and delete completed, expired, or failed diagnostic records older than the configured maximum age (default: 1 hour). This prevents the database from accumulating stale diagnostic data.

**Work Order Maintenance Task** runs frequently (default: every 10 seconds) to handle work order lifecycle transitions. It moves work orders from RETRY_PENDING status back to PENDING when their backoff period has elapsed, allowing them to be claimed again. It also reclaims work orders that were claimed but never completed within the timeout period, returning them to PENDING for another agent to attempt.

**Webhook Delivery Worker** runs on a short interval (default: every 5 seconds) to process pending webhook deliveries. It fetches a batch of pending deliveries (default: 50), retrieves and decrypts the webhook URL and authentication header for each subscription, performs an HTTP POST with a 30-second timeout, and records the result. Failed deliveries are scheduled for retry with exponential backoff until they exceed the maximum retry count, at which point they are marked as dead.

**Webhook Cleanup Task** runs less frequently (default: hourly) to delete completed and dead webhook deliveries older than the retention period (default: 7 days), preventing unbounded growth of delivery history.

**Audit Log Cleanup Task** runs daily to delete audit log entries older than the configured retention period (default: 90 days), balancing compliance requirements against storage costs.

### Audit Logger

The audit logger provides asynchronous, batched recording of security-relevant events for compliance and forensic purposes. Like the event bus, it is implemented as a singleton with an internal mpsc channel and background writer task.

Rather than writing each audit entry immediately to the database, which would create significant write amplification, the logger buffers entries and writes them in batches. The writer task accumulates entries up to a configurable batch size (default: 100) or until a flush interval elapses (default: 1 second), whichever comes first. This approach dramatically reduces database write pressure while maintaining a nearly real-time audit trail.

Audit entries capture the actor type (admin, agent, generator, or system), the actor's ID, the action performed, the resource type affected, the resource's ID, and additional contextual details. This information enables security teams to reconstruct the sequence of events leading to any system state.

### Configuration Hot-Reload

When running in Kubernetes, the broker can automatically detect and apply configuration changes without requiring a pod restart. The config watcher uses the Kubernetes API to watch the broker's ConfigMap, detecting modifications through the watch stream.

When a change is detected, the watcher applies a configurable debounce period (default: 5 seconds) to coalesce rapid successive changes into a single reload operation. Only certain configuration values support hot-reload: log level, diagnostic cleanup intervals, webhook delivery settings, and CORS configuration. Settings that affect initialization, like database connection parameters or TLS configuration, require a pod restart.

The admin API also exposes a manual reload endpoint at `POST /api/v1/admin/config/reload` that triggers an immediate configuration reload, useful for non-Kubernetes deployments or when changes need to take effect immediately.

## Agent Service Architecture

The agent is a Kubernetes-native component that runs inside target clusters, responsible for applying deployment objects to the cluster and reporting status back to the broker. It is designed to be resilient, continuing to operate with cached state during broker unavailability.

### Startup and Initialization

When the agent starts, it first loads its configuration, which must include the broker URL, the agent's name and cluster name, and its PAK for authentication. It then enters a readiness loop, repeatedly checking the broker's health endpoint until it receives a successful response or exceeds its retry limit.

Once the broker is reachable, the agent verifies its PAK by calling the authentication endpoint. A successful response confirms the agent is properly registered and authorized. The agent then fetches its full details from the broker, including its assigned labels and annotations that determine which deployments it will receive.

With authentication confirmed, the agent initializes its Kubernetes client using in-cluster configuration (when running as a pod) or a specified kubeconfig path (for development). It validates connectivity by fetching the cluster's API server version.

Finally, the agent starts an HTTP server on port 8080 for health checks and metrics, then enters its main control loop.

### Main Control Loop

The agent's control loop is implemented using Tokio's `select!` macro to concurrently await multiple timer-based tasks. This design allows multiple activities to proceed in parallel while ensuring orderly shutdown when a termination signal is received.

{{< mermaid >}}
flowchart TB
    subgraph Loop["Main Control Loop"]
        direction TB
        HB[Heartbeat Timer]
        Deploy[Deployment Check Timer]
        WO[Work Order Timer]
        Health[Health Check Timer]
        Diag[Diagnostics Timer]
        Shutdown[Shutdown Signal]
    end

    subgraph Broker["Broker Communication"]
        Fetch[Fetch Deployments]
        Report[Report Events]
        Beat[Send Heartbeat]
    end

    subgraph K8s["Kubernetes Operations"]
        Apply[Apply Resources]
        Delete[Delete Resources]
        Monitor[Monitor Health]
    end

    HB --> Beat
    Deploy --> Fetch
    Fetch --> Apply
    Apply --> Report
    WO --> Fetch
    Health --> Monitor
    Monitor --> Report
{{< /mermaid >}}

**Heartbeat Timer** fires at a configurable interval (default: 30 seconds) to send a heartbeat to the broker, maintaining the agent's "alive" status. The response includes updated agent details, allowing the broker to push configuration changes like label updates.

**Deployment Check Timer** fires at the configured polling interval to fetch the agent's target state from the broker. The agent compares this desired state against what it has previously applied and performs reconciliation to converge them.

**Work Order Timer** polls for pending work orders assigned to the agent. Work orders represent transient operations like container image builds that don't fit the declarative deployment model.

**Health Check Timer** (when enabled) periodically evaluates the health of deployments the agent has applied, checking pod status, container states, and conditions to produce health summaries reported back to the broker.

**Diagnostics Timer** polls for on-demand diagnostic requests, which allow administrators to collect debugging information from specific deployments.

### Reconciliation Engine

The reconciliation engine is the heart of the agent, responsible for applying Kubernetes resources to achieve the desired state while handling the complexities of resource dependencies, conflicts, and failures.

When the agent receives deployment objects from the broker, it first parses the YAML content into Kubernetes DynamicObject instances. The objects are then sorted to apply cluster-scoped resources (Namespaces and CustomResourceDefinitions) before namespace-scoped resources, ensuring dependencies exist before resources that need them.

Before applying each resource, the agent injects standard metadata labels that link the resource back to Brokkr: the stack ID, a checksum of the YAML content for change detection, the deployment object ID, and the agent's ID as the owner. This metadata enables the agent to identify which resources it manages and detect changes.

Application uses Kubernetes server-side apply with field ownership, which handles merge conflicts more gracefully than client-side apply. Before applying in earnest, resources are validated with a dry-run request to catch schema errors or policy violations early.

When deployment objects are removed (indicated by deletion markers), the agent deletes the corresponding resources from the cluster. However, it only deletes resources it owns, verified by checking the owner ID label. This prevents accidental deletion of resources created by other systems.

The reconciliation engine implements exponential backoff retry logic for transient failures like rate limiting (HTTP 429) or temporary server errors (5xx). Non-retryable errors like authorization failures (403) or resource not found (404) fail immediately to avoid wasting retry attempts.

### Work Order Processing

Work orders handle operations that don't fit the declarative resource model, such as building container images. The agent polls for pending work orders, claims one for exclusive processing, executes it, and reports the result.

For build work orders, the agent parses the YAML to extract Shipwright Build resources, applies them to the cluster, creates a BuildRun to trigger execution, and monitors the BuildRun status until completion. Build progress is checked every 5 seconds with a 15-minute overall timeout. On success, the agent reports the resulting image digest; on failure, it reports the error reason.

The work order system includes retry logic: when a work order fails with a retryable error, it is scheduled for retry with exponential backoff. Non-retryable errors are marked as permanently failed.

### Health Monitoring

When deployment health monitoring is enabled, the agent periodically evaluates the health of resources it has applied. For each tracked deployment object, it queries for pods matching the deployment object ID label and analyzes their status.

The health checker examines pod phase, conditions, and container states to produce a health assessment. It specifically looks for problematic conditions like ImagePullBackOff, CrashLoopBackOff, OOMKilled, and various container creation errors. Based on its analysis, it assigns one of four health statuses: healthy (all pods running and ready), degraded (issues detected but not failed), failing (pod in Failed phase), or unknown (unable to determine status).

Health summaries include the count of ready versus total pods, a list of detected issues, and detailed resource information. This data is reported to the broker, which stores it for display in management interfaces and can trigger webhook notifications based on health changes.

## Component Interaction Patterns

Understanding how the broker and agents interact is essential for operating and troubleshooting Brokkr deployments.

### Deployment Flow

The deployment lifecycle begins when an administrator or CI/CD system (acting as a generator) creates a stack and adds deployment objects to it. The stack serves as a logical grouping with labels that determine which agents will receive its deployments.

When a deployment object is created, the broker's matching engine evaluates all registered agents to find those whose labels satisfy the stack's targeting requirements. For each matching agent, it creates an agent target record linking the agent to the stack.

{{< mermaid >}}
sequenceDiagram
    participant Admin as Administrator
    participant Broker as Broker
    participant DB as PostgreSQL
    participant Agent as Agent
    participant K8s as Kubernetes

    Admin->>Broker: Create Stack with Labels
    Broker->>DB: Store Stack
    Admin->>Broker: Create Deployment Object
    Broker->>DB: Store Deployment Object
    Broker->>Broker: Find Matching Agents
    Broker->>DB: Create Agent Targets

    loop Every Polling Interval
        Agent->>Broker: Request Target State
        Broker->>DB: Query Agent's Deployments
        Broker-->>Agent: Deployment Objects
        Agent->>Agent: Compare Desired vs Actual
        Agent->>K8s: Apply Resources
        K8s-->>Agent: Result
        Agent->>Broker: Report Event
        Broker->>DB: Store Event
        Broker->>Broker: Dispatch Webhooks
    end
{{< /mermaid >}}

On the agent side, the deployment check timer fires and requests the agent's target state from the broker. The broker returns all deployment objects from stacks the agent is targeting, along with sequence IDs that enable incremental synchronization. The agent compares this desired state against what it has applied, performs the necessary create, update, or delete operations in Kubernetes, and reports events back to the broker.

### Event and Webhook Flow

Events flow from agents through the broker to external systems via webhooks. When an agent reports an event (success, failure, health change), the broker stores it in the database and publishes it to the event bus.

The event bus dispatcher receives the event and queries for matching webhook subscriptions. For each match, it creates a webhook delivery record. The webhook delivery worker picks up these records and attempts HTTP delivery to the configured endpoints.

This decoupled architecture ensures that event processing doesn't block API responses—the agent receives an immediate acknowledgment while webhook delivery proceeds asynchronously with its own retry logic.

## Performance Characteristics

The broker is designed to handle substantial load with modest resources. The API layer uses Axum's async request handling to efficiently multiplex many concurrent connections onto a small number of OS threads. Connection pooling minimizes database connection overhead.

**API Performance**: Under typical conditions, API requests complete in under 50 milliseconds at the 95th percentile, with the broker capable of handling over 1,000 requests per second on a single instance.

**Database Performance**: Query latency is typically under 20 milliseconds, with indexed lookups for authentication completing in single-digit milliseconds. The connection pool maintains 20-100 connections depending on configuration.

**Agent Performance**: Resource application completes in under 100 milliseconds per resource, with the reconciliation loop typically completing in under 1 second. Event reporting has sub-50ms latency to the broker.

For larger deployments, the broker can be horizontally scaled behind a load balancer since it maintains no in-memory state beyond caches—all persistent state lives in PostgreSQL. Session affinity is recommended for WebSocket connections if that feature is used.

## Scaling Patterns

### Horizontal Broker Scaling

The broker's stateless design enables straightforward horizontal scaling. Multiple broker instances can run behind a load balancer, all connecting to the same PostgreSQL database. Each instance runs its own background tasks, but these are designed to be safe for concurrent execution through database-level coordination (e.g., work order claiming uses atomic updates to prevent double-claiming).

{{< mermaid >}}
graph TD
    LB[Load Balancer] --> B1[Broker 1]
    LB --> B2[Broker 2]
    LB --> B3[Broker 3]
    B1 --> DB[(PostgreSQL Primary)]
    B2 --> DB
    B3 --> DB
    DB --> DBR[(PostgreSQL Replica)]
{{< /mermaid >}}

### Agent Deployment

Each Kubernetes cluster runs one agent instance. The agent handles all deployments for that cluster, with no need for multiple agents per cluster. Agents operate independently with no inter-agent communication, simplifying the operational model.

For very large clusters or high deployment volumes, the agent's polling interval can be tuned to balance responsiveness against API load.

## Resource Requirements

| Component | CPU Request | Memory Request | CPU Limit | Memory Limit |
|-----------|-------------|----------------|-----------|--------------|
| Broker | 100m | 256Mi | 500m | 512Mi |
| Agent | 50m | 128Mi | 200m | 256Mi |
| PostgreSQL | 250m | 256Mi | 500m | 512Mi |

These are conservative defaults suitable for small to medium deployments. Production deployments handling thousands of deployment objects or hundreds of agents should increase these limits based on observed resource utilization.

## Multi-Tenancy

The broker supports multi-tenant deployments through PostgreSQL schema isolation. When configured with a schema name, the broker creates a dedicated schema and sets the connection's `search_path` to use it for all queries. This provides data isolation between tenants sharing a PostgreSQL instance without requiring separate databases.

Schema names are validated to prevent SQL injection, accepting only alphanumeric characters and underscores, and requiring the name to start with a letter.
