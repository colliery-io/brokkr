# Stable Error Codes

Every documented 4xx/5xx response from the broker's v1 API carries a JSON body conforming to `ErrorResponse`:

```json
{ "code": "agent_not_found", "message": "...", "details": { ... } }
```

The `code` string is **part of the API contract**: SDK consumers pattern-match on it, so the broker treats renames as breaking changes. The `message` is human-readable and is **not** stable. The `details` map carries structured context (e.g. the ID that wasn't found); keys are documented per code when present.

Some responses are produced before a handler runs and therefore carry no `ErrorResponse` body and no `code`. Client code must not assume every 4xx is parseable:

- **401 from the authentication middleware** — missing `Authorization` header, malformed PAK, or a PAK that matches no identity. Bare `401 Unauthorized`.
- **403 from the authentication middleware** — a read-only credential (the console token) attempting anything other than a read. Reads plus two allowlisted `POST` routes are permitted; every other method is a bare `403 Forbidden`. See [Read-only credentials](#read-only-credentials).
- **400 from request parsing** — a query parameter or request body that fails to deserialize before dispatch (for example a malformed UUID in `?pak_id=`) is rejected by the HTTP framework with its own plain response.

## SDK mapping

All three SDK wrappers surface the wire code through a typed error:

| SDK | Error type | Code accessor |
|-----|------------|---------------|
| Python (`brokkr-client`) | `BrokkrError` | `err.code` |
| Rust (`brokkr-client`) | `BrokkrError` | `err.code()` (returns `Option<&str>`) |
| TypeScript (`@colliery-io/brokkr-client`) | `BrokkrError` | `err.code` |

See the [SDK guides](../how-to/sdks/README.md) for per-language error-handling examples.

## Authentication and authorization

| Code | Status | Meaning |
|------|--------|---------|
| `unauthorized` | 401 | Caller's identity (agent, generator, or admin) grants no access to the requested deployment object. |
| `admin_required` | 403 | Caller authenticated, but the operation requires an admin PAK. |
| `agent_pak_required` | 403 | Operation requires an agent PAK (e.g. reporting a webhook delivery result). |
| `agent_pak_mismatch` | 403 | Agent PAK does not match the agent referenced in the path. |

### Read-only credentials

The broker's zero-config console token authenticates as a read-only admin. It passes the admin gate on `GET`/`HEAD` requests anywhere in the API, plus `POST /auth/pak` and `POST /deployment-objects/{id}/diagnostics`. Any other request is rejected by the authentication middleware with a bare `403 Forbidden` — no `ErrorResponse` body and no `code`, so an SDK's error-code accessor returns nothing. Distinguish this case by the absence of a body rather than by a code.

## Database-mapped codes

These come from the database classifier (`ApiError::from_diesel`) on create/update paths. For `unique_violation` and `foreign_key_violation`, the constraint name is included under `details.constraint` when available.

| Code | Status | Meaning |
|------|--------|---------|
| `unique_violation` | 409 | Duplicate of an existing row (e.g. two generators with the same name). |
| `foreign_key_violation` | 422 | Referenced row does not exist (e.g. payload references a missing parent). |
| `check_violation` | 422 | A `CHECK` constraint failed. |
| `not_null_violation` | 422 | A required column was null. |
| `not_found` | 404 | Row not found at the database layer (safety net; most handlers return a resource-specific `*_not_found` code instead). |
| `internal_error` | 500 | Unclassified server-side failure. Check broker logs. |

## Agents

| Code | Status | Meaning |
|------|--------|---------|
| `agent_not_found` | 404 | No agent with the given ID. |
| `invalid_agent` | 400 | Agent payload rejected by validation on `POST /agents` (e.g. empty `name` or `cluster_name`). |
| `agent_id_mismatch` | 400 | The `agent_id` in the body does not match the agent ID in the path. Raised on the agent event, label, annotation, and target creation endpoints. |
| `name_and_cluster_required` | 400 | Agent search requires both `name` and `cluster_name` query parameters. |
| `agent_label_not_found` | 404 | The named label does not exist on the agent. |
| `agent_annotation_not_found` | 404 | The named annotation does not exist on the agent. |
| `agent_event_not_found` | 404 | No agent event with the given ID. |
| `agent_target_not_found` | 404 | No agent target for the given agent/stack pair. |
| `agent_not_registered` | 403 | Agent is not registered with the stack's owning generator; targets cannot be added or removed until it is. Admin cannot bypass. |
| `invalid_generator_id` | 400 | A `generator_ids` entry supplied to `POST /agents` does not name an existing generator. |
| `target_generator_mismatch` | 403 | A generator can only target its own stacks. |
| `target_create_denied` | 403 | Creating an agent target requires admin or the owning generator. |

## Generators

| Code | Status | Meaning |
|------|--------|---------|
| `generator_not_found` | 404 | No generator with the given ID. |
| `generator_not_owned` | 403 | Caller is neither admin nor the generator referenced in the path. |
| `not_authorized` | 403 | Listing a generator's registered agents (`GET /generators/{id}/registered-agents`) requires admin or that same generator's PAK. |
| `cannot_delete_system_generator` | 403 | `DELETE /generators/{id}` targeted the built-in system generator, which cannot be deleted. |
| `system_generator_not_a_tenant` | 403 | A non-admin caller asked for a tenant-scoped view of the system generator (`GET /agents` under a system-generator PAK, or `GET /generators/{id}/registered-agents` with that id). Every agent is auto-registered with the system generator, so it is not a tenant and has no scoped view; admin callers are unaffected. |
| `generator_required` | 403 | `GET /agents` was called by a credential that is neither admin nor a generator — an agent PAK, for example. Distinct from `system_generator_not_a_tenant`, which means the caller *is* a generator but not a tenant one. |
| `missing_agent_id` | 400 | An admin caller invoked `POST /generators/{id}/register` without an `agent_id` in the body; only an agent self-registering may omit it. |
| `already_registered` | 409 | The agent is already registered with this generator. `POST /generators/{id}/register` is not idempotent; a repeat returns this. (The agent's startup self-registration treats it as success.) |
| `invalid_generator_id` | 400 | A `generator_ids` entry supplied to `POST /agents` references a generator that does not exist. |
| `forbidden` | 403 | A generator PAK called a registration endpoint (`POST`/`DELETE /generators/{id}/register`); only an agent (self) or an admin may. |

## Stacks

| Code | Status | Meaning |
|------|--------|---------|
| `stack_not_found` | 404 | No stack with the given ID. |
| `stack_not_owned` | 403 | Caller is neither admin nor the stack's owning generator. |
| `stack_not_accessible` | 403 | Caller is not authorized to read this stack (labels/annotations listing). |
| `stacks_list_denied` | 403 | Listing stacks requires admin or generator access. |
| `stack_create_denied` | 403 | Creating a stack requires admin or generator access. |
| `stack_generator_mismatch` | 403 | A generator can only create stacks for itself. |
| `stack_id_mismatch` | 400 | Stack ID in the path does not match the ID in the body. |
| `invalid_label` | 400 | Label value rejected by validation (also raised on template labels). |
| `stack_label_not_found` | 404 | The named label does not exist on the stack. |
| `stack_annotation_not_found` | 404 | The named annotation does not exist on the stack. |

## Deployment objects

| Code | Status | Meaning |
|------|--------|---------|
| `deployment_object_not_found` | 404 | No deployment object with the given ID. |
| `invalid_deployment_object` | 400 | Deployment object payload rejected (e.g. empty or invalid YAML content; also raised when a rendered template produces an invalid object). |
| `agent_not_associated` | 403 | Agent is not targeted by the stack owning this deployment object. |
| `generator_not_associated` | 403 | Generator does not own the stack owning this deployment object. |

## Templates

| Code | Status | Meaning |
|------|--------|---------|
| `template_not_found` | 404 | No stack template with the given ID. |
| `template_not_owned` | 403 | Caller is neither admin nor the template's owning generator (modification paths). |
| `template_not_accessible` | 403 | Caller is not authorized to read this template (owned by another generator, or caller has no generator identity). |
| `templates_not_accessible` | 403 | Listing or creating templates requires admin or generator access. |
| `invalid_template_syntax` | 400 | Template body failed Tera syntax validation. |
| `invalid_parameters_schema` | 400 | The declared parameters schema is not valid JSON Schema. |
| `invalid_parameters` | 400 | Instantiation parameters failed schema validation. Validation messages are included under `details.validation_errors`. |
| `template_render_failed` | 400 | Template parsed but failed to render against the provided parameters. |
| `template_stack_mismatch` | 422 | The template's labels/annotations do not all match the target stack. Missing keys are listed under `details.missing_labels` and `details.missing_annotations`. |
| `invalid_annotation` | 400 | Annotation key/value rejected by validation. |
| `template_label_not_found` | 404 | The named label does not exist on the template. |
| `template_annotation_not_found` | 404 | The named annotation does not exist on the template. |

## Diagnostics

| Code | Status | Meaning |
|------|--------|---------|
| `diagnostic_not_found` | 404 | No diagnostic request with the given ID. |
| `diagnostic_not_owned` | 403 | Caller is neither admin nor the agent the diagnostic request targets. |
| `diagnostic_already_claimed` | 409 | Diagnostic request is not in `pending` status; it cannot be claimed. |
| `diagnostic_not_claimed` | 409 | Diagnostic request is not in `claimed` status; a result cannot be submitted. |
| `invalid_diagnostic_request` | 400 | Diagnostic request payload rejected by validation. |
| `invalid_diagnostic_result` | 400 | Diagnostic result payload rejected by validation. |

## Webhooks

| Code | Status | Meaning |
|------|--------|---------|
| `webhook_not_found` | 404 | No webhook subscription with the given ID. |
| `invalid_webhook` | 400 | Webhook subscription payload rejected by validation. |
| `unsupported_field` | 422 | The request body carries a field the webhook API no longer accepts. `details.field` names the offending key and `details.use_instead` names its replacement. |
| `invalid_request_body` | 422 | A webhook create/update body parsed as JSON but did not match the expected shape. |
| `url_required` | 400 | Webhook URL is empty. |
| `invalid_url_scheme` | 400 | Webhook URL must start with `http://` or `https://`. |
| `webhook_test_failed` | 400 | Test delivery failed: the endpoint returned a non-success status (status and truncated body under `details.status_code` / `details.body`) or the request itself failed. |
| `delivery_not_found` | 404 | No webhook delivery with the given ID. |
| `delivery_not_acquired_by_agent` | 403 | The reporting agent did not acquire this delivery. |

## Work orders

| Code | Status | Meaning |
|------|--------|---------|
| `work_order_not_found` | 404 | No work order with the given ID. |
| `invalid_work_order` | 400 | Work order payload rejected by validation. |
| `no_targeting_specified` | 400 | At least one targeting method (`agent_ids`, `labels`, or `annotations`) must be specified. |
| `work_order_not_claimable` | 404 | Work order does not exist or is not claimable by this agent. |
| `work_order_not_claimed_by_agent` | 403 | Work order is not claimed by the agent attempting to complete it. |
| `work_order_log_entry_not_found` | 404 | No work order log entry with the given ID. |

## Deployment health

| Code | Status | Meaning |
|------|--------|---------|
| `invalid_health_record` | 422 | A health record in a `PATCH /agents/{id}/health-status` batch failed validation. The whole batch is rejected. |

## Admin

| Code | Status | Meaning |
|------|--------|---------|
| `config_reload_disabled` | 503 | Configuration hot-reload is not enabled on this broker. |

## Stability

Codes are short, lowercase, snake_case strings and are stable across releases; new broker error paths add new codes rather than repurposing existing ones. This catalog is maintained by hand alongside broker changes.
