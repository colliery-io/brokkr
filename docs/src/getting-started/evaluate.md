# Evaluate Brokkr Locally

This page gets you from zero to a working Brokkr deployment fast, so you can decide whether it fits your needs. It offers two paths — pick one:

- **[Path A — Fastest look](#path-a--fastest-look-angreal-local-up)** — one command builds the stack from source and bundles its own Kubernetes (k3s); nothing to install but Docker.
- **[Path B — Realistic evaluation](#path-b--realistic-evaluation-helm-on-a-local-cluster)** — Helm installs the published `v0.8.0` images onto a local cluster you bring yourself (kind or k3d); no source build.

Both paths end with an agent reconciling a real Kubernetes resource onto a cluster.

---

## Path A — Fastest look (`angreal local up`)

This path builds Brokkr from source and runs the full broker + agent + k3s + local registry stack in Docker. It is the quickest way to a working playground, building container images from the repo.

### Prerequisites

- **Docker** with Docker Compose
- **Git**
- **[Angreal](https://pypi.org/project/angreal/)**, the project's task runner: `pip install angreal`
- **`curl`** and **`jq`** — the verification steps below use both

The first `angreal local up` compiles the broker and agent from source — usually 5–15 minutes depending on your machine and Docker cache. Docker streams the build output to your terminal as it works, so you can watch it make progress (subsequent runs are much faster once layers are cached).

### 1. Clone and start the stack

One command brings up every backing service.

```bash
git clone https://github.com/colliery-io/brokkr.git
cd brokkr
angreal local up
```

This starts the `brokkr-dev` Docker Compose project: PostgreSQL, a local container registry, the broker, a k3s cluster (with Tekton + Shipwright), a pre-created agent, and a couple of demo containers. The first run builds images from your working tree, so give it a few minutes.

When it finishes, the broker is on `http://localhost:3000` and the stack has already created an agent named `brokkr-integration-test-agent` (cluster `brokkr-dev-integration-cluster`) and started an agent container for it.

> **If `angreal local up` fails:** make sure Docker is running and you have enough free disk for the source build, and that ports `3000`, `3001`, and `5433` aren't already in use. To start clean, run `angreal local down --hard` (removes volumes) and try again. The same stack is documented in more depth in [Local Development Environment](./development.md).

### 2. Set your admin key and confirm the broker is up

The dev broker runs with the default configuration, so the publicly known dev admin PAK (Prefixed API Key) works against it — fine for this throwaway environment, **never for production**.

```bash
export ADMIN_PAK="brokkr_BR3rVsDa_GK3QN7CDUzYc6iKgMkJ98M2WSimM5t6U8"

# Confirm the broker is healthy
curl http://localhost:3000/healthz
```

### 3. See your agent

List registered agents — the pre-created agent should already be there.

```bash
curl -s http://localhost:3000/api/v1/agents \
  -H "Authorization: Bearer $ADMIN_PAK" | jq '.[] | {id, name, cluster_name, status}'
```

You should see `brokkr-integration-test-agent`. A freshly registered agent starts with `status` `INACTIVE` — the broker only hands it deployment objects once you mark it `ACTIVE`. Save its ID, then activate it:

```bash
export AGENT_ID=$(curl -s http://localhost:3000/api/v1/agents \
  -H "Authorization: Bearer $ADMIN_PAK" \
  | jq -r '.[] | select(.name=="brokkr-integration-test-agent") | .id')

# Activate the agent so it will pull and reconcile deployment objects
curl -s -X PUT http://localhost:3000/api/v1/agents/$AGENT_ID \
  -H "Authorization: Bearer $ADMIN_PAK" \
  -H "Content-Type: application/json" \
  -d '{"status": "ACTIVE"}' | jq '{name, status}'
```

The status should now read `ACTIVE`, meaning the agent will pull and reconcile what you target to it.

### 4. Deploy something and watch it reconcile

Create a stack, target your agent to it, and push a namespace. Stacks are owned by a generator; the broker creates an `admin-generator` at first startup.

```bash
# Look up the admin generator and create a stack owned by it
GEN_ID=$(curl -s http://localhost:3000/api/v1/generators \
  -H "Authorization: Bearer $ADMIN_PAK" \
  | jq -r '.[] | select(.name=="admin-generator") | .id')

STACK_ID=$(curl -s -X POST http://localhost:3000/api/v1/stacks \
  -H "Authorization: Bearer $ADMIN_PAK" \
  -H "Content-Type: application/json" \
  -d "{\"name\": \"evaluate\", \"description\": \"Evaluation stack\", \"generator_id\": \"$GEN_ID\"}" \
  | jq -r '.id')

# Target the agent to the stack so it receives the deployment
# The body `agent_id` must match the agent id in the path — the broker rejects a mismatch with 400.
curl -s -X POST http://localhost:3000/api/v1/agents/$AGENT_ID/targets \
  -H "Authorization: Bearer $ADMIN_PAK" \
  -H "Content-Type: application/json" \
  -d "{\"agent_id\": \"$AGENT_ID\", \"stack_id\": \"$STACK_ID\"}"

# Push a deployment object (a single namespace)
curl -s -X POST "http://localhost:3000/api/v1/stacks/$STACK_ID/deployment-objects" \
  -H "Authorization: Bearer $ADMIN_PAK" \
  -H "Content-Type: application/json" \
  -d '{"yaml_content": "apiVersion: v1\nkind: Namespace\nmetadata:\n  name: brokkr-evaluate", "is_deletion_marker": false}'
```

The agent polls the broker on its next cycle and applies the namespace. Point `kubectl` at the bundled k3s cluster (its host kubeconfig is written to `/tmp/brokkr-keys/`) and verify:

```bash
export KUBECONFIG=/tmp/brokkr-keys/kubeconfig.local.yaml

# Visible result: the namespace your agent reconciled onto the cluster
kubectl get namespace brokkr-evaluate
```

Once `brokkr-evaluate` shows up, you have seen the full loop: an API call to the broker, an agent picking it up, and a real resource on a cluster.

### Optional: look at the demo UI

The stack also serves a **demo** admin UI at `http://localhost:3001` (from `examples/ui-slim`) — a demonstration of what a consumer could build on the API, not a supported product. Open it for a visual feel, but drive your actual evaluation through the API or the [`brokkr` CLI](../how-to/cli-apply.md).

### Tear down

```bash
angreal local down          # stop the stack
angreal local down --hard   # stop and remove volumes
```

---

## Path B — Realistic evaluation (Helm on a local cluster)

This path installs the **published `v0.8.0` images** with Helm onto a local Kubernetes cluster — no source build. It mirrors a real install closely enough to evaluate operational behavior.

### Prerequisites

- A local Kubernetes cluster via **[kind](https://kind.sigs.k8s.io/)** or **[k3d](https://k3d.io/)**
- **kubectl** configured to reach that cluster
- **Helm** 3.8 or later
- **`curl`** and **`jq`** — the steps below use both

### 1. Create a local cluster

Use whichever tool you have. For example, with kind:

```bash
kind create cluster --name brokkr-eval
```

Or with k3d:

```bash
k3d cluster create brokkr-eval
```

Confirm `kubectl` is pointed at it:

```bash
kubectl cluster-info
```

### 2. Install the broker

Install the broker chart with bundled PostgreSQL. The chart pulls the published `brokkr-broker` image.

```bash
helm install brokkr-broker oci://ghcr.io/colliery-io/charts/brokkr-broker \
  --version 0.8.0 \
  --set postgresql.enabled=true \
  --wait

# Verify the broker is running
kubectl get pods -l app.kubernetes.io/name=brokkr-broker
```

### 3. Reach the broker and set your admin key

Port-forward the broker, then export the default admin PAK. The default chart install ships a publicly known PAK hash, so this PAK works — fine for a throwaway eval cluster, **never for production**.

This port-forward must stay running for every remaining step — leave it in this shell (or its own terminal) until teardown.

```bash
kubectl port-forward svc/brokkr-broker 3000:3000 &

export ADMIN_PAK="brokkr_BR3rVsDa_GK3QN7CDUzYc6iKgMkJ98M2WSimM5t6U8"

# Confirm the broker answers
curl http://localhost:3000/api/v1/agents -H "Authorization: Bearer $ADMIN_PAK"
```

### 4. Create an agent and capture its PAK

The agent authenticates to the broker with its own one-time PAK.

```bash
export AGENT_PAK=$(curl -s -X POST http://localhost:3000/api/v1/agents \
  -H "Authorization: Bearer $ADMIN_PAK" \
  -H "Content-Type: application/json" \
  -d '{"name": "eval-agent", "cluster_name": "evaluation"}' \
  | jq -r '.initial_pak')

echo "$AGENT_PAK"   # shown only once
```

### 5. Install the agent

Install the agent chart with the PAK from the previous step.

```bash
helm install brokkr-agent oci://ghcr.io/colliery-io/charts/brokkr-agent \
  --version 0.8.0 \
  --set broker.url=http://brokkr-broker:3000 \
  --set broker.pak="$AGENT_PAK" \
  --wait

# Visible result: the agent pod is Running and the agent has registered
kubectl get pods -l app.kubernetes.io/name=brokkr-agent
curl -s http://localhost:3000/api/v1/agents \
  -H "Authorization: Bearer $ADMIN_PAK" | jq '.[] | {name, cluster_name, status}'
```

When the agent pod is `Running` and `eval-agent` appears in the broker's agent list, the broker and agent are talking. The agent registers as `INACTIVE`; you activate it in the next step before deploying.

### 6. Deploy a test resource and watch it reconcile

Create a stack, target the agent, and push a namespace through the broker.

```bash
export AGENT_ID=$(curl -s http://localhost:3000/api/v1/agents \
  -H "Authorization: Bearer $ADMIN_PAK" \
  | jq -r '.[] | select(.name=="eval-agent") | .id')

# Activate the agent so it will pull and reconcile deployment objects
curl -s -X PUT http://localhost:3000/api/v1/agents/$AGENT_ID \
  -H "Authorization: Bearer $ADMIN_PAK" \
  -H "Content-Type: application/json" \
  -d '{"status": "ACTIVE"}' | jq '{name, status}'

GEN_ID=$(curl -s http://localhost:3000/api/v1/generators \
  -H "Authorization: Bearer $ADMIN_PAK" \
  | jq -r '.[] | select(.name=="admin-generator") | .id')

STACK_ID=$(curl -s -X POST http://localhost:3000/api/v1/stacks \
  -H "Authorization: Bearer $ADMIN_PAK" \
  -H "Content-Type: application/json" \
  -d "{\"name\": \"evaluate\", \"description\": \"Evaluation stack\", \"generator_id\": \"$GEN_ID\"}" \
  | jq -r '.id')

# The body `agent_id` must match the agent id in the path — the broker rejects a mismatch with 400.
curl -s -X POST http://localhost:3000/api/v1/agents/$AGENT_ID/targets \
  -H "Authorization: Bearer $ADMIN_PAK" \
  -H "Content-Type: application/json" \
  -d "{\"agent_id\": \"$AGENT_ID\", \"stack_id\": \"$STACK_ID\"}"

curl -s -X POST "http://localhost:3000/api/v1/stacks/$STACK_ID/deployment-objects" \
  -H "Authorization: Bearer $ADMIN_PAK" \
  -H "Content-Type: application/json" \
  -d '{"yaml_content": "apiVersion: v1\nkind: Namespace\nmetadata:\n  name: brokkr-evaluate", "is_deletion_marker": false}'
```

After the agent's next poll, the namespace appears on your cluster:

```bash
# Visible result: the namespace your agent reconciled
kubectl get namespace brokkr-evaluate
```

### Tear down

```bash
# Stop the port-forward first (or just close the shell it runs in)
kill %1 2>/dev/null

helm uninstall brokkr-agent
helm uninstall brokkr-broker
kind delete cluster --name brokkr-eval   # or: k3d cluster delete brokkr-eval
```

---

## Next steps

Once you have seen Brokkr work, go deeper:

- **[Deploy Your First Application](../tutorials/first-deployment.md)** — a fuller guided tutorial of the deployment workflow.
- **[Submitting a Folder of Manifests (CLI)](../how-to/cli-apply.md)** — apply a whole directory of manifests with one idempotent `brokkr` command instead of curling each object.
- **[Monitoring Your Agent Fleet](../how-to/fleet-monitoring.md)** — see which agents are connected, healthy, and keeping up.
- **[Monitoring Deployment Health](../how-to/deployment-health.md)** — watch what your agents are reconciling and surface failures.
- **[Installing Brokkr](./installation.md)** — the full, production-aware Helm install with values files, real PAKs, and a hardening checklist.
