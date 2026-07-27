---
id: webhooks-docs-promise-filtering
level: task
title: "Webhooks: docs promise filtering, 4xx retry classification, hot-reloadable batch size, and timeout control that code doesn't implement; unset encryption key breaks all deliveries on restart"
short_code: "BROKKR-T-0288"
created_at: 2026-07-27T14:27:49.818680+00:00
updated_at: 2026-07-27T14:27:49.818680+00:00
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

# Webhooks: docs promise filtering, 4xx retry classification, hot-reloadable batch size, and timeout control that code doesn't implement; unset encryption key breaks all deliveries on restart

## Objective

Reconcile the webhook docs (`how-to/webhooks.md`, `reference/webhooks.md`) with actual delivery behavior. Findings (2026-07-27 review; details in `docs/REVIEW-2026-07-27.md`, search "webhook").

> **Numbering note:** this original list is ordered by severity, while the verification and DECISIONS sections below are ordered **filters (1) → retry classification (2) → batch/interval/timeout (3) → encryption key (4)**. The later numbering is the canonical one — cite it. The mapping from this list is: 1→1, 2→4, 3→2, 4→3.

1. **Blocker — filters are decorative.** Docs instruct creating subscriptions with a `filters` object ({agent_id, stack_id, labels}); the broker stores and echoes it but no code path evaluates filters during delivery — every matching-event-type subscription gets every event. (Verified: `filters` appears only in `api/v1/webhooks.rs`, not in the delivery path.) Decide: implement filtering or remove from docs + deprecate the field.
2. **Major — restart-bricked deliveries.** When `BROKKR__BROKER__WEBHOOK_ENCRYPTION_KEY` is unset the broker generates a random per-process key; after any restart every stored webhook URL/auth header is undecryptable and deliveries fail permanently. reference/webhooks.md never warns.
3. **Major — retry classification fiction.** Docs say 4xx (except 429) are non-retryable; no classification exists — all failures retry to max_retries then dead.
4. **Major — config claims.** `webhook_delivery_batch_size` documented as hot-reloadable but read once at serve startup; docs imply agent batch = broker batch (50) but agents claim at most 10 per poll; documented `timeout_seconds: 60` example has no effect on broker delivery (fixed 30s client).

## 2026-07-27 — verification of all four items

**1. `filters` — CONFIRMED never evaluated.** Stored (`migrations/13_webhooks/up.sql:12`), echoed back for display, and absent from the entire delivery chain: emission → `event_bus.rs:30` → `dal/webhook_subscriptions.rs:100 get_matching_subscriptions` (event-type/wildcard match only) → delivery rows. The one place it would go is `emit_event` (`event_bus.rs:64`), where subscription and event data are both in hand. **Data availability is uneven**: `agent_id` is absent from `stack.*`, `deployment.created/deleted`, `workorder.created`; `stack_id` is absent from `deployment.applied/failed` (needs a deployment_object→stack lookup); **`labels` is in no payload at all** and would cost per-event DAL lookups on the write path, with undefined semantics (the how-to example never says *whose* labels) that also collide with `target_labels`, which is real and already does label routing.
**RECOMMENDATION: implement `agent_id` + `stack_id` only (~100-150 LOC, no schema change, "event lacks the field ⇒ no match"); drop `labels` from the API and docs.**

**2. Retry classification — CONFIRMED absent, and the ticket understates the impact.** Any non-2xx becomes a flat `Err(String)`; dead is decided purely by attempt count (`webhook_deliveries.rs:316-329`). A 404 endpoint burns 5 attempts. **Agents already transmit `status_code` and the broker discards it** (`api/v1/webhooks.rs:946-949`) — so classification is cheaper than filed. Sensible policy: retryable = transport/timeout/408/425/429/5xx; everything else 4xx ⇒ dead on first attempt with the status in `last_error`. ~60-80 LOC, zero agent changes. Also note `max_retries` is a total-*attempt* cap (default 5 = 5 attempts, 4 retries) but is documented as "retry attempts".
**RECOMMENDATION: implement.**

