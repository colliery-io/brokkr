# Templates Reference

Stack templates provide reusable, parameterized Kubernetes manifests with JSON Schema validation. This reference covers the data model, API endpoints, Tera template syntax, and matching rules.

## Data Model

### StackTemplate

| Field | Type | Description |
|-------|------|-------------|
| `id` | UUID | Unique identifier |
| `created_at` | DateTime | Creation timestamp |
| `updated_at` | DateTime | Last update timestamp |
| `deleted_at` | DateTime? | Soft deletion timestamp |
| `generator_id` | UUID? | Owning generator (NULL = system template, admin-only) |
| `name` | String | Template name (1-255 characters) |
| `description` | String? | Optional description |
| `version` | Integer | Version number (starts at 1, auto-increments) |
| `template_content` | String | Tera template (Kubernetes YAML with placeholders) |
| `parameters_schema` | String | JSON Schema defining valid parameters |
| `checksum` | String | SHA-256 hash of `template_content` |

**Constraints:**
- Unique combination of `(generator_id, name, version)`
- `version` must be >= 1
- `name`, `template_content`, and `parameters_schema` cannot be empty
- `checksum` is auto-computed on creation

### Template Types

| Type | `generator_id` | Created By | Visible To |
|------|-----------------|------------|------------|
| System template | NULL | Admin | Admin + all generators |
| Generator template | UUID | Generator | Admin + owning generator |

### RenderedDeploymentObject

When a template is instantiated, Brokkr records the provenance:

| Field | Type | Description |
|-------|------|-------------|
| `id` | UUID | Unique identifier |
| `deployment_object_id` | UUID | Resulting deployment object |
| `template_id` | UUID | Source template |
| `template_version` | Integer | Version used |
| `template_parameters` | String (JSON) | Parameters provided |
| `created_at` | DateTime | Instantiation timestamp |

---

## API Endpoints

### List Templates

```
GET /api/v1/templates
```

**Auth:** Admin sees all templates. Generator sees system templates + own templates.

**Response:** `200 OK` — `StackTemplate[]`

---

### Create Template

```
POST /api/v1/templates
```

**Auth:** Admin only (creates system templates). Generators can also create templates (owned by the generator).

**Request body:**

```json
{
  "name": "web-service",
  "description": "Standard web service template",
  "template_content": "apiVersion: apps/v1\nkind: Deployment\nmetadata:\n  name: {{ service_name }}\nspec:\n  replicas: {{ replicas }}",
  "parameters_schema": "{\"type\": \"object\", \"required\": [\"service_name\"], \"properties\": {\"service_name\": {\"type\": \"string\"}, \"replicas\": {\"type\": \"integer\", \"default\": 2}}}"
}
```

**Validation:**
- Template content is validated for Tera syntax errors
- Parameters schema is validated as a valid JSON Schema
- Name must be 1-255 characters

**Response:** `201 Created` — `StackTemplate`

---

### Get Template

```
GET /api/v1/templates/{id}
```

**Auth:** Admin or owning generator.

**Response:** `200 OK` — `StackTemplate`

---

### Update Template (New Version)

```
PUT /api/v1/templates/{id}
```

**Auth:** Admin or owning generator.

Updating a template creates a **new version**. The previous version remains available. The version number auto-increments.

**Request body:**

```json
{
  "description": "Standard web service template v2",
  "template_content": "...",
  "parameters_schema": "..."
}
```

> **Note:** The `name` field is not accepted on update — it is preserved from the existing template.

**Response:** `200 OK` — `StackTemplate` (with incremented `version`)

---

### Delete Template

```
DELETE /api/v1/templates/{id}
```

**Auth:** Admin or owning generator.

**Response:** `204 No Content`

---

### Template Labels

```
GET    /api/v1/templates/{id}/labels
POST   /api/v1/templates/{id}/labels          Body: "label-string"
DELETE /api/v1/templates/{id}/labels/{label}
```

**Auth:** Admin or owning generator.

