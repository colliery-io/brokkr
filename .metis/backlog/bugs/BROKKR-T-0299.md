---
id: diagnostics-select-pods-by-a-label
level: task
title: "Diagnostics select pods by a label Brokkr never applies (stamped as an annotation), so pod statuses and log tails come back empty"
short_code: "BROKKR-T-0299"
created_at: 2026-07-27T19:10:48.681867+00:00
updated_at: 2026-07-28T00:17:06.346151+00:00
parent: 
blocked_by: []
archived: false

tags:
  - "#task"
  - "#bug"
  - "#phase/completed"


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

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] Diagnostics resolve pods for a deployment object using the same strategy as `deployment_health.rs` (direct label → direct annotation → ownerReference chain), factored into a shared helper rather than copy-pasted.
- [ ] Integration or unit coverage for a Deployment→ReplicaSet→Pod chain where the pod carries neither label nor annotation.
- [ ] `collect_events` either honors the selector it currently ignores (`diagnostics.rs:332-371` takes `_label_selector` and lists the whole namespace) or documents namespace-scoping as deliberate.
- [ ] `docs/src/reference/diagnostics.md:250`'s limitation note updated to match the fixed behavior.

## Status Updates

**2026-07-28 — FIXED** on branch `docs/tenancy-review-2026-07`.

The attribution strategy was extracted from `HealthChecker` into a public `PodAttributor` in `deployment_health.rs`, with a private `OwnerFetcher` trait seam so the chain walk is testable without a `kube::Client`. `HealthChecker::discover_pods` now delegates to it — a provable no-op refactor on the health side (ordering, cache lifetime, miss memoization, `MAX_OWNER_DEPTH`, and break conditions are byte-for-byte the same). Kept in `deployment_health.rs` rather than a new module because adding one needs `pub mod` in `lib.rs` (outside ownership) and that file is where "what belongs to a deployment object" was already defined. Promoting it to `k8s/attribution.rs` is mechanical if a third consumer appears.

Diagnostics now take `deployment_object_id: Uuid` instead of a label selector and resolve pods through `PodAttributor::pods_for`, **once per namespace**, shared by both `pod_statuses` and `log_tails` (each previously listed separately). One attributor spans the request so discovery and owner lookups are reused across namespaces. `collect_pod_statuses` became the pure free function `pod_status_of`, which is what made it unit-testable.

**`collect_events` stays namespace-scoped, deliberately — and the ticket's framing of it as a plain oversight is wrong.** `ListParams::labels` on `Api<Event>` matches labels on the *Event* resource, which controllers never set, so honoring the parameter as written would have returned zero events in every real cluster — strictly worse than ignoring it. And attribution would delete the most useful payload: the events explaining a failure are routinely recorded against something other than the pod (`FailedCreate` on the ReplicaSet, `FailedScheduling` for a pod that never existed, quota/PVC/node events). The dead parameter is gone, `MAX_EVENTS` is named, and the rationale is in both the rustdoc and the reference page. Consumers can attribute client-side via `involved_object` + `involved_object_kind`.

Tests: `test_owner_chain_resolves_deployment_replicaset_pod` (the requested Deployment→ReplicaSet→Pod case with neither label nor annotation, asserting the direct lookup misses and the walk resolves in exactly two fetches), plus memoization, miss-caching, max-depth, and skip cases; `test_pod_status_of_projects_waiting_container` and `test_pod_status_of_without_status_is_unknown` in diagnostics. Clippy clean, 86 lib tests pass. `KubeOwnerFetcher` and the client-dependent paths stay integration-only rather than being covered by a contrived HTTP mock.

**Note for review:** the agent edited two lines of `crates/brokkr-agent/tests/integration/diagnostics.rs` (outside its stated ownership) because the signature change breaks compilation and `--all-targets` builds it. Assertions and fixtures unchanged.

**Follow-up filed as part of this work:** `docs/src/reference/agent-annotations.md` still tells readers diagnostics require the label and return empty without it — now actively wrong. Corrected here.