**3. Batch size / interval / timeout — CONFIRMED, with one ticket claim overstated.** Batch size and interval are captured into the spawned worker at `cli/commands.rs:160-164` and frozen; `ReloadableConfig` is built *after* the tasks start and its accessors have zero non-test callers (same root cause as BROKKR-T-0292). Agent claim cap is hard-coded 10 (`api/v1/webhooks.rs:802`) against a 10s poll, not 50. **Overstated:** `timeout_seconds` IS honored by agent delivery (`brokkr-agent/src/webhooks.rs:213`) and by `POST /webhooks/{id}/test` — only the broker worker ignores it (fixed 30s client, `background_tasks.rs:274-277`). Third wrong doc surface the ticket misses: the chart annotates these values `@hot-reload: true` (`configmap.yaml:46`, `values.yaml:12-14`).
**RECOMMENDATION: honor `timeout_seconds` per-delivery in the broker (~3 LOC, mirrors `test_webhook`, removes a real broker-vs-agent inconsistency); demote batch size/interval to restart-only in code, docs, and chart comments rather than building a hot-reload path the chart cannot drive.**

**4. Encryption key — CONFIRMED, and consequences are worse than filed.** Random per-process key when unset (`encryption.rs:232-245`). Two encrypted columns only (`url_encrypted`, `auth_header_encrypted`), so after a restart with a new key subscriptions still *list* fine but can never deliver and can only be repaired by PUT-with-fresh-URL or recreation. Failure handling differs by path:
- Broker path: marked `dead` on first touch with an audit row — loud enough, though it reads as a delivery failure rather than a config failure.
- **Agent path is broken twice over** (`api/v1/webhooks.rs:824-843`): a URL decryption failure `continue`s *after the delivery was already claimed*, so the row cycles claimed → TTL expiry → reclaimed → fails, **forever, with `attempts` never incrementing and no path to `dead`**; and an auth-header decryption failure is swallowed to `None`, so **the webhook is delivered unauthenticated**. Filed separately as BROKKR-T-0302.

No webhook Prometheus metrics exist at all, and `reference/webhooks.md` never mentions the key.
**RECOMMENDATION: refuse to start when the key is unset AND subscriptions already exist (unambiguous misconfiguration); for a wrong-but-set key, fail deliveries loudly with a `brokkr_webhook_decrypt_failures_total` metric rather than taking the broker down. Document the trap and recovery.**

**Bonus defect, same class as `filters`:** `CreateWebhookRequest.validate` (`api/v1/webhooks.rs:50`) is accepted and documented as "Send test request on creation" (`reference/webhooks.md:126`) but **never read** by `create_webhook`. Fold into whichever decision is taken on item 1.

## DECISIONS (Dylan, 2026-07-27)

1. **`filters`** — implement `agent_id` + `stack_id` in `emit_event`; **drop `labels`** from the API and docs (`target_labels` already does label routing). Rule: an event that carries no such field does not match. Same pass removes the unread `validate` field from the API and docs.
   - **AMENDED (Dylan, 2026-07-27): legacy `filters.labels` and `validate` are REJECTED WITH 422, not ignored-and-dropped.** The first implementation chose ignore-and-drop to avoid breaking stored manifests and idempotent IaC re-applies. Overridden: silently dropping a field the caller believes is scoping their deliveries is worse than failing, because the caller walks away thinking their webhook is filtered when it is not. This is a deliberate breaking API change for clients still sending those fields, and the docs must say so with the migration (`target_labels`; `POST /webhooks/{id}/test`).
   - **Rejection is write-path only.** Existing stored rows containing a legacy `labels` key must keep deserializing and keep delivering — no `deny_unknown_fields` on the stored-model read path. The read/write asymmetry is the crux; `test_legacy_stored_labels_filter_still_delivers` guards it.
