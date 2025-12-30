---
title: "Technical Architecture"
weight: 1
---

# Technical Architecture

This document provides detailed technical implementation information about Brokkr's architecture, focusing on performance, scaling, and deployment patterns.

## Implementation Details

### Broker Service Implementation

#### API Server
- Built with Axum web framework
- Async request handling with Tokio runtime
- Middleware-based authentication and authorization
- Request validation using Serde
- OpenAPI/Swagger documentation generation

#### Database Layer
- PostgreSQL with SQLx for async database access
- Connection pooling with deadpool
- Transaction management with automatic rollback
- Optimistic locking for concurrent updates
- Prepared statement caching

#### Event System
- In-memory event bus for internal communication
- WebSocket-based real-time updates
- Event batching for performance
- Event persistence for audit trail
- Event replay capabilities

### Agent Service Implementation

#### Kubernetes Client
- Dynamic client generation
- Resource watching with exponential backoff
- Custom resource definition support
- Resource validation using OpenAPI schemas
- Conflict resolution strategies

#### State Management
- In-memory state cache
- Periodic state reconciliation
- Optimistic concurrency control
- State persistence for recovery
- State diff calculation

## Component Architecture

This section provides visual diagrams showing how components interact within each service and across the system.

### Broker Internal Architecture

The broker service consists of several interconnected components:

{{< mermaid >}}
flowchart TB
    subgraph API["API Layer (Axum)"]
        Routes[Routes & Handlers]
        Middleware[Auth Middleware]
        OpenAPI[OpenAPI Docs]
    end

    subgraph Core["Core Services"]
        DAL[Data Access Layer]
        EventBus[Event Bus]
        AuditLog[Audit Logger]
    end

    subgraph Background["Background Services"]
        ConfigWatch[Config Watcher]
        Cleanup[Cleanup Tasks]
        WebhookDispatch[Webhook Dispatcher]
    end

    subgraph Utils["Utilities"]
        PAK[PAK Validator]
        Encryption[Encryption]
        Matching[Agent Matching]
        Templating[Template Engine]
    end

    DB[(PostgreSQL)]

    Routes --> Middleware
    Middleware --> PAK
    Routes --> DAL
    Routes --> EventBus
    DAL --> DB
    EventBus --> WebhookDispatch
    EventBus --> AuditLog
    AuditLog --> DAL
    ConfigWatch --> DAL
    Cleanup --> DAL
    Templating --> DAL
{{< /mermaid >}}

**Component Responsibilities:**

| Component | Purpose |
|-----------|---------|
| **API Layer** | HTTP request handling, authentication, OpenAPI documentation |
| **DAL** | Database operations for all entities (agents, stacks, deployments, etc.) |
| **Event Bus** | Internal pub/sub for decoupled event handling |
| **Audit Logger** | Async audit trail with batched writes |
| **Background Services** | Config reload, cleanup tasks, webhook delivery |
| **Utilities** | PAK validation, encryption, agent matching, templating |

### Agent Internal Architecture

The agent service manages Kubernetes resources based on broker instructions:

{{< mermaid >}}
flowchart TB
    subgraph BrokerComm["Broker Communication"]
        Client[Broker Client]
        Poller[Polling Loop]
        EventReporter[Event Reporter]
    end

    subgraph K8s["Kubernetes Operations"]
        K8sAPI[K8s API Client]
        Objects[Object Manager]
        Reconciler[Reconciler]
    end

    subgraph Health["Health & Diagnostics"]
        HealthCheck[Health Endpoints]
        DeployHealth[Deployment Health]
        Diagnostics[Diagnostics Runner]
    end

    subgraph WorkOrders["Work Orders"]
        WOExecutor[Work Order Executor]
        WOReporter[Result Reporter]
    end

    Broker[Broker API]
    Cluster[Kubernetes Cluster]

    Poller --> Client
    Client --> Broker
    Client --> Reconciler
    Reconciler --> Objects
    Objects --> K8sAPI
    K8sAPI --> Cluster
    Reconciler --> EventReporter
    EventReporter --> Client
    DeployHealth --> K8sAPI
    DeployHealth --> EventReporter
    Diagnostics --> K8sAPI
    WOExecutor --> K8sAPI
    WOExecutor --> WOReporter
    WOReporter --> Client
{{< /mermaid >}}

