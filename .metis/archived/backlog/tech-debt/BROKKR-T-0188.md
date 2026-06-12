---
id: pak-verification-is-not-constant
level: task
title: "PAK verification is not constant-time"
short_code: "BROKKR-T-0188"
created_at: 2026-06-10T03:04:03.509945+00:00
updated_at: 2026-06-10T04:58:17.092467+00:00
parent: 
blocked_by: []
archived: true

tags:
  - "#task"
  - "#tech-debt"
  - "#phase/completed"


exit_criteria_met: false
initiative_id: NULL
---

# PAK verification is not constant-time

## Objective

PAK verification compares hashes with plain `String ==` (`crates/brokkr-broker/src/utils/pak.rs:105-110`), not a constant-time comparison. Practical exposure is limited (hashes, not raw secrets, are compared, and lookup is by indexed hash), but a constant-time compare (e.g. the `subtle` crate) removes the timing side-channel class entirely and lets the security docs make the claim they previously made incorrectly.

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] Hash comparison in PAK verification is constant-time
- [ ] Unit tests pass; no measurable hot-path regression (auth is cached, default TTL 60s)
- [ ] `docs/src/explanation/security-model.md` updated to state the property once true

## Status Updates

- 2026-06-09: Found during /docs-diataxis accuracy review (docs claimed constant-time; claim removed from docs).
- 2026-06-09: IMPLEMENTED (uncommitted, unit tests green): `verify_pak` uses `subtle::ConstantTimeEq` on the hash bytes; security-model.md updated.