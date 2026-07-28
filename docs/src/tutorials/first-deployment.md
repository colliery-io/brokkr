# Tutorial: Deploy Your First Application

In this tutorial, you'll deploy an nginx web server to a Kubernetes cluster through Brokkr. You'll learn the core workflow: creating a stack, adding Kubernetes manifests as deployment objects, and watching an agent apply them to a cluster.

**What you'll learn:**

- How stacks organize Kubernetes resources
- How deployment objects carry YAML manifests
- How agents poll for and apply resources
- How to verify deployments succeeded

**Prerequisites:**

- A broker you can reach at `http://localhost:3000`, from either a [Helm install](../getting-started/installation.md) (`kubectl port-forward svc/brokkr-broker 3000:3000`) or the [local development environment](../getting-started/development.md) (`angreal local up`)
- The admin PAK (Prefixed API Key) for that broker. The PAK itself is never logged. In the development environment it is the publicly known dev PAK `brokkr_BR3rVsDa_GK3QN7CDUzYc6iKgMkJ98M2WSimM5t6U8`, which corresponds to the embedded default `broker.pak_hash`. After a Helm install it is the PAK you minted with `brokkr-broker generate-pak` — you can run that from the published image without installing anything: `docker run --rm ghcr.io/colliery-io/brokkr-broker:latest generate-pak`. (Only if a broker is explicitly configured with an empty `broker.pak_hash` does it generate a fresh PAK at first startup and write it to `/tmp/brokkr-keys/key.txt` inside the broker container; leaving the setting untouched keeps the embedded default hash.)
- **A running agent connected to a Kubernetes cluster.** This tutorial checks its own results with `kubectl`, so an agent *process* has to be polling and applying — an agent record in the broker alone is not enough. The development environment pre-creates one called `brokkr-integration-test-agent`; after a Helm install it is the agent you created and installed in [Quick Start steps 4 and 5](../getting-started/installation.md#4-create-an-agent-and-get-its-pak), named by your `broker.agentName`.
- `curl` and `jq` installed

Export your agent's name before you start — the commands below use it:

```bash
export AGENT_NAME=brokkr-integration-test-agent   # development environment
# export AGENT_NAME=my-agent                      # Helm install: your broker.agentName
```

A freshly created agent — the development environment's test agent included — starts `INACTIVE` and is registered only with the **system generator**. You'll activate it and register it with the `admin-generator` in Step 3 before targeting it to your stack — see [Registering Agents with Generators](../how-to/agent-registration.md) for the operational details. (If you already activated and registered your agent while following the installation guide, those steps are idempotent enough to re-run: re-registering returns `409 already_registered`.)

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
  -H "Authorization: Bearer <your-admin-pak>" | jq '.[].name'
```

You should see the name you exported as `$AGENT_NAME` in that list: `"brokkr-integration-test-agent"` in the development environment, or your own `broker.agentName` after a Helm install. If the list is empty, no agent has been created yet — go back to [Quick Start step 4](../getting-started/installation.md#4-create-an-agent-and-get-its-pak).

> **Tip:** Throughout this tutorial, replace `<your-admin-pak>` with your actual admin PAK. In the development environment that is `brokkr_BR3rVsDa_GK3QN7CDUzYc6iKgMkJ98M2WSimM5t6U8`; on your own install it is the PAK whose hash you configured as `broker.pakHash` / `broker.pakHashExistingSecret`.

## Step 2: Create a Stack

A **stack** is a named container that groups related Kubernetes resources. Think of it as a logical application — everything needed to run your service lives inside one stack.

Stacks are always owned by a **generator** (the entity that manages deployments, typically a CI/CD pipeline). The broker creates an `admin-generator` at first startup, linked to the admin PAK — as an admin, you'll create the stack under it. Look up its ID first:

> **Admin PAK vs. generator PAK.** This tutorial uses the **admin** PAK for everything because it's the simplest way to learn the workflow. In real use, the admin PAK is a privileged break-glass credential — day-to-day, each CI pipeline or tenant gets its own **generator** (with its own PAK) that owns the stacks it manages. See [Working with Generators](../how-to/generators.md) for the production pattern.

```bash
GEN_ID=$(curl -s http://localhost:3000/api/v1/generators \
  -H "Authorization: Bearer <your-admin-pak>" \
  | jq -r '.[] | select(.name=="admin-generator") | .id')

STACK_ID=$(curl -s -X POST http://localhost:3000/api/v1/stacks \
  -H "Authorization: Bearer <your-admin-pak>" \
  -H "Content-Type: application/json" \
  -d "{
    \"name\": \"tutorial-nginx\",
    \"description\": \"Tutorial: a simple nginx deployment\",
    \"generator_id\": \"${GEN_ID}\"
  }" | jq -r '.id')

echo "Stack ID: $STACK_ID"
```

The response contains the new stack with its ID. The `generator_id` field ties the stack to its owning generator — we'll explore generators in a [later tutorial](./cicd-generators.md).

## Step 3: Activate, Register, and Target the Agent

Agents receive a stack's resources in two ways: via **agent targets** — explicit assignments that require the agent to be registered with the stack's owning generator first — or via label/annotation matching, when the stack's selectors match the agent's labels or annotations. In this tutorial you'll use an agent target.

Registration is the agent's opt-in consent boundary: an agent must be registered with a generator before any stack that generator owns can be targeted at it. A new agent — the development environment's test agent, or one you created through the API — is registered only with the system generator, so you must register it with the `admin-generator` before targeting. For the operational details, see [Registering Agents with Generators](../how-to/agent-registration.md).

First, get the agent ID (using the `$AGENT_NAME` you exported in the prerequisites):

```bash
AGENT_ID=$(curl -s http://localhost:3000/api/v1/agents \
  -H "Authorization: Bearer <your-admin-pak>" \
  | jq -r --arg name "$AGENT_NAME" '.[] | select(.name==$name) | .id')

echo "Agent ID: $AGENT_ID"
```

Every new agent starts with status `INACTIVE`, and an inactive agent skips all deployment work — nothing reaches the cluster until you activate it. Activate yours now (harmless if it is already `ACTIVE`):

```bash
curl -s -X PUT "http://localhost:3000/api/v1/agents/${AGENT_ID}" \
  -H "Authorization: Bearer <your-admin-pak>" \
  -H "Content-Type: application/json" \
  -d '{"status": "ACTIVE"}' | jq '{name, status}'
```

The response should show `"status": "ACTIVE"`.

Next, register the agent with the `admin-generator` (the `$GEN_ID` you looked up in Step 2):

```bash
curl -s -X POST "http://localhost:3000/api/v1/generators/${GEN_ID}/register" \
  -H "Authorization: Bearer <your-admin-pak>" \
  -H "Content-Type: application/json" \
  -d "{\"agent_id\": \"${AGENT_ID}\"}" | jq .
```

Registration gates which generators an agent can serve, ensuring agents opt in to application scopes before receiving their deployments. Without it, the next request fails with a `403 agent_not_registered` error — and admins cannot bypass the gate.

Now target the agent to your stack:

```bash
curl -s -X POST "http://localhost:3000/api/v1/agents/${AGENT_ID}/targets" \
  -H "Authorization: Bearer <your-admin-pak>" \
  -H "Content-Type: application/json" \
  -d "{\"agent_id\": \"${AGENT_ID}\", \"stack_id\": \"${STACK_ID}\"}" | jq .
```

The request body carries both `agent_id` and `stack_id` — the broker requires both fields even though the agent also appears in the URL.

The agent will now receive deployment objects from this stack on its next poll cycle.

## Step 4: Create a Deployment Object

A **deployment object** contains the actual Kubernetes YAML that the agent will apply to its cluster. You can include multiple Kubernetes resources in a single deployment object using multi-document YAML (separated by `---`).

Create a deployment object with an nginx namespace, deployment, and service:

```bash
curl -s -X POST "http://localhost:3000/api/v1/stacks/${STACK_ID}/deployment-objects" \
  -H "Authorization: Bearer <your-admin-pak>" \
  -H "Content-Type: application/json" \
  -d '{
    "yaml_content": "apiVersion: v1\nkind: Namespace\nmetadata:\n  name: tutorial-nginx\n---\napiVersion: apps/v1\nkind: Deployment\nmetadata:\n  name: nginx\n  namespace: tutorial-nginx\n  labels:\n    app: nginx\nspec:\n  replicas: 2\n  selector:\n    matchLabels:\n      app: nginx\n  template:\n    metadata:\n      labels:\n        app: nginx\n    spec:\n      containers:\n      - name: nginx\n        image: nginx:1.27\n        ports:\n        - containerPort: 80\n---\napiVersion: v1\nkind: Service\nmetadata:\n  name: nginx\n  namespace: tutorial-nginx\nspec:\n  selector:\n    app: nginx\n  ports:\n  - port: 80\n    targetPort: 80"
  }' | jq .
