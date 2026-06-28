---
id: slice-5-telemetry-view-kube-events
level: task
title: "Slice 5: Telemetry view — kube events + pod logs (REST-poll, 6h retention)"
short_code: "BROKKR-T-0260"
created_at: 2026-06-28T01:44:26.917487+00:00
updated_at: 2026-06-28T01:44:26.917487+00:00
parent: brokkr-operator-console
blocked_by: []
archived: false

tags:
  - "#task"
  - "#phase/todo"


exit_criteria_met: false
initiative_id: BROKKR-I-0031
---

# Slice 5: Telemetry view

## Parent Initiative

[[BROKKR-I-0031]] · decision [[BROKKR-A-0010]]

## Objective

Telemetry view with Kube-events / Pod-logs tabs, per the handoff §Telemetry. Bind via **REST polling**
(the design's assumed live-WS telemetry stream does not exist; live streaming is a deferred broker
enhancement — see [[BROKKR-A-0010]]).

### Type
- [x] Feature — view slice

## Acceptance Criteria

- [ ] Tab segmented control: **Kube events** / **Pod logs**; gold caption "⚠ 6h retention window · ship
      to Datadog for long-term".
- [ ] Kube events: rows — severity pill (normal/warning), mono reason, ellipsized message, mono `ns/…`,
      mono "ago". Bound to `/stacks/:id/events` (and/or `/agent-events`).
- [ ] Pod logs: `--inset` well with mono `<pre>` tailing `[HH:MM:SS] ns/pod/container: <line>`; bound to
      `/stacks/:id/logs`; keep last ~90 lines; **poll** on the live interval (gated by Live/Paused).
- [ ] Loading/Empty/Error states; respect the 6h retention reality (empty/older-than-window messaging).

## Dependencies

- Depends on [[BROKKR-T-0255]], [[BROKKR-T-0256]] (poll loop), slice 1.

## Implementation Notes

- Reference: handoff §4 Telemetry; broker `/stacks/:id/{events,logs}`, `/agent-events`. NOTE the design
  assumes a live WS for events/logs — v1 polls; flag the gap for the future live-telemetry enhancement.

## Status Updates

*To be added during implementation*
