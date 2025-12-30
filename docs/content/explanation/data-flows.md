---
title: "Data Flows"
weight: 6
---

# Data Flows

This document explains how data flows through the Brokkr system, from deployment creation through resource application in target clusters.

## Deployment Lifecycle

### Creating a Deployment

The deployment lifecycle begins when an admin or generator creates a stack and deployment objects:

{{< mermaid >}}
sequenceDiagram
    participant Client as Admin/Generator
    participant Broker as Broker API
    participant DB as PostgreSQL
    participant Match as Matching Engine

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

**Key Points:**
- Stacks group related deployment objects
- Labels on stacks determine which agents receive the deployment
- Deployment objects are immutable (append-only with soft delete)
- Agent targets are created automatically based on label matching

### Agent Reconciliation Loop

Agents continuously poll the broker and reconcile their cluster state:

{{< mermaid >}}
sequenceDiagram
    participant Agent as Agent
    participant Broker as Broker API
    participant DB as PostgreSQL
    participant K8s as Kubernetes API
    participant Cluster as Cluster State

    loop Every polling interval (default: 30s)
        Agent->>Broker: GET /api/v1/agent/deployment-objects
        Broker->>DB: Query targeted objects for agent
        DB-->>Broker: Deployment objects list
        Broker-->>Agent: Deployment objects (with sequence IDs)

        Agent->>Agent: Calculate diff (desired vs actual)

        alt New objects to apply
            Agent->>K8s: Apply resource (create/update)
            K8s->>Cluster: Resource created/updated
            K8s-->>Agent: Success/Failure
            Agent->>Broker: POST /api/v1/agent/events
            Broker->>DB: INSERT agent_event
        end

        alt Objects to delete (deletion markers)
            Agent->>K8s: Delete resource
            K8s->>Cluster: Resource deleted
            K8s-->>Agent: Success/Failure
            Agent->>Broker: POST /api/v1/agent/events
            Broker->>DB: INSERT agent_event
        end

        Agent->>Agent: Update local state cache
    end
{{< /mermaid >}}

**Reconciliation Logic:**
1. Fetch all deployment objects targeted at this agent
2. Compare with previously applied objects (using sequence IDs)
3. Apply new or updated objects
4. Delete objects marked with deletion markers
5. Report events for each operation

### Deletion Flow

Deleting resources uses a "deletion marker" pattern for reliable cleanup:

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
    Agent->>Broker: GET /api/v1/agent/deployment-objects
    Broker-->>Agent: Includes deletion marker

    Agent->>Agent: Detect deletion marker
    Agent->>K8s: DELETE resource
    K8s-->>Agent: Deleted

    Agent->>Broker: POST /api/v1/agent/events
    Note over Broker: Event type: DELETED

    Note over DB: Both original and marker<br/>remain for audit trail
{{< /mermaid >}}

## Event Flow

### Agent Event Reporting

Events flow from agents through the broker to external systems:

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
        Webhook[Webhook Dispatcher]
        Audit[Audit Logger]
    end

    subgraph External
        Endpoints[Webhook Endpoints]
        Logs[Audit Logs]
    end

    Apply --> Report
    Report -->|POST /agent/events| API
    API --> DB
    API --> EventBus
    EventBus --> Webhook
    EventBus --> Audit
    Webhook --> Endpoints
    Audit --> Logs
{{< /mermaid >}}

### Event Types

| Event Type | Trigger | Data Included |
|------------|---------|---------------|
| `APPLIED` | Resource successfully applied | Resource details, timestamp |
| `UPDATED` | Resource successfully updated | Resource details, changes |
| `DELETED` | Resource successfully deleted | Resource details |
| `FAILED` | Operation failed | Error message, resource details |
| `HEALTH_CHECK` | Periodic health status | Deployment health status |

### Webhook Dispatch

When webhooks are configured, events trigger deliveries:

{{< mermaid >}}
sequenceDiagram
    participant EventBus as Event Bus
    participant Queue as Delivery Queue
    participant Dispatcher as Webhook Dispatcher
    participant Endpoint as External Endpoint
    participant DB as PostgreSQL

    EventBus->>Queue: Enqueue event

    loop Every delivery interval
        Dispatcher->>Queue: Fetch pending deliveries
        Queue-->>Dispatcher: Batch of events

        par For each webhook subscription
            Dispatcher->>Endpoint: POST event payload
            alt Success (2xx)
                Endpoint-->>Dispatcher: 200 OK
                Dispatcher->>DB: Mark delivered
            else Failure
                Endpoint-->>Dispatcher: Error
                Dispatcher->>DB: Increment retry count
                Note over DB: Retry with exponential backoff
            end
        end
    end
{{< /mermaid >}}

**Webhook Configuration:**
- Delivery interval: configurable (default: 5 seconds)
- Batch size: configurable (default: 50)
- Retry policy: exponential backoff with max retries
- Retention: configurable cleanup (default: 7 days)

## Authentication Flows

### PAK (Pre-Authentication Key) Verification

