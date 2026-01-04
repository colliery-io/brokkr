---
title: "Managing Stacks"
weight: 4
---

# Managing Stacks

Stacks are the fundamental organizational unit in Brokkr, representing collections of related Kubernetes resources that belong together. This guide covers creating stacks, configuring them with labels and annotations for targeting, managing their lifecycle, and understanding how they connect to agents and deployment objects.

## Understanding Stacks

A stack serves as a container for deployment objects—the versioned snapshots of Kubernetes resources you want to deploy. When you create a stack, you establish a logical boundary for a set of resources, whether that's an application, a service, or any collection of related Kubernetes manifests. Agents are then assigned to stacks through targeting relationships, enabling you to control which clusters receive which resources.

Each stack maintains a history of deployment objects, providing an immutable audit trail of every configuration change. This history enables rollback capabilities and satisfies compliance requirements for tracking what was deployed and when.

## Creating Stacks

### Basic Stack Creation

To create a stack, send a POST request to the stacks endpoint with a name and optional description:

```bash
curl -X POST http://localhost:3000/api/v1/stacks \
  -H "Authorization: Bearer $ADMIN_PAK" \
  -H "Content-Type: application/json" \
  -d '{
    "name": "payment-service",
    "description": "Payment processing microservice and dependencies"
  }'
```

The response includes the stack's UUID which you'll use for all subsequent operations:

```json
{
  "id": "550e8400-e29b-41d4-a716-446655440000",
  "name": "payment-service",
  "description": "Payment processing microservice and dependencies",
  "created_at": "2025-01-02T10:00:00Z",
  "updated_at": "2025-01-02T10:00:00Z",
  "generator_id": "00000000-0000-0000-0000-000000000000"
}
```

### Stack Naming Conventions

Stack names must be non-empty strings up to 255 characters. While Brokkr doesn't enforce naming conventions, consistent patterns make your infrastructure easier to navigate. Consider including:

- The application or service name
- The environment (if not using labels)
- A version or variant indicator for parallel deployments

Examples: `frontend-app`, `database-cluster`, `monitoring-stack`, `api-gateway-v2`

## Configuring Labels and Annotations

Labels and annotations provide metadata for stacks that enables dynamic targeting and integration with external systems. Labels are simple string values used for categorization and selection. Annotations are key-value pairs for richer metadata.

### Adding Labels

Labels enable pattern-based targeting where agents with matching labels automatically receive stacks. Add a label with a POST request:

```bash
curl -X POST http://localhost:3000/api/v1/stacks/$STACK_ID/labels \
  -H "Authorization: Bearer $ADMIN_PAK" \
  -H "Content-Type: application/json" \
  -d '"production"'
```

Labels must be non-empty strings up to 64 characters with no whitespace. Common labeling patterns include:

| Purpose | Example Labels |
|---------|---------------|
| Environment | `production`, `staging`, `development` |
| Region | `us-east`, `eu-west`, `apac` |
| Tier | `frontend`, `backend`, `data` |
| Criticality | `critical`, `standard` |

### Listing Labels

View all labels for a stack:

```bash
curl http://localhost:3000/api/v1/stacks/$STACK_ID/labels \
  -H "Authorization: Bearer $ADMIN_PAK"
```

### Removing Labels

Remove a label by its value:

```bash
curl -X DELETE http://localhost:3000/api/v1/stacks/$STACK_ID/labels/production \
  -H "Authorization: Bearer $ADMIN_PAK"
```

### Adding Annotations

Annotations carry key-value metadata for integration with external systems or configuration hints:

```bash
curl -X POST http://localhost:3000/api/v1/stacks/$STACK_ID/annotations \
  -H "Authorization: Bearer $ADMIN_PAK" \
  -H "Content-Type: application/json" \
  -d '{
    "stack_id": "550e8400-e29b-41d4-a716-446655440000",
    "key": "cost-center",
    "value": "engineering-team-a"
  }'
```

Both keys and values must be non-empty strings up to 64 characters with no whitespace. Annotations are useful for:

| Purpose | Example |
|---------|---------|
| Cost allocation | `cost-center=team-alpha` |
| Owner tracking | `owner=platform-team` |
| SLA classification | `sla-tier=gold` |
| External references | `jira-project=PLAT` |

### Listing Annotations

View all annotations for a stack:

```bash
curl http://localhost:3000/api/v1/stacks/$STACK_ID/annotations \
  -H "Authorization: Bearer $ADMIN_PAK"
```

### Removing Annotations

Remove an annotation by its key:

```bash
curl -X DELETE http://localhost:3000/api/v1/stacks/$STACK_ID/annotations/cost-center \
  -H "Authorization: Bearer $ADMIN_PAK"
```

## Targeting Stacks to Agents

Targeting establishes the relationship between stacks and the agents that should manage them. Without targeting, an agent won't receive deployment objects from a stack regardless of other configuration.

### Direct Assignment

Create a targeting relationship by associating an agent with a specific stack:

```bash
curl -X POST http://localhost:3000/api/v1/agents/$AGENT_ID/targets \
  -H "Authorization: Bearer $ADMIN_PAK" \
  -H "Content-Type: application/json" \
  -d "{
    \"agent_id\": \"$AGENT_ID\",
    \"stack_id\": \"$STACK_ID\"
  }"
```

