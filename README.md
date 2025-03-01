# Brokkr

Brokkr is an environment-aware control plane for dynamically distributing Kubernetes objects across multiple clusters.

## Overview

Brokkr allows you to define stacks of Kubernetes resources and intelligently distribute them across connected Kubernetes clusters based on environmental factors and policies. It consists of a central broker service that orchestrates resources and agents running on target clusters that apply the resources.

### Key Features

- **Dynamic Resource Distribution**: Distribute Kubernetes objects across clusters based on policy
- **Environment-Aware**: Make deployment decisions based on cluster status and capabilities
- **Centralized Management**: Manage multiple Kubernetes clusters from a single control plane
- **Agent-Based Architecture**: Lightweight agents connect clusters to the control plane
- **API-First Design**: RESTful API for integration with external tools and systems
- **Extensible Generator Framework**: Support for custom resource generation workflows

## Architecture

Brokkr follows a microservices architecture with the following components:

- **Broker**: Central control plane that manages resources, agents, and distribution policies
- **Agents**: Lightweight services running on target Kubernetes clusters
- **Models**: Shared data models and schemas
- **Utilities**: Common utilities for logging, configuration, and error handling
- **UI**: Administrative interface for managing the system

```mermaid
graph TD
    A[Admin/API Users] --> B[Broker]
    B --> C[Database]
    B <--> D[Agent 1]
    B <--> E[Agent 2]
    B <--> J[Agent 2a]
    B <--> F[Agent N]
    D --> G[Kubernetes Cluster 1]
    E --> H[Kubernetes Cluster 2]
    J --> H[Kubernetes Cluster 2]
    F --> I[Kubernetes Cluster N]
```

## Message Flow

Brokkr operates using a message-based system for distributing resources across clusters. The following sequence diagrams illustrate the flow of messages in different scenarios.

### Agent-Broker Communication

This diagram shows the communication flow between an agent and the broker:

```mermaid
sequenceDiagram
    participant Agent
    participant Broker
    participant K8s as Kubernetes Cluster

    Agent->>Broker: Health Check (/readyz)
    Broker-->>Agent: 200 OK

    Agent->>Broker: Verify PAK (Authentication)
    Broker-->>Agent: PAK Status

    loop Every polling interval
        Agent->>Broker: Send Heartbeat
        Broker-->>Agent: Acknowledge Heartbeat

        Agent->>Broker: Fetch Deployments (target-state)
        Broker-->>Agent: Deployment Objects

        loop For each deployment object
            Agent->>K8s: Validate Objects
            K8s-->>Agent: Validation Result

            Agent->>K8s: Apply Objects
            K8s-->>Agent: Apply Result

            alt Success
                Agent->>Broker: Report Success Event
            else Failure
                Agent->>Broker: Report Failure Event
            end
        end
    end
```

### Client-Broker API Interaction

This diagram illustrates how users or external systems interact with the broker API:

```mermaid
sequenceDiagram
    participant Client as Client/Admin
    participant API as Broker API
    participant DB as Database
    participant Agent

    Client->>API: Authenticate (PAK)
    API-->>Client: Authentication Response

    Client->>API: Create/Update Stack
    API->>DB: Store Stack Data
    DB-->>API: Confirmation
    API-->>Client: Stack Created/Updated

    Client->>API: Define Deployment Objects
    API->>DB: Store Deployment Objects
    DB-->>API: Confirmation
    API-->>Client: Objects Stored

    Client->>API: Create/Configure Agent
    API->>DB: Store Agent Configuration
    DB-->>API: Confirmation
    API-->>Client: Agent PAK & Configuration

    Note over Agent,API: Agent connects using PAK

    Client->>API: Query Agent Status
    API->>DB: Fetch Agent Status
    DB-->>API: Agent Status Data
    API-->>Client: Agent Status Response
```

### End-to-End Deployment Flow

This diagram shows the complete flow of a deployment from creation to application in a Kubernetes cluster:

```mermaid
sequenceDiagram
    participant User as User/Admin
    participant Broker
    participant DB as Database
    participant Agent
    participant K8s as Kubernetes Cluster

    User->>Broker: Create Deployment Objects
    Broker->>DB: Store Deployment Objects
    DB-->>Broker: Confirmation
    Broker-->>User: Objects Created

    User->>Broker: Associate Objects with Agent(s)
    Broker->>DB: Update Object Assignments
    DB-->>Broker: Confirmation
    Broker-->>User: Assignment Confirmed

    loop Agent polling
        Agent->>Broker: Send Heartbeat
        Broker-->>Agent: Acknowledge

        Agent->>Broker: Request Target State
        Broker->>DB: Fetch Assigned Objects
        DB-->>Broker: Deployment Objects
        Broker-->>Agent: Deployment Objects

        loop For each object
            Agent->>K8s: Validate Kubernetes Resources
            K8s-->>Agent: Validation Result

            Agent->>K8s: Apply Resources
            K8s-->>Agent: Application Result

            alt Success
                Agent->>Broker: Report Success Event
                Broker->>DB: Record Success
                DB-->>Broker: Confirmation
            else Failure
                Agent->>Broker: Report Failure Event
                Broker->>DB: Record Failure
                DB-->>Broker: Confirmation
            end
        end
    end

    User->>Broker: Query Deployment Status
    Broker->>DB: Fetch Status
    DB-->>Broker: Status Data
    Broker-->>User: Deployment Status Report
```

