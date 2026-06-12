---
id: ts-sdk-real-yaml-validation-in
level: task
title: "TS SDK: real YAML validation in readManifests"
short_code: "BROKKR-T-0213"
created_at: 2026-06-11T11:02:08.176225+00:00
updated_at: 2026-06-11T19:21:30.817572+00:00
parent: sdk-parity-retry-validation-and
blocked_by: []
archived: true

tags:
  - "#task"
  - "#task"
  - "#phase/completed"


exit_criteria_met: false
initiative_id: BROKKR-I-0025
---

# TS SDK: real YAML validation in readManifests

## Parent Initiative

[[BROKKR-I-0025]]

## Objective

TS `readManifests` validates with regexes (`/^apiVersion:/m`, `/^kind:/m`, split on `/^---\s*$/m` — client.ts:409-417) while Rust/Python fully parse each document. Confirmed divergences: (1) false-accept — `kind: : : [unbalanced` passes TS, is rejected by Rust/Python; because TS validates before networking but the broker rejects on ingest, `apply` leaves a half-created stack (created + labeled, no deployment object); (2) false-reject — a trailing comments-only document (`---\n# comment`) throws in TS, parses to null and is skipped by Rust/Python (wrapper.rs:568-570, client.py:270-271); the regex split can also split inside block scalars containing a `---` line. Also: a subdirectory named `sub.yaml` throws raw `EISDIR` (no isFile filter, client.ts:393-396).

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] Documents parsed with a real YAML parser (dynamic import, keeping the browser bundle clean — same pattern as the node:* imports); semantics match Rust/Python: null docs skipped, every non-null doc requires apiVersion + kind.
- [ ] Directory entries filtered to files (EISDIR fixed); fs errors wrapped in BrokkrError.
- [ ] Byte output of readManifests UNCHANGED for valid inputs (the concatenation is parity-verified — only validation changes).
- [ ] Unit tests: malformed-YAML reject, comment-only-doc accept (skipped), block-scalar containing `---` accepted intact, subdir-named-*.yaml ignored.
- [ ] Contract suite green.

## Implementation Notes

Add the YAML lib as a regular dependency but only load it via dynamic import inside the Node-only path (mirroring node:fs). Check bundle impact for browser consumers — the import must stay unreachable there.

## Status Updates

*To be added during implementation*
## Status Updates

- 2026-06-11: DONE (branch feat/i0025-sdk-parity). Replaced readManifests' regex validation (`/^---\s*$/m` split + `/^apiVersion:/m` / `/^kind:/m`) with a real multi-document parse via the `yaml` package (added as a dep, dynamic-imported alongside node:fs so the browser bundle stays clean). Now matches Rust/Python: malformed YAML rejected (`invalid YAML`), null/comment-only documents skipped, every non-null document must have apiVersion+kind, and an embedded `---` inside a block scalar is no longer mistaken for a separator. Also fixed: directory entries filtered with `withFileTypes`+`isFile()` (a sub-dir named `x.yaml` is skipped instead of throwing raw EISDIR), and fs.readFile errors wrapped in BrokkrError. **Byte output unchanged** for valid input (still per-file `\s+$` strip, joined `\n---\n`, trailing `\n`) — the cross-SDK checksum parity from T-0197 holds; only validation changed. Tests: 4 new (malformed reject, comment-only skip, block-scalar-with-`---` accepted, subdir ignored) — 36 vitest pass, build clean. This also fixes the apply side-effect leak (TS used to create the stack + labels before the bad-YAML POST failed on ingest; now it rejects before any network call).