```

The response includes a `sequence_id` — an auto-incrementing number that orders deployment objects within a stack. The agent uses this to know which version is latest.

## Step 5: Watch the Agent Apply Resources

The agent polls the broker at its configured interval — the binary's embedded default is 10 seconds, while the Helm chart sets 30 seconds via `agent.pollingInterval`. Allow one full poll cycle before checking; then you should see the resources appear in your Kubernetes cluster.

Check the agent events to confirm the deployment was applied:

```bash
curl -s "http://localhost:3000/api/v1/agents/${AGENT_ID}/events" \
  -H "Authorization: Bearer <your-admin-pak>" | jq '.[] | {event_type, status, message, created_at}'
```

You should see a SUCCESS event:

```json
{
  "event_type": "DEPLOY",
  "status": "SUCCESS",
  "message": null,
  "created_at": "2025-01-15T10:01:30Z"
}
```

> **Troubleshooting:** If the events list stays empty and nothing appears on the cluster, the usual cause is an agent that is still `INACTIVE` — re-run the activation command from Step 3 and check that the response shows `"status": "ACTIVE"`.

With `kubectl` pointed at the cluster your agent manages (the bundled k3s in the development environment, or the cluster you installed the agent chart into), verify the resources directly:

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
  -H "Authorization: Bearer <your-admin-pak>" \
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

To remove the deployed resources, create a **deletion marker** — a special deployment object with `is_deletion_marker: true`. This tells the agent to delete **all resources previously applied for this stack** from the cluster. The agent finds them by the `k8s.brokkr.io/stack` annotation it stamped on every object it applied, so the marker's YAML body plays no part in deciding what gets removed. Send an empty `yaml_content` — that is the only body a deletion marker is allowed to skip validation for:

```bash
curl -s -X POST "http://localhost:3000/api/v1/stacks/${STACK_ID}/deployment-objects" \
  -H "Authorization: Bearer <your-admin-pak>" \
  -H "Content-Type: application/json" \
  -d '{
    "yaml_content": "",
    "is_deletion_marker": true
  }' | jq .
