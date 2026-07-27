# Using Stack Templates

Stack templates allow you to define reusable Kubernetes manifests with parameterized values. Templates use [Tera](https://tera.netlify.app/) for templating and [JSON Schema](https://json-schema.org/) for parameter validation.

## Concepts

### What Templates Provide

- **Reusability**: Define common patterns once, instantiate many times
- **Validation**: Parameters are validated against JSON Schema before rendering
- **Safety**: Template syntax is validated at creation time
- **Versioning**: Updates create a new version with a **new template ID**; the old ID keeps rendering the old content
- **Access Control**: System templates (admin, usable by everyone) vs generator-owned templates (usable only by their owner)

### Template Matching

Templates can be constrained to specific stacks using labels and annotations:

- **No labels/annotations**: Template can be used with any stack
- **With labels**: ALL template labels must exist on the target stack
- **With annotations**: ALL template annotation key-value pairs must exist on the target stack

Labels and annotations are attached to one specific template version. Keep this in mind when you update a template — see [Template Versioning](#template-versioning).

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
1. Check that you own the target stack, and that the template is yours or a system template
2. Validate template labels match the stack
3. Validate parameters against the JSON Schema
4. Render the template with Tera
5. Create a deployment object in the stack

`template_id` identifies one specific version, and that is the version that gets rendered. There is no "use the latest version" option — see [Template Versioning](#template-versioning).

## Template Labels and Annotations

### Restricting Template Usage

A template with no labels or annotations can be instantiated into any stack. Add labels or annotations to restrict it — the target stack must then carry the same ones:

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

Templates are immutable. An update does not change the existing template — it creates a new version, stored as a new row with **its own template ID**. The old ID stays valid and keeps rendering the old content forever, so you must capture the new ID from the response and use it from then on.

### Update a Template and Pick Up the New ID

```bash
# Update template (creates version 2 with a NEW id)
UPDATED=$(curl -s -X PUT http://localhost:3000/api/v1/templates/$TEMPLATE_ID \
  -H "Authorization: Bearer $ADMIN_PAK" \
  -H "Content-Type: application/json" \
  -d '{
    "description": "Updated nginx template with HPA support",
    "template_content": "...",
    "parameters_schema": "..."
  }')

# Keep the old id if you still need it, then switch to the new one
OLD_TEMPLATE_ID=$TEMPLATE_ID
TEMPLATE_ID=$(echo "$UPDATED" | jq -r '.id')
echo "$UPDATED" | jq '{id, version}'   # → new id, version 2
```

Anything still sending `$OLD_TEMPLATE_ID` — a pipeline, a saved script, a CI variable — keeps deploying version 1 silently, with no error. Roll the new ID out to every caller that should be on the new version.

### Re-apply Labels and Annotations

**Labels and annotations do not carry over to the new version.** They belong to the previous row, so version 2 starts with none and will match *any* stack until you re-add them:

```bash
# Re-apply every label the previous version had, to the NEW template id
curl -X POST http://localhost:3000/api/v1/templates/$TEMPLATE_ID/labels \
  -H "Authorization: Bearer $ADMIN_PAK" \
  -H "Content-Type: application/json" \
  -d '"env=production"'

# ...and every annotation
curl -X POST http://localhost:3000/api/v1/templates/$TEMPLATE_ID/annotations \
  -H "Authorization: Bearer $ADMIN_PAK" \
  -H "Content-Type: application/json" \
  -d '{"key": "tier", "value": "1"}'
```

To be sure you re-apply the full set, list what the previous version carried (do this before the update if you would rather not chase it afterwards):

```bash
curl -s http://localhost:3000/api/v1/templates/$OLD_TEMPLATE_ID/labels \
  -H "Authorization: Bearer $ADMIN_PAK" | jq -r '.[].label'
curl -s http://localhost:3000/api/v1/templates/$OLD_TEMPLATE_ID/annotations \
  -H "Authorization: Bearer $ADMIN_PAK" | jq -r '.[] | "\(.key)=\(.value)"'
```

Old versions remain available, and deployment objects record the specific template version they were rendered from. To find the current version of a template, list templates and take the highest `version` for that name.

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
- Instantiate system templates and their own templates, and only into stacks they own

Supplying another generator's template ID is rejected with `403 template_not_accessible`, even when you own the target stack. Knowing an ID grants nothing.

To share a template across generators, create it as a **system template** — an admin PAK creating a template produces one (`generator_id` is null), and every generator can then instantiate it. That is the supported sharing mechanism; there is no way to grant one generator access to another's template.

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

### 2. Restrict the Template with a Label

```bash
curl -X POST http://localhost:3000/api/v1/templates/$TEMPLATE_ID/labels \
  -H "Authorization: Bearer $ADMIN_PAK" \
  -H "Content-Type: application/json" \
  -d '"database=postgresql"'
```

### 3. Give the Target Stack the Same Label

The template now only matches stacks that carry `database=postgresql`, so add it to the stack you intend to deploy into — otherwise instantiation fails with `template_stack_mismatch`:

```bash
curl -X POST http://localhost:3000/api/v1/stacks/$PROD_STACK_ID/labels \
  -H "Authorization: Bearer $ADMIN_PAK" \
  -H "Content-Type: application/json" \
  -d '"database=postgresql"'
```

### 4. Instantiate for Production

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

### Template Not Accessible

```json
{
  "code": "template_not_accessible",
  "message": "not authorized to access this template"
}
```

The template belongs to another generator. Use one of your own templates, or ask an admin to publish it as a system template.

### Updated the Template, but Deployments Are Unchanged

You are still instantiating the old version. `PUT` created a new row with a new ID; the ID your pipeline holds still points at the previous version. List the template's versions, take the highest one, and update the ID your callers use:

```bash
curl -s http://localhost:3000/api/v1/templates \
  -H "Authorization: Bearer $ADMIN_PAK" \
  | jq '[.[] | select(.name=="nginx-deployment")] | sort_by(.version) | .[-1] | {id, version}'
```

### A Restricted Template Suddenly Matches Every Stack

Its labels and annotations were left behind on the previous version. Re-apply them to the new template ID — see [Re-apply Labels and Annotations](#re-apply-labels-and-annotations).
