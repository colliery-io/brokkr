# Tutorial: CI/CD with Generators

In this tutorial, you'll set up a generator — Brokkr's mechanism for CI/CD integration — and use it to push deployments from a simulated pipeline. Generators are non-admin identities with scoped permissions, designed for automation.

**What you'll learn:**

- What generators are and why they exist
- How to create a generator and manage its PAK
- How generators create and manage stacks
- How to push deployment objects from a CI/CD pipeline
- Access control differences between admin and generator roles

**Prerequisites:**

- A running Brokkr development environment (`angreal local up`)
- Your admin PAK
- Completed the [Deploy Your First Application](./first-deployment.md) tutorial

## Step 1: Create a Generator

Generators represent automated systems (CI/CD pipelines, GitOps controllers, deployment scripts) that push resources to Brokkr. They have their own PAK (Prefixed API Key) and can only manage resources they own.

Create a generator via the API (the `brokkr-broker` CLI can do this too — see the [CLI Reference](../reference/cli.md)):

```bash
GENERATOR_RESPONSE=$(curl -s -X POST http://localhost:3000/api/v1/generators \
  -H "Authorization: Bearer <your-admin-pak>" \
  -H "Content-Type: application/json" \
  -d '{"name": "github-actions", "description": "GitHub Actions deployment pipeline"}')

GENERATOR_ID=$(echo "$GENERATOR_RESPONSE" | jq -r '.generator.id')
GENERATOR_PAK=$(echo "$GENERATOR_RESPONSE" | jq -r '.pak')

echo "Generator ID: $GENERATOR_ID"
echo "Generator PAK: $GENERATOR_PAK"
```

**Save this PAK immediately** — it's only shown once. You'll store it as a CI secret.

## Step 2: Create a Stack as the Generator

Switch to using the generator's PAK. The generator can create stacks, and those stacks become "owned" by the generator. The request must carry a `generator_id`, and a generator PAK may only pass its **own** ID — anything else is rejected with a `403`:

```bash
STACK_ID=$(curl -s -X POST http://localhost:3000/api/v1/stacks \
  -H "Authorization: Bearer ${GENERATOR_PAK}" \
  -H "Content-Type: application/json" \
  -d "{\"name\": \"myapp-v2\", \"description\": \"My application deployed via CI/CD\", \"generator_id\": \"${GENERATOR_ID}\"}" \
  | jq -r '.id')

echo "Stack ID: $STACK_ID"
```

The stack's `generator_id` field is set to your generator's ID. This ownership means:

- The generator can update and delete this stack
- The generator can push deployment objects to this stack
- Other generators **cannot** modify this stack
- Admins can still manage any stack

## Step 3: See Generator Permissions in Action

Generators have scoped access — they can only manage resources they own. Let's see this in practice:

```bash
# Generator can list its own stacks
curl -s http://localhost:3000/api/v1/stacks \
  -H "Authorization: Bearer ${GENERATOR_PAK}" | jq '.[].name'

# Generator CANNOT list agents (admin-only)
curl -s http://localhost:3000/api/v1/agents \
  -H "Authorization: Bearer ${GENERATOR_PAK}"
# Returns: 403 Forbidden
```

The key rule: generators can create, update, and delete their own stacks and push deployment objects to them, but they cannot manage agents, targets, or other generators' resources. See the [Security Model](../explanation/security-model.md) for the complete access control matrix.

## Step 4: Target an Agent (So Deployments Reach a Cluster)

Before pushing deployment objects, an agent must be targeted to the stack. Otherwise the deployment exists in the broker but no agent will apply it. As an admin, target the default agent:

```bash
AGENT_ID=$(curl -s http://localhost:3000/api/v1/agents \
  -H "Authorization: Bearer <your-admin-pak>" | jq -r '.[0].id')

curl -s -X POST "http://localhost:3000/api/v1/agents/${AGENT_ID}/targets" \
  -H "Authorization: Bearer <your-admin-pak>" \
  -H "Content-Type: application/json" \
  -d "{\"agent_id\": \"${AGENT_ID}\", \"stack_id\": \"${STACK_ID}\"}" | jq .
```

> **Note:** Generators cannot manage agents or targets — that requires admin access. In production, an admin sets up the targeting once and the generator just pushes deployments.

## Step 5: Push a Deployment (Simulating CI/CD)

Now simulate what a CI/CD pipeline would do — push a deployment object after building an image:

```bash
# This is what your CI/CD pipeline would run after building
curl -s -X POST "http://localhost:3000/api/v1/stacks/${STACK_ID}/deployment-objects" \
  -H "Authorization: Bearer ${GENERATOR_PAK}" \
  -H "Content-Type: application/json" \
  -d '{
    "yaml_content": "apiVersion: v1\nkind: Namespace\nmetadata:\n  name: myapp\n---\napiVersion: apps/v1\nkind: Deployment\nmetadata:\n  name: myapp\n  namespace: myapp\n  labels:\n    app: myapp\nspec:\n  replicas: 2\n  selector:\n    matchLabels:\n      app: myapp\n  template:\n    metadata:\n      labels:\n        app: myapp\n    spec:\n      containers:\n      - name: myapp\n        image: myregistry.example.com/myapp:v1.2.3\n        ports:\n        - containerPort: 8080\n        env:\n        - name: VERSION\n          value: v1.2.3"
  }' | jq '{id, sequence_id, yaml_checksum}'
```

