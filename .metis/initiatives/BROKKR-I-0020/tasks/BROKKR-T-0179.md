---
id: c2-ratelimiter-api-cleanup-rename
level: task
title: "C2: RateLimiter API cleanup — rename Pass to Drop, add DropAndGap"
short_code: "BROKKR-T-0179"
created_at: 2026-05-24T12:56:53.000000+00:00
updated_at: 2026-05-24T12:56:53.000000+00:00
parent: BROKKR-I-0020
blocked_by: []
archived: false

tags:
  - "#task"
  - "#phase/todo"


exit_criteria_met: false
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

*To be added during implementation*