**Component Responsibilities:**

| Component | Purpose |
|-----------|---------|
| **Broker Client** | REST API communication with broker, PAK authentication |
| **Polling Loop** | Periodic fetch of deployment objects and work orders |
| **Reconciler** | Compares desired vs actual state, applies changes |
| **K8s API Client** | Dynamic Kubernetes client for resource operations |
| **Event Reporter** | Reports deployment events back to broker |
| **Work Order Executor** | Executes transient operations (builds, commands) |
| **Deployment Health** | Monitors health of deployed resources |

### Cross-Service Interactions

This diagram shows how the broker and agents communicate:

{{< mermaid >}}
sequenceDiagram
    participant Admin as Admin/Generator
    participant Broker as Broker
    participant DB as PostgreSQL
    participant Agent as Agent
    participant K8s as Kubernetes

    Note over Admin,K8s: Deployment Creation Flow
    Admin->>Broker: Create Stack & Deployment Object
    Broker->>DB: Store deployment object
    Broker->>Broker: Match agents via labels
    Broker->>DB: Create agent targets

    Note over Admin,K8s: Agent Reconciliation Loop
    loop Every polling interval
        Agent->>Broker: GET /api/v1/agent/deployment-objects
        Broker->>DB: Query targeted objects
        Broker-->>Agent: Return deployment objects
        Agent->>Agent: Calculate diff (desired vs actual)
        Agent->>K8s: Apply/Update/Delete resources
        K8s-->>Agent: Operation result
        Agent->>Broker: POST /api/v1/agent/events
        Broker->>DB: Store events
        Broker->>Broker: Trigger webhooks
    end

    Note over Admin,K8s: Status Reporting
    Agent->>K8s: Watch resource status
    K8s-->>Agent: Status updates
    Agent->>Broker: Report deployment health
    Broker->>DB: Update health status
{{< /mermaid >}}

**Key Interaction Patterns:**

1. **Push Configuration**: Admins/generators push deployment configurations to the broker
2. **Pull Deployment**: Agents poll the broker for their targeted deployment objects
3. **Apply Resources**: Agents apply resources to their Kubernetes cluster
4. **Report Events**: Agents report success/failure events back to the broker
5. **Webhook Dispatch**: Broker dispatches events to configured webhook endpoints

## Performance Characteristics

### Broker Performance

#### API Performance
- Request latency: < 50ms for 95th percentile
- Concurrent request handling: 1000+ requests/second
- WebSocket connections: 1000+ concurrent
- Event processing: 5000+ events/second

#### Database Performance
- Query latency: < 20ms for 95th percentile
- Connection pool: 20-100 connections
- Transaction throughput: 1000+ TPS
- Cache hit ratio: > 90%

### Agent Performance

#### Resource Application
- Resource application latency: < 100ms
- Concurrent resource updates: 100+
- State reconciliation: < 1s
- Event reporting: < 50ms

#### Kubernetes API
- API request latency: < 200ms
- Watch connection stability: 99.9%
- Resource validation: < 50ms
- Conflict resolution: < 100ms

## Scaling Patterns

### Horizontal Scaling

#### Broker Scaling
```mermaid
graph TD
    LB[Load Balancer] --> B1[Broker 1]
    LB --> B2[Broker 2]
    LB --> B3[Broker 3]
    B1 --> DB[(Database)]
    B2 --> DB
    B3 --> DB
```

- Stateless design enables horizontal scaling
- Shared database for state persistence
- Load balancer for request distribution
- Session affinity for WebSocket connections
- Distributed caching with Redis

#### Agent Scaling
```mermaid
graph TD
    B[Broker] --> A1[Agent 1]
    B --> A2[Agent 2]
    B --> A3[Agent 3]
    A1 --> K1[Cluster 1]
    A2 --> K2[Cluster 2]
    A3 --> K3[Cluster 3]
```

- One agent per cluster
- Independent operation
- No inter-agent communication
- Local state management
- Cluster-specific configuration

### Vertical Scaling

#### Resource Requirements

