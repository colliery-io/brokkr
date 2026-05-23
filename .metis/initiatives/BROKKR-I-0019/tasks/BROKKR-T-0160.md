---
id: ws-05-agent-broker-uplink
level: task
title: "WS-05: Agent→broker uplink — heartbeat, events, health over WS"
short_code: "BROKKR-T-0160"
created_at: 2026-05-23T02:12:35.870220+00:00
updated_at: 2026-05-23T02:12:35.870220+00:00
parent: BROKKR-I-0019
blocked_by: []
archived: false

tags:
  - "#task"
  - "#phase/todo"


exit_criteria_met: false
initiative_id: BROKKR-I-0019
---

# WS-05: Agent→broker uplink — heartbeat, events, health over WS

**Parent**: [[BROKKR-I-0019]] · **ADR**: [[BROKKR-A-0008]]

## Objective

Move agent→broker traffic (heartbeat, agent events, health reports) onto the WS channel when connected. REST endpoints stay available — agent uses REST automatically when WS is down.

## Acceptance Criteria

- [ ] Heartbeat tick sends `Heartbeat` over WS when connected, falls back to REST POST when not
- [ ] Agent events sent via `AgentEvent` over WS; REST `POST /agents/{id}/events` retained as fallback
- [ ] Agent health sent via `AgentHealth` over WS; REST fallback retained
- [ ] Metric: per-channel send rate confirms REST traffic on these paths drops to ~0 when WS is healthy
- [ ] Integration test: kill WS, observe REST resumes; restore WS, observe REST stops

## Implementation Notes

- **Approach**: introduce a small `BrokerSink` trait with WS + REST implementations; an enum dispatcher picks based on the connection-state watch from WS-03.
- **Dependencies**: WS-03.
- **Risk**: dual-emission during the WS-up/WS-down transition. Use the connection-state watch as the single source of truth; never emit on both paths simultaneously.