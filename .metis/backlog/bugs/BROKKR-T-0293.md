---
id: getting-started-path-is-un
level: task
title: "Getting-started path is un-followable for Helm-only consumers: tutorials require the angreal dev env, brokkr-broker binary unobtainable, wget verification fails, v0.8.0 pins"
short_code: "BROKKR-T-0293"
created_at: 2026-07-27T14:27:59.006124+00:00
updated_at: 2026-07-27T14:27:59.006124+00:00
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

# Getting-started path is un-followable for Helm-only consumers: tutorials require the angreal dev env, brokkr-broker binary unobtainable, wget verification fails, v0.8.0 pins

## Objective

Make the learning path executable by the standalone-consumer audience (external team, Helm install, no source checkout). Grouped findings (2026-07-27 review; `docs/REVIEW-2026-07-27.md`):

1. **tutorials/README.md contradiction (major)**: index says "follow the Installation Guide first" (Helm path), but every tutorial actually requires `angreal local up`, the pre-created `brokkr-integration-test-agent`, and the public dev PAK. A Helm-installed reader hits missing agents and a wrong PAK; there is no consumer-runnable variant of the learning path.
2. **generate-pak binary unobtainable (major)**: the recommended production bootstrap runs `brokkr-broker generate-pak` offline, but no doc says where a Helm-only team gets the `brokkr-broker` binary (container image `docker run`, cargo install, or source build) — the only binary-download instructions cover the `brokkr` CLI.
3. **wget verification fails (major)**: installation.md verification runs `kubectl exec ... wget`, but both published images are `debian:bookworm-slim` with only curl installed.
4. **Stale version pins (major)**: evaluate.md Path B pins v0.8.0 (predates console, existingSecret, named PAKs); install-operations.md pins `--version 0.8.0` throughout — with the nasty interaction that 0.8.0 charts silently ignore existingSecret values, resurrecting the dev-PAK footgun (BROKKR-T-0286).
5. **evaluate.md concept dump (major, clarity)**: first hands-on page uses stack/generator/deployment-object/registration/target with no link to Core Concepts.

## Backlog Item Details

### Type
- [x] Bug - Production issue that needs fixing

### Priority
- [x] P1 - High (the audience the docs target cannot complete the path)

## Acceptance Criteria

- [ ] Tutorials state prerequisites honestly and/or gain a Helm-install variant (at minimum: how to adapt agent name/PAK/URL to a consumer install).
- [ ] A documented way to run `brokkr-broker generate-pak` without a source checkout (e.g. `docker run --rm <broker-image> generate-pak`), verified to work.
- [ ] All `kubectl exec` verifications use curl; all version pins reference the current lockstep release or omit the pin.
- [ ] evaluate.md links Core Concepts before first use of the jargon.

## Status Updates

*To be added during implementation*
