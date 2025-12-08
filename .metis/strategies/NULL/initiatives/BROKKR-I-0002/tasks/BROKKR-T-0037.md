---
id: write-integration-tests-for
level: task
title: "Write integration tests for template system"
short_code: "BROKKR-T-0037"
created_at: 2025-12-07T17:58:03.755634+00:00
updated_at: 2025-12-07T17:58:03.755634+00:00
parent: BROKKR-I-0002
blocked_by: []
archived: false

tags:
  - "#task"
  - "#phase/todo"


exit_criteria_met: false
strategy_id: NULL
initiative_id: BROKKR-I-0002
---

# Write integration tests for template system

## Parent Initiative

[[BROKKR-I-0002]] - Stack Templating System

## Objective

Write comprehensive integration tests covering the full template lifecycle: creation, label management, and instantiation into deployment objects.

## Acceptance Criteria

- [ ] DAL integration tests for all template operations
- [ ] API integration tests for template CRUD endpoints
- [ ] API integration tests for template instantiation
- [ ] Authorization tests (admin vs generator vs unauthorized)
- [ ] Label matching edge case tests
- [ ] Version management tests
- [ ] Provenance tracking verification tests
- [ ] All tests pass in CI pipeline

## Test Cases

### DAL Tests (`tests/integration/dal/templates.rs`)

| Test | Description |
|------|-------------|
| `test_create_template` | Create template, verify all fields |
| `test_create_template_version` | Create same name, verify version increments |
| `test_get_latest_version` | Get latest version of template |
| `test_list_versions` | List all versions of a template |
| `test_soft_delete` | Soft delete, verify not returned in list |
| `test_filter_by_labels` | Filter templates by labels (AND/OR) |
| `test_list_for_generator` | List generator-owned templates |
| `test_list_system_templates` | List templates with NULL generator_id |

### API Tests (`tests/integration/api/templates.rs`)

| Test | Description |
|------|-------------|
| `test_create_template_admin` | Admin creates system template |
| `test_create_template_generator` | Generator creates own template |
| `test_create_template_invalid_tera` | 400 on invalid Tera syntax |
| `test_update_creates_new_version` | PUT creates new version |
| `test_delete_template` | Soft delete template |
| `test_add_remove_labels` | Label management endpoints |
| `test_authorization_system_template` | Only admin can modify NULL generator_id |
| `test_authorization_generator_template` | Generator can modify own |

### Instantiation Tests (`tests/integration/api/template_instantiation.rs`)

| Test | Description |
|------|-------------|
| `test_instantiate_success` | Full happy path |
| `test_instantiate_label_mismatch` | 422 when labels don't match |
| `test_instantiate_invalid_params` | 400 when params fail schema |
| `test_instantiate_creates_provenance` | Verify rendered_deployment_objects record |
| `test_instantiate_no_labels_template` | Template with no labels matches any stack |

## Implementation Notes

### Technical Approach

Follow existing test patterns in `crates/brokkr-broker/tests/integration/`:

```rust
// tests/integration/dal/templates.rs
use brokkr_broker::dal::DAL;
use brokkr_models::models::stack_templates::NewStackTemplate;

#[tokio::test]
async fn test_create_template() {
    let dal = setup_test_dal().await;
    let generator = create_test_generator(&dal).await;
    
    let new_template = NewStackTemplate::new(
        Some(generator.id),
        "test-template".to_string(),
        Some("Test description".to_string()),
        "apiVersion: v1\nkind: ConfigMap\nmetadata:\n  name: {{ name }}".to_string(),
        r#"{"type":"object","properties":{"name":{"type":"string"}}}"#.to_string(),
    ).unwrap();
    
    let template = dal.templates().create(&new_template).unwrap();
    
    assert_eq!(template.name, "test-template");
    assert_eq!(template.version, 1);
    assert_eq!(template.generator_id, Some(generator.id));
    
    cleanup_test_dal(dal).await;
}
```

### Dependencies

- BROKKR-T-0035: All implementation tasks must be complete

### Reference Files

- `tests/integration/dal/stacks.rs` - DAL test patterns
- `tests/integration/api/stacks.rs` - API test patterns

## Status Updates

*To be added during implementation*