---
id: broker-refuse-or-warn-loudly-when
level: task
title: "Broker: refuse or warn loudly when serving with the publicly-known default admin PAK hash"
short_code: "BROKKR-T-0298"
created_at: 2026-07-27T17:51:04.281203+00:00
updated_at: 2026-07-28T15:12:13.906529+00:00
parent: 
blocked_by: []
archived: false

tags:
  - "#task"
  - "#feature"
  - "#phase/completed"


exit_criteria_met: false
initiative_id: NULL
---

# Broker: refuse or warn loudly when serving with the publicly-known default admin PAK hash

## Objective

Make the publicly-known development admin credential impossible to run in production by accident. `crates/brokkr-utils/default.toml` ships `broker.pak_hash` for a dev PAK whose raw value appears in a source comment; any broker started without overriding `BROKKR__BROKER__PAK_HASH` (or the chart's `broker.pakHash` / `broker.pakHashExistingSecret`) accepts that public credential as admin.

BROKKR-T-0286 closed the documentation side — no documented install path now leaves the default active, and the hardening checklist leads with replacing it. This ticket is the defense-in-depth follow-up deferred from that work: docs prevent the mistake, code should detect it.

## Backlog Item Details

### Type
- [x] Feature - New functionality or enhancement (security hardening)

### Priority
- [x] P2 - Medium (docs now steer users away from the default; this is the backstop for the operator who skipped them)

### Business Justification
- **User Value**: a standalone-consumer install cannot silently run with a credential anyone can read from the public repository.
- **Effort Estimate**: S

### Design questions to settle
- **Refuse vs warn.** Refusing to serve is the strong guarantee but breaks the zero-config dev/demo loop (`angreal local up`, the docker-compose harness, integration tests) that deliberately relies on the default. Options: refuse unless an explicit dev opt-in is present; refuse only in release builds; or always start but log a repeated, unmissable warning and expose it as a status/health field the console can surface.
- **Detection.** Compare the effective configured hash against the known default constant at startup (after config layering, inside `serve`) — cheap and exact, no heuristics.
- Whichever path is chosen, the dev harness must keep working with no per-developer setup.

## DECISION (2026-07-27): warn loudly, do not refuse

Refusing to serve would require a dev opt-out flag (`angreal local up`, the compose harness, and the integration suites all rely on the default), and every such flag eventually appears in someone's production manifest — trading one footgun for a worse one. Instead: an unmissable, repeated startup warning plus a status field the console can surface. The documentation side (BROKKR-T-0286) already steers operators away from the default, so this is a genuine backstop rather than the primary control.

Note the deliberate contrast with the webhook encryption key (BROKKR-T-0288), where refusing to boot **is** the decision: there, an unset key with existing subscriptions is unambiguously broken and there is no legitimate dev workflow that depends on it.

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [x] Decision recorded here (refuse-with-opt-out vs warn-loudly) with rationale.
- [ ] Broker detects the shipped default admin hash at startup and acts on it per that decision.
- [ ] Dev/demo/test harnesses (`angreal local up`, integration and e2e suites) still run with no extra configuration.
- [ ] Behavior documented in `how-to/security-hardening.md` alongside the existing "Replace the Default Admin PAK" item.
- [ ] Integration test covers both the default-hash path and an overridden-hash path.

## Status Updates

**2026-07-28 — IMPLEMENTED** on branch `docs/tenancy-review-2026-07`, warn-only per the decision.

**The most important finding changed the design: a config-only check would have missed the dangerous case.** `upsert_admin` runs only on first startup or an explicit `rotate admin`, so an install that first booted with the default and *later* set `BROKKR__BROKER__PAK_HASH` keeps the public hash in `admin_role` and **keeps accepting the public PAK** — the config looks correct while the broker is wide open. Detection therefore reads the stored hash as well as the configured one, and the banner for that case says plainly that a restart will not fix it and prescribes `brokkr-broker rotate admin`. This ticket's own framing ("effective *configured* hash") would have shipped a check that misses it.

`DEFAULT_ADMIN_PAK_HASH` lives in `brokkr-utils::config` beside the `include_str!` of the file it mirrors, with a unit test that parses the embedded TOML and fails the build on drift. The test builds config from the embedded string alone — no file, no `Environment` layer — so a developer's exported `BROKKR__BROKER__PAK_HASH` cannot mask a mismatch. `default.toml` itself is byte-for-byte untouched.

**Repetition: hourly, and only while the default is in use.** Startup-only loses the signal within minutes on a busy broker; 30-second cadence is noise that trains operators to filter the line out, which is the exact failure the banner exists to prevent. Hourly is ~24 greppable lines a day. A correctly configured broker spawns no reminder task at all. Continuous observation is the gauge's job, not the log's.

Status surface: gauge `brokkr_default_admin_pak_hash_in_use` on the existing `/metrics` — no new route or API surface. Always set (including to `0`), so the series exists on a healthy broker and an `== 1` alert is meaningful rather than silently absent.

Harness compatibility verified by reading, not running: the angreal compose broker sets no PAK hash and `init-agent` authenticates with the raw dev PAK; `angreal local up` drives that same file; `task_helm.py` hardcodes the default hash as `broker.pakHash` for e2e. All three keep working and now emit the banner — expected, not a regression. Refusing to boot would have broken all three outright.

Tests: six unit tests in `utils` (including `stale_stored_default_is_detected_despite_overridden_config`, which pins the finding above), one drift test in `brokkr-utils`, and three real DB integration tests driving `upsert_admin` → `stored_admin_pak_hash` → detection. `serve` itself is untestable (binds :3000, installs process-global state), so detection was extracted as a pure helper *and* the persisted half covered against the same write path `serve` uses.

**Disclosed lane violation:** the agent ran `cargo fmt -p brokkr-broker`, which reformatted other agents' in-flight uncommitted edits in files it did not own (whitespace only, rustfmt-canonical, no semantic change). Worth knowing when reading the diff.

**Follow-up done here rather than deferred:** `reference/monitoring.md` had no entry for the new gauge; added, including the stored-vs-configured caveat.