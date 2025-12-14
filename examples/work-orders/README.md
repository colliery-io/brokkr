# Work Order Examples

This directory contains example YAML specifications for Brokkr work orders.

## Overview

Work orders are transient operations (builds, tests, backups) that Brokkr routes to agents for execution. Unlike deployment objects which represent persistent state, work orders are one-time operations.

## Directory Structure

```
work-orders/
  README.md                    # This file
  simple-build.yaml           # Basic container build example
  build-with-args.yaml        # Build with custom arguments
  private-repo-build.yaml     # Build from private Git repository
```

## Usage

### Creating a Work Order via API

```bash
# Create a work order from the example files
curl -X POST http://localhost:3000/api/v1/work-orders \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer <ADMIN_PAK>" \
  -d '{
    "work_type": "build",
    "yaml_content": "<contents of example yaml file>",
    "targeting": {
      "labels": ["env=dev"]
    }
  }'
```

### Targeting Options

Work orders can be targeted to agents using:

1. **Direct agent IDs**: Specific agents by UUID
2. **Labels**: Agents with matching labels (OR logic)
3. **Annotations**: Agents with matching annotations (OR logic)

Example targeting configurations:

```json
// Target specific agents
{
  "targeting": {
    "agent_ids": ["550e8400-e29b-12d3-a456-426614174000"]
  }
}

// Target by labels (any agent with env=dev OR env=staging)
{
  "targeting": {
    "labels": ["env=dev", "env=staging"]
  }
}

// Target by annotations
{
  "targeting": {
    "annotations": {
      "capability": "gpu",
      "region": "us-east-1"
    }
  }
}

// Combined targeting (OR logic across all methods)
{
  "targeting": {
    "labels": ["env=prod"],
    "annotations": {"tier": "premium"}
  }
}
```

## Build Work Orders

Build work orders use Shipwright Build to create container images. The YAML content should contain a Shipwright Build specification.

### Required Components

1. **Shipwright Build**: Defines the build configuration (source, strategy, output)
2. **Agent with Shipwright**: Target agent must have Shipwright enabled

### Build Lifecycle

1. Admin creates work order via broker API
2. Broker routes to matching agents
3. Agent claims work order
4. Agent creates Shipwright BuildRun
5. Shipwright executes build (via Tekton)
6. Agent reports completion to broker
7. Work order moves to log

## Monitoring Work Orders

```bash
# List active work orders
curl http://localhost:3000/api/v1/work-orders \
  -H "Authorization: Bearer <ADMIN_PAK>"

# Get work order details
curl http://localhost:3000/api/v1/work-orders/<id> \
  -H "Authorization: Bearer <ADMIN_PAK>"

# View completed work orders (log)
curl http://localhost:3000/api/v1/work-order-log \
  -H "Authorization: Bearer <ADMIN_PAK>"
```
