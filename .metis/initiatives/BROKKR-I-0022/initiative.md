---
id: release-pipeline-blockers-retired
level: initiative
title: "Release pipeline blockers: retired runner, Node 24 deadline, tag guard"
short_code: "BROKKR-I-0022"
created_at: 2026-06-11T11:01:39.334430+00:00
updated_at: 2026-06-11T11:14:17.787661+00:00
parent: brokkr-environment-aware
blocked_by: []
archived: false

tags:
  - "#initiative"
  - "#initiative"
  - "#phase/active"


exit_criteria_met: false
estimated_complexity: M
initiative_id: release-pipeline-blockers-retired
---

# Release pipeline blockers: retired runner, Node 24 deadline, tag guard

## Context

Pre-0.7.0 quality sweep (2026-06-11) found three issues that break or endanger the next tagged release. Verified against the live GitHub API: (1) `release.yml` pins the retired `macos-13` runner, so the `x86_64-apple-darwin` CLI build leg fails and `publish-cli-binaries` (needs all legs) attaches nothing; (2) GitHub forces Node-20 actions onto Node 24 starting **2026-06-16** — several of our pinned actions (incl. `azure/setup-helm@v3`, which is node16, on the release critical path) are affected; (3) nothing guards tag-vs-crate-version agreement, so tagging without the bump commit ships mislabeled binaries.

## Goals & Non-Goals

**Goals:**
- The next `v*` tag produces all four CLI tarballs and publishes charts/images without manual intervention.
- All workflows run supported action runtimes before the 2026-06-16 forcing date, validated green on main.
- A version-mismatch tag fails fast with a clear error.

**Non-Goals:**
- Broader CI flake hardening and efficiency (BROKKR-I-0026).
- Any change to release content/artifacts beyond fixing the pipeline.

## Detailed Design

Three independent tasks, each a small workflow edit; see tasks for file:line specifics. Validation: action bumps are exercised by normal PR/main CI; the runner swap and tag guard can only be fully proven at the next tag, so review extra carefully.

## Alternatives Considered

- Drop the `x86_64-apple-darwin` leg instead of fixing the runner — rejected: Intel Macs are still a real CLI audience.
- Wait for the Node-24 forcing and see what breaks — rejected: it lands on release-critical workflows within days.

## Implementation Plan

T-0200 (action bumps) first — it must soak on main before 06-16. T-0199 and T-0201 follow in one PR.