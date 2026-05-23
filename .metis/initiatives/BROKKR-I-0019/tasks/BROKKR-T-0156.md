---
id: ws-01-shared-ws-message-protocol
level: task
title: "WS-01: Shared WS message protocol (typed enum, serde, SDK-version pinned)"
short_code: "BROKKR-T-0156"
created_at: 2026-05-23T02:12:29.574029+00:00
updated_at: 2026-05-23T02:29:50.178187+00:00
parent: BROKKR-I-0019
blocked_by: []
archived: false

tags:
  - "#task"
  - "#phase/completed"


exit_criteria_met: false
initiative_id: BROKKR-I-0019
---

# WS-01: Shared WS message protocol (typed enum, serde, SDK-version pinned)

**Parent**: [[BROKKR-I-0019]] · **ADR**: [[BROKKR-A-0008]]

## Objective

Define the canonical wire schema for the internal broker↔agent WebSocket channel. A single typed Rust enum, serde-serialized, used identically by `brokkr-broker` and `brokkr-agent`. Pinned to the same version as the generated SDK so REST and WS share types.

## Acceptance Criteria

- [x] Module / lightweight crate exposes a `WsMessage` enum with v1 variants: `WorkOrder`, `TargetChanged`, `StackChanged`, `Heartbeat`, `AgentEvent`, `AgentHealth`, `K8sEvent`, `PodLogLine`, `LogGap`
- [x] Each variant carries the same body type the REST/SDK exposes (re-exported / aliased, not duplicated)
- [x] Serde round-trip tests for every variant
- [x] Backward-compat test fixture (golden JSON) so accidental schema breaks fail CI
- [x] Crate version pinned to broker/SDK version per [[project_release_versioning]]

## Implementation Notes

- **Approach**: prefer a new internal crate (e.g. `brokkr-wire`) re-exporting from `brokkr-models` and the SDK types; broker and agent both depend on it. Avoids putting WS-specific types in the public SDK.
- **Dependencies**: none — foundational, unblocks WS-02, WS-03, WS-04, WS-05, WS-07, WS-08.
- **Risk**: schema drift between WS and REST. Mitigated by sharing types at the source and the golden-fixture test.

## Status Updates

**2026-05-22** — Done on branch `feat/i-0019-ws-broker-agent-channel`.

- Created `crates/brokkr-wire` (`version = "0.4.2"`, `publish = false`) — internal-only, not part of the public SDK.
- `WsMessage` enum with all 9 v1 variants. Tagged JSON: `{"type": "...", "body": {...}}`, snake_case tags.
- Body types for `WorkOrder` / `AgentTarget` / `Stack` / `AgentEvent` / `DeploymentHealth` re-exported from `brokkr-models` (no duplication; one definition for REST and WS). New types `Heartbeat`, `K8sEvent`, `ObjectRef`, `PodLogLine`, `LogGap`, `GapReason` defined locally — no REST equivalents.
- `WIRE_VERSION` constant pulled from `CARGO_PKG_VERSION` (lockstep with broker/SDK per `project_release_versioning`).
- Tests in `tests/golden.rs`:
  - `every_variant_roundtrips` — serialize → deserialize → re-serialize equals
  - `variant_tags_are_snake_case` — pins the exact on-wire tag for every variant
  - `golden_fixture_matches_current_serialization` — `tests/fixtures/ws_message_v1.json` is the canonical v1 wire shape; drift requires intentional fixture update + release bump
  - `wire_version_is_pinned`
- All 4 tests pass: `cargo test -p brokkr-wire --test golden` → 4 passed; 0 failed.
- `brokkr-wire` added to `.angreal/task_tests.py` unit-tests allowlist.

**Follow-ups for downstream tasks**:
- WS-02 (broker endpoint) and WS-03 (agent client) depend on `brokkr-wire` via path.
- If WS-07/WS-08 need richer fields on `K8sEvent` / `PodLogLine`, extending these structs is a wire change — bump the fixture and release version per the stance.