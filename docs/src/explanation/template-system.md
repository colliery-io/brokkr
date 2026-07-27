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
              ┌─────────────────┐
              │ Check access    │ ← Own or system template
              │ Match stack     │ ← Label/annotation check
              │ Validate params │ ← JSON Schema check
              │ Render Tera     │ ← Variable substitution
              └────────┬────────┘
                       ▼
              ┌────────────────┐
              │ Deployment     │
              │ Object         │ (rendered YAML)
              └────────────────┘
```

## Rendering Pipeline

### Step 1: Access Checks

Instantiation touches two objects, and each has its own owner. The target stack must belong to the caller (or the caller must be an admin), and the template must be one the caller is allowed to read — its own, or a system template. Rendering somebody else's template into your own stack is refused, because the rendered output would hand you the template's source content in all but name.

Both checks run before any of the template's content — or any detail of why it did or didn't match the stack — reaches the response, so a refused caller learns nothing beyond whether the identifier exists.

### Step 2: Stack Matching

Templates can have labels and annotations that restrict which stacks they can be instantiated into. This is a safety mechanism — it prevents, for example, a production-hardened template from being used in a development stack where the configuration doesn't make sense.

The matching is strict AND logic: the stack must have **every** label and annotation the template requires. Extra labels on the stack are ignored — it only matters that the required ones are present.

The important default: a template carrying **no** labels and no annotations matches every stack unconditionally. Templates are "go anywhere" until somebody constrains them. That default is deliberate — most templates are meant to be broadly usable, and requiring ceremony to make a template work would push people back toward hand-written YAML. It also means matching is a guardrail, not a boundary: the thing that keeps a template out of another tenant's hands is ownership (Step 1), not labels.

For the complete matching rules table with examples, see the [Templates Reference](../reference/templates.md#matching-rules).

### Step 3: Parameter Validation

Before rendering, Brokkr validates the provided parameters against the JSON Schema. This catches issues early:

```json
// Schema requires service_name (string, 1-63 chars) and replicas (integer, 1-20)
// Caller provides: {"service_name": "", "replicas": 100}
// Result: Validation fails — service_name too short, replicas exceeds maximum
```

If validation fails, the request is rejected with a detailed error message explaining which constraints were violated. No YAML is rendered.

### Step 4: Tera Rendering

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

### Step 5: Deployment Object Creation

The rendered YAML is stored as a new deployment object in the target stack, along with provenance metadata:

- `rendered_deployment_objects.template_id` — which template was used
- `rendered_deployment_objects.template_version` — which version
- `rendered_deployment_objects.template_parameters` — the exact parameters provided

This provenance enables re-rendering with different parameters or auditing what parameters produced a given deployment.

## Versioning Design

### Why Version Templates?

Templates evolve: you add a liveness probe, change resource defaults, introduce a new parameter. Without versioning, updating a template could silently change the meaning of existing deployments.

Brokkr's answer is that a template version is never edited in place. Each version is a separate, immutable record with its own identifier; an update writes a new record beside the old one and leaves the old one exactly as it was.

That gives three properties:

- **Existing deployments are stable** — a deployment object rendered from version 1 stays as-is even after version 2 exists
- **Instantiation is pinned** — an instantiation renders the version whose identifier you supplied, and nothing else
- **Provenance is preserved** — you can trace any deployment back to the exact template version and parameters

### Version Lifecycle

```
Version 1 (id A) ─── Version 2 (id B) ─── Version 3 (id C)
    │                     │
    │                     ├── Deployment Object A (rendered from id B)
    │                     └── Deployment Object B (rendered from id B)
    │
    ├── Deployment Object C (rendered from id A, still lives in cluster)
    └── Deployment Object D (rendered from id A)
```

Updating a template creates the next version: the version number auto-increments within a given owner-and-name pair, and old versions remain in the database. The update response describes the new version, including its new identifier.

### Pinned, Not Latest

The consequence people trip over: **there is no "use the latest version" mode.** Instantiation takes a template identifier and renders that exact record. Because an update produces a *new* identifier, an automation that keeps sending the identifier it captured months ago keeps deploying the content from months ago — indefinitely, with no warning and no error. Updating the template does not update anyone's pipeline.

Pinning is the deliberate choice. A template identifier behaves like a pinned dependency: what rendered yesterday renders identically today, and a change to a shared template cannot ripple into every consumer's next deployment unannounced. The cost of that guarantee is that adopting a new version is an explicit act — somebody has to pick up the new identifier and use it. Callers that want the newest content should look the template up by name, take the highest version, and use that version's identifier.

### What an Update Does Not Carry Forward

Labels and annotations belong to a specific version record, not to the template's name. A new version therefore begins with **none** of them, even when the previous version had a carefully chosen set.

This is the sharpest edge in the system, because it fails in the permissive direction. A version 1 restricted to production stacks by an `env` label becomes, on update, a version 2 that matches every stack (that is the "no labels means go anywhere" default from Step 2 doing its job). Nothing errors; the guardrail simply is not there anymore. Re-applying the intended labels and annotations to the new version is part of publishing an update, not an optional follow-up.

## System Templates vs. Generator Templates

Templates have two ownership modes:

### System Templates (`generator_id = NULL`)

- Created by admins
- Readable and instantiable by all generators and admins
- Represent organization-wide standards (e.g., "standard web service", "batch job")
- Cannot be modified by generators

### Generator Templates (`generator_id = UUID`)

- Created by a specific generator
- Readable, instantiable, and modifiable only by the owning generator (admins can do all three)
- Represent pipeline-specific templates (e.g., templates tailored for a particular CI/CD system)

This separation allows centralized governance (admin-managed standards) while still allowing individual teams (generators) to create specialized templates.

Ownership travels with instantiation, not just with reads. A generator that supplies another generator's template identifier is refused, whether or not it owns the target stack and whether or not the labels would have matched — knowing an identifier grants nothing. Templates are not a shared pool addressed by UUID.

That leaves system templates as the one sanctioned way to share a template across tenants: promote it to a system template and every generator can use it. There is no per-generator grant, and no way to share one template with some tenants but not others — sharing is all-or-nothing by design, which keeps the ownership rule simple enough to reason about.

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
- Syntax can be validated when a template is created, long before anything is rendered

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
