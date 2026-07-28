---
id: broker-chart-sets-no
level: task
title: "Broker chart sets no serviceAccountName and creates no RBAC, so ConfigMap watching runs as default SA without get/watch"
short_code: "BROKKR-T-0309"
created_at: 2026-07-28T15:14:26.875347+00:00
updated_at: 2026-07-28T15:14:26.875347+00:00
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

# Broker chart sets no serviceAccountName and creates no RBAC, so ConfigMap watching runs as default SA without get/watch

## Objective

Determine whether the broker's ConfigMap watching can actually work under the chart as shipped, and fix whichever side is wrong.

`configReload.enabled` defaults to **true** and the chart renders `BROKKR_CONFIG_WATCHER_ENABLED: "true"`, but `charts/brokkr-broker/templates/deployment.yaml` sets no `serviceAccountName` and the chart creates no ServiceAccount, Role, or RoleBinding. The pod therefore runs as the namespace's `default` ServiceAccount, which in a default cluster has no `get`/`watch` on ConfigMaps. Contrast the agent chart, which does create RBAC for what it needs.

Found 2026-07-28 while documenting the chart values (BROKKR-T-0294). Only the chart-side facts were recorded there; what the broker binary does when a watch is denied was deliberately not asserted.

**Interaction with BROKKR-T-0292 — read that first.** That ticket establishes the watcher only activates when `BROKKR_CONFIG_FILE` points at an existing file, which the chart never sets, and that nothing reads the reloaded config back anyway. So today this RBAC gap may be entirely masked: the watcher never starts, so it never attempts a watch, so the missing permission never surfaces. **The gap becomes real the moment T-0292's slice 2 mounts the ConfigMap and sets `BROKKR_CONFIG_FILE`** — at which point config reload would fail on permissions rather than on plumbing, and the failure could easily be misdiagnosed as the plumbing still being wrong.

## Backlog Item Details

### Type
- [x] Bug - Production issue that needs fixing (latent; blocks BROKKR-T-0292 from actually working)

### Priority
- [x] P2 - Medium (currently masked; would silently defeat T-0292's slice 2)

## Acceptance Criteria

- [ ] Establish what the broker does when a ConfigMap watch is denied — fail loudly, or degrade silently. If it degrades silently, that is its own defect and should be fixed alongside, since a silent permission failure is indistinguishable from the feature simply not running.
- [ ] Decide whether the chart should create a ServiceAccount + Role/RoleBinding granting `get`/`watch` on its own ConfigMap, or whether ConfigMap watching should be removed entirely per BROKKR-T-0292's original recommendation. These are the same decision; do not answer them separately.
- [ ] If RBAC is added, `helm template` shows the ServiceAccount bound and the Deployment referencing it, covered by a chart test.
- [ ] Coordinate with BROKKR-T-0292 so the chart is not left half-wired in either direction.

## Status Updates

*To be added during implementation*
