---
title: "Core Concepts"
weight: 1
---

## What is Brokkr?

Brokkr is an environment-aware control plane for dynamically distributing Kubernetes objects. Think of it as a smart traffic controller for your Kubernetes resources - it knows not just what to deploy, but where and when to deploy it based on your environment's specific needs and policies.

{{< mermaid >}}
graph LR
    subgraph "Control Plane"
        UA[User/Admin] -->|Creates/Updates| BR[Broker]
    end

    subgraph "Agents"
        AG[Agent]
    end

    subgraph "Kubernetes Clusters"
        KC[K8s Cluster]
    end

    AG -- Fetches Target State --> BR
    AG -- Reports Status --> BR
    AG -- Applies --> KC
{{< /mermaid >}}

*Note: This diagram shows a single agent and cluster for clarity. In real deployments, Brokkr supports multiple agents and clusters, each following the same pattern.*

---
# Key Components
### The Broker: The Source of Truth

The Broker is the source of truth for Brokkr. It records the desired state of your applications and environments, and provides APIs for users and agents to interact with this state.

The broker does not directly control clusters or agents. Instead, agents regularly poll the broker to fetch their target state and report back on what has been applied. The broker's responsibilities include:
- Recording what should be deployed and where (desired state)
- Providing RESTful APIs for users and agents
- Authenticating and authorizing API requests
- Recording events and status reports from agents

All orchestration and application of resources is performed by the agents, not the broker.


### The Agent: The Executor

Agents are responsible for making Brokkr's desired state a reality in your Kubernetes clusters. Each agent runs in a specific environment (typically a cluster) and is responsible for:
- Polling the broker to fetch its target state (what should be deployed)
- Validating and applying Kubernetes resources to its cluster
- Reporting status and events back to the broker

Agents do not make deployment decisions or manage global state. They simply execute the instructions recorded in the broker, ensuring that their cluster matches the desired state. All validation, application, and reconciliation of resources is performed locally by the agent.

---
# Internal Data Architecture

Brokkr's internal data model is designed to track what should be deployed, where, and by whom, while providing a clear audit trail of what has actually happened. The following are the core data entities and their relationships:

---

### Stacks

A **Stack** is simply a collection of related Kubernetes objects that are managed as a unit. Stacks are used to group resources for versioning, deployment, and organizational purposes. There is no enforced structure or semantics beyond this grouping.

---

### Deployment Objects

A **Deployment Object** is a versioned snapshot of the full set of Kubernetes resources in a Stack. Each time a Stack is updated, a new Deployment Object is created, capturing the desired state at that point in time. Deployment Objects are immutable and provide a historical record of changes.

---

### Agents

An **Agent** represents a Brokkr process running in a specific environment (typically a Kubernetes cluster). Agents are responsible for polling the broker, fetching their assigned target state, applying resources to their cluster, and reporting back status and events.

---

### Agent Targets

An **Agent Target** is an association between an Agent and a Stack. It defines which Stacks an Agent is responsible for managing. This mapping allows Brokkr to distribute workloads across multiple clusters and environments.

---

### Agent Events

An **Agent Event** records the outcome of an Agent's attempt to apply a Deployment Object. Events capture both successes and failures, providing an audit trail for troubleshooting and compliance.

---

### Defining Agent Targets

Brokkr supports several mechanisms for associating agents with stacks ("agent targets"). This flexibility allows you to control which agents are responsible for which stacks, supporting a variety of deployment and organizational models.

#### 1. Direct Assignment (One-to-One)
- An agent is explicitly assigned to a stack by ID.
- This is the simplest and most direct method.
- Example: Assigning a specific agent to manage a specific stack.

#### 2. Label-Based Targeting (One-to-Many/Many-to-Many)
- Agents and stacks can be tagged with labels.
- An agent can be configured to target all stacks with a matching label, or vice versa.
- This enables dynamic and scalable targeting, such as "all production agents manage all production stacks."

#### 3. Annotation-Based Targeting
- Similar to labels, but with key-value pairs that can encode more complex rules or metadata.
- Useful for advanced scenarios where targeting logic depends on more than just a label match.

| Targeting Method      | Use Case Example                        |
|----------------------|-----------------------------------------|
| Direct Assignment    | Agent A manages Stack X                 |
| Label-Based          | All agents labeled "prod" manage all stacks labeled "prod" |
| Annotation-Based     | Agents with region=us-east manage stacks with region=us-east |

---

## How These Pieces Fit Together

