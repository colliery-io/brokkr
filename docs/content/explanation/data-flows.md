---
title: "Data Flows"
weight: 6
---

# Data Flows

This document traces the journey of data through the Brokkr system, from initial deployment creation through resource application in target clusters and event propagation to external systems. Understanding these flows is essential for debugging issues, optimizing performance, and building integrations with Brokkr.

## Deployment Lifecycle

The deployment lifecycle encompasses the complete journey of a deployment object from creation through application on target clusters. This flow demonstrates Brokkr's immutable, append-only data model and its approach to eventual consistency.

### Creating a Deployment

Deployments begin their lifecycle when an administrator or generator creates a stack and submits deployment objects to it. The broker processes these submissions through several stages, ultimately targeting them to appropriate agents.

{{< mermaid >}}
sequenceDiagram
    participant Client as Admin/Generator
    participant Broker as Broker API
    participant DB as PostgreSQL
    participant Match as Targeting Logic

    Client->>Broker: POST /api/v1/stacks
    Broker->>DB: INSERT stack
    DB-->>Broker: Stack created
    Broker-->>Client: Stack response (with ID)

    Client->>Broker: POST /api/v1/stacks/{id}/labels
    Broker->>DB: INSERT stack_labels
    Note over Broker: Labels used for agent targeting

    Client->>Broker: POST /api/v1/stacks/{id}/deployment-objects
    Broker->>DB: INSERT deployment_object
    Broker->>Match: Find matching agents
    Match->>DB: Query agents by labels
    DB-->>Match: Matching agents
    Match->>DB: INSERT agent_targets
    Broker-->>Client: Deployment object response

    Note over DB: Deployment objects are immutable after creation
{{< /mermaid >}}

The broker assigns each deployment object a sequence ID upon creation, establishing a strict ordering that agents use to process updates in the correct sequence. This sequence ID is monotonically increasing within each stack, ensuring that newer deployment objects always have higher sequence IDs than older ones. The combination of stack ID and sequence ID provides a reliable mechanism for agents to track which objects they have already processed.

When a deployment object is created, the broker does not immediately push it to agents. Instead, the targeting logic creates entries in the `agent_targets` table that associate the stack with eligible agents. Agents discover these targets during their next polling cycle and fetch the relevant deployment objects.

### Agent Reconciliation

Agents continuously poll the broker and reconcile their cluster state to match the desired state defined by deployment objects. The reconciliation loop runs at a configurable interval, defaulting to 30 seconds.

{{< mermaid >}}
sequenceDiagram
    participant Agent as Agent
    participant Broker as Broker API
    participant DB as PostgreSQL
    participant K8s as Kubernetes API
    participant Cluster as Cluster State

    loop Every polling interval (default: 30s)
        Agent->>Broker: GET /api/v1/agents/{id}/target-state
        Broker->>DB: Query targeted objects for agent
        DB-->>Broker: Deployment objects list
        Broker-->>Agent: Deployment objects (with sequence IDs)

        Agent->>Agent: Calculate diff (desired vs actual)

        alt New objects to apply
            Agent->>K8s: Apply resource (create/update)
            K8s->>Cluster: Resource created/updated
            K8s-->>Agent: Success/Failure
            Agent->>Broker: POST /api/v1/agents/{id}/events
            Broker->>DB: INSERT agent_event
        end

        alt Objects to delete (deletion markers)
            Agent->>K8s: Delete resource
            K8s->>Cluster: Resource deleted
            K8s-->>Agent: Success/Failure
            Agent->>Broker: POST /api/v1/agents/{id}/events
            Broker->>DB: INSERT agent_event
        end

        Agent->>Agent: Update local state cache
    end
{{< /mermaid >}}

The agent's `GET /api/v1/agents/{id}/target-state` endpoint returns deployment objects the agent is responsible for, filtered to exclude objects already deployed (based on agent events). This optimization reduces payload size and processing time for agents managing large numbers of deployments.

During reconciliation, the agent uses Kubernetes server-side apply to create or update resources. This approach preserves fields managed by other controllers while allowing the agent to manage its own fields. The agent orders resource application to respect dependencies: Namespaces and CustomResourceDefinitions are applied before resources that depend on them.

