---
title: "Working with Generators"
weight: 5
---

# Working with Generators

Generators are identity principals that enable external systems to interact with Brokkr. They provide a way for CI/CD pipelines, automation tools, and other services to authenticate and manage resources within defined boundaries. This guide covers creating generators, integrating them with CI/CD systems, and managing their lifecycle.

## Understanding Generators

A generator represents an external system that creates and manages Brokkr resources. Each generator receives a Pre-Authentication Key (PAK) that grants it permission to create stacks, templates, and deployment objects. Resources created by a generator are scoped to that generator, providing natural isolation between different automation pipelines or teams.

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
  "pak": "brk_gen_abc123...xyz789"
}
```

### Step 2: Store the PAK Securely

The PAK is only returned once at creation time. Store it immediately in your secret management system:

- **GitHub Actions**: Add as a repository or organization secret
- **GitLab CI**: Add as a protected variable
- **Jenkins**: Store in credentials manager
- **Vault/AWS Secrets Manager**: Store with appropriate access policies

If you lose the PAK, you'll need to rotate it (see PAK Rotation below).

## CI/CD Integration

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
          BROKKR_URL: ${{ vars.BROKKR_URL }}
        run: |
          curl -X POST "$BROKKR_URL/api/v1/stacks" \
            -H "Authorization: Bearer $BROKKR_PAK" \
            -H "Content-Type: application/json" \
            -d '{
              "name": "my-app-${{ github.sha }}",
              "description": "Deployed from commit ${{ github.sha }}"
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
          \"description\": \"Pipeline $CI_PIPELINE_ID\"
        }"
  only:
    - main
```

### Using Templates

Generators can create and use stack templates for consistent deployments:

```bash
# Create a template (using generator PAK)
curl -X POST "http://broker:3000/api/v1/templates" \
  -H "Authorization: Bearer $GENERATOR_PAK" \
  -H "Content-Type: application/json" \
  -d '{
    "name": "web-service",
    "description": "Standard web service deployment",
    "template_yaml": "...",
    "schema_json": "..."
  }'

# Create stack from template
curl -X POST "http://broker:3000/api/v1/stacks" \
  -H "Authorization: Bearer $GENERATOR_PAK" \
  -H "Content-Type: application/json" \
  -d '{
    "name": "my-web-service",
    "template_name": "web-service",
    "parameters": {
      "replicas": 3,
      "image": "myapp:v1.2.3"
    }
  }'
```

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

Update the generator's metadata:

```bash
curl -X PUT "http://broker:3000/api/v1/generators/$GENERATOR_ID" \
  -H "Authorization: Bearer $GENERATOR_PAK" \
  -H "Content-Type: application/json" \
  -d '{
    "description": "Updated description"
  }'
```

### Delete Generator

Soft-delete a generator (admin or generator itself):

```bash
curl -X DELETE "http://broker:3000/api/v1/generators/$GENERATOR_ID" \
  -H "Authorization: Bearer $ADMIN_PAK"
```

Deleting a generator does not delete its associated stacks and resources. Those remain in the system and can be managed by an admin.

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
  "pak": "brk_gen_new123...newxyz"
}
```

After rotation:

1. The old PAK is immediately invalidated
2. Update all CI/CD systems with the new PAK
3. Verify deployments work with the new credentials

Consider rotating PAKs:

- On a regular schedule (quarterly, annually)
- When team members with access leave
- If the PAK may have been exposed
- After security incidents

## Access Control

Generators operate under a scoped permission model:

| Operation | Admin PAK | Generator PAK |
|-----------|-----------|---------------|
| Create generators | Yes | No |
| List all generators | Yes | No |
| View own generator | Yes | Yes |
| Update own generator | Yes | Yes |
| Delete own generator | Yes | Yes |
| Rotate own PAK | Yes | Yes |
| Create stacks | Yes | Yes |
| View own stacks | Yes | Yes |
| View other generators' stacks | Yes | No |
| Manage agents | Yes | No |
| Manage webhooks | Yes | No |

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

### Secret Management

Never store PAKs in:

- Source code repositories
- Unencrypted configuration files
- Logs or console output

Always use:

- CI/CD secret management (GitHub Secrets, GitLab Variables)
- Secret management systems (Vault, AWS Secrets Manager)
- Encrypted environment variables

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

- [Generators API Reference](/reference/generators) - Complete API documentation
- [Stack Templates](/how-to/templates) - Using templates with generators
- [Authentication](/explanation/security-model) - Understanding Brokkr authentication
