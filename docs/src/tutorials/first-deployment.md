# Tutorial: Deploy Your First Application

In this tutorial, you'll deploy an nginx web server to a Kubernetes cluster through Brokkr. You'll learn the core workflow: creating a stack, adding Kubernetes manifests as deployment objects, and watching an agent apply them to a cluster.

**What you'll learn:**

- How stacks organize Kubernetes resources
- How deployment objects carry YAML manifests
- How agents poll for and apply resources
- How to verify deployments succeeded

**Prerequisites:**

- A running Brokkr development environment (see [Installation Guide](../getting-started/installation.md) — `angreal local up` starts the full stack)
- The admin PAK (Pre-Authentication Key) printed during first startup (check broker logs if you missed it)
- `curl` and `jq` installed

## Step 1: Verify the Environment

First, confirm the broker is running and healthy:

```bash
curl -s http://localhost:3000/healthz
```

You should see:

```
OK
```

Check that at least one agent is registered:

```bash
curl -s http://localhost:3000/api/v1/agents \
  -H "Authorization: <your-admin-pak>" | jq '.[].name'
```

You should see the default agent name (typically `"DEFAULT"`). Note the agent's `id` field — you'll need it later.

> **Tip:** Throughout this tutorial, replace `<your-admin-pak>` with the actual admin PAK from your broker startup logs. It looks like `brokkr_BR3rVsDa_GK3QN7CDUzYc6iKgMkJ98M2WSimM5t6U8`.

## Step 2: Create a Stack

A **stack** is a named container that groups related Kubernetes resources. Think of it as a logical application — everything needed to run your service lives inside one stack.

Stacks are always owned by a **generator** (the entity that manages deployments, typically a CI/CD pipeline). As an admin, you can create a stack on behalf of any generator. For this tutorial, we'll use a nil generator ID to indicate the stack is admin-managed:

```bash
STACK_ID=$(curl -s -X POST http://localhost:3000/api/v1/stacks \
  -H "Authorization: <your-admin-pak>" \
  -H "Content-Type: application/json" \
  -d '{
    "name": "tutorial-nginx",
    "description": "Tutorial: a simple nginx deployment",
    "generator_id": "00000000-0000-0000-0000-000000000000"
  }' | jq -r '.id')

echo "Stack ID: $STACK_ID"
```

The response contains the new stack with its ID. The `generator_id` field ties the stack to its owning generator — we'll explore generators in a [later tutorial](./cicd-generators.md).

## Step 3: Target the Agent to the Stack

Agents don't automatically receive every stack's resources. You need to explicitly **target** an agent to a stack, telling it "you are responsible for deploying this stack's resources."

First, get the agent ID:

```bash
AGENT_ID=$(curl -s http://localhost:3000/api/v1/agents \
  -H "Authorization: <your-admin-pak>" | jq -r '.[0].id')

echo "Agent ID: $AGENT_ID"
```

Now target the agent to your stack:

```bash
curl -s -X POST "http://localhost:3000/api/v1/agents/${AGENT_ID}/targets" \
  -H "Authorization: <your-admin-pak>" \
  -H "Content-Type: application/json" \
  -d "{\"stack_id\": \"${STACK_ID}\"}" | jq .
```

The agent will now receive deployment objects from this stack on its next poll cycle.

## Step 4: Create a Deployment Object

A **deployment object** contains the actual Kubernetes YAML that the agent will apply to its cluster. You can include multiple Kubernetes resources in a single deployment object using multi-document YAML (separated by `---`).

Create a deployment object with an nginx namespace, deployment, and service:

```bash
curl -s -X POST "http://localhost:3000/api/v1/stacks/${STACK_ID}/deployment-objects" \
  -H "Authorization: <your-admin-pak>" \
  -H "Content-Type: application/json" \
  -d '{
    "yaml_content": "apiVersion: v1\nkind: Namespace\nmetadata:\n  name: tutorial-nginx\n---\napiVersion: apps/v1\nkind: Deployment\nmetadata:\n  name: nginx\n  namespace: tutorial-nginx\n  labels:\n    app: nginx\nspec:\n  replicas: 2\n  selector:\n    matchLabels:\n      app: nginx\n  template:\n    metadata:\n      labels:\n        app: nginx\n    spec:\n      containers:\n      - name: nginx\n        image: nginx:1.27\n        ports:\n        - containerPort: 80\n---\napiVersion: v1\nkind: Service\nmetadata:\n  name: nginx\n  namespace: tutorial-nginx\nspec:\n  selector:\n    app: nginx\n  ports:\n  - port: 80\n    targetPort: 80"
  }' | jq .
```

The response includes a `sequence_id` — an auto-incrementing number that orders deployment objects within a stack. The agent uses this to know which version is latest.

## Step 5: Watch the Agent Apply Resources

The agent polls the broker at its configured interval (default: 10 seconds). Within a few seconds, you should see the resources appear in your Kubernetes cluster.

Check the agent events to confirm the deployment was applied:

