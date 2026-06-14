# BROKKR-I-0029 — Phase 1 Discovery Inventory (working artifact)

Raw inventories from the parallel discovery agents. Assembled/curated into the
initiative once all five land. Each section is verbatim agent output (file:line
citations are the agent's; verify before asserting in final docs).

Status:
- [x] Agent 1 — Broker HTTP API surface
- [x] Agent 2 — CLI / config keys / env vars
- [x] Agent 3 — WS / wire / SDK surface
- [x] Agent 4 — Existing docs audit (UI-as-product, onboarding, integration gaps)
- [x] Agent 5 — Background tasks / telemetry / data model

---

## Agent 2 — CLI, Config, Environment

### CLI Surface

**brokkr-broker** (`crates/brokkr-broker/src/cli/mod.rs`; dispatch `bin.rs`):
- `serve` — start HTTP server. Binds **hardcoded** `0.0.0.0:3000` (`cli/commands.rs:199-201`), not a flag/config.
- `create agent --name <S> --cluster-name <S>`
- `create generator --name <S> [--description <S>]`
- `rotate agent --uuid <Uuid>` / `rotate generator --uuid <Uuid>` / `rotate admin`

**brokkr-agent** (`crates/brokkr-agent/src/cli/mod.rs`): only `start`. **Zero flags** — all config/env driven. Health server binds `0.0.0.0:{health_port}` (default 8080, `cli/commands.rs:186-189`).

**brokkr-cli** (binary `brokkr`, `crates/brokkr-cli/src/main.rs`):
- Global (`ConnectionArgs`, global=true): `--broker-url` (→ env `BROKKR_BROKER_URL` → config; `/api/v1` appended if omitted), `--pak` (→ env `BROKKR_PAK` → config), `--config` (default `~/.brokkr/config`).
- `apply -f/--filename <path> --stack <S> [--target-label <k:v> ...]` — folder of top-level `*.yaml|*.yml` or single file; stack created if absent; target-labels repeatable.

### Config keys (`brokkr-utils`)
Struct: `crates/brokkr-utils/src/config.rs`. Defaults: `crates/brokkr-utils/default.toml`. Load order (later wins): embedded default.toml → optional external file → `BROKKR__`-prefixed env (`config.rs:408-427`).

- `[database]`: `url` (default `postgres://brokkr:brokkr@localhost:5433/brokkr`), `schema` (Option, None=public).
- `[log]`: `level` (default `debug`), `format` (default `text`).
- `[broker]` (all Option): `pak_hash`; `diagnostic_cleanup_interval_seconds`=900; `diagnostic_max_age_hours`=1; `webhook_encryption_key` (None→random at startup); `webhook_delivery_interval_seconds`=5; `webhook_delivery_batch_size`=50; `webhook_cleanup_retention_days`=7; `audit_log_retention_days` (⚠ not in default.toml; doc says 90 — verify where applied); `auth_cache_ttl_seconds`=60 (0 disables); `agent_events_retention_days`=30 (0/unset disables eviction).
- `[pak]`: `prefix`=brokkr, `rng`=osrng, `digest`=8 (⚠ field is Option<String> but default.toml sets integer 8), `short_token_length`=8, `long_token_length`=24, `short_token_prefix`=BR, plus derived `*_str` fields.
- `[cors]`: `allowed_origins`=["http://localhost:3001"], `allowed_methods`=[GET,POST,PUT,DELETE,OPTIONS], `allowed_headers`=[Authorization,Content-Type], `max_age_seconds`=3600.
- `[agent]`: `broker_url`=http://localhost:3000, `polling_interval`=10, `kubeconfig_path`=`/home/${USER}/.kube/config` (⚠ `${USER}` NOT expanded by loader), `max_retries`=60, `pak`, `agent_name`=DEFAULT, `cluster_name`=DEFAULT, `health_port`=8080, `deployment_health_enabled`=true, `deployment_health_interval`=60, `ws_force_rest`=false, `ws_url` (None→derived from broker_url), `kube_event_uid_cache_cap`=10000, `watch_namespace` (None=cluster-wide).
- `[telemetry]`: `enabled`=false, `otlp_endpoint`=http://localhost:4317, `service_name`=brokkr, `sampling_rate`=0.1; plus `[telemetry.broker]` (service_name=brokkr-broker) / `[telemetry.agent]` (service_name=brokkr-agent) overrides that inherit base. Merge: `Telemetry::for_broker()/for_agent()` (`config.rs:315-351`).
- **brokkr-cli has a SEPARATE config** (`crates/brokkr-cli/src/config.rs:19-26`): only `broker_url` + `pak` in `~/.brokkr/config`. Do NOT conflate with `BROKKR__` broker/agent config.

### Environment variables
- `BROKKR__<SECTION>__<FIELD>` (double underscore) — generic over the whole Settings struct (`config.rs:420`). Every config key settable this way.
- Standalone (single underscore, direct `env::var`): `BROKKR_CONFIG_FILE` (external config path; broker `bin.rs:29`, agent `cli/commands.rs:103`, watcher `config_watcher.rs:47`); `BROKKR_CONFIG_WATCHER_ENABLED` (default on unless false/0, `config_watcher.rs:65`); `BROKKR_CONFIG_WATCHER_DEBOUNCE_SECONDS`=5 (`config_watcher.rs:75`); `BROKKR_BROKER_URL` / `BROKKR_PAK` (CLI only, `crates/brokkr-cli/src/config.rs:105-106`).
- `KUBECONFIG` — **set, not read** by agent from `agent.kubeconfig_path` (`k8s/api.rs:1055`); consumed implicitly by `kube` crate.
- No `RUST_LOG`, `dotenv`, or `envy` anywhere. Log level only via `log.level`/`BROKKR__LOG__LEVEL`.

### Ambiguities to resolve before documenting
1. `pak.digest` — Option<String> vs integer default `8`; verify intended semantics.
2. `broker.audit_log_retention_days` — doc says default 90 but absent from default.toml & no `unwrap_or(90)` in config.rs; find where 90 is actually applied (audit worker?).
3. `agent.kubeconfig_path` default literal `${USER}` is not expanded — footgun to document.
4. Broker bind `0.0.0.0:3000` is hardcoded (not configurable); agent has no CLI flags.
5. Two distinct config systems (brokkr-utils Settings vs brokkr-cli config).

---

## Agent 1 — Broker HTTP API surface (key facts)

**Auth model:** All `/api/v1/*` behind `auth_middleware` (middleware.rs:68, applied v1/mod.rs:65-68) — requires `Authorization` header (raw PAK or `Bearer <pak>`), missing/invalid → 401. Resolves to `AuthPayload{admin,agent,generator}`; per-handler role checks. Root `/healthz`,`/readyz`,`/metrics` (api/mod.rs) and `/docs/openapi.json`,`/swagger-ui` are mounted OUTSIDE auth → no auth. ApiError statuses (error.rs:70-97): 400/401/403/404/409/422/500; unique-violation→409.

**Endpoint groups (handlers in crates/brokkr-broker/src/api/v1/):** auth (verify_pak), agents (full CRUD + labels/annotations/targets/events/heartbeat/target-state/stacks/rotate-pak), agent-events (admin-only — see discrepancy), deployment-objects (GET with JSON or application/yaml negotiation), stacks (CRUD + deployment-objects + from-template + labels/annotations + events/logs telemetry), templates (CRUD + labels/annotations, versioned), generators (CRUD + rotate-pak), work-orders (list/create/get/delete/claim/complete + log + pending-for-agent), webhooks (CRUD + event-types + deliveries + test + agent-pending + delivery-result), diagnostics (request/get/pending/claim/result), health (PATCH agent health-status, GET deployment/stack health), admin (config/reload, audit-logs, ws/connections).

**FLEET (v0.8.0) — exact routes (corrects earlier assumption):**
- `GET /api/v1/fleet` → `fleet::list_fleet` (registered agents.rs:63 → fleet.rs:331). **admin-only** (require_admin fleet.rs:311). Returns `Vec<FleetAgentRecord>`.
- `GET /api/v1/agents/:id/fleet-status` → `fleet::get_agent_fleet_status` (fleet.rs:355). **admin-only**. Returns `AgentFleetStatusResponse`. **There is NO `/fleet/{id}` route** — per-agent view is under `/agents/{id}/fleet-status`.
- `GET /api/v1/fleet/live` → `ws/fleet_subscribe.rs:97` WS upgrade, **admin-only**. (Not in OpenAPI.)

**Doc gaps in OpenAPI paths():** `/healthz`,`/readyz`,`/metrics`,`/docs/openapi.json`,`/swagger-ui` not documented; 3 WS upgrades intentionally excluded.

**Security-annotation vs enforcement discrepancies (correctness bugs to flag, not document as-is):**
- `agent_events::list_agent_events`/`get_agent_event`: OpenAPI declares admin+agent+generator, code enforces **admin-only** (agent_events.rs:51,89).
- `agents::search_agent` (GET /agents/): OpenAPI admin-only, code also allows matching agent (agents.rs:282).
- `agents::add_target`/`remove_target`: OpenAPI admin+generator; code is admin OR *owning* generator (agents.rs:785).
- `POST /admin/config/reload`: 500s if hot-reload disabled (Extension<ReloadableConfig> missing) before admin check (v1/mod.rs:72-74).
- `complete_work_order` also returns **202** `{"status":"retry_scheduled"}` at runtime (work_orders.rs:645), not just 200.

---

## Agent 5 — Background tasks / telemetry / data model (key facts)

**Background tasks (crates/brokkr-broker/src/utils/background_tasks.rs; wired in cli/commands.rs + api/mod.rs):**
- Diagnostic cleanup (bg:46) — 900s, max-age 1h. Configurable: `broker.diagnostic_cleanup_interval_seconds`/`diagnostic_max_age_hours`.
- Work-order maintenance (bg:114) — 10s, NOT configurable. Retry-pending→pending, reclaim stale claims.
- Agent-metrics refresh (bg:178, T-0226) — 30s, NOT configurable. Recomputes `brokkr_active_agents` + `brokkr_agent_heartbeat_age_seconds` from DB independent of API traffic.
- Webhook delivery worker (bg:268) — 5s, batch 50, configurable. 30s HTTP client timeout.
- Webhook cleanup (bg:504) — 3600s (HARDCODED interval), retention `webhook_cleanup_retention_days`=7.
- Audit-log cleanup (bg:558) — 86400s (HARDCODED), retention `audit_log_retention_days`=90.
- Agent-events cleanup (bg:618, T-0228) — 3600s (HARDCODED), retention `agent_events_retention_days`=30; **only spawned when retention_days>0** (commands.rs:170); 0/unset disables. Hard-deletes agent_events older than retention.
- Fleet live-push sweep (bg:694, T-0230/I-0028) — **20s HARDCODED**. Recomputes all fleet records, broadcasts `FleetUpdate` only for agents whose COMPUTED signals changed (pending_object_count, pending_work_orders, claimed_work_orders, health_failing, health_degraded). First tick seeds baseline only. Pure diff fn `select_changed_fleet_records` (bg:674).
- WS telemetry eviction worker (ws/eviction.rs, spawned api/mod.rs:224) — **6h retention HARD CEILING, 60s tick**. Evicts agent_k8s_events + agent_pod_logs by server `created_at`. Drives `brokkr_ws_log_eviction_runs_total`/`brokkr_ws_telemetry_evicted_total`.
- **Flag:** the "configurable" cleanup tasks only expose retention_days; intervals are hardcoded in commands.rs (struct Default matches) — document interval as fixed.

**Metrics:** `GET /metrics` on the single `0.0.0.0:3000` listener (NO separate metrics port), Prometheus text `version=0.0.4`. Metrics (metrics.rs): brokkr_http_requests_total{endpoint,method,status}, brokkr_http_request_duration_seconds{endpoint,method}, brokkr_active_agents, brokkr_agent_heartbeat_age_seconds{agent_id,agent_name}, brokkr_stacks_total, brokkr_deployment_objects_total, brokkr_ws_connected_agents, brokkr_ws_messages_total{direction,type}, brokkr_ws_live_subscribers, brokkr_fleet_live_subscribers, brokkr_ws_log_eviction_runs_total, brokkr_ws_telemetry_evicted_total{table}. Endpoint labels normalized (UUIDs/ids→:id).
- **REMOVED (commit 5d4fccd) — do NOT document:** `brokkr_database_queries_total`, `brokkr_database_query_duration_seconds`, helper `record_db_query`. Confirmed gone. Stale dashboards/docs referencing them must be fixed.
- OTel tracing (brokkr-utils/telemetry.rs): OTLP/gRPC to `telemetry.otlp_endpoint`; sampler from sampling_rate; honors RUST_LOG via EnvFilter else log_level; disabled→plain subscriber. **Flag:** two parallel logging stacks (tracing fmt layer vs standalone `log`-crate BrokkrLogger to stderr) — verify which the broker activates at startup before documenting.

**Data model (crates/brokkr-models/migrations/):** 21 migrations, 00_common…20_agent_k8s_connectivity.
- Newest `20_agent_k8s_connectivity` (T-0227): adds nullable `k8s_reachable BOOLEAN`, `k8s_api_latency_ms INTEGER`, `k8s_reported_at TIMESTAMPTZ` to agents. Agent self-reports k8s reachability on heartbeat; broker stores latest snapshot; nullable = trust-the-agent/graceful-degradation; `k8s_reported_at` = server ingestion time (freshness).
- `18_agent_telemetry` (I-0019): `agent_k8s_events` + `agent_pod_logs` short-lived buffers (6h eviction). NOT a log warehouse (matches project_log_retention_stance).
- agents table (01_agents): id, timestamps, deleted_at (soft delete), name, cluster_name, last_heartbeat (drives heartbeat_age), status (ACTIVE→brokkr_active_agents), pak_hash, +k8s cols. Model struct crates/brokkr-models/src/models/agents.rs:60-91.
- **Flag:** agents.rs module header comment (lines 14-25) is STALE — omits the 3 k8s columns. Derive column list from struct + migration 20, not the header.

---

## Agent 3 — WS / wire / SDK surface (key facts)

**brokkr-wire (crates/brokkr-wire/src/lib.rs) — INTERNAL, not in OpenAPI/SDKs.** `WsMessage` envelope (`#[serde(tag="type",content="body",rename_all="snake_case")]`, lib.rs:165):
- broker→agent: `WorkOrder`, `TargetChanged`, `StackChanged`
- agent→broker: `Heartbeat`(with #[serde(default)] k8s_reachable/k8s_api_latency_ms — back-compat), `AgentEvent`, `AgentHealth`
- agent→broker telemetry: `K8sEvent`, `PodLogLine`, `LogGap`
- broker→consumer: `FleetUpdate(FleetAgentRecord)` (I-0028) — consumer replaces its row per agent_id.
- `FleetAgentRecord` wire twin (lib.rs:126) has NO serde(default) → evolving it is a breaking wire change (lockstep version). Must stay field-for-field identical to REST twin (api/v1/fleet.rs:39); synced only by `to_wire()` (fleet.rs:79) — drift risk, no compile-time guarantee.

**Broker WS subsystem (crates/brokkr-broker/src/ws/) — 3 distinct surfaces:**
- A. Internal `/internal/ws/agent` (handler.rs:57) — agent-PAK only (admin/generator→403). Two bounded mpsc lanes: control(64) priority-biased over telemetry(1024) so control plane never starved. Fires fleet live-push on connect AND disconnect. Uplink dispatch persists via DAL; LogGap broadcast-only (not persisted); body agent_id mismatch dropped (authenticated id wins).
- B. Stack live-tail `/api/v1/stacks/:id/live` (subscribe.rs:47) — admin (any stack) OR owning generator; agents NOT allowed. Browser PAK via `Sec-WebSocket-Protocol: brokkr.pak.<PAK>`; echoes only `brokkr.v1` marker. Slow subscriber → `Lagged` → synth `LogGap{reason:BufferFull}` (visible gap, ADR-0008).
- C. Fleet live `/api/v1/fleet/live` (fleet_subscribe.rs:49) — **admin-only**; streams every `FleetUpdate` from single fleet-wide `FleetBroadcaster`; drives `brokkr_fleet_live_subscribers`. Slow subscriber → log & continue, **NO gap frame** (latest record per agent_id supersedes).
- Broadcasters (broadcaster.rs): LiveBroadcaster per-stack keyed; FleetBroadcaster single channel, cap 1024, never blocks.
- Push helpers (push.rs): post-commit fire-and-forget; "WS is a hint, REST is source of truth"; NotConnected/LaneUnavailable dropped, REST polling fallback. remove-target NOT pushed in v1.
- Registry (registry.rs): one-connection-per-agent (register evicts prior); generation guard via connected_since; close_for_agent used by PAK-revocation teardown (T-0176).
- Eviction (eviction.rs): 6h HARD ceiling (clamps >6h down), 60s tick.

**SDKs — two layers per language: generated (full coverage incl fleet) + ergonomic wrapper (curated, NO fleet):**
| Surface | list_fleet | get_agent_fleet_status | fleet/live WS helper |
|---|---|---|---|
| Python brokkr-client (generated) | YES (api/fleet/list_fleet.py) | YES | No (WS not generated) |
| Python brokkr (wrapper) | No | No | No |
| TS brokkr-client schema (generated) | YES (schema.d.ts:570) | YES (:276) | No |
| TS brokkr-client wrapper (client.ts) | No | No | only stack tail liveSubscriptionUrl |
| Rust brokkr-client wrapper (wrapper.rs) | via .api() | via .api() | No |
- Wrapper public methods (all 3 langs): submit_manifests/apply, list_telemetry_events, list_telemetry_logs, list_ws_connections, retry. Fleet reachable only via the raw generated client.
- **Flag:** NO `fleet/live` WS URL helper in any SDK — consumers must construct the WS URL themselves. Worth documenting.

---

## Agent 4 — Existing docs audit (key findings)

**UI-as-product hits (the hard constraint):**
- **CLEAR VIOLATION — only one:** `how-to/deployment-health.md:7` — "can be viewed through the API **or UI**." Fix → "through the API (or any UI you build on it)."
- **BORDERLINE (densest UI language):** `explanation/internal-ws-channel.md` (L138/L217/L250-252 "the UI tails/renders…", ui-slim "Go Live" toggle). Acceptable as illustration but qualify as "a consumer (e.g. the ui-slim demo)".
- **EXEMPLARY models to copy:** `reference/container-images.md:15` ("demo only; not currently built or published by CI") and `getting-started/development.md:67,84` ("Demo admin UI"). These are the gold-standard framing.
- `sdks/typescript.md`, `health-endpoints.md:144`, rustdoc comments — NOT violations.
- Minor: container-images.md self-contradiction (says CI doesn't build UI but still gives it a build-stage + size table).

**Fleet observability (the v0.8.0 gap):** Documented ONLY in reference — `reference/api/README.md` ("Fleet Legibility": /fleet, /agents/:id/fleet-status) and `reference/ws-protocol.md` (/fleet/live, fleet_update/FleetAgentRecord). **NO how-to or tutorial; absent from nav's "Observe & debug" grouping.** A newcomer asking "is my fleet healthy?" lands on monitoring-setup.md (Prometheus) or deployment-health.md (per-deployment), neither of which mentions fleet. → strongest candidate for a new task-oriented "monitor your agent fleet" how-to. Also: REST catalog (api/README.md) doesn't cross-ref /fleet/live + FleetUpdate.

**Correctness nits found:**
- `reference/ws-protocol.md` intro says "**two** WebSocket surfaces" but table lists **three** (/internal/ws/agent, /stacks/{id}/live, /fleet/live). Stale count — /fleet/live (I-0028) added after intro written.
- `architecture.md` — verify diagrams are C4-form (user memory feedback_c4_architecture).
- Other reference-only-no-task-guide: GET /admin/ws/connections, POST /auth/pak, /stacks/:id/{health,events,logs}, work-order-log.

**Nav / onboarding assessment:**
1. **Biggest gap: no trial-fast path.** First actionable page (installation.md) assumes a standing k8s cluster + Helm + manual PAK/agent/target curl. The genuinely fast all-in-one (`angreal local up` = broker+agent+k3s in one command) is buried LAST in getting-started under "Local Development Environment" with a contributor framing. Evaluators can't find the fast path.
2. **"Quick Start" misnamed + redundant:** getting-started/quick-start.md is a long curl tutorial requiring install first, duplicating tutorials/first-deployment.md nearly beat-for-beat.
3. **Misfiled getting-started pages:** development.md (really how-to) and quick-start.md (really tutorial) sit under Getting Started — blurs Diátaxis at the section newcomers read first.
4. **No "common integration patterns, start here" signpost** after install — path forks into 18 how-tos; contributor-facing tasks (Building & Publishing Images) sit beside consumption guides.
5. **Integration-pattern verdict:** templates = EXCELLENT (4 quadrants, findable); CLI push/apply = STRONG (cli-apply.md + generators tutorial); monitoring = SPLIT/WEAK as a consumption pattern (infra metrics well covered; application/fleet observability is reference-only, no task guide, no nav presence).

**Auto-generated rustdoc** (docs/src/api/rust/**, 117 files) — skipped per scope; machine-generated from source; not individually audited.
