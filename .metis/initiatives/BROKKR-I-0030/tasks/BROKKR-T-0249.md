---
id: agent-helm-chart-expose-brokkr
level: task
title: "Agent Helm chart: expose BROKKR_GENERATOR_IDS via generatorIds value"
short_code: "BROKKR-T-0249"
created_at: 2026-06-27T13:42:34.319542+00:00
updated_at: 2026-06-27T13:59:19.043544+00:00
parent: multi-application-isolation-safe
blocked_by: []
archived: false

tags:
  - "#task"
  - "#phase/completed"


exit_criteria_met: false
initiative_id: BROKKR-I-0030
---

# Agent Helm chart: expose BROKKR_GENERATOR_IDS via generatorIds value

## Parent Initiative

[[BROKKR-I-0030]]

## Objective

Make the generator-registration scope settable at deploy time through the
`brokkr-agent` Helm chart. I-0030 (#79, v0.8.2) shipped the agent's
`BROKKR_GENERATOR_IDS` self-registration path, but the agent chart was only
version-bumped (`Chart.yaml`) — `values.yaml`, `configmap.yaml`, and the
per-environment values were never wired. Today a Helm-deployed agent cannot
subscribe to any application generator without bypassing the chart (manual
`kubectl edit` or a post-startup API call), which defeats the ADR-0009
"declare the agent's application scope at deploy time" workflow.

### Type
- [x] Bug — released feature is unreachable through its intended (Helm) interface

### Priority
- [x] P1 — blocks the documented operator workflow for the 0.8.2 feature

## Acceptance Criteria

- [x] `charts/brokkr-agent/values.yaml` gains a documented `generatorIds` value
      (list or string) defaulting to **empty** (`broker.generatorIds: []`).
- [x] `charts/brokkr-agent/templates/configmap.yaml` emits `BROKKR_GENERATOR_IDS`
      from `generatorIds`; the agent already consumes the configmap via
      `envFrom: configMapRef` (`deployment.yaml:56-58`), so no deployment-template
      change was required. Uses `{{- with }}` + `kindIs "slice"` to accept both a
      list (joined to CSV) and a bare comma string.
- [x] Empty default is a true no-op: the key is omitted entirely from the rendered
      ConfigMap (verified via `helm template`), so the agent's
      `if !generator_ids_raw.trim().is_empty()` guard skips self-registration.
- [x] Per-environment values (`values/development.yaml`, `values/staging.yaml`,
      `values/production.yaml`, `values-dev.yaml`) documented with a commented
      `generatorIds` example; no non-empty default committed.
- [x] Chart `README.md` documents the value and its semantics, explicitly stating
      that **empty = system/fleet scope only** (the broker auto-registers every
      agent with the `__system__` generator at `POST /agents`), NOT the admin
      generator. See [[BROKKR-T-0250]] for the config-struct promotion.
- [x] `helm lint` clean + `helm template` renders verified for empty / list /
      string cases. Full `angreal helm test` (k3s) recommended pre-merge in CI.

## Implementation Notes

### Technical Approach

- `BROKKR_GENERATOR_IDS` is read by the agent via a raw `std::env::var`
  (`crates/brokkr-agent/src/cli/commands.rs:+143`), single-underscore, NOT a
  `BROKKR__AGENT__*` config key. So it must be injected as a plain env var; the
  existing `envFrom: configMapRef` already does this once the key is present in
  the configmap. Adding the key to `configmap.yaml` next to the
  `BROKKR__AGENT__*` keys is the minimal, consistent wiring.
- Accept either a YAML list or a comma string in `values.yaml`; normalise to a
  comma-separated string in the template (e.g. `{{ join "," .Values.generatorIds }}`
  when a list). Document the chosen shape.

### Dependencies

- Pairs with [[BROKKR-T-0250]] (config-struct promotion). If T-0250 lands first
  and adds `BROKKR__AGENT__GENERATOR_IDS`, prefer emitting that key instead of
  the bare `BROKKR_GENERATOR_IDS` for convention consistency — coordinate the
  key name between the two tasks.

### Risk Considerations

- Empty/whitespace handling must round-trip cleanly so the default never
  triggers a spurious registration attempt or a malformed-UUID warning.

## Status Updates

**2026-06-27 — Implemented.** Wired `broker.generatorIds` end-to-end in the
`brokkr-agent` chart:
- `values.yaml`: added `broker.generatorIds: []` with documented semantics
  (empty = system/fleet scope only, not the admin generator; accepts list or
  comma string).
- `templates/configmap.yaml`: emits `BROKKR_GENERATOR_IDS` via
  `{{- with .Values.broker.generatorIds }}` + `kindIs "slice"` (list→CSV, else
  passthrough string). Consumed through the existing `envFrom: configMapRef`.
- `values/{development,staging,production}.yaml` + `values-dev.yaml`: commented
  `generatorIds` examples; no non-empty default committed.
- `README.md`: new "Generator scope" subsection + values-table row.

Verified with `helm lint` (clean) and `helm template`:
- default (empty) → `BROKKR_GENERATOR_IDS` absent (no-op);
- list `{a,b}` → `"a,b"`; escaped comma string → passthrough;
- `BROKKR__AGENT__PAK` and other keys unaffected; production overlay renders 12
  manifests.

Done on branch `feat/i0030-operator-surface` (rebased onto v0.8.2 / #79). Key name
left as bare `BROKKR_GENERATOR_IDS` pending [[BROKKR-T-0250]]; the configmap
comment flags the coordination point.