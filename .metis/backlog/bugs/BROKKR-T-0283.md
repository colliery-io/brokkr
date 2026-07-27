---
id: docs-pak-expiry-phrasing-implies-a
level: task
title: "Docs: PAK 'expiry' phrasing implies a TTL that doesn't exist (generators.md, installation.md, agent chart README)"
short_code: "BROKKR-T-0283"
created_at: 2026-07-27T14:19:56.852706+00:00
updated_at: 2026-07-27T14:19:56.852706+00:00
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

# Docs: PAK 'expiry' phrasing implies a TTL that doesn't exist (generators.md, installation.md, agent chart README)

## Objective

Remove the implication that PAKs expire. Verification is a pure hash lookup/compare with no timestamp or TTL on the credential (`crates/brokkr-broker/src/api/v1/middleware.rs:192-309`, `utils/pak.rs:106-126`); PAKs are valid until rotated or their entity is soft-deleted. Three troubleshooting passages say otherwise:

- `docs/src/how-to/generators.md` ~line 318: "Verify the PAK is correct and not expired".
- `docs/src/getting-started/installation.md` ~line 406: same phrase.
- `charts/brokkr-agent/README.md` ~line 323: "Ensure PAK has not expired".

Consumers debugging a 401 will chase a nonexistent expiry mechanism instead of the real causes (rotation elsewhere, entity soft-deleted, wrong credential class). Found in the 2026-07-27 auth-drift sweep.

## Backlog Item Details

### Type
- [x] Bug - Production issue that needs fixing (misleading troubleshooting guidance)

### Priority
- [x] P2 - Medium

## Acceptance Criteria

- [ ] All three passages replaced with "rotated or revoked (entity deleted)" phrasing.
- [ ] A grep for `expir` across docs/src and chart READMEs shows no remaining implication that PAKs carry a TTL (auth-cache TTL wording, which is real, stays).

## Status Updates

*To be added during implementation*
