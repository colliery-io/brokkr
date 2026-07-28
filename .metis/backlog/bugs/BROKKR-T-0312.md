---
id: brokkr-register-help-and-the-rust
level: task
title: "brokkr register --help and the Rust wrapper rustdoc claim duplicate registration is a no-op; the API returns 409"
short_code: "BROKKR-T-0312"
created_at: 2026-07-28T18:12:15.997317+00:00
updated_at: 2026-07-28T18:12:15.997317+00:00
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

# brokkr register --help and the Rust wrapper rustdoc claim duplicate registration is a no-op; the API returns 409

## Objective

Correct two in-code documentation strings that contradict the endpoint they describe.

`brokkr register --help` states that registering an already-registered agent/generator pair is a no-op, and the Rust client wrapper's rustdoc says it returns the existing registration. Neither is true: `POST /generators/{id}/register` returns **409 `already_registered`** for a duplicate, and the CLI exits non-zero.

This matters more than a wording slip because the CLI help is what a scripting user reads before writing a bootstrap loop. "No-op" invites `register` in an idempotent provisioning script that will then fail on the second run.

Found 2026-07-28 during the BROKKR-T-0295 documentation sweep. The prose docs (`how-to/agent-registration.md`) are **correct** and now state the 409 explicitly, including the contrast with agent *startup* self-registration, which does treat 409 as success. So the published docs and the tool's own help now disagree — fix the code strings, not the prose.

## Backlog Item Details

### Type
- [x] Bug - Production issue that needs fixing (incorrect in-code documentation)

### Priority
- [x] P2 - Medium (misleads anyone scripting against the CLI; no runtime misbehavior)

## Acceptance Criteria

- [ ] `brokkr register --help` describes the real behavior: duplicate registration returns 409 and exits non-zero.
- [ ] The Rust client wrapper's rustdoc for the register call matches, and mentions that agent startup self-registration deliberately treats 409 as success.
- [ ] Grep the CLI and client crates for other help/rustdoc strings asserting idempotency that the API does not provide — this is unlikely to be the only one.
- [ ] Consider whether an `--if-not-exists` flag would serve the scripting use case that "no-op" was presumably written for; if so, file it separately rather than widening this fix.

## Status Updates

*To be added during implementation*
