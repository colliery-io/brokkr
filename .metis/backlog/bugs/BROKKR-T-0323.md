---
id: ws-e2e-scenarios-never-run
level: task
title: "Four WebSocket e2e scenarios exist and are never executed by any workflow"
short_code: "BROKKR-T-0323"
created_at: 2026-07-31T18:15:00+00:00
updated_at: 2026-07-31T18:15:00+00:00
parent: 
blocked_by: []
archived: false

tags:
  - "#task"
  - "#bug"
  - "#phase/backlog"


exit_criteria_met: false
initiative_id: NULL
---

# Four WebSocket e2e scenarios exist and are never executed by any workflow

## Objective

Make the WebSocket e2e scenarios run somewhere, or record why they should not.

`tests/e2e/src/main.rs:104-135` reaches `ws-smoke`, `ws-chaos`, `ws-workorders` and `ws-telemetry` **only** through the single-scenario dispatch path, which requires `E2E_SCENARIO` to be set (`angreal tests e2e --scenario <name>`). The default walkthrough runs Parts 1–9 and none of them.

**Nothing sets that flag.** `.github/workflows/e2e_tests.yml:57` runs `angreal tests e2e --skip-docker`, and that workflow is the only e2e entry point — used by both `nightly.yml` and `release.yml`. So these four scenarios have never run in CI.

## Backlog Item Details

### Type
- [x] Bug - Production issue that needs fixing

### Priority
- [x] P2 - Medium (no product defect; it is four tests' worth of coverage that reads as real and is not)

## 2026-07-31 — how it surfaced, and proof it matters

Found while fixing the registration-consent fallout in BROKKR-T-0287 (PRs #91, #92). Auditing every targeting call in the suite turned up `test_ws_smoke` calling `add_agent_target` on a generator the agent was never registered with — which T-0246 refuses at creation with 403 `agent_not_registered`.

**That scenario had been broken and silent.** It was fixed in #92 along with the two that were failing loudly, and verified by running `angreal tests e2e --scenario ws-smoke` by hand. Nothing else would ever have caught it: the assertion cannot fail if the code never executes.

The other three are still unexercised and may be broken right now — `ws-chaos`, `ws-workorders` and `ws-telemetry` were not run during that work, and their last verified-good state is unknown.

This is the same defect class as BROKKR-T-0308's inert chart values, BROKKR-T-0313's `> /dev/null` helm checks, and the Live/Paused toggle in BROKKR-T-0319: something that reads as coverage, is counted as coverage, and executes nothing. The pattern is common enough in this repo to be worth naming.

### Why they were probably split out

Not established, and worth finding out before changing anything. The plausible reasons are cost and flakiness: `ws-chaos` drives toxiproxy to sever connections, `ws-telemetry` applies a failing pod to real k3s and polls for events, and `ws-workorders` combines work orders with a WS sever. Those are slower and more failure-prone than Parts 1–9, which is a legitimate reason to keep them out of a gating path — but not a reason to run them *nowhere*.

## Decisions needed

1. **Where should they run?** Nightly is the natural home — it already tolerates a long runtime and files an issue on failure. Adding them to the default walkthrough would put them on the release path too, which may be more than is wanted given the flakiness above.
2. **Should they gate the release?** If not, they need somewhere that is watched, or this recurs.
3. **Whatever is chosen, first run all four as they stand** and record the result. Three of them have an unknown baseline; adopting them into a workflow without knowing that is how a suite gets disabled again a week later.

## Acceptance Criteria

- [ ] All four scenarios are run as-is and their current pass/fail state recorded here.
- [ ] Any that fail are fixed, or explicitly quarantined with a reason and a ticket.
- [ ] The four run automatically somewhere, or a decision not to run them is recorded with reasoning.
- [ ] If they land in a workflow, verify by observation that a deliberately broken one fails that workflow — the check this repo keeps skipping.
- [ ] `run_scenario_allow_fail!` is considered for the genuinely external-dependency ones, matching how Part 5b handles ttl.sh + Shipwright.

## Status Updates

*To be added during implementation*
