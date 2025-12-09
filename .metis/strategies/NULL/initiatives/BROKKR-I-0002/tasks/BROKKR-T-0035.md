---
id: implement-template-instantiation
level: task
title: "Implement template instantiation endpoint"
short_code: "BROKKR-T-0035"
created_at: 2025-12-07T17:57:55.807216+00:00
updated_at: 2025-12-07T17:57:55.807216+00:00
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

# Implement template instantiation endpoint

## Parent Initiative

[[BROKKR-I-0002]] - Stack Templating System

## Objective

Implement the core template instantiation endpoint that renders a template with parameters and creates a deployment object in a stack.

## Acceptance Criteria

- [ ] `POST /api/v1/stacks/:stack_id/deployment-objects/from-template` endpoint implemented
- [ ] Request body accepts `template_id` and `parameters` object
- [ ] Validates template exists and is not deleted
- [ ] Validates stack exists and is not deleted  
- [ ] Validates template/stack label compatibility (422 on mismatch)
- [ ] Validates parameters against JSON Schema (400 on invalid)
- [ ] Renders template with Tera engine
- [ ] Creates DeploymentObject with rendered YAML
- [ ] Creates RenderedDeploymentObject provenance record
- [ ] Returns created DeploymentObject (same as manual creation)
- [ ] Authorization: admin or generator with stack access

## Implementation Notes

### Technical Approach

Add to `crates/brokkr-broker/src/api/v1/stacks.rs`:

```rust
#[derive(Deserialize, ToSchema)]
pub struct TemplateInstantiationRequest {
    pub template_id: Uuid,
    pub parameters: serde_json::Value,
}

#[utoipa::path(
    post,
    path = "/api/v1/stacks/{stack_id}/deployment-objects/from-template",
    tag = "stacks",
    request_body = TemplateInstantiationRequest,
    responses(
        (status = 201, description = "Deployment object created", body = DeploymentObject),
        (status = 400, description = "Invalid parameters or Tera rendering failed"),
        (status = 404, description = "Template or stack not found"),
        (status = 422, description = "Template labels don't match stack"),
    ),
)]
async fn instantiate_template(
    State(dal): State<DAL>,
    Extension(auth): Extension<AuthPayload>,
    Path(stack_id): Path<Uuid>,
    Json(req): Json<TemplateInstantiationRequest>,
) -> Result<(StatusCode, Json<DeploymentObject>), (StatusCode, Json<Value>)> {
    // 1. Verify authorization
    // 2. Get template (404 if not found/deleted)
    // 3. Get stack (404 if not found/deleted)
    // 4. Get template labels/annotations
    // 5. Get stack labels/annotations
    // 6. Validate label matching (422 with details on mismatch)
    // 7. Validate parameters against JSON Schema (400 on invalid)
    // 8. Render template with Tera (400 on render error)
    // 9. Create DeploymentObject
    // 10. Create RenderedDeploymentObject provenance
    // 11. Return DeploymentObject
}
```

**Request/Response Example:**
```json
// Request
POST /api/v1/stacks/abc-123/deployment-objects/from-template
{
  "template_id": "def-456",
  "parameters": {
    "database_name": "myapp",
    "storage_size": "10Gi",
    "replicas": 3
  }
}

// Response (201 Created)
{
  "id": "ghi-789",
  "stack_id": "abc-123",
  "yaml_content": "rendered YAML...",
  "yaml_checksum": "sha256...",
  "sequence_id": 42,
  ...
}
```

### Dependencies

- BROKKR-T-0033: Template CRUD API
- BROKKR-T-0034: Label matching validation
- BROKKR-T-0036: Tera rendering and JSON Schema validation

### Error Handling

| Status | Condition |
|--------|-----------|
| 400 | Invalid parameters, Tera render failure, malformed request |
| 403 | Not authorized |
| 404 | Template or stack not found |
| 422 | Template/stack label mismatch (includes details) |
| 500 | Database error |

## Status Updates

*To be added during implementation*