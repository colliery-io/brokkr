---
id: getting-started-path-is-un
level: task
title: "Getting-started path is un-followable for Helm-only consumers: tutorials require the angreal dev env, brokkr-broker binary unobtainable, wget verification fails, v0.8.0 pins"
short_code: "BROKKR-T-0293"
created_at: 2026-07-27T14:27:59.006124+00:00
updated_at: 2026-07-28T16:14:07.112153+00:00
parent: 
blocked_by: []
archived: false

tags:
  - "#task"
  - "#bug"
  - "#phase/completed"


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

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] Tutorials state prerequisites honestly and/or gain a Helm-install variant (at minimum: how to adapt agent name/PAK/URL to a consumer install).
- [ ] A documented way to run `brokkr-broker generate-pak` without a source checkout (e.g. `docker run --rm <broker-image> generate-pak`), verified to work.
- [ ] All `kubectl exec` verifications use curl; all version pins reference the current lockstep release or omit the pin.
- [ ] evaluate.md links Core Concepts before first use of the jargon.

## Status Updates

**2026-07-28 — COMPLETE** on branch `docs/tenancy-review-2026-07`. `angreal docs build` passes.

**The ticket's version claim was imprecise in a way that mattered.** It says 0.8.0 ignores the `existingSecret` values. More precisely: `postgresql.existingSecret` **does** work in 0.8.0; the three that do not exist before **v0.8.4** are the admin-PAK-hash, webhook-key, and agent-PAK ones — and **0.8.3 is equally affected**, not just 0.8.0. Since the danger window is therefore *every released chart except the newest*, pinning the docs to a fixed number is the fragile option. Decision: installs and upgrades are **unpinned** (resolve latest), the one deliberate pin example is `0.8.4`, and the number appears in exactly one file — `install-operations.md`'s new "Chart Versions and the `existingSecret` Values" section, with a per-value "first released in" table. `installation.md` states no version at all and links there, so the two pages cannot drift apart.

**The ticket's tutorial claim was also too broad.** "Every tutorial requires `angreal local up`, the pre-created agent, and the public dev PAK" is wrong: `templates.md` and `multi-cluster-targeting.md` need **no agent whatsoever**, and `cicd-generators.md` used a positional `.[0]` lookup rather than the dev agent by name. Verified requirement per tutorial: first-deployment needs a live agent process (the only one with `kubectl` checks); cicd-generators needs one pre-existing agent record, live only if the deployment should actually land; multi-cluster-targeting and templates need broker + admin PAK only. `README.md` now carries a per-tutorial "what it needs" column plus an "adapting the commands to your install" section covering the four things that genuinely differ — broker URL, admin PAK, agent name, and poll-cycle length (10s binary default vs 30s chart default). The hardcoded `brokkr-integration-test-agent` lookups are parameterised via `AGENT_NAME`.

**`docker run … generate-pak` verified by actually running it**, not by reading the Dockerfile: `docker run --rm ghcr.io/colliery-io/brokkr-broker:0.8.4 generate-pak` prints the pair and the day-zero flow. Also confirmed in-image that `curl` is present and `wget` is **absent** in both broker and agent images — which is why all three `kubectl exec … wget` verifications in `installation.md` are now `curl -fsS`.

**Left for a later pass (files were locked by concurrent work):**
- `getting-started/evaluate.md` — still needs the Core Concepts link before first jargon use, and still pins `--version 0.8.0` in two commands plus prose. These are now the **last stale 0.8.0 pins in docs/src**, and they carry exactly the silent-plaintext-credentials footgun above.
- `how-to/network-configuration.md` — the last surviving `kubectl exec … wget`.
- **Book-wide dead links, filed as BROKKR-T-0311**: mdBook renders `*/README.md` as `index.html` but leaves `href=".../README.html"` in the output, so every cross-section overview link 404s. New links here follow the existing convention rather than diverging in isolated files; the fix is one change applied across many.