# Working with Generators

A generator is an identity principal — a CI/CD pipeline, automation tool, service, or team — that creates and manages Brokkr resources. Each generator receives a Prefixed API Key (PAK) scoped to itself: it can create stacks, templates, and deployment objects, but can only access resources it created. Unlike the admin PAK, a generator PAK cannot perform administrative operations such as creating other generators or managing agents.

**The generator is also Brokkr's tenant.** Creating a generator and handing over its PAK is how you onboard a team or an application onto a shared broker: its stacks are its own, its templates are private to it, and an agent only receives its stacks after registering with it. Generators are what the operator console's tenant scope selector lists, and what `?pak_id=` narrows admin listings to. If you are standing up a multi-team install, start from [Multi-Tenant Setup](./multi-tenant-setup.md), which uses the steps below as its onboarding path; for the tenancy rules themselves see [Multi-Tenancy](../reference/multi-tenancy.md).

This guide covers creating generators, integrating them with CI/CD systems, registering agents, and managing their lifecycle.

## Prerequisites

- Admin PAK for creating and managing generators
- Access to the Brokkr broker API
- CI/CD system or automation tool to configure

## Creating a Generator

### Step 1: Create the Generator

Create a new generator using the admin PAK:

```bash
curl -X POST "http://broker:3000/api/v1/generators" \
  -H "Authorization: Bearer $ADMIN_PAK" \
  -H "Content-Type: application/json" \
  -d '{
    "name": "github-actions-prod",
    "description": "Production deployment pipeline"
  }'
```

The response includes the generator details and its PAK:

```json
{
  "generator": {
    "id": "a1b2c3d4-e5f6-7890-abcd-ef1234567890",
    "name": "github-actions-prod",
    "description": "Production deployment pipeline",
    "created_at": "2025-01-02T10:00:00Z",
    "updated_at": "2025-01-02T10:00:00Z"
  },
  "pak": "brokkr_BRgen12ab_GeneratorLongTokenExample01"
}
```

### Step 2: Store the PAK Securely

The PAK is only returned once at creation time, and the generator's UUID is what stack creation requests must reference as `generator_id`. Store both immediately in your secret management system — never in source code, configuration files, or logs:

- **GitHub Actions**: Add as a repository or organization secret
- **GitLab CI**: Add as a protected variable
- **Jenkins**: Store in credentials manager
- **Vault/AWS Secrets Manager**: Store with appropriate access policies

If you lose the PAK, you'll need to rotate it (see PAK Rotation below).

If you have the PAK but not the ID, read it back from the credential itself — the `generator` field is the tenant ID:

```bash
curl -s -X POST "http://broker:3000/api/v1/auth/pak" \
  -H "Authorization: Bearer $GENERATOR_PAK" | jq -r '.generator'
```

### Step 3: Confirm It Appears as a Tenant

A new generator is immediately a tenant. Confirm it shows up in the tenant list that backs the operator console's scope selector (admin credential required):

```bash
curl -s "http://broker:3000/api/v1/paks" \
  -H "Authorization: Bearer $ADMIN_PAK"
```

```json
[
  { "id": "a1b2c3d4-e5f6-7890-abcd-ef1234567890", "name": "github-actions-prod" }
]
```

The path is `/api/v1/paks`, not `/api/v1/auth/paks`. Operators can then narrow the admin fleet, stack, and agent-event listings to this generator with `?pak_id=<generator-id>`. That parameter is a display filter for an admin who could already see everything — it is not an access boundary, so never substitute it for handing a team its own generator PAK.

## CI/CD Integration

When a generator creates a stack, the request body must include `generator_id` set to the generator's own ID — the broker rejects any other value for a generator PAK.

### GitHub Actions Example

Configure your workflow to deploy through Brokkr:

```yaml
name: Deploy to Production
on:
  push:
    branches: [main]

jobs:
  deploy:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - name: Create Stack
        env:
          BROKKR_PAK: ${{ secrets.BROKKR_GENERATOR_PAK }}
          BROKKR_GENERATOR_ID: ${{ secrets.BROKKR_GENERATOR_ID }}
          BROKKR_URL: ${{ vars.BROKKR_URL }}
        run: |
          curl -X POST "$BROKKR_URL/api/v1/stacks" \
            -H "Authorization: Bearer $BROKKR_PAK" \
            -H "Content-Type: application/json" \
            -o stack-response.json \
            -d '{
              "name": "my-app-${{ github.sha }}",
              "description": "Deployed from commit ${{ github.sha }}",
              "generator_id": "'"$BROKKR_GENERATOR_ID"'"
            }'

      - name: Add Deployment Objects
        env:
          BROKKR_PAK: ${{ secrets.BROKKR_GENERATOR_PAK }}
          BROKKR_URL: ${{ vars.BROKKR_URL }}
        run: |
          STACK_ID=$(cat stack-response.json | jq -r '.id')
          curl -X POST "$BROKKR_URL/api/v1/stacks/$STACK_ID/deployment-objects" \
            -H "Authorization: Bearer $BROKKR_PAK" \
            -H "Content-Type: application/json" \
            -d @deployment.json
```