1. **Stacks** are created by users to group related Kubernetes objects.
2. Each Stack has one or more **Deployment Objects** (versioned snapshots of the stack's resources).
3. **Agents** are registered and assigned responsibility for one or more Stacks via **Agent Targets**.
4. When an Agent polls the broker, it receives the latest Deployment Object(s) for its assigned Stacks.
5. The Agent applies the resources and reports the outcome as **Agent Events**.

{{< mermaid >}}
erDiagram
    STACK ||--o{ DEPLOYMENT_OBJECT : has
    AGENT ||--o{ AGENT_TARGET : assigned_to
    STACK ||--o{ AGENT_TARGET : targeted_by
    DEPLOYMENT_OBJECT ||--o{ AGENT_EVENT : triggers
    AGENT ||--o{ AGENT_EVENT : reports
{{< /mermaid >}}

This architecture allows Brokkr to provide a clear, auditable, and scalable way to manage Kubernetes resources across many environments.

---

## The Deployment Journey

The deployment process in Brokkr follows a pull-based model, with agents responsible for fetching, validating, and applying their assigned target state. The broker acts as a source of truth and event recorder, but does not push deployments or perform environment-specific validation.

1. **Stack Creation:**
   The user creates or updates a stack, which results in a new deployment object (versioned snapshot) being created in the broker.

2. **Agent Polling:**
   Each agent regularly polls the broker for its target state (the latest deployment objects for its assigned stacks).

3. **Validation & Application:**
   The agent validates the deployment object(s) locally (e.g., checks YAML, resource constraints) and applies the resources to its cluster.

4. **Event Reporting:**
   The agent reports the outcome (success or failure) of each deployment object application back to the broker as an event.

5. **Audit & History:**
   The broker records these events, providing an audit trail and deployment history.

{{< mermaid >}}
sequenceDiagram
    participant User
    participant Broker
    participant Agent
    participant Cluster

    User->>Broker: Create/Update Stack (creates Deployment Object)
    loop Every polling interval
        Agent->>Broker: Fetch Target State (Deployment Objects)
        Broker-->>Agent: Return Deployment Objects
        Agent->>Agent: Validate & Apply Resources
        Agent->>Cluster: Apply Resources
        Cluster-->>Agent: Result
        Agent->>Broker: Report Event (Success/Failure)
    end
{{< /mermaid >}}

## Security Model

Brokkr uses a strict API key (PAK: Prefixed API Key) authentication and role-based authorization model for all API access.

### Authentication

- **All API requests require a valid PAK** in the `Authorization` header.
- There are three types of PAKs:
  - **Admin PAK:** Grants full administrative access to all API endpoints.
  - **Agent PAK:** Grants access to endpoints and data relevant to a specific agent (e.g., fetching target state, reporting events).
  - **Generator PAK:** Grants access to endpoints for resource generators (e.g., creating deployment objects).

- **PAK Verification:**
  - The API middleware extracts the PAK from the request header.
  - The PAK is checked against the stored hashes for admins, agents, and generators.
  - If the PAK is valid, the request is allowed to proceed with the associated role and identity; otherwise, it is rejected with an unauthorized error.

### Authorization

- **Role-based access control** is enforced at the endpoint level:
  - **Admin-only endpoints:** Creating agents, listing all agents, managing generators, etc.
  - **Agent endpoints:** Only accessible by the agent's own PAK (e.g., fetching its target state, reporting events).
  - **Generator endpoints:** Only accessible by the generator's own PAK.
  - **Resource access:** Agents and generators can only access resources (e.g., deployment objects, stacks) they are associated with.

- **Fine-grained checks** are performed in the middleware and handlers to ensure that:
  - Agents cannot access or modify resources belonging to other agents.
  - Generators cannot access or modify resources belonging to other generators.
  - Only admins can perform global or cross-entity operations.

### Key Management

- **PAKs are generated and rotated** using secure random generation and hashing.
- **PAK hashes** are stored in the database; the actual PAK is only shown once at creation/rotation.
- **PAK rotation** endpoints are available for both agents and generators, requiring either admin or self-authentication.

### Example Flow

1. **Request:**  Client sends a request with `Authorization: Bearer <PAK>`.
2. **Middleware:**  Extracts and verifies the PAK, determines the role (admin, agent, generator) and identity, and attaches this information to the request context.
3. **Handler:**  Checks if the role/identity is authorized for the requested operation, then proceeds or returns a forbidden/unauthorized error.

#### Row-Based Access Control

In addition to endpoint-level authorization, Brokkr enforces row-based access control. After authenticating the request, the API checks whether the requesting entity (admin, agent, or generator) is permitted to access or modify each specific resource, based on ownership or association. For example, an agent can only fetch deployment objects for stacks it is assigned to.

{{< mermaid >}}
sequenceDiagram
    participant Client
    participant API
    participant DB

    Client->>API: Request (with PAK)
    API->>API: Authenticate PAK
    API->>API: Determine role/identity
    API->>DB: Query resource (with access check)
    alt Access allowed
        DB-->>API: Resource data
        API-->>Client: Success/Resource
    else Access denied
        API-->>Client: Forbidden/Unauthorized
    end
{{< /mermaid >}}

## Next Steps

Now that you understand the core concepts of Brokkr, you can:
- Follow our [Quick Start Guide](../../getting-started/quick-start) to try it out
- Learn about [Architecture Decisions](../architecture-decisions) to understand why we made these choices
- Explore [Best Practices](../best-practices) to get the most out of Brokkr
- Read about [Advanced Topics](../advanced-topics) to dive deeper
