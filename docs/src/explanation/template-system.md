# Template Matching & Rendering

This document explains the design of Brokkr's template system — how templates are matched to stacks and rendered into deployment objects via the Tera engine with JSON Schema validation.

## The Problem Templates Solve

Without templates, every deployment requires hand-crafted YAML. In a multi-environment setup (staging, production, 10 regional clusters), you end up with nearly-identical YAML files that differ only in replica counts, image tags, resource limits, and environment variables. This leads to:

- **Duplication drift** — copies fall out of sync
- **Manual errors** — wrong value in wrong environment
- **No validation** — any YAML is accepted, mistakes caught only at apply time

Templates solve this with parameterized YAML, schema validation, and matching rules that prevent production templates from being instantiated into staging stacks.

## Architecture

The template system has three components:

```
┌─────────────────────────────────────────────┐
│ Template                                     │
│  ┌─────────────────┐  ┌──────────────────┐  │
│  │ Tera Content     │  │ JSON Schema      │  │
│  │ (YAML with       │  │ (parameter       │  │
│  │  placeholders)   │  │  validation)     │  │
│  └─────────────────┘  └──────────────────┘  │
│  ┌─────────────────────────────────────────┐ │
│  │ Labels + Annotations (matching rules)   │ │
│  └─────────────────────────────────────────┘ │
└──────────────────────┬──────────────────────┘
                       │ instantiate(parameters)
                       ▼
              ┌────────────────┐
              │ Validate params │ ← JSON Schema check
              │ Match stack     │ ← Label/annotation check
              │ Render Tera     │ ← Variable substitution
              └────────┬───────┘
                       ▼
              ┌────────────────┐
              │ Deployment     │
              │ Object         │ (rendered YAML)
              └────────────────┘
```

## Rendering Pipeline

### Step 1: Parameter Validation

Before touching the template, Brokkr validates the provided parameters against the JSON Schema. This catches issues early:

```json
// Schema requires service_name (string, 1-63 chars) and replicas (integer, 1-20)
// Caller provides: {"service_name": "", "replicas": 100}
// Result: Validation fails — service_name too short, replicas exceeds maximum
```

If validation fails, the request is rejected with a detailed error message explaining which constraints were violated. No YAML is rendered.

### Step 2: Stack Matching

Templates can have labels and annotations that restrict which stacks they can be instantiated into. This is a safety mechanism — it prevents, for example, a production-hardened template from being used in a development stack where the configuration doesn't make sense.

The matching is strict AND logic: the stack must have **every** label and annotation the template requires. A template with no labels/annotations matches any stack (universal). Extra labels on the stack are ignored — it only matters that the required ones are present.

For the complete matching rules table with examples, see the [Templates Reference](../reference/templates.md#matching-rules).

### Step 3: Tera Rendering

With validation and matching passed, Brokkr renders the Tera template:

1. Creates a Tera context from the JSON parameters (flat key-value mapping)
2. Adds the parameter values to the context
3. Renders the template content through Tera
4. The resulting string is the final Kubernetes YAML

Tera supports rich template features:

- **Variables**: `{{ service_name }}`
- **Conditionals**: `{% if enable_monitoring %}...{% endif %}`
- **Loops**: `{% for port in ports %}...{% endfor %}`
- **Filters**: `{{ name | upper }}`, `{{ x | default(value="fallback") }}`
- **Math**: `{{ replicas * 2 }}`

### Step 4: Deployment Object Creation

The rendered YAML is stored as a new deployment object in the target stack, along with provenance metadata:

- `rendered_deployment_objects.template_id` — which template was used
- `rendered_deployment_objects.template_version` — which version
- `rendered_deployment_objects.template_parameters` — the exact parameters provided

This provenance enables re-rendering with different parameters or auditing what parameters produced a given deployment.

## Versioning Design

### Why Version Templates?

Templates evolve: you add a liveness probe, change resource defaults, introduce a new parameter. Without versioning, updating a template could silently change the meaning of existing deployments.

Brokkr's versioning ensures:

- **Existing deployments are immutable** — a deployment object rendered from template v1 stays as-is even after v2 is created
- **New instantiations use latest** — when you instantiate a template, you always get the newest version
- **Provenance is preserved** — you can trace any deployment back to the exact template version and parameters

### Version Lifecycle

```
Version 1 ────── Version 2 ────── Version 3 (latest)
    │                 │
    │                 ├── Deployment Object A (rendered from v2)
    │                 └── Deployment Object B (rendered from v2)
    │
    ├── Deployment Object C (rendered from v1, still lives in cluster)
    └── Deployment Object D (rendered from v1)
```

Updating a template via `PUT /api/v1/templates/{id}` creates a new version. The version number auto-increments. Old versions remain in the database.

## System Templates vs. Generator Templates

Templates have two ownership modes:

### System Templates (`generator_id = NULL`)

- Created by admins
- Visible to all generators and admins
- Represent organization-wide standards (e.g., "standard web service", "batch job")
- Cannot be modified by generators

### Generator Templates (`generator_id = UUID`)

- Created by a specific generator
- Visible only to the owning generator and admins
- Represent pipeline-specific templates (e.g., templates tailored for a particular CI/CD system)
- Can be modified by the owning generator

This separation allows centralized governance (admin-managed standards) while still allowing individual teams (generators) to create specialized templates.

## Why Tera?

Brokkr chose [Tera](https://keats.github.io/tera/) over alternatives:

| Feature | Tera | Go templates | Jinja2 | Handlebars |
|---------|------|-------------|--------|------------|
| Language | Rust-native | Go | Python | JS/Rust |
| Syntax | `{{ var }}`, `{% if %}` | `{{ .Var }}`, `{{ if }}` | `{{ var }}`, `{% if %}` | `{{ var }}`, `{{#if}}` |
| Filters | Rich built-in | Limited | Rich | Limited |
| Whitespace control | Yes | Yes | Yes | Yes |
| Safe by default | Yes (auto-escape) | No | Yes (configurable) | Yes |

Tera was chosen because:
- Native Rust integration (no FFI or subprocess)
- Familiar Jinja2-like syntax widely known by DevOps engineers
- Rich filter and function library
- Compile-time syntax validation via `add_raw_template`

## Why JSON Schema?

JSON Schema was chosen for parameter validation because:

- **Industry standard** — widely understood, extensive tooling
- **Declarative** — schema defines constraints, engine enforces them
- **Rich constraints** — types, ranges, patterns, required fields, enums, string lengths
- **Self-documenting** — the `description` field in each property serves as parameter documentation
- **Client-side validation** — CI/CD systems can validate parameters before hitting the API

## Related Documentation

- [Templates Reference](../reference/templates.md) — API endpoints and data model
- [Using Stack Templates](../how-to/templates.md) — how-to guide
- [Tutorial: Standardized Deployments](../tutorials/templates.md) — step-by-step tutorial
- [Data Model](./data-model.md) — template entity relationships
