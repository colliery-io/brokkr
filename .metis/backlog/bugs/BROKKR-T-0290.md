---
id: templates-docs-claim-newest
level: task
title: "Templates: docs claim newest-version instantiation and owner-only visibility; code renders the pinned row, drops labels on PUT, and lets any generator instantiate another tenant's template by UUID"
short_code: "BROKKR-T-0290"
created_at: 2026-07-27T14:27:53.380126+00:00
updated_at: 2026-07-27T14:27:53.380126+00:00
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

# Templates: docs claim newest-version instantiation and owner-only visibility; code renders the pinned row, drops labels on PUT, and lets any generator instantiate another tenant's template by UUID

## Objective

Reconcile template documentation with versioning and access reality (2026-07-27 review; `docs/REVIEW-2026-07-27.md`, search "template"):

1. **Blocker — explanation/template-system.md**: "when you instantiate a template, you always get the newest version" is false — instantiation renders the exact row identified by `template_id`; PUT creates a NEW row with a NEW id, so the old id renders the old version forever. (`how-to/templates.md` also never tells the reader the PUT response carries a new id; tutorial and reference get it right — three pages, three stories.)
2. **Major — labels/annotations don't carry over on PUT**: they're attached to the old row; matching restrictions silently vanish from the new version unless re-added. No doc mentions this.
3. **Major, isolation-relevant — cross-tenant instantiation**: docs claim generator templates are visible/usable only by the owning generator and admins, but the instantiate path performs no template read-access check — any generator knowing another tenant's template UUID can instantiate it into its own stack and read the rendered content. Needs a code-vs-docs decision alongside BROKKR-T-0287.

## 2026-07-27 — verification: the cross-tenant instantiate path is fully reachable

Traced the code rather than trusting the review summary. Findings:

- `instantiate_template` (`api/v1/stacks.rs:739`) calls `fetch_owned_stack` for the *target stack*, then fetches the template with a bare `dal.templates().get(...)` and **never calls `check_read_access`**. That helper exists in `api/v1/templates.rs:82` and is applied by `get_template`, `list_labels`, and `list_annotations` — instantiate is the sole path that skips it, which reads as an oversight, not a design choice.
- The label/annotation guard is not a boundary. `template_matches_stack` (`utils/matching.rs:50-57`) returns `matches: true` unconditionally for a template with no labels/annotations. When it does mismatch, the 4xx body enumerates `missing_labels` / `missing_annotations` — so a failed attempt tells the caller exactly which selectors to add to their own stack (which they own) before retrying. It is a two-step oracle, not a lock.
- **UUID secrecy is not a mitigation — the UUID is the primary key** (Dylan, 2026-07-27), present in every API response, error body, and audit row. Concretely: `create_template` audit-logs `resource_id = template.id` plus the template name (`templates.rs:221-229`), and `GET /admin/audit-logs` gates only on `auth.admin` (`admin.rs:360`) — which the ephemeral read-only console UI PAK satisfies. Since the console is served to anyone who can reach the broker port with that token embedded in its HTML, **any party with network access to :3000 can enumerate every template UUID and name across all tenants**; any party who additionally holds a generator PAK can then instantiate any of them into their own stack and read the rendered content.

### CORRECTION (same day, Dylan): severity over-stated above — containment holds

The paragraphs above framed this as a P0-class cross-tenant disclosure path. That was wrong, and the correction matters for anyone triaging this later:

- `instantiate_template` requires `fetch_owned_stack`, so the rendered object lands **only in the caller's own stack**, which carries the caller's `generator_id`. Post-BROKKR-T-0287, delivery requires an agent registered with that generator. **There is no cross-tenant execution path**: a tenant cannot reach another tenant's cluster or agents through this. Agent/generator binding provides the containment.
- UUID publication is therefore not a problem in itself (Dylan's point): knowing an id buys reach only if the binding lets you act on it, and it doesn't.
- **Residual is narrow**: the caller receives the *rendered output* of another tenant's `template_content`, rendered with the caller's own parameters (the victim's parameter values are supplied per-instantiation and are never in the template, so they do not leak). The exposure is the template's Tera source — internal registry paths, hostnames, deployment patterns — i.e. a confidentiality judgement about whether template bodies are sensitive engineering artifacts, not a containment failure.

**Revised assessment: not P0.** This is an API-consistency gap — `instantiate_template` is the only template path that skips `check_read_access`, while `get_template`, `list_labels`, and `list_annotations` all enforce it — carrying a low-severity disclosure. Worth fixing so the asymmetry doesn't mislead the next reader; not worth an emergency.

**DECISION (2026-07-27): add the access check.** `instantiate_template` gets `check_read_access(&auth_payload, &template)?`, matching `get_template`, `list_labels`, and `list_annotations`. Sanctioned cross-tenant sharing remains available through system templates (`generator_id = NULL`), which that helper already permits — no new mechanism needed. Add a cross-tenant integration test in the shape of `tests/integration/api/registration_consent.rs`.

*Original framing, retained:* is cross-tenant template use intended? If yes, system-owned templates (`generator_id = NULL`) already express it and `check_read_access` already permits them, so no new code is needed. If no, add `check_read_access(&auth_payload, &template)?` to `instantiate_template`.

**Keep regardless of that decision:** the mismatch 4xx enumerates `missing_labels`/`missing_annotations`, which is a mild information leak worth trimming. Separately, audit logs being readable by the console's readonly-admin UI PAK (actor names, IP addresses, user agents) deserves its own evaluation on its own merits — it is currently a line item inside BROKKR-T-0295.

## Backlog Item Details

### Type
- [x] Bug - Production issue that needs fixing

### Priority
- [x] P1 - High (silent stale-version deploys; cross-tenant template read)

## Acceptance Criteria

- [ ] template-system.md, how-to/templates.md, reference/templates.md tell one consistent, code-true versioning story (pinned row, new-id-on-PUT, label carry-over behavior).
- [ ] Cross-tenant instantiate: access check added (preferred, with test) or the visibility claim removed and the exposure documented.
- [ ] how-to PUT example captures the new template id and re-applies labels.

## Status Updates

*To be added during implementation*
