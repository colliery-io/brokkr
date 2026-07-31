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
  - "#phase/completed"
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

**2026-07-27 — FIXED** on branch `docs/tenancy-review-2026-07`.

Added a private `fail_delivery_undecryptable(dal, delivery, reason)` helper in `api/v1/webhooks.rs` that calls `mark_failed(delivery.id, reason, 0)` — `max_retries = 0` sends the row straight to `dead`, clears `acquired_by`/`acquired_until`, and sets `completed_at`, mirroring the broker path. Errors are logged rather than propagated so one poisoned delivery cannot fail the whole poll for an agent's other deliveries.

1. URL-decrypt failure now fails the delivery before `continue`, ending the claimed → TTL-expired → reclaimed cycle.
2. Auth-header decrypt failure no longer degrades to `None`; the delivery fails instead, so a subscription configured with an `Authorization` header is never dispatched without it.

`last_error` distinguishes the two ("Failed to decrypt URL: …" vs "Failed to decrypt auth header: …"). A `NOTE:` comment marks where `brokkr_webhook_decrypt_failures_total` attaches under BROKKR-T-0288. No DAL change was needed.

Tests: `test_pending_agent_webhooks_undecryptable_url_marks_delivery_dead` and `test_pending_agent_webhooks_undecryptable_auth_header_never_delivers`. Undecryptable rows are produced by encrypting with a foreign `EncryptionKey`, not by restarting the process. The URL test polls twice and asserts the row stays `dead` with `attempts == 1`, proving the loop is over. Clippy clean; integration target compiles.

**Corrections to this ticket found during implementation:**
- The claim that the broker path marks such deliveries dead "with an audit row (`background_tasks.rs:356-370`)" is **wrong** — those lines contain no audit call; the broker's decrypt-failure path is a bare `let _ = mark_failed(...)`. Audit rows there only appear on the HTTP delivery-attempt failure path. The agent path now emits one anyway (matching `report_delivery_result`'s idiom), making it slightly *more* observable than the broker path it was meant to mirror.
- Defect 2 is agent-path-only; the broker path already handles auth-header decrypt failure correctly (`background_tasks.rs:372-392`). No broker-side change was needed.
- Testing gotcha worth knowing: pre-existing tests store **plaintext** in `url_encrypted`, which does not fail decryption — a leading byte that is not `0x00`/`0x01` falls through to a legacy-XOR path that never errors and returns garbage.

**Follow-up filed:** the same unbounded reclaim loop exists in the `Ok(None)` "subscription not found" arm and the subscription-fetch `Err` arm of the same function — both `continue` after the claim with `attempts` pinned at 0. Left alone as out of scope here; now a two-line change given the helper exists. See BROKKR-T-0304.
