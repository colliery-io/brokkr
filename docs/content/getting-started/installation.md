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
- PostgreSQL database (v12+)
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

## Initial Setup

### 1. Database Setup
1. Create a PostgreSQL database:
```bash
# Create the database and user
createdb brokkr
createuser -P brokkr  # You'll be prompted for a password
```

2. Set the database URL:
```bash
export BROKKR__DATABASE__URL="postgres://brokkr:brokkr@localhost:5432/brokkr"
```

### 2. Broker Setup
1. Start the broker:
```bash
./brokkr-broker serve
```

2. The broker will automatically:
   - Run database migrations
   - Create the initial admin role
   - Generate the first admin PAK

3. Save the admin PAK securely - you'll need it for administrative operations.

### 3. Agent Setup
1. Create an agent and get its PAK:
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
```

2. Configure the agent:
```bash
# Set the agent's PAK
export BROKKR__AGENT__PAK="<generated_pak>"

# Set the broker URL
export BROKKR__AGENT__BROKER_URL="http://localhost:3000"

# Set the agent name and cluster name
export BROKKR__AGENT__NAME="production-agent"
export BROKKR__AGENT__CLUSTER_NAME="production"

# Optional: Set a custom kubeconfig path
export BROKKR__AGENT__KUBECONFIG_PATH="/path/to/your/kubeconfig"
```

3. Start the agent:
```bash
./brokkr-agent start
```

### 4. Stack Setup
1. Create a new stack:
```bash
curl -X POST http://localhost:3000/api/v1/stacks \
  -H "Content-Type: application/json" \
  -d '{"name": "my-stack", "description": "My first stack"}'
```

2. Target the stack with your agent:
```bash
# Replace <agent_id> and <stack_id> with the IDs from the previous steps
curl -X POST http://localhost:3000/api/v1/agents/<agent_id>/targets \
  -H "Content-Type: application/json" \
  -d '{"agent_id": "<agent_id>", "stack_id": "<stack_id>"}'
```

## Verifying the Installation

### 1. Check Broker Health
```bash
# Check if the broker is running
curl http://localhost:3000/healthz

# Verify agent connection
curl http://localhost:3000/api/v1/agents
```

### 2. Verify Agent Connection
```bash
# The agent will automatically verify Kubernetes connectivity during startup
# by attempting to list namespaces. If successful, you'll see a log message:
# "Successfully connected to Kubernetes cluster and verified API access"
```

### 3. Test Deployment
```bash
# Create a test namespace
curl -X POST http://localhost:3000/api/v1/stacks/<stack_id>/deployment-objects \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer <admin_pak>" \
  -d '{
    "yaml_content": "apiVersion: v1\nkind: Namespace\nmetadata:\n  name: test-namespace",
    "is_deletion_marker": false
  }'

# Verify the namespace was created
kubectl get namespace test-namespace

# Clean up the test namespace
curl -X POST http://localhost:3000/api/v1/stacks/<stack_id>/deployment-objects \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer <admin_pak>" \
  -d '{
    "yaml_content": "apiVersion: v1\nkind: Namespace\nmetadata:\n  name: test-namespace",
    "is_deletion_marker": true
  }'
```

## Next Steps
- Follow our [Quick Start Guide](quick-start) to deploy your first application
- Learn about [Basic Concepts](../explanation/core-concepts) in Brokkr
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
- Join our [Discord Community](https://discord.gg/brokkr)
- Read the [Troubleshooting Guide](../../how-to/troubleshoot)
