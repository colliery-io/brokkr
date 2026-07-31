---
id: docs-charts-documented-install
level: task
title: "Docs+charts: documented install paths leave the publicly-known dev admin PAK active (empty pakHash claim wrong, no bootstrap in chart examples)"
short_code: "BROKKR-T-0286"
created_at: 2026-07-27T14:27:45.781189+00:00
updated_at: 2026-07-27T17:50:55.098272+00:00
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

# Docs+charts: documented install paths leave the publicly-known dev admin PAK active (empty pakHash claim wrong, no bootstrap in chart examples)

## Objective

Close the day-zero credential footgun. The embedded `crates/brokkr-utils/default.toml:9-11` ships `broker.pak_hash` for a dev PAK whose raw value is printed in a source comment — publicly known. Three documented paths leave it live:

1. **installation.md claims the opposite of reality (blocker):** it says setting `broker.pakHash` to an empty value makes the broker generate a fresh PAK and write `/tmp/brokkr-keys/key.txt`. The chart template renders `BROKKR__BROKER__PAK_HASH` only when the value is *truthy*, so empty = omitted = **fall back to the public dev hash**, silently.
2. **multi-tenant-setup.md Helm path (blocker):** never sets a pak hash; both tenant brokers run the public admin PAK; its recovery step (`kubectl exec ... cat /tmp/brokkr-keys/key.txt`) fails because no key file is written when a hash is configured; chart values (`broker.pakHash`/`pakHashExistingSecret`) are never mentioned.
3. **charts/brokkr-broker/README.md (blocker):** no install example — including "Production Installation" — sets `broker.pakHash`/`pakHashExistingSecret`; they exist only as bare Values-table rows with no day-zero flow.
4. **security-hardening.md (major):** the hardening checklist never says to override the default admin hash.

Evidence: `docs/REVIEW-2026-07-27.md` (search "publicly-known"). Related: BROKKR-T-0278 (Secret-based sourcing), BROKKR-T-0282 (key.txt semantics).

## Backlog Item Details

### Type
- [x] Bug - Production issue that needs fixing (publicly-known admin credential in production)

### Priority
- [x] P0 - Critical

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [x] installation.md's empty-pakHash claim corrected to match template truthiness behavior, with the public-default warning stated loudly at the decision point.
- [x] Every broker install example in chart README + docs/src sets `broker.pakHash` or `broker.pakHashExistingSecret` (generate-pak flow shown first, including a `docker run` variant for Helm-only users).
- [x] security-hardening.md checklist item #1: override the default admin hash; verify with POST /auth/pak using the dev PAK (must 401).
- [x] multi-tenant-setup.md recovery guidance matches when key.txt does/doesn't exist (plus per-tenant generate-pak flow in Step 2 and the note that `--set broker.pakHash=""` is not a generation trigger via Helm).
- [ ] Consider a code hardening follow-up: refuse to serve (or warn loudly) when running with the known default hash outside dev builds.

## Status Updates

**2026-07-27** — installation.md portion done (agent-assisted, verified against `charts/brokkr-broker/templates/configmap.yaml:63`, `crates/brokkr-utils/default.toml:10`, `utils/mod.rs:95-115`): false empty-pakHash bullet deleted, truthiness warning callout added, production checklist item rewritten to generate-pak → `broker.pakHash`/`pakHashExistingSecret`. Also fixed the same wrong claim in tutorials/first-deployment.md prerequisites (nuance: an *explicitly empty* `broker.pak_hash` config value triggers key generation at the broker level; an *untouched* config uses the embedded default hash — the chart simply omits the env var unless truthy). multi-tenant-setup.md + charts/brokkr-broker/README.md + security-hardening.md checklist item in flight with the other two doc agents.

**2026-07-27 (later)** — All doc surfaces complete; docs build passes. Chart README gained a dedicated "Admin Credential (Day-Zero Bootstrap)" section (pakHash vs pakHashExistingSecret precedence, bare 64-hex format, key.txt semantics, rotate-admin recovery) and every install example now bootstraps the credential. multi-tenant-setup.md Step 2/3 rewritten per acceptance criteria. security-hardening.md leads with "Replace the Default Admin PAK (do this first)" + 401 verification. Only the optional code-hardening follow-up (refuse/warn on the known default hash outside dev) remains open — related new finding filed as BROKKR-T-0297 (CLI rotate ignores DATABASE_SCHEMA).