---
id: brokkr-cli-agent-generator
level: task
title: "brokkr-cli: agent-generator registration commands (register/deregister/list)"
short_code: "BROKKR-T-0251"
created_at: 2026-06-27T13:42:34.479955+00:00
updated_at: 2026-06-27T14:22:35.294056+00:00
parent: multi-application-isolation-safe
blocked_by: []
archived: false

tags:
  - "#task"
  - "#phase/completed"


exit_criteria_met: false
initiative_id: BROKKR-I-0030
---

# brokkr-cli: agent-generator registration commands (register/deregister/list)

## Parent Initiative

[[BROKKR-I-0030]]

## Objective

Give operators a first-class CLI path for the generator-registration lifecycle.
I-0030 added the API/SDK surface (`POST`/`DELETE /generators/:id/register`,
`GET /agents/:id/registrations`, `GET /generators/:id/registered-agents`) and
agent self-registration, but `brokkr-cli` — the `brokkr` control-plane CLI —
only has `apply` (`crates/brokkr-cli/src/main.rs`). There is no CLI way for an
admin to bootstrap or inspect registrations; today it is API/SDK-only. This is
the admin-side counterpart to the agent self-registration path, useful for
bootstrapping an agent before it is live (ADR-0009 names admin registration as
the secondary path).

### Type
- [x] Feature — new admin-facing CLI surface

### Priority
- [x] P2 — self-registration (T-0243) is the primary path; this is admin ergonomics

## Acceptance Criteria

- [x] `brokkr register --agent <id> --generator <id>` → `POST /generators/:id/register`.
- [x] `brokkr deregister --agent <id> --generator <id>` → `DELETE /generators/:id/register`.
      The destructive cascade (target removal + WS `TargetChanged` to the agent) is
      surfaced in `--help` (long_about) and in the command's stdout note.
- [x] `brokkr registrations --agent <id>` → `GET /agents/:id/registrations`.
- [x] `brokkr registrations --generator <id>` → `GET /generators/:id/registered-agents`.
      `--agent`/`--generator` are a required, mutually-exclusive clap `ArgGroup`.
- [x] Commands reuse the existing `ConnectionArgs` (broker URL / PAK / config file).
      Added four convenience methods to the high-level `BrokkrClient` wrapper
      (`register_agent`, `deregister_agent`, `list_agent_registrations`,
      `list_generator_registered_agents`) over the generated SDK, matching the
      existing wrapper pattern (e.g. `list_telemetry_events`).
- [x] Human-readable output + non-zero exit on API error (errors flow through the
      existing `run()` → `ExitCode::FAILURE` path); admin PAK enforced server-side.
- [x] `--help` / `long_about` written for each subcommand.
- [x] Coverage: register/deregister/list **endpoints** are covered by I-0030's
      integration suite (`tests/integration/api/generator_registration.rs`, 8 tests
      from #79); added CLI **parse-level** tests (clap `debug_assert`, arg-group
      exclusivity, UUID validation). Full CLI-against-live-broker e2e deferred.

## Implementation Notes

### Technical Approach

- Extend the `Commands` enum in `crates/brokkr-cli/src/main.rs` alongside `Apply`.
  This is the only command today, so establish a small subcommand pattern.
- Decide command grouping: flat verbs (`register`/`deregister`/`registrations`)
  vs. a `registration` subgroup. Flat keeps parity with the lightweight `apply`
  surface; document the choice.

### Dependencies

- Depends only on the released SDK surface (v0.8.2). Independent of
  [[BROKKR-T-0249]] / [[BROKKR-T-0250]] (those are agent-side / Helm).

### Risk Considerations

- `deregister` is destructive (cascades target removal + live WS push to the
  departing agent). The CLI must make that consequence explicit; consider a
  confirmation prompt (TTY-guarded, per the project's non-interactive-run rule).

## Status Updates

**2026-06-27 — Implemented.** Chose **flat verbs** (`register` / `deregister` /
`registrations`) over a subgroup, to match the lightweight `apply` surface.

- `crates/brokkr-client/src/wrapper.rs`: four convenience methods on `BrokkrClient`
  (`register_agent`, `deregister_agent`, `list_agent_registrations`,
  `list_generator_registered_agents`) over the generated client; imported
  `AgentGeneratorRegistration` / `AgentRegistrationBody` from `crate::types`.
- `crates/brokkr-cli/src/main.rs`: `Register` / `Deregister` (shared `RegisterArgs`,
  both `--agent` + `--generator` required) and `Registrations` (`RegistrationsArgs`
  with a required mutually-exclusive `ArgGroup`). Handlers print human-readable
  output; `deregister` prints the destructive-cascade note. Errors flow through the
  existing `run()` → `eprintln` + `ExitCode::FAILURE`.
- `crates/brokkr-cli/Cargo.toml`: added `uuid` (workspace) dep.
- Decision: **no interactive confirmation prompt** for `deregister`. The action is
  already explicit (operator types two UUIDs), and a prompt would add CI friction;
  the consequence is surfaced in `--help` + output instead. Revisit if a `--yes`
  guard is wanted later.

Verified: `cargo check --workspace` clean; `cargo test -p brokkr-cli` (config +
4 new parse tests) green; manual `--help` / arg-group / bad-UUID checks behave.
`brokkr-cli` is bin-only, so its tests run via `cargo test -p brokkr-cli` (the
`angreal tests unit` task targets lib crates). Branch `feat/i0030-operator-surface`.