After each successful operation, the agent reports an event to the broker. These events serve multiple purposes: they update the broker's view of deployment state, they trigger webhook notifications to external systems, and they provide an audit trail of all operations.

### Deployment Object States

Deployment objects follow an implicit lifecycle tracked through their presence, associated agent events, and deletion markers. The state model uses soft deletion to maintain a complete audit trail while supporting reliable cleanup.

{{< mermaid >}}
stateDiagram-v2
    [*] --> Created: POST deployment-object
    Created --> Targeted: Agent matching
    Targeted --> Applied: Agent applies
    Applied --> Updated: New version created
    Updated --> Applied: Agent applies update
    Applied --> MarkedForDeletion: Deletion marker created
    MarkedForDeletion --> Deleted: Agent deletes
    Deleted --> [*]: Soft delete (retained)
{{< /mermaid >}}

**Created** indicates a deployment object exists in the database but has not yet been targeted to any agents. This state typically transitions quickly to Targeted as the broker processes agent matching.

**Targeted** means one or more agents are responsible for this deployment object. The `agent_targets` table records these associations, linking stacks to agents based on label matching.

**Applied** indicates the agent has successfully applied the resource to its cluster and reported an event confirming the operation. The agent event records the deployment object ID, timestamp, and outcome.

**Updated** represents a transitional state where a new deployment object with a higher sequence ID has been created for the same logical resource. The agent detects this during reconciliation and applies the update.

**MarkedForDeletion** occurs when a deletion marker deployment object is created. Deletion markers are deployment objects with a special flag indicating the agent should delete the referenced resource rather than apply it.

**Deleted** indicates the agent has removed the resource from the cluster. Both the original deployment object and the deletion marker remain in the database with `deleted_at` timestamps for audit purposes.

### Deletion Flow

Deleting resources uses a marker pattern that ensures reliable cleanup even when agents are temporarily unavailable. Rather than immediately removing data, the broker creates a deletion marker that agents process during their normal reconciliation cycle.

{{< mermaid >}}
sequenceDiagram
    participant Client as Admin
    participant Broker as Broker API
    participant DB as PostgreSQL
    participant Agent as Agent
    participant K8s as Kubernetes

    Client->>Broker: POST /api/v1/stacks/{id}/deployment-objects
    Note over Client,Broker: is_deletion_marker: true

    Broker->>DB: INSERT deployment_object (deletion marker)
    Broker-->>Client: Deletion marker created

    Note over Agent: Next polling interval
    Agent->>Broker: GET /api/v1/agents/{id}/target-state
    Broker-->>Agent: Includes deletion marker

    Agent->>Agent: Detect deletion marker
    Agent->>K8s: DELETE resource
    K8s-->>Agent: Deleted

    Agent->>Broker: POST /api/v1/agents/{id}/events
    Note over Broker: Event type: DELETED

    Note over DB: Both original and marker<br/>remain for audit trail
{{< /mermaid >}}

This approach has several advantages over immediate deletion. First, it provides reliable cleanup even when agents are offline—when they reconnect, they process accumulated deletion markers. Second, it maintains a complete history of what was deployed and when it was removed. Third, it allows for rollback by creating new deployment objects that restore deleted resources.

## Event Flow

Events form the nervous system of Brokkr, propagating state changes from agents through the broker to external systems. The event system handles agent reports, webhook notifications, and audit logging through an asynchronous architecture designed for high throughput and reliability.

### Event Architecture

The broker implements an event bus pattern using Tokio's asynchronous channels. Events flow through several processing stages before reaching their final destinations.

{{< mermaid >}}
flowchart LR
    subgraph Agent
        Apply[Apply Resource]
        Report[Event Reporter]
    end

    subgraph Broker
        API[API Handler]
        DB[(Database)]
        EventBus[Event Bus]
        Webhook[Webhook Worker]
        Audit[Audit Logger]
    end

    subgraph External
        Endpoints[Webhook Endpoints]
        Logs[Audit Logs]
    end

    Apply --> Report
    Report -->|POST /agents/{id}/events| API
    API --> DB
    API --> EventBus
    EventBus --> Webhook
    EventBus --> Audit
    Webhook --> Endpoints
    Audit --> Logs
{{< /mermaid >}}

The event bus initializes with a 1000-entry bounded channel, providing backpressure when the system is under heavy load. Events are emitted using a fire-and-forget pattern—the emitter does not wait for downstream processing to complete. This design prevents slow webhook endpoints from blocking the main API handlers.