## Getting Started

### Prerequisites

- Rust 1.8+
- PostgreSQL database
- Access to Kubernetes clusters
- Docker (for containerized deployment)

### Environment Setup

#### Required Tools

Brokkr uses [Angreal](https://angreal.github.io/) as a task runner and project management tool to streamline development workflows.

- **Angreal (for development workflow)**
  ```bash
  pip install angreal
  ```

#### Required Libraries

- **macOS**
  ```bash
  brew install libpq

  # you might need to do this if diesel fails
  export LDFLAGS="-L/opt/homebrew/opt/libpq/lib"
  export CPPFLAGS="-I/opt/homebrew/opt/libpq/include"
  export PATH="/opt/homebrew/opt/libpq/bin:$PATH"
  ```

- **Linux**
  ```bash
  sudo apt-get install libpq-dev
  ```

#### Rust Toolchain Setup

```bash
cargo install cargo-binstall
cargo binstall diesel_cli
```

### Development Environment

Angreal provides several commands to help manage the development environment:

```bash
# Bring up backing services (PostgreSQL, K3s)
angreal local up

# Bring down backing services
angreal local down

# Reset backing services (down and up)
angreal local reset

# Clean up backing services and volumes
angreal local clean

# Generate project documentation
angreal local docs

# Rebuild a specific service (broker, agent, ui)
angreal local rebuild broker
```

### Building from Source

1. Clone the repository
   ```bash
   git clone https://github.com/your-org/brokkr.git
   cd brokkr
   ```

2. Build the project
   ```bash
   cargo build --release
   ```

### Running with Docker

Brokkr uses Docker Compose to run the complete development environment with the broker, agents, database, and a K3s cluster for testing:

```bash
# Start the development environment
angreal local up

# Rebuild and restart a specific service
angreal local rebuild broker
angreal local rebuild agent
angreal local rebuild ui
```

## Configuration

Brokkr uses environment variables and configuration files for setup:

| Variable | Description | Default |
|----------|-------------|---------|
| `DATABASE_URL` | PostgreSQL connection URL | `postgres://postgres:postgres@localhost/brokkr` |
| `BROKER_URL` | URL for agents to connect to broker | `http://localhost:8080` |
| `LOG_LEVEL` | Logging level | `info` |

## Usage

### Setting Up a Stack

Stacks represent collections of Kubernetes resources that should be deployed together.

```bash
# Create a new stack using the broker CLI
./brokkr-broker create stack --name my-stack --description "My first stack"

# Or use the API to create a stack
curl -X POST http://localhost:3000/api/v1/stacks -H "Content-Type: application/json" -d '{"name": "my-stack", "description": "My first stack"}'
```

### Connecting an Agent

Agents connect to the broker and apply changes to their respective Kubernetes clusters.

```bash
# Create a new agent using the broker CLI
./brokkr-broker create agent --name production-agent --cluster-name production

# Run the agent with the generated PAK
export BROKKR__AGENT__PAK=<generated_pak>
export BROKKR__AGENT__BROKER_URL=http://localhost:3000
./brokkr-agent start
```

### Working with Models

Angreal provides tasks for working with database models:

```bash
# Generate schema.rs from database migrations
angreal models schema

# Run migration tests
angreal models migrations

# Test database models
angreal models test
```

## Development

### Project Structure

The project is organized as a Rust workspace with several crates:

- `brokkr-broker`: The central management service
- `brokkr-agent`: The Kubernetes cluster agent
- `brokkr-models`: Shared data models
- `brokkr-utils`: Common utilities and helpers

### Running Tests

Angreal provides comprehensive test commands:

```bash
# Run unit tests for a specific crate
angreal tests unit brokkr-broker

# Run unit tests for all crates
angreal tests unit all

# Run integration tests for a specific crate
angreal tests integration brokkr-broker

# Run integration tests for all crates
angreal tests integration all --skip-docker
```

## License

Provided under Elastic License 2.0. 

## Contributing

Pull requests / Issues are welcome if you have a new feature you'd like to provide. 
If you're looking for specific help in implementing or permission to use commercially, contact me to discuss. 

## Roadmap

