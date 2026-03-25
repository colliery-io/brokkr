# Tutorial: Standardized Deployments with Templates

In this tutorial, you'll create a reusable deployment template with parameterized values and JSON Schema validation, then instantiate it across multiple stacks. Templates eliminate YAML duplication and enforce consistency.

**What you'll learn:**

- How to create a template with Tera syntax
- How to define a parameter schema using JSON Schema
- How to instantiate a template into a stack
- How template versioning works
- How template targeting restricts which stacks can use a template

**Prerequisites:**

- A running Brokkr development environment (`angreal local up`)
- Your admin PAK
- Completed the [Deploy Your First Application](./first-deployment.md) tutorial

## Step 1: Understand the Template Concept

A Brokkr template has two parts:

1. **Template content** — Kubernetes YAML with [Tera](https://keats.github.io/tera/) placeholders (e.g., `{{ replicas }}`, `{{ image_tag }}`)
2. **Parameters schema** — a [JSON Schema](https://json-schema.org/) that defines which parameters exist, their types, defaults, and constraints

When you **instantiate** a template into a stack, you provide parameter values. Brokkr validates them against the schema, renders the Tera template, and creates a deployment object with the resulting YAML.

## Step 2: Create a Template

Create a template for a standard web service deployment:

```bash
curl -s -X POST http://localhost:3000/api/v1/templates \
  -H "Authorization: <your-admin-pak>" \
  -H "Content-Type: application/json" \
  -d '{
    "name": "web-service",
    "description": "Standard web service with configurable replicas, image, and resource limits",
    "template_content": "apiVersion: v1\nkind: Namespace\nmetadata:\n  name: {{ namespace }}\n---\napiVersion: apps/v1\nkind: Deployment\nmetadata:\n  name: {{ service_name }}\n  namespace: {{ namespace }}\n  labels:\n    app: {{ service_name }}\n    managed-by: brokkr\nspec:\n  replicas: {{ replicas }}\n  selector:\n    matchLabels:\n      app: {{ service_name }}\n  template:\n    metadata:\n      labels:\n        app: {{ service_name }}\n    spec:\n      containers:\n      - name: {{ service_name }}\n        image: {{ image_repository }}:{{ image_tag }}\n        ports:\n        - containerPort: {{ container_port }}\n        resources:\n          requests:\n            cpu: {{ cpu_request }}\n            memory: {{ memory_request }}\n          limits:\n            cpu: {{ cpu_limit }}\n            memory: {{ memory_limit }}\n---\napiVersion: v1\nkind: Service\nmetadata:\n  name: {{ service_name }}\n  namespace: {{ namespace }}\nspec:\n  selector:\n    app: {{ service_name }}\n  ports:\n  - port: 80\n    targetPort: {{ container_port }}",
    "parameters_schema": "{\"type\": \"object\", \"required\": [\"service_name\", \"namespace\", \"image_repository\", \"image_tag\"], \"properties\": {\"service_name\": {\"type\": \"string\", \"description\": \"Name of the service\", \"minLength\": 1, \"maxLength\": 63}, \"namespace\": {\"type\": \"string\", \"description\": \"Kubernetes namespace\", \"minLength\": 1}, \"image_repository\": {\"type\": \"string\", \"description\": \"Container image repository\"}, \"image_tag\": {\"type\": \"string\", \"description\": \"Container image tag\", \"default\": \"latest\"}, \"replicas\": {\"type\": \"integer\", \"description\": \"Number of replicas\", \"default\": 2, \"minimum\": 1, \"maximum\": 20}, \"container_port\": {\"type\": \"integer\", \"description\": \"Container port\", \"default\": 8080}, \"cpu_request\": {\"type\": \"string\", \"description\": \"CPU request\", \"default\": \"100m\"}, \"memory_request\": {\"type\": \"string\", \"description\": \"Memory request\", \"default\": \"128Mi\"}, \"cpu_limit\": {\"type\": \"string\", \"description\": \"CPU limit\", \"default\": \"500m\"}, \"memory_limit\": {\"type\": \"string\", \"description\": \"Memory limit\", \"default\": \"256Mi\"}}}"
  }' | jq '{id, name, version, checksum}'
```

The response shows the template was created with `version: 1`:

```json
{
  "id": "t1234567-...",
  "name": "web-service",
  "version": 1,
  "checksum": "abc123..."
}
```

Save the template ID:

```bash
TEMPLATE_ID="t1234567-..."  # use the actual ID from the response
```

## Step 3: Understand the Parameters Schema

The JSON Schema you provided defines 10 parameters: four required (`service_name`, `namespace`, `image_repository`, `image_tag`) and six optional with defaults (`replicas` defaults to 2, `container_port` to 8080, etc.). The schema enforces constraints — for example, `replicas` must be between 1 and 20, and `service_name` must be 1-63 characters.

The schema ensures that callers provide the required values and that constraints are enforced at instantiation time, before any YAML is rendered. See the [Templates Reference](../reference/templates.md#json-schema-for-parameters) for the full JSON Schema syntax guide.

## Step 4: Create a Stack and Instantiate the Template

Create a stack, then instantiate the template into it:

```bash
# Create the stack
STACK_ID=$(curl -s -X POST http://localhost:3000/api/v1/stacks \
  -H "Authorization: <your-admin-pak>" \
  -H "Content-Type: application/json" \
  -d '{"name": "frontend-app", "description": "Frontend web application", "generator_id": "00000000-0000-0000-0000-000000000000"}' \
  | jq -r '.id')

# Instantiate the template
curl -s -X POST "http://localhost:3000/api/v1/stacks/${STACK_ID}/deployment-objects/from-template" \
  -H "Authorization: <your-admin-pak>" \
  -H "Content-Type: application/json" \
  -d "{
    \"template_id\": \"${TEMPLATE_ID}\",
    \"parameters\": {
      \"service_name\": \"frontend\",
      \"namespace\": \"frontend-app\",
      \"image_repository\": \"myregistry.example.com/frontend\",
      \"image_tag\": \"v2.1.0\",
      \"replicas\": 3,
      \"container_port\": 3000,
      \"memory_limit\": \"512Mi\"
    }
  }" | jq '.[0] | {id, sequence_id, yaml_checksum}'
```

Brokkr validated the parameters, rendered the Tera template, and created a deployment object. The resulting YAML has all placeholders replaced with actual values.

## Step 5: Verify the Rendered Output

Fetch the deployment object to see the rendered YAML:

```bash
DO_ID=$(curl -s "http://localhost:3000/api/v1/stacks/${STACK_ID}/deployment-objects" \
  -H "Authorization: <your-admin-pak>" | jq -r '.[0].id')

curl -s "http://localhost:3000/api/v1/deployment-objects/${DO_ID}" \
  -H "Authorization: <your-admin-pak>" | jq -r '.yaml_content'
```

You'll see fully-rendered Kubernetes YAML with all template variables replaced:

```yaml
apiVersion: v1
kind: Namespace
metadata:
  name: frontend-app
---
apiVersion: apps/v1
kind: Deployment
metadata:
  name: frontend
  namespace: frontend-app
  labels:
    app: frontend
    managed-by: brokkr
spec:
  replicas: 3
  selector:
    matchLabels:
      app: frontend
  template:
    metadata:
      labels:
        app: frontend
    spec:
      containers:
      - name: frontend
        image: myregistry.example.com/frontend:v2.1.0
        ports:
        - containerPort: 3000
        resources:
          requests:
            cpu: 100m
            memory: 128Mi
          limits:
            cpu: 500m
            memory: 512Mi
---
apiVersion: v1
kind: Service
metadata:
  name: frontend
  namespace: frontend-app
spec:
  selector:
    app: frontend
  ports:
  - port: 80
    targetPort: 3000
```

## Step 6: Re-use the Template for Another Service

The same template works for a different service by changing the parameters:

```bash
# Create a backend stack
BACKEND_STACK=$(curl -s -X POST http://localhost:3000/api/v1/stacks \
  -H "Authorization: <your-admin-pak>" \
  -H "Content-Type: application/json" \
  -d '{"name": "backend-api", "description": "Backend API service", "generator_id": "00000000-0000-0000-0000-000000000000"}' \
  | jq -r '.id')

# Instantiate with different parameters
curl -s -X POST "http://localhost:3000/api/v1/stacks/${BACKEND_STACK}/deployment-objects/from-template" \
  -H "Authorization: <your-admin-pak>" \
  -H "Content-Type: application/json" \
  -d "{
    \"template_id\": \"${TEMPLATE_ID}\",
    \"parameters\": {
      \"service_name\": \"api\",
      \"namespace\": \"backend-api\",
      \"image_repository\": \"myregistry.example.com/api\",
      \"image_tag\": \"v3.0.1\",
      \"replicas\": 5,
      \"container_port\": 8080,
      \"cpu_limit\": \"1000m\",
      \"memory_limit\": \"1Gi\"
    }
  }" | jq '.[0] | {id, sequence_id}'
```

One template, multiple services, each with appropriate configuration.

## Step 7: Update the Template (Versioning)

Templates are versioned. Updating a template creates a new version while preserving the old one:

```bash
curl -s -X PUT "http://localhost:3000/api/v1/templates/${TEMPLATE_ID}" \
  -H "Authorization: <your-admin-pak>" \
  -H "Content-Type: application/json" \
  -d '{
    "description": "Standard web service v2 - adds liveness probe",
    "template_content": "apiVersion: v1\nkind: Namespace\nmetadata:\n  name: {{ namespace }}\n---\napiVersion: apps/v1\nkind: Deployment\nmetadata:\n  name: {{ service_name }}\n  namespace: {{ namespace }}\n  labels:\n    app: {{ service_name }}\n    managed-by: brokkr\nspec:\n  replicas: {{ replicas }}\n  selector:\n    matchLabels:\n      app: {{ service_name }}\n  template:\n    metadata:\n      labels:\n        app: {{ service_name }}\n    spec:\n      containers:\n      - name: {{ service_name }}\n        image: {{ image_repository }}:{{ image_tag }}\n        ports:\n        - containerPort: {{ container_port }}\n        livenessProbe:\n          httpGet:\n            path: /healthz\n            port: {{ container_port }}\n          initialDelaySeconds: 10\n          periodSeconds: 30\n        resources:\n          requests:\n            cpu: {{ cpu_request }}\n            memory: {{ memory_request }}\n          limits:\n            cpu: {{ cpu_limit }}\n            memory: {{ memory_limit }}\n---\napiVersion: v1\nkind: Service\nmetadata:\n  name: {{ service_name }}\n  namespace: {{ namespace }}\nspec:\n  selector:\n    app: {{ service_name }}\n  ports:\n  - port: 80\n    targetPort: {{ container_port }}",
    "parameters_schema": "{\"type\": \"object\", \"required\": [\"service_name\", \"namespace\", \"image_repository\", \"image_tag\"], \"properties\": {\"service_name\": {\"type\": \"string\", \"minLength\": 1, \"maxLength\": 63}, \"namespace\": {\"type\": \"string\", \"minLength\": 1}, \"image_repository\": {\"type\": \"string\"}, \"image_tag\": {\"type\": \"string\", \"default\": \"latest\"}, \"replicas\": {\"type\": \"integer\", \"default\": 2, \"minimum\": 1, \"maximum\": 20}, \"container_port\": {\"type\": \"integer\", \"default\": 8080}, \"cpu_request\": {\"type\": \"string\", \"default\": \"100m\"}, \"memory_request\": {\"type\": \"string\", \"default\": \"128Mi\"}, \"cpu_limit\": {\"type\": \"string\", \"default\": \"500m\"}, \"memory_limit\": {\"type\": \"string\", \"default\": \"256Mi\"}}}"
  }' | jq '{id, name, version}'
```

The response shows `version: 2`. Existing deployment objects rendered from version 1 are unaffected. New instantiations will use the latest version.

## Step 8: Schema Validation in Action

Try instantiating with invalid parameters to see validation:

```bash
# Missing required field (service_name)
curl -s -X POST "http://localhost:3000/api/v1/stacks/${STACK_ID}/deployment-objects/from-template" \
  -H "Authorization: <your-admin-pak>" \
  -H "Content-Type: application/json" \
  -d "{
    \"template_id\": \"${TEMPLATE_ID}\",
    \"parameters\": {
      \"namespace\": \"test\",
      \"image_repository\": \"nginx\",
      \"image_tag\": \"latest\"
    }
  }" | jq .

# Replicas out of range (max is 20)
curl -s -X POST "http://localhost:3000/api/v1/stacks/${STACK_ID}/deployment-objects/from-template" \
  -H "Authorization: <your-admin-pak>" \
  -H "Content-Type: application/json" \
  -d "{
    \"template_id\": \"${TEMPLATE_ID}\",
    \"parameters\": {
      \"service_name\": \"test\",
      \"namespace\": \"test\",
      \"image_repository\": \"nginx\",
      \"image_tag\": \"latest\",
      \"replicas\": 100
    }
  }" | jq .
```

Both requests return validation errors, preventing invalid YAML from reaching your clusters.

## Clean Up

```bash
curl -s -X DELETE "http://localhost:3000/api/v1/stacks/${STACK_ID}" \
  -H "Authorization: <your-admin-pak>"
curl -s -X DELETE "http://localhost:3000/api/v1/stacks/${BACKEND_STACK}" \
  -H "Authorization: <your-admin-pak>"
curl -s -X DELETE "http://localhost:3000/api/v1/templates/${TEMPLATE_ID}" \
  -H "Authorization: <your-admin-pak>"
```

## What You've Learned

- **Templates** combine Tera-syntax YAML with JSON Schema parameter validation
- **Instantiation** validates parameters, renders the template, and creates a deployment object
- **Versioning** preserves old template versions while allowing updates
- **JSON Schema** enforces types, required fields, ranges, and string constraints
- Templates reduce duplication — one template serves many stacks with different parameters

For the complete Tera template syntax (conditionals, loops, filters) and JSON Schema reference, see the [Templates Reference](../reference/templates.md).

## Next Steps

- [Using Stack Templates](../how-to/templates.md) — detailed how-to guide for template workflows
- [Templates Reference](../reference/templates.md) — complete API reference for templates
- [Core Concepts](../explanation/core-concepts.md) — how templates fit into the Brokkr architecture
