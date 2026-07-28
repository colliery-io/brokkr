# Tutorials

Step-by-step tutorials for learning Brokkr. Each tutorial walks you through a complete workflow from start to finish, building practical skills along the way.

Every tutorial is driven with `curl` and `jq` against the broker's HTTP API, so none of them needs a source checkout or a Rust toolchain. What they do need is a broker you can reach and an admin PAK — and, for two of them, an agent.

## Which Brokkr are you practising against?

The tutorials are written against a broker at `http://localhost:3000`. Two setups get you one:

| Setup | Who it's for | What you get |
|-------|--------------|--------------|
| [Helm install](../getting-started/installation.md) | Anyone running Brokkr in their own cluster | A broker you reach with `kubectl port-forward svc/brokkr-broker 3000:3000`, an admin PAK you minted yourself, and one agent named after the `broker.agentName` you installed it with |
| [Local development environment](../getting-started/development.md) (`angreal local up`) | Contributors working on Brokkr itself | A broker already published on `localhost:3000`, a pre-created agent called `brokkr-integration-test-agent`, and the publicly known development admin PAK |

Both work for every tutorial below. The commands are identical; only three values differ, and [Adapting the commands to your install](#adapting-the-commands-to-your-install) covers all three.

## The tutorials

| Tutorial | What You'll Learn | What it needs |
|----------|-------------------|---------------|
| [Deploy Your First Application](./first-deployment.md) | Create a stack, add a deployment object, register an agent, and watch Kubernetes resources get applied | Broker + admin PAK + **a running agent** attached to a cluster — it is the one tutorial that verifies its results with `kubectl` |
| [Multi-Cluster Targeting](./multi-cluster-targeting.md) | Use labels and annotations to direct deployments to specific agents | Broker + admin PAK only — it creates its own two agent *records* and never needs an agent process |
| [CI/CD with Generators](./cicd-generators.md) | Create a generator and use it from a CI/CD pipeline to push deployments | Broker + admin PAK + **one existing agent** (a running one if you want to watch the deployment land on a cluster) |
| [Standardized Deployments with Templates](./templates.md) | Create reusable templates with JSON Schema validation and instantiate them across stacks | Broker + admin PAK only — templates are rendered broker-side, no agent involved |

Start with [Deploy Your First Application](./first-deployment.md). The other three assume you have met stacks, deployment objects, and targets there, but they do not depend on any resource it creates.

## Adapting the commands to your install

### 1. The broker URL

Every command targets `http://localhost:3000`. The development environment publishes the broker there directly. After a Helm install, forward the service first and leave it running in another terminal:

```bash
kubectl port-forward svc/brokkr-broker 3000:3000
```

If your broker is exposed through an Ingress instead, substitute that base URL everywhere `http://localhost:3000` appears.

### 2. The admin PAK

The tutorials write `<your-admin-pak>` in the `Authorization: Bearer` header. Export it once and use `$ADMIN_PAK` if you prefer:

- **Development environment:** the publicly known development PAK `brokkr_BR3rVsDa_GK3QN7CDUzYc6iKgMkJ98M2WSimM5t6U8`, which comes from the embedded default `broker.pak_hash`.
- **Helm install:** the PAK you minted with `brokkr-broker generate-pak` and whose hash you passed as `broker.pakHash` / `broker.pakHashExistingSecret`. You do not need the binary installed — the published broker image runs the command offline, with no database and no cluster:

  ```bash
  docker run --rm ghcr.io/colliery-io/brokkr-broker:latest generate-pak
  ```

  See [Get the Admin PAK](../getting-started/installation.md#3-get-the-admin-pak). The development PAK above will **not** authenticate against a broker configured with your own hash — and if it does, your broker is still running the publicly known default credential and needs fixing before anything else.

### 3. The agent name

The two tutorials that need an agent look one up by name. The development environment pre-creates `brokkr-integration-test-agent`; a Helm-installed agent is named by the `broker.agentName` you set at install time (`my-agent` in the installation guide). Export the right one up front and the tutorial commands work unchanged:

```bash
# Development environment
export AGENT_NAME=brokkr-integration-test-agent

# Helm install — whatever you passed as broker.agentName
export AGENT_NAME=my-agent
```

If you are not sure, list what the broker knows about:

```bash
curl -s http://localhost:3000/api/v1/agents \
  -H "Authorization: Bearer <your-admin-pak>" | jq '.[] | {name, cluster_name, status}'
```

A Helm install has no agent at all until you create one through the API and install the agent chart with its PAK — [Quick Start steps 4 and 5](../getting-started/installation.md#4-create-an-agent-and-get-its-pak) walk through it.

### 4. How long a poll cycle takes

Tutorials tell you to wait "one poll cycle" before checking the cluster. That is about **10 seconds** in the development environment (the agent binary's own default) and **30 seconds** for a Helm-installed agent (the chart's `agent.pollingInterval` default).
