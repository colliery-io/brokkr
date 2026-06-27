# Tutorial: CI/CD with Generators

In this tutorial, you'll set up a generator — Brokkr's mechanism for CI/CD integration — and use it to push deployments from a simulated pipeline. Generators are non-admin identities with scoped permissions, designed for automation.

**What you'll learn:**

- What generators are and why they exist
- How to create a generator and manage its PAK
- How generators create and manage stacks
- How agents must register with a generator before being targeted to its stacks
- How to push deployment objects from a CI/CD pipeline
- Access control differences between admin and generator roles

**Prerequisites:**

- A running Brokkr development environment (`angreal local up`)
- Your admin PAK
- Completed the [Deploy Your First Application](./first-deployment.md) tutorial

## Step 1: Create a Generator

Generators represent automated systems (CI/CD pipelines, GitOps controllers, deployment scripts) that push resources to Brokkr. They have their own PAK (Prefixed API Key) and can only manage resources they own.

A generator is also an isolated application scope: any agent that will run a generator's stacks must first be **registered** with that generator. Registration is the agent's opt-in boundary — it prevents a stack from one application accidentally being targeted at an agent that never agreed to serve it. You'll register an agent in Step 4 before targeting. For the full picture, see [Generator Registration and Application Scopes](../explanation/security-model.md#generator-registration-and-application-scopes).

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

> **Why registration is needed only for application generators:** Behind the scenes, every new agent is automatically registered with the built-in system generator (`__system__`), which carries fleet-wide stacks that reach all agents without any explicit opt-in. The application generator you just created is different — its stacks may be proprietary or customer-specific, so an agent must explicitly register before they can be targeted at it.

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

The key rule: generators can create, update, and delete their own stacks and push deployment objects to them, but they cannot manage agents, targets, or other generators' resources. Registering an agent with a generator is likewise an admin operation (or the agent acting on itself) — a generator PAK can't register agents on its own. See the [Security Model](../explanation/security-model.md) for the complete access control matrix.

## Step 4: Register and Target an Agent (So Deployments Reach a Cluster)

Before pushing deployment objects, an agent must be targeted to the stack. Otherwise the deployment exists in the broker but no agent will apply it. Targeting has a prerequisite: the agent must first be **registered** with the stack's generator. Targeting an unregistered agent is rejected with a `403` (`agent_not_registered`) — admins can't bypass this gate either.

First, grab the default agent's ID and register it with your generator. With no `agent_id` in the body the agent would register itself; as admin you supply the `agent_id` you want to register:

```bash
AGENT_ID=$(curl -s http://localhost:3000/api/v1/agents \
  -H "Authorization: Bearer <your-admin-pak>" | jq -r '.[0].id')

curl -s -X POST "http://localhost:3000/api/v1/generators/${GENERATOR_ID}/register" \
  -H "Authorization: Bearer <your-admin-pak>" \
  -H "Content-Type: application/json" \
  -d "{\"agent_id\": \"${AGENT_ID}\"}" | jq .
```

Re-running this registration returns `409 already_registered` (harmless if the agent is already registered). Now target the agent at the stack:

```bash
curl -s -X POST "http://localhost:3000/api/v1/agents/${AGENT_ID}/targets" \
  -H "Authorization: Bearer <your-admin-pak>" \
  -H "Content-Type: application/json" \
  -d "{\"agent_id\": \"${AGENT_ID}\", \"stack_id\": \"${STACK_ID}\"}" | jq .
```

> **Note:** Generators cannot manage agents, registrations, or targets — that requires admin access (or the agent acting on itself). In production, an admin registers and targets the agent once and the generator just pushes deployments.

> **Troubleshooting:** A `403` with code `agent_not_registered` means the agent isn't registered with this stack's generator. Register it with `POST /generators/{id}/register` (as above), or start the agent with `--generator-ids {id}` so it self-registers. See the [error-codes reference](../reference/error-codes.md) and the operational [agent-registration how-to](../how-to/agent-registration.md).

