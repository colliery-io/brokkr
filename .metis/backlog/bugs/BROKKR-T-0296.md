---
id: docs-first-deployment-md-deletion
level: task
title: "Docs: first-deployment.md deletion-marker example posts '# deletion', which the broker rejects with 400 (null YAML document)"
short_code: "BROKKR-T-0296"
created_at: 2026-07-27T14:47:27.661984+00:00
updated_at: 2026-07-27T14:47:27.661984+00:00
parent: 
blocked_by: []
archived: false

tags:
  - "#task"
  - "#phase/backlog"
  - "#bug"


exit_criteria_met: false
initiative_id: NULL
---

# Docs: first-deployment.md deletion-marker example posts '# deletion', which the broker rejects with 400 (null YAML document)

## Objective

Fix the deletion-marker example in `docs/src/tutorials/first-deployment.md`. It posts `yaml_content: "# deletion"` and claims a placeholder comment is optional/harmless, but a comment-only body parses to a null YAML document and the broker rejects it with 400 `invalid_deployment_object` ("YAML content has no documents") — the empty-body exemption applies only to a truly empty string. A tutorial reader's final step fails.

Blocker-severity finding from the 2026-07-27 full-tree review (`docs/REVIEW-2026-07-27.md`, search "deletion"); split out of the review sweep because it was not covered by BROKKR-T-0284's activation-focused edits to the same file.

## Backlog Item Details

### Type
- [x] Bug - Production issue that needs fixing

### Priority
- [x] P1 - High (last step of the flagship tutorial fails as written)

## Acceptance Criteria

- [ ] The example uses a payload the broker accepts (empty string `yaml_content` with `is_deletion_marker: true`, per the validation code), and the prose no longer claims comment-only bodies are acceptable.
- [ ] Example verified against the deployment-object validation path in the broker (and ideally exercised against a dev stack).
- [ ] Grep other docs for the same `"# deletion"` pattern and fix any copies.

## Status Updates

*To be added during implementation*
