---
id: add-tera-and-jsonschema
level: task
title: "Add tera and jsonschema dependencies"
short_code: "BROKKR-T-0029"
created_at: 2025-12-07T17:57:55.167455+00:00
updated_at: 2025-12-07T20:37:04.560233+00:00
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

# Add tera and jsonschema dependencies

## Parent Initiative

[[BROKKR-I-0002]] - Stack Templating System

## Objective

Add the `tera` and `jsonschema` crates to the workspace dependencies to enable template rendering and parameter validation for the stack templating system.

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] `tera` crate added to workspace Cargo.toml with version ~1.20
- [ ] `jsonschema` crate added to workspace Cargo.toml with version ~0.18
- [ ] Dependencies added to brokkr-broker crate's Cargo.toml
- [ ] Project compiles successfully with new dependencies
- [ ] Basic smoke test confirming crates are usable

## Implementation Notes

### Technical Approach

1. Add to `/Cargo.toml` workspace dependencies:
```toml
tera = { version = "1.20", features = [] }
jsonschema = { version = "0.18", features = [] }
```

2. Add to `/crates/brokkr-broker/Cargo.toml`:
```toml
tera = { workspace = true }
jsonschema = { workspace = true }
```

3. Verify compilation with `cargo build`

### Dependencies

None - this is a foundational task.

### Risk Considerations

- Version compatibility with existing dependencies (minimal risk)
- Tera and jsonschema are well-maintained, stable crates

## Status Updates

*To be added during implementation*