Direct assignment is appropriate when you have explicit control over which agents manage which stacks and the relationship is stable.

### Label-Based Targeting

When both agents and stacks carry matching labels, you can configure agents to automatically target all stacks with those labels. This enables patterns like "all production agents receive all production stacks" without maintaining explicit per-pair associations.

To set up label-based targeting:

1. Add labels to your stacks that represent their characteristics
2. Add corresponding labels to agents that should manage those stacks
3. Configure the targeting policy (see agent configuration)

The agent polls for stacks matching its label configuration and creates targeting relationships automatically.

### Multi-Cluster Deployments

A single stack can be targeted by multiple agents, enabling multi-cluster deployment scenarios. Each agent independently polls for the stack's deployment objects and applies them to its cluster. This is useful for:

- High availability across regions
- Disaster recovery setups
- Consistent infrastructure across environments

## Working with Deployment Objects

Once a stack exists and is targeted to agents, you populate it with deployment objects containing Kubernetes resources.

### Creating Deployment Objects

Submit Kubernetes YAML as a deployment object:

```bash
curl -X POST "http://localhost:3000/api/v1/stacks/$STACK_ID/deployment-objects" \
  -H "Authorization: Bearer $ADMIN_PAK" \
  -H "Content-Type: application/json" \
  -d "$(jq -n --arg yaml "$(cat resources.yaml)" '{yaml_content: $yaml, is_deletion_marker: false}')"
```

Each deployment object receives a sequence ID that guarantees ordering. Agents process deployment objects in sequence order, ensuring resources are applied in the intended order.

### Listing Deployment Objects

View all deployment objects in a stack:

```bash
curl "http://localhost:3000/api/v1/stacks/$STACK_ID/deployment-objects" \
  -H "Authorization: Bearer $ADMIN_PAK"
```

### Using Templates

For standardized deployments, instantiate a template into a deployment object:

```bash
curl -X POST "http://localhost:3000/api/v1/stacks/$STACK_ID/deployment-objects/from-template" \
  -H "Authorization: Bearer $ADMIN_PAK" \
  -H "Content-Type: application/json" \
  -d '{
    "template_id": "template-uuid-here",
    "parameters": {
      "replicas": 3,
      "image_tag": "v1.2.3"
    }
  }'
```

The template's parameters are validated against its JSON Schema before rendering. If the template has labels, they must match the stack's labels for instantiation to succeed.

## Stack Lifecycle

### Updating Stacks

Modify a stack's name or description:

```bash
curl -X PUT http://localhost:3000/api/v1/stacks/$STACK_ID \
  -H "Authorization: Bearer $ADMIN_PAK" \
  -H "Content-Type: application/json" \
  -d '{
    "id": "550e8400-e29b-41d4-a716-446655440000",
    "name": "payment-service-v2",
    "description": "Updated payment service with new features"
  }'
```

### Deleting Stacks

Brokkr uses soft deletion for stacks. When you delete a stack, it's marked with a `deleted_at` timestamp rather than being removed from the database:

```bash
curl -X DELETE http://localhost:3000/api/v1/stacks/$STACK_ID \
  -H "Authorization: Bearer $ADMIN_PAK"
```

Soft deletion triggers several cascading actions:
- All deployment objects in the stack are soft-deleted
- A deletion marker deployment object is created so agents know to remove resources
- The stack stops appearing in list queries

The underlying data remains intact for audit purposes and potential recovery.

### Understanding Deletion Markers

When a stack is soft-deleted, Brokkr creates a special deployment object with `is_deletion_marker: true`. Agents receiving this marker understand they should delete the resources rather than apply them. This ensures resources are cleaned up from clusters even after the stack is deleted.

## Generator Integration

Generators are external systems like CI/CD pipelines that create stacks programmatically. When a generator creates a stack, that stack's `generator_id` links it to the creating generator, establishing ownership and access control.

Generators can only access stacks they created. This scoping ensures pipelines can't accidentally modify resources belonging to other systems. See the [Generators Guide](/how-to/generators) for details on integrating CI/CD systems.

## Best Practices

**Organize by responsibility**: Group resources into stacks based on what changes together. A stack should contain resources that are deployed, updated, and scaled as a unit.

**Use labels for targeting**: Rather than creating explicit targeting relationships for each stack-agent pair, use labels to establish patterns. This reduces configuration overhead as your infrastructure grows.

**Keep stacks focused**: While you can put any resources in a stack, keeping stacks focused on specific applications or services makes management clearer and reduces blast radius for changes.

**Document with annotations**: Use annotations to record metadata that helps teams understand the stack's purpose, ownership, and relationship to business systems.

**Plan for deletion**: Remember that deleting a stack triggers resource deletion on targeted clusters. Ensure you understand which clusters are affected before deleting.

## Related Documentation

- [Quick Start Guide](/getting-started/quick-start) - First deployment walkthrough
- [Core Concepts](/explanation/core-concepts) - Understanding Brokkr's architecture
- [Generators](/how-to/generators) - CI/CD integration
- [Templates](/how-to/templates) - Standardized deployments