### Alternative: Agent Self-Registration at Startup

Manual admin registration is convenient in a tutorial, but in production agents usually declare their generator scopes when they start, so no per-agent API call is needed. The agent resolves its scopes from, in precedence order:

1. The `--generator-ids` CLI flag (highest priority)
2. The `BROKKR__AGENT__GENERATOR_IDS` environment variable (config key `agent.generator_ids`)
3. The legacy bare `BROKKR_GENERATOR_IDS` variable (deprecated — still honored, but logs a warning)

Each is a comma-separated list of generator UUIDs; an empty value leaves the agent in system/fleet scope only. For example:

```bash
brokkr-agent start --generator-ids "${GENERATOR_ID}"
```

Self-registration happens alongside the automatic system-generator registration, so a self-registering agent still serves fleet-wide stacks. See the [environment-variables reference](../reference/environment-variables.md) for the config keys and the [agent-registration how-to](../how-to/agent-registration.md) for the full workflow.

## Step 5: Push a Deployment (Simulating CI/CD)

Now simulate what a CI/CD pipeline would do — push a deployment object after building an image:

First write the manifests to a file — in a real pipeline these come straight from your repo (or `kustomize build` / `helm template` output):

```bash
cat > myapp.yaml <<'YAML'
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
  labels:
    app: myapp
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
        image: myregistry.example.com/myapp:v1.2.3
        ports:
        - containerPort: 8080
        env:
        - name: VERSION
          value: v1.2.3
YAML
```

Then submit the file as-is. With `Content-Type: application/yaml` the raw body *is* the manifest — no JSON envelope or `\n`-escaping:

```bash
# This is what your CI/CD pipeline would run after building
curl -s -X POST "http://localhost:3000/api/v1/stacks/${STACK_ID}/deployment-objects" \
  -H "Authorization: Bearer ${GENERATOR_PAK}" \
  -H "Content-Type: application/yaml" \
  --data-binary @myapp.yaml | jq '{id, sequence_id, yaml_checksum}'
```

Each push creates a new deployment object with an incrementing `sequence_id`. The agent sees the new sequence and applies the latest version.

## Step 6: Simulate a Deployment Update

Push a new version (as a CI/CD pipeline would on the next merge):

```bash
# Bump the image tag in the same file and re-submit it.
sed -i 's/v1\.2\.3/v1.3.0/g' myapp.yaml

curl -s -X POST "http://localhost:3000/api/v1/stacks/${STACK_ID}/deployment-objects" \
  -H "Authorization: Bearer ${GENERATOR_PAK}" \
  -H "Content-Type: application/yaml" \
  --data-binary @myapp.yaml | jq '{id, sequence_id, yaml_checksum}'
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
          # Submit the rendered file as-is — application/yaml means no jq
          # escaping or JSON envelope.
          curl -sf -X POST "${BROKKR_URL}/api/v1/stacks/${STACK_ID}/deployment-objects" \
            -H "Authorization: Bearer ${{ secrets.BROKKR_GENERATOR_PAK }}" \
            -H "Content-Type: application/yaml" \
            --data-binary @deployment.yaml
```

For a non-Rust-curl pipeline, the [`brokkr` CLI](../how-to/cli-apply.md) is even simpler — `brokkr apply -f ./manifests --stack <name>` is idempotent (it re-submits only when the bundle changed), so it drops straight into a CI step.

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
- An agent must be **registered** with a generator before its stacks can be targeted at that agent — manually via the API or by self-registering at startup with `--generator-ids`
- Pushing deployment objects is as simple as a `curl` POST with YAML content
- **Sequence IDs** ensure agents always apply the latest version
- Generator PAKs should be stored as CI secrets and **rotated** periodically

## Next Steps

- [Standardized Deployments with Templates](./templates.md) — reduce YAML duplication with templates
- [Working with Generators](../how-to/generators.md) — detailed generator management guide
- [Generators Reference](../reference/generators.md) — complete API reference
- [Security Model](../explanation/security-model.md) — understand the full authorization model
