---
id: ws-05-agent-broker-uplink
level: task
title: "WS-05: Agent→broker uplink — heartbeat, events, health over WS"
short_code: "BROKKR-T-0160"
created_at: 2026-05-23T02:12:35.870220+00:00
updated_at: 2026-05-23T04:20:34.675559+00:00
parent: BROKKR-I-0019
blocked_by: []
archived: false

tags:
  - "#task"
  - "#phase/completed"


exit_criteria_met: false
initiative_id: BROKKR-I-0019
---

# WS-05: Agent→broker uplink — heartbeat, events, health over WS

**Parent**: [[BROKKR-I-0019]] · **ADR**: [[BROKKR-A-0008]]

## Objective

Move agent→broker traffic (heartbeat, agent events, health reports) onto the WS channel when connected. REST endpoints stay available — agent uses REST automatically when WS is down.

## Acceptance Criteria

## Acceptance Criteria

- [x] Heartbeat tick sends `Heartbeat` over WS when connected, falls back to REST POST when not (`broker::send_heartbeat` short-circuits on WS-up + try_send-ok)
- [x] Agent events sent via `AgentEvent` over WS; REST `POST /agents/{id}/events` retained as fallback (`send_success_event` / `send_failure_event`)
- [x] Agent health sent via `AgentHealth` over WS; REST batch endpoint retained as fallback (`send_health_status` — falls back if any line in the batch can't fit on the WS lane)
- [x] Per-channel proof: broker `reader_task::dispatch_uplink` writes to the same DAL paths the REST handlers do (agents.record_heartbeat, agent_events.create, deployment_health.upsert); the WS-05 integration test asserts those rows appear after an uplink frame, proving REST is bypassed
- [x] Integration test for round-trip end-to-end persistence via WS (`ws_uplink_persists_heartbeat_event_and_health`)
- [ ] Kill-WS / restore-WS observable-rate test deferred to WS-06 chaos suite (the BrokerSink-style branch already short-circuits cleanly via `WsUplink::is_up()`; chaos tests prove the rate behaviour under faults)

## Implementation Notes

- **Approach**: `WsUplink` (a cheap clonable bundle of `watch<WsState> + mpsc::Sender<WsMessage>`) is the single decision point. Emitters in `broker.rs` take `Option<&WsUplink>`; when present and state is Up they `try_send` and return early on success.
- **Dependencies**: WS-03 (uplink handle), WS-02 (broker-side dispatch needed `dispatch_uplink` to write to DAL).
- **Risk**: dual-emission during WS-up/down transitions — handled by checking `is_up()` *and* `try_send()` returning Ok. A frame that misses the lane falls through to REST in the same call; there's no dual emission.

## Status Updates

**2026-05-23** — Done on branch `feat/i-0019-ws-broker-agent-channel`.

- New `WsUplink` type in `broker_ws.rs` exposing `is_up()` + `try_send(msg) -> Result<(), WsMessage>` (returns the unsent message so callers don't need to clone defensively).
- `broker.rs` emitters extended with `Option<&WsUplink>`:
  - `send_heartbeat` — builds `WsMessage::Heartbeat` and short-circuits REST when WS path succeeds
  - `send_success_event` / `send_failure_event` — `synth_agent_event` helper assembles the wire body from the same `NewAgentEvent` shape REST uses
  - `send_health_status` — one `AgentHealth` frame per update; falls back to REST batch endpoint if **any** frame back-pressures, to keep the batch atomic on the broker side
- Broker reader_task in `ws/handler.rs` now `dispatch_uplink`s frames into the DAL:
  - `Heartbeat` → `agents.record_heartbeat`
  - `AgentEvent` → `agent_events.create` (body `agent_id` must match the connection's authenticated id; mismatched frames are warned and dropped)
  - `AgentHealth` → `deployment_health.upsert`
  - `K8sEvent` / `PodLogLine` / `LogGap` are accepted but not persisted (WS-09)
  - Broker→agent shapes coming back the wrong way are logged + dropped
- Agent main (`cli/commands.rs`): `let ws_client = broker_ws::spawn(&config); let ws_uplink = ws_client.uplink();` — heartbeat / success / failure / health all pass `Some(&ws_uplink)`. Force-rest is honoured automatically (state never becomes Up).
- Test (`tests/integration/api/ws.rs::ws_uplink_persists_heartbeat_event_and_health`): real WS round-trip; agent sends Heartbeat → assert `last_heartbeat` populated; sends AgentEvent → assert `agent_events` row exists with matching fields; sends AgentHealth → assert `deployment_health` row exists. 7/7 WS integration tests now green.
- Existing agent integration tests updated to pass `None` for the new `ws_uplink` param (REST-only path remains the default for those tests).

**Deferred**:
- `brokkr_ws_messages_total{direction, type}` metric instrumentation lands in WS-13.
- The literal "kill WS, observe REST resumes" rate-flap test lands in WS-06's chaos suite — the structural mechanism (single decision in `try_ws_send`) is in place here.
- Selector-based work-order target push (label/annotation) is still REST-polled by agents; the inverse uplink path is fine.