A dedicated dispatcher task receives events from the channel and routes them to appropriate handlers. For webhook events, the dispatcher queries matching subscriptions and creates delivery records in the database. For audit events, the dispatcher batches writes to optimize database performance.

### Agent Event Reporting

Agents report events to the broker after completing each operation. The `POST /api/v1/agents/{id}/events` endpoint accepts event data and persists it to the `agent_events` table.

| Event Type | Trigger | Data Included |
|------------|---------|---------------|
| `APPLIED` | Resource successfully applied | Resource details, timestamp |
| `UPDATED` | Resource successfully updated | Resource details, changes |
| `DELETED` | Resource successfully deleted | Resource details |
| `FAILED` | Operation failed | Error message, resource details |
| `HEALTH_CHECK` | Periodic health status | Deployment health summary |

The agent includes comprehensive metadata with each event: the deployment object ID, resource GVK (Group/Version/Kind), namespace and name, operation result, and any error messages. This data enables precise tracking of deployment state and troubleshooting of failures.

Events are processed synchronously in the API handler—the database insert must succeed before the endpoint returns. However, downstream processing (webhook delivery, audit logging) happens asynchronously through the event bus.

### Webhook Delivery

Webhook subscriptions enable external systems to receive notifications when events occur in Brokkr. The delivery system prioritizes reliability through persistent queuing and automatic retries.

{{< mermaid >}}
sequenceDiagram
    participant EventBus as Event Bus
    participant DB as PostgreSQL
    participant Worker as Webhook Worker
    participant Endpoint as External Endpoint

    EventBus->>DB: Find matching subscriptions
    DB-->>EventBus: Subscription list
    EventBus->>DB: INSERT webhook_delivery (per subscription)

    loop Every 5 seconds
        Worker->>DB: SELECT pending deliveries (batch of 50)
        DB-->>Worker: Delivery batch

        par For each delivery
            Worker->>DB: Get subscription details
            Worker->>Worker: Decrypt URL and auth header
            Worker->>Endpoint: POST event payload
            alt Success (2xx)
                Endpoint-->>Worker: 200 OK
                Worker->>DB: Mark success
            else Failure
                Endpoint-->>Worker: Error
                Worker->>DB: Schedule retry (exponential backoff)
            end
        end
    end
{{< /mermaid >}}

The webhook worker runs as a background task, polling for pending deliveries every 5 seconds (configurable via `broker.webhookDeliveryIntervalSeconds`). Each polling cycle processes up to 50 deliveries (configurable via `broker.webhookDeliveryBatchSize`), enabling high throughput while controlling resource usage.

Delivery URLs and authentication headers are stored encrypted in the database using AES-256-GCM. The worker decrypts these values just before making the HTTP request, minimizing the time sensitive data exists in memory.

Failed deliveries are retried with exponential backoff. The first retry occurs after 2 seconds, the second after 4 seconds, then 8, 16, and so on. After exhausting the maximum retry count (configurable), deliveries are marked as "dead" and no longer retried. A cleanup task removes old delivery records after 7 days (configurable via `broker.webhookCleanupRetentionDays`).

### Event Types

Brokkr emits events for various system activities, enabling external systems to react to state changes.

| Category | Event Types | Description |
|----------|-------------|-------------|
| **Deployment** | `deployment.applied`, `deployment.updated`, `deployment.deleted`, `deployment.failed` | Resource operations on clusters |
| **Work Order** | `workorder.completed`, `workorder.failed` | Work order lifecycle events |
| **Health** | `deployment.health.changed` | Health state transitions |
| **Agent** | `agent.connected`, `agent.disconnected` | Agent connectivity changes |

Webhook subscriptions can filter by event type, receiving only the events relevant to their use case. This filtering reduces unnecessary network traffic and processing on the receiving end.

## Authentication Flows

Authentication in Brokkr varies by actor type: agents use PAKs, administrators use bearer tokens, and generators use API keys. Each flow has distinct security properties appropriate to its use case.

### PAK Authentication

Prefixed API Keys (PAKs) provide secure, stateless authentication for agents. The PAK contains both an identifier and a secret component, enabling the broker to authenticate requests without storing plaintext secrets.