| Component | CPU | Memory | Storage |
|-----------|-----|--------|---------|
| Broker    | 2-4 cores | 4-8GB | 20GB |
| Agent     | 1-2 cores | 2-4GB | 10GB |
| Database  | 4-8 cores | 8-16GB | 100GB+ |

#### Scaling Triggers
- CPU utilization > 70%
- Memory utilization > 80%
- Request latency > 100ms
- Connection count > 1000
- Event queue length > 1000

## Deployment Patterns

### High Availability

#### Broker HA
```mermaid
graph TD
    LB[Load Balancer] --> B1[Broker 1]
    LB --> B2[Broker 2]
    B1 --> DB1[(Primary DB)]
    B2 --> DB1
    DB1 --> DB2[(Replica DB)]
```

- Active-active deployment
- Database replication
- Automatic failover
- Health checking
- Graceful shutdown

#### Agent HA
```mermaid
graph TD
    B[Broker] --> A1[Agent 1]
    B --> A2[Agent 2]
    A1 --> K[Cluster]
    A2 --> K
```

- Active-passive deployment
- Leader election
- State synchronization
- Automatic failover
- Health monitoring

### Disaster Recovery

#### Backup Strategy
- Database backups every 6 hours
- Point-in-time recovery
- Cross-region replication
- Backup verification
- Recovery testing

#### Recovery Procedures
1. Database restoration
2. Broker service recovery
3. Agent reconnection
4. State reconciliation
5. Health verification

## Monitoring and Debugging

### Metrics Collection

#### Broker Metrics
- Request latency
- Error rates
- Connection counts
- Queue lengths
- Cache statistics
- Database performance
- Event processing

#### Agent Metrics
- Resource application
- State reconciliation
- API latency
- Error rates
- Memory usage
- CPU utilization
- Network I/O

### Logging

#### Log Levels
- ERROR: System errors
- WARN: Potential issues
- INFO: Normal operation
- DEBUG: Detailed debugging
- TRACE: Verbose tracing

#### Log Format
```json
{
  "timestamp": "2024-03-14T12:00:00Z",
  "level": "INFO",
  "component": "broker",
  "trace_id": "abc123",
  "message": "Request processed",
  "metadata": {
    "request_id": "xyz789",
    "duration_ms": 45,
    "status": "success"
  }
}
```

### Tracing

#### Trace Points
- Request handling
- Database operations
- Event processing
- Resource application
- State reconciliation
- Error handling

#### Trace Context
- Request ID
- Parent span
- Component
- Operation
- Duration
- Status

## Troubleshooting Guide

### Common Issues

#### Broker Issues
1. **High Latency**
   - Check database performance
   - Monitor connection pool
   - Verify cache hit ratio
   - Check system resources

2. **Connection Issues**
   - Verify network connectivity
   - Check firewall rules
   - Monitor connection limits
   - Review error logs

3. **Database Issues**
   - Check connection pool
   - Monitor query performance
   - Verify transaction logs
   - Review error messages

#### Agent Issues
1. **Resource Application**
   - Check Kubernetes API access
   - Verify resource validation
   - Monitor state reconciliation
   - Review error logs

2. **State Management**
   - Check state cache
   - Verify state persistence
   - Monitor reconciliation
   - Review error messages

3. **Communication Issues**
   - Verify broker connectivity
   - Check authentication
   - Monitor heartbeat
   - Review error logs

### Debugging Tools

#### Broker Tools
- API debugging
- Database inspection
- Event tracing
- Performance profiling
- Memory analysis

#### Agent Tools
- Resource inspection
- State analysis
- API debugging
- Performance profiling
- Memory analysis

## Performance Tuning

### Broker Tuning

#### API Server
- Thread pool size
- Connection limits
- Request timeout
- Buffer sizes
- Cache settings

#### Database
- Connection pool size
- Statement cache
- Transaction timeout
- Batch size
- Index optimization

### Agent Tuning

#### Kubernetes Client
- Watch timeout
- Retry settings
- Batch size
- Cache settings
- Resource limits

#### State Management
- Cache size
- Reconciliation interval
- Batch size
- Persistence interval
- Cleanup settings
