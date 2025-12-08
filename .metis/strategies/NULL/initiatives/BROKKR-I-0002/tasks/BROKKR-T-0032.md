---
id: implement-dal-for-templates-and
level: task
title: "Implement DAL for templates and related entities"
short_code: "BROKKR-T-0032"
created_at: 2025-12-07T17:57:55.448971+00:00
updated_at: 2025-12-07T21:42:53.881530+00:00
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

# Implement DAL for templates and related entities

## Parent Initiative

[[BROKKR-I-0002]] - Stack Templating System

## Objective

Implement Data Access Layer (DAL) modules for all template-related entities, providing database operations following existing patterns.

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] `TemplatesDAL` created with CRUD + versioning methods
- [ ] `TemplateLabelsDAL` created following stack_labels pattern
- [ ] `TemplateAnnotationsDAL` created following stack_annotations pattern
- [ ] `TemplateTargetsDAL` created following agent_targets pattern
- [ ] `RenderedDeploymentObjectsDAL` created for provenance
- [ ] DAL struct updated with accessor methods for all new DALs
- [ ] `filter_by_labels` and `filter_by_annotations` methods implemented
- [ ] Version management: `get_latest_version`, `list_versions` methods
- [ ] All methods return `Result<T, diesel::result::Error>`

## Implementation Notes

### Technical Approach

Create DAL files in `crates/brokkr-broker/src/dal/`:

**1. templates.rs**
```rust
pub struct TemplatesDAL<'a> {
    pub dal: &'a DAL,
}

impl TemplatesDAL<'_> {
    // Standard CRUD
    pub fn create(&self, new_template: &NewStackTemplate) -> Result<StackTemplate, Error>
    pub fn get(&self, template_id: Uuid) -> Result<Option<StackTemplate>, Error>
    pub fn list(&self) -> Result<Vec<StackTemplate>, Error>
    pub fn soft_delete(&self, template_id: Uuid) -> Result<usize, Error>
    
    // Versioning
    pub fn get_latest_version(&self, generator_id: Option<Uuid>, name: &str) -> Result<Option<StackTemplate>, Error>
    pub fn list_versions(&self, generator_id: Option<Uuid>, name: &str) -> Result<Vec<StackTemplate>, Error>
    pub fn create_new_version(&self, new_template: &NewStackTemplate) -> Result<StackTemplate, Error>
    
    // Filtering
    pub fn filter_by_labels(&self, labels: Vec<String>, filter_type: FilterType) -> Result<Vec<StackTemplate>, Error>
    pub fn filter_by_annotations(&self, annotations: Vec<(String, String)>, filter_type: FilterType) -> Result<Vec<StackTemplate>, Error>
    
    // Ownership
    pub fn list_for_generator(&self, generator_id: Uuid) -> Result<Vec<StackTemplate>, Error>
    pub fn list_system_templates(&self) -> Result<Vec<StackTemplate>, Error>  // generator_id IS NULL
}
```

**2. template_labels.rs** - Follow stack_labels.rs pattern
**3. template_annotations.rs** - Follow stack_annotations.rs pattern
**4. template_targets.rs** - Follow agent_targets.rs pattern
**5. rendered_deployment_objects.rs**
```rust
pub fn create(&self, new: &NewRenderedDeploymentObject) -> Result<RenderedDeploymentObject, Error>
pub fn get_by_deployment_object(&self, deployment_object_id: Uuid) -> Result<Option<RenderedDeploymentObject>, Error>
pub fn list_by_template(&self, template_id: Uuid, version: Option<i32>) -> Result<Vec<RenderedDeploymentObject>, Error>
```

**6. Update mod.rs**
```rust
impl DAL {
    pub fn templates(&self) -> TemplatesDAL { TemplatesDAL { dal: self } }
    pub fn template_labels(&self) -> TemplateLabelsDAL { ... }
    pub fn template_annotations(&self) -> TemplateAnnotationsDAL { ... }
    pub fn template_targets(&self) -> TemplateTargetsDAL { ... }
    pub fn rendered_deployment_objects(&self) -> RenderedDeploymentObjectsDAL { ... }
}
```

### Dependencies

- BROKKR-T-0031: Diesel models must be complete

### Reference Files

- `dal/stacks.rs` - CRUD and filtering patterns
- `dal/agent_targets.rs` - Target DAL pattern
- `dal/mod.rs` - DAL struct accessor pattern

## Status Updates

*To be added during implementation*