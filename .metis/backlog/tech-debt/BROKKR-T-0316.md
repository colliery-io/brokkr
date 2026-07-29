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
- [x] `integration_tests` and `sdk_contract_tests` are added to required contexts, or a reason not to is recorded.
- [ ] Before any `build-and-test.yml` job is marked required, that workflow reports on every PR (jobs skipped via `if:`, not workflows skipped via `paths:`) — verified on a docs-only PR that it reports rather than hangs.
- [ ] Verify by observation, not by reading settings: open a PR that breaks a newly-required check and confirm merge is actually blocked.

## Status Updates

**2026-07-29 — PARTIALLY DONE (Dylan): `integration_tests` and `sdk_contract_tests` are now required.** Required contexts on `main` went from four to ten:

```
unit_tests / unit_tests (brokkr-agent | brokkr-broker | brokkr-models | brokkr-utils)
openapi / drift_and_lint
integration_tests / integration_tests (brokkr-agent | brokkr-broker)
sdk_contract_tests / sdk_contract_tests (rust | python | typescript)
```

Safe with no restructuring, and checked rather than assumed: both are `workflow_call` reusables invoked from `main.yml` behind the `changes` job's `if:`, so on a PR that touches neither they report **skipped** (accepted) rather than never reporting (hangs forever). Both matrices are static literals with `fail-fast: false`, so every leg reports its own status — no context can silently fail to materialize. The five names were taken from what the live PR actually reported, not inferred from the YAML.

**What this changes in practice:** `integration_tests` is the suite that exercises the API and DAL against a real database, and it could previously be red on every push without blocking anything. That was the largest gap between "CI ran" and "CI mattered".

### Still open — the Helm jobs

`build-and-test.yml` continues to self-trigger behind a top-level `paths:` filter, so its jobs are still advisory and **still cannot be required without restructuring** — the trap documented above would make every docs-only PR unmergeable. Most valuable target remains `Helm Template Validation`: since BROKKR-T-0313 it is a ~20s check that proves chart values actually take effect, and it currently blocks nothing. The fix is the same shape applied to `openapi.yml`: convert to `workflow_call` and invoke from `main.yml` behind a `changes` filter.

`Helm Deployment Tests` and the multi-arch image builds are still judged better left advisory — slow and infrastructure-flaky, and nightly already covers them.

### Acceptance criterion 4 is deliberately still open

Nobody has yet confirmed by observation that a broken required check actually blocks a merge. The settings say it does; that is not the same as having seen it. Given this ticket exists because a red check blocked nothing for ten pushes, the distinction is the whole point.
