---
id: integrate-tera-rendering-and-json
level: task
title: "Integrate Tera rendering and JSON Schema validation"
short_code: "BROKKR-T-0036"
created_at: 2025-12-07T17:57:55.945441+00:00
updated_at: 2025-12-13T03:24:45.057922+00:00
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

# Integrate Tera rendering and JSON Schema validation

## Parent Initiative

[[BROKKR-I-0002]] - Stack Templating System

## Objective

Create utility modules for Tera template rendering and JSON Schema parameter validation, used by both template creation (syntax validation) and instantiation (rendering).

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] `validate_tera_syntax()` function - validates template at creation time
- [ ] `render_template()` function - renders template with parameters
- [ ] `validate_json_schema()` function - validates schema is valid JSON Schema
- [ ] `validate_parameters()` function - validates params against schema
- [ ] Clear error messages for all failure cases
- [ ] Unit tests for Tera edge cases (missing vars, syntax errors)
- [ ] Unit tests for JSON Schema validation

## Implementation Notes

### Technical Approach

Create `crates/brokkr-broker/src/utils/templating.rs`:

```rust
use tera::{Context, Tera};
use jsonschema::{JSONSchema, ValidationError};

/// Validate Tera template syntax without rendering
/// Called at template creation time (fail fast)
pub fn validate_tera_syntax(template_content: &str) -> Result<(), String> {
    match Tera::one_off(template_content, &Context::new(), false) {
        Ok(_) => Ok(()),
        Err(e) => {
            // Parse Tera error to extract line/column info
            Err(format!("Invalid Tera syntax: {}", e))
        }
    }
}

/// Render template with provided parameters
/// Returns rendered YAML string
pub fn render_template(
    template_content: &str,
    parameters: &serde_json::Value,
) -> Result<String, String> {
    let mut context = Context::new();
    
    // Flatten JSON parameters into Tera context
    if let serde_json::Value::Object(map) = parameters {
        for (key, value) in map {
            context.insert(key, value);
        }
    }
    
    Tera::one_off(template_content, &context, false)
        .map_err(|e| format!("Template rendering failed: {}", e))
}

/// Validate that a string is valid JSON Schema
/// Called at template creation time
pub fn validate_json_schema(schema_str: &str) -> Result<(), String> {
    let schema: serde_json::Value = serde_json::from_str(schema_str)
        .map_err(|e| format!("Invalid JSON: {}", e))?;
    
    JSONSchema::compile(&schema)
        .map_err(|e| format!("Invalid JSON Schema: {}", e))?;
    
    Ok(())
}

/// Validate parameters against JSON Schema
/// Called at template instantiation time
pub fn validate_parameters(
    schema_str: &str,
    parameters: &serde_json::Value,
) -> Result<(), Vec<String>> {
    let schema: serde_json::Value = serde_json::from_str(schema_str)
        .map_err(|e| vec![format!("Invalid schema JSON: {}", e)])?;
    
    let compiled = JSONSchema::compile(&schema)
        .map_err(|e| vec![format!("Invalid schema: {}", e)])?;
    
    let result = compiled.validate(parameters);
    if let Err(errors) = result {
        let error_msgs: Vec<String> = errors
            .map(|e| format!("{}: {}", e.instance_path, e))
            .collect();
        return Err(error_msgs);
    }
    
    Ok(())
}
```

**Example Template:**
```yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: {{ database_name }}
spec:
  replicas: {{ replicas }}
  template:
    spec:
      containers:
        - name: postgres
          image: postgres:{{ postgres_version | default(value="15") }}
          resources:
            requests:
              storage: {{ storage_size }}
```

**Example JSON Schema:**
```json
{
  "type": "object",
  "required": ["database_name", "storage_size"],
  "properties": {
    "database_name": { "type": "string", "minLength": 1 },
    "storage_size": { "type": "string", "pattern": "^[0-9]+[GMK]i$" },
    "replicas": { "type": "integer", "minimum": 1, "default": 1 },
    "postgres_version": { "type": "string", "default": "15" }
  }
}
```

### Dependencies

- BROKKR-T-0029: tera and jsonschema crates must be added

### Test Cases

1. Valid Tera syntax passes validation
2. Invalid Tera syntax (unclosed braces) returns clear error
3. Missing required variable during render -> error with var name
4. Valid JSON Schema passes
5. Invalid JSON Schema (bad type) returns error
6. Parameters matching schema pass
7. Parameters missing required field -> error with field name
8. Parameters with wrong type -> error with details

## Status Updates

*To be added during implementation*