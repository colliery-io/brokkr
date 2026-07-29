---
id: no-chart-render-assertions-values
level: task
title: "No chart-render assertions: values that do nothing can regress silently, as four just did"
short_code: "BROKKR-T-0313"
created_at: 2026-07-28T20:55:32.830942+00:00
updated_at: 2026-07-28T21:53:42.872206+00:00
parent: 
blocked_by: []
archived: false

tags:
  - "#task"
  - "#tech-debt"
  - "#phase/completed"


exit_criteria_met: false
initiative_id: NULL
---

# No chart-render assertions: values that do nothing can regress silently, as four just did

## Objective

Add render-level assertions over both Helm charts so a value that stops taking effect fails a check rather than shipping.

BROKKR-T-0308 fixed four values that were accepted, documented, and rendered nothing — a ServiceMonitor selecting a port no Service defined, a collector endpoint pointing at a sidecar that was never rendered, a `metrics.enabled` that only gated a NetworkPolicy rule, and a `service.annotations` that appeared nowhere in the output. All four passed `helm lint`. All four survived multiple releases. Two were only found because someone was writing the README and read the templates line by line.

That is the actual defect: **nothing in CI asserts that setting a value changes the rendered output.** `helm lint` checks syntax and schema, not effect. The four fixes are individually correct but there is no reason to believe they will stay correct, and no reason to believe the remaining values are any better than the ones that were checked.

Deferred from BROKKR-T-0308's acceptance criterion 3, which asked for exactly this and which that ticket could not satisfy: verification there was by hand, and `angreal helm test` needs a live cluster.

## Backlog Item Details

### Type
- [x] Tech Debt - Code improvement or refactoring

### Priority
- [x] P2 - Medium (no live defect; this is what stops the class from recurring)

## 2026-07-28 — approach settled from what the repo already does

**This repo already has this exact pattern, for OpenAPI drift — mirror it rather than inventing one.** `angreal openapi check` renders into a temp location and compares, returning non-zero on drift; `.github/workflows/openapi.yml` runs it behind a `paths:` filter so unrelated pushes do not pay for it. The chart equivalent is a new `angreal helm` subcommand plus a path-filtered workflow on `charts/**`.

Two facts that shape the design:
- **`angreal helm test` is the only existing helm *task* and it needs a live k3s cluster** (builds images, pushes to a local registry, installs). The new check must be `helm template` only — no cluster, no docker — so it runs on every chart change in seconds.
- **CORRECTED (Dylan, 2026-07-28): chart validation DOES already run on PRs, and that is the sharper version of this ticket.** `build-and-test.yml` triggers on `pull_request` with `charts/**` in its paths filter and contains a `helm-template-tests` job ("Helm Template Validation") that renders both charts against default, production, and development values. **Every one of those renders pipes to `/dev/null`.** It asserts only that templating does not error — it cannot catch a value that renders nothing, because rendering nothing is not an error.

That is exactly how all four BROKKR-T-0308 defects passed CI for multiple releases: rendered, discarded, declared passing. So this ticket is **not adding missing coverage** — it is replacing a check that has been running, passing, and proving less than its name implies. Implementation therefore **extends `helm-template-tests` in `build-and-test.yml`** rather than adding a parallel workflow, keeping the job name intact so required-check configuration does not break, and keeping the existing Helm setup and `helm dependency build` steps. `helm-deployment-tests` (which runs `angreal helm test smoke` against a cluster) is left alone.

**Full inventory of what exists today** (checked properly on the second correction — the earlier summaries above were both incomplete):

| Layer | Location | What it actually asserts |
|---|---|---|
| CI template job | `build-and-test.yml` → `helm-template-tests`, on PR *and* main push, path-filtered `charts/**` | Both charts × 3 values files render — **piped to `/dev/null`** |
| Task template phase | `run_parallel_template_tests` → Phase 1 of `angreal helm test` | Both charts × **4** values files (includes staging, which the CI job misses) — **also `> /dev/null 2>&1`**, with the comment "we only care about exit code" |
| Deployment tests | `helm-deployment-tests` → `angreal helm test smoke` on real k3s | Substantial and real: bundled/external DB, multi-tenant schema, agent RBAC modes, per-values-file installs, Shipwright e2e |

So chart testing is **not absent** — the deployment tier genuinely asserts behavior. The gap is precisely the middle rung: **nothing asserts that setting a value produces the rendered output it claims**, and the render-to-`/dev/null` pattern exists independently in two places. That duplication is plausibly how the habit survived.

Scope consequence: the work must reconcile *both* sites, not just the CI job, and must not lose the in-task path's staging coverage. The cluster-based deployment tests are out of scope and stay as they are.

The lesson generalizes past this ticket: a green check named "validation" is not evidence unless someone has seen it fail. Part 3 of the work — break a template, watch the suite fail, restore it — exists for that reason. Worth noting that I asserted this ticket's premise wrongly twice before reading the test code, which is the same failure in miniature.

