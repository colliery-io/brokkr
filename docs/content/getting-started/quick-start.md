---
title: "Quick Start Guide"
weight: 2
---

# Quick Start Guide

This guide will help you deploy your first application using Brokkr. We'll create a simple but complete application stack that demonstrates Brokkr's core features.

## Prerequisites

- Completed the [Installation Guide](installation)
- A running Kubernetes cluster
- `kubectl` configured to access your cluster
- Admin PAK from the broker setup

## Deploying Your First Application

### 1. Create a Stack

First, let's create a stack to organize our deployment:

```bash
curl -X POST http://localhost:3000/api/v1/stacks \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer <admin_pak>" \
  -d '{
    "name": "quick-start-app",
    "description": "My first Brokkr deployment"
  }'
```

Save the returned stack ID for later use.

### 2. Target the Stack with Your Agent

If you haven't already, target the stack with your agent:

```bash
curl -X POST http://localhost:3000/api/v1/agents/<agent_id>/targets \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer <admin_pak>" \
  -d '{
    "agent_id": "<agent_id>",
    "stack_id": "<stack_id>"
  }'
```

### 3. Deploy the Application

Let's deploy a simple application stack. First, create a file named `quick-start.yaml` with the following content:

```yaml
# quick-start.yaml
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
  config.yaml: |
    environment: development
    debug: true
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
        image: busybox
        command: ["sleep", "3600"]
        envFrom:
        - configMapRef:
            name: app-config
```

Then, deploy the application using the YAML file:

```bash
curl -X POST http://localhost:3000/api/v1/stacks/<stack_id>/deployment-objects \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer <admin_pak>" \
  -d "{
    \"yaml_content\": \"$(cat quick-start.yaml | sed 's/"/\\"/g' | tr '\n' 'n')\",
    \"is_deletion_marker\": false
  }"
```

This deployment:
- Creates a `quick-start` namespace
- Adds a ConfigMap with application configuration
- Deploys a simple application using the busybox image
- Uses proper resource ordering (namespace first)
- Includes proper labeling for the application

### 4. Verify the Deployment

Check that your resources were created:

```bash
# Verify the namespace
kubectl get namespace quick-start

# Check the ConfigMap
kubectl get configmap app-config -n quick-start

# Verify the deployment
kubectl get deployment quick-start-app -n quick-start
```

### 5. Update the Application

Let's update the application by modifying the ConfigMap. Create a new file `quick-start-updated.yaml` with the complete updated application stack:

```yaml
# quick-start-updated.yaml
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
  config.yaml: |
    environment: development
    debug: false
    new_setting: enabled
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
        image: busybox
        command: ["sleep", "3600"]
        envFrom:
        - configMapRef:
            name: app-config
```

Then, deploy the updated application:

```bash
curl -X POST http://localhost:3000/api/v1/stacks/<stack_id>/deployment-objects \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer <admin_pak>" \
  -d "{
    \"yaml_content\": \"$(cat quick-start-updated.yaml | sed 's/"/\\"/g' | tr '\n' 'n')\",
    \"is_deletion_marker\": false
  }"
```

Note: When updating an application in Brokkr, you must provide the complete set of resources, not just the changed ones. This ensures that the agent has the complete desired state of your application.

### 6. Clean Up

When you're done, you can delete the entire application. Create a file `quick-start-delete.yaml` with the complete application stack:

```yaml
# quick-start-delete.yaml
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
  config.yaml: |
    environment: development
    debug: false
    new_setting: enabled
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
        image: busybox
        command: ["sleep", "3600"]
        envFrom:
        - configMapRef:
            name: app-config
```

Then, send the deletion request with `is_deletion_marker: true`:

```bash
curl -X POST http://localhost:3000/api/v1/stacks/<stack_id>/deployment-objects \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer <admin_pak>" \
  -d "{
    \"yaml_content\": \"$(cat quick-start-delete.yaml | sed 's/"/\\"/g' | tr '\n' 'n')\",
    \"is_deletion_marker\": true
  }"
```

Note: When deleting an application in Brokkr, you must provide the complete set of resources with `is_deletion_marker: true`. The YAML content is required so that the agent knows exactly which resources to remove from the cluster.

## What's Next?

Now that you've deployed your first application with Brokkr, you can:

- Learn about [Core Concepts](../../explanation/core-concepts) in Brokkr
- Check out our [Tutorials](../../tutorials) for more advanced use cases
- Read about [Best Practices](../../how-to/best-practices) for managing your deployments
- Explore the [API Reference](../../reference/api) for more details on available endpoints
