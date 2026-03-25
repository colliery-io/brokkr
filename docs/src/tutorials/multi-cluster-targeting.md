# Tutorial: Multi-Cluster Targeting

In this tutorial, you'll deploy different configurations to different clusters using Brokkr's label and annotation targeting system. You'll register two agents representing two environments (staging and production), then direct deployments to each one selectively.

**What you'll learn:**

- How labels categorize agents and stacks
- How annotations attach key-value metadata
- How targeting rules match deployments to specific agents
- How one stack can reach multiple clusters while another targets only one

**Prerequisites:**

- A running Brokkr development environment (`angreal local up`)
- Your admin PAK
- Completed the [Deploy Your First Application](./first-deployment.md) tutorial

## Step 1: Create Two Agents

In a real deployment, each agent runs in a different Kubernetes cluster. For this tutorial, we'll create two agent records in the broker to simulate a multi-cluster setup.

```bash
# Create a staging agent
brokkr-broker create agent --name staging-agent --cluster_name staging-cluster
```

Note the PAK printed for the staging agent. Then create the production agent:

```bash
# Create a production agent
brokkr-broker create agent --name prod-agent --cluster_name prod-cluster
```

If you're using the API instead of the CLI:

```bash
STAGING=$(curl -s -X POST http://localhost:3000/api/v1/agents \
  -H "Authorization: <your-admin-pak>" \
  -H "Content-Type: application/json" \
  -d '{"name": "staging-agent", "cluster_name": "staging-cluster"}' | jq -r '.agent.id')

PROD=$(curl -s -X POST http://localhost:3000/api/v1/agents \
  -H "Authorization: <your-admin-pak>" \
  -H "Content-Type: application/json" \
  -d '{"name": "prod-agent", "cluster_name": "prod-cluster"}' | jq -r '.agent.id')

echo "Staging agent: $STAGING"
echo "Production agent: $PROD"
```

## Step 2: Label the Agents

**Labels** are simple tags that categorize agents (e.g., `env:staging`, `tier:web`). **Annotations** are key-value metadata pairs (e.g., `region=us-east-1`). Both are used for organizing and filtering agents. Labels are typically used for broad categories, while annotations carry specific configuration values.

Add environment labels to each agent:

```bash
# Label the staging agent
curl -s -X POST "http://localhost:3000/api/v1/agents/${STAGING}/labels" \
  -H "Authorization: <your-admin-pak>" \
  -H "Content-Type: application/json" \
  -d '"env:staging"' | jq .

# Label the production agent
curl -s -X POST "http://localhost:3000/api/v1/agents/${PROD}/labels" \
  -H "Authorization: <your-admin-pak>" \
  -H "Content-Type: application/json" \
  -d '"env:production"' | jq .
```

Add a region annotation to the production agent:

```bash
curl -s -X POST "http://localhost:3000/api/v1/agents/${PROD}/annotations" \
  -H "Authorization: <your-admin-pak>" \
  -H "Content-Type: application/json" \
  -d '{"key": "region", "value": "us-east-1"}' | jq .
```

Verify the labels are set:

```bash
curl -s "http://localhost:3000/api/v1/agents/${STAGING}/labels" \
  -H "Authorization: <your-admin-pak>" | jq '.[].label'

curl -s "http://localhost:3000/api/v1/agents/${PROD}/labels" \
  -H "Authorization: <your-admin-pak>" | jq '.[].label'
```

## Step 3: Create Stacks with Matching Labels

Now create two stacks — one for staging, one for production — and label them to match the agents.

> **Important:** Labels on agents and stacks are metadata for organization and filtering. The actual connection between an agent and a stack is created by **agent targets** (Step 4). Labels help you *categorize* resources; targets tell the agent "deploy this stack's resources."

```bash
# Create staging stack
STAGING_STACK=$(curl -s -X POST http://localhost:3000/api/v1/stacks \
  -H "Authorization: <your-admin-pak>" \
  -H "Content-Type: application/json" \
  -d '{"name": "myapp-staging", "description": "My app - staging environment", "generator_id": "00000000-0000-0000-0000-000000000000"}' \
  | jq -r '.id')

# Add label to staging stack
curl -s -X POST "http://localhost:3000/api/v1/stacks/${STAGING_STACK}/labels" \
  -H "Authorization: <your-admin-pak>" \
  -H "Content-Type: application/json" \
  -d '"env:staging"' | jq .

# Create production stack
PROD_STACK=$(curl -s -X POST http://localhost:3000/api/v1/stacks \
  -H "Authorization: <your-admin-pak>" \
  -H "Content-Type: application/json" \
  -d '{"name": "myapp-production", "description": "My app - production environment", "generator_id": "00000000-0000-0000-0000-000000000000"}' \
  | jq -r '.id')

# Add label to production stack
curl -s -X POST "http://localhost:3000/api/v1/stacks/${PROD_STACK}/labels" \
  -H "Authorization: <your-admin-pak>" \
  -H "Content-Type: application/json" \
  -d '"env:production"' | jq .
```

## Step 4: Target Agents to Stacks

Connect each agent to its corresponding stack:

```bash
# Staging agent targets staging stack
curl -s -X POST "http://localhost:3000/api/v1/agents/${STAGING}/targets" \
  -H "Authorization: <your-admin-pak>" \
  -H "Content-Type: application/json" \
  -d "{\"stack_id\": \"${STAGING_STACK}\"}" | jq .

# Production agent targets production stack
curl -s -X POST "http://localhost:3000/api/v1/agents/${PROD}/targets" \
  -H "Authorization: <your-admin-pak>" \
  -H "Content-Type: application/json" \
  -d "{\"stack_id\": \"${PROD_STACK}\"}" | jq .
```

