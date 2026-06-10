---
id: deployment-health-discovery-label
level: task
title: "Deployment health discovery label is never injected — health always 'unknown'"
short_code: "BROKKR-T-0191"
created_at: 2026-06-10T03:04:17.905088+00:00
updated_at: 2026-06-10T03:04:17.905088+00:00
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

# Deployment health discovery label is never injected — health always "unknown"

## Objective

Deployment health discovers pods via the label selector `brokkr.io/deployment-object-id=<uuid>` across all namespaces (`crates/brokkr-agent/src/deployment_health.rs:235`), but nothing in Brokkr injects that label: the agent stamps tracking keys as **annotations** on the top-level applied objects only (`crates/brokkr-agent/src/k8s/objects.rs:80-100`), and those don't propagate to controller-created pods. Result: zero pods found ⇒ every deployment object reports `unknown` unless an operator manually labels pod templates with a UUID they can only learn after creation. Related defect found in the same code: `obj.metadata.annotations = Some(annotations)` REPLACES the manifest's own top-level annotations rather than merging.

Options: inject the label into `spec.template.metadata.labels` for workload kinds during `create_k8s_objects`, or resolve pods via ownerReferences from stack-annotated objects. Fix the annotation-clobbering while in there (merge, don't replace).

## Acceptance Criteria

- [ ] A plain Deployment submitted through Brokkr reports real health (healthy/degraded/failing) with no manual labeling
- [ ] User-supplied annotations on manifests survive apply (merged with Brokkr's keys)
- [ ] Diagnostics (which use the same selector) benefit or are explicitly covered by BROKKR-T-0190
- [ ] Docs updated: `docs/src/how-to/deployment-health.md` + `docs/src/reference/agent-annotations.md` "does not inject" caveats removed

## Status Updates

- 2026-06-09: Found during /docs-diataxis run while verifying health-discovery claims.
- 2026-06-09: Annotation-clobbering half FIXED — `create_k8s_objects` now merges Brokkr keys into the manifest's own annotation map (`k8s/objects.rs`). Label-injection half NOT implemented pending a design decision: (a) injecting `brokkr.io/deployment-object-id=<uuid>` into pod templates makes discovery trivial but the value changes on every stack revision, forcing a rollout of every workload in the stack per revision; (b) resolving pods via ownerReference chains from the stack-annotated top-level objects avoids manifest mutation and rollouts but needs generic ownerRef walking (Deployment→ReplicaSet→Pod). Decision needed before implementing.
- 2026-06-10: IMPLEMENTED ownerRef discovery (user chose option b): `check_deployment_objects` now does one cluster-wide pod list per cycle and attributes pods via label → own annotation → controller ownerReference chain walk (≤4 hops, memoized per owner, lazy Discovery) to the Brokkr-annotated top-level object (`deployment_health.rs:discover_pods/resolve_owner_doid`). Standard Deployments now report real health with zero manifest changes and no forced rollouts. Unit tests added (helper attribution logic); docs updated (reference/deployment-health, how-to, agent-annotations). Remaining AC: integration test in a live cluster.
- 2026-06-10 (closure pass): integration test added (tests/integration/deployment_health.rs) — Deployment with bare pod template, attribution via ownerReference chain, plus negative control.
