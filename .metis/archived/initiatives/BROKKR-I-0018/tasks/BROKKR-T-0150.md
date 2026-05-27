---
id: r2-f-update-sdk-docs-for-published
level: task
title: "R2-F: Update SDK docs for published packages"
short_code: "BROKKR-T-0150"
created_at: 2026-05-15T22:30:00+00:00
updated_at: 2026-05-16T03:53:52.819731+00:00
parent: BROKKR-I-0018
blocked_by: [BROKKR-T-0149]
archived: true

tags:
  - "#task"
  - "#phase/completed"


exit_criteria_met: false
initiative_id: BROKKR-I-0018
---

# R2-F: Update SDK docs for published packages

## Parent Initiative

[[BROKKR-I-0018]]

## Objective

Flip the SDK docs from "not yet published" placeholder language to real install instructions pointing at the registries, and add a brief note on the lockstep version compatibility model.

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [x] `docs/src/how-to/sdks/rust.md` install: `cargo add brokkr-client` primary, in-tree path-dep as footnote.
- [x] `docs/src/how-to/sdks/python.md` install: `pip install brokkr-client` / `uv pip install brokkr-client` primary, editable path-dep as footnote.
- [x] `docs/src/how-to/sdks/README.md`: added a per-language install/import table at the top of "Getting started" and a "Versioning and compatibility" section explaining lockstep.
- [x] TS install reference uses `@colliery-io/brokkr-client` (no in-tree TS SDK README exists; the docs-site table covers it).
- [x] All "not yet published" hedge language removed from the docs.
- [x] `sdks/python/brokkr/README.md` also updated with a `pip install brokkr-client` block + a one-liner pointing at lockstep versioning.
- [x] `angreal docs build` clean (only pre-existing API-docs HTML-tag warnings unrelated to this change).

## Implementation Notes

### Technical Approach

1. Three short doc edits; no code changes.
2. The lockstep note should be one paragraph, not a section essay. Pointer to BROKKR-I-0018's design doc is enough for anyone who wants more.
3. After this task lands, BROKKR-I-0018 is done.

### Dependencies

- Hard: [[BROKKR-T-0149]] (must have a real version published to point at).

### Risk Considerations

- Doc rot. Same risk as the original SDK docs. The mitigation remains: if you change SDK structure, sweep `docs/src/how-to/sdks/` in the same PR.

## Status Updates

*To be added during implementation*