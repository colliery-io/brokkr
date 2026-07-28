---
id: docs-operator-console-is
level: task
title: "Docs: Operator Console is completely undocumented (access, zero-config UI PAK auth, tenant scope selector, read-only limits)"
short_code: "BROKKR-T-0276"
created_at: 2026-07-27T14:13:05.180272+00:00
updated_at: 2026-07-28T15:13:16.251072+00:00
parent: 
blocked_by: []
archived: false

tags:
  - "#task"
  - "#tech-debt"
  - "#phase/active"


exit_criteria_met: false
initiative_id: NULL
---

# Docs: Operator Console is completely undocumented (access, zero-config UI PAK auth, tenant scope selector, read-only limits)

## Objective

Document the Operator Console (shipped PR #82 / BROKKR-I-0031, extended PR #87 / BROKKR-I-0032) in the mdbook. As of 2026-07-27, `grep -ri console docs/src` matches only unrelated contexts (log-streaming, TypeScript SDK) and there are zero mentions of the UI PAK, `brokkr-ui-token`, or zero-config auth anywhere in `docs/src` — a consumer cannot discover the console exists.

## Backlog Item Details

### Type
- [x] Tech Debt - Code improvement or refactoring (documentation debt)

### Priority
- [x] P1 - High (important for user experience)

### Technical Debt Impact
- **Current Problems**: PRs #82 and #87 shipped with no `docs/src` changes. New tenants standing up standalone Brokkr have no way to learn: that browsing `http://<broker>:3000/` serves a console; that auth is zero-config (ephemeral read-only admin UI PAK minted per process, injected as `<meta name="brokkr-ui-token">`, `crates/brokkr-broker/src/utils/ui_pak.rs` + `api/assets.rs:86-126`); that network access to the console URL IS the auth boundary; that multi-replica deployments need sticky sessions; that the console is read-only (GET/HEAD + two allowlisted POSTs, `api/v1/middleware.rs:160-177`); that the tenant scope selector exists and is a view filter, not isolation; that the console requires the `embed-ui` build feature (official images have it; bare `cargo build` serves a placeholder).
- **Benefits of Fixing**: Consumers can self-onboard and correctly reason about the console's security posture instead of assuming it does more (writes, isolation) or less (exists at all) than it does.
- **Risk Assessment**: The zero-config token model is easy to misjudge from the outside; undocumented, it invites both accidental exposure (operators not realizing network access = read access) and false bug reports (mutations failing with 403).

### Content to write (suggested placement)
- New how-to or getting-started page "Operator Console" + SUMMARY.md entry: reaching it, what each of the 7 views shows, tenant scope selector (populated from `GET /api/v1/paks`, hidden on single-tenant installs, `?pak_id=` = view filter only), Live/Paused toggle currently inert.
- Security-model/explanation update: the UI PAK credential class (`admin: true, readonly: true`), its allowlisted POSTs, network-as-auth-boundary rationale, sticky-sessions caveat.
- Update `explanation/components.md` + architecture pages to include `brokkr-web`; distinguish from the unsupported `examples/ui-slim` React demo (which still claims write features and manual PAK entry).
- Do NOT document the "Run diagnostic" button as working until BROKKR-T-0275 is fixed.

### Carried over from BROKKR-T-0280 (closed 2026-07-27)
security-model.md now has a "Read-Only Console Authentication (the UI PAK)" section describing the credential, its readonly allowlist, the network-as-auth-boundary design, and mitigations. When the console page lands, cross-link the two (page → security model for the threat model; security model → page for operational use) and make sure neither duplicates nor contradicts the other. That coordination was T-0280's last open criterion.

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] A console page exists in docs/src, linked from SUMMARY.md, covering access, auth model, read-only limits, tenant scope selector, embed-ui build caveat, and multi-replica sticky-session note.
- [ ] security-model.md describes the ephemeral UI PAK credential class and its boundary.
- [ ] components/architecture explanation pages include brokkr-web and disambiguate examples/ui-slim.
- [ ] No page claims console write capabilities beyond what actually works.

## Status Updates

*To be added during implementation*