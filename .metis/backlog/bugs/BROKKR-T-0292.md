---
id: configreload-documented-as
level: task
title: "configReload documented as automatic in chart installs, but the chart never sets BROKKR_CONFIG_FILE or mounts the ConfigMap; BROKKR_CONFIGMAP_NAME is read by nothing"
short_code: "BROKKR-T-0292"
created_at: 2026-07-27T14:27:57.085741+00:00
updated_at: 2026-07-27T14:27:57.085741+00:00
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

# configReload documented as automatic in chart installs, but the chart never sets BROKKR_CONFIG_FILE or mounts the ConfigMap; BROKKR_CONFIGMAP_NAME is read by nothing

## Objective

Resolve a feature that is documented and templated but non-functional in the delivery vehicle everyone uses. `getting-started/installation.md` and `getting-started/configuration.md` claim `configReload.enabled=true` (default true) makes the broker watch the ConfigMap and hot-reload settings. In code, the config watcher activates only when `BROKKR_CONFIG_FILE` points at an existing file; the chart consumes the ConfigMap via `envFrom` only — it never mounts it as a file nor sets `BROKKR_CONFIG_FILE` — and `BROKKR_CONFIGMAP_NAME` (rendered by the chart) is referenced nowhere in broker code. The documented mechanism works only for hand-configured file deployments. (2026-07-27 review, two independent reviewers; `docs/REVIEW-2026-07-27.md`, search "configReload". Ground truth note: `charts/brokkr-broker/templates/configmap.yaml:79-87` renders the watcher env vars.)

## 2026-07-27 — verification: the defect is deeper than filed

All filed claims verified correct (`config_watcher.rs:45-62` gates on `BROKKR_CONFIG_FILE` existing before it ever consults `BROKKR_CONFIG_WATCHER_ENABLED`; `deployment.yaml:55-57` is `envFrom` only; `BROKKR_CONFIGMAP_NAME` appears once in the chart and zero times in `crates/`). Cosmetic correction: the configmap watcher block is lines 72-80, not 79-87.

**Two findings neither this ticket nor the review recorded — both larger than the filed bug:**

1. **Hot reload changes nothing even when the watcher runs.** `ReloadableConfig::reload()` (`brokkr-utils/src/config.rs:586-665`) recomputes `DynamicConfig` and swaps it under the `RwLock`, but **no code outside config.rs and its own tests ever reads it back**. Every accessor (`log_level()`, `webhook_delivery_batch_size()`, `cors_allowed_origins()`, …) has zero call sites in the broker. The real consumers capture startup `Settings`: background tasks at `cli/commands.rs:137-194`, the CORS layer built once at `api/v1/mod.rs:50`, log level at `bin.rs:34`. A successful reload logs a diff, emits a `config.reloaded` audit event, and alters no behavior.
2. **`POST /api/v1/admin/config/reload` cannot detect changes in a chart install.** It never 503s (ReloadableConfig is always layered) but calls `Settings::new(None)`, re-reading embedded defaults + process env — and env is frozen for the pod's lifetime. It always reports "No changes detected". **This invalidates the acceptance criterion below that proposed documenting it as a real operator path.**

Nominally-dynamic keys (all currently unread after reload): `log.level`, `broker.diagnostic_cleanup_interval_seconds`, `broker.diagnostic_max_age_hours`, `broker.webhook_delivery_interval_seconds`, `broker.webhook_delivery_batch_size`, `broker.webhook_cleanup_retention_days`, `cors.allowed_origins`, `cors.max_age_seconds`.

**Why Option 1 (mount the ConfigMap) is worse than it sounds:** the ConfigMap holds `BROKKR__FOO__BAR` env keys, not TOML, so it needs a second ConfigMap rendered as `config.toml`; env beats file in the precedence chain (`config.rs:419-438`), so unless the hot keys are *removed* from `envFrom` the file is silently overridden — two sources of truth, and `helm upgrade` on a hot key would no longer restart the pod. It must be a directory mount (subPath never receives updates), adding ~60s kubelet sync on top of the 5s debounce. And after all that it still delivers nothing until the eight values are actually consumed at runtime.

### DECISION (Dylan, 2026-07-27): make hot reload REAL — Option 1, not the recommendation below

This ticket is therefore **no longer a bug fix; it is a feature** whose current filed scope (chart plumbing) is the *last* step, not the first. Sequencing matters because doing the chart work first delivers nothing:

**Slice 1 — make dynamic config actually consumable (the real work, no chart changes).**
- Move `ReloadableConfig` construction in `cli/commands.rs` *above* the background-task spawns (currently built at `:198-208`, after `:137-194`), and pass a clone into each task.
- Background tasks re-read their settings per tick instead of capturing startup values (webhook delivery batch size + interval, diagnostic cleanup interval + max age, webhook cleanup retention). Interval changes must rebuild the `tokio::time::interval`, not just the value.
- CORS layer must be rebuildable rather than built once at `api/v1/mod.rs:50`.
- `log.level` needs a `tracing` reload handle (captured once at `bin.rs:34` today).
- Exit criterion for the slice: changing a dynamic value through `ReloadableConfig` demonstrably alters runtime behavior, provable by test without any file or chart involvement.

