# Brokkr Documentation Inventory (Diátaxis pass, 2026-06-09)

Working artifact for the docs-diataxis run. Phase 2/3 coverage and the Phase 4
completeness review are measured against this checklist. Not part of the
published book (lives outside `docs/src/`).

Citations are `path:symbol` (line numbers approximate; re-verify when writing).

---

## 1. Entrypoints & top-level surfaces

| Surface | Source | Notes |
|---|---|---|
| `brokkr-broker` binary | `crates/brokkr-broker/src/bin.rs`, `cli/` | subcommands: `serve`, `create {agent,generator}`, `rotate {admin,agent,generator}` |
| `brokkr-agent` binary | `crates/brokkr-agent/src/bin.rs`, `cli/` | single subcommand: `start` |
| README.md | repo root | quick start, claims broker :3000 / UI :3001 |
| openapi spec | `openapi/brokkr-v1.json`, exporter `crates/brokkr-broker/examples/openapi_export.rs` | 3.1→3.0.3 downgrade in exporter |
| Helm charts | `charts/brokkr-broker`, `charts/brokkr-agent` (both 0.5.0) | agent chart kubeVersion >=1.29.0 |
| SDKs | `crates/brokkr-client` (Rust), `sdks/python/brokkr` + `sdks/python/brokkr-client`, `sdks/typescript/brokkr-client` | all 0.5.0, lockstep |
| Examples | `examples/ui-slim` (React admin UI), `examples/webhook-catcher` (python http server) | also `tools/ws-loadtest`, `tools/webhook-catcher` |

## 2. Broker CLI

- `serve` — `cli/commands.rs:serve()`: runs migrations, first-run admin-role creation, starts background tasks, listens 0.0.0.0:3000, pool size 50, graceful SIGINT shutdown.
- `create agent --name --cluster-name` → prints PAK once.
- `create generator --name [--description]` → prints PAK once.
- `rotate admin` / `rotate agent --uuid` / `rotate generator --uuid` — `cli/commands.rs:rotate_*`.

## 3. Agent CLI & lifecycle

- `start` — `crates/brokkr-agent/src/cli/commands.rs:start()`.
- Startup: settings → telemetry → wait_for_broker_ready (max_retries) → verify_agent_pak → SDK client → spawn WS (`broker_ws::spawn`) → k8s client → spawn kube_events + pod_logs tailers → health server (:8080) → main select! loop.
- Loop timers: heartbeat/deployments/work-orders every `agent.polling_interval`; deployment health every `agent.deployment_health_interval` (60); diagnostics every 10s (hardcoded); webhook poll every 10s (hardcoded).
- Failure modes: broker unreachable → retries then exit(1); bad PAK → exit; WS auth rejected 5x → WS permanently down (REST continues), needs PAK rotation + restart.

## 4. Broker REST API (all under /api/v1 unless noted)

Auth tiers: admin PAK, agent PAK (own id), generator PAK (own resources). Router: `crates/brokkr-broker/src/api/v1/mod.rs`; middleware `middleware.rs:auth_middleware` (Bearer or raw header; moka cache TTL `broker.auth_cache_ttl_seconds` default 60, 0=off).

