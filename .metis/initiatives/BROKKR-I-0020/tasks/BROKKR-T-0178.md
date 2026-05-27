---
id: c1-browser-side-live-tail-adr
level: task
title: "C1: Browser-side live tail — ADR + implementation"
short_code: "BROKKR-T-0178"
created_at: 2026-05-24T12:56:51.000000+00:00
updated_at: 2026-05-24T12:56:51.000000+00:00
parent: BROKKR-I-0020
blocked_by: []
archived: false

tags:
  - "#task"
  - "#phase/completed"


exit_criteria_met: true
initiative_id: BROKKR-I-0020
---

# C1: Browser-side live tail — ADR + implementation

## Parent Initiative

[[BROKKR-I-0020]]

## Objective

WS-12 had to defer browser-side live tail because `new WebSocket()` can't
set custom headers, so the PAK can't ride on `Authorization`. ui-slim
currently shows a stub. Decide between the two viable approaches, record
the decision in an ADR amendment or new ADR, and implement.

## Acceptance Criteria

- [ ] Short ADR (ADR-0009 or amendment to ADR-0008) recording the chosen
      approach and rejected alternative, with rationale. Options:
  - (a) Broker accepts PAK via `Sec-WebSocket-Protocol` header — the
        standard browser workaround. Bug: it's a slight abuse of the
        subprotocol mechanism
  - (b) ui-slim adds a server-side SSE proxy endpoint that opens the
        WS upstream with the PAK and re-streams to the browser as SSE
- [ ] ADR transitioned to `decided` with human sign-off
- [ ] Implementation lands the chosen approach
- [ ] ui-slim's live-tail stub replaced with an actual live tail view
      that mirrors the WS-12 telemetry tabs (events + logs)
- [ ] Integration test (browser-side or Node-side equivalent) exercises
      the new path end-to-end
- [ ] Operator note in `docs/src/explanation/internal-ws-channel.md`
      updated to reflect new browser-supported flow

## Implementation Notes

### Technical Approach

Recommend (a) — `Sec-WebSocket-Protocol` — for two reasons: it's the
industry-standard workaround (Kubernetes API server does exactly this),
and it keeps the broker as the single source of truth for auth. Approach
(b) doubles the proxy surface area for what's a transport-level limitation.

But: log the trade-off honestly. (a) means writing a tiny bit of
non-obvious auth code on the broker side. (b) keeps the broker's
auth surface unchanged at the cost of a new proxy responsibility for ui-slim.

### Dependencies

None.

### Risk Considerations

- Whichever path, the change must NOT break the existing Node-side WS
  contract test (which uses `headers: { Authorization }` from the `ws`
  package) — both auth paths should be supported on the broker
- If we go with (a), reject unknown subprotocols cleanly so we don't
  open new protocol-negotiation surface area

## Status Updates

### 2026-05-26 — Done (approach a: Sec-WebSocket-Protocol)

Human-approved approach **(a)** — broker accepts the PAK via
`Sec-WebSocket-Protocol`. Decision recorded as an ADR-0008 amendment
("Browser WS auth via Sec-WebSocket-Protocol"); the rejected SSE-proxy
alternative is documented there.

- **Broker** (`ws/subscribe.rs`): `ws_subprotocol_auth` middleware (outermost
  on the subscribe route) lifts `brokkr.pak.<PAK>` from
  `Sec-WebSocket-Protocol` into an `Authorization: Bearer` header **only when
  none is present**, so header-based callers are untouched. `live_upgrade`
  echoes back only the non-secret `brokkr.v1` marker (never the PAK).
- **ui-slim**: `openStackLiveStream` now returns a real `WebSocket` with the
  subprotocol auth; the Telemetry panel gained a **Go Live** toggle that
  streams `k8s_event` / `pod_log_line` frames into the existing events/logs
  tabs and renders `log_gap` markers, with capped in-memory buffers.
- **Docs**: "Browser live tail (subprotocol auth)" operating note added.

**Tests (green):** broker integration `live_subscription_authenticates_via_subprotocol`
(subprotocol auth end-to-end + marker echoed, no Authorization header) and
`live_subscription_subprotocol_with_bad_pak_is_rejected` (401). The existing
header-based `live_subscription_forwards_agent_telemetry_to_subscribers` still
passes — **no regression on the Node `ws` contract path**. Full
`api::ws::live_subscription` module: 4 passed.

**Note:** `npm run build`/lint of ui-slim wasn't run this session (per
direction); the broker integration tests prove the wire/auth path end-to-end
and the React changes are standard hooks — a build/lint is the trivial
pre-ship confirmation.
