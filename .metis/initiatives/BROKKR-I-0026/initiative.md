---
id: docs-and-ci-hygiene-staleness
level: initiative
title: "Docs and CI hygiene: staleness, coverage gaps, flake hardening"
short_code: "BROKKR-I-0026"
created_at: 2026-06-11T11:01:39.530373+00:00
updated_at: 2026-06-11T21:29:00.157997+00:00
parent: brokkr-environment-aware
blocked_by: []
archived: false

tags:
  - "#initiative"
  - "#initiative"
  - "#phase/completed"


exit_criteria_met: false
estimated_complexity: M
initiative_id: docs-and-ci-hygiene-staleness
---

# Docs and CI hygiene: staleness, coverage gaps, flake hardening

## Context

Pre-0.7.0 sweep findings that are individually small but collectively erode trust: docs that went stale around I-0021 and the 0.6.0 release (a pinned `0.5` install snippet whose page examples then cannot compile; the CI/CD tutorial still teaching jq-escaped JSON when `brokkr apply` was built for exactly that case; phantom `brokkr-ui` image; dead `develop-*` tag examples), three crates whose unit tests never run in CI, a setup job that warms a cache nobody reads, no retries on the two flake classes that actually bit us this week, and four crates missing license metadata. Verified clean (no action): SUMMARY↔disk match, zero link rot, reference/cli.md broker/agent spot-checks accurate.

## Goals & Non-Goals

**Goals:**
- Every doc claim checked in the sweep is current; submission docs lead with the on-ramp.
- Every crate's unit tests run in CI; observed flake classes have retries.
- All crates carry license/repository metadata.

**Non-Goals:**
- A full Diátaxis re-review (done recently; this is targeted staleness only).
- Release-blocking workflow fixes (BROKKR-I-0022).

## Detailed Design

Five tasks: docs staleness (T-0215), submission-example modernization (T-0216), unit-matrix gaps (T-0217), flake hardening + CI efficiency (T-0218), crate metadata (T-0219). File:line specifics in each task.

## Alternatives Considered

- Folding into the release-blocker initiative — rejected: none of this gates the tag; different urgency.
- Auto-generating version strings in docs — worth considering at T-0215 (a release-checklist grep is the cheap version).

## Implementation Plan

Any order; T-0217+T-0219 are trivial and can ship together. T-0218's Docker Hub auth needs a secrets decision (DOCKERHUB_TOKEN) — flag at implementation.