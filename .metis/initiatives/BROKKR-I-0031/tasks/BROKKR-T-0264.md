---
id: broker-global-webhook-deliveries
level: task
title: "Broker: global webhook-deliveries feed endpoint"
short_code: "BROKKR-T-0264"
created_at: 2026-06-29T10:45:50.467288+00:00
updated_at: 2026-06-29T10:45:50.467288+00:00
parent: brokkr-operator-console
blocked_by: []
archived: false

tags:
  - "#task"
  - "#phase/todo"


exit_criteria_met: false
initiative_id: BROKKR-I-0031
---

# Broker: global webhook-deliveries feed endpoint

## Parent Initiative

[[BROKKR-I-0031]]

## Objective

Add a broker-wide webhook **delivery feed** so operators can see the latest delivery
attempts across *all* subscriptions at once (incident triage), instead of only
per-subscription. Surfaced as a "Recent deliveries" panel on the Operator Console
Webhooks view.

### Type
- [x] Feature - New functionality or enhancement

### Priority
- [x] P2 - Medium (nice to have)

### Business Justification
- **User Value**: Webhook delivery in Brokkr runs *through agents*, so a fleet-wide
  "which integrations are failing right now" view is real incident-triage signal. The
  current per-subscription modal (`GET /webhooks/:id/deliveries`, wired in I-0031) only
  helps once you already suspect a specific webhook.
- **Effort Estimate**: S–M (one read endpoint + DAL method + OpenAPI/SDK regen).

## Context / current state

The only delivery query today is per-subscription: `GET /api/v1/webhooks/:id/deliveries`
(`list_deliveries`, returns `Vec<WebhookDelivery>`). There is **no** endpoint returning
deliveries across all subscriptions. `get_pending_agent_webhooks` returns an *agent's*
claimed pending deliveries (a work queue), not an operator feed. UI fan-out (fetch
`/webhooks` then N× `/webhooks/:id/deliveries`) is an N+1 that scales with subscription
count — rejected.

## Acceptance Criteria

- [ ] New `GET /api/v1/webhook-deliveries` — global, newest-first, admin-gated; supports
      `?status=` and a bounded `?limit=` (default ~50) with sane pagination.
- [ ] Backed by a `webhook_deliveries().list_recent(...)` DAL method (single query, no N+1).
- [ ] `#[utoipa::path]` annotation + registered in `openapi.rs`; `angreal openapi export`
      + `gen-python`/`gen-typescript` run; all three drift checks green.
- [ ] Operator Console Webhooks view gains a "Recent deliveries" panel (subscription name +
      status + event + attempts + age), pixel-verified via the web-e2e harness.
- [ ] Response excludes/redacts the encrypted URL (consistent with the `has_url` DTO rule).

## Implementation Notes

### Technical Approach
Add the DAL list method (order by `created_at` desc, optional status filter, limit), a
handler mirroring `list_deliveries`'s auth shape (admin PAK), the utoipa annotation, and the
openapi.rs registration. Then regen spec + SDKs (same loop used for the `cluster_name` add).

### Dependencies
None hard. Complements the per-subscription deliveries already wired in I-0031.

## Status Updates

*To be added during implementation.*
