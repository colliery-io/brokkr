---
id: agent-misc-hardening-sigterm
level: task
title: "Agent: misc hardening — SIGTERM, namespace fallbacks, tracked-object growth, dead config"
short_code: "BROKKR-T-0206"
created_at: 2026-06-11T11:02:07.827265+00:00
updated_at: 2026-06-11T11:02:07.827265+00:00
parent: agent-reconciler-hardening-crash
blocked_by: []
archived: false

tags:
  - "#task"
  - "#phase/todo"
  - "#task"
  - "#phase/todo"


exit_criteria_met: false
initiative_id: BROKKR-I-0023
---

# Agent: misc hardening — SIGTERM, namespace fallbacks, tracked-object growth, dead config

## Parent Initiative

[[BROKKR-I-0023]]

## Objective

Checklist of small, independent hardening items from the sweep — each a contained fix.

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