**Slice 2 — chart delivery.**
- Render a second ConfigMap (or key) as parseable TOML; the existing one holds `BROKKR__FOO__BAR` env keys, which `File::with_name` cannot consume.
- **Remove the hot keys from `envFrom`** — env beats file in the precedence chain (`config.rs:419-438`), so leaving them there silently overrides every reload.
- Mount as a **directory, not `subPath`** (subPath mounts never receive ConfigMap updates). Expect ~60s kubelet sync on top of the 5s debounce.
- Accept the consequence that `helm upgrade` on a hot key no longer restarts the pod (the checksum annotation covers all values), and that static-vs-dynamic keys now live in two ConfigMaps that must stay consistent.

**Also in scope:** `POST /admin/config/reload` only becomes meaningful once slice 1 lands *and* a config file exists; until then it must not be documented as an operator path. Delete `BROKKR_CONFIGMAP_NAME` (referenced nowhere in `crates/`) regardless of slice.

**Chart annotations must be reconciled as part of slice 1 (added 2026-07-27).** The chart marks values `@hot-reload: true` in `values.yaml` and `templates/configmap.yaml`. Because no code reads `ReloadableConfig` back after a reload, **every one of those annotations is currently false** — `log.level`, `broker.diagnosticCleanupIntervalSeconds`, `broker.diagnosticMaxAgeHours`, `cors.allowedOrigins`, `cors.maxAgeSeconds`.

Three webhook values were corrected to `@hot-reload: false` during BROKKR-T-0288 because they are captured into their worker's config struct at spawn and are restart-only even after slice 1 unless that slice explicitly makes those loops re-read per tick: `webhookDeliveryIntervalSeconds`, `webhookDeliveryBatchSize`, and `webhookCleanupRetentionDays` (verified: `WebhookCleanupConfig { retention_days: ... }` is built in `cli/commands.rs` and moved into the spawned task).

So slice 1 must finish with the chart telling the truth in whichever direction it lands: flip these three back to `true` if the loops are made to re-read, and leave the remaining five as `true` only once consumers genuinely read the reloaded values. Do not let the annotations drift from behavior again — they are the operator-facing contract and were wrong in three separate places today.

**Note:** this is multi-day and spans two distinct deliverables — consider promoting to an initiative and decomposing, rather than carrying it as one backlog task.

---

*Superseded recommendation, retained for context:* **Option 2** — remove the dead plumbing (`configReload.*` values, the watcher block, `BROKKR_CONFIGMAP_NAME`), correct the doc claims, and document `helm upgrade` / `kubectl rollout restart` as the mechanism. Keep the file-watcher code for hand-rolled file deployments, described honestly as a config-*file* watcher. Do **not** document the admin reload endpoint as a working path. If hot reload is wanted later, file it as a feature whose first slice is *consumers reading `ReloadableConfig`* — chart plumbing is the last step, not the first.

Doc claims to correct: `getting-started/installation.md:340-341`, `getting-started/configuration.md:13,87`, `reference/cli.md:210`, `reference/environment-variables.md:126`, `explanation/architecture.md:147`, plus comments at `configmap.yaml:8-10,33-36` and `values.yaml:23-24`.

## Backlog Item Details

### Type
- [x] Bug - Production issue that needs fixing (chart/code gap plus doc overclaim)

