---
id: console-playwright-e2e-scope
level: task
title: "Console: Playwright e2e scope-selector scene and fixture updates"
short_code: "BROKKR-T-0274"
created_at: 2026-07-03T00:09:13.550494+00:00
updated_at: 2026-07-03T03:48:07.586637+00:00
parent: BROKKR-I-0032
blocked_by: [BROKKR-T-0273]
archived: false

tags:
  - "#task"
  - "#phase/completed"


exit_criteria_met: false
initiative_id: BROKKR-I-0032
---

# Console: Playwright e2e scope-selector scene and fixture updates

## Parent Initiative

[[BROKKR-I-0032]]

## Objective

Update the console's Playwright screenshot/e2e harness (`crates/brokkr-web/web-e2e/shots.mjs`) for the new auth flow (no PAK paste) and add a scope-selector scene: select a tenant, assert the Fleet view narrows.

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [x] The harness runs against a token-less dev server, so the `localStorage["brokkr_pak"]` seed stays (documented as the dev-server path); against an embed-ui broker the injected token makes the seed unnecessary
- [x] Fixture data includes two named tenants (`team-payments`, `team-ingest`) with ids aligned to `STACKS.generator_id` and split fleet fixtures
- [x] New scenes: `scope-selector` (selector visible, "All") and `fleet-scoped` (team-payments selected → only its 2 agents, KPIs recomputed); both screenshots visually verified
- [x] Existing 14 scenes still render (full run regenerated all shots)
- [x] Run locally before completing — done (three full runs during verification)

## Implementation Notes

### Technical Approach
- `crates/brokkr-web/web-e2e/shots.mjs` — existing scenes and any localStorage seeding live here; follow its scene conventions.
- Tenant fixtures: seed via broker API (create generators + agents + registrations + stacks) the same way existing fixtures are seeded.

### Dependencies
BROKKR-T-0273 (selector must exist).

## Status Updates

**2026-07-02 — implemented, verification pending**
- `shots.mjs`: `PAKS` fixture (team-payments / team-ingest, ids aligned with `STACKS.generator_id`); mock router now query-aware (exact `path?query` match wins, falls back to bare path); two new scenes — `scope-selector` (selector visible on Fleet) and `fleet-scoped` (selectOption team-payments → narrowed fleet fixture); `brokkr_scope` cleared after each scene for independence; localStorage PAK seed kept (dev-server path).
- PENDING: run `trunk serve` + `node shots.mjs` to regenerate screenshots (blocked on Bash availability).

**2026-07-03 — VERIFIED**
- Full suite run against `trunk serve` on :9080 (16 scenes). The first run exposed the gloo-net trailing-`&` bug (see T-0273) — after the fix, `fleet-scoped.png` correctly shows only team-payments' agents. Harness hardened: query-keyed mocks normalize trailing separators, and unmocked `/paks` returns `[]` (selector hidden) instead of 404 noise on legacy scenes.
- Note: the harness's "CONSOLE ERRORS" 404 spam on legacy scenes is pre-existing (every scene boots on Overview, which fetches endpoints only the overview scene mocks) — unchanged by this work.