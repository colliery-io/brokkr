---
id: agent-webhook-path-decryption
level: task
title: "Agent webhook path: decryption failure loops forever and a failed auth-header decrypt delivers the webhook unauthenticated"
short_code: "BROKKR-T-0302"
created_at: 2026-07-27T19:16:41.024793+00:00
updated_at: 2026-07-27T19:16:41.024793+00:00
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

# Agent webhook path: decryption failure loops forever and a failed auth-header decrypt delivers the webhook unauthenticated

## Objective

Fix two defects in `get_pending_agent_webhooks` (`crates/brokkr-broker/src/api/v1/webhooks.rs:824-843`), both triggered whenever a stored webhook subscription can no longer be decrypted (the common cause being a restart with a changed or regenerated `BROKKR__BROKER__WEBHOOK_ENCRYPTION_KEY` — see BROKKR-T-0288):

1. **Infinite reclaim loop.** A URL decryption failure `continue`s *after* the delivery has already been claimed by `claim_for_agent` (`:802`). The row then sits `acquired` until its 60s TTL lapses, is released by `release_expired` (`utils/background_tasks.rs:286`), is re-claimed on the next poll, and fails again — indefinitely. `attempts` never increments, so it never reaches `dead`. Contrast the broker delivery path, which marks such a delivery `dead` on first touch with an audit row (`background_tasks.rs:356-370`).
2. **Silent unauthenticated delivery.** An auth-header decryption failure is swallowed to `None` (`:837-843`), so the request is dispatched to the subscriber's endpoint **without the configured `Authorization` header**. A receiver that authenticates will reject it; a receiver that does not will accept an unauthenticated webhook that was configured to be authenticated. Either way the failure is invisible.

Found 2026-07-27 during the BROKKR-T-0288 investigation; filed separately because these are behavioral defects independent of the webhook docs-vs-code decisions.

## Backlog Item Details

### Type
- [x] Bug - Production issue that needs fixing

### Priority
- [x] P1 - High (silent auth downgrade on an outbound integration, plus an unbounded retry loop)

### Impact Assessment
- **Affected Users**: any deployment whose webhook encryption key changed — including every deployment that never set one, since the broker generates a random key per process (`utils/encryption.rs:232-245`).
- **Reproduction**: create a subscription with an auth header; restart the broker without a persisted key; observe agent-path deliveries cycling claimed→expired→claimed with `attempts` pinned at 0, and requests arriving at the endpoint with no `Authorization` header.

## Acceptance Criteria

- [ ] A URL decryption failure on the agent path marks the delivery failed/dead (mirroring the broker path) instead of `continue`, ending the reclaim loop.
- [ ] An auth-header decryption failure never results in a delivery being sent — the delivery fails with a distinct, logged reason.
- [ ] Both failures are attributable to a decryption problem rather than a generic delivery error (distinct `last_error` text; ideally the `brokkr_webhook_decrypt_failures_total` metric proposed in BROKKR-T-0288).
- [ ] Integration coverage for both paths with an undecryptable subscription.

## Status Updates

*To be added during implementation*
