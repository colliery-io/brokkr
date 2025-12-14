---
id: implement-diesel-models-for
level: task
title: "Implement Diesel models for templates"
short_code: "BROKKR-T-0031"
created_at: 2025-12-07T17:57:55.348032+00:00
updated_at: 2025-12-07T21:32:51.144406+00:00
parent: BROKKR-I-0002
blocked_by: []
archived: false

tags:
  - "#task"
  - "#phase/completed"


exit_criteria_met: false
strategy_id: NULL
initiative_id: BROKKR-I-0002
---

# Implement Diesel models for templates

## Parent Initiative

[[BROKKR-I-0002]] - Stack Templating System

## Objective

Implement Diesel ORM models for all template-related tables, including read models, write models, and validation logic following existing patterns.

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] `StackTemplate` and `NewStackTemplate` models created
- [ ] `TemplateLabel` and `NewTemplateLabel` models created
- [ ] `TemplateAnnotation` and `NewTemplateAnnotation` models created
- [ ] `TemplateTarget` and `NewTemplateTarget` models created
- [ ] `RenderedDeploymentObject` and `NewRenderedDeploymentObject` models created
- [ ] All models derive required traits (Queryable, Insertable, Serialize, etc.)
- [ ] Validation in `new()` methods with `Result<Self, String>` return
- [ ] Unit tests for all validation paths
- [ ] Schema definitions updated in `schema.rs`

## Implementation Notes

### Technical Approach

Create models in `crates/brokkr-models/src/models/`:

**1. stack_templates.rs**
```rust
#[derive(Queryable, Selectable, Identifiable, AsChangeset, Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct StackTemplate {
    pub id: Uuid,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
    pub generator_id: Option<Uuid>,  // NULL = system template
    pub name: String,
    pub description: Option<String>,
    pub version: i32,
    pub template_content: String,
    pub parameters_schema: String,
    pub checksum: String,
}

#[derive(Insertable, Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct NewStackTemplate {
    pub generator_id: Option<Uuid>,
    pub name: String,
    pub description: Option<String>,
    pub template_content: String,
    pub parameters_schema: String,
    pub checksum: String,
}

impl NewStackTemplate {
    pub fn new(...) -> Result<Self, String> {
        // Validate name not empty
        // Validate template_content is valid Tera syntax
        // Validate parameters_schema is valid JSON Schema
        // Generate checksum
    }
}
```

**2. template_labels.rs** - Follow `stack_labels.rs` pattern
**3. template_annotations.rs** - Follow `stack_annotations.rs` pattern  
**4. template_targets.rs** - Follow `agent_targets.rs` pattern
**5. rendered_deployment_objects.rs** - Provenance tracking model

### Dependencies

- BROKKR-T-0030: Database migration must be complete
- Run `diesel print-schema` to generate schema after migration

### Reference Files

- `models/stacks.rs` - Pattern for read/write models
- `models/stack_labels.rs` - Label model pattern
- `models/agent_targets.rs` - Target model pattern

## Status Updates

*To be added during implementation*