```

> **Note:** "Empty" means genuinely empty. A placeholder comment such as `# deletion` is not empty — it parses to a YAML document with no content, and the broker rejects it with `400 invalid_deployment_object` ("YAML content has no documents"). Anything else you send must be valid, non-null YAML.

The agent will remove the Kubernetes resources on its next poll. Verify:

```bash
kubectl get namespace tutorial-nginx
```

After a few seconds, the namespace and all its contents will be gone.

Optionally, remove the agent target and delete the stack:

```bash
# Remove the target
curl -s -X DELETE "http://localhost:3000/api/v1/agents/${AGENT_ID}/targets/${STACK_ID}" \
  -H "Authorization: Bearer <your-admin-pak>"

# Delete the stack (soft delete — marks as deleted but preserves the record)
curl -s -X DELETE "http://localhost:3000/api/v1/stacks/${STACK_ID}" \
  -H "Authorization: Bearer <your-admin-pak>"
```

> **Note:** Deletion in Brokkr is a "soft delete" — the record is marked with a `deleted_at` timestamp but not removed from the database. Soft-deleting a stack also soft-deletes its deployment objects and automatically inserts a deletion marker, so deleting the stack alone is enough to clean its resources off the cluster. See [Soft Deletion](../reference/soft-deletion.md) for details.

## What You've Learned

- **Stacks** group related Kubernetes resources under a single name
- **Deployment objects** carry the YAML manifests inside a stack
- **Agent targets** connect agents to stacks, controlling which clusters receive which resources
- **Sequence IDs** let the agent know when a newer version is available
- **Deletion markers** trigger resource cleanup on the cluster
- Agents use a **pull-based model** — they poll the broker, so clusters behind firewalls work without inbound connections

## Next Steps

- [Submitting a Folder of Manifests (CLI)](../how-to/cli-apply.md) — `brokkr apply -f ./manifests` instead of hand-escaping a `yaml_content` string
- [Multi-Cluster Targeting](./multi-cluster-targeting.md) — direct deployments to specific clusters using labels
- [CI/CD with Generators](./cicd-generators.md) — automate deployment pushes from a CI pipeline
- [Managing Stacks](../how-to/managing-stacks.md) — deeper guide on stack lifecycle management
- [Configuration Guide](../getting-started/configuration.md) — tune polling intervals, database settings, and more