### Priority
- [x] P1 - High (operators will edit ConfigMaps expecting live reload; changes silently don't apply until pod restart)

## Acceptance Criteria

- [ ] Decision recorded: mount the ConfigMap as a file + set BROKKR_CONFIG_FILE in the chart (making the docs true), or remove the dead configReload plumbing and document `kubectl rollout restart` + POST /api/v1/admin/config/reload as the real paths.
- [ ] Helm-install hot-reload behavior verified end-to-end (`angreal helm test`) and the docs match the outcome.
- [ ] Dead env var `BROKKR_CONFIGMAP_NAME` removed from templates or wired to something real.

## Status Updates

*To be added during implementation*

**2026-08-01 — SLICE 0 DONE (the claims), for 0.9.1. The feature is untouched.**

Dylan asked to knock this out for a 0.9.1. What shipped is the separable half identified in the 2026-07-29 check above: **every false hot-reload claim is corrected. Neither slice 1 nor slice 2 was built**, so this ticket stays open for the feature itself.

Corrected:

| Surface | Was | Now |
|---|---|---|
| `charts/brokkr-broker/values.yaml` header | "Some settings can be changed without restarting the broker pod", with a list of five | states plainly that **every** setting requires a restart, and gives both reasons (no config file mounted → watcher never starts; nothing reads reloaded values back) |
| `values.yaml` per-key | 5 × `@hot-reload: true` | all `@hot-reload: false - requires restart`, each naming why (log filter / cleanup task / CORS layer captured at startup) |
| `templates/configmap.yaml` | section header "changes apply without pod restart" + 3 × `@hot-reload: true` | header rewritten as *intended but not today*; all three annotations `false` |
| `getting-started/configuration.md` | "A subset of broker settings can change at runtime without a restart" | leads with a warning that hot reload does not change behaviour, then describes what the machinery actually does |
| `getting-started/installation.md` | `configReload.enabled` = "Watch the ConfigMap and reload hot-reloadable settings automatically" | "Currently has no effect", with the reason |

**`configReload.*` was deliberately kept and left defaulting to `true`.** It is a real input to a feature that is intended to exist; removing it would be a breaking values change to buy nothing, and defaulting it false would mean flipping it back when the feature lands. The comments now say it does nothing rather than the values pretending otherwise.

**One thing the docs were already right about**, and it was left alone: `configuration.md` already explained that the watcher never starts under Helm because no `BROKKR_CONFIG_FILE` is mounted. What it missed was the deeper half — that even *with* a file, nothing consumes the reloaded values — so a reload is detection and an audit entry, and no behaviour change. That is now stated.

Verified: `angreal helm check-values` (52 assertions), `helm lint`, `angreal docs build` all pass.

**Still open — the feature.** Slice 1 (make `ReloadableConfig` actually consumable) then slice 2 (chart delivery), exactly as sequenced above. Doing slice 2 first still delivers nothing.

**2026-08-02 — SLICES 1 AND 2 DONE. Hot reload now changes behaviour.**

Scope decided with Dylan: the tunables and log level become genuinely hot; **CORS stays restart-only** and remains annotated `@hot-reload: false`. Replacing `tower_http`'s `CorsLayer` — which is baked into the router at construction and cannot be swapped at runtime — would mean owning preflight, credentials and origin matching by hand, which is disproportionate risk for a setting that changes rarely.

### Slice 1 — reloaded values are now consumed

- **`ReloadableConfig` is constructed before the background tasks**, not after. It used to be built at `commands.rs:271` while the tasks spawned at `:205`, so no task could ever hold a handle to it.
- **Three task loops re-read per tick** — diagnostic cleanup, webhook delivery, webhook cleanup — through the same accessors that previously had zero call sites. Where an interval changed, the ticker is rebuilt and its immediate first tick consumed, so a rebuild does not run the pass twice.
- **Log level is applied inside `reload()`**, not by callers, so the ConfigMap watcher and `POST /admin/config/reload` both get it. `update_log_level` already worked and simply had no caller outside its own tests. The config write lock is released before calling into `logging`.

### Slice 2 — the chart actually delivers it

- New `configmap-dynamic.yaml` renders the hot keys as **TOML**, mounted at `/etc/brokkr` as a **directory** (a `subPath` mount is resolved once at container start and never sees updates).
- `BROKKR_CONFIG_FILE` is set, without which the watcher never starts.
- **The hot keys are removed from the env ConfigMap.** This is the subtle half: environment beats file in the precedence chain, so a key left behind would silently override the mounted file and reloading would appear to do nothing. With `configReload.enabled=false` they fall back to env, so disabling the feature loses nothing.

### Two corrections found while building

**A claim in the earlier claims-only commit was wrong.** It said `helm upgrade` restarts the pod anyway via the ConfigMap checksum. It does not: the checksum annotation is on the *ConfigMap*, for external reload controllers, and **the pod template has no annotation at all**. Changing a restart-only value updates the ConfigMap and leaves the pod running with its old environment. Corrected to say so and to name the remedy.

**Adding a second ConfigMap broke five existing render assertions**, which used `single_manifest(docs, "ConfigMap")` and silently began matching the dynamic one. Added an `env_configmap()` helper rather than renaming the template to win the sort order. Worth noting the assertions did their job — this would otherwise have been a quiet mis-assertion.

### Verification

57 chart render assertions (52 + 5 new), `helm lint`, 26 + 151 unit tests, docs build. The new `test_reload_applies_log_level_to_the_logger` is the exit criterion this ticket asked for: it asserts the logger's level actually moves, not merely that a diff is reported. It also surfaced that reloading to the value already in effect is correctly a no-op, so the test has to use a level differing from `default.toml`.

Four clippy warnings remain in these crates; all four are present with the change stashed, so they are pre-existing.

**Remaining, deliberately:** CORS. `cors.allowedOrigins` and `cors.maxAgeSeconds` are still restart-only and still annotated as such, and `reload()` still reports them as changes — the report is accurate, only nothing acts on it. Anyone picking that up should read the CORS note above first.
