---
title: "Technical Reference"
description: "Complete technical documentation for Brokkr"
weight: 1
---

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

[View Interactive API Documentation](/openapi)

### API Endpoints Overview

The Brokkr API is organized into the following main sections:

#### Health Check
- `GET /health` - Check the health status of the broker
- `GET /health/ready` - Check if the broker is ready to accept requests

#### Agent Management
- `POST /v1/agents/register` - Register a new agent
- `GET /v1/agents/{agent_id}` - Get agent details
- `DELETE /v1/agents/{agent_id}` - Deregister an agent

#### Deployment Management
- `POST /v1/deployments` - Create a new deployment
- `GET /v1/deployments/{deployment_id}` - Get deployment details
- `PUT /v1/deployments/{deployment_id}` - Update a deployment
- `DELETE /v1/deployments/{deployment_id}` - Delete a deployment

#### Event Management
- `POST /v1/events` - Report a deployment event
- `GET /v1/events/{deployment_id}` - Get events for a deployment

For detailed information about each endpoint, including request/response formats and examples, please refer to the [Interactive API Documentation](/openapi).

## Rust API Documentation

The Brokkr codebase is written in Rust and provides a rich set of APIs for both the Broker and Agent components. You can explore the complete Rust API documentation here:

[View Rust API Documentation](/api)

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

For detailed information about the Rust implementation, including module structure and function documentation, please refer to the [Rust API Documentation](/api).
