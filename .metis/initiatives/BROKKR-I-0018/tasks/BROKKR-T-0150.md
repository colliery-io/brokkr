---
id: r2-f-sdk-docs-published
level: task
title: "R2-F: Update SDK docs for published packages"
short_code: "BROKKR-T-0150"
created_at: 2026-05-15T22:30:00.000000+00:00
updated_at: 2026-05-15T22:30:00.000000+00:00
parent: BROKKR-I-0018
blocked_by: [BROKKR-T-0149]
archived: false

tags:
  - "#task"
  - "#phase/todo"


exit_criteria_met: false
initiative_id: BROKKR-I-0018
---

# R2-F: Update SDK docs for published packages

## Parent Initiative

[[BROKKR-I-0018]]

## Objective

Flip the SDK docs from "not yet published" placeholder language to real install instructions pointing at the registries, and add a brief note on the lockstep version compatibility model.

## Acceptance Criteria

- [ ] `docs/src/how-to/sdks/rust.md` "Install" section uses `cargo add brokkr-client` (not the local path dep) as the primary instruction. The path-dep variant stays as a "for in-tree workspace consumers" footnote.
- [ ] `docs/src/how-to/sdks/python.md` "Install" section uses `pip install brokkr-client` (or `uv pip install brokkr-client`). Same footnote pattern for in-tree.
- [ ] `docs/src/how-to/sdks/README.md` adds a short "Versioning & compatibility" section: SDK versions track broker versions in lockstep; an SDK at `0.3.x` is the canonical client for broker `0.3.x`; mixing major versions is unsupported.
- [ ] TypeScript SDK install reference points to `@colliery-io/brokkr-client` (the README under `sdks/typescript/brokkr-client/` and any mention from the docs site).
- [ ] All "this is not yet on $REGISTRY" hedge language removed.
- [ ] `angreal docs build` still clean.

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
