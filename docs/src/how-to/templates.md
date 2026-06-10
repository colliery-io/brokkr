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

Template content uses `{{ variable }}` substitution, `{% if %}` conditionals, `{% for %}` loops, and filters like `{{ replicas | default(value=1) }}` for optional parameters. See [Tera Template Syntax](../reference/templates.md#tera-template-syntax) in the reference and the [Tera documentation](https://tera.netlify.app/docs/#filters) for the full feature set.

## JSON Schema Validation

The `parameters_schema` is a standard JSON Schema declaring each parameter's type, which parameters are `required`, and constraints like `minimum`/`maximum`, `pattern`, and `enum`. See [JSON Schema for Parameters](../reference/templates.md#json-schema-for-parameters) in the reference for examples of each.

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

When instantiation fails due to label mismatch, you'll receive a 422 response with the missing keys under `details`:

```json
{
  "code": "template_stack_mismatch",
  "message": "template labels do not match stack",
  "details": {
    "missing_labels": ["tier=1"],
    "missing_annotations": []
  }
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

Errors follow the standard API error shape: `{"code": ..., "message": ..., "details": ...}`.

### Invalid Tera Syntax

Template creation fails with syntax errors:

```json
{
  "code": "invalid_template_syntax",
  "message": "..."
}
```

Check for:
- Unclosed `{{ }}` or `{% %}` blocks
- Missing `{% endif %}` or `{% endfor %}`
- Invalid filter names

### Invalid JSON Schema

```json
{
  "code": "invalid_parameters_schema",
  "message": "..."
}
```

Validate your schema at [jsonschemavalidator.net](https://www.jsonschemavalidator.net/).

### Parameter Validation Failed

```json
{
  "code": "invalid_parameters",
  "message": "invalid parameters",
  "details": {
    "validation_errors": [
      "/replicas: 0 is less than the minimum of 1"
    ]
  }
}
```

Check that parameters match the schema constraints.

### Template Rendering Failed

```json
{
  "code": "template_render_failed",
  "message": "Variable `name` not found"
}
```

Ensure all required template variables are provided in parameters, or use `| default(value=...)` for optional ones.
