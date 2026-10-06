# Tutorial: Your First Tenant, From a Fresh Broker to a Running Manifest

In this tutorial you take a broker that has nothing in it and get a manifest running on a cluster under a tenant's own credential. Two people take part. The **admin** runs the broker and holds the admin PAK. The **tenant** is a team or a CI pipeline; it gets its own PAK and never sees the admin's. Each command below says which of the two runs it.

**What you'll learn:**

- What the admin does once per tenant, and what the tenant does on its own after that
- Which credential each step needs, and what each credential cannot do
- How a tenant checks that its manifest reached the cluster

**Time to complete:** 15 minutes.

**Prerequisites:**

- A broker at `http://localhost:3000` with a running agent. The quickest is the development environment: `angreal local up` prints the broker URL, the console URL, the admin PAK and the path of the agent's key when it finishes. A [Helm install](../getting-started/installation.md) works too; then the admin PAK is the one you minted with `generate-pak`, and the agent is the one you installed.
- `curl`, `jq`, and `kubectl` pointed at the cluster the agent manages. In the development environment: `export KUBECONFIG=/tmp/brokkr-keys/kubeconfig.local.yaml`.
- The `brokkr` CLI on your `PATH`. Download it from the [GitHub Release](https://github.com/colliery-io/brokkr/releases), or build it with `cargo build --release -p brokkr-cli`. Each step that uses it also shows the `curl` form.

Export the two values the admin starts with:

```bash
export BROKER=http://localhost:3000
export ADMIN_PAK=brokkr_BR3rVsDa_GK3QN7CDUzYc6iKgMkJ98M2WSimM5t6U8   # the development PAK; on your own install, the one you minted
export AGENT_NAME=brokkr-integration-test-agent                     # the development agent; on your own install, your broker.agentName
export BROKKR_BROKER_URL=$BROKER                                    # the brokkr CLI reads the broker URL from here
```

Each `brokkr` command below gives its PAK with `--pak`, so you can see which of the two people runs it.

> The development PAK is publicly known. It opens only a broker that still runs the embedded default hash. Never use it in production.

## Part 1: The admin sets the tenant up

The admin does three things, once. After this part the tenant works alone.

### Step 1 (admin): Create the tenant

A tenant is a **generator** in the API. Creating one returns its PAK once; the broker keeps only a hash.

```bash
TENANT=$(curl -s -X POST "$BROKER/api/v1/generators" \
  -H "Authorization: Bearer $ADMIN_PAK" \
  -H "Content-Type: application/json" \
  -d '{"name": "team-tutorial", "description": "Tutorial tenant"}')

TENANT_ID=$(echo "$TENANT" | jq -r '.generator.id')
TENANT_PAK=$(echo "$TENANT" | jq -r '.pak')
echo "Tenant id: $TENANT_ID"
```

Store `TENANT_PAK` where the tenant's CI will read it. It cannot be shown again; the only way back is `POST /api/v1/generators/{id}/rotate-pak`, which the admin runs.

The operator console does the same from **Tenants → New tenant**, with the admin PAK typed into the form.

### Step 2 (admin): Activate the agent

A new agent starts `INACTIVE` and applies nothing. Until you activate it, its log says so on each poll. Activate it by name. This is safe to repeat.

```bash
brokkr --pak "$ADMIN_PAK" agent activate "$AGENT_NAME"
```

```
agent "brokkr-integration-test-agent" (6b1f...) is ACTIVE. It applies its stacks on its next poll.
```

`brokkr --pak "$ADMIN_PAK" agent list` shows each agent with its id and status. In the console, the agent's drawer under **Fleet** has an **Activate** button that does the same.

<details>
<summary>The same step with <code>curl</code></summary>

```bash
AGENT_ID=$(curl -s "$BROKER/api/v1/agents" \
  -H "Authorization: Bearer $ADMIN_PAK" \
  | jq -r --arg name "$AGENT_NAME" '.[] | select(.name==$name) | .id')

curl -s -X PUT "$BROKER/api/v1/agents/$AGENT_ID" \
  -H "Authorization: Bearer $ADMIN_PAK" \
  -H "Content-Type: application/json" \
  -d '{"status": "ACTIVE"}' | jq '{name, status}'
```

</details>

### Step 3 (admin): Register the agent with the tenant

Registration is the agent's consent to receive stacks from this tenant. Without it, the tenant's target in Step 6 fails with `agent_not_registered`. `brokkr register` takes ids, so get the agent id from the agent list first:

```bash
AGENT_ID=$(brokkr --pak "$ADMIN_PAK" agent list | awk -v name="$AGENT_NAME" '$1 == name {print $2}')
echo "Agent id: $AGENT_ID"

brokkr --pak "$ADMIN_PAK" register --agent "$AGENT_ID" --generator "$TENANT_ID"
```

<details>
<summary>The same step with <code>curl</code></summary>

```bash
curl -s -X POST "$BROKER/api/v1/generators/$TENANT_ID/register" \
  -H "Authorization: Bearer $ADMIN_PAK" \
  -H "Content-Type: application/json" \
  -d "{\"agent_id\": \"$AGENT_ID\"}" | jq '{agent_id, generator_id}'
```

</details>

### The hand-off

The admin gives the tenant three values: the broker URL, `TENANT_PAK`, and the agent name, `AGENT_NAME`. A tenant PAK lists only the agents that are registered with its tenant, so after Step 3 the tenant finds the agent by name. That is the whole hand-off. Everything after this is the tenant's.

## Part 2: The tenant deploys

### Step 4 (tenant): Confirm the credential

The tenant can ask the broker what its PAK is:

```bash
curl -s -X POST "$BROKER/api/v1/auth/pak" \
  -H "Authorization: Bearer $TENANT_PAK" | jq .
```

```json
{
  "admin": false,
  "agent": null,
  "generator": "e5fe8d83-4704-489a-a791-a53af689b456",
  "readonly": false
}
```

The `generator` value is `TENANT_ID`. A tenant that only has its PAK gets its id this way.

### Step 5 (tenant): Create a stack

A stack holds the manifests of one application. The tenant must name itself as the owner; any other `generator_id` is a `403`.

```bash
STACK_ID=$(curl -s -X POST "$BROKER/api/v1/stacks" \
  -H "Authorization: Bearer $TENANT_PAK" \
  -H "Content-Type: application/json" \
  -d "{\"name\": \"tutorial-hello\", \"description\": \"Tutorial: hello from a tenant\", \"generator_id\": \"$TENANT_ID\"}" \
  | jq -r '.id')
echo "Stack id: $STACK_ID"
```

### Step 6 (tenant): Target the stack to the agent

The tenant connects its stack to the agent the admin registered, by name:

```bash
brokkr --pak "$TENANT_PAK" stack target tutorial-hello "$AGENT_NAME"
```

```
targeted stack "tutorial-hello" at agent "brokkr-integration-test-agent"
```

An error that says the agent `is not registered with the tenant that owns stack` means Step 3 did not happen for this tenant. A line that starts with `unchanged:` means the target exists already; that is fine.

<details>
<summary>The same step with <code>curl</code></summary>

The tenant gets the agent id from `GET /api/v1/agents`, which lists the agents registered with it. The body repeats the agent id; the broker requires both fields. A `403 agent_not_registered` means Step 3 did not happen; a `409 unique_violation` means the target exists already.

```bash
AGENT_ID=$(curl -s "$BROKER/api/v1/agents" \
  -H "Authorization: Bearer $TENANT_PAK" \
  | jq -r --arg name "$AGENT_NAME" '.[] | select(.name==$name) | .id')

curl -s -X POST "$BROKER/api/v1/agents/$AGENT_ID/targets" \
  -H "Authorization: Bearer $TENANT_PAK" \
  -H "Content-Type: application/json" \
  -d "{\"agent_id\": \"$AGENT_ID\", \"stack_id\": \"$STACK_ID\"}" | jq '{agent_id, stack_id}'
```

</details>

### Step 7 (tenant): Push the manifest

Write the manifest to a file and send it as YAML. No JSON escaping is needed.

```bash
cat > hello.yaml <<'YAML'
apiVersion: v1
kind: Namespace
metadata:
  name: tutorial-hello
---
apiVersion: apps/v1
kind: Deployment
metadata:
  name: hello
  namespace: tutorial-hello
  labels:
    app: hello
spec:
  replicas: 1
  selector:
    matchLabels:
      app: hello
  template:
    metadata:
      labels:
        app: hello
    spec:
      containers:
      - name: hello
        image: nginx:1.27
        ports:
        - containerPort: 80
YAML

curl -s -X POST "$BROKER/api/v1/stacks/$STACK_ID/deployment-objects" \
  -H "Authorization: Bearer $TENANT_PAK" \
  -H "Content-Type: application/yaml" \
  --data-binary @hello.yaml | jq '{id, sequence_id}'
```

The response has a `sequence_id`. It is a broker-wide counter, so the first push of a new stack is not always 1; each push to the stack gets a higher one, and the agent applies the newest.

With the `brokkr` CLI, the same step is `brokkr apply -f ./manifests --stack tutorial-hello` with `BROKKR_PAK=$TENANT_PAK`; see [Submitting a Folder of Manifests](../how-to/cli-apply.md). With the admin PAK, add `--generator team-tutorial` to say which tenant owns the stack.

### Step 8: See it on the cluster

Wait one poll cycle: 10 seconds in the development environment, 30 seconds for a chart-installed agent. Then:

```bash
kubectl get all -n tutorial-hello
```

```
NAME                         READY   STATUS    RESTARTS   AGE
pod/hello-576894d9b7-x2k1    1/1     Running   0          20s

NAME                    READY   UP-TO-DATE   AVAILABLE   AGE
deployment.apps/hello   1/1     1            1           20s
```

The tenant can also list what it pushed:

```bash
curl -s "$BROKER/api/v1/stacks/$STACK_ID/deployment-objects" \
  -H "Authorization: Bearer $TENANT_PAK" | jq '.[] | {sequence_id, created_at}'
```

The agent's events and the stack's health are admin views. The admin sees them with:

```bash
curl -s "$BROKER/api/v1/agents/$AGENT_ID/events" \
  -H "Authorization: Bearer $ADMIN_PAK" | jq '.[0] | {event_type, status}'
```

```json
{ "event_type": "DEPLOY", "status": "SUCCESS" }
```

In the operator console, **Deployments** shows the stack with its tenant and its health, and the stack's drawer lists the agents that have it.

### Step 9 (tenant): Change it

Edit `hello.yaml` to `replicas: 2` and push it again with the same command as Step 7. The response has a higher `sequence_id`. After one poll cycle, `kubectl get deployment hello -n tutorial-hello` shows `2/2`.

## Part 3: Clean up

The tenant deletes its stack. The broker records a deletion marker, and the agent removes the stack's resources on its next poll:

```bash
curl -s -X DELETE "$BROKER/api/v1/stacks/$STACK_ID" \
  -H "Authorization: Bearer $TENANT_PAK"
```

After one poll cycle, `kubectl get namespace tutorial-hello` says not found.

The admin removes the tenant:

```bash
curl -s -X DELETE "$BROKER/api/v1/generators/$TENANT_ID" \
  -H "Authorization: Bearer $ADMIN_PAK"
```

## What you've learned

- The admin does three things once per tenant: create it, activate the agent, register the agent with it. The hand-off is the broker URL, the tenant PAK and the agent name.
- The tenant does the rest with its own PAK: find its id, create a stack, target it, push manifests, delete the stack.
- A tenant PAK lists only the agents registered with it, and it cannot read agent labels, agent events or stack health. Those are admin views, and the console shows them.
- The agent pulls. It starts inactive, and it applies only the stacks of tenants it is registered with.

## Troubleshooting

- **`agent_not_registered` on the target.** The admin did not register this agent with this tenant (Step 3), or registered it with a different tenant.
- **`no agent has the name or id` from a tenant command.** A tenant sees only the agents registered with it. Do Step 3 first.
- **Nothing reaches the cluster.** The agent may still be `INACTIVE`. Its log says so on each poll. Run Step 2 again.
- **`403` with `admin_required` from a tenant command.** The endpoint is an admin view: agents, agent events, stack health. Use the admin PAK, or the console.
- **`DEPLOY SUCCESS`, but `kubectl` finds nothing.** The agent is on a different cluster than your `kubectl`, or you checked before one poll cycle passed. If the resources appear and then vanish one poll cycle later, upgrade the agent: agents up to 0.9.2 deleted a stack's resources one poll after a successful apply.
- **The tenant PAK is lost.** The admin rotates it: `POST /api/v1/generators/{id}/rotate-pak` with the admin PAK returns a new one, once.

## Next steps

- [CI/CD with Generators](./cicd-generators.md): the same tenant PAK in a pipeline, and PAK rotation
- [Multi-Cluster Targeting](./multi-cluster-targeting.md): labels instead of one target per agent
- [Submitting a Folder of Manifests (CLI)](../how-to/cli-apply.md): `brokkr apply` instead of `curl`
- [Using the Operator Console](../how-to/operator-console.md): what the admin sees
