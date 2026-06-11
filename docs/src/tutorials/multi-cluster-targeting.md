# Tutorial: Multi-Cluster Targeting

In this tutorial, you'll deploy different configurations to different clusters using Brokkr's label and annotation targeting system. You'll register two agents representing two environments (staging and production), then direct deployments to each one selectively.

**What you'll learn:**

- How shared labels associate stacks with agents automatically
- How annotations attach key-value metadata that also participates in matching
- How explicit agent targets bind a stack to an agent unconditionally
- How one stack can reach multiple clusters while another targets only one

**Prerequisites:**

- A running Brokkr development environment (`angreal local up`)
- Your admin PAK
- Completed the [Deploy Your First Application](./first-deployment.md) tutorial

## Step 1: Create Two Agents

In a real deployment, each agent runs in a different Kubernetes cluster. For this tutorial, we'll create two agent records in the broker to simulate a multi-cluster setup. (You can also create agents with the `brokkr-broker` CLI — see the [CLI Reference](../reference/cli.md); in docker-based setups the CLI lives inside the broker container, e.g. `docker compose exec broker brokkr-broker create agent --name staging-agent --cluster-name staging-cluster`.)

```bash
STAGING=$(curl -s -X POST http://localhost:3000/api/v1/agents \
  -H "Authorization: Bearer <your-admin-pak>" \
  -H "Content-Type: application/json" \
  -d '{"name": "staging-agent", "cluster_name": "staging-cluster"}' | jq -r '.agent.id')

PROD=$(curl -s -X POST http://localhost:3000/api/v1/agents \
  -H "Authorization: Bearer <your-admin-pak>" \
  -H "Content-Type: application/json" \
  -d '{"name": "prod-agent", "cluster_name": "prod-cluster"}' | jq -r '.agent.id')

echo "Staging agent: $STAGING"
echo "Production agent: $PROD"
```

These two agents are database records only — no agent process is running for them, so nothing will be applied to a real cluster. That's exactly what we want here: this tutorial is about how the *broker* decides which agent receives which deployment objects.

## Step 2: Label the Agents

**Labels** are simple tags (e.g., `env:staging`, `tier:web`). **Annotations** are key-value pairs (e.g., `region=us-east-1`). Both do more than organize and filter: they drive **dynamic matching**. When the broker computes which stacks an agent is responsible for, a stack that shares *any* label string — or *any* annotation key and value — with the agent is associated with it automatically. Labels are typically used for broad categories, while annotations carry more specific values.

Add environment labels to each agent:

```bash
# Label the staging agent (agent labels take an object, unlike stack labels)
curl -s -X POST "http://localhost:3000/api/v1/agents/${STAGING}/labels" \
  -H "Authorization: Bearer <your-admin-pak>" \
  -H "Content-Type: application/json" \
  -d "{\"agent_id\": \"${STAGING}\", \"label\": \"env:staging\"}" | jq .

# Label the production agent
curl -s -X POST "http://localhost:3000/api/v1/agents/${PROD}/labels" \
  -H "Authorization: Bearer <your-admin-pak>" \
  -H "Content-Type: application/json" \
  -d "{\"agent_id\": \"${PROD}\", \"label\": \"env:production\"}" | jq .
```

Add a region annotation to the production agent:

```bash
curl -s -X POST "http://localhost:3000/api/v1/agents/${PROD}/annotations" \
  -H "Authorization: Bearer <your-admin-pak>" \
  -H "Content-Type: application/json" \
  -d "{\"agent_id\": \"${PROD}\", \"key\": \"region\", \"value\": \"us-east-1\"}" | jq .
```

Verify the labels are set:

```bash
curl -s "http://localhost:3000/api/v1/agents/${STAGING}/labels" \
  -H "Authorization: Bearer <your-admin-pak>" | jq '.[].label'

curl -s "http://localhost:3000/api/v1/agents/${PROD}/labels" \
  -H "Authorization: Bearer <your-admin-pak>" | jq '.[].label'
```

## Step 3: Create Stacks with Matching Labels

Now create two stacks — one for staging, one for production — and label them to match the agents.

> **How association works:** Agents are associated with stacks two ways, OR-ed together: (1) **dynamic matching** — a stack that shares any label string or any annotation key+value with the agent is associated at query time (no target records are created), and (2) **explicit agent targets** (Step 4) — unconditional bindings that hold regardless of labels. Once the `env:staging` label below is in place, the staging agent is already associated with the staging stack.

Stacks need an owning generator; as in the first tutorial, use the `admin-generator`:

