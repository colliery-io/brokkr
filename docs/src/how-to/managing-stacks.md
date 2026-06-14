# Managing Stacks

Stacks are the fundamental organizational unit in Brokkr, representing collections of related Kubernetes resources that belong together. This guide covers creating stacks, configuring them with labels and annotations for targeting, managing their lifecycle, and understanding how they connect to agents and deployment objects.

## Understanding Stacks

A stack is a container for deployment objects—the versioned snapshots of Kubernetes resources you want to deploy—and the unit that agents target. See [Core Concepts](../explanation/core-concepts.md) for how stacks fit into Brokkr's architecture.

## Creating Stacks

### Basic Stack Creation

Every stack belongs to a generator, so `generator_id` is required when creating one. Generators use their own ID (the broker rejects anything else). Admins use the UUID of the auto-created `admin-generator`, which you can look up first:

```bash
GENERATOR_ID=$(curl -s http://localhost:3000/api/v1/generators \
  -H "Authorization: Bearer $ADMIN_PAK" \
  | jq -r '.[] | select(.name == "admin-generator") | .id')
```

Then create the stack with a name, optional description, and the generator ID:

```bash
curl -X POST http://localhost:3000/api/v1/stacks \
  -H "Authorization: Bearer $ADMIN_PAK" \
  -H "Content-Type: application/json" \
  -d "{
    \"name\": \"payment-service\",
    \"description\": \"Payment processing microservice and dependencies\",
    \"generator_id\": \"$GENERATOR_ID\"
  }"
```

The response includes the stack's UUID which you'll use for all subsequent operations:

```json
{
  "id": "550e8400-e29b-41d4-a716-446655440000",
  "name": "payment-service",
  "description": "Payment processing microservice and dependencies",
  "created_at": "2025-01-02T10:00:00Z",
  "updated_at": "2025-01-02T10:00:00Z",
  "generator_id": "a1b2c3d4-e5f6-7890-abcd-ef1234567890"
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
  -d "{
    \"stack_id\": \"$STACK_ID\",
    \"key\": \"cost-center\",
    \"value\": \"engineering-team-a\"
  }"
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

Targeting establishes the relationship between stacks and the agents that should manage them. Without either an explicit target or a shared label/annotation, an agent won't receive deployment objects from a stack regardless of other configuration.

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

Beyond direct assignment, stacks and agents are associated dynamically through their metadata: a stack is associated with an agent if it shares **any** label string with the agent, or **any** annotation key-value pair (OR semantics). The broker evaluates this match at query time whenever the agent polls—no targeting rows are created and no configuration step is required. Deployment objects from every matched stack flow into the agent's target state automatically.

For example, to have every production agent receive a stack:

```bash
# Label the stack
curl -X POST http://localhost:3000/api/v1/stacks/$STACK_ID/labels \
  -H "Authorization: Bearer $ADMIN_PAK" \
  -H "Content-Type: application/json" \
  -d '"production"'

# Add the same label to each agent that should manage it
curl -X POST http://localhost:3000/api/v1/agents/$AGENT_ID/labels \
  -H "Authorization: Bearer $ADMIN_PAK" \
  -H "Content-Type: application/json" \
  -d "{
    \"agent_id\": \"$AGENT_ID\",
    \"label\": \"production\"
  }"
```

From the next poll onward, the agent's target state includes the stack's deployment objects. Removing the shared label (or annotation) dissolves the association just as automatically.

### Multi-Cluster Deployments

A single stack can be targeted by multiple agents, enabling multi-cluster deployment scenarios. Each agent independently polls for the stack's deployment objects and applies them to its cluster. This is useful for:

- High availability across regions
- Disaster recovery setups
- Consistent infrastructure across environments

## Working with Deployment Objects

Once a stack exists and is targeted to agents, you populate it with deployment objects containing Kubernetes resources.

### Creating Deployment Objects

The deployment object's body is a single multi-document YAML stream (the complete desired state for the stack). The endpoint accepts it two ways.

**Raw YAML (recommended).** Send the manifest as the request body with `Content-Type: application/yaml` — no JSON wrapping or newline escaping:

```bash
curl -X POST "http://localhost:3000/api/v1/stacks/$STACK_ID/deployment-objects" \
  -H "Authorization: Bearer $ADMIN_PAK" \
  -H "Content-Type: application/yaml" \
  --data-binary @resources.yaml
