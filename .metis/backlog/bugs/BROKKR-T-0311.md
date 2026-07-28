---
id: every-cross-section-overview-link
level: task
title: "Every cross-section overview link in the built book 404s: mdBook renders README.md as index.html but links point at README.html"
short_code: "BROKKR-T-0311"
created_at: 2026-07-28T16:14:06.181848+00:00
updated_at: 2026-07-28T16:14:06.181848+00:00
parent: 
blocked_by: []
archived: false

tags:
  - "#task"
  - "#phase/backlog"
  - "#bug"


exit_criteria_met: false
initiative_id: NULL
---

# Every cross-section overview link in the built book 404s: mdBook renders README.md as index.html but links point at README.html

## Objective

Fix a systemic link defect in the published book. mdBook renders each section's `README.md` as `index.html`, but source links written as `../tutorials/README.md` are rewritten to `../tutorials/README.html` — a file that does not exist. So **every cross-section overview link in the book is dead**, including the navigational "start here" links between getting-started, tutorials, how-to, reference, and explanation.

`angreal docs build` does **not** catch this: mdBook only validates that the source `.md` target exists, which it does. The breakage appears solely in the rendered HTML, which is why a book that builds green has been shipping 404s.

Found 2026-07-28 during BROKKR-T-0293. New links added during the 2026-07 documentation work deliberately follow the existing convention rather than diverging in isolated files, so a single consistent fix can correct all of them at once — do not fix these piecemeal.

## Backlog Item Details

### Type
- [x] Bug - Production issue that needs fixing

### Priority
- [x] P2 - Medium (navigation only — no wrong information, but the book's cross-section wayfinding is broken for every reader)

### Impact Assessment
- **Affected Users**: every reader of the published book who follows an overview link between sections.
- **Reproduction**: `angreal docs build`, then open `docs/book/getting-started/index.html` and click through to any other section's overview.

## Acceptance Criteria

- [ ] Cross-section overview links resolve in the built HTML. Pick one convention and apply it everywhere (linking to the directory, e.g. `../tutorials/`, is the usual mdBook answer — verify against this book's `book.toml` and mdBook version before committing to it).
- [ ] A link check runs over the **built HTML**, not the markdown source, so this cannot regress silently. Consider wiring it into `angreal docs build` — the absence of such a check is the actual root cause here.
- [ ] `angreal docs build` still passes and the SUMMARY navigation is unaffected.

## Status Updates

*To be added during implementation*
