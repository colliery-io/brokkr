---
id: ts-sdk-real-yaml-validation-in
level: task
title: "TS SDK: real YAML validation in readManifests"
short_code: "BROKKR-T-0213"
created_at: 2026-06-11T11:02:08.176225+00:00
updated_at: 2026-06-11T11:02:08.176225+00:00
parent: sdk-parity-retry-validation-and
blocked_by: []
archived: false

tags:
  - "#task"
  - "#phase/todo"
  - "#task"
  - "#phase/todo"


exit_criteria_met: false
initiative_id: BROKKR-I-0025
---

# TS SDK: real YAML validation in readManifests

## Parent Initiative

[[BROKKR-I-0025]]

## Objective

TS `readManifests` validates with regexes (`/^apiVersion:/m`, `/^kind:/m`, split on `/^---\s*$/m` — client.ts:409-417) while Rust/Python fully parse each document. Confirmed divergences: (1) false-accept — `kind: : : [unbalanced` passes TS, is rejected by Rust/Python; because TS validates before networking but the broker rejects on ingest, `apply` leaves a half-created stack (created + labeled, no deployment object); (2) false-reject — a trailing comments-only document (`---\n# comment`) throws in TS, parses to null and is skipped by Rust/Python (wrapper.rs:568-570, client.py:270-271); the regex split can also split inside block scalars containing a `---` line. Also: a subdirectory named `sub.yaml` throws raw `EISDIR` (no isFile filter, client.ts:393-396).

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