2. **Retry classification** — implement. Retryable: transport errors, timeouts, 408, 425, 429, all 5xx. Non-retryable: every other 4xx ⇒ `dead` on first attempt with the status recorded in `last_error`. Use the `status_code` agents already transmit and the broker currently discards.
3. **Batch size / interval / timeout** — honor `timeout_seconds` per-delivery in the broker worker (mirrors `test_webhook`; removes a real broker-vs-agent inconsistency). Batch size and interval become restart-only **for now**; note this interacts with BROKKR-T-0292, where the decision is to make hot reload genuinely work — once that lands these can be revisited, so do NOT strip them from `DynamicConfig`, and fix the chart's misleading `@hot-reload: true` annotations to match whatever is true at ship time.
4. **Encryption key** — refuse to start when the key is unset **and** `webhook_subscriptions` is non-empty (unambiguous misconfiguration; error must name `BROKKR__BROKER__WEBHOOK_ENCRYPTION_KEY`). A wrong-but-set key fails deliveries loudly with a `brokkr_webhook_decrypt_failures_total` metric rather than blocking boot. Document the trap and the recovery path. The silent agent-path failures are BROKKR-T-0302.

## Implementation progress

**2026-07-27 — item 4 (encryption key) DONE** on branch `docs/tenancy-review-2026-07`.

`utils/encryption.rs` gained `check_webhook_encryption_key(key_hex, existing_subscriptions)`, returning an error only when the key is unset **and** at least one subscription row exists. The message names `BROKKR__BROKER__WEBHOOK_ENCRYPTION_KEY`, states the consequence (undecryptable URLs and auth headers; subscriptions keep listing as healthy while never delivering), and gives both recovery paths. A set-but-wrong key deliberately passes — it fails loudly at delivery time instead of taking the broker down for a webhook-only fault.

A `configured_key` normalizer was extracted so the guard and `init_encryption_key` share one definition of "unset" (`None` or `Some("")`) and cannot drift — worth preserving in review; two independent definitions of unset is exactly how this class of bug returns.

Placement in `serve` is load-bearing: after `run_pending_migrations` (the table must exist), after `conn`, and **before `init_encryption_key`** — otherwise the random key is already installed in the `OnceCell` and the process is committed to the broken state. It sits before background tasks and the listener bind, so a refusal starts nothing. Fresh installs (zero rows) fall through to the existing warning, which was strengthened to say subscriptions created now become permanently undeliverable after the next restart.

Tests: four unit tests in `encryption.rs` on the extracted helper (unset + rows blocks for both `None` and `Some("")`; zero rows allows; any set key allows even at 100 rows; `configured_key` semantics pinned against `init_encryption_key`). The `serve` path itself is **not** testable in this harness — it binds a port, spawns seven tasks, and `init_encryption_key` writes a process-global `OnceCell` that cannot be re-run inside the shared integration binary — so no integration test was contrived.

**Correction to this ticket's own framing:** `tests/integration/db.rs` does not exist; it is a directory (`db/mod.rs`, `db/multi_tenant.rs`), and neither file exercises `serve`.

**2026-07-27 — items 2 and 3 plus the metric DONE.**

*Item 2 (retry classification):* `is_retryable_status(u16)` in `dal/webhook_deliveries.rs` is the single policy point both delivery paths call — retryable covers 408/425/429, all 5xx, and anything not 4xx/5xx (an unfollowed 3xx keeps prior behavior); every other 4xx is terminal. Added `mark_dead(id, error)` beside `mark_failed`, sharing a private `write_dead` so both release the claim and set `completed_at` identically. `mark_failed`'s signature is unchanged, so the `max_retries = 0` force-dead idiom that BROKKR-T-0302 relies on still works. Broker path: `attempt_delivery` now returns a typed `DeliveryFailure { message, status, retryable }` routed to `mark_dead` vs `mark_failed`. Agent path: `report_delivery_result` consumes the `status_code` agents already transmit (no status ⇒ transport failure ⇒ retryable), and `delivery_failure_message` guarantees the status reaches `last_error` even when the agent sends no body. Status and retryable are now in the audit payload. **Zero agent-side changes were required — confirmed.**

