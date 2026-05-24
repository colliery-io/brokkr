---
id: c3-annotation-lookup-bounded
level: task
title: "C3: annotation_lookup bounded NotOurs LRU cache"
short_code: "BROKKR-T-0180"
created_at: 2026-05-24T12:56:55.000000+00:00
updated_at: 2026-05-24T12:56:55.000000+00:00
parent: BROKKR-I-0020
blocked_by: []
archived: false

tags:
  - "#task"
  - "#phase/todo"


exit_criteria_met: false
initiative_id: BROKKR-I-0020
---

# C3: annotation_lookup bounded NotOurs LRU cache

## Parent Initiative

[[BROKKR-I-0020]]

## Objective

`kube_events::annotation_lookup` does one dynamic-API call per UID
cache-miss. In a cluster with high churn of unmanaged objects (any
non-Brokkr-deployed pod), that's an API call per Event. The existing
`UidCache` caches `Owned(stack_id)` and `NotOurs` entries with a 5min
TTL, but it's unbounded — under high churn it could grow without
limit. Replace with a bounded LRU so the miss path is O(1) for
non-managed UIDs after first lookup and the cache size stays capped.

## Acceptance Criteria

- [ ] `UidCache` (or successor) bounded with `lru::LruCache` (or equivalent),
      cap configurable, default 10_000 entries
- [ ] Cache TTL behavior preserved (5min)
- [ ] New unit test in `kube_events.rs` that hammers the lookup with 50_000
      unique non-managed UIDs and asserts:
  - Cache size never exceeds the cap
  - Dynamic-API call count stays bounded (≤ 50_000 + small fudge for cap
    evictions — should be exactly 50_000 if cap ≥ that many)
- [ ] No regression in existing kube_events tests
- [ ] Brief perf note in `docs/src/explanation/internal-ws-channel.md`
      mentioning the bound and how to tune it

## Implementation Notes

### Technical Approach

- `lru` crate (already in cargo.io ecosystem, MIT, low dep count) wraps
  `HashMap` with LRU eviction; pairs cleanly with the existing TTL pattern
  (store `(value, inserted_at)` tuples)
- Hot-path correctness matters more than performance here: when we evict
  an `Owned(stack_id)` entry, the next event for that UID does an API
  call — annoying but correct
- The cap should be exposed via `brokkr-utils` config so operators can
  tune for large clusters

### Dependencies

None.

### Risk Considerations

- LRU eviction during a burst of managed-object events could spike API
  calls; the cap is a balance between memory and API pressure. Document
  this trade-off
- 10_000 is a reasonable default for clusters with up to ~10k managed
  pods; larger clusters should bump it

## Status Updates

*To be added during implementation*
