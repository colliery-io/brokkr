---
id: add-openapi-documentation-for
level: task
title: "Add OpenAPI documentation for template endpoints"
short_code: "BROKKR-T-0038"
created_at: 2025-12-07T17:58:03.819158+00:00
updated_at: 2025-12-13T03:24:45.167615+00:00
parent: BROKKR-I-0002
blocked_by: []
archived: true

tags:
  - "#task"
  - "#phase/completed"


exit_criteria_met: false
strategy_id: NULL
initiative_id: BROKKR-I-0002
---

# Add OpenAPI documentation for template endpoints

## Parent Initiative

[[BROKKR-I-0002]] - Stack Templating System

## Objective

Add comprehensive OpenAPI documentation for all template-related endpoints using utoipa annotations, ensuring they appear correctly in the Swagger UI.

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] All template endpoints have `#[utoipa::path]` annotations
- [ ] `StackTemplate` model has `ToSchema` derive
- [ ] Request/response bodies documented with examples
- [ ] Error responses documented (400, 403, 404, 422, 500)
- [ ] "templates" tag added to OpenAPI config
- [ ] Template instantiation endpoint documented
- [ ] Swagger UI displays all endpoints correctly
- [ ] OpenAPI spec validates without errors

## Implementation Notes

### Technical Approach

**1. Add ToSchema to models** (`brokkr-models/src/models/stack_templates.rs`):
```rust
#[derive(... ToSchema)]
#[schema(example = json!({
    "id": "550e8400-e29b-41d4-a716-446655440000",
    "name": "postgres-db",
    "version": 1,
    "template_content": "apiVersion: v1\nkind: ConfigMap...",
    "parameters_schema": "{\"type\":\"object\"...}"
}))]
pub struct StackTemplate { ... }
```

**2. Add path annotations** (`api/v1/templates.rs`):
```rust
#[utoipa::path(
    post,
    path = "/api/v1/templates",
    tag = "templates",
    request_body = NewStackTemplate,
    responses(
        (status = 201, description = "Template created", body = StackTemplate),
        (status = 400, description = "Invalid Tera syntax or JSON Schema"),
        (status = 403, description = "Not authorized"),
    ),
    security(("pak" = []))
)]
async fn create_template(...) { }

#[utoipa::path(
    get,
    path = "/api/v1/templates",
    tag = "templates",
    params(
        ("generator_id" = Option<Uuid>, Query, description = "Filter by generator"),
        ("name" = Option<String>, Query, description = "Filter by name"),
    ),
    responses(
        (status = 200, description = "List of templates", body = Vec<StackTemplate>),
    ),
    security(("pak" = []))
)]
async fn list_templates(...) { }
```

**3. Update OpenAPI config** (`api/v1/openapi.rs`):
```rust
#[derive(OpenApi)]
#[openapi(
    paths(
        // Existing paths...
        templates::create_template,
        templates::list_templates,
        templates::get_template,
        templates::update_template,
        templates::delete_template,
        stacks::instantiate_template,
    ),
    components(schemas(
        // Existing schemas...
        StackTemplate,
        NewStackTemplate,
        TemplateInstantiationRequest,
        TemplateLabel,
        TemplateAnnotation,
    )),
    tags(
        // Existing tags...
        (name = "templates", description = "Stack template management"),
    )
)]
pub struct ApiDoc;
```

### Endpoints to Document

| Method | Path | Description |
|--------|------|-------------|
| POST | /api/v1/templates | Create template |
| GET | /api/v1/templates | List templates |
| GET | /api/v1/templates/:id | Get template |
| PUT | /api/v1/templates/:id | Update template |
| DELETE | /api/v1/templates/:id | Delete template |
| POST | /api/v1/templates/:id/labels | Add label |
| DELETE | /api/v1/templates/:id/labels/:label | Remove label |
| POST | /api/v1/templates/:id/annotations | Add annotation |
| DELETE | /api/v1/templates/:id/annotations/:key | Remove annotation |
| POST | /api/v1/stacks/:id/deployment-objects/from-template | Instantiate |

### Dependencies

- BROKKR-T-0033: Template CRUD API
- BROKKR-T-0035: Template instantiation endpoint

### Reference Files

- `api/v1/stacks.rs` - OpenAPI annotation patterns
- `api/v1/openapi.rs` - OpenAPI configuration

## Status Updates

*To be added during implementation*