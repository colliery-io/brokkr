---
title: "Core Concepts"
weight: 1
---

# Core Concepts

## What is Brokkr?

Brokkr is an environment-aware control plane for dynamically distributing Kubernetes objects. It serves as a bridge between your application deployments and your various Kubernetes environments, providing intelligent routing and management of resources based on environment-specific rules and policies.

## Key Components

### Broker
The central control plane component that:
- Manages deployment policies
- Routes requests to appropriate agents
- Maintains the global state
- Handles API requests

### Agent
The environment-specific component that:
- Executes deployments in target environments
- Reports status back to the broker
- Manages local resources
- Handles environment-specific configurations

### Stacks
A collection of Kubernetes resources that:
- Define your application structure
- Specify deployment requirements
- Configure environment variations
- Manage dependencies

### Targets
Environment definitions that:
- Specify where deployments should go
- Define environment-specific parameters
- Set access controls
- Configure routing rules

## Design Principles

### Environment-First
- Every operation is environment-aware
- Configurations are environment-specific
- Resources are isolated by environment
- Promotion paths are clearly defined

### GitOps Compatible
- Configuration as code
- Version controlled
- Declarative specifications
- Automated reconciliation

### Security-Focused
- RBAC integration
- Environment isolation
- Secure communication
- Audit logging

### Scalable Architecture
- Distributed processing
- Horizontal scaling
- Efficient resource usage
- High availability options

## Use Cases

### Multi-Environment Management
- Development/Staging/Production environments
- Regional deployments
- Customer-specific environments
- Edge computing scenarios

### Deployment Automation
- Continuous deployment
- Blue-green deployments
- Canary releases
- Rolling updates

### Configuration Management
- Environment-specific configs
- Secret management
- Resource quotas
- Access controls

### Compliance and Governance
- Audit trails
- Policy enforcement
- Resource tracking
- Compliance reporting

## Next Steps
- Follow our [Quick Start Guide](../../getting-started/quick-start)
- Learn about [Architecture Decisions](../architecture-decisions)
- Explore [Best Practices](../best-practices)
- Read about [Advanced Topics](../advanced-topics)
