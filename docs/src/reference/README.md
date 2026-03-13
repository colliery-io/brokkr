# Technical Reference

This section provides comprehensive technical documentation for Brokkr, including API references, code documentation, and implementation details.

## API Documentation

The Brokkr API provides a comprehensive set of endpoints for managing deployments, agents, and system configuration. You can explore the complete API documentation in two ways:

### Interactive API Documentation

Our interactive Swagger UI provides a complete reference of all available endpoints, including:
- Detailed request/response schemas
- Authentication requirements
- Example requests and responses
- Interactive testing interface

[View Interactive API Documentation](./api/README.md)

### API Endpoints Overview

The Brokkr API is organized into the following main sections:

#### Health Check
- `GET /healthz` - Liveness check
- `GET /readyz` - Readiness check
- `GET /api/v1/health` - Detailed health diagnostics

#### Agent Management
- `POST /api/v1/agents` - Create a new agent
- `GET /api/v1/agents` - List all agents
- `GET /api/v1/agents/{agent_id}` - Get agent details
- `PUT /api/v1/agents/{agent_id}` - Update an agent
- `DELETE /api/v1/agents/{agent_id}` - Delete an agent

#### Stack Management
- `POST /api/v1/stacks` - Create a new stack
- `GET /api/v1/stacks` - List all stacks
- `GET /api/v1/stacks/{stack_id}` - Get stack details
- `PUT /api/v1/stacks/{stack_id}` - Update a stack
- `DELETE /api/v1/stacks/{stack_id}` - Delete a stack

#### Deployment Object Management
- `POST /api/v1/stacks/{stack_id}/deployment-objects` - Create a deployment object
- `GET /api/v1/stacks/{stack_id}/deployment-objects` - List deployment objects

#### Event Management
- `POST /api/v1/events` - Report a deployment event
- `GET /api/v1/events` - List events

For detailed information about each endpoint, including request/response formats and examples, please refer to the [Interactive API Documentation](./api/README.md).

## Rust API Documentation

The Brokkr codebase is written in Rust and provides a rich set of APIs for both the Broker and Agent components. You can explore the complete Rust API documentation here:

[View Rust API Documentation](../api/README.md)

The Rust documentation includes:
- Detailed module and function documentation
- Type definitions and trait implementations
- Code examples and usage patterns
- Implementation details for core components

### Key Components

#### Broker
- API Server implementation
- Database layer
- Event system
- Authentication and authorization

#### Agent
- Kubernetes client
- Broker communication
- State management
- Deployment orchestration

For detailed information about the Rust implementation, including module structure and function documentation, please refer to the [Rust API Documentation](../api/README.md).