{{< mermaid >}}
sequenceDiagram
    participant Agent as Agent
    participant Broker as Broker API
    participant Auth as Auth Middleware
    participant DB as PostgreSQL

    Note over Agent: Agent startup
    Agent->>Agent: Load PAK from config

    Agent->>Broker: GET /api/v1/agents/{id}/target-state
    Note over Agent,Broker: Authorization: Bearer {PAK}

    Broker->>Auth: Validate PAK
    Auth->>Auth: Parse PAK structure
    Auth->>Auth: Extract short token (identifier)
    Auth->>DB: Lookup agent by short token
    DB-->>Auth: Agent record with hash
    Auth->>Auth: Hash long token from request
    Auth->>Auth: Compare with stored hash

    alt Hashes match
        Auth-->>Broker: Agent identity
        Broker->>Broker: Continue with request
        Broker-->>Agent: Response
    else Invalid/Revoked
        Auth-->>Broker: Authentication failed
        Broker-->>Agent: 401 Unauthorized
    end
{{< /mermaid >}}

PAK structure follows a defined format: `brokkr_BR{short_token}_{long_token}`. The short token serves as an identifier that can be safely logged and displayed. The long token is the secret component—it is hashed with SHA-256 before storage, and the plaintext is never persisted.

When an agent authenticates, the middleware extracts the short token to locate the agent record, then hashes the provided long token and compares it with the stored hash. This constant-time comparison prevents timing attacks that could reveal information about valid tokens.

PAKs can be rotated through the `POST /api/v1/agents/{id}/rotate-pak` endpoint, which generates a new PAK and invalidates the previous one. The new PAK is returned only once—it cannot be retrieved later.

### Admin Authentication

Administrators authenticate using bearer tokens for management operations. These tokens grant broad access to system configuration and monitoring endpoints.

{{< mermaid >}}
sequenceDiagram
    participant Admin as Admin Client
    participant Broker as Broker API
    participant Auth as Auth Middleware
    participant DB as PostgreSQL

    Admin->>Broker: POST /api/v1/admin/config/reload
    Note over Admin,Broker: Authorization: Bearer {token}

    Broker->>Auth: Validate token
    Auth->>DB: Lookup admin token
    DB-->>Auth: Token record (if exists)

    alt Valid Token
        Auth->>Auth: Verify permissions
        Auth-->>Broker: Admin identity
        Broker->>Broker: Execute admin operation
        Broker-->>Admin: Response
    else Invalid Token
        Auth-->>Broker: Authentication failed
        Broker-->>Admin: 401 Unauthorized
    end
{{< /mermaid >}}

Admin tokens enable access to sensitive operations including configuration reload, audit log queries, agent management, and system health endpoints. The token is verified against stored hashes, following the same security pattern as PAK authentication.

### Generator Authentication

Generators, typically CI/CD systems, authenticate using API keys. These keys enable automated deployment workflows while maintaining security boundaries.

{{< mermaid >}}
sequenceDiagram
    participant Generator as Generator/CI
    participant Broker as Broker API
    participant Auth as Auth Middleware
    participant DB as PostgreSQL

    Generator->>Broker: POST /api/v1/stacks
    Note over Generator,Broker: X-Generator-Key: {api_key}

    Broker->>Auth: Validate generator key
    Auth->>DB: Lookup generator
    DB-->>Auth: Generator record

    alt Valid Key
        Auth-->>Broker: Generator identity
        Broker->>DB: Create stack (with generator_id)
        Broker-->>Generator: Stack created
    else Invalid Key
        Auth-->>Broker: Authentication failed
        Broker-->>Generator: 401 Unauthorized
    end
{{< /mermaid >}}

Generators can create and manage stacks and deployment objects, but they cannot access admin endpoints or manage other generators. Resources created by a generator are associated with its identity, enabling audit tracking and future access control enhancements.

## Work Order Flow

Work orders enable the broker to dispatch tasks to agents for execution. Unlike deployment objects which represent desired state, work orders represent one-time operations like container image builds or diagnostic commands.

