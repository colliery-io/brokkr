---
title: "Quick Start Guide"
weight: 2
---

# Quick Start Guide

This guide walks through deploying your first application using Brokkr. You will create a stack, associate it with an agent, deploy Kubernetes resources, and then clean up. By the end, you will understand the core deployment workflow that Brokkr provides.

## Prerequisites

Before starting this guide, ensure you have completed the [Installation Guide](installation) and have both the broker and at least one agent running. You will also need the admin PAK (Prefixed API Key) from the broker setup and `kubectl` configured to access your target cluster.

If you followed the installation guide's quick start, you should have the broker accessible at `http://localhost:3000` via port-forward. The examples below assume this setup.

## Understanding the Deployment Model

Brokkr organizes deployments around three concepts: stacks, agents, and targeting. A **stack** represents a logical grouping of Kubernetes resources that belong together—think of it as an application or service. An **agent** runs in a Kubernetes cluster and applies resources. **Targeting** connects stacks to agents, telling Brokkr which agents should receive which stacks.

When you create a deployment object in a stack, Brokkr stores it in the broker's database. Agents that target that stack poll the broker and retrieve pending deployment objects. Each agent then applies the resources to its cluster using Kubernetes server-side apply.

## Step 1: Create a Stack

Stacks serve as containers for related Kubernetes resources. Start by creating a stack for this quick start application:

```bash
# Set your admin PAK for convenience
export ADMIN_PAK="brokkr_BR..."  # Replace with your actual PAK

# Create a stack
curl -s -X POST http://localhost:3000/api/v1/stacks \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer $ADMIN_PAK" \
  -d '{
    "name": "quick-start-app",
    "description": "My first Brokkr deployment"
  }' | jq .
```

The response includes a stack ID—save this for subsequent commands:

```json
{
  "id": "550e8400-e29b-41d4-a716-446655440000",
  "name": "quick-start-app",
  "description": "My first Brokkr deployment",
  "created_at": "2024-01-15T10:30:00Z"
}
```

```bash
# Save the stack ID
export STACK_ID="550e8400-e29b-41d4-a716-446655440000"  # Use your actual ID
```

## Step 2: Target the Stack to Your Agent

For an agent to receive deployment objects from a stack, you must create a targeting relationship. First, find your agent's ID:

```bash
# List agents to find the ID
curl -s http://localhost:3000/api/v1/agents \
  -H "Authorization: Bearer $ADMIN_PAK" | jq '.[].id'
```

Then create the targeting:

```bash
# Save the agent ID
export AGENT_ID="your-agent-id-here"

# Create the targeting relationship
curl -s -X POST http://localhost:3000/api/v1/agents/$AGENT_ID/targets \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer $ADMIN_PAK" \
  -d "{
    \"agent_id\": \"$AGENT_ID\",
    \"stack_id\": \"$STACK_ID\"
  }" | jq .
```

Once targeted, the agent will poll for deployment objects from this stack on its next polling cycle.

## Step 3: Deploy the Application

Create a YAML file containing the Kubernetes resources for a simple application. This example creates a namespace, a ConfigMap, and a Deployment:

```bash
cat > quick-start.yaml << 'EOF'
apiVersion: v1
kind: Namespace
metadata:
  name: quick-start
---
apiVersion: v1
kind: ConfigMap
metadata:
  name: app-config
  namespace: quick-start
data:
  message: "Hello from Brokkr!"
  environment: development
---
apiVersion: apps/v1
kind: Deployment
metadata:
  name: quick-start-app
  namespace: quick-start
spec:
  replicas: 1
  selector:
    matchLabels:
      app: quick-start-app
  template:
    metadata:
      labels:
        app: quick-start-app
    spec:
      containers:
      - name: app
        image: busybox:1.36
        command: ["sh", "-c", "while true; do echo $MESSAGE; sleep 30; done"]
        env:
        - name: MESSAGE
          valueFrom:
            configMapKeyRef:
              name: app-config
              key: message
EOF
```

Now deploy this YAML through Brokkr. The `jq` command properly encodes the YAML content for the JSON request:

```bash
curl -s -X POST "http://localhost:3000/api/v1/stacks/$STACK_ID/deployment-objects" \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer $ADMIN_PAK" \
  -d "$(jq -n --arg yaml "$(cat quick-start.yaml)" '{yaml_content: $yaml, is_deletion_marker: false}')" | jq .
```

