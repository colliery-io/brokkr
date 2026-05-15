# Stable error codes

Every 4xx/5xx response from the broker carries a JSON body conforming to `ErrorResponse`:

```json
{ "code": "agent_not_found", "message": "...", "details": { ... } }
```

The `code` string is **part of the API contract**: SDK consumers pattern-match on it, so the broker treats renames as breaking changes. The `message` is human-readable and is **not** stable. The `details` map carries structured context (e.g. the ID that wasn't found); keys are documented per code when present.

## Codes consumers should expect to match on

### Authentication / authorization (401, 403)

| Code                  | Status | Meaning                                                              |
|-----------------------|--------|----------------------------------------------------------------------|
| `unauthorized`        | 401    | PAK missing, malformed, or rejected.                                 |
| `admin_required`      | 403    | Caller authenticated, but operation requires an admin PAK.           |
| `agent_pak_required`  | 403    | Operation requires an agent PAK (e.g. heartbeat).                    |
| `agent_pak_mismatch`  | 403    | Agent PAK does not match the agent referenced in the path.           |

### Not found (404)

| Code                            | Meaning                                          |
|---------------------------------|--------------------------------------------------|
| `agent_not_found`               | No agent with the given ID.                      |
| `generator_not_found`           | No generator with the given ID.                  |
| `stack_not_found`               | No stack with the given ID.                      |
| `template_not_found`            | No stack template with the given ID.             |
| `webhook_not_found`             | No webhook subscription with the given ID.       |
| `work_order_not_found`          | No work order with the given ID.                 |
| `deployment_object_not_found`   | No deployment object with the given ID.          |
| `agent_event_not_found`         | No agent event with the given ID.                |
| `agent_target_not_found`        | No agent target with the given ID.               |
| `diagnostic_not_found`          | No diagnostic request/result with the given ID.  |
| `delivery_not_found`            | No webhook delivery with the given ID.           |
| `agent_label_not_found` / `agent_annotation_not_found` / `stack_label_not_found` / `stack_annotation_not_found` / `template_label_not_found` / `template_annotation_not_found` | The named label/annotation does not exist on the entity. |

### Conflict / validation (409, 422)

These come from the database classifier (`ApiError::from_diesel`) on create / update paths. The constraint name is included under `details.constraint` when available.

| Code                    | Status | Meaning                                                              |
|-------------------------|--------|----------------------------------------------------------------------|
| `unique_violation`      | 409    | Duplicate of an existing row (e.g. two generators with the same name).|
| `foreign_key_violation` | 422    | Referenced row does not exist (e.g. stack references missing template).|
| `check_violation`       | 422    | A `CHECK` constraint failed.                                         |
| `not_null_violation`    | 422    | A required column was null.                                          |

### Bad request (400)

| Code                          | Meaning                                                                |
|-------------------------------|------------------------------------------------------------------------|
| `invalid_parameters`          | Generic parameter validation failure.                                  |
| `invalid_parameters_schema`   | Parameters did not conform to the schema declared by the entity.       |
| `invalid_label` / `invalid_annotation` | Label or annotation key/value rejected.                       |
| `invalid_deployment_object`   | Deployment object payload rejected.                                    |
| `invalid_diagnostic_request` / `invalid_diagnostic_result` | Diagnostic payload rejected.              |
| `invalid_webhook`             | Webhook payload rejected.                                              |
| `invalid_work_order`          | Work order payload rejected.                                           |
| `invalid_template_syntax`     | Template body failed to parse.                                         |
| `template_render_failed`      | Template parsed but failed to render against the provided parameters.  |
| `stack_id_mismatch`           | Stack ID in path doesn't match the ID in the body.                     |
| `url_required` / `yaml_content_required` | Required field missing on webhook / deployment object payload. |

### Server (5xx)

| Code             | Status | Meaning                                                                |
|------------------|--------|------------------------------------------------------------------------|
| `internal_error` | 500    | Unclassified server-side failure. Check broker logs.                   |

## Adding a new code

New codes are added as part of broker work. The contract: pick a short, lowercase, snake_case string that is stable; document it here in the same PR; and prefer reusing an existing code over coining a near-synonym. The drift CI does not catch documentation rot — keep this table honest by hand.
