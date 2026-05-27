---
id: c2-ratelimiter-api-cleanup-rename
level: task
title: "C2: RateLimiter API cleanup — rename Pass to Drop, add DropAndGap"
short_code: "BROKKR-T-0179"
created_at: 2026-05-24T12:56:53+00:00
updated_at: 2026-05-24T12:56:53+00:00
parent: BROKKR-I-0020
blocked_by: []
archived: true

tags:
  - "#task"
  - "#phase/completed"


exit_criteria_met: true
initiative_id: BROKKR-I-0020
---

# C2: RateLimiter API cleanup — rename Pass to Drop, add DropAndGap

## Parent Initiative

[[BROKKR-I-0020]]

## Objective

The `pod_logs::RateLimiter` API currently returns `Allowance::Pass` for
silently-dropped-after-first-drop log lines, which is exactly the
opposite of what "Pass" suggests. The test even has an explanatory
comment because of how confusing it reads. Rename to honest variants;
on-the-wire `LogGap{RateLimit}` frame stays exactly as-is.

## Acceptance Criteria

## Acceptance Criteria

- [ ] `Allowance` enum in `crates/brokkr-agent/src/pod_logs.rs` updated:
  - `Allow` — within budget, ship the line
  - `Drop` — over budget, silently drop (subsequent-drops-in-window case)
  - `DropAndGap` — over budget AND this is the first drop in the window,
                   so we also emit a `LogGap{RateLimit}` frame
- [ ] All callers in `pod_logs.rs` updated to match the new enum
- [ ] All RateLimiter unit tests updated; the awkward "Pass-but-still-drop"
      explanatory comment goes away
- [ ] On-wire behavior unchanged: `LogGap{RateLimit, dropped: n}` frames
      emit at the same boundaries with the same payload shape — verify
      via existing wire golden test
- [ ] No regression in agent unit/integration suites

## Implementation Notes

### Technical Approach

This is a pure rename + variant split. The actual rate-limiting math
doesn't change. The "first drop in window" detection already exists
implicitly — explicitly model it in the enum

### Dependencies

None.

### Risk Considerations

- This touches code that's already shipped on the branch; make sure the
  contract tests (which only assert wire shape) stay green
- Don't accidentally change the gap counter semantics — the `dropped: n`
  field on the wire is cumulative-since-last-gap-frame and must remain so

## Status Updates

### 2026-05-26 — Rename done; the rename surfaced (and fixed) a real bug

Split `Allowance` into honest variants: `Allow` (ship), `Drop` (over budget,
silent), `DropAndGap(n)` (over budget + first drop of window → emit one
`LogGap{RateLimit}`). Callers and the two RateLimiter unit tests updated; the
awkward "Pass-but-still-drop" comment is gone.

**Finding (real bug, not just cosmetics):** the old code returned
`Allowance::Pass` for *both* under-budget lines and over-budget
subsequent-drops, and `tail_container`'s `Pass` arm **ships the line**. So
before this change, only the *first* over-budget line per 1s window was
actually dropped (turned into a gap) — every further over-budget line was
**shipped**, defeating the 100 lines/sec ceiling the module documents. The
module doc says "over-rate lines are dropped"; the code didn't. Splitting
`Pass`→`Allow`/`Drop` makes the `Drop` arm a no-op (no `try_send`), so the
ceiling is now actually enforced. This is the kind of latent bug the I-0020
cleanup pass is meant to catch.

**Wire contract preserved:** `LogGap{RateLimit}` frames emit at the exact same
boundary (first drop of a window) with the same payload (`dropped_count: 1`,
`reason: RateLimit`) — `DropAndGap` is unchanged. The fix only stops shipping
the *extra* `PodLogLine` frames that should never have gone out. No brokkr-wire
change (the enum is private to the agent).

**Tests:** agent unit green (62) incl. the updated RateLimiter tests. E2e
`ws-telemetry` green (Allow path end-to-end — chatty pod logs still reach the
broker; events too): `1 passed, 0 failed`.