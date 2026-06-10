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

The full endpoint catalog — every route, method, and authorization requirement — is maintained in the [API Reference](./api/README.md), with an interactive Swagger UI served by the broker at `/swagger-ui` and the OpenAPI spec at `/docs/openapi.json`.

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
