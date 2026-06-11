---
id: raw-yaml-submission-primitive
level: task
title: "Raw-YAML submission primitive (application/yaml body)"
short_code: "BROKKR-T-0194"
created_at: 2026-06-11T02:19:29.351560+00:00
updated_at: 2026-06-11T05:47:25.124286+00:00
parent: BROKKR-I-0021
blocked_by: []
archived: false

tags:
  - "#task"
  - "#phase/completed"


exit_criteria_met: false
initiative_id: BROKKR-I-0021
---

# Raw-YAML submission primitive (application/yaml body)

## Parent Initiative

[[BROKKR-I-0021]]

## Objective

Make `POST /api/v1/stacks/{id}/deployment-objects` content-type aware so a deployment object can be submitted as a raw multi-document YAML body instead of a JSON-escaped string. This is the primitive the SDK helpers and CLI POST to, and an immediate relief for curl users.

## Design

- `crates/brokkr-broker/src/api/v1/stacks.rs`: replace the `Json<CreateDeploymentObjectRequest>` extractor on the create handler with a content-type-aware extractor.
  - `application/json` → existing `CreateDeploymentObjectRequest` (unchanged, backward compatible).
  - `application/yaml` / `text/yaml` → raw body string becomes `yaml_content`; read `is_deletion_marker` from a `?deletion_marker=<bool>` query param (default false).
- Validate on ingest: run the body through `multidoc_deserialize` (or equivalent) and return `400 invalid_deployment_object` on parse failure, rather than letting it fail at agent apply.
- Deletion-marker wart: allow an empty body when `deletion_marker=true`; relax `NewDeploymentObject::new`'s non-empty requirement for markers (the agent prunes by stack annotation, the body is ignored for markers).
- Optional round-trip: honor `Accept: application/yaml` on `GET /deployment-objects/{id}` to return `yaml_content` as a raw YAML file.
- Checksum stays server-side — no downstream change. Keep the utoipa annotations/OpenAPI accurate (the JSON request body shape is unchanged; document the alternate content type).

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] `curl -H "Content-Type: application/yaml" --data-binary @manifests.yaml .../deployment-objects` creates a deployment object with the body as `yaml_content`
- [ ] JSON path unchanged; existing tests pass
- [ ] Malformed YAML body → 400 `invalid_deployment_object`
- [ ] `?deletion_marker=true` with an empty body creates a deletion marker
- [ ] Integration test covering the YAML body path + the deletion-marker case
- [ ] `angreal openapi check` green (regen spec if content types are annotated); SDK drift gates green
- [ ] Docs: API reference + manifest-submission how-to updated

## Status Updates

- 2026-06-11: Created under BROKKR-I-0021 (folder-of-objects on-ramp).
- 2026-06-11: IMPLEMENTED (branch feat/i0021-raw-yaml-submission). Content-type-aware create handler (`application/yaml` raw body + `?deletion_marker=`, falls back to JSON); validate-on-ingest via serde_yaml (malformed → 400 invalid_deployment_object); model relaxed to allow empty body for deletion markers; `Accept: application/yaml` round-trip on GET. Unit tests: 11 helper tests (content-type routing, marker flag, validation) + model empty-marker test. Functional integration tests: yaml-body create, empty deletion marker, malformed→400, Accept round-trip, json-still-works. OpenAPI spec regenerated (adds `deletion_marker` query param) + Python/TS SDKs; all drift gates green. Docs: managing-stacks how-to + api reference. Remaining: integration tests run on CI (need Postgres).