- Root (no /api/v1): `GET /healthz`, `GET /readyz`, `GET /metrics` (public) — `api/mod.rs`.
- Auth: `POST /auth/pak` → AuthResponse {admin, agent, generator}.
- Agents: CRUD `GET|POST /agents`, `GET|PUT|DELETE /agents/{id}`, `POST /agents/{id}/rotate-pak`; heartbeat `POST /agents/{id}/heartbeat` (agent-only, no admin override); labels/annotations/targets subresources (GET/POST/DELETE); `GET /agents/{id}/target-state`; `GET /agents/{id}/stacks`; events `GET|POST /agents/{id}/events`; `GET /agents/{id}/work-orders/pending`; `GET /agents/{id}/diagnostics/pending`; `GET /agents/{agent_id}/webhooks/pending` (auto-claims).
- Agent events global: `GET /agent-events`, `GET /agent-events/{id}` (any PAK).
- Generators: CRUD + rotate-pak — `generators.rs`.
- Stacks: CRUD (generator-scoped ownership); deployment-objects `GET|POST /stacks/{id}/deployment-objects`, `POST .../from-template`; labels/annotations subresources; telemetry `GET /stacks/{id}/events`, `GET /stacks/{id}/logs` (6h window); health `GET /stacks/{id}/health` (computed, not stored).
- Deployment objects: `GET /deployment-objects/{id}`; health `GET /deployment-objects/{id}/health`; diagnostics `POST /deployment-objects/{id}/diagnostics`.
- Health reporting: `PATCH /agents/{id}/health-status` — `health.rs:update_health_status`.
- Templates: CRUD + labels/annotations — `templates.rs`.
- Work orders: `GET|POST /work-orders`, `GET|DELETE /work-orders/{id}`, `POST /work-orders/{id}/claim`, `POST /work-orders/{id}/complete`; log `GET /work-order-log`, `GET /work-order-log/{id}`.
- Diagnostics: `GET /diagnostics/{id}`, `POST /diagnostics/{id}/claim`, `POST /diagnostics/{id}/result`.
- Webhooks: CRUD `/webhooks`, `GET /webhooks/event-types`, `POST /webhooks/{id}/test`, `GET /webhooks/{id}/deliveries`, `POST /webhooks/deliveries/{id}/result`.
- Admin: `POST /admin/config/reload`, `GET /admin/audit-logs` (filters + limit default 100 max 1000 + offset), `GET /admin/ws/connections`.
- OpenAPI: `GET /docs/openapi.json` (public), Swagger UI at `/swagger-ui`.
- Errors: `error.rs` — `{code, message, details}`; diesel mapping unique→409, FK/check/not-null→422, not-found→404, else 500. Stable `code` strings are SDK contract.

## 5. WebSocket surfaces

- `GET /internal/ws/agent` — agent-only upgrade (`ws/handler.rs:ws_upgrade`); wire = `brokkr_wire::WsMessage`; dual lanes: control cap 64 (work orders, target/stack changed, heartbeat) drains before telemetry cap 1024 (K8sEvent, PodLogLine, DeploymentHealth/AgentHealth, AgentEvent). Not in OpenAPI (ADR-0008).
- `GET /api/v1/stacks/{id}/live` — live tail fan-out (`ws/subscribe.rs:live_upgrade`); per-stack broadcast cap 1024; lagged → synthetic `LogGap`; browser auth via `Sec-WebSocket-Protocol: brokkr.pak.<PAK>, brokkr.v1` (broker echoes only `brokkr.v1`).
- Registry `ws/registry.rs`; push helpers `ws/push.rs` (fire-and-forget, post-commit); eviction `ws/eviction.rs` (HARD_RETENTION_CEILING 6h, tick 60s, server-side created_at).
- Wire crate: `crates/brokkr-wire/src/lib.rs` — externally tagged JSON `{type, body}` snake_case; variants WorkOrder, TargetChanged, StackChanged, Heartbeat, AgentEvent, AgentHealth, K8sEvent, PodLogLine, LogGap; WIRE_VERSION = crate version; golden tests `tests/golden.rs`.

## 6. Configuration

Loading: embedded `crates/brokkr-utils/default.toml` → optional file → env `BROKKR__` prefix `__` separator (`brokkr-utils/src/config.rs:Settings`).

