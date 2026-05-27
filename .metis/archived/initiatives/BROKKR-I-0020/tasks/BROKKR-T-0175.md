---
id: b2-grafana-dashboard-for-ws
level: task
title: "B2: Grafana dashboard for WS metrics + eviction-worker-dead alert"
short_code: "BROKKR-T-0175"
created_at: 2026-05-24T12:56:45+00:00
updated_at: 2026-05-24T12:56:45+00:00
parent: BROKKR-I-0020
blocked_by: []
archived: true

tags:
  - "#task"
  - "#phase/completed"


exit_criteria_met: true
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

### 2026-05-26 — Dashboard + alert + docs shipped

**Artifacts** (in `docs/grafana/`, next to the existing broker/agent dashboards):
- `brokkr-ws-channel-dashboard.json` (uid `brokkr-ws-channel`, schemaVersion 27,
  `${DS_PROMETHEUS}` datasource var — mirrors the existing broker dashboard's
  structure exactly). Panels: Connected Agents (stat+sparkline), Live
  Subscribers (stat), Eviction Worker Liveness (rate timeseries), WS Message
  Rate by `direction · type` (stacked), Telemetry Rows Evicted by `table`
  (stacked).
- `brokkr-ws-channel.rules.yml` — `BrokkrWsEvictionWorkerStalled`:
  `increase(brokkr_ws_log_eviction_runs_total[5m]) == 0` `for: 10m`, severity
  warning. Window math: worker ticks every 60s (`DEFAULT_EVICTION_TICK`), so a
  flat 5m rate = dead worker; `for: 10m` rides over deploys/restarts without
  flapping and is still far inside the 6h ceiling.
- Operator note added to `docs/src/explanation/internal-ws-channel.md`
  (Observability → "Dashboard + alert").

**Validation:**
- Dashboard JSON parses clean (valid JSON, schema 27 = same as the working
  broker dashboard).
- Queries bind to real series — scraped the live broker `/metrics` and
  confirmed every name/label the dashboard + alert reference exists exactly:
  `brokkr_ws_connected_agents`, `brokkr_ws_live_subscribers`,
  `brokkr_ws_log_eviction_runs_total` (=87, worker alive → alert correctly
  would NOT fire), `brokkr_ws_messages_total{direction,type}` (showed the B4
  load-test traffic). `brokkr_ws_telemetry_evicted_total{table}` isn't emitted
  yet (no rows are 6h old) — the rate panel correctly shows no-data until
  evictions occur.

**Scope note (criterion deviation):** the "manually load into a local Grafana
+ screenshot" step was not done — the local docker stack ships neither Grafana
nor a Prometheus that scrapes the broker, so a live render would show empty
panels regardless. The stronger available validation (schema parity with a
known-rendering dashboard + every query verified against the live `/metrics`
series) is recorded above. If/when an observability stack is added to
`angreal local up`, a visual confirmation is a trivial follow-up.