Each push creates a new deployment object with an incrementing `sequence_id`. The agent sees the new sequence and applies the latest version.

## Step 6: Simulate a Deployment Update

Push a new version (as a CI/CD pipeline would on the next merge):

```bash
curl -s -X POST "http://localhost:3000/api/v1/stacks/${STACK_ID}/deployment-objects" \
  -H "Authorization: Bearer ${GENERATOR_PAK}" \
  -H "Content-Type: application/json" \
  -d '{
    "yaml_content": "apiVersion: v1\nkind: Namespace\nmetadata:\n  name: myapp\n---\napiVersion: apps/v1\nkind: Deployment\nmetadata:\n  name: myapp\n  namespace: myapp\n  labels:\n    app: myapp\nspec:\n  replicas: 2\n  selector:\n    matchLabels:\n      app: myapp\n  template:\n    metadata:\n      labels:\n        app: myapp\n    spec:\n      containers:\n      - name: myapp\n        image: myregistry.example.com/myapp:v1.3.0\n        ports:\n        - containerPort: 8080\n        env:\n        - name: VERSION\n          value: v1.3.0"
  }' | jq '{id, sequence_id, yaml_checksum}'
```

Notice the `sequence_id` incremented. The agent will apply this new version.

## Step 7: A Real GitHub Actions Workflow

Here's how you'd integrate Brokkr into a real GitHub Actions pipeline:

```yaml
# .github/workflows/deploy.yml
name: Deploy to Brokkr

on:
  push:
    branches: [main]

env:
  BROKKR_URL: https://brokkr.example.com
  STACK_ID: "a1b2c3d4-e5f6-7890-abcd-ef1234567890"

jobs:
  build-and-deploy:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - name: Build and push image
        run: |
          docker build -t myregistry.example.com/myapp:${{ github.sha }} .
          docker push myregistry.example.com/myapp:${{ github.sha }}

      - name: Generate deployment YAML
        run: |
          cat > deployment.yaml << 'YAML'
          apiVersion: v1
          kind: Namespace
          metadata:
            name: myapp
          ---
          apiVersion: apps/v1
          kind: Deployment
          metadata:
            name: myapp
            namespace: myapp
          spec:
            replicas: 2
            selector:
              matchLabels:
                app: myapp
            template:
              metadata:
                labels:
                  app: myapp
              spec:
                containers:
                - name: myapp
                  image: myregistry.example.com/myapp:${{ github.sha }}
                  ports:
                  - containerPort: 8080
          YAML

      - name: Push to Brokkr
        run: |
          YAML_CONTENT=$(cat deployment.yaml | jq -Rs .)
          curl -sf -X POST "${BROKKR_URL}/api/v1/stacks/${STACK_ID}/deployment-objects" \
            -H "Authorization: Bearer ${{ secrets.BROKKR_GENERATOR_PAK }}" \
            -H "Content-Type: application/json" \
            -d "{\"yaml_content\": ${YAML_CONTENT}}"
```

Store the generator PAK as `BROKKR_GENERATOR_PAK` in your repository's GitHub Actions secrets.

## Step 8: Rotate the Generator PAK

PAKs should be rotated periodically (also possible via the `brokkr-broker` CLI — see the [PAK Management guide](../how-to/pak-management.md)):

```bash
GENERATOR_PAK=$(curl -s -X POST "http://localhost:3000/api/v1/generators/${GENERATOR_ID}/rotate-pak" \
  -H "Authorization: Bearer ${GENERATOR_PAK}" | jq -r '.pak')
echo "New PAK: ${GENERATOR_PAK}"
```

The response contains the new PAK (captured above so the cleanup step below keeps working). Update your CI secrets immediately — the old PAK stops working the moment rotation succeeds.

## Clean Up

```bash
# Delete the stack (as generator)
curl -s -X DELETE "http://localhost:3000/api/v1/stacks/${STACK_ID}" \
  -H "Authorization: Bearer ${GENERATOR_PAK}"

# Delete the generator (requires admin)
curl -s -X DELETE "http://localhost:3000/api/v1/generators/${GENERATOR_ID}" \
  -H "Authorization: Bearer <your-admin-pak>"
```

## What You've Learned

- **Generators** are scoped identities for CI/CD pipeline integration
- Each generator gets its own **PAK** for authentication
- Generators **own** the stacks they create — other generators can't modify them
- Pushing deployment objects is as simple as a `curl` POST with YAML content
- **Sequence IDs** ensure agents always apply the latest version
- Generator PAKs should be stored as CI secrets and **rotated** periodically

## Next Steps

- [Standardized Deployments with Templates](./templates.md) — reduce YAML duplication with templates
- [Working with Generators](../how-to/generators.md) — detailed generator management guide
- [Generators Reference](../reference/generators.md) — complete API reference
- [Security Model](../explanation/security-model.md) — understand the full authorization model