*Item 3 (timeout):* `attempt_delivery` takes a per-request timeout of `subscription.timeout_seconds.max(1)` applied via `RequestBuilder::timeout`. The worker's client-wide 30s timeout was **removed**, deliberately: a client-level cap would silently override any subscription configured above it. **Behavior change to flag at release:** subscriptions configured above 30s now genuinely get up to their configured ceiling, which increases worst-case worker occupancy per delivery. Batch size, interval, and `DynamicConfig` untouched, per the T-0292 interaction.

*Metric:* `brokkr_webhook_decrypt_failures_total` (`IntCounterVec`, labels `field` ∈ {url, auth_header} × `path` ∈ {broker, agent}) with a `record_webhook_decrypt_failure` accessor; wired into both the broker decrypt sites and BROKKR-T-0302's helper, which also gained a `field` parameter and now uses `mark_dead`.

*Incidental fix:* `metrics.rs` had a pre-existing flake — `test_init_registers_all_metrics`, `test_set_active_agents`, and `test_set_stacks_total` race on the process-global registry. Now serialized behind a shared lock (reproduced ~1-in-3 before, 5/5 clean after).

**Second correction to this ticket:** item 4 above claimed the broker decrypt path marks deliveries dead "with an audit row — loud enough". It does **not**; there is no `audit::log_action` on that branch (audit rows exist only on the delivery-attempt failure branch). The agent path does log one, so the two paths remain asymmetric. The new metric now covers the broker side. **Open question worth a deliberate decision:** should the broker decrypt path emit an audit row for parity?

**2026-07-27 — ALL FOUR ITEMS COMPLETE; ticket closed.** Items 2, 3, and the metric plus item 4 are described above. Item 1 (filters) shipped with `agent_id`/`stack_id` evaluation in `emit_event`, `labels` and `validate` removed, and — per Dylan's amendment — both rejected with a genuine 422 (`ApiError::unprocessable`, the file's existing idiom for semantic validation) carrying `code: "unsupported_field"` and `details: {field, use_instead}`. Detection reads the raw JSON body before typed deserialization, deliberately: `deny_unknown_fields` would surface as axum's plain-text serde error and could not name the replacement, and modelling the removed keys as typed fields would resurrect them in the generated OpenAPI schema and SDKs. Read-path tolerance is preserved and guarded from both sides.

Spec, in-crate mirror, and both SDKs regenerated (`angreal openapi check{,-python,-typescript}` all pass). Docs rewritten with a per-event-type filter table derived by reading every emission site, which also corrected several wrong payload field lists. Three chart `@hot-reload` annotations corrected. Full integration suite: **510 passed, 0 failed**.

Behavior changes recorded for the release notes: subscriptions with `timeout_seconds` above 30 now genuinely get their configured ceiling; a wrongly-shaped request body now returns the API's JSON error shape instead of axum's plain text (status unchanged); `PUT` validates the body before the existence lookup, so an unknown id with a removed field returns 422 rather than 404. `validate` is rejected on create only — it was never accepted on `PUT`, where it remains an ordinary unknown key.

Commits: `3fa574e` (code), `d72a82e` (spec + SDKs), `b9a1303` (docs + chart).

## Backlog Item Details

### Type
- [x] Bug - Production issue that needs fixing (docs promise behavior code doesn't have; one operational trap)

### Priority
- [x] P1 - High

## Acceptance Criteria

- [ ] Product decision per item (implement vs document reality) recorded here.
- [ ] Docs updated so every stated webhook behavior is code-backed (filters, retries, batch sizes, timeouts).
- [ ] Encryption-key guidance added: set the key explicitly (chart value + existingSecret variant per BROKKR-T-0278); warn about the random-key restart trap; document recovery (recreate subscriptions).

## Status Updates

*To be added during implementation*
