---
id: implement-template-crud-api
level: task
title: "Implement template CRUD API endpoints"
short_code: "BROKKR-T-0033"
created_at: 2025-12-07T17:57:55.576586+00:00
updated_at: 2025-12-07T23:08:21.251159+00:00
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

# Implement template CRUD API endpoints

## Parent Initiative

[[BROKKR-I-0002]] - Stack Templating System

## Objective

Implement REST API endpoints for template management including create, read, update, delete, and label/annotation management.

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] `POST /api/v1/templates` - Create new template
- [ ] `GET /api/v1/templates` - List templates (with filters)
- [ ] `GET /api/v1/templates/:id` - Get template by ID
- [ ] `PUT /api/v1/templates/:id` - Update template (creates new version)
- [ ] `DELETE /api/v1/templates/:id` - Soft delete template
- [ ] `POST /api/v1/templates/:id/labels` - Add label
- [ ] `DELETE /api/v1/templates/:id/labels/:label` - Remove label
- [ ] `POST /api/v1/templates/:id/annotations` - Add annotation
- [ ] `DELETE /api/v1/templates/:id/annotations/:key` - Remove annotation
- [ ] Authorization: system templates admin-only, generator templates owner+admin
- [ ] Tera syntax validation on create/update
- [ ] Routes registered in API v1 module

## Implementation Notes

### Technical Approach

Create `crates/brokkr-broker/src/api/v1/templates.rs`:

```rust
pub fn routes() -> Router<DAL> {
    Router::new()
        .route("/templates", get(list_templates).post(create_template))
        .route("/templates/:id", get(get_template).put(update_template).delete(delete_template))
        .route("/templates/:id/labels", get(list_labels).post(add_label))
        .route("/templates/:id/labels/:label", delete(remove_label))
        .route("/templates/:id/annotations", get(list_annotations).post(add_annotation))
        .route("/templates/:id/annotations/:key", delete(remove_annotation))
}
```

**Authorization Logic:**
```rust
// System templates (generator_id = NULL): admin only
// Generator templates: admin OR owning generator

fn can_modify_template(auth: &AuthPayload, template: &StackTemplate) -> bool {
    if auth.admin { return true; }
    match (auth.generator, template.generator_id) {
        (Some(auth_gen), Some(tmpl_gen)) => auth_gen == tmpl_gen,
        _ => false,
    }
}
```

**Create Template Flow:**
1. Validate Tera syntax at creation time (fail fast)
2. If name+generator_id exists, auto-increment version
3. Generate SHA-256 checksum of template_content
4. Insert and return created template

**Error Responses:**
- 400: Invalid Tera syntax, validation errors
- 403: Not authorized to modify template
- 404: Template not found
- 500: Database errors

### Dependencies

- BROKKR-T-0032: DAL must be complete

### Reference Files

- `api/v1/stacks.rs` - CRUD endpoint patterns
- `api/v1/agents.rs` - Authorization patterns

## Status Updates

*To be added during implementation*