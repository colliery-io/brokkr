---
id: integrate-plissken-for-auto
level: task
title: "Integrate plissken for auto-generated Rust API reference docs"
short_code: "BROKKR-T-0129"
created_at: 2026-03-25T20:50:57.370360+00:00
updated_at: 2026-03-25T21:01:34.445117+00:00
parent: 
blocked_by: []
archived: false

tags:
  - "#task"
  - "#feature"
  - "#phase/active"


exit_criteria_met: false
initiative_id: NULL
---

# Integrate plissken for auto-generated Rust API reference docs

## Objective

Integrate the `plissken` documentation generator (colliery-io/plissken) into the brokkr docs build pipeline so that Rust API reference documentation is auto-generated from doc comments across all 4 crates, embedded into the existing mdBook site, and kept in sync via CI.

## Backlog Item Details

### Type
- [x] Feature - New functionality or enhancement

### Priority
- [x] P1 - High (important for user experience)

### Business Justification
- **User Value**: Developers get always-current API reference docs generated directly from source code doc comments, instead of stale external links to docs.rs
- **Business Value**: Eliminates manual API doc maintenance; docs drift is impossible since they're generated from code
- **Effort Estimate**: M

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] `plissken.toml` exists at project root, configured for all 4 crates with mdBook template
- [ ] `plissken render` generates Rust API reference markdown into `docs/src/api/rust/`
- [ ] Generated files are integrated into the existing mdBook `SUMMARY.md`
- [ ] `docs/src/api/README.md` updated to reference local generated docs (not docs.rs links)
- [ ] `.angreal/task_docs.py` rewritten: replaces stale Hugo references with mdBook + plissken workflow
- [ ] `.github/workflows/docs.yml` updated to install plissken and render before `mdbook build`
- [ ] Generated API docs in `docs/src/api/rust/` are gitignored (auto-generated, not committed)
- [ ] `angreal docs serve` and `angreal docs build` work end-to-end locally
- [ ] The mdBook site builds cleanly with the generated API reference pages

## Implementation Notes

### Technical Approach

1. **Create `plissken.toml`** at project root:
   - `output.path` = staging directory (e.g., `.plissken/output`)
   - `output.template` = `"mdbook"`
   - `[rust].crates` = `["crates/brokkr-broker", "crates/brokkr-agent", "crates/brokkr-models", "crates/brokkr-utils"]`
   - No `[python]` section (pure Rust project)

2. **Integration script** in `task_docs.py`:
   - Run `plissken render` to staging dir
   - Copy only `src/rust/` content into `docs/src/api/rust/`
   - Discard plissken's generated `book.toml` and `SUMMARY.md` (we maintain our own)
   - Then run `mdbook build` / `mdbook serve`

3. **SUMMARY.md** — add entries under "API Documentation" for each crate's generated pages

4. **CI workflow** — add plissken install step + render step before mdbook build

5. **Gitignore** — add `docs/src/api/rust/` since it's auto-generated

### Key Files
- `plissken.toml` (new) — plissken configuration
- `.angreal/task_docs.py` — rewrite from Hugo to mdBook+plissken
- `docs/src/SUMMARY.md` — add generated API doc entries
- `docs/src/api/README.md` — update to reference local docs
- `.github/workflows/docs.yml` — add plissken to CI
- `.gitignore` — add generated API docs path

### Key Challenge
Plissken generates a standalone mdBook project (book.toml + SUMMARY.md + content). We need to extract only the markdown content and integrate it into our existing mdBook project. The angreal task script handles this orchestration.

## Status Updates

### 2026-03-25 — Implementation complete
- Created `plissken.toml` at project root, configured for all 4 crates with mdBook template
- Plissken renders to `.plissken/output/` staging dir; `src/rust/` content is copied into `docs/src/api/rust/`
- Updated `docs/src/SUMMARY.md` with top-level crate entries under "API Documentation"
- Rewrote `docs/src/api/README.md` to reference local generated docs (removed stale docs.rs links)
- Rewrote `.angreal/task_docs.py` entirely: replaced Hugo references with mdBook + plissken workflow
  - `angreal docs build` — runs plissken render, copies output, runs mdbook build
  - `angreal docs serve` — same + starts mdbook serve
- Updated `.github/workflows/docs.yml`:
  - Added `PLISSKEN_VERSION` env var
  - Added plissken install step (`cargo install plissken@0.0.2`)
  - Added render + copy step before mdbook build
  - Added `crates/*/src/**` and `plissken.toml` to trigger paths (docs rebuild on code changes)
- Added `.plissken/` and `docs/src/api/rust/` to `.gitignore`
- mdBook builds cleanly with all 101 generated API reference pages
- 4 crate root pages + full module hierarchy rendered in sidebar