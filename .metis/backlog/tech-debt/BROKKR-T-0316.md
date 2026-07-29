---
id: required-checks-cover-only-unit-tests
level: task
title: "Branch protection requires only the four unit_tests contexts, so integration, SDK-contract and chart checks are advisory"
short_code: "BROKKR-T-0316"
created_at: 2026-07-29T04:30:00+00:00
updated_at: 2026-07-29T04:30:00+00:00
parent: 
blocked_by: []
archived: false

tags:
  - "#task"
  - "#tech-debt"
  - "#phase/backlog"


exit_criteria_met: false
initiative_id: NULL
---

# Branch protection requires only the four unit_tests contexts, so integration, SDK-contract and chart checks are advisory

## Objective

Decide which CI checks should block a merge to `main`, and make branch protection match that decision.

As of 2026-07-29, `main`'s required status checks are exactly four:

```
unit_tests / unit_tests (brokkr-agent)
unit_tests / unit_tests (brokkr-broker)
unit_tests / unit_tests (brokkr-models)
unit_tests / unit_tests (brokkr-utils)
openapi / drift_and_lint     <- added by BROKKR-T-0315 follow-up
```

Everything else that runs on a PR is advisory — it can be red on every push and merge is still permitted. That includes:

- `integration_tests` (the suite that actually exercises the API and DAL)
- `sdk_contract_tests`
- `Helm Template Validation` (`angreal helm check-values`, BROKKR-T-0313)
- `Helm Deployment Tests`
- `Build Multi-Arch Images` / `Create Multi-Arch Manifests`

## Backlog Item Details

### Type
- [x] Tech Debt - Code improvement or refactoring

### Priority
- [x] P2 - Medium (no live defect; it is the difference between "CI ran" and "CI mattered")

## 2026-07-29 — how this was found

Not by reading settings. `angreal openapi check` was failing on **ten consecutive pushes** to PR #89, from 2026-07-28T19:06 onward, and nothing surfaced it — the workflow ran, went red, and blocked nothing. The drift itself dated to commit `39f2b0a`, which edited a `WebhookFilters` doc comment (rendered into the spec description) without re-exporting, leaving `openapi/brokkr-v1.json` and both generated SDKs stale for several commits.

It was found only because BROKKR-T-0315 happened to regenerate the spec for an unrelated reason. That is the actual lesson: **a check nobody is required to pass is a check nobody reads.** The same reasoning that motivated BROKKR-T-0313 ("a green check named validation is not evidence unless someone has seen it fail") applies to red checks nobody is obliged to look at.

### The trap that makes this non-trivial

**A required check whose workflow never triggers blocks the PR forever.** GitHub distinguishes:

| Situation | Reported status | Effect as a required check |
|---|---|---|
| Workflow not triggered (top-level `paths:` did not match) | *nothing* | PR waits forever — unmergeable |
| Job skipped by an `if:` inside a workflow that ran | `skipped` | Accepted — merge allowed |

So any workflow that self-triggers behind a top-level `paths:` filter **cannot** simply be marked required. This is exactly why BROKKR-T-0315 converted `openapi.yml` to `workflow_call` and invoked it from `main.yml` behind the `changes` job: `main.yml` has no top-level path filter, so it reports on every PR.

`build-and-test.yml` (both Helm jobs, the image builds) still self-triggers with a top-level `paths:` filter on `crates/**`, `charts/**`, `.angreal/**`, etc. **Marking any of its jobs required without restructuring it first would make every docs-only PR unmergeable.** The fix is the same shape: move the path decision into a `changes` job, or add an equivalent always-reporting shim.

`integration_tests` and `sdk_contract_tests` are already called from `main.yml` behind `changes`, so they are safe to require today with no restructuring.

### Suggested split

- **Safe to require now** (already report on every PR via `main.yml`): `integration_tests`, `sdk_contract_tests`.
- **Needs restructuring first**: everything in `build-and-test.yml` — most valuably `Helm Template Validation`, which is a ~20s check since BROKKR-T-0313 and currently proves chart values still take effect while blocking nothing.
- **Probably should stay advisory**: `Helm Deployment Tests` and the multi-arch image builds, which are slow and infrastructure-flaky; requiring them trades merge throughput for a signal that nightly already provides.

Note that `enforce_admins` is `false` and `required_approving_review_count` is `0`, so required checks are the only real gate on `main` today. Worth deciding deliberately rather than by default.

## Acceptance Criteria

- [ ] A decision is recorded on which checks block a merge and which stay advisory, with the reasoning.
- [ ] `integration_tests` and `sdk_contract_tests` are added to required contexts, or a reason not to is recorded.
- [ ] Before any `build-and-test.yml` job is marked required, that workflow reports on every PR (jobs skipped via `if:`, not workflows skipped via `paths:`) — verified on a docs-only PR that it reports rather than hangs.
- [ ] Verify by observation, not by reading settings: open a PR that breaks a newly-required check and confirm merge is actually blocked.

## Status Updates

*To be added during implementation*
