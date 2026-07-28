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
| `generator_id` | UUID? | Owning generator (NULL = system template, created and modified by admins only) |
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

| Type | `generator_id` | Created By | Readable + instantiable by | Modifiable by |
|------|-----------------|------------|----------------------------|---------------|
| System template | NULL | Admin | Admin + all generators | Admin |
| Generator template | UUID | Generator | Admin + owning generator | Admin + owning generator |

A generator that references a template it cannot read — another generator's template — receives `403 template_not_accessible` from every template endpoint, including instantiation. System templates are the only cross-generator sharing mechanism; there is no per-generator grant.

### Labels and Annotations

Template labels and annotations are attached to a single template row — that is, to one version — and are used only for stack matching. See [Matching Rules](#matching-rules).

| Field | Type | Description |
|-------|------|-------------|
| `template_id` | UUID | The template version the label/annotation belongs to |
| `label` | String | Label text: 1-64 characters, no whitespace (labels only) |
| `key` / `value` | String | Annotation key and value: 1-64 characters each, no whitespace (annotations only) |

Labels and annotations are opaque strings matched by exact equality — Brokkr does not parse `key=value` label syntax.

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

Every version is a separate entry in the response, each with its own `id`. Group by `name` (and `generator_id`) and take the highest `version` to find the current one.

**Response:** `200 OK` — `StackTemplate[]`

---

### Create Template

```
POST /api/v1/templates
```

**Auth:** Admin (creates system templates) or generator (owns the template).

Version numbers are scoped to `(generator_id, name)`. A new name starts at version 1; re-using an existing name that you own creates the next version of that template rather than returning a conflict.

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

**Auth:** Admin, the owning generator, or any generator for a system template. Otherwise `403 template_not_accessible`.

Returns the exact version identified by `{id}`, not the latest version of that name.

**Response:** `200 OK` — `StackTemplate`

---

### Update Template (New Version)

```
PUT /api/v1/templates/{id}
```

**Auth:** Admin or owning generator (generators cannot update system templates). Otherwise `403 template_not_owned`.

Updating a template creates a **new version**: a new row, with a **new `id`** and the next version number. The row identified by `{id}` is not modified and remains available.

Two consequences:

- The `id` in the response is the one to use from now on. Requests that keep using the old `id` keep resolving to the old version.
- **Labels and annotations are not copied to the new version.** They belong to the previous row, so the new version starts with none and matches every stack until they are re-applied. Re-add them with the label and annotation endpoints, using the new `id`.

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

**Auth:** Admin or owning generator (generators cannot delete system templates). Otherwise `403 template_not_owned`.

Soft-deletes the single version identified by `{id}`. Other versions of the same template name are unaffected and must be deleted individually.

**Response:** `204 No Content`

---

### Template Labels

```
GET    /api/v1/templates/{id}/labels
POST   /api/v1/templates/{id}/labels          Body: "label-string"
DELETE /api/v1/templates/{id}/labels/{label}
```

**Auth:** `GET` — admin, owning generator, or any generator for a system template. `POST`/`DELETE` — admin or owning generator.

Labels control which stacks a template can be instantiated into. See [Matching Rules](#matching-rules). Labels apply to the version identified by `{id}` only; a version created by `PUT` has no labels until they are added to it.

---

### Template Annotations

```
GET    /api/v1/templates/{id}/annotations
POST   /api/v1/templates/{id}/annotations     Body: {"key": "k", "value": "v"}
DELETE /api/v1/templates/{id}/annotations/{key}
```

**Auth:** `GET` — admin, owning generator, or any generator for a system template. `POST`/`DELETE` — admin or owning generator.

Like labels, annotations apply to the version identified by `{id}` only.

---

### Instantiate Template

```
POST /api/v1/stacks/{stack_id}/deployment-objects/from-template
```

**Auth:** Two independent checks, in this order:

1. The target stack must be owned by the caller, or the caller must be an admin — otherwise `403 stack_not_owned`
2. The template must be readable by the caller (own template, or a system template) — otherwise `403 template_not_accessible`

A generator cannot instantiate another generator's template, even into a stack it owns.

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
1. Fetches the target stack and enforces stack ownership
2. Fetches the exact template row identified by `template_id` and enforces template read access
3. Checks template-to-stack matching rules (labels/annotations)
4. Validates parameters against the JSON Schema
5. Renders the Tera template with the provided parameters
6. Creates a deployment object with the rendered YAML
7. Records the rendered deployment object provenance

`template_id` identifies one version. Instantiation renders that version's content — it never resolves to the latest version of the template's name.

**Response:** `201 Created` — `DeploymentObject`

**Errors:**

| Status | Code | Cause |
|--------|------|-------|
| 403 | `stack_not_owned` | Target stack belongs to another generator |
| 403 | `template_not_accessible` | Template belongs to another generator |
| 404 | `stack_not_found` / `template_not_found` | Unknown or deleted id |
| 422 | `template_stack_mismatch` | Stack lacks required labels/annotations; `details` lists `missing_labels` and `missing_annotations` |
| 400 | `invalid_parameters` | Parameters failed JSON Schema validation; `details.validation_errors` lists the failures |
| 400 | `template_render_failed` | Tera rendering failed (for example, a referenced variable was not supplied) |

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

Labels and annotation values are compared as exact strings; extra labels and annotations on the stack are ignored.

**Example:**

Template with labels `["env=production", "tier=frontend"]`:
- Stack with `["env=production", "tier=frontend", "region=us"]` → **matches** (has all required)
- Stack with `["env=production"]` → **no match** (missing `tier=frontend`)
- Stack with `["env=staging", "tier=frontend"]` → **no match** (wrong env)

Matching is a targeting guardrail, not an access control boundary — a template with no labels is instantiable into any stack the caller is authorized for. Access is governed by [template ownership](#template-types).

---

## Versioning Behavior

- Version numbers are scoped to `(generator_id, name)`. A new name starts at version 1
- Updating via `PUT` inserts a new row at version+1 with a **new template ID**; the old ID continues to identify the old version
- `POST` with a name that already exists for the same owner behaves the same way — it creates the next version, not a duplicate
- **Labels and annotations are not carried over to the new version.** They belong to the previous row; the new version starts with none and therefore matches every stack until they are re-applied
- Instantiation uses the exact template version identified by `template_id` — to render a newer version, reference the new version's ID. There is no "latest" alias
- To find the newest version of a name, list templates and take the highest `version` for that `(generator_id, name)` pair
- Old versions remain in the database for provenance, and `DELETE` removes one version at a time
- Deployment objects rendered from old versions are not affected by template updates
- The `rendered_deployment_objects` table records which version was used

---

## Related Documentation

- [Using Stack Templates](../how-to/templates.md) — how-to guide for template workflows
- [Tutorial: Standardized Deployments](../tutorials/templates.md) — step-by-step tutorial
- [API Reference](./api/README.md) — complete API documentation
- [Data Model](../explanation/data-model.md) — entity relationships