Agents authenticate using PAKs generated during agent creation:

{{< mermaid >}}
sequenceDiagram
    participant Agent as Agent
    participant Broker as Broker API
    participant PAK as PAK Validator
    participant DB as PostgreSQL

    Note over Agent: Agent startup
    Agent->>Agent: Load PAK from config

    Agent->>Broker: GET /api/v1/agent/deployment-objects
    Note over Agent,Broker: Authorization: Bearer {PAK}

    Broker->>PAK: Validate PAK
    PAK->>PAK: Extract agent ID from PAK
    PAK->>PAK: Verify HMAC signature
    PAK->>DB: Lookup agent, verify not revoked
    DB-->>PAK: Agent record

    alt Valid PAK
        PAK-->>Broker: Agent identity
        Broker->>Broker: Continue with request
        Broker-->>Agent: Response
    else Invalid/Revoked PAK
        PAK-->>Broker: Authentication failed
        Broker-->>Agent: 401 Unauthorized
    end
{{< /mermaid >}}

**PAK Structure:**
```
brokkr_BR{base64_agent_id}_{base64_hmac_signature}
```

- Prefix: `brokkr_BR` identifies this as a Brokkr PAK
- Agent ID: UUID encoded in base64
- HMAC: Signature ensuring PAK integrity

### Admin Token Authentication

Admin users authenticate with tokens:

{{< mermaid >}}
sequenceDiagram
    participant Admin as Admin Client
    participant Broker as Broker API
    participant Auth as Auth Middleware
    participant DB as PostgreSQL

    Admin->>Broker: POST /api/v1/admin/...
    Note over Admin,Broker: Authorization: Bearer {token}

    Broker->>Auth: Validate token
    Auth->>DB: Lookup admin token
    DB-->>Auth: Token record (if exists)

    alt Valid Token
        Auth->>Auth: Check permissions
        Auth-->>Broker: Admin identity
        Broker->>Broker: Execute admin operation
        Broker-->>Admin: Response
    else Invalid Token
        Auth-->>Broker: Authentication failed
        Broker-->>Admin: 401 Unauthorized
    end
{{< /mermaid >}}

### Generator Authentication

Generators (CI/CD systems) use API keys:

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

**Generator Permissions:**
- Create/update/delete stacks
- Create deployment objects
- Cannot access admin endpoints
- Cannot manage other generators

## State Transitions

### Deployment Object States

Deployment objects have an implicit lifecycle based on their presence and deletion markers:

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

### Agent Event States

Events capture the outcome of each operation:

| Previous State | Event | New State |
|----------------|-------|-----------|
| Pending | Applied successfully | Active |
| Active | Updated successfully | Active |
| Active | Deletion marker | Pending Delete |
| Pending Delete | Deleted successfully | Deleted |
| Any | Operation failed | Failed (retry on next poll) |

## Work Order Flow

Work orders enable transient operations like container builds:

{{< mermaid >}}
sequenceDiagram
    participant Client as Admin/API
    participant Broker as Broker API
    participant DB as PostgreSQL
    participant Agent as Agent
    participant K8s as Kubernetes
    participant Build as Build System

    Client->>Broker: POST /api/v1/work-orders
    Broker->>DB: INSERT work_order (status: pending)
    Broker-->>Client: Work order created

    Note over Agent: Next polling interval
    Agent->>Broker: GET /api/v1/agent/work-orders
    Broker->>DB: Query pending work orders
    DB-->>Broker: Work orders
    Broker-->>Agent: Work order details

    Agent->>DB: Update status: in_progress
    Agent->>K8s: Create Build resource
    K8s->>Build: Execute build

    loop Monitor progress
        Agent->>K8s: Check build status
        K8s-->>Agent: Build status
        Agent->>Broker: PATCH /work-orders/{id}
        Note over Broker: Update progress
    end

    Build-->>K8s: Build complete
    K8s-->>Agent: Success + artifacts
    Agent->>Broker: PATCH /work-orders/{id}
    Broker->>DB: Update status: completed
    Broker-->>Agent: Acknowledged
{{< /mermaid >}}

**Work Order Types:**
- Container image builds (via Shipwright)
- Diagnostic commands
- Custom operations

## Data Retention

### Immutability Pattern

Brokkr uses an append-only pattern for deployment objects:
- Objects are never modified after creation
- Updates create new versions with incremented sequence IDs
- Deletions create deletion markers
- All history is retained for audit

### Cleanup Policies

| Data Type | Retention | Cleanup Method |
|-----------|-----------|----------------|
| Deployment objects | Permanent | Soft delete only |
| Agent events | Configurable | Background task |
| Webhook deliveries | 7 days (default) | Background task |
| Audit logs | 90 days (default) | Background task |
| Diagnostic results | 1 hour (default) | Background task |

### Sequence IDs

Every deployment object has a monotonically increasing sequence ID:
- Agents track the highest sequence ID they've processed
- On reconnection, agents request objects since their last sequence
- Ensures reliable, ordered delivery of updates
