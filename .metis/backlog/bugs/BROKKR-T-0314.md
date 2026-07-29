---
id: shipped-values-overlays-set-three
level: task
title: "Shipped values/ overlays set three keys that render nothing: securityContext in both charts, extraEnv in the agent"
short_code: "BROKKR-T-0314"
created_at: 2026-07-29T00:43:39.850276+00:00
updated_at: 2026-07-29T00:43:39.850276+00:00
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

# Shipped values/ overlays set three keys that render nothing: securityContext in both charts, extraEnv in the agent

## Objective

Fix three values that the shipped `values/` overlays set and the templates ignore. Found mechanically by the generic guard added in BROKKR-T-0313 — nobody suspected them, and they are currently tolerated by that guard's allowlist with `DEFECT` reasons that print on every run.

1. **`securityContext.*` — both charts.** `charts/brokkr-{broker,agent}/values/{development,production,staging}.yaml` set a top-level `securityContext`. Both charts implement `podSecurityContext` and `containerSecurityContext` instead. The overlays render nothing, so anyone relying on the production overlay for hardening does not get it.
2. **`extraEnv` — agent only.** `charts/brokkr-agent/values/{development,production,staging}.yaml` set `extraEnv`; only the *broker* deployment renders it. The agent's production overlay sets `RUST_LOG=info` and reads as working configuration. It is silently dropped.

The sharp part is that these live in the shipped overlays rather than in `values.yaml` — the files a consumer is most likely to adopt wholesale, and the ones nobody had scanned. `values.yaml` itself came back clean apart from one legitimate subchart entry.

## Backlog Item Details

### Type
- [x] Bug - Production issue that needs fixing

### Priority
- [x] P2 - Medium (no runtime failure; each silently withholds configuration the operator wrote down, and the security-context case withholds hardening)

## Acceptance Criteria

- [ ] Decide per item whether the overlay keys are renamed to the keys the templates implement (`podSecurityContext`/`containerSecurityContext`), or the templates are taught to honour the key the overlays use. Renaming the overlays is the smaller change; teaching the agent to render `extraEnv` is arguably the right one, since the broker already does and the asymmetry is itself surprising.
- [ ] The three `DEFECT` entries are deleted from `VALUES_KEY_ALLOWLIST` in `.angreal/task_helm.py` as part of the fix — leaving them would re-hide the problem, and `angreal helm check-values` will then enforce it.
- [ ] `angreal helm check-values` passes with those entries removed.
- [ ] Chart README values tables reflect whichever key survives; check that neither README currently documents a `securityContext` key that does not exist.

## Status Updates

*To be added during implementation*
