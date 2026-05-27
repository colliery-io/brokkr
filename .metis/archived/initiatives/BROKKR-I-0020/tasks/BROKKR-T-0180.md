---
id: c3-annotation-lookup-bounded
level: task
title: "C3: annotation_lookup bounded NotOurs LRU cache"
short_code: "BROKKR-T-0180"
created_at: 2026-05-24T12:56:55+00:00
updated_at: 2026-05-24T12:56:55+00:00
parent: BROKKR-I-0020
blocked_by: []
archived: true

tags:
  - "#task"
  - "#phase/completed"


exit_criteria_met: true
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

### 2026-05-26 — Done

Replaced the unbounded `HashMap`-backed `UidCache` with a bounded
`lru::LruCache` (added `lru = "0.12"` to brokkr-agent). TTL behavior preserved
(5min); an entry read past TTL is treated as a miss AND evicted so the size
accounting stays honest. `get` now takes `&mut self` (LRU promotes recency),
so `resolve_stack` takes the write lock for the lookup — cheap relative to the
API call a miss triggers.

- Cap is configurable: `agent.kube_event_uid_cache_cap` (`brokkr-utils`
  config, `Option<usize>`), default `DEFAULT_UID_CACHE_CAP = 10_000`. Threaded
  through `kube_events::spawn`. Commented entry added to `default.toml`.
- Docs: "Tuning the kube-events UID cache" subsection added to
  `internal-ws-channel.md` (the bound, the default, the memory-vs-API
  trade-off).

**Tests** (in `kube_events.rs`, all green; agent unit 64 total):
- `cache_stays_bounded_under_high_unique_churn` — 50_000 unique UIDs against a
  10_000 cap: asserts `len() <= cap` on every insert, ends pinned at exactly
  10_000 (the old HashMap would have grown to 50_000), and the API-call count
  equals 50_000 (unique churn can't be helped — the win is bounded memory).
- `cache_serves_hot_set_without_re_hitting_the_api` — a hot set within cap,
  looked up 3× over, costs exactly one API call per UID.
- Existing 3 cache tests preserved (no regression).

Full workspace unit suite green (64 agent / 97 broker / 128 models / 24 utils).