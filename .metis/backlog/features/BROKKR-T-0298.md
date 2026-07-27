---
id: broker-refuse-or-warn-loudly-when
level: task
title: "Broker: refuse or warn loudly when serving with the publicly-known default admin PAK hash"
short_code: "BROKKR-T-0298"
created_at: 2026-07-27T17:51:04.281203+00:00
updated_at: 2026-07-27T17:51:04.281203+00:00
parent: 
blocked_by: []
archived: false

tags:
  - "#task"
  - "#phase/backlog"
  - "#feature"


exit_criteria_met: false
initiative_id: NULL
---

# Broker: refuse or warn loudly when serving with the publicly-known default admin PAK hash

## Objective

Make the publicly-known development admin credential impossible to run in production by accident. `crates/brokkr-utils/default.toml` ships `broker.pak_hash` for a dev PAK whose raw value appears in a source comment; any broker started without overriding `BROKKR__BROKER__PAK_HASH` (or the chart's `broker.pakHash` / `broker.pakHashExistingSecret`) accepts that public credential as admin.

BROKKR-T-0286 closed the documentation side — no documented install path now leaves the default active, and the hardening checklist leads with replacing it. This ticket is the defense-in-depth follow-up deferred from that work: docs prevent the mistake, code should detect it.

## Backlog Item Details

### Type
- [x] Feature - New functionality or enhancement (security hardening)

### Priority
- [x] P2 - Medium (docs now steer users away from the default; this is the backstop for the operator who skipped them)

### Business Justification
- **User Value**: a standalone-consumer install cannot silently run with a credential anyone can read from the public repository.
- **Effort Estimate**: S

### Design questions to settle
- **Refuse vs warn.** Refusing to serve is the strong guarantee but breaks the zero-config dev/demo loop (`angreal local up`, the docker-compose harness, integration tests) that deliberately relies on the default. Options: refuse unless an explicit dev opt-in is present; refuse only in release builds; or always start but log a repeated, unmissable warning and expose it as a status/health field the console can surface.
- **Detection.** Compare the effective configured hash against the known default constant at startup (after config layering, inside `serve`) — cheap and exact, no heuristics.
- Whichever path is chosen, the dev harness must keep working with no per-developer setup.

## DECISION (2026-07-27): warn loudly, do not refuse

Refusing to serve would require a dev opt-out flag (`angreal local up`, the compose harness, and the integration suites all rely on the default), and every such flag eventually appears in someone's production manifest — trading one footgun for a worse one. Instead: an unmissable, repeated startup warning plus a status field the console can surface. The documentation side (BROKKR-T-0286) already steers operators away from the default, so this is a genuine backstop rather than the primary control.

Note the deliberate contrast with the webhook encryption key (BROKKR-T-0288), where refusing to boot **is** the decision: there, an unset key with existing subscriptions is unambiguously broken and there is no legitimate dev workflow that depends on it.

## Acceptance Criteria

- [x] Decision recorded here (refuse-with-opt-out vs warn-loudly) with rationale.
- [ ] Broker detects the shipped default admin hash at startup and acts on it per that decision.
- [ ] Dev/demo/test harnesses (`angreal local up`, integration and e2e suites) still run with no extra configuration.
- [ ] Behavior documented in `how-to/security-hardening.md` alongside the existing "Replace the Default Admin PAK" item.
- [ ] Integration test covers both the default-hash path and an overridden-hash path.

## Status Updates

*To be added during implementation*
