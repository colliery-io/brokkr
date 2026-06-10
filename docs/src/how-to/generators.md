# Working with Generators

Generators are identity principals that enable external systems to interact with Brokkr. They provide a way for CI/CD pipelines, automation tools, and other services to authenticate and manage resources within defined boundaries. This guide covers creating generators, integrating them with CI/CD systems, and managing their lifecycle.

## Understanding Generators

A generator represents an external system that creates and manages Brokkr resources. Each generator receives a Prefixed API Key (PAK) that grants it permission to create stacks, templates, and deployment objects. Resources created by a generator are scoped to that generator, providing natural isolation between different automation pipelines or teams.

Generators differ from the admin PAK in important ways. The admin PAK has full access to all resources and administrative functions. Generator PAKs can only access resources they created and cannot perform administrative operations like creating other generators or managing agents.

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

### One Generator Per Pipeline

Create separate generators for each deployment pipeline or team. This provides:

- Clear ownership of resources
- Independent PAK rotation
- Easier auditing and troubleshooting
- Isolation between environments

### Naming Conventions

Use descriptive names that identify the purpose and scope:

- `github-actions-prod` - Production pipeline in GitHub Actions
- `gitlab-ci-staging` - Staging pipeline in GitLab CI
- `jenkins-nightly-builds` - Nightly build automation
- `team-platform-prod` - Platform team's production deployments

## Troubleshooting

### Authentication Failures

If API calls fail with 401 or 403:

1. Verify the PAK is correct and not expired
2. Check if the PAK was rotated
3. Ensure you're using the generator PAK, not the admin PAK (for generator-scoped operations)

### Cannot See Resources

If a generator cannot see expected stacks or templates:

1. Verify the resources were created with this generator's PAK
2. Resources created by other generators are not visible
3. Use admin PAK to view all resources across generators

### PAK Lost

If you've lost a generator's PAK:

1. Use the admin PAK to rotate: `POST /api/v1/generators/{id}/rotate-pak`
2. Store the new PAK securely
3. Update all systems using the old PAK

## Related Documentation

- [Generators API Reference](../reference/generators.md) - Complete API documentation
- [Stack Templates](./templates.md) - Using templates with generators
- [Authentication](../explanation/security-model.md) - Understanding Brokkr authentication
