---
id: typescript-sdk-submitmanifests
level: task
title: "TypeScript SDK: submitManifests/apply folder helper"
short_code: "BROKKR-T-0197"
created_at: 2026-06-11T02:19:33.135354+00:00
updated_at: 2026-06-11T05:47:27.947179+00:00
parent: BROKKR-I-0021
blocked_by: []
archived: true

tags:
  - "#task"
  - "#phase/completed"


exit_criteria_met: false
initiative_id: BROKKR-I-0021
---

# TypeScript SDK: submitManifests/apply folder helper

## Parent Initiative

[[BROKKR-I-0021]]

## Objective

Mirror the folder helpers in the TypeScript SDK wrapper (`sdks/typescript/brokkr-client/src/client.ts`) — `submitManifests`/`apply`.

## Design

- `submitManifests(stackId, pathOrPaths)` — accept a dir / file / globs (Node `fs`); read `*.yaml`/`*.yml`; concatenate with `---`; validate each doc parses (a YAML lib) and has `apiVersion`+`kind`; POST the stream (raw-YAML body when available).
- `apply(stackName, path, targeting?)` — idempotent create-or-reuse + submit-on-change + targeting; return `{ status: "created"|"updated"|"unchanged" }`.
- Node-only (filesystem); keep the browser build clean — folder helpers behind a Node entry point if needed.

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] `submitManifests`/`apply` reading a directory of YAML files
- [ ] Idempotent `apply` (unchanged folder → no new revision)
- [ ] Per-doc validation with `BrokkrError`s
- [ ] vitest coverage (folder fixture; idempotency)
- [ ] TypeScript SDK how-to updated

## Status Updates

- 2026-06-11: Created under BROKKR-I-0021. Parallel with T-0195/T-0196.
- 2026-06-11: IMPLEMENTED (branch feat/i0021-raw-yaml-submission). Added `submitManifests(stackId, path)` and idempotent `apply(stackName, path, targeting=[]) -> ApplyResult` (discriminated union created/updated/unchanged) to `sdks/typescript/brokkr-client/src/client.ts`, plus Node-only module helpers `readManifests` (dynamic-import node:fs/path; folder walk + per-doc apiVersion/kind check) and `sha256Hex` (dynamic-import node:crypto; matches broker checksum). fs/crypto are dynamically imported so the browser bundle stays clean. ApplyResult/readManifests/sha256Hex exported from index. Added @types/node devDep (types only, not shipped). Unit tests: 5 vitest in src/manifests.test.ts (sorted folder concat, single-file multidoc, missing apiVersion/kind reject, empty/missing errors, sha256 known-vector) — 31 pass. Functional: tests/sdk-contract/typescript/src/manifest-apply.test.ts (created→unchanged→updated + label + submitManifests). typecheck clean. Docs: TS SDK how-to. Remaining: contract test runs on CI.