```

Because the body is just a YAML stream, anything that emits one pipes straight in:

```bash
kustomize build ./overlay | curl -X POST ".../stacks/$STACK_ID/deployment-objects" \
  -H "Authorization: Bearer $ADMIN_PAK" -H "Content-Type: application/yaml" --data-binary @-
helm template app ./chart -f values.yaml | curl ... --data-binary @-
```

To submit a deletion marker this way, add `?deletion_marker=true` (the body may be empty):

```bash
curl -X POST ".../stacks/$STACK_ID/deployment-objects?deletion_marker=true" \
  -H "Authorization: Bearer $ADMIN_PAK" -H "Content-Type: application/yaml" --data-binary ''
```

**JSON envelope.** The original form still works, with the YAML embedded as a string field:

```bash
curl -X POST "http://localhost:3000/api/v1/stacks/$STACK_ID/deployment-objects" \
  -H "Authorization: Bearer $ADMIN_PAK" \
  -H "Content-Type: application/json" \
  -d "$(jq -n --arg yaml "$(cat resources.yaml)" '{yaml_content: $yaml, is_deletion_marker: false}')"
```

Either way the broker validates the YAML parses on ingest (malformed input is rejected with `400 invalid_deployment_object`) and computes the checksum. Each deployment object receives a sequence ID that guarantees ordering; the latest is the stack's desired state.

You can also pull a deployment object back out as raw YAML with `Accept: application/yaml`:

```bash
curl ".../deployment-objects/$OBJECT_ID" -H "Authorization: Bearer $ADMIN_PAK" -H "Accept: application/yaml"
```

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

The update endpoint takes the complete stack object (`PUT` replaces, it does not patch), so fetch the current record, modify the fields you want, and send the whole object back:

```bash
curl -s http://localhost:3000/api/v1/stacks/$STACK_ID \
  -H "Authorization: Bearer $ADMIN_PAK" \
  | jq '.name = "payment-service-v2" | .description = "Updated payment service with new features"' \
  | curl -X PUT http://localhost:3000/api/v1/stacks/$STACK_ID \
      -H "Authorization: Bearer $ADMIN_PAK" \
      -H "Content-Type: application/json" \
      -d @-
```

### Deleting Stacks

Brokkr uses soft deletion for stacks. When you delete a stack, it's marked with a `deleted_at` timestamp rather than being removed from the database:

```bash
curl -X DELETE http://localhost:3000/api/v1/stacks/$STACK_ID \
  -H "Authorization: Bearer $ADMIN_PAK"
```

Soft deletion does three things, atomically via a database trigger:
- The stack stops appearing in list queries, while the underlying data remains intact for audit purposes
- All of the stack's deployment objects are soft-deleted along with it
- An empty deletion marker deployment object is inserted automatically

The deletion marker is how cluster cleanup happens: agents targeting the stack receive it on their next poll and delete the stack's resources rather than apply them. No manual step is needed—deleting the stack is sufficient.

### Understanding Deletion Markers

You can also create a deletion marker yourself by submitting a deployment object with `is_deletion_marker: true`. This removes the stack's resources from clusters while keeping the stack itself alive—useful for clearing out a stack you intend to repopulate. For stack deletion, the marker is created for you by the trigger described above.

## Generator Integration

Generators are external systems like CI/CD pipelines that create stacks programmatically. When a generator creates a stack, that stack's `generator_id` links it to the creating generator, establishing ownership and access control.

Generators can only access stacks they created. This scoping ensures pipelines can't accidentally modify resources belonging to other systems. See the [Generators Guide](./generators.md) for details on integrating CI/CD systems.

## Warnings

**Deletion removes cluster resources**: Deleting a stack automatically creates a deletion marker, and every targeted agent will delete the stack's resources from its cluster. Confirm which agents are associated with the stack before deleting.

**Shared labels create associations**: Because any shared label or annotation associates a stack with an agent, an overly generic label (e.g., `default`) can unintentionally deploy a stack to agents you didn't have in mind. Choose label values deliberately.

## Related Documentation

- [Deploy Your First Application](../tutorials/first-deployment.md) - First deployment walkthrough
- [Core Concepts](../explanation/core-concepts.md) - Understanding Brokkr's architecture
- [Generators](./generators.md) - CI/CD integration
- [Templates](./templates.md) - Standardized deployments