### Technical Debt Impact
- **Current Problems**: a value can silently stop taking effect, and the failure is invisible — `helm install` succeeds, the capability is absent, and the operator believes it is enabled. This class of bug is discovered only by reading templates.
- **Benefits of Fixing**: the next inert value fails a check instead of shipping. Also gives the chart READMEs' values tables something to be checked against, since they are currently maintained by hand and stamped with a chart version.
- **Risk Assessment**: without it, BROKKR-T-0308's fixes are a snapshot rather than a guarantee, and the same four could regress under a refactor with nothing to catch it.

### Technical Approach

`helm template` plus assertions is enough and needs no cluster — this should run in ordinary CI, not the k3s e2e path. For each security- or capability-relevant value: render with it set and with it unset, and assert the rendered output differs in the expected way. The high-value cases are the ones already known to have failed or to matter:

- `metrics.podMonitor.enabled` renders a PodMonitor whose `port` matches a declared container port name.
- `networkPolicy.allowMetricsScraping` toggles the metrics ingress rule.
- `service.annotations` appears on the Service.
- `telemetry.otlpEndpoint` reaches `BROKKR__TELEMETRY__OTLP_ENDPOINT`.
- Each `existingSecret` value omits its plaintext counterpart from the ConfigMap **and** adds the `secretKeyRef` — the pairing that makes the credential work is exactly what a refactor could half-break.
- `broker.pakHash` renders only when non-empty (the empty-value trap that left the public dev credential live).

A generic guard is worth considering alongside the specific ones: assert that every leaf key in `values.yaml` appears somewhere in `templates/`, with an explicit allowlist for keys consumed by subcharts or intentionally inert. That would have caught all four of BROKKR-T-0308's defects mechanically, without anyone needing to suspect them.

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] Render assertions cover the values listed above for both charts, runnable without a cluster.
- [ ] They run in CI on chart changes, and fail loudly rather than warning.
- [ ] A deliberately broken template (e.g. drop the `service.annotations` block) makes the suite fail — verify the check actually catches the class, rather than trusting that it would.
- [ ] Decide on the generic "every values key is referenced" guard, and either implement it with a documented allowlist or record why not.

## Status Updates

**2026-07-28 — DONE** on branch `docs/tenancy-review-2026-07` (commit `3678bec`). `angreal helm check-values`: 52 assertions, ~2s, no cluster. Verified independently — all pass, and the missing-PyYAML path exits 1 with actionable guidance rather than silently skipping.

**This ticket oversold its own generic guard, and that correction matters.** It claimed the guard "would have caught all four of BROKKR-T-0308's defects mechanically". It would have caught **one** — `service.annotations`, the only unreferenced key. `metrics.enabled` *was* referenced (in `networkpolicy.yaml`) and satisfies a reference-existence check exactly as written; the ServiceMonitor port mismatch is cross-resource linkage with no unreferenced key at all; `telemetry.collector.enabled` was referenced but pointed at a sidecar never rendered. Only the targeted layer catches that class. Anyone reading the original wording might conclude the targeted assertions are optional — they are the opposite.

**The guard earned its place differently than predicted**: it found three defects nobody suspected, all in the shipped `values/` overlays rather than `values.yaml` — `securityContext.*` in both charts (templates implement `podSecurityContext`/`containerSecurityContext`) and `extraEnv` in the agent (only the broker renders it; the production overlay sets `RUST_LOG=info` there and it is dropped). Filed as BROKKR-T-0314, tolerated meanwhile by allowlist entries whose `DEFECT` reasons print on every run. `values.yaml` itself came back clean apart from one legitimate Bitnami subchart entry.

**A brief instruction of mine was wrong and was correctly refused:** I told the agent to allowlist `postgresql.auth.*` as subchart-consumed. `_helpers.tpl` reads those to build `BROKKR__DATABASE__URL`, so allowlisting them would have blinded the guard to a real regression.

**Both `/dev/null` sites reconciled.** `run_parallel_template_tests` now delegates to the same in-process assertions and `helm_template_test` is deleted, taking Phase 1 from ~30s of container startup to ~2s and removing an unnecessary cluster coupling — rendering never needed the cluster. Staging coverage is preserved; the new suite covers six values-file cases per chart against the old four.

**Proven by breaking, not by reading** (acceptance criterion 3): deleting the `service.annotations` block failed both the targeted assertion and the guard; deleting `hostAliases`, which has no targeted assertion, failed the guard alone. Both restored, `git status` clean on templates.

**CI**: `helm-template-tests` keeps its job name so required-check configuration survives, and the replacement is a strict superset — same binary, same pinned `--kube-version 1.30.0`, twelve renders instead of six (adding staging and values-dev), asserting non-empty output rather than exit code alone. No new workflow and no new path filter were needed; `.angreal/**` and `charts/**` were already covered.

**Left deliberately:** `helm-template-tests` still has `needs: merge-manifests`, so a two-second render check waits on multi-arch image builds — cheap to fix, out of scope, and worth doing. No `helm lint` step was added; it is the check whose weakness prompted this ticket.