- database: `url` (default postgres://brokkr:brokkr@localhost:5433/brokkr), `schema` (multi-tenant, validated).
- log: `level` (default **debug**), `format` (text|json).
- pak: prefix/digest/rng/short_token_length/long_token_length/short_token_prefix.
- broker: `pak_hash` (default-admin hash), `auth_cache_ttl_seconds` 60, `diagnostic_cleanup_interval_seconds` 900, `diagnostic_max_age_hours` 1, `webhook_encryption_key` (32-byte hex; random+logged if unset — rotation breaks decryption), `webhook_delivery_interval_seconds` 5, `webhook_delivery_batch_size` 50, `webhook_cleanup_retention_days` 7, `audit_log_retention_days` 90.
- agent: `broker_url`, `polling_interval` 10 (chart default 30), `kubeconfig_path` (literal `${USER}` not expanded!), `max_retries` 60, `pak`, `agent_name`, `cluster_name`, `max_event_message_retries` 2, `event_message_retry_delay` 5, `health_port` 8080, `deployment_health_enabled` true, `deployment_health_interval` 60, `ws_force_rest` false, `ws_url` (override), `kube_event_uid_cache_cap` 10000.
- cors: `allowed_origins` ["http://localhost:3001"] (string-or-vec deserialize), methods, headers, `max_age_seconds` 3600.
- telemetry: enabled false, otlp_endpoint http://localhost:4317, service_name, sampling_rate 0.1; per-component overrides `telemetry.broker.*` / `telemetry.agent.*`.
- Hot-reload (`config.rs:DynamicConfig` + `ReloadableConfig`): log.level, diagnostic_cleanup_interval_seconds, diagnostic_max_age_hours, webhook_delivery_interval_seconds, webhook_delivery_batch_size, webhook_cleanup_retention_days, cors.allowed_origins, cors.max_age_seconds. Everything else = restart.
- Watcher env vars (bypass Settings): `BROKKR_CONFIG_FILE`, `BROKKR_CONFIG_WATCHER_ENABLED` (true), `BROKKR_CONFIG_WATCHER_DEBOUNCE_SECONDS` (5) — `utils/config_watcher.rs`. Manual reload endpoint also works without watcher.
- `KUBECONFIG` set programmatically from agent.kubeconfig_path — `k8s/api.rs`.
- Encryption at rest: AES-256-GCM, version byte 0x01 (legacy 0x00 XOR read-only) — `utils/encryption.rs`.

## 7. Background tasks (broker)

`utils/background_tasks.rs`: diagnostic cleanup (900s/1h max age); work-order maintenance (10s: RETRY_PENDING→PENDING, reclaim stale claims); webhook delivery (5s, batch 50, 30s HTTP timeout, broker delivers target_labels IS NULL); webhook cleanup (3600s/7d); audit log cleanup (daily/90d). Plus `ws/eviction.rs` telemetry eviction (60s tick/6h ceiling).

## 8. Data model (brokkr-models)

Entities & relationships (see `src/models/*.rs`, schema `src/schema.rs`, migrations `crates/brokkr-models/migrations/00..18`):
- agents (unique name+cluster_name, pak_hash, last_heartbeat, soft-delete) → labels, annotations, targets, events, health, claimed work orders.
- generators (pak_hash, is_active, soft-delete) → stacks, templates.
- stacks (generator_id FK, soft-delete) → deployment_objects (sequence_id BIGSERIAL ordering, yaml_content immutable, sha-256 yaml_checksum, is_deletion_marker), labels, annotations.
- stack_templates (version auto-increment per (generator_id,name), Tera content, JSON Schema params, checksum, NULL generator = system template) → labels, annotations, template_targets; rendered_deployment_objects = provenance (template_id, version, params).
- agent_targets (agent_id, stack_id unique pair).
- work_orders (status PENDING|CLAIMED|RETRY_PENDING, claim_timeout 3600, max_retries 3, backoff 60 exp) → work_order_targets, labels, annotations; completed → work_order_log (immutable, same id).
- agent_events (SUCCESS|FAILURE per deployment object).
- agent_k8s_events, agent_pod_logs (6h retention telemetry; JSONB involved_object).
- deployment_health (status healthy|degraded|failing|unknown, JSON summary, per agent+deployment-object).
- diagnostic_requests (pending|claimed|completed|failed|expired, expires_at; retention 1–1440 min default 60) → diagnostic_results.
- webhook_subscriptions (url/auth encrypted BYTEA, event_types array + wildcards, target_labels NULL=broker-delivers, max_retries 5, timeout 30) → webhook_deliveries (pending|acquired|success|failed|dead, event_id idempotency).
- audit_logs (actor_type admin|agent|generator|system, action constants, JSONB details).
- Soft-deletion: `deleted_at IS NULL` filters in DAL; migrations 14/17 fix cascade + unique constraints.
- Multi-tenancy: `db.rs` schema-per-tenant via validated `search_path`.

## 9. Engines

- Templating `utils/templating.rs`: Tera; JSON Schema validation (jsonschema crate); instantiate → render → create deployment object + rendered_deployment_objects row.
- Matching `utils/matching.rs:template_matches_stack`: no labels/annotations = match all; otherwise ALL labels AND ALL annotation pairs must be present.
- Event bus `utils/event_bus.rs:emit_event`: matches subscriptions (wildcards e.g. `deployment.*`) → inserts webhook_deliveries. Event types: agent.registered/deregistered, stack.created/deleted, deployment.created/applied/failed/deleted, workorder.created/claimed/completed/failed.

## 10. Agent workflows

- Reconciliation: fetch target state via SDK → `k8s/objects.rs:create_k8s_objects` (annotations: `k8s.brokkr.io/stack`, `k8s.brokkr.io/deployment-checksum`, `brokkr.io/deployment-object-id`, `k8s.brokkr.io/last-config-applied`, `brokkr.io/owner-id`; Namespaces/CRDs first) → `k8s/api.rs:reconcile_target_state` (server-side apply, field manager `brokkr-controller`, force=true, backoff 1s→60s, 5min cap; prune by checksum mismatch; skip objects with ownerReferences).
- WS client `broker_ws.rs`: states Down/Up/ForceRestOnly/AuthRejected; backoff 1s→60s; 5 auth rejections = terminal; queues 256 each way; `WsUplink::try_send` → REST fallback.
- Pod logs `pod_logs.rs`: opt-in `brokkr.io/stream-logs: "true"` + stack annotation; 100 lines/s/container token bucket; LogGap on drop.
- Kube events `kube_events.rs`: cluster watch, ownership via stack annotation, LRU uid cache 10k/5min TTL.
- Health `deployment_health.rs`: pods by `brokkr.io/deployment-object-id` label; DEGRADED_CONDITIONS (ImagePullBackOff, CrashLoopBackOff, …), TERMINATED_ISSUES (OOMKilled, …); statuses healthy/degraded/failing/unknown.
- Agent health server `health.rs`: /healthz, /readyz (k8s reachable), /health (JSON), /metrics on :8080.
- Diagnostics `diagnostics.rs`: poll→claim→collect (pod statuses, events, 100-line log tails)→submit. KNOWN LIMITATION: namespace hardcoded "default".
- Work orders `work_orders/`: poll→claim→execute by work_type (`build` = Shipwright Build+BuildRun, poll 5s, timeout 900s, digest returned; `custom` placeholder)→complete (202 = retry scheduled).
- Webhook delivery (agent side) `webhooks.rs`: poll pending (broker decrypts URL/auth), POST, report result.

## 11. Metrics

Broker (`brokkr-broker/src/metrics.rs`, GET /metrics): brokkr_http_requests_total{endpoint,method,status}, brokkr_http_request_duration_seconds{endpoint,method}, brokkr_database_queries_total{query_type}, brokkr_database_query_duration_seconds{query_type}, brokkr_active_agents, brokkr_agent_heartbeat_age_seconds{agent_id,agent_name}, brokkr_stacks_total, brokkr_deployment_objects_total, brokkr_ws_connected_agents, brokkr_ws_messages_total{direction,type}, brokkr_ws_live_subscribers, brokkr_ws_log_eviction_runs_total, brokkr_ws_telemetry_evicted_total{table}. Endpoint label normalizes ids → `:id`.

Agent (`brokkr-agent/src/metrics.rs`, :8080/metrics): brokkr_agent_poll_requests_total{status}, brokkr_agent_poll_duration_seconds, brokkr_agent_kubernetes_operations_total{operation}, brokkr_agent_kubernetes_operation_duration_seconds{operation}, brokkr_agent_heartbeat_sent_total, brokkr_agent_last_successful_poll_timestamp.

Dashboards: `docs/grafana/brokkr-{agent,broker,ws-channel}-dashboard.json`, alert rules `brokkr-ws-channel.rules.yml`.

## 12. SDKs / OpenAPI pipeline

- Spec: `openapi/brokkr-v1.json` ← `angreal openapi export` (runs `cargo run -p brokkr-broker --example openapi_export`); mirrored to `crates/brokkr-client/spec/brokkr-v1.json`. Drift checks: `angreal openapi check{,-python,-typescript}`. redocly.yaml = minimal ruleset.
- Rust: progenitor `generate_api!` at build; wrapper `BrokkrClient`/`BrokkrError` (Api/Transport/UnexpectedResponse/InvalidRequest; `.code()`, `.is_retryable()`, `retry()`).
- Python: PyPI `brokkr-client` (import `brokkr`) wraps generated `brokkr-client-generated` (import `brokkr_broker_client`, openapi-python-client@0.28.4). Constructor: base_url, token, request_timeout 30, connect_timeout 10, max_retries 3, initial_backoff 0.2; `BrokkrError` dataclass; retryable {408,429,502,503,504}; `TemplateGenerator` alias.
- TypeScript: npm `@colliery-io/brokkr-client`; openapi-typescript@7.13.0 → schema.d.ts; `createBrokkrClient` (openapi-fetch) + `BrokkrClient` wrapper + `BrokkrError`.
- Never hand-edit: spec, schema.d.ts, generated python package, progenitor output.
- Contract tests: `tests/sdk-contract/{rust,python,typescript}` via `angreal tests sdk-contract <lang|all>`; env BROKER_URL/ADMIN_PAK.
- plissken.toml → `plissken render` → `docs/src/api/rust/` (generated; SUMMARY.md rewritten by `angreal docs build|serve`).

## 13. Dev / build / test / release lifecycle

- Local dev: `angreal local up|down|reset|clean|rebuild <svc>|docs`; compose `.angreal/files/docker-compose.yaml`, project `brokkr-dev` (suites must not run in parallel). Services/ports: postgres 5433, registry 5050, broker 3000, toxiproxy 8474 (WS chaos on 8666), k3s 6443, agent health 8080 (internal), ui 3001, webhook-catcher 8090. Keys land in /tmp/brokkr-keys/ (agent.pak, kubeconfigs). Demo admin PAK `brokkr_BR3rVsDa_GK3QN7CDUzYc6iKgMkJ98M2WSimM5t6U8`.
- Tests: `angreal tests unit <crate|all>` (also brokkr-wire), `integration <crate|all> [--skip-docker]`, `sdk-contract <lang|all>`, `e2e [--scenario]` (mirrors `examples/ui-slim/DEMO_WALKTHROUGH.md`, env E2E_SCENARIO).
- Models: `angreal models schema|migrations|test` — diesel CLI required; DATABASE_URL port 5432 vs compose 5433 gotcha.
- Build: `angreal build multi-arch <component> [--push --tag --platforms --registry]` (buildx, ghcr.io/colliery-io).
- Helm: `angreal helm test` — isolated k3s + local registry, template validation for dev/staging/prod values, RBAC modes, external-DB and multi-tenant scenarios.
- Docs: `angreal docs build|serve` (plissken render → mdbook; serve on :3000 — collides with broker).
- CI: `.github/workflows/` main.yml (unit/integration/sdk-contract, path-filtered), build-and-test.yml (multi-arch images + helm tests), release.yml (tag `v*`: full tests → build → manual approval env `release` → push ghcr + GitHub Release w/ charts), openapi.yml, docs.yml, nightly.yml, cleanup-pr.yml. Lockstep versioning, currently 0.5.0.
- Toolchain: Rust edition 2024, rust-version 1.85 (verify), mdbook, plissken, diesel CLI, docker, helm, npx, uv.

## 14. Tribal knowledge (must surface in docs)

1. Default log level is `debug`; default admin PAK is publicly known — both must be changed in production.
2. `webhook_encryption_key` unset = random per boot → encrypted webhook URLs unreadable after restart.
3. `agent.kubeconfig_path` default contains literal `${USER}` — not expanded.
4. WS is an optimization; REST polling is load-bearing (ADR-0008). 5 WS auth rejections = terminal until restart.
5. Checksum mismatch ⇒ pruning deletes old objects; ownerReferences exempt.
6. Log streaming is per-pod opt-in annotation; 6h hard ceiling; "use Datadog" stance.
7. Agent currently requires cluster-wide RBAC (namespace-scoped mode documented but non-functional, charts/brokkr-agent/RBAC.md).
8. Diagnostics namespace hardcoded to "default": when the agent fulfills an on-demand diagnostics request, the Kubernetes namespace it searches is the unconditional literal `"default"` (`crates/brokkr-agent/src/cli/commands.rs:387`, passed to `collect_diagnostics()` in `diagnostics.rs:146`, scoping pod statuses, events, and log tails). The pod label selector (`brokkr.io/deployment-object-id=<id>`) is correct, but it is evaluated in the wrong namespace for any workload not deployed to `default` — so the agent finds zero pods and submits an empty-but-successful result (`pod_statuses: []`) with no error. A code comment marks this as a known TODO ("should be derived from the deployment object"). The other `"default"` literals in the agent (reconciler `k8s/api.rs:582`, Shipwright builds `work_orders/build.rs:121`) are normal fallbacks for manifests that omit `metadata.namespace`, not hardcoding.
9. Heartbeat endpoint rejects admin PAK (agent-only).
10. Pending-webhooks poll auto-claims deliveries.
11. brokkr-dev compose project is shared — no parallel suites.
12. mdbook serve and broker both default to port 3000.
13. Postgres 5433 (compose) vs 5432 (models tasks).
14. Stack health is computed on read, never stored.
15. ~~One deployment object processed per poll tick~~ — WRONG, verified against code: the agent applies every fetched deployment object each tick (`cli/commands.rs` for-loop). Also stale: `custom` work orders are not a placeholder — `execute_custom_work_order` applies YAML.

## 15. Existing docs map (mdBook, docs/src) — refreshed 2026-06-09 post-fix-pass

Quadrants: tutorials/ (first-deployment, templates, multi-cluster-targeting, cicd-generators); how-to/ (install-operations, shipwright-builds, build-and-publish-images, webhooks, managing-stacks, generators, deployment-health, log-streaming, troubleshoot-reconciliation, templates, diagnostics, pak-management, multi-tenant-setup, audit-logs, monitoring-setup, network-configuration, security-hardening, sdks/{rust,python,typescript,regeneration}); reference/ (cli, environment-variables, error-codes, templates, work-orders, webhooks, generators, diagnostics, multi-tenancy, soft-deletion, audit-logs, health-endpoints, deployment-health, ws-protocol, agent-annotations, network-ports, container-images, monitoring, api/); explanation/ (core-concepts, architecture, components, data-model, network-flows, data-flows, security-model, publishing-strategy, work-orders, template-system, reconciliation, internal-ws-channel); getting-started/ (installation, quick-start, configuration, development). Deleted: how-to/understanding-reconciliation.md (split into explanation/reconciliation + how-to/troubleshoot-reconciliation), how-to/sdks/errors.md (moved to reference/error-codes). Generated: api/rust/** (plissken — do not hand-edit), openapi/.
