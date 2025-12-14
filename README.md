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

## Quick Start

### Prerequisites

- Rust 1.8+
- PostgreSQL database
- Docker and Docker Compose
- [Angreal](https://angreal.github.io/) task runner: `pip install angreal`

### Running Locally

```bash
# Clone the repository
git clone https://github.com/your-org/brokkr.git
cd brokkr

# Start development environment
angreal local up

# The broker API will be available at http://localhost:3000
# The admin UI will be available at http://localhost:3001
```

### First Deployment

```bash
# Create a stack
curl -X POST http://localhost:3000/api/v1/stacks \
  -H "Authorization: Bearer <admin-pak>" \
  -H "Content-Type: application/json" \
  -d '{"name": "my-app", "description": "My application stack"}'

# Check agent status
curl http://localhost:3000/api/v1/agents \
  -H "Authorization: Bearer <admin-pak>"
```

## Documentation

Comprehensive documentation is available at [docs/](./docs/) or can be built locally:

```bash
angreal local docs
```

### Documentation Sections

- **[Getting Started](./docs/content/getting-started/)** - Installation and first steps
- **[Tutorials](./docs/content/tutorials/)** - Step-by-step guides
- **[How-To Guides](./docs/content/how-to/)** - Specific task instructions
- **[Explanation](./docs/content/explanation/)** - Architecture and concepts
- **[API Reference](./docs/content/reference/api/)** - Complete API documentation

## Development

### Project Structure

- `crates/brokkr-broker/` - Central management service
- `crates/brokkr-agent/` - Kubernetes cluster agent
- `crates/brokkr-models/` - Shared data models
- `crates/brokkr-utils/` - Common utilities
- `docs/` - Documentation site

### Common Tasks

```bash
# Run tests
angreal tests unit all

# Rebuild a service
angreal local rebuild broker

```

## License

Provided under Elastic License 2.0. See [LICENSE.txt](./LICENSE.txt)

## Contributing

Pull requests and issues are welcome. For commercial use or implementation help, please contact us.
