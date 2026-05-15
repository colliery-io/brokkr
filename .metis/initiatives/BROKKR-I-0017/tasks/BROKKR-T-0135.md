---
id: b2-prototype-rust-client-with
level: task
title: "B2: Prototype Rust client with progenitor"
short_code: "BROKKR-T-0135"
created_at: 2026-05-14T18:26:22.086013+00:00
updated_at: 2026-05-14T22:36:18.427834+00:00
parent: BROKKR-I-0017
blocked_by: [BROKKR-T-0133]
archived: false

tags:
  - "#task"
  - "#phase/completed"


exit_criteria_met: false
initiative_id: BROKKR-I-0017
---

# B2: Prototype Rust client with progenitor

## Parent Initiative

[[BROKKR-I-0017]]

## Objective

Scaffold `crates/brokkr-client`, generate a Rust client from the hardened `openapi/brokkr-v1.json` using `progenitor`, and confirm the generated code compiles and produces a working low-level client (no ergonomic wrapper yet — that's C1).

## Acceptance Criteria

## Acceptance Criteria

- [x] New crate `crates/brokkr-client` added to the workspace (auto-discovered via `crates/*` glob).
- [x] `progenitor::generate_api!` macro wired in `src/lib.rs`. Spec at `../../openapi/brokkr-v1.json`, `interface = Builder`.
- [x] `cargo build -p brokkr-client` succeeds. `cargo doc -p brokkr-client --no-deps` produces a 69-type schema surface in `types/`, a `Client` struct, and per-operation builder structs (one per route, ~85 operations).
- [~] Surface test (`tests/surface.rs`) compiles against the generated `Client`, exercises `list_agents`, `create_agent`, `get_agent`, `list_stacks`, `create_stack`, `list_work_orders`, `create_work_order`, `claim_work_order`, `complete_work_order`, `verify_pak`, `update_health_status`, `list_webhooks`, `get_pending_agent_webhooks`, and the `types::ErrorResponse` shape. Round-trip against a real broker requires DB infra and is properly the ergonomic wrapper's smoke test (T-C1) — not blocked, just sequenced.
- [x] Findings captured in status updates below (4 spec-side fixes needed; ongoing version-pin discipline).

## Implementation Notes

### Technical Approach

1. Add `crates/brokkr-client` to the root workspace `Cargo.toml`.
2. Use `progenitor::generate_api!("../../openapi/brokkr-v1.json")` (or `build.rs` for more control) — confirm chosen approach with the rest of the workspace's build conventions.
3. Decide on async runtime (`tokio`, presumably — matches the broker and agent).
4. Wire a minimal smoke test using the existing broker integration test harness if practical.

### Dependencies

- Hard: [[BROKKR-T-0133]] (need the hardened spec).
- Soft: [[BROKKR-T-0134]] — CI drift check is nice to have first, but B2 can proceed in parallel with B1.

### Risk Considerations

- `progenitor` has rough edges around certain OpenAPI features (e.g. discriminated unions, oneOf). If the spec uses any, expect to either work around in the spec or in the wrapper.
- The "any of three PAK schemes maps to one Authorization header" accepted-limitation (A1 from the audit) may surface here — flag what progenitor generates so C1 can hide it.

## Status Updates

### 2026-05-14 — Completed

**Files added:**

- `crates/brokkr-client/Cargo.toml` — depends on `progenitor 0.14`, `progenitor-client 0.14`, `reqwest 0.13` (default-features off, `json` + `stream`), plus serde/uuid/chrono support types.
- `crates/brokkr-client/src/lib.rs` — single `progenitor::generate_api!` macro invocation. Builder interface.
- `crates/brokkr-client/tests/surface.rs` — 3 surface tests that exercise the generated client surface without network access.

**Build & test:**

- `cargo build -p brokkr-client` — clean.
- `cargo test -p brokkr-client --tests` — 3/3 pass.
- `cargo doc -p brokkr-client --no-deps` — clean.
- `cargo build --workspace` — clean (no regressions in other crates).
- `angreal openapi check` — clean.
- `redocly lint` — `valid 🎉` with 7 unused-component warnings (pre-existing).

**Generator quality findings (input for C1 and beyond):**

Progenitor 0.14 / openapiv3 2.x do **not** accept OpenAPI 3.1, which utoipa 5.x emits by default. Resolved by extending `examples/openapi_export.rs` with a downgrade pass that runs on every export. Four 3.1→3.0 fixups currently applied:

1. **Top-level `openapi` version** → coerced to `"3.0.3"`.
2. **`type: [<primitive>, "null"]`** → split into `type: <primitive>` + `nullable: true`. Affected 100 schema sites pre-fix.
3. **Nullable `$ref`s spelled as `oneOf: [{type:"null"}, X]`** → unwrapped to bare `X`. 3.0 has no clean way to express nullable refs; the field's optional-ness is still carried by `required: []` so callers behave correctly.
4. **`propertyNames`** keyword → dropped. JSON keys are always strings; the constraint was vacuous.

After these, the spec compiles cleanly through progenitor and lints clean.

**Progenitor's other constraints:**

- **Single 2xx response type per operation.** `complete_work_order` originally documented both 200 (WorkOrderLog) and 202 (retry scheduled, no body). Progenitor refuses. Dropped the 202 from the annotation (still emitted at runtime; SDK callers wanting to react to retry scheduling must match the raw status code). Documented inline in `work_orders.rs`. C1 may want to type-wrap this back into an enum at the ergonomic layer.
- **Reqwest version coupling.** progenitor-client transitively pulls `reqwest 0.13`. brokkr-client must use the same line or get 256+ trait-mismatch errors. Pinned to `^0.13`.
- **No `rustls-tls` on reqwest 0.13** (features renamed). Currently using default TLS (OpenSSL/native). C1 can opt into rustls explicitly if we want a deterministic crypto stack.

**Generated surface shape:**

- Single `Client` struct, constructed via `Client::new(base_url)`.
- Per-operation **builder structs** (`AgentsListLabels`, `CreateWorkOrder`, ...) returned by `Client::method()`. Each has setter methods for params/body, then a terminal `.send().await`.
- ~85 operations, 69 schema types under `types::`, plus a `types::ErrorResponse` that flows through 4xx/5xx responses cleanly.
- Auth (the A1 single-credential abstraction) is **not** in the generated layer. The three `*_pak` security schemes all map to the same `Authorization` header so the wrapper just needs to inject it once.

**Implications for downstream tasks:**

- **T-C1 (ergonomic wrapper)**: must add (a) `Authorization` header injection on every request, (b) retry/backoff on 5xx, (c) re-typing of `complete_work_order`'s 202 into an enum, (d) optional `rustls-tls` configuration. The accepted limitation A1 from the audit is fully hidden — confirmed.
- **T-B3 (Python client)**: will hit the same 3.1→3.0 issue (`openapi-python-client` also targets 3.0). The downgrade pass in `examples/openapi_export.rs` already handles it; nothing extra needed on the spec side.
- **T-C3 (regeneration drift CI)**: brokkr-client regenerates on every `cargo build` via the proc macro; no committed generated source to diff. The drift check effectively reduces to "does the crate compile against the committed spec", which is already covered by `cargo build` in CI. T-C3's Rust half may end up being a no-op modulo a comment.

**Decisions / tradeoffs:**

- Used the proc macro (`generate_api!`) over `build.rs`. For a 230kB spec the macro expansion is ~59k lines of generated code — compiles in ~20s clean, fine in CI. If iteration time becomes a problem we can switch to `build.rs` with `include!(concat!(env!("OUT_DIR"), "/client.rs"))`.
- Chose `Builder` interface over `Positional`. Builder is verbose for arg-less GETs but consistent and avoids ambiguous method overloads on multi-arg ops.