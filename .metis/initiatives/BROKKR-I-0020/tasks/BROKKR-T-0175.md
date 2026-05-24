---
id: b2-grafana-dashboard-for-ws
level: task
title: "B2: Grafana dashboard for WS metrics + eviction-worker-dead alert"
short_code: "BROKKR-T-0175"
created_at: 2026-05-24T12:56:45.000000+00:00
updated_at: 2026-05-24T12:56:45.000000+00:00
parent: BROKKR-I-0020
blocked_by: []
archived: false

tags:
  - "#task"
  - "#phase/todo"


exit_criteria_met: false
initiative_id: BROKKR-I-0020
---

# B2: Grafana dashboard for WS metrics + eviction-worker-dead alert

## Parent Initiative

[[BROKKR-I-0020]]

## Objective

I-0019 added five new Prometheus metrics for the WS channel but didn't
update any Grafana dashboards. Operators have no visibility into WS
health out of the box. Ship a dashboard JSON + one alert rule that
catches the most-dangerous silent failure mode: the eviction worker
dying, which would silently violate the 6h retention ceiling.

## Acceptance Criteria

- [ ] `dashboards/ws-channel.json` (or panels added to an existing dashboard)
      with these panels:
  - `brokkr_ws_connected_agents` — single-stat + sparkline
  - `brokkr_ws_messages_total` rate by `{direction, type}` — stacked area
  - `brokkr_ws_live_subscribers` — gauge
  - `brokkr_ws_log_eviction_runs_total` rate — sanity check graph
  - `brokkr_ws_telemetry_evicted_total{table}` rate — stacked area by table
- [ ] Alert rule (Prometheus alerting rule YAML): `brokkr_ws_log_eviction_runs_total`
      has not incremented for > 2× the configured tick interval; severity warning
      (firing means 6h ceiling is about to be silently violated)
- [ ] Dashboard JSON checked into the docs / observability folder where
      existing dashboards live
- [ ] Brief operator note in `docs/src/explanation/internal-ws-channel.md`
      pointing to the dashboard + alert
- [ ] Manually loaded into a local Grafana to confirm rendering (screenshot
      attached to task notes, or recorded in status updates)

## Implementation Notes

### Technical Approach

- Check existing dashboards in the repo (search for `*.json` under
  `dashboards/` or similar) — match style, datasource UID variable usage,
  template variable conventions
- The eviction alert uses `increase(brokkr_ws_log_eviction_runs_total[5m]) == 0`
  with a `for: 10m` clause — straightforward, but verify against actual tick
  interval default (1 min last we checked) and adjust window accordingly

### Dependencies

None.

### Risk Considerations

- If the alert is too tight it will flap during deploys; tune the `for:`
  window to be at least 2× the tick interval plus deploy time
- Dashboards drift; consider committing a script that regenerates from a
  source-of-truth YAML, but for this task, raw JSON checked in is fine

## Status Updates

*To be added during implementation*
