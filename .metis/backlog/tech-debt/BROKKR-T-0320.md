---
id: remove-console-localstorage-pak-override
level: task
title: "Remove the console's localStorage PAK override: a persistent admin credential with no remaining justification"
short_code: "BROKKR-T-0320"
created_at: 2026-07-30T05:00:00+00:00
updated_at: 2026-07-30T05:00:00+00:00
parent: 
blocked_by: []
archived: false

tags:
  - "#task"
  - "#tech-debt"
  - "#phase/completed"


exit_criteria_met: false
initiative_id: NULL
---

# Remove the console's localStorage PAK override: a persistent admin credential with no remaining justification

## Objective

Delete `api::pak()` and the `localStorage["brokkr_pak"]` override, leaving the broker-injected read-only token as the console's only ambient credential.

## Backlog Item Details

### Type
- [x] Tech Debt - Code improvement or refactoring

### Priority
- [x] P2 - Medium (not an escalation path — it required a valid admin PAK — but it kept one at rest in browser storage indefinitely, and 0.9.0 is the right moment to drop it)

## 2026-07-30 — what it was, and why it stopped being justified

Introduced deliberately in BROKKR-I-0032: *"the `localStorage["brokkr_pak"]` override remains supported for full-write operator use."* Nothing in the interface ever wrote it — devtools only, and `how-to/operator-console.md` said so.

**Not a network-reach escalation.** A caller already needed a valid admin PAK, so reaching the console granted nothing extra. Worth stating plainly, because the risk is easy to overstate.

**What it actually was:** an admin PAK persisted indefinitely in browser storage, readable by any script on the origin — the same origin that serves both the console bundle and the API — with no expiry, no clearing, and nothing in the UI indicating it was set.

Two things made it worth removing rather than tolerating:

1. **Precedence.** `token()` was `pak().or_else(injected_token)`, so a stored PAK won for *every* request. Setting it silently promoted the whole console from read-only to full admin write — not one action, everything, until someone cleared it by hand.
2. **BROKKR-T-0318 established the opposite pattern in the same crate.** The tenants panel takes an admin PAK per action, holds it in memory, and clears it on both the success and error paths. Keeping both meant `api.rs` shipped a persistent credential store while `tenants.rs` argued credentials must never be stored.

**`security-model.md` never mentioned it** — the document defining the console's credential model was silent on its only write-capable path. That omission was arguably the worse half of the problem.

### All three justifications were tested, not assumed

| Claim | Verdict |
|---|---|
| "The only auth path under `trunk serve`" | **False.** No proxy is configured, so `/api/v1/*` 404s under `trunk serve` — there is no broker to authenticate to. The console's only dev data source is the Playwright route mocks. |
| "The e2e harness needs the seed" | **False.** Verified by deleting the seed and running the full 23-scene suite: no nav failures, every view rendered its fixtures. The mocks fulfil regardless of headers — the harness's own comment already conceded *"the mock ignores it"*. |
| "Full-write operator use" | **Superseded** by BROKKR-T-0318, for the one write anyone wanted. |

## Acceptance Criteria

- [x] `api::pak()` is gone and `token()` returns only the injected token.
- [x] The e2e harness runs green with no seeded credential.
- [x] `how-to/operator-console.md` no longer describes the override as available.
- [x] `security-model.md` documents the console's credential paths, including that the override existed and was removed.
- [x] `clippy --target wasm32-unknown-unknown`, `trunk build`, and `angreal docs build` all pass.
- [ ] Release notes for 0.9.0 record the removal (below).

## Status Updates

**2026-07-30 — DONE.** `pak()` deleted; `token()` is now `injected_token()`. Harness seed removed and the suite re-run green. Docs updated in three places, including the `security-model.md` gap and a new "Step 6: Create a Tenant" section in `operator-console.md` — the panel from BROKKR-T-0318 was otherwise undocumented.

**This reverses an explicit I-0032 decision, and that is the point** — the justification lapsed rather than being overlooked. Recorded so nobody reinstates it by finding the old rationale.

**Behaviour removal for the 0.9.0 notes:** an operator who today drives writes from the console by putting an admin PAK in `localStorage["brokkr_pak"]` loses that. There is no in-product replacement for arbitrary writes, by design; tenant creation is served by the per-action prompt. Since 0.9.0 already carries breaking chart-interface changes, this is the right release to land it in — but it must be listed, because the mechanism is invisible enough that an affected operator would otherwise see the console silently stop being write-capable.