```bash
curl -s "http://localhost:3000/api/v1/agents/${AGENT_ID}/events" \
  -H "Authorization: <your-admin-pak>" | jq '.[] | {event_type, status, message, created_at}'
```

You should see a SUCCESS event:

```json
{
  "event_type": "DEPLOYMENT",
  "status": "SUCCESS",
  "message": "Successfully applied deployment object",
  "created_at": "2025-01-15T10:01:30Z"
}
```

If you have `kubectl` configured to talk to your development cluster (k3s), verify the resources directly:

```bash
kubectl get all -n tutorial-nginx
```

Expected output:

```
NAME                         READY   STATUS    RESTARTS   AGE
pod/nginx-7c5ddbdf54-abc12   1/1     Running   0          30s
pod/nginx-7c5ddbdf54-def34   1/1     Running   0          30s

NAME            TYPE        CLUSTER-IP     EXTERNAL-IP   PORT(S)   AGE
service/nginx   ClusterIP   10.43.120.50   <none>        80/TCP    30s

NAME                    READY   UP-TO-DATE   AVAILABLE   AGE
deployment.apps/nginx   2/2     2            2           30s
```

## Step 6: Update the Deployment

To update a deployment, create a new deployment object in the same stack. The agent detects the new `sequence_id` and applies the updated manifests, reconciling the cluster state.

Scale nginx to 3 replicas:

```bash
curl -s -X POST "http://localhost:3000/api/v1/stacks/${STACK_ID}/deployment-objects" \
  -H "Authorization: <your-admin-pak>" \
  -H "Content-Type: application/json" \
  -d '{
    "yaml_content": "apiVersion: v1\nkind: Namespace\nmetadata:\n  name: tutorial-nginx\n---\napiVersion: apps/v1\nkind: Deployment\nmetadata:\n  name: nginx\n  namespace: tutorial-nginx\n  labels:\n    app: nginx\nspec:\n  replicas: 3\n  selector:\n    matchLabels:\n      app: nginx\n  template:\n    metadata:\n      labels:\n        app: nginx\n    spec:\n      containers:\n      - name: nginx\n        image: nginx:1.27\n        ports:\n        - containerPort: 80\n---\napiVersion: v1\nkind: Service\nmetadata:\n  name: nginx\n  namespace: tutorial-nginx\nspec:\n  selector:\n    app: nginx\n  ports:\n  - port: 80\n    targetPort: 80"
  }' | jq .
```

After the next poll cycle, verify the update:

```bash
kubectl get deployment nginx -n tutorial-nginx
```

You should see `3/3` in the READY column.

## Step 7: Clean Up

To remove the deployed resources, create a **deletion marker** — a special deployment object with `is_deletion_marker: true`. This tells the agent to delete **all resources previously applied for this stack** from the cluster (not just the resources listed in the YAML content):

```bash
curl -s -X POST "http://localhost:3000/api/v1/stacks/${STACK_ID}/deployment-objects" \
  -H "Authorization: <your-admin-pak>" \
  -H "Content-Type: application/json" \
  -d '{
    "yaml_content": "apiVersion: v1\nkind: Namespace\nmetadata:\n  name: tutorial-nginx",
    "is_deletion_marker": true
  }' | jq .
```

The agent will remove the Kubernetes resources on its next poll. Verify:

```bash
kubectl get namespace tutorial-nginx
```

After a few seconds, the namespace and all its contents will be gone.

Optionally, remove the agent target and delete the stack:

```bash
# Remove the target
curl -s -X DELETE "http://localhost:3000/api/v1/agents/${AGENT_ID}/targets/${STACK_ID}" \
  -H "Authorization: <your-admin-pak>"

# Delete the stack (soft delete — marks as deleted but preserves the record)
curl -s -X DELETE "http://localhost:3000/api/v1/stacks/${STACK_ID}" \
  -H "Authorization: <your-admin-pak>"
```

> **Note:** Deletion in Brokkr is a "soft delete" — the record is marked with a `deleted_at` timestamp but not removed from the database. See [Soft Deletion](../reference/soft-deletion.md) for details.

## What You've Learned

- **Stacks** group related Kubernetes resources under a single name
- **Deployment objects** carry the YAML manifests inside a stack
- **Agent targets** connect agents to stacks, controlling which clusters receive which resources
- **Sequence IDs** let the agent know when a newer version is available
- **Deletion markers** trigger resource cleanup on the cluster
- Agents use a **pull-based model** — they poll the broker, so clusters behind firewalls work without inbound connections

## Next Steps

- [Multi-Cluster Targeting](./multi-cluster-targeting.md) — direct deployments to specific clusters using labels
- [CI/CD with Generators](./cicd-generators.md) — automate deployment pushes from a CI pipeline
- [Managing Stacks](../how-to/managing-stacks.md) — deeper guide on stack lifecycle management
- [Configuration Guide](../getting-started/configuration.md) — tune polling intervals, database settings, and more
