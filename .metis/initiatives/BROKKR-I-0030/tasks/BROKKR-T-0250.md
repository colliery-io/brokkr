---
id: promote-generator-ids-to
level: task
title: "Promote generator IDs to AgentConfig (BROKKR__AGENT__GENERATOR_IDS) and CLI flag"
short_code: "BROKKR-T-0250"
created_at: 2026-06-27T13:42:34.404803+00:00
updated_at: 2026-06-27T14:15:04.453085+00:00
parent: multi-application-isolation-safe
blocked_by: []
archived: false

tags:
  - "#task"
  - "#phase/completed"


exit_criteria_met: false
initiative_id: BROKKR-I-0030
---

# Promote generator IDs to AgentConfig (BROKKR__AGENT__GENERATOR_IDS) and CLI flag

## Parent Initiative

[[BROKKR-I-0030]]

## Objective

Bring generator-scope configuration into the agent's first-class config surface.
I-0030 shipped `BROKKR_GENERATOR_IDS` as a bare `std::env::var` read
(`crates/brokkr-agent/src/cli/commands.rs:+143`), bypassing the
`Settings`/`AgentConfig` system entirely. Every other agent setting is a
`BROKKR__AGENT__*` key resolved through `brokkr-utils` config (env + config
file + defaults). The current one-off path means generator scope cannot be set
via the config file, breaks the `BROKKR__SECTION__KEY` naming convention, and is
invisible to anyone reading the `Agent` config struct.

### Type
- [x] Tech Debt — feature works but is off-convention and config-file-unreachable

### Priority
- [x] P2 — functional today via raw env; this is consistency + completeness

## Acceptance Criteria

- [x] `Agent` config struct (`crates/brokkr-utils/src/config.rs`) gains a
      `generator_ids: Option<String>` field (`#[serde(default)]`), resolvable via
      `BROKKR__AGENT__GENERATOR_IDS` or a config file. Chose `Option<String>`
      (comma-separated) over `Vec<Uuid>`: the `config` crate's `Environment`
      source has no `list_separator` configured, so env values arrive as strings;
      a string round-trips cleanly for both env and file and is parsed to UUIDs at
      the existing call site.
- [x] Optional `--generator-ids` CLI flag on the agent `start` command
      (`cli/mod.rs`), threaded through `bin.rs` into `commands::start(...)`.
- [x] Agent startup resolves scope via `resolve_generator_ids(...)` instead of a
      bare `std::env::var`; reads `config.agent.generator_ids`.
- [x] Back-compat: bare `BROKKR_GENERATOR_IDS` is still honored as the lowest-
      precedence fallback and logs a one-time **deprecation warning** when it is
      the source. Documented in the struct doc-comment and `--help`.
- [x] Malformed-UUID handling and the empty/no-op default preserved (skip
      registration when the resolved string is blank; warn per malformed entry).
- [x] Coordinated with [[BROKKR-T-0249]]: settled on `BROKKR__AGENT__GENERATOR_IDS`;
      the Helm configmap now emits that key (was the bare env var).
- [x] `angreal tests unit brokkr-utils` (24 ✓) / `brokkr-agent` (79 ✓, incl. 5 new
      `resolve_generator_ids` precedence tests).

## Implementation Notes

### Technical Approach

- Mirror the existing `Agent` struct fields (`watch_namespace`, `ws_url`, etc.)
  for resolution order: CLI flag > env > config file > default.
- Parsing: comma-separated UUIDs → `Vec<Uuid>`; reject malformed entries with the
  same warning behaviour the agent already has, or fail fast at config load —
  decide and document.

### Dependencies

- Tightly coupled to [[BROKKR-T-0249]] (Helm). Land the key-name decision here,
  consume it there.

### Risk Considerations

- Don't silently change behaviour for existing deployments that set the bare
  `BROKKR_GENERATOR_IDS`; the back-compat window is an acceptance criterion.

## Status Updates

**2026-06-27 — Implemented.** Resolution precedence: `--generator-ids` flag >
`BROKKR__AGENT__GENERATOR_IDS` / `agent.generator_ids` (config file) > legacy bare
`BROKKR_GENERATOR_IDS` (deprecated, warns when used).

- `crates/brokkr-utils/src/config.rs`: `Agent.generator_ids: Option<String>`
  (`#[serde(default)]`).
- `crates/brokkr-agent/src/cli/mod.rs`: `Start { generator_ids: Option<String> }`
  via `#[arg(long)]`; `bin.rs` threads it into `commands::start(...)`.
- `crates/brokkr-agent/src/cli/commands.rs`: `start()` now takes the override and
  resolves via a pure `resolve_generator_ids(flag, config, legacy) -> (String, bool)`
  helper (the bool drives the deprecation warning). Malformed-UUID + empty-default
  behaviour unchanged.
- Helm (finalizes [[BROKKR-T-0249]]): `configmap.yaml` now emits
  `BROKKR__AGENT__GENERATOR_IDS`; README/value docs updated to match.

Verified: `cargo check --workspace` clean; `angreal tests unit brokkr-utils` (24)
and `brokkr-agent` (79, incl. 5 new precedence tests) pass; `helm lint` clean and
`helm template` emits the new key (empty→absent, list→CSV). Branch
`feat/i0030-operator-surface`.