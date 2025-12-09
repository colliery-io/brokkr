---
id: implement-template-to-stack-label
level: task
title: "Implement template-to-stack label matching validation"
short_code: "BROKKR-T-0034"
created_at: 2025-12-07T17:57:55.691184+00:00
updated_at: 2025-12-07T23:15:21.693088+00:00
parent: BROKKR-I-0002
blocked_by: []
archived: false

tags:
  - "#task"
  - "#phase/active"


exit_criteria_met: false
strategy_id: NULL
initiative_id: BROKKR-I-0002
---

# Implement template-to-stack label matching validation

## Parent Initiative

[[BROKKR-I-0002]] - Stack Templating System

## Objective

Implement validation logic to check if a template's labels/annotations are compatible with a target stack before instantiation. This prevents deploying templates to incompatible stacks.

## Acceptance Criteria

## Acceptance Criteria

- [ ] `template_matches_stack()` function implemented
- [ ] Templates with no labels match any stack (permissive)
- [ ] Templates with labels must have all labels present on stack
- [ ] Annotation matching follows same logic (key-value pairs)
- [ ] Returns detailed mismatch info for 422 error responses
- [ ] Unit tests for all matching scenarios
- [ ] Edge cases handled (empty labels, partial matches)

## Implementation Notes

### Technical Approach

Create matching utility in `crates/brokkr-broker/src/utils/matching.rs`:

```rust
pub struct MatchResult {
    pub matches: bool,
    pub missing_labels: Vec<String>,
    pub missing_annotations: Vec<(String, String)>,
}

/// Check if template can be instantiated into stack
/// 
/// Rules:
/// - Template with NO labels = matches ANY stack (go anywhere)
/// - Template WITH labels = ALL labels must exist on stack
/// - Same logic applies to annotations
pub fn template_matches_stack(
    template_labels: &[String],
    template_annotations: &[(String, String)],
    stack_labels: &[String],
    stack_annotations: &[(String, String)],
) -> MatchResult {
    // If template has no labels/annotations, it matches everything
    if template_labels.is_empty() && template_annotations.is_empty() {
        return MatchResult { matches: true, ..Default::default() };
    }
    
    // Check all template labels exist on stack
    let missing_labels: Vec<String> = template_labels
        .iter()
        .filter(|tl| !stack_labels.contains(tl))
        .cloned()
        .collect();
    
    // Check all template annotations exist on stack
    let missing_annotations: Vec<(String, String)> = template_annotations
        .iter()
        .filter(|ta| !stack_annotations.contains(ta))
        .cloned()
        .collect();
    
    MatchResult {
        matches: missing_labels.is_empty() && missing_annotations.is_empty(),
        missing_labels,
        missing_annotations,
    }
}
```

**Usage in instantiation endpoint:**
```rust
let result = template_matches_stack(&template_labels, &template_annotations, &stack_labels, &stack_annotations);
if !result.matches {
    return Err((StatusCode::UNPROCESSABLE_ENTITY, Json(json!({
        "error": "Template labels do not match stack",
        "missing_labels": result.missing_labels,
        "missing_annotations": result.missing_annotations,
    }))));
}
```

### Dependencies

- BROKKR-T-0032: DAL for retrieving labels/annotations

### Test Cases

1. Template with no labels -> matches any stack
2. Template labels subset of stack labels -> matches
3. Template has label stack doesn't -> no match
4. Mixed: some labels match, some don't -> no match with details
5. Annotation key matches but value differs -> no match

## Status Updates

*To be added during implementation*