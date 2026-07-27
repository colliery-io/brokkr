---
id: diagnostics-select-pods-by-a-label
level: task
title: "Diagnostics select pods by a label Brokkr never applies (stamped as an annotation), so pod statuses and log tails come back empty"
short_code: "BROKKR-T-0299"
created_at: 2026-07-27T19:10:48.681867+00:00
updated_at: 2026-07-27T19:10:48.681867+00:00
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

# Diagnostics select pods by a label Brokkr never applies (stamped as an annotation), so pod statuses and log tails come back empty

## Objective

Make deployment-object-scoped diagnostics actually attribute pods. The agent filters pod statuses and log tails with the label selector `brokkr.io/deployment-object-id=<id>` (`crates/brokkr-agent/src/cli/commands.rs:639-651`), but Brokkr stamps that key as an **annotation on the top-level applied object** — never as a **label on pods** (`crates/brokkr-agent/src/k8s/objects.rs:82-94`). Unless an operator hand-labels their pod templates, `pod_statuses` and `log_tails` come back empty and only the (namespace-wide, unfiltered) events are populated.

`crates/brokkr-agent/src/deployment_health.rs:250-296` already solves exactly this problem correctly: direct label, then direct annotation, then an ownerReference-chain walk up to the owning workload. Diagnostics never received that fix. `docs/src/reference/diagnostics.md:250` already concedes the limitation in prose.

Found 2026-07-27 while scoping BROKKR-T-0275 (console diagnostics button). Filed separately because fixing the console route without this ships a button that reliably returns empty pod data — a fix that looks broken.

## Backlog Item Details

### Type
- [x] Bug - Production issue that needs fixing

### Priority
- [x] P1 - High (the feature's primary payload is silently empty in the normal case)

### Impact Assessment
- **Affected Users**: anyone running diagnostics on a deployment object whose pod templates don't carry the Brokkr label by hand — i.e. the default.
- **Expected vs Actual**: expected pod statuses and recent log tails for the deployment object's workloads; actual empty `pod_statuses`, no `log_tails`, and up to 50 namespace-wide events with no attribution.
- **Aggravating factor**: because collection errors also produce a completed-with-empty-payload result (see BROKKR-T-0291), an operator cannot distinguish "nothing matched" from "collection failed".

## Acceptance Criteria

- [ ] Diagnostics resolve pods for a deployment object using the same strategy as `deployment_health.rs` (direct label → direct annotation → ownerReference chain), factored into a shared helper rather than copy-pasted.
- [ ] Integration or unit coverage for a Deployment→ReplicaSet→Pod chain where the pod carries neither label nor annotation.
- [ ] `collect_events` either honors the selector it currently ignores (`diagnostics.rs:332-371` takes `_label_selector` and lists the whole namespace) or documents namespace-scoping as deliberate.
- [ ] `docs/src/reference/diagnostics.md:250`'s limitation note updated to match the fixed behavior.

## Status Updates

*To be added during implementation*
