---
id: docs-drift-sweep-remaining-major
level: task
title: "Docs drift sweep: remaining major/minor findings from the 2026-07-27 full-tree review (see docs/REVIEW-2026-07-27.md)"
short_code: "BROKKR-T-0295"
created_at: 2026-07-27T14:28:02.502537+00:00
updated_at: 2026-07-28T15:17:49.098960+00:00
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

# Docs drift sweep: remaining major/minor findings from the 2026-07-27 full-tree review (see docs/REVIEW-2026-07-27.md)

## Objective

Work through the remaining findings of the 2026-07-27 full-tree review (25-agent Diátaxis workflow + auth-drift sweep) that are not covered by the dedicated tickets BROKKR-T-0275..0294. The complete itemized list with evidence and suggested fixes lives in `docs/REVIEW-2026-07-27.md` (230 deduped findings: 23 blocker / 100 major / 107 minor; all blockers and the largest major clusters are already ticketed).

Representative remaining majors (not exhaustive — the artifact is the checklist):

- **Reference drift**: audit-logs.md (best-effort writer with drop-on-overflow undocumented; missing `agent.registered`/`agent.deregistered` actions; missing `actor_name` field; "only admin PAKs can query" predates the UI PAK), error-codes.md (five live codes missing; bodyless readonly-403 unmentioned), fleet.md (missing `cluster_name` field), container-images.md (`{branch}-{short-sha}` tags never actually pushed; cargo-chef claim false), generators.md (Update example fails as written).
- **Landing/index pages**: reference/README.md links 2 of 21 pages; how-to/README.md omits agent-registration and fleet-monitoring; publishing-strategy.md says OCI chart publishing is "planned" though release.yml ships it.
- **SDK docs**: rust.md retry example does not compile (borrow in `FnMut(&Client)`); regeneration.md names the wrong spec file (in-crate mirror `crates/brokkr-client/spec/brokkr-v1.json` is what progenitor reads) and omits the mirror from the commit list.
- **Misc wrong claims**: managing-stacks.md (names are globally unique → 409, not "nothing enforced"); monitoring-setup.md readyz text (overlaps T-0291); webhooks how-to timeout example (overlaps T-0288); log-streaming framing must stay immediate-ops-only (6h ceiling).
- **Diátaxis/misfiled**: helm values reference tables living inside installation.md (only chart-values docs in the book — belongs in reference/), plus the lane findings tagged `misfiled` in the artifact.
- **All 107 minors** (terminology, stale ports/versions, broken index links, phrasing).
- **Carried in 2026-07-27**: document the new `brokkr-broker create agent --generator-ids` flag in `reference/cli.md` (added by BROKKR-T-0289); confirm the CLI/API registration parity statements on that page now read true.

## Backlog Item Details

### Type
- [x] Tech Debt - Code improvement or refactoring (documentation debt)

### Priority
- [x] P2 - Medium (blockers and top majors are ticketed separately)

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] Every finding in docs/REVIEW-2026-07-27.md is either fixed, covered by another ticket, or explicitly waived (annotate the artifact inline).
- [ ] Fixes verified against code citations rather than trusting the reviewer text.
- [ ] `angreal docs build` passes; SUMMARY/index pages link-checked.

## Status Updates

**2026-07-28 — PASS 1 COMPLETE** (11 files) on branch `docs/tenancy-review-2026-07`. `angreal docs build` passes. Scoped deliberately to files no concurrent agent owned; a second pass covers the rest.

Files done: `reference/{audit-logs,error-codes,fleet,container-images,README,cli}.md`, `how-to/{README,managing-stacks}.md`, `how-to/sdks/{rust,regeneration}.md`, `explanation/publishing-strategy.md`.

**Three findings in the review artifact were WRONG or stale — recorded so nobody re-applies them:**
1. **audit-logs "drops on overflow" is wrong for the live path.** `AuditLogger::log` spawns and `send().await`s, so a full queue **back-pressures rather than discards**. `try_log` does drop, but nothing outside `audit.rs` calls it. The real loss modes are a failed batch insert being discarded without retry, and queued entries dying on an abrupt stop — those are what got documented.
2. **cli.md's "create agent performs no registrations" is stale** — BROKKR-T-0289 fixed it, so the registration-parity statements on that page now read true.
3. **how-to/README's "multi-tenant guide is really schema-per-tenant" is stale** — BROKKR-T-0277 rewrote it around generators-as-tenants. Only the title mismatch was real.

Several cli.md findings were also already fixed by earlier tickets in this loop (the serve endpoint table already lists the console and `/docs/openapi.json`; the generate-pak hash is already bare hex) and were correctly left alone.

**Two error codes the review missed** were added: `unsupported_field` (422, with `details.field`/`details.use_instead`) and `invalid_request_body` (422). The first belongs in the catalog because `reference/webhooks.md` already shows it in a response body while the catalog omitted it.

**The rust.md retry fix was verified by compiling both versions**, not by inspection: the old example fails with `lifetime may not live long enough`; the replacement compiles clean against `brokkr-client` 0.8.4.

Also corrected in passing: audit-log access control (the read-only console token satisfies the admin gate, so network reachability of the broker port is the real boundary protecting IP/user-agent data); the bodyless-response list in error-codes (three cases, not one); container-images (commit-SHA tags are never actually pushed; the cargo-chef claim was false); publishing-strategy (OCI chart publishing already ships).

**2026-07-28 — PASS 2 COMPLETE.** Three parallel sweeps over reference (13 files), how-to (12 changed), and explanation/getting-started (13). `angreal docs build` passes.

**Roughly twenty findings across both passes were already fixed** by earlier commits on this branch and were correctly left alone — all four template findings, five of six webhook findings, three of four diagnostics findings, the `/readyz` and pod-attribution items, `core-concepts`' PAK-class count, `data-flows`' auth-flow and retention tables, and the multi-tenancy "no tenant management API" claim. Two artifact findings are now **stale in the opposite direction**: `template-system.md`'s "instantiate skips read access" and `generators.md`'s "auto-registers every agent" are both true-as-written now that the code was fixed.

**New problem found that the artifact missed:** `environment-variables.md` advertised a list of hot-reloadable settings and claimed log level applies without a restart. **Nothing consumes the reloaded values** — the log filter, CORS layer, delivery worker and cleanup tasks all capture at startup, and `update_log_level` has no non-test callers. Rewritten as detection-only, which also resolves a contradiction with `webhooks.md`. This is the same root cause as BROKKR-T-0292 and reinforces that its slice 1 must come before any chart plumbing.

**Also found:** `GET /health` on the broker port returns 200 HTML from the console SPA fallback, so an HTTP monitor pointed there passes unconditionally — worth knowing for anyone wiring external checks.

Follow-ups filed during the sweep: BROKKR-T-0312 (`brokkr register --help` and the client rustdoc claim duplicate registration is a no-op; the API returns 409).

*Original pass-2 file list, now done:* `getting-started/**`, `explanation/{components,architecture,security-model,data-flows,core-concepts}.md`, `how-to/{install-operations,security-hardening,webhooks,diagnostics,fleet-monitoring,log-streaming,templates}.md`, `reference/{templates,multi-tenancy,generators,diagnostics,monitoring,health-endpoints,ws-protocol,work-orders,webhooks,soft-deletion,agent-annotations,network-ports,deployment-health}.md`, plus all 107 minors and the Diátaxis misfilings. The artifact remains the checklist; annotate it inline as items are resolved or waived.