### GitLab CI Example

```yaml
deploy:
  stage: deploy
  script:
    - |
      curl -X POST "$BROKKR_URL/api/v1/stacks" \
        -H "Authorization: Bearer $BROKKR_GENERATOR_PAK" \
        -H "Content-Type: application/json" \
        -d "{
          \"name\": \"my-app-$CI_COMMIT_SHA\",
          \"description\": \"Pipeline $CI_PIPELINE_ID\",
          \"generator_id\": \"$BROKKR_GENERATOR_ID\"
        }"
  only:
    - main
```

### Using Templates

Generators can create stack templates and instantiate them into stacks they own. A template carries `template_content` (Tera-templated YAML) and `parameters_schema` (a JSON Schema string); instantiation renders the template into a deployment object within an existing stack:

```bash
# Create a template (using generator PAK)
curl -X POST "http://broker:3000/api/v1/templates" \
  -H "Authorization: Bearer $GENERATOR_PAK" \
  -H "Content-Type: application/json" \
  -d '{
    "name": "web-service",
    "description": "Standard web service deployment",
    "template_content": "...",
    "parameters_schema": "..."
  }'

# Instantiate the template into an existing stack
curl -X POST "http://broker:3000/api/v1/stacks/$STACK_ID/deployment-objects/from-template" \
  -H "Authorization: Bearer $GENERATOR_PAK" \
  -H "Content-Type: application/json" \
  -d '{
    "template_id": "'"$TEMPLATE_ID"'",
    "parameters": {
      "replicas": 3,
      "image": "myapp:v1.2.3"
    }
  }'
```

The instantiation endpoint returns `201 Created` with the rendered deployment object. See [Stack Templates](./templates.md) for the full workflow.

A generator can read and instantiate its own templates plus admin-owned system templates; another generator's template returns `403 template_not_accessible`. Publish anything meant to be shared across teams as a system template using the admin PAK.

## Managing Generators

### List Generators

View all generators (admin only):

```bash
curl "http://broker:3000/api/v1/generators" \
  -H "Authorization: Bearer $ADMIN_PAK"
```

### Get Generator Details

A generator can view its own details:

```bash
curl "http://broker:3000/api/v1/generators/$GENERATOR_ID" \
  -H "Authorization: Bearer $GENERATOR_PAK"
```

### Update Generator

The update endpoint takes the complete generator object (`PUT` replaces, it does not patch), so fetch the current record, modify the fields you want, and send the whole object back:

```bash
curl -s "http://broker:3000/api/v1/generators/$GENERATOR_ID" \
  -H "Authorization: Bearer $GENERATOR_PAK" \
  | jq '.description = "Updated description"' \
  | curl -X PUT "http://broker:3000/api/v1/generators/$GENERATOR_ID" \
      -H "Authorization: Bearer $GENERATOR_PAK" \
      -H "Content-Type: application/json" \
      -d @-
```

### Delete Generator

Soft-delete a generator (admin or generator itself):

```bash
curl -X DELETE "http://broker:3000/api/v1/generators/$GENERATOR_ID" \
  -H "Authorization: Bearer $ADMIN_PAK"
```

Deleting a generator cascades the soft-delete to all stacks owned by the generator and their deployment objects. This is handled by a database trigger, so the cascade is atomic.

## Registering Agents

An agent must be registered with a generator before any of that generator's stacks reach it. Registration is the agent's opt-in consent boundary, and it applies to every way a stack can be associated with an agent:

- Creating an explicit target for an unregistered agent returns `403 agent_not_registered`, and admin cannot bypass it.
- A shared label or annotation only associates a stack with an agent that is registered with the stack's owning generator. An agent with no registrations receives nothing from label or annotation matching, and two generators can safely reuse the same label vocabulary without crossing into each other.

Registration is performed by an admin or by the agent itself — a generator PAK cannot register agents on its own behalf. This section covers registration from the admin's side; for the full operational workflow (including agent self-registration at startup) see [Agent Registration](./agent-registration.md).

### Step 1: Register an Agent

Register an agent with the generator (admin, or the agent acting on itself — a generator PAK is rejected):

```bash
curl -X POST "http://broker:3000/api/v1/generators/$GENERATOR_ID/register" \
  -H "Authorization: Bearer $ADMIN_PAK" \
  -H "Content-Type: application/json" \
  -d '{
    "agent_id": "f1e2d3c4-b5a6-7890-abcd-ef0123456789"
  }'
```

The response is the registration record:

```json
{
  "id": "9a8b7c6d-5e4f-3210-fedc-ba9876543210",
  "agent_id": "f1e2d3c4-b5a6-7890-abcd-ef0123456789",
  "generator_id": "a1b2c3d4-e5f6-7890-abcd-ef1234567890",
  "registered_at": "2025-01-02T10:05:00Z"
}
```

