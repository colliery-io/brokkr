---
id: typescript-sdk-submitmanifests
level: task
title: "TypeScript SDK: submitManifests/apply folder helper"
short_code: "BROKKR-T-0197"
created_at: 2026-06-11T02:19:33.135354+00:00
updated_at: 2026-06-11T02:19:33.135354+00:00
parent: BROKKR-I-0021
blocked_by: []
archived: false

tags:
  - "#task"
  - "#phase/todo"


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

- [ ] `submitManifests`/`apply` reading a directory of YAML files
- [ ] Idempotent `apply` (unchanged folder → no new revision)
- [ ] Per-doc validation with `BrokkrError`s
- [ ] vitest coverage (folder fixture; idempotency)
- [ ] TypeScript SDK how-to updated

## Status Updates

- 2026-06-11: Created under BROKKR-I-0021. Parallel with T-0195/T-0196.