```bash
GEN_ID=$(curl -s http://localhost:3000/api/v1/generators \
  -H "Authorization: Bearer <your-admin-pak>" \
  | jq -r '.[] | select(.name=="admin-generator") | .id')

# Create staging stack
STAGING_STACK=$(curl -s -X POST http://localhost:3000/api/v1/stacks \
  -H "Authorization: Bearer <your-admin-pak>" \
  -H "Content-Type: application/json" \
  -d "{\"name\": \"myapp-staging\", \"description\": \"My app - staging environment\", \"generator_id\": \"${GEN_ID}\"}" \
  | jq -r '.id')

# Add label to staging stack
curl -s -X POST "http://localhost:3000/api/v1/stacks/${STAGING_STACK}/labels" \
  -H "Authorization: Bearer <your-admin-pak>" \
  -H "Content-Type: application/json" \
  -d '"env:staging"' | jq .

# Create production stack
PROD_STACK=$(curl -s -X POST http://localhost:3000/api/v1/stacks \
  -H "Authorization: Bearer <your-admin-pak>" \
  -H "Content-Type: application/json" \
  -d "{\"name\": \"myapp-production\", \"description\": \"My app - production environment\", \"generator_id\": \"${GEN_ID}\"}" \
  | jq -r '.id')

# Add label to production stack
curl -s -X POST "http://localhost:3000/api/v1/stacks/${PROD_STACK}/labels" \
  -H "Authorization: Bearer <your-admin-pak>" \
  -H "Content-Type: application/json" \
  -d '"env:production"' | jq .
```

## Step 4: Target Agents to Stacks

The matching `env:*` labels already associate each agent with its stack. You can additionally pin each pairing with an **explicit agent target** — an unconditional binding that keeps working even if labels are later changed or removed. The body requires both `agent_id` and `stack_id`:

```bash
# Staging agent targets staging stack
curl -s -X POST "http://localhost:3000/api/v1/agents/${STAGING}/targets" \
  -H "Authorization: Bearer <your-admin-pak>" \
  -H "Content-Type: application/json" \
  -d "{\"agent_id\": \"${STAGING}\", \"stack_id\": \"${STAGING_STACK}\"}" | jq .

# Production agent targets production stack
curl -s -X POST "http://localhost:3000/api/v1/agents/${PROD}/targets" \
  -H "Authorization: Bearer <your-admin-pak>" \
  -H "Content-Type: application/json" \
  -d "{\"agent_id\": \"${PROD}\", \"stack_id\": \"${PROD_STACK}\"}" | jq .
```

## Step 5: Deploy to Staging Only

Push a deployment to the staging stack. Only the staging agent will receive it:

```bash
curl -s -X POST "http://localhost:3000/api/v1/stacks/${STAGING_STACK}/deployment-objects" \
  -H "Authorization: Bearer <your-admin-pak>" \
  -H "Content-Type: application/json" \
  -d '{
    "yaml_content": "apiVersion: v1\nkind: Namespace\nmetadata:\n  name: myapp-staging\n---\napiVersion: apps/v1\nkind: Deployment\nmetadata:\n  name: myapp\n  namespace: myapp-staging\nspec:\n  replicas: 1\n  selector:\n    matchLabels:\n      app: myapp\n  template:\n    metadata:\n      labels:\n        app: myapp\n    spec:\n      containers:\n      - name: myapp\n        image: nginx:1.27\n        env:\n        - name: ENVIRONMENT\n          value: staging"
  }' | jq .
```

Because these agents are database records with no running process, no deployment *events* will ever appear — events are reported by live agents after they apply resources. Instead, verify the association by asking the broker what each agent would receive. The new deployment object shows up only in the staging agent's target state:

```bash
# Staging agent's target state includes the new object
curl -s "http://localhost:3000/api/v1/agents/${STAGING}/target-state" \
  -H "Authorization: Bearer <your-admin-pak>" | jq '.[] | {stack_id, sequence_id}'

# Production agent's target state is still empty
curl -s "http://localhost:3000/api/v1/agents/${PROD}/target-state" \
  -H "Authorization: Bearer <your-admin-pak>" | jq '.[] | {stack_id, sequence_id}'
```

## Step 6: Deploy to Production

Now push to production with a different replica count:

```bash
curl -s -X POST "http://localhost:3000/api/v1/stacks/${PROD_STACK}/deployment-objects" \
  -H "Authorization: Bearer <your-admin-pak>" \
  -H "Content-Type: application/json" \
  -d '{
    "yaml_content": "apiVersion: v1\nkind: Namespace\nmetadata:\n  name: myapp-production\n---\napiVersion: apps/v1\nkind: Deployment\nmetadata:\n  name: myapp\n  namespace: myapp-production\nspec:\n  replicas: 3\n  selector:\n    matchLabels:\n      app: myapp\n  template:\n    metadata:\n      labels:\n        app: myapp\n    spec:\n      containers:\n      - name: myapp\n        image: nginx:1.27\n        env:\n        - name: ENVIRONMENT\n          value: production"
  }' | jq .
```

Re-run the target-state checks from Step 5: the production agent's target state now contains the 3-replica object, while staging still sees only its own 1-replica object — each environment gets exactly what it needs.

## Step 7: Create a Shared Stack

What if you have infrastructure (like monitoring) that should go to both environments? This stack carries no labels, so dynamic matching won't associate it with anything — explicit targets are how it reaches both agents:

```bash
# Create shared stack
SHARED_STACK=$(curl -s -X POST http://localhost:3000/api/v1/stacks \
  -H "Authorization: Bearer <your-admin-pak>" \
  -H "Content-Type: application/json" \
  -d "{\"name\": \"monitoring-shared\", \"description\": \"Monitoring stack for all clusters\", \"generator_id\": \"${GEN_ID}\"}" \
  | jq -r '.id')

# Both agents target the shared stack
curl -s -X POST "http://localhost:3000/api/v1/agents/${STAGING}/targets" \
  -H "Authorization: Bearer <your-admin-pak>" \
  -H "Content-Type: application/json" \
  -d "{\"agent_id\": \"${STAGING}\", \"stack_id\": \"${SHARED_STACK}\"}" | jq .

curl -s -X POST "http://localhost:3000/api/v1/agents/${PROD}/targets" \
  -H "Authorization: Bearer <your-admin-pak>" \
  -H "Content-Type: application/json" \
  -d "{\"agent_id\": \"${PROD}\", \"stack_id\": \"${SHARED_STACK}\"}" | jq .
```

Now any deployment object pushed to `monitoring-shared` will be applied by both agents.

## Step 8: Verify Targeting

Review each agent's explicit target list:

```bash
echo "=== Staging Agent Targets ==="
curl -s "http://localhost:3000/api/v1/agents/${STAGING}/targets" \
  -H "Authorization: Bearer <your-admin-pak>" | jq '.[].stack_id'

echo "=== Production Agent Targets ==="
curl -s "http://localhost:3000/api/v1/agents/${PROD}/targets" \
  -H "Authorization: Bearer <your-admin-pak>" | jq '.[].stack_id'
```

Staging should show two stacks (its own + shared), and production should also show two (its own + shared). To see the full association — explicit targets *and* label/annotation matches combined — ask for the agent's associated stacks instead:

```bash
curl -s "http://localhost:3000/api/v1/agents/${STAGING}/stacks" \
  -H "Authorization: Bearer <your-admin-pak>" | jq '.[].name'
```

## Clean Up

Remove the test resources:

```bash
# Delete stacks (soft delete)
curl -s -X DELETE "http://localhost:3000/api/v1/stacks/${STAGING_STACK}" \
  -H "Authorization: Bearer <your-admin-pak>"
curl -s -X DELETE "http://localhost:3000/api/v1/stacks/${PROD_STACK}" \
  -H "Authorization: Bearer <your-admin-pak>"
curl -s -X DELETE "http://localhost:3000/api/v1/stacks/${SHARED_STACK}" \
  -H "Authorization: Bearer <your-admin-pak>"

# Delete agents (soft delete)
curl -s -X DELETE "http://localhost:3000/api/v1/agents/${STAGING}" \
  -H "Authorization: Bearer <your-admin-pak>"
curl -s -X DELETE "http://localhost:3000/api/v1/agents/${PROD}" \
  -H "Authorization: Bearer <your-admin-pak>"
```

## What You've Learned

- **Labels** like `env:staging` do double duty: they categorize resources *and* dynamically associate any stack sharing a label with the agent
- **Annotations** like `region=us-east-1` participate in matching the same way, on key+value
- **Agent targets** create explicit, unconditional bindings — the only mechanism for unlabeled stacks like the shared one
- An agent's effective stack set is the **union** of label matches, annotation matches, and explicit targets
- Multiple agents can be associated with the same stack for shared infrastructure
- Use `GET /agents/{id}/stacks` or `GET /agents/{id}/target-state` to see what an agent will receive

## Next Steps

- [CI/CD with Generators](./cicd-generators.md) — automate deployments from pipelines
- [Submitting a Folder of Manifests (CLI)](../how-to/cli-apply.md) — `brokkr apply -f ./manifests --stack <name> --target-label <k:v>` applies the same label-based fan-out from a single idempotent command
- [Standardized Deployments with Templates](./templates.md) — use templates to reduce YAML duplication across environments
- [Core Concepts](../explanation/core-concepts.md) — deeper understanding of the targeting model