Re-registering an already-registered agent returns `409` with code `already_registered` (the call is not idempotent). When an agent self-registers, it omits `agent_id` and authenticates with its own credentials.

### Step 2: List Registered Agents

View the agents registered with a generator:

```bash
curl "http://broker:3000/api/v1/generators/$GENERATOR_ID/registered-agents" \
  -H "Authorization: Bearer $ADMIN_PAK"
```

### Step 3: Deregister an Agent

Remove an agent's registration. This is destructive: it cascades by removing the agent's explicit targets for this generator's stacks and pushes a target-changed notification to the agent, which prunes those Kubernetes resources on its next reconcile.

```bash
curl -X DELETE "http://broker:3000/api/v1/generators/$GENERATOR_ID/register" \
  -H "Authorization: Bearer $ADMIN_PAK" \
  -H "Content-Type: application/json" \
  -d '{
    "agent_id": "f1e2d3c4-b5a6-7890-abcd-ef0123456789"
  }'
```

### CLI Equivalents

The `brokkr` CLI wraps these endpoints (an admin PAK is required for cross-entity operations):

```bash
# Register an agent with a generator
brokkr register --agent <agent-id> --generator <generator-id>

# Deregister (destructive — cascades target removal)
brokkr deregister --agent <agent-id> --generator <generator-id>

# List registrations by agent or by generator
brokkr registrations --generator <generator-id>
```

## PAK Rotation

Rotate the generator's PAK for security best practices or if the current PAK is compromised:

```bash
curl -X POST "http://broker:3000/api/v1/generators/$GENERATOR_ID/rotate-pak" \
  -H "Authorization: Bearer $GENERATOR_PAK"
```

The response contains the new PAK:

```json
{
  "generator": {
    "id": "a1b2c3d4-...",
    "name": "github-actions-prod",
    ...
  },
  "pak": "brokkr_BRnew34cd_GeneratorLongTokenExample02"
}
```

After rotation:

1. The old PAK is immediately invalidated — the API endpoint replaces the stored hash and evicts the old PAK from the broker's auth cache, so it stops working right away. (This immediate invalidation applies only to API rotation; the CLI `rotate generator` command behaves differently — see [Managing PAKs](./pak-management.md).)
2. Update all CI/CD systems with the new PAK
3. Verify deployments work with the new credentials

Consider rotating PAKs:

- On a regular schedule (quarterly, annually)
- When team members with access leave
- If the PAK may have been exposed
- After security incidents

## Access Control

Generators operate under a scoped permission model: a generator PAK can manage the generator itself and the resources it created, while administrative operations remain admin-only. See the [permission model](../reference/generators.md#permission-model) and [resource scoping](../reference/generators.md#resource-scoping) sections of the Generators API Reference for the full matrix.

## Best Practices

### One Generator Per Pipeline or Team

Create a separate generator for each deployment pipeline or team. Because the generator is the tenant, this is what gives you clear resource ownership, independent PAK rotation, per-team separation, scoped operator views, and easier auditing. Sharing one generator across two teams merges them into one tenant.

### Naming Conventions

Use descriptive names that identify purpose and scope, e.g. `github-actions-prod`, `gitlab-ci-staging`, or `team-platform-prod`.

## Troubleshooting

### Authentication Failures

If API calls fail with 401 or 403:

1. Verify the PAK is correct — PAKs do not expire, so a previously working key has been rotated or revoked
2. Check whether the PAK was rotated elsewhere, or the generator deleted (deleting a generator revokes its PAK)
3. Ensure you're using the generator PAK, not the admin PAK (for generator-scoped operations)

### Cannot See Resources

If a generator cannot see expected stacks or templates:

1. Verify the resources were created with this generator's PAK
2. Resources created by other generators are not visible; another generator's template returns `403 template_not_accessible`
3. Use admin PAK to view all resources across generators

### Agent Not Receiving Stacks

If an agent applies nothing despite matching labels, check its registrations first — a label match is only honored for generators the agent has registered with:

```bash
curl -s "http://broker:3000/api/v1/agents/$AGENT_ID/registrations" \
  -H "Authorization: Bearer $ADMIN_PAK"
```

An agent with no registrations, or one missing this generator, receives nothing from label or annotation matching. Register it (above) or set `broker.generatorIds` on the agent deployment.

### PAK Lost

If you've lost a generator's PAK:

1. Use the admin PAK to rotate: `POST /api/v1/generators/{id}/rotate-pak`
2. Store the new PAK securely
3. Update all systems using the old PAK

## Related Documentation

- [Generators API Reference](../reference/generators.md) - Complete API documentation
- [Multi-Tenancy](../reference/multi-tenancy.md) - The tenant model, tenant listing, and `pak_id` scoped views
- [Multi-Tenant Setup](./multi-tenant-setup.md) - Onboarding multiple teams onto one broker
- [Agent Registration](./agent-registration.md) - Registering agents with generators (tenant consent)
- [Stack Templates](./templates.md) - Using templates with generators
- [Authentication](../explanation/security-model.md) - Understanding Brokkr authentication
