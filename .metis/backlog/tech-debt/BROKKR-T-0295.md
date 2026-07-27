---
id: docs-drift-sweep-remaining-major
level: task
title: "Docs drift sweep: remaining major/minor findings from the 2026-07-27 full-tree review (see docs/REVIEW-2026-07-27.md)"
short_code: "BROKKR-T-0295"
created_at: 2026-07-27T14:28:02.502537+00:00
updated_at: 2026-07-27T14:28:02.502537+00:00
parent: 
blocked_by: []
archived: false

tags:
  - "#task"
  - "#phase/backlog"
  - "#tech-debt"


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

- [ ] Every finding in docs/REVIEW-2026-07-27.md is either fixed, covered by another ticket, or explicitly waived (annotate the artifact inline).
- [ ] Fixes verified against code citations rather than trusting the reviewer text.
- [ ] `angreal docs build` passes; SUMMARY/index pages link-checked.

## Status Updates

*To be added during implementation*