## Step 5: Deploy to Staging Only

Push a deployment to the staging stack. Only the staging agent will receive it:

```bash
curl -s -X POST "http://localhost:3000/api/v1/stacks/${STAGING_STACK}/deployment-objects" \
  -H "Authorization: <your-admin-pak>" \
  -H "Content-Type: application/json" \
  -d '{
    "yaml_content": "apiVersion: v1\nkind: Namespace\nmetadata:\n  name: myapp-staging\n---\napiVersion: apps/v1\nkind: Deployment\nmetadata:\n  name: myapp\n  namespace: myapp-staging\nspec:\n  replicas: 1\n  selector:\n    matchLabels:\n      app: myapp\n  template:\n    metadata:\n      labels:\n        app: myapp\n    spec:\n      containers:\n      - name: myapp\n        image: nginx:1.27\n        env:\n        - name: ENVIRONMENT\n          value: staging"
  }' | jq .
```

Check agent events — only the staging agent should have a deployment event:

```bash
# Staging agent should have an event
curl -s "http://localhost:3000/api/v1/agents/${STAGING}/events" \
  -H "Authorization: <your-admin-pak>" | jq 'length'

# Production agent should have no new events
curl -s "http://localhost:3000/api/v1/agents/${PROD}/events" \
  -H "Authorization: <your-admin-pak>" | jq 'length'
```

## Step 6: Deploy to Production

Now push to production with a different replica count:

```bash
curl -s -X POST "http://localhost:3000/api/v1/stacks/${PROD_STACK}/deployment-objects" \
  -H "Authorization: <your-admin-pak>" \
  -H "Content-Type: application/json" \
  -d '{
    "yaml_content": "apiVersion: v1\nkind: Namespace\nmetadata:\n  name: myapp-production\n---\napiVersion: apps/v1\nkind: Deployment\nmetadata:\n  name: myapp\n  namespace: myapp-production\nspec:\n  replicas: 3\n  selector:\n    matchLabels:\n      app: myapp\n  template:\n    metadata:\n      labels:\n        app: myapp\n    spec:\n      containers:\n      - name: myapp\n        image: nginx:1.27\n        env:\n        - name: ENVIRONMENT\n          value: production"
  }' | jq .
```

The production agent receives 3 replicas while staging has 1 — each environment gets exactly what it needs.

## Step 7: Create a Shared Stack

What if you have infrastructure (like monitoring) that should go to both environments? Create a stack that both agents target:

```bash
# Create shared stack
SHARED_STACK=$(curl -s -X POST http://localhost:3000/api/v1/stacks \
  -H "Authorization: <your-admin-pak>" \
  -H "Content-Type: application/json" \
  -d '{"name": "monitoring-shared", "description": "Monitoring stack for all clusters", "generator_id": "00000000-0000-0000-0000-000000000000"}' \
  | jq -r '.id')

# Both agents target the shared stack
curl -s -X POST "http://localhost:3000/api/v1/agents/${STAGING}/targets" \
  -H "Authorization: <your-admin-pak>" \
  -H "Content-Type: application/json" \
  -d "{\"stack_id\": \"${SHARED_STACK}\"}" | jq .

curl -s -X POST "http://localhost:3000/api/v1/agents/${PROD}/targets" \
  -H "Authorization: <your-admin-pak>" \
  -H "Content-Type: application/json" \
  -d "{\"stack_id\": \"${SHARED_STACK}\"}" | jq .
```

Now any deployment object pushed to `monitoring-shared` will be applied by both agents.

## Step 8: Verify Targeting

Review each agent's full target list:

```bash
echo "=== Staging Agent Targets ==="
curl -s "http://localhost:3000/api/v1/agents/${STAGING}/targets" \
  -H "Authorization: <your-admin-pak>" | jq '.[].stack_id'

echo "=== Production Agent Targets ==="
curl -s "http://localhost:3000/api/v1/agents/${PROD}/targets" \
  -H "Authorization: <your-admin-pak>" | jq '.[].stack_id'
```

Staging should show two stacks (its own + shared), and production should also show two (its own + shared).

## Clean Up

Remove the test resources:

```bash
# Delete stacks (soft delete)
curl -s -X DELETE "http://localhost:3000/api/v1/stacks/${STAGING_STACK}" \
  -H "Authorization: <your-admin-pak>"
curl -s -X DELETE "http://localhost:3000/api/v1/stacks/${PROD_STACK}" \
  -H "Authorization: <your-admin-pak>"
curl -s -X DELETE "http://localhost:3000/api/v1/stacks/${SHARED_STACK}" \
  -H "Authorization: <your-admin-pak>"

# Delete agents (soft delete)
curl -s -X DELETE "http://localhost:3000/api/v1/agents/${STAGING}" \
  -H "Authorization: <your-admin-pak>"
curl -s -X DELETE "http://localhost:3000/api/v1/agents/${PROD}" \
  -H "Authorization: <your-admin-pak>"
```

## What You've Learned

- **Labels** categorize agents and stacks with simple tags like `env:staging`
- **Annotations** attach key-value metadata like `region=us-east-1`
- **Agent targets** create explicit bindings between agents and stacks
- An agent only receives deployment objects from stacks it targets
- Multiple agents can target the same stack for shared infrastructure
- One agent can target multiple stacks for layered deployments

## Next Steps

- [CI/CD with Generators](./cicd-generators.md) — automate deployments from pipelines
- [Standardized Deployments with Templates](./templates.md) — use templates to reduce YAML duplication across environments
- [Core Concepts](../explanation/core-concepts.md) — deeper understanding of the targeting model
