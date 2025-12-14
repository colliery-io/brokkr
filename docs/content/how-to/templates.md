---
title: "Using Stack Templates"
description: "How to create and use reusable stack templates with Tera and JSON Schema"
weight: 10
---

# Using Stack Templates

Stack templates allow you to define reusable Kubernetes manifests with parameterized values. Templates use [Tera](https://tera.netlify.app/) for templating and [JSON Schema](https://json-schema.org/) for parameter validation.

## Concepts

### What Templates Provide

- **Reusability**: Define common patterns once, instantiate many times
- **Validation**: Parameters are validated against JSON Schema before rendering
- **Safety**: Template syntax is validated at creation time
- **Versioning**: Updates create new versions, preserving history
- **Access Control**: System templates (admin) vs generator-owned templates

### Template Matching

Templates can be constrained to specific stacks using labels and annotations:

- **No labels/annotations**: Template can be used with any stack
- **With labels**: ALL template labels must exist on the target stack
- **With annotations**: ALL template annotation key-value pairs must exist on the target stack

## Creating a Template

### Basic Template Structure

A template consists of:
1. **Name**: Identifier for the template
2. **Template Content**: Tera-templated YAML
3. **Parameters Schema**: JSON Schema defining valid parameters

### Example: Nginx Deployment Template

```bash
curl -X POST http://localhost:3000/api/v1/templates \
  -H "Authorization: Bearer $ADMIN_PAK" \
  -H "Content-Type: application/json" \
  -d '{
    "name": "nginx-deployment",
    "description": "Simple nginx deployment with configurable replicas and image",
    "template_content": "apiVersion: apps/v1\nkind: Deployment\nmetadata:\n  name: {{ name }}\n  namespace: {{ namespace | default(value=\"default\") }}\nspec:\n  replicas: {{ replicas | default(value=1) }}\n  selector:\n    matchLabels:\n      app: {{ name }}\n  template:\n    metadata:\n      labels:\n        app: {{ name }}\n    spec:\n      containers:\n      - name: nginx\n        image: nginx:{{ version | default(value=\"latest\") }}\n        ports:\n        - containerPort: 80",
    "parameters_schema": "{\"type\": \"object\", \"required\": [\"name\"], \"properties\": {\"name\": {\"type\": \"string\", \"minLength\": 1, \"description\": \"Deployment name\"}, \"namespace\": {\"type\": \"string\", \"description\": \"Target namespace\"}, \"replicas\": {\"type\": \"integer\", \"minimum\": 1, \"maximum\": 10, \"description\": \"Number of replicas\"}, \"version\": {\"type\": \"string\", \"description\": \"Nginx image tag\"}}}"
  }'
```

## Tera Templating

### Variable Substitution

Use `{{ variable }}` syntax for simple substitution:

```yaml
metadata:
  name: {{ name }}
  namespace: {{ namespace }}
```

### Default Values

Use the `default` filter for optional parameters:

```yaml
spec:
  replicas: {{ replicas | default(value=1) }}
```

### Conditionals

Use `{% if %}` blocks for conditional content:

```yaml
{% if enable_hpa %}
---
apiVersion: autoscaling/v2
kind: HorizontalPodAutoscaler
metadata:
  name: {{ name }}-hpa
spec:
  scaleTargetRef:
    apiVersion: apps/v1
    kind: Deployment
    name: {{ name }}
  minReplicas: {{ min_replicas | default(value=1) }}
  maxReplicas: {{ max_replicas | default(value=10) }}
{% endif %}
```

### Loops

Use `{% for %}` to iterate over arrays:

```yaml
spec:
  containers:
  {% for container in containers %}
  - name: {{ container.name }}
    image: {{ container.image }}
    ports:
    {% for port in container.ports %}
    - containerPort: {{ port }}
    {% endfor %}
  {% endfor %}
```

### Filters

Tera provides many built-in filters:

```yaml
metadata:
  name: {{ name | lower }}           # lowercase
  labels:
    version: "{{ version | upper }}" # uppercase
    hash: {{ content | sha256 }}     # hash value
```

See the [Tera documentation](https://tera.netlify.app/docs/#filters) for all available filters.

## JSON Schema Validation

### Basic Schema

Define required and optional parameters:

```json
{
  "type": "object",
  "required": ["name", "image"],
  "properties": {
    "name": {
      "type": "string",
      "minLength": 1,
      "description": "Resource name"
    },
    "image": {
      "type": "string",
      "pattern": "^[a-z0-9./-]+:[a-zA-Z0-9.-]+$"
    },
    "replicas": {
      "type": "integer",
      "minimum": 1,
      "maximum": 100,
      "default": 1
    }
  }
}
```

### Validation Constraints

Common JSON Schema constraints:

| Constraint | Type | Description |
|------------|------|-------------|
| `minLength`, `maxLength` | string | String length limits |
| `pattern` | string | Regex pattern |
| `minimum`, `maximum` | number | Numeric bounds |
| `enum` | any | Allowed values |
| `minItems`, `maxItems` | array | Array length limits |

### Nested Objects

```json
{
  "type": "object",
  "properties": {
    "resources": {
      "type": "object",
      "properties": {
        "cpu": {"type": "string", "pattern": "^[0-9]+m?$"},
        "memory": {"type": "string", "pattern": "^[0-9]+[GMK]i$"}
      }
    }
  }
}
```

## Instantiating Templates

Once a template is created, instantiate it to create deployment objects:

```bash
curl -X POST http://localhost:3000/api/v1/stacks/$STACK_ID/deployment-objects/from-template \
  -H "Authorization: Bearer $ADMIN_PAK" \
  -H "Content-Type: application/json" \
  -d '{
    "template_id": "'"$TEMPLATE_ID"'",
    "parameters": {
      "name": "my-nginx",
      "namespace": "production",
      "replicas": 3,
      "version": "1.25"
    }
  }'
```

The broker will:
1. Validate template labels match the stack
2. Validate parameters against the JSON Schema
3. Render the template with Tera
4. Create a deployment object in the stack

## Template Labels and Annotations

### Restricting Template Usage

Add labels to restrict which stacks can use a template:

```bash
# Add label to template
curl -X POST http://localhost:3000/api/v1/templates/$TEMPLATE_ID/labels \
  -H "Authorization: Bearer $ADMIN_PAK" \
  -H "Content-Type: application/json" \
  -d '"env=production"'

# Add annotation to template
curl -X POST http://localhost:3000/api/v1/templates/$TEMPLATE_ID/annotations \
  -H "Authorization: Bearer $ADMIN_PAK" \
  -H "Content-Type: application/json" \
  -d '{"key": "tier", "value": "1"}'
```

### Matching Rules

| Template | Stack | Result |
|----------|-------|--------|
| No labels | Any labels | Matches |
| `env=prod` | `env=prod, team=platform` | Matches |
| `env=prod` | `env=staging` | No match |
| `env=prod, tier=1` | `env=prod` | No match (missing tier) |

When instantiation fails due to label mismatch, you'll receive a 422 response with details:

```json
{
  "error": "Template labels do not match stack",
  "missing_labels": ["tier=1"],
  "missing_annotations": []
}
```

## Template Versioning

Templates are immutable. Updates create new versions:

```bash
# Update template (creates version 2)
curl -X PUT http://localhost:3000/api/v1/templates/$TEMPLATE_ID \
  -H "Authorization: Bearer $ADMIN_PAK" \
  -H "Content-Type: application/json" \
  -d '{
    "description": "Updated nginx template with HPA support",
    "template_content": "...",
    "parameters_schema": "..."
  }'
```

Each version has a unique ID. Deployment objects reference the specific template version used.

## Generator-Owned Templates

Generators can create and manage their own templates:

```bash
# Generator creates template
curl -X POST http://localhost:3000/api/v1/templates \
  -H "Authorization: Bearer $GENERATOR_PAK" \
  -H "Content-Type: application/json" \
  -d '{
    "name": "my-app-template",
    "template_content": "...",
    "parameters_schema": "..."
  }'
```

Generators can only:
- View system templates (no generator_id) and their own templates
- Modify/delete only their own templates
- Instantiate templates into stacks they own

## Complete Example: PostgreSQL Database

### 1. Create the Template

```bash
curl -X POST http://localhost:3000/api/v1/templates \
  -H "Authorization: Bearer $ADMIN_PAK" \
  -H "Content-Type: application/json" \
  -d '{
    "name": "postgresql-database",
    "description": "PostgreSQL StatefulSet with PVC",
    "template_content": "apiVersion: v1\nkind: Secret\nmetadata:\n  name: {{ name }}-credentials\n  namespace: {{ namespace }}\nstringData:\n  POSTGRES_USER: {{ username | default(value=\"postgres\") }}\n  POSTGRES_PASSWORD: {{ password }}\n  POSTGRES_DB: {{ database }}\n---\napiVersion: v1\nkind: Service\nmetadata:\n  name: {{ name }}\n  namespace: {{ namespace }}\nspec:\n  ports:\n  - port: 5432\n  clusterIP: None\n  selector:\n    app: {{ name }}\n---\napiVersion: apps/v1\nkind: StatefulSet\nmetadata:\n  name: {{ name }}\n  namespace: {{ namespace }}\nspec:\n  serviceName: {{ name }}\n  replicas: {{ replicas | default(value=1) }}\n  selector:\n    matchLabels:\n      app: {{ name }}\n  template:\n    metadata:\n      labels:\n        app: {{ name }}\n    spec:\n      containers:\n      - name: postgres\n        image: postgres:{{ version | default(value=\"15\") }}\n        ports:\n        - containerPort: 5432\n        envFrom:\n        - secretRef:\n            name: {{ name }}-credentials\n        volumeMounts:\n        - name: data\n          mountPath: /var/lib/postgresql/data\n  volumeClaimTemplates:\n  - metadata:\n      name: data\n    spec:\n      accessModes: [\"ReadWriteOnce\"]\n      resources:\n        requests:\n          storage: {{ storage_size }}",
    "parameters_schema": "{\"type\": \"object\", \"required\": [\"name\", \"namespace\", \"database\", \"password\", \"storage_size\"], \"properties\": {\"name\": {\"type\": \"string\", \"minLength\": 1}, \"namespace\": {\"type\": \"string\", \"minLength\": 1}, \"database\": {\"type\": \"string\", \"minLength\": 1}, \"username\": {\"type\": \"string\"}, \"password\": {\"type\": \"string\", \"minLength\": 8}, \"version\": {\"type\": \"string\"}, \"replicas\": {\"type\": \"integer\", \"minimum\": 1}, \"storage_size\": {\"type\": \"string\", \"pattern\": \"^[0-9]+[GMK]i$\"}}}"
  }'
```

### 2. Add Production Label

```bash
curl -X POST http://localhost:3000/api/v1/templates/$TEMPLATE_ID/labels \
  -H "Authorization: Bearer $ADMIN_PAK" \
  -H "Content-Type: application/json" \
  -d '"database=postgresql"'
```

### 3. Instantiate for Production

```bash
curl -X POST http://localhost:3000/api/v1/stacks/$PROD_STACK_ID/deployment-objects/from-template \
  -H "Authorization: Bearer $ADMIN_PAK" \
  -H "Content-Type: application/json" \
  -d '{
    "template_id": "'"$TEMPLATE_ID"'",
    "parameters": {
      "name": "orders-db",
      "namespace": "production",
      "database": "orders",
      "password": "secure-password-here",
      "version": "15",
      "replicas": 3,
      "storage_size": "100Gi"
    }
  }'
```

## Troubleshooting

### Invalid Tera Syntax

Template creation fails with syntax errors:

```json
{
  "error": "Invalid Tera syntax: ..."
}
```

Check for:
- Unclosed `{{ }}` or `{% %}` blocks
- Missing `{% endif %}` or `{% endfor %}`
- Invalid filter names

### Invalid JSON Schema

```json
{
  "error": "Invalid JSON Schema: ..."
}
```

Validate your schema at [jsonschemavalidator.net](https://www.jsonschemavalidator.net/).

### Parameter Validation Failed

```json
{
  "error": "Invalid parameters",
  "validation_errors": [
    "/replicas: 0 is less than the minimum of 1"
  ]
}
```

Check that parameters match the schema constraints.

### Template Rendering Failed

```json
{
  "error": "Template rendering failed: Variable `name` not found"
}
```

Ensure all required template variables are provided in parameters, or use `| default(value=...)` for optional ones.