Labels control which stacks a template can be instantiated into. See [Matching Rules](#matching-rules).

---

### Template Annotations

```
GET    /api/v1/templates/{id}/annotations
POST   /api/v1/templates/{id}/annotations     Body: {"key": "k", "value": "v"}
DELETE /api/v1/templates/{id}/annotations/{key}
```

**Auth:** Admin or owning generator.

---

### Instantiate Template

```
POST /api/v1/stacks/{stack_id}/deployment-objects/from-template
```

**Auth:** Admin or owning generator (for the stack).

**Request body:**

```json
{
  "template_id": "uuid-of-template",
  "parameters": {
    "service_name": "frontend",
    "replicas": 3
  }
}
```

**Process:**
1. Fetches the latest version of the template
2. Validates parameters against the JSON Schema
3. Checks template-to-stack matching rules (labels/annotations)
4. Renders the Tera template with the provided parameters
5. Creates a deployment object with the rendered YAML
6. Records the rendered deployment object provenance

**Response:** `200 OK` — `DeploymentObject[]`

---

## Tera Template Syntax

Templates use the [Tera](https://keats.github.io/tera/) engine. Key features:

### Variable Substitution

```yaml
name: {{ service_name }}
replicas: {{ replicas }}
image: {{ repository }}:{{ tag }}
```

### Conditionals

```yaml
{% if enable_hpa %}
---
apiVersion: autoscaling/v2
kind: HorizontalPodAutoscaler
metadata:
  name: {{ service_name }}
spec:
  scaleTargetRef:
    apiVersion: apps/v1
    kind: Deployment
    name: {{ service_name }}
  minReplicas: {{ min_replicas }}
  maxReplicas: {{ max_replicas }}
{% endif %}
```

### Loops

```yaml
env:
{% for key, value in env_vars %}
- name: {{ key }}
  value: "{{ value }}"
{% endfor %}
```

### Filters

| Filter | Usage | Result |
|--------|-------|--------|
| `default` | `{{ x \| default(value="y") }}` | Use "y" if x is undefined |
| `upper` | `{{ x \| upper }}` | Uppercase |
| `lower` | `{{ x \| lower }}` | Lowercase |
| `trim` | `{{ x \| trim }}` | Strip whitespace |
| `replace` | `{{ x \| replace(from="a", to="b") }}` | String replacement |
| `json_encode` | `{{ x \| json_encode }}` | JSON-encode value |

See the [Tera documentation](https://keats.github.io/tera/docs/) for the complete filter and function reference.

---

## JSON Schema for Parameters

The `parameters_schema` field accepts a standard [JSON Schema](https://json-schema.org/understanding-json-schema/) document. Commonly used features:

### Type Constraints

```json
{
  "type": "object",
  "properties": {
    "replicas": { "type": "integer", "minimum": 1, "maximum": 100 },
    "name": { "type": "string", "minLength": 1, "maxLength": 63 },
    "debug": { "type": "boolean" },
    "cpu": { "type": "string", "pattern": "^[0-9]+m$" }
  }
}
```

### Required Fields

```json
{
  "type": "object",
  "required": ["name", "image"],
  "properties": {
    "name": { "type": "string" },
    "image": { "type": "string" }
  }
}
```

### Defaults

```json
{
  "properties": {
    "replicas": { "type": "integer", "default": 2 },
    "port": { "type": "integer", "default": 8080 }
  }
}
```

### Enum Values

```json
{
  "properties": {
    "environment": {
      "type": "string",
      "enum": ["development", "staging", "production"]
    }
  }
}
```

---

## Matching Rules

Templates with labels or annotations are restricted to stacks with matching metadata. This prevents production-only templates from being instantiated into staging stacks.

**Rules:**

1. Template with **no labels and no annotations** → matches **any** stack (universal)
2. Template with **labels** → stack must have **all** of the template's labels
3. Template with **annotations** → stack must have **all** of the template's annotations (key-value match)
4. Template with **both** → stack must satisfy **both** label AND annotation requirements

**Example:**

Template with labels `["env:production", "tier:frontend"]`:
- Stack with `["env:production", "tier:frontend", "region:us"]` → **matches** (has all required)
- Stack with `["env:production"]` → **no match** (missing `tier:frontend`)
- Stack with `["env:staging", "tier:frontend"]` → **no match** (wrong env)

---

## Versioning Behavior

- Creating a template starts at version 1
- Updating via `PUT` auto-increments the version
- Instantiation always uses the **latest version** of the template
- Old versions remain in the database for provenance
- Deployment objects rendered from old versions are not affected by template updates
- The `rendered_deployment_objects` table records which version was used

---

## Related Documentation

- [Using Stack Templates](../how-to/templates.md) — how-to guide for template workflows
- [Tutorial: Standardized Deployments](../tutorials/templates.md) — step-by-step tutorial
- [API Reference](./api/README.md) — complete API documentation
- [Data Model](../explanation/data-model.md) — entity relationships