{{< mermaid >}}
sequenceDiagram
    participant Client as Admin/API
    participant Broker as Broker API
    participant DB as PostgreSQL
    participant Agent as Agent
    participant K8s as Kubernetes
    participant Build as Build System

    Client->>Broker: POST /api/v1/work-orders
    Broker->>DB: INSERT work_order (status: PENDING)
    Broker-->>Client: Work order created

    Note over Agent: Next polling interval
    Agent->>Broker: GET /api/v1/agents/{id}/work-orders/pending
    Broker->>DB: Query matching work orders
    DB-->>Broker: Work orders
    Broker-->>Agent: Work order details

    Agent->>Broker: POST /api/v1/work-orders/{id}/claim
    Broker->>DB: Update status: CLAIMED
    Broker-->>Agent: Claim confirmed

    Agent->>K8s: Create Build resource
    K8s->>Build: Execute build

    loop Monitor progress
        Agent->>K8s: Check build status
        K8s-->>Agent: Build status
    end

    Build-->>K8s: Build complete
    K8s-->>Agent: Success + artifacts
    Agent->>Broker: POST /api/v1/work-orders/{id}/complete
    Broker->>DB: Move to work_order_log
    Broker->>DB: DELETE work_order
    Broker-->>Agent: Acknowledged
{{< /mermaid >}}

Work orders support sophisticated targeting through three mechanisms: hard targets (specific agent IDs), label matching (agents with matching labels), and annotation matching (agents with matching annotations). An agent is eligible to claim a work order if it matches any of these criteria.

The claiming mechanism prevents multiple agents from processing the same work order. When an agent claims a work order, the broker atomically updates its status to CLAIMED and records the claiming agent's ID. If the claim succeeds, the agent proceeds with execution; if another agent already claimed it, the claim fails.

### Work Order States

Work orders transition through a defined set of states, with automatic retry handling for transient failures.

| State | Description | Transitions To |
|-------|-------------|----------------|
| **PENDING** | Awaiting claim by an agent | CLAIMED |
| **CLAIMED** | Agent is processing | SUCCESS (to log), RETRY_PENDING |
| **RETRY_PENDING** | Scheduled for retry after failure | PENDING (after backoff) |

Successful completion moves the work order to the `work_order_log` table and deletes the original record. This design keeps the active work order table small while maintaining a complete history of completed operations.

Failed work orders may be retried depending on configuration. When a retryable failure occurs, the work order enters RETRY_PENDING status with a scheduled retry time based on exponential backoff. A background task runs every 10 seconds, resetting RETRY_PENDING work orders to PENDING once their retry time has elapsed.

Stale claims are automatically detected and reset. If an agent claims a work order but fails to complete it within the configured timeout (due to crash or network partition), the broker resets the work order to PENDING, allowing another agent to claim it.

## Data Retention

Brokkr maintains extensive data for auditing, debugging, and compliance purposes. Different data types have different retention characteristics based on their importance and storage requirements.

### Immutability Pattern

Deployment objects use an append-only pattern that preserves complete history. Objects are never modified after creation—updates create new objects with higher sequence IDs, and deletions create deletion markers rather than removing data. This approach enables precise audit trails and potential rollback to any previous state.

The `deleted_at` timestamp implements soft deletion across most entity types. Queries filter by `deleted_at IS NULL` by default, hiding deleted records from normal operations while preserving them for auditing. Special "include deleted" query variants provide access to the full history when needed.

### Retention Policies

| Data Type | Default Retention | Cleanup Method |
|-----------|-------------------|----------------|
| Deployment objects | Permanent | Soft delete only |
| Agent events | Permanent | Soft delete only |
| Webhook deliveries | 7 days | Background cleanup task |
| Audit logs | 90 days | Background cleanup task |
| Diagnostic results | 1 hour | Background cleanup task |

Background tasks run at regular intervals to enforce retention policies. The webhook cleanup task runs hourly, removing deliveries older than the configured retention period. The audit log cleanup task runs daily, removing entries beyond the retention window. Diagnostic results have a short retention period (1 hour by default) as they contain point-in-time debugging information.

### Sequence ID Tracking

Agents track the highest sequence ID they have processed for each stack, enabling efficient incremental fetching. When an agent reconnects after downtime, it requests only objects with sequence IDs higher than its last processed value. This mechanism ensures reliable delivery of all updates while minimizing network traffic and processing time.

The `GET /api/v1/agents/{id}/target-state` endpoint leverages sequence tracking to return only unprocessed objects, reducing response size for agents managing large deployments.
