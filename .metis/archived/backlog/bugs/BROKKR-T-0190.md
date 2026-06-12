---
id: diagnostics-namespace-hardcoded-to
level: task
title: "Diagnostics namespace hardcoded to 'default' — derive from deployment object"
short_code: "BROKKR-T-0190"
created_at: 2026-06-10T03:04:11.031061+00:00
updated_at: 2026-06-10T11:19:07.512767+00:00
parent: 
blocked_by: []
archived: true

tags:
  - "#task"
  - "#bug"
  - "#phase/completed"


exit_criteria_met: false
initiative_id: NULL
---

# Diagnostics namespace hardcoded to "default" — derive from deployment object

## Objective

The namespace the agent searches when fulfilling an on-demand diagnostics request is the unconditional literal `"default"` (`crates/brokkr-agent/src/cli/commands.rs:387`, flowing into `collect_diagnostics()` at `diagnostics.rs:146` for pod statuses, events, and log tails). For workloads in any other namespace the label selector matches nothing and the agent submits an empty-but-successful result (`pod_statuses: []`) — no error indicates the namespace mismatch. The code comment marks it as a known TODO ("should be derived from the deployment object").

Derive the namespace(s) from the deployment object's manifests (parse `metadata.namespace` from `yaml_content`), or search across namespaces by label.

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] Diagnostics return data for workloads outside `default`
- [ ] Empty results vs. wrong-namespace are distinguishable (or impossible)
- [ ] Integration test with a non-default-namespace workload
- [ ] "Known Limitations" notes removed from `docs/src/reference/diagnostics.md` and `docs/src/how-to/diagnostics.md`

## Status Updates

- 2026-06-09: Documented as a known limitation during /docs-diataxis run; this task removes the limitation.
- 2026-06-09: IMPLEMENTED (uncommitted, unit tests green incl. new manifest_namespaces tests): diagnostics namespaces now derived from the deployment object manifests (fetch via SDK → `manifest_namespaces()` → `collect_diagnostics_in()` merging across namespaces; falls back to `default` if the fetch fails). Docs updated. Remaining AC: integration test with a non-default-namespace workload.
- 2026-06-10 (closure pass): integration test added (tests/integration/diagnostics.rs) — pods in two namespaces collected by collect_diagnostics_in.