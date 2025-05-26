---
title: "Installation Guide"
weight: 1
---

# Installing Brokkr

This guide will help you install Brokkr and set up your environment.

## System Requirements

### Hardware Requirements
- CPU: 2+ cores recommended
- RAM: 2GB minimum, 4GB recommended
- Disk: 1GB available space

### Software Requirements
- Operating System: Linux, macOS, or Windows
- Rust toolchain (1.8+)
- PostgreSQL database
- Kubernetes cluster (v1.20+)
- `kubectl` CLI tool
- Docker (for container deployments)
- Python 3.x (for development tools)

## Installation Methods

### Building from Source (Recommended)
```bash
# Clone the repository
git clone https://github.com/colliery-io/brokkr.git
cd brokkr

# Build using Cargo
cargo build --release

# The binaries will be available in target/release/
# - brokkr-broker: The central management service
# - brokkr-agent: The Kubernetes cluster agent
```

### Using Docker Development Environment
```bash
# Install Angreal (our development task runner)
pip install angreal

# Start the development environment
angreal local up

# Rebuild specific services as needed
angreal local rebuild broker
angreal local rebuild agent
angreal local rebuild ui
```

## Initial Configuration

### Basic Setup
1. Set up your environment variables:
```bash
# Database configuration
export BROKKR__DATABASE__URL="postgres://brokkr:brokkr@localhost:5432/brokkr"

# Logging configuration
export BROKKR__LOG__LEVEL="debug"

# Agent configuration
export BROKKR__AGENT__BROKER_URL="http://localhost:3000"

# Kubernetes configuration
# Option 1: Use default kubeconfig location
# The agent will look for kubeconfig at ~/.kube/config

# Option 2: Specify a custom kubeconfig path
export BROKKR__AGENT__KUBECONFIG_PATH="/path/to/your/kubeconfig"
```

2. Create an agent and get its PAK:
```bash
# Create a new agent and generate its PAK
curl -X POST http://localhost:3000/api/v1/agents \
  -H "Content-Type: application/json" \
  -d '{"name": "production-agent", "cluster_name": "production"}'

# The response will include the agent's ID and PAK:
# {
#   "id": "uuid-here",
#   "name": "production-agent",
#   "cluster_name": "production",
#   "status": "ACTIVE",
#   "pak": "brokkr_BRxxxxxxxx_yyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyy"
# }
# Save the PAK securely - you'll need it to run the agent
```

3. Set the agent's PAK:
```bash
# Replace <generated_pak> with the PAK from the previous step
export BROKKR__AGENT__PAK="<generated_pak>"
```

4. Start the agent:
```bash
./brokkr-agent start
```

5. Create a new stack:
```bash
curl -X POST http://localhost:3000/api/v1/stacks \
  -H "Content-Type: application/json" \
  -d '{"name": "my-stack", "description": "My first stack"}'

# The response will include the stack's ID:
# {
#   "id": "uuid-here",
#   "name": "my-stack",
#   "description": "My first stack"
# }
```

6. Target the stack with your agent:
```bash
# Replace <agent_id> and <stack_id> with the IDs from the previous steps
curl -X POST http://localhost:3000/api/v1/agents/<agent_id>/targets \
  -H "Content-Type: application/json" \
  -d '{"agent_id": "<agent_id>", "stack_id": "<stack_id>"}'
```

### Verifying the Installation
```bash
# Check if the broker is running
curl http://localhost:3000/health

# Verify agent connection
curl http://localhost:3000/api/v1/agents

# Verify stack targeting
curl http://localhost:3000/api/v1/agents/<agent_id>/targets

# The agent will automatically verify Kubernetes connectivity during startup
# by attempting to list namespaces. If successful, you'll see a log message:
# "Successfully connected to Kubernetes cluster and verified API access"
# If there's an issue, the agent will fail to start with an error message
# about cluster connectivity.

# Test the deployment system by creating a simple namespace
curl -X POST http://localhost:3000/api/v1/stacks/<stack_id>/deployment-objects \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer <admin_pak>" \
  -d '{
    "yaml_content": "apiVersion: v1\nkind: Namespace\nmetadata:\n  name: test-namespace",
    "is_deletion_marker": false
  }'

# Verify the namespace was created
kubectl get namespace test-namespace

# Delete the test namespace using the agent
curl -X POST http://localhost:3000/api/v1/stacks/<stack_id>/deployment-objects \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer <admin_pak>" \
  -d '{
    "yaml_content": "apiVersion: v1\nkind: Namespace\nmetadata:\n  name: test-namespace",
    "is_deletion_marker": true
  }'

# Verify the namespace was deleted
kubectl get namespace test-namespace
```

## Next Steps
- Follow our [Quick Start Guide](../quick-start) to deploy your first application
- Learn about [Basic Concepts](../first-steps) in Brokkr
- Configure [Environment Management](../../how-to/environment-management)

## Troubleshooting

### Common Issues
- **Build Errors**: Ensure Rust toolchain is properly installed
- **Database Connection**: Verify PostgreSQL is running and accessible
- **Agent Connection**: Check broker URL and PAK configuration
- **Kubernetes Access**: Verify kubeconfig is properly configured and accessible
- **Missing Dependencies**: Check system requirements

### Getting Help
- Check our [GitHub Issues](https://github.com/colliery-io/brokkr/issues)