The broker stores this deployment object and marks it pending. On the agent's next polling cycle (default: every 30 seconds), it retrieves the object and applies the resources to the cluster.

## Step 4: Verify the Deployment

Wait a few seconds for the agent to poll and apply the resources, then verify they were created:

```bash
# Check the namespace
kubectl get namespace quick-start

# Verify the ConfigMap
kubectl get configmap app-config -n quick-start -o yaml

# Check the Deployment and its pods
kubectl get deployment quick-start-app -n quick-start
kubectl get pods -n quick-start

# View the application logs
kubectl logs -n quick-start -l app=quick-start-app
```

You should see the pod running and logging "Hello from Brokkr!" every 30 seconds.

You can also check the deployment status through the Brokkr API:

```bash
# View deployment objects in the stack
curl -s "http://localhost:3000/api/v1/stacks/$STACK_ID/deployment-objects" \
  -H "Authorization: Bearer $ADMIN_PAK" | jq '.[] | {id, sequence_id, status, created_at}'
```

## Step 5: Update the Application

To update an application in Brokkr, you submit a new deployment object with the complete desired state. Brokkr does not support partial updates—each deployment object represents the full set of resources for that deployment.

Create an updated version of the application:

```bash
cat > quick-start-updated.yaml << 'EOF'
apiVersion: v1
kind: Namespace
metadata:
  name: quick-start
---
apiVersion: v1
kind: ConfigMap
metadata:
  name: app-config
  namespace: quick-start
data:
  message: "Updated: Brokkr is working!"
  environment: production
  debug: "false"
---
apiVersion: apps/v1
kind: Deployment
metadata:
  name: quick-start-app
  namespace: quick-start
spec:
  replicas: 2
  selector:
    matchLabels:
      app: quick-start-app
  template:
    metadata:
      labels:
        app: quick-start-app
    spec:
      containers:
      - name: app
        image: busybox:1.36
        command: ["sh", "-c", "while true; do echo $MESSAGE; sleep 30; done"]
        env:
        - name: MESSAGE
          valueFrom:
            configMapKeyRef:
              name: app-config
              key: message
EOF
```

Deploy the updated resources:

```bash
curl -s -X POST "http://localhost:3000/api/v1/stacks/$STACK_ID/deployment-objects" \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer $ADMIN_PAK" \
  -d "$(jq -n --arg yaml "$(cat quick-start-updated.yaml)" '{yaml_content: $yaml, is_deletion_marker: false}')" | jq .
```

After the agent polls and applies the update, verify the changes:

```bash
# Check that replicas scaled to 2
kubectl get pods -n quick-start

# Verify the ConfigMap was updated
kubectl get configmap app-config -n quick-start -o jsonpath='{.data.message}'
```

## Step 6: Clean Up

To delete resources through Brokkr, submit a deployment object with `is_deletion_marker: true`. This tells the agent to remove the resources rather than apply them.

```bash
curl -s -X POST "http://localhost:3000/api/v1/stacks/$STACK_ID/deployment-objects" \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer $ADMIN_PAK" \
  -d "$(jq -n --arg yaml "$(cat quick-start-updated.yaml)" '{yaml_content: $yaml, is_deletion_marker: true}')" | jq .
```

The YAML content is required even for deletions so the agent knows exactly which resources to remove. After the agent processes this, verify the resources are gone:

```bash
# The namespace should be terminating or gone
kubectl get namespace quick-start
```

Optionally, clean up the Brokkr resources:

```bash
# Delete the targeting
curl -s -X DELETE "http://localhost:3000/api/v1/agents/$AGENT_ID/targets/$STACK_ID" \
  -H "Authorization: Bearer $ADMIN_PAK"

# Delete the stack
curl -s -X DELETE "http://localhost:3000/api/v1/stacks/$STACK_ID" \
  -H "Authorization: Bearer $ADMIN_PAK"
```

## Next Steps

You have now completed the basic Brokkr workflow: creating stacks, targeting them to agents, deploying resources, updating them, and cleaning up. From here, you can explore more advanced topics:

- Read about [Core Concepts](../explanation/core-concepts) to understand Brokkr's design philosophy
- Learn about the [Data Model](../explanation/data-model) to understand how stacks, agents, and deployment objects relate
- Explore the [Configuration Guide](configuration) for production deployment options
- Check the [Work Orders](../reference/work-orders) reference for understanding deployment processing
