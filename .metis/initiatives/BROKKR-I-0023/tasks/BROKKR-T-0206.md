---
id: agent-misc-hardening-sigterm
level: task
title: "Agent: misc hardening — SIGTERM, namespace fallbacks, tracked-object growth, dead config"
short_code: "BROKKR-T-0206"
created_at: 2026-06-11T11:02:07.827265+00:00
updated_at: 2026-06-11T14:46:44.115781+00:00
parent: agent-reconciler-hardening-crash
blocked_by: []
archived: false

tags:
  - "#task"
  - "#task"
  - "#phase/completed"


exit_criteria_met: false
initiative_id: BROKKR-I-0023
---

# Agent: misc hardening — SIGTERM, namespace fallbacks, tracked-object growth, dead config

## Parent Initiative

[[BROKKR-I-0023]]

## Objective

Checklist of small, independent hardening items from the sweep — each a contained fix.

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] SIGTERM handled: `cli/commands.rs:184-190` selects only `ctrl_c()` (SIGINT); k8s sends SIGTERM → no graceful drain, no `telemetry::shutdown()`. Module docs (`:40-45`) already claim SIGTERM. Add `signal(SignalKind::terminate())`.
- [ ] Namespace fallbacks honor `watch_namespace`: six `"default"` fallbacks in `k8s/api.rs:171-176, 392-397, 473-478, 578-583, 783-788, 847-852` default to `watch_namespace` when set; factor into one helper. (The diagnostics fallback `commands.rs:404-409` is deliberate per BROKKR-T-0190 — leave it.)
- [ ] `tracked_deployment_objects` (`commands.rs:204, 281-283`) no longer grows unbounded: rebuild from the latest target-state fetch each cycle instead of insert-only.
- [ ] Dead config keys `agent.max_event_message_retries` / `agent.event_message_retry_delay` (`brokkr-utils/src/config.rs:200-202`, referenced nowhere): implement or remove.
- [ ] Error payload `commands.rs:434` built via `format!("[{{\"error\": \"{}\"}}]", e)` breaks on quotes → `serde_json::json!`.
- [ ] `let _ =` on `submit_diagnostic_result` error path (`commands.rs:438`) → log at error level.
- [ ] `LAST_CONFIG_ANNOTATION` (`k8s/objects.rs:95`) stores `format!("{:?}", obj)` (Debug dump; can blow the 256KiB annotation limit) → store checksum or canonical JSON.
- [ ] Discovery built once per `reconcile_target_state` and reused (`api.rs:606, 811, 884` rebuild per object).
- [ ] Watcher restart loops get exponential backoff with cap (`kube_events.rs:163`, `pod_logs.rs:83` — fixed 5s forever, even on persistent RBAC denial), mirroring `broker_ws::BackoffSchedule`.
- [ ] Pod-log tails can re-attach: remove UID from `active` when tail tasks finish (`pod_logs.rs:148-185, 230-234, 281` — currently a pod that EOFs once is never tailed again).
- [ ] Clippy: `broker_ws.rs:147` `result_large_err` (box or #[allow] with comment); `pod_logs.rs:155,187` `type_complexity` (type alias).

## Implementation Notes

Stretch (do only if cheap): `work_orders/mod.rs:50-104` `is_error_retryable` classifies by substring matching on error strings — classify on typed errors first. Startup `wait_for_broker_ready` (`broker.rs:104-137`) fixed 1s retry + `exit(1)` skipping telemetry shutdown — low value, k8s restart covers it.

## Status Updates

*To be added during implementation*
## Status Updates

- 2026-06-11: DONE (9 of 11 checklist items; 2 split to [[BROKKR-T-0221]]). Implemented on branch feat/i0023-agent-reconciler-hardening:
  - **SIGTERM**: shutdown task now selects ctrl_c() OR SIGTERM (tokio::signal::unix; cfg(unix)) — k8s pod termination now drains gracefully and flushes telemetry. (commands.rs)
  - **Diagnostics error payload**: `format!("[{{\"error\":...}}]")` → `serde_json::json!([{ "error": e.to_string() }])` (no JSON injection on quoted error messages); the `let _ =` on submit_diagnostic_result now logs failures. (commands.rs)
  - **Dead config keys**: removed `agent.max_event_message_retries` / `event_message_retry_delay` from Settings + default.toml (referenced nowhere; serde ignores stale keys in user configs).
  - **LAST_CONFIG_ANNOTATION**: stored the bounded yaml_checksum instead of `format!("{:?}", obj)` (Debug dump could push total annotations past the 256 KiB limit and fail apply; value is only ever presence-checked).
  - **tracked_deployment_objects**: was insert-only (unbounded). Now rebuilt from each deployment cycle's applied ids, so superseded objects stop being health-checked.
  - **Watcher backoff**: kube_events.rs and pod_logs.rs restart loops now use capped exponential backoff (1s→60s, reset on clean exit) instead of a fixed 5s, so a persistent RBAC denial doesn't re-dial every 5s forever.
  - **Namespace fallback**: reconcile apply + prune namespace fallback now `.or(watch_namespace)` before "default" (a manifest with no explicit namespace lands in the agent's watch_namespace under namespace-scoped RBAC — same class as the prior hardcoded-default bug). The cluster-scoped priority-object site (apply_single_object) is unaffected (namespace ignored for Scope::Cluster).
  - **Clippy**: broker_ws.rs try_send `#[allow(result_large_err)]` (the large Err is intentional — returns the message for REST fallback); pod_logs.rs `type ActiveTails` alias for the complex type. Agent clippy now fully clean.
  DEFERRED to T-0221 (each needs its own change + integration test): Discovery reuse in reconcile (interacts with T-0203 fail-closed / CRD establishment timing) and pod-log tail re-attach (behavioral, handle-ownership race). Build clean, 73 agent + 24 utils unit tests pass, integration compiles, clippy clean.