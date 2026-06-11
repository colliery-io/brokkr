# Code Index

> Generated: 2026-06-11T14:47:19Z | 398 files | JavaScript, Python, Rust, TypeScript

## Project Structure

```
├── crates/
│   ├── brokkr-agent/
│   │   ├── src/
│   │   │   ├── bin.rs
│   │   │   ├── broker.rs
│   │   │   ├── broker_sdk.rs
│   │   │   ├── broker_ws.rs
│   │   │   ├── cli/
│   │   │   │   ├── commands.rs
│   │   │   │   └── mod.rs
│   │   │   ├── deployment_health.rs
│   │   │   ├── diagnostics.rs
│   │   │   ├── health.rs
│   │   │   ├── k8s/
│   │   │   │   ├── api.rs
│   │   │   │   ├── mod.rs
│   │   │   │   └── objects.rs
│   │   │   ├── kube_events.rs
│   │   │   ├── lib.rs
│   │   │   ├── metrics.rs
│   │   │   ├── pod_logs.rs
│   │   │   ├── utils.rs
│   │   │   ├── webhooks.rs
│   │   │   └── work_orders/
│   │   │       ├── broker.rs
│   │   │       ├── build.rs
│   │   │       └── mod.rs
│   │   └── tests/
│   │       ├── fixtures.rs
│   │       └── integration/
│   │           ├── broker.rs
│   │           ├── broker_ws.rs
│   │           ├── deployment_health.rs
│   │           ├── diagnostics.rs
│   │           ├── health.rs
│   │           ├── k8s/
│   │           │   ├── api.rs
│   │           │   ├── mod.rs
│   │           │   └── objects.rs
│   │           └── main.rs
│   ├── brokkr-broker/
│   │   ├── examples/
│   │   │   └── openapi_export.rs
│   │   ├── src/
│   │   │   ├── api/
│   │   │   │   ├── mod.rs
│   │   │   │   └── v1/
│   │   │   │       ├── admin.rs
│   │   │   │       ├── agent_events.rs
│   │   │   │       ├── agents.rs
│   │   │   │       ├── auth.rs
│   │   │   │       ├── deployment_objects.rs
│   │   │   │       ├── diagnostics.rs
│   │   │   │       ├── error.rs
│   │   │   │       ├── generators.rs
│   │   │   │       ├── health.rs
│   │   │   │       ├── middleware.rs
│   │   │   │       ├── mod.rs
│   │   │   │       ├── openapi.rs
│   │   │   │       ├── stacks.rs
│   │   │   │       ├── templates.rs
│   │   │   │       ├── webhooks.rs
│   │   │   │       └── work_orders.rs
│   │   │   ├── bin.rs
│   │   │   ├── cli/
│   │   │   │   ├── commands.rs
│   │   │   │   └── mod.rs
│   │   │   ├── dal/
│   │   │   │   ├── agent_annotations.rs
│   │   │   │   ├── agent_events.rs
│   │   │   │   ├── agent_k8s_events.rs
│   │   │   │   ├── agent_labels.rs
│   │   │   │   ├── agent_pod_logs.rs
│   │   │   │   ├── agent_targets.rs
│   │   │   │   ├── agents.rs
│   │   │   │   ├── audit_logs.rs
│   │   │   │   ├── deployment_health.rs
│   │   │   │   ├── deployment_objects.rs
│   │   │   │   ├── diagnostic_requests.rs
│   │   │   │   ├── diagnostic_results.rs
│   │   │   │   ├── generators.rs
│   │   │   │   ├── mod.rs
│   │   │   │   ├── rendered_deployment_objects.rs
│   │   │   │   ├── stack_annotations.rs
│   │   │   │   ├── stack_labels.rs
│   │   │   │   ├── stacks.rs
│   │   │   │   ├── template_annotations.rs
│   │   │   │   ├── template_labels.rs
│   │   │   │   ├── template_targets.rs
│   │   │   │   ├── templates.rs
│   │   │   │   ├── webhook_deliveries.rs
│   │   │   │   ├── webhook_subscriptions.rs
│   │   │   │   └── work_orders.rs
│   │   │   ├── db.rs
│   │   │   ├── lib.rs
│   │   │   ├── metrics.rs
│   │   │   ├── utils/
│   │   │   │   ├── audit.rs
│   │   │   │   ├── background_tasks.rs
│   │   │   │   ├── config_watcher.rs
│   │   │   │   ├── encryption.rs
│   │   │   │   ├── event_bus.rs
│   │   │   │   ├── matching.rs
│   │   │   │   ├── mod.rs
│   │   │   │   ├── pak.rs
│   │   │   │   └── templating.rs
│   │   │   └── ws/
│   │   │       ├── broadcaster.rs
│   │   │       ├── eviction.rs
│   │   │       ├── handler.rs
│   │   │       ├── mod.rs
│   │   │       ├── push.rs
│   │   │       ├── registry.rs
│   │   │       └── subscribe.rs
│   │   └── tests/
│   │       ├── fixtures.rs
│   │       └── integration/
│   │           ├── api/
│   │           │   ├── admin.rs
│   │           │   ├── agent_events.rs
│   │           │   ├── agents.rs
│   │           │   ├── audit_logs.rs
│   │           │   ├── auth.rs
│   │           │   ├── deployment_objects.rs
│   │           │   ├── diagnostics.rs
│   │           │   ├── generators.rs
│   │           │   ├── health.rs
│   │           │   ├── mod.rs
│   │           │   ├── stacks.rs
│   │           │   ├── templates.rs
│   │           │   ├── webhooks.rs
│   │           │   ├── work_orders.rs
│   │           │   └── ws.rs
│   │           ├── cli.rs
│   │           ├── dal/
│   │           │   ├── agent_annotations.rs
│   │           │   ├── agent_events.rs
│   │           │   ├── agent_labels.rs
│   │           │   ├── agent_targets.rs
│   │           │   ├── agents.rs
│   │           │   ├── deployment_health.rs
│   │           │   ├── deployment_objects.rs
│   │           │   ├── diagnostic_requests.rs
│   │           │   ├── diagnostic_results.rs
│   │           │   ├── event_emission.rs
│   │           │   ├── generators.rs
│   │           │   ├── mod.rs
│   │           │   ├── stack_annotations.rs
│   │           │   ├── stack_labels.rs
│   │           │   ├── stacks.rs
│   │           │   ├── templates.rs
│   │           │   ├── webhook_deliveries.rs
│   │           │   ├── webhook_subscriptions.rs
│   │           │   └── work_orders.rs
│   │           ├── db/
│   │           │   ├── mod.rs
│   │           │   └── multi_tenant.rs
│   │           └── main.rs
│   ├── brokkr-cli/
│   │   ├── src/
│   │   │   ├── config.rs
│   │   │   └── main.rs
│   │   └── tests/
│   │       └── cli.rs
│   ├── brokkr-client/
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   └── wrapper.rs
│   │   └── tests/
│   │       └── surface.rs
│   ├── brokkr-models/
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── models/
│   │       │   ├── agent_annotations.rs
│   │       │   ├── agent_events.rs
│   │       │   ├── agent_k8s_events.rs
│   │       │   ├── agent_labels.rs
│   │       │   ├── agent_pod_logs.rs
│   │       │   ├── agent_targets.rs
│   │       │   ├── agents.rs
│   │       │   ├── audit_logs.rs
│   │       │   ├── deployment_health.rs
│   │       │   ├── deployment_objects.rs
│   │       │   ├── diagnostic_requests.rs
│   │       │   ├── diagnostic_results.rs
│   │       │   ├── generator.rs
│   │       │   ├── mod.rs
│   │       │   ├── rendered_deployment_objects.rs
│   │       │   ├── stack_annotations.rs
│   │       │   ├── stack_labels.rs
│   │       │   ├── stack_templates.rs
│   │       │   ├── stacks.rs
│   │       │   ├── template_annotations.rs
│   │       │   ├── template_labels.rs
│   │       │   ├── template_targets.rs
│   │       │   ├── webhooks.rs
│   │       │   ├── work_order_annotations.rs
│   │       │   ├── work_order_labels.rs
│   │       │   └── work_orders.rs
│   │       └── schema.rs
│   ├── brokkr-utils/
│   │   ├── src/
│   │   │   ├── config.rs
│   │   │   ├── lib.rs
│   │   │   ├── logging.rs
│   │   │   └── telemetry.rs
│   │   └── tests/
│   │       └── integration.rs
│   └── brokkr-wire/
│       ├── src/
│       │   └── lib.rs
│       └── tests/
│           └── golden.rs
├── docs/
│   ├── mermaid-init.js
│   └── mermaid.min.js
├── examples/
│   ├── ui-slim/
│   │   └── src/
│   │       ├── App.js
│   │       ├── api.js
│   │       ├── components.js
│   │       └── index.js
│   └── webhook-catcher/
│       └── main.py
├── sdks/
│   ├── python/
│   │   ├── brokkr/
│   │   │   ├── brokkr/
│   │   │   │   ├── __init__.py
│   │   │   │   ├── client.py
│   │   │   │   └── errors.py
│   │   │   └── tests/
│   │   │       └── test_wrapper.py
│   │   └── brokkr-client/
│   │       ├── brokkr_broker_client/
│   │       │   ├── __init__.py
│   │       │   ├── api/
│   │       │   │   ├── __init__.py
│   │       │   │   ├── admin/
│   │       │   │   │   ├── __init__.py
│   │       │   │   │   ├── list_audit_logs.py
│   │       │   │   │   ├── list_ws_connections.py
│   │       │   │   │   └── reload_config.py
│   │       │   │   ├── agent_annotations/
│   │       │   │   │   ├── __init__.py
│   │       │   │   │   ├── agents_add_annotation.py
│   │       │   │   │   ├── agents_list_annotations.py
│   │       │   │   │   └── agents_remove_annotation.py
│   │       │   │   ├── agent_events/
│   │       │   │   │   ├── __init__.py
│   │       │   │   │   ├── create_event.py
│   │       │   │   │   ├── get_agent_event.py
│   │       │   │   │   ├── list_agent_events.py
│   │       │   │   │   └── list_events.py
│   │       │   │   ├── agent_labels/
│   │       │   │   │   ├── __init__.py
│   │       │   │   │   ├── agents_add_label.py
│   │       │   │   │   ├── agents_list_labels.py
│   │       │   │   │   └── agents_remove_label.py
│   │       │   │   ├── agent_targets/
│   │       │   │   │   ├── __init__.py
│   │       │   │   │   ├── add_target.py
│   │       │   │   │   ├── list_targets.py
│   │       │   │   │   └── remove_target.py
│   │       │   │   ├── agents/
│   │       │   │   │   ├── __init__.py
│   │       │   │   │   ├── create_agent.py
│   │       │   │   │   ├── delete_agent.py
│   │       │   │   │   ├── get_agent.py
│   │       │   │   │   ├── get_associated_stacks.py
│   │       │   │   │   ├── get_target_state.py
│   │       │   │   │   ├── list_agents.py
│   │       │   │   │   ├── record_heartbeat.py
│   │       │   │   │   ├── rotate_agent_pak.py
│   │       │   │   │   ├── search_agent.py
│   │       │   │   │   └── update_agent.py
│   │       │   │   ├── auth/
│   │       │   │   │   ├── __init__.py
│   │       │   │   │   └── verify_pak.py
│   │       │   │   ├── deployment_objects/
│   │       │   │   │   ├── __init__.py
│   │       │   │   │   └── get_deployment_object.py
│   │       │   │   ├── diagnostics/
│   │       │   │   │   ├── __init__.py
│   │       │   │   │   ├── claim_diagnostic.py
│   │       │   │   │   ├── create_diagnostic_request.py
│   │       │   │   │   ├── get_diagnostic.py
│   │       │   │   │   ├── get_pending_diagnostics.py
│   │       │   │   │   └── submit_diagnostic_result.py
│   │       │   │   ├── generators/
│   │       │   │   │   ├── __init__.py
│   │       │   │   │   ├── create_generator.py
│   │       │   │   │   ├── delete_generator.py
│   │       │   │   │   ├── get_generator.py
│   │       │   │   │   ├── list_generators.py
│   │       │   │   │   ├── rotate_generator_pak.py
│   │       │   │   │   └── update_generator.py
│   │       │   │   ├── health/
│   │       │   │   │   ├── __init__.py
│   │       │   │   │   ├── get_deployment_health.py
│   │       │   │   │   ├── get_stack_health.py
│   │       │   │   │   └── update_health_status.py
│   │       │   │   ├── stack_telemetry/
│   │       │   │   │   ├── __init__.py
│   │       │   │   │   ├── list_telemetry_events.py
│   │       │   │   │   └── list_telemetry_logs.py
│   │       │   │   ├── stacks/
│   │       │   │   │   ├── __init__.py
│   │       │   │   │   ├── create_deployment_object.py
│   │       │   │   │   ├── create_stack.py
│   │       │   │   │   ├── delete_stack.py
│   │       │   │   │   ├── get_stack.py
│   │       │   │   │   ├── instantiate_template.py
│   │       │   │   │   ├── list_deployment_objects.py
│   │       │   │   │   ├── list_stacks.py
│   │       │   │   │   ├── stacks_add_annotation.py
│   │       │   │   │   ├── stacks_add_label.py
│   │       │   │   │   ├── stacks_list_annotations.py
│   │       │   │   │   ├── stacks_list_labels.py
│   │       │   │   │   ├── stacks_remove_annotation.py
│   │       │   │   │   ├── stacks_remove_label.py
│   │       │   │   │   └── update_stack.py
│   │       │   │   ├── templates/
│   │       │   │   │   ├── __init__.py
│   │       │   │   │   ├── create_template.py
│   │       │   │   │   ├── delete_template.py
│   │       │   │   │   ├── get_template.py
│   │       │   │   │   ├── list_templates.py
│   │       │   │   │   ├── templates_add_annotation.py
│   │       │   │   │   ├── templates_add_label.py
│   │       │   │   │   ├── templates_list_annotations.py
│   │       │   │   │   ├── templates_list_labels.py
│   │       │   │   │   ├── templates_remove_annotation.py
│   │       │   │   │   ├── templates_remove_label.py
│   │       │   │   │   └── update_template.py
│   │       │   │   ├── webhooks/
│   │       │   │   │   ├── __init__.py
│   │       │   │   │   ├── create_webhook.py
│   │       │   │   │   ├── delete_webhook.py
│   │       │   │   │   ├── get_pending_agent_webhooks.py
│   │       │   │   │   ├── get_webhook.py
│   │       │   │   │   ├── list_deliveries.py
│   │       │   │   │   ├── list_event_types.py
│   │       │   │   │   ├── list_webhooks.py
│   │       │   │   │   ├── report_delivery_result.py
│   │       │   │   │   ├── test_webhook.py
│   │       │   │   │   └── update_webhook.py
│   │       │   │   └── work_orders/
│   │       │   │       ├── __init__.py
│   │       │   │       ├── claim_work_order.py
│   │       │   │       ├── complete_work_order.py
│   │       │   │       ├── create_work_order.py
│   │       │   │       ├── delete_work_order.py
│   │       │   │       ├── get_work_order.py
│   │       │   │       ├── get_work_order_log.py
│   │       │   │       ├── list_pending_for_agent.py
│   │       │   │       ├── list_work_order_log.py
│   │       │   │       └── list_work_orders.py
│   │       │   ├── client.py
│   │       │   ├── errors.py
│   │       │   ├── helpers.py
│   │       │   ├── models/
│   │       │   │   ├── __init__.py
│   │       │   │   ├── add_annotation_request.py
│   │       │   │   ├── agent.py
│   │       │   │   ├── agent_annotation.py
│   │       │   │   ├── agent_event.py
│   │       │   │   ├── agent_k8s_event.py
│   │       │   │   ├── agent_label.py
│   │       │   │   ├── agent_pod_log.py
│   │       │   │   ├── agent_target.py
│   │       │   │   ├── audit_log.py
│   │       │   │   ├── audit_log_list_response.py
│   │       │   │   ├── auth_response.py
│   │       │   │   ├── claim_work_order_request.py
│   │       │   │   ├── complete_work_order_request.py
│   │       │   │   ├── config_change_info.py
│   │       │   │   ├── config_reload_response.py
│   │       │   │   ├── create_agent_response.py
│   │       │   │   ├── create_deployment_object_request.py
│   │       │   │   ├── create_diagnostic_request.py
│   │       │   │   ├── create_generator_response.py
│   │       │   │   ├── create_template_request.py
│   │       │   │   ├── create_webhook_request.py
│   │       │   │   ├── create_work_order_request.py
│   │       │   │   ├── delivery_result_request.py
│   │       │   │   ├── deployment_health.py
│   │       │   │   ├── deployment_health_response.py
│   │       │   │   ├── deployment_object.py
│   │       │   │   ├── deployment_object_health_summary.py
│   │       │   │   ├── deployment_object_health_update.py
│   │       │   │   ├── diagnostic_request.py
│   │       │   │   ├── diagnostic_response.py
│   │       │   │   ├── diagnostic_result.py
│   │       │   │   ├── error_response.py
│   │       │   │   ├── error_response_details_type_0.py
│   │       │   │   ├── generator.py
│   │       │   │   ├── health_status_update.py
│   │       │   │   ├── health_summary.py
│   │       │   │   ├── k8s_event_history_response.py
│   │       │   │   ├── list_deliveries_query.py
│   │       │   │   ├── new_agent.py
│   │       │   │   ├── new_agent_annotation.py
│   │       │   │   ├── new_agent_event.py
│   │       │   │   ├── new_agent_label.py
│   │       │   │   ├── new_agent_target.py
│   │       │   │   ├── new_deployment_object.py
│   │       │   │   ├── new_generator.py
│   │       │   │   ├── new_stack.py
│   │       │   │   ├── new_stack_annotation.py
│   │       │   │   ├── new_stack_label.py
│   │       │   │   ├── new_stack_template.py
│   │       │   │   ├── new_template_annotation.py
│   │       │   │   ├── new_template_label.py
│   │       │   │   ├── pending_webhook_delivery.py
│   │       │   │   ├── pod_log_history_response.py
│   │       │   │   ├── resource_health.py
│   │       │   │   ├── retention_info.py
│   │       │   │   ├── stack.py
│   │       │   │   ├── stack_annotation.py
│   │       │   │   ├── stack_health_response.py
│   │       │   │   ├── stack_label.py
│   │       │   │   ├── stack_template.py
│   │       │   │   ├── submit_diagnostic_result.py
│   │       │   │   ├── template_annotation.py
│   │       │   │   ├── template_instantiation_request.py
│   │       │   │   ├── template_label.py
│   │       │   │   ├── update_template_request.py
│   │       │   │   ├── update_webhook_request.py
│   │       │   │   ├── webhook_delivery.py
│   │       │   │   ├── webhook_filters.py
│   │       │   │   ├── webhook_filters_labels_type_0.py
│   │       │   │   ├── webhook_response.py
│   │       │   │   ├── webhook_subscription.py
│   │       │   │   ├── work_order.py
│   │       │   │   ├── work_order_log.py
│   │       │   │   ├── work_order_targeting.py
│   │       │   │   ├── work_order_targeting_annotations_type_0.py
│   │       │   │   ├── ws_connection_info.py
│   │       │   │   └── ws_connections_response.py
│   │       │   └── types.py
│   │       └── tests/
│   │           ├── test_helpers.py
│   │           └── test_surface.py
│   └── typescript/
│       └── brokkr-client/
│           └── src/
│               ├── client.ts
│               ├── error.ts
│               ├── index.ts
│               ├── manifests.test.ts
│               ├── schema.d.ts
│               ├── surface.test.ts
│               └── wrapper.test.ts
├── tests/
│   ├── e2e/
│   │   └── src/
│   │       ├── api.rs
│   │       ├── main.rs
│   │       └── scenarios.rs
│   └── sdk-contract/
│       ├── python/
│       │   ├── conftest.py
│       │   ├── test_manifest_apply.py
│       │   ├── test_telemetry_and_ws.py
│       │   └── test_uat_walkthrough.py
│       ├── rust/
│       │   └── src/
│       │       └── main.rs
│       └── typescript/
│           ├── src/
│           │   ├── manifest-apply.test.ts
│           │   ├── telemetry-and-ws.test.ts
│           │   └── uat-walkthrough.test.ts
│           └── vitest.config.ts
└── tools/
    ├── webhook-catcher/
    │   └── app.py
    └── ws-loadtest/
        └── src/
            └── main.rs
```

## Modules

### crates/brokkr-agent/src

> *Semantic summary to be generated by AI agent.*

#### crates/brokkr-agent/src/bin.rs

-  `main` function L11-21 — `() -> Result<(), Box<dyn std::error::Error>>`

#### crates/brokkr-agent/src/broker.rs

- pub `wait_for_broker_ready` function L100-137 — `(config: &Settings)` — Waits for the broker service to become ready.
- pub `verify_agent_pak` function L141-162 — `(config: &Settings) -> Result<(), Box<dyn std::error::Error>>` — Verifies the agent's Personal Access Key (PAK) with the broker.
- pub `fetch_agent_details` function L165-208 — `( config: &Settings, client: &BrokkrClient, ) -> Result<Agent, Box<dyn std::erro...` — Fetches the details of the agent from the broker.
- pub `fetch_and_process_deployment_objects` function L211-262 — `( _config: &Settings, client: &BrokkrClient, agent: &Agent, ) -> Result<Vec<Depl...` — Fetches deployment objects to apply from the broker's target-state view.
- pub `send_success_event` function L265-323 — `( _config: &Settings, client: &BrokkrClient, agent: &Agent, deployment_object_id...` — Sends a success event to the broker for the given deployment object.
- pub `send_failure_event` function L326-384 — `( _config: &Settings, client: &BrokkrClient, agent: &Agent, deployment_object_id...` — Sends a failure event to the broker for the given deployment object.
- pub `send_heartbeat` function L387-434 — `( _config: &Settings, client: &BrokkrClient, agent: &Agent, ws_uplink: Option<&W...` — Sends a heartbeat to the broker for the given agent.
- pub `send_health_status` function L437-527 — `( _config: &Settings, client: &BrokkrClient, agent: &Agent, health_updates: Vec<...` — Sends health status updates for deployment objects to the broker.
- pub `fetch_pending_diagnostics` function L530-564 — `( _config: &Settings, client: &BrokkrClient, agent: &Agent, ) -> Result<Vec<Diag...` — Fetches pending diagnostic requests for the agent.
- pub `fetch_deployment_object` function L570-599 — `( client: &BrokkrClient, deployment_object_id: Uuid, ) -> Result<DeploymentObjec...` — Claims a diagnostic request for processing.
- pub `claim_diagnostic_request` function L601-634 — `( _config: &Settings, client: &BrokkrClient, request_id: Uuid, ) -> Result<Diagn...` — frequencies we operate at (seconds-scale).
- pub `submit_diagnostic_result` function L637-675 — `( _config: &Settings, client: &BrokkrClient, request_id: Uuid, result: SubmitDia...` — Submits diagnostic results for a request.
-  `try_ws_send` function L46-52 — `(uplink: Option<&WsUplink>, build: impl FnOnce() -> WsMessage) -> bool` — Try to send an event over the WS uplink.
-  `synth_agent_event` function L57-70 — `(new_event: &NewAgentEvent) -> WsMessage` — Build the wire-side `AgentEvent` body from the to-be-inserted shape.
-  `status_u16` function L75-77 — `(err: &BrokkrError) -> Option<u16>` — HTTP status helper.
-  `convert` function L82-85 — `(value: From) -> Result<To, serde_json::Error>` — JSON-round-trip between two `serde`-compatible types.
-  `boxed` function L89-95 — `(prefix: &str, err: BrokkrError) -> Box<dyn std::error::Error>` — Map a `BrokkrError` into the agent's historical `Box<dyn Error>` shape with

#### crates/brokkr-agent/src/broker_sdk.rs

- pub `build_client` function L35-43 — `(config: &Settings) -> Result<BrokkrClient, BrokkrError>` — Build a `BrokkrClient` from agent `Settings`.
-  `bearer_token` function L24-26 — `(pak: &str) -> String` — Bearer-token form expected by the broker's auth middleware.

#### crates/brokkr-agent/src/broker_ws.rs

- pub `WsState` enum L40-56 — `Down | Up | ForceRestOnly | AuthRejected` — Current state of the WS channel from the agent's point of view.
- pub `is_up` function L59-61 — `(self) -> bool` — state stays [`WsState::ForceRestOnly`] for the lifetime of the agent.
- pub `WsClient` struct L87-95 — `{ state: watch::Receiver<WsState>, outbound_tx: mpsc::Sender<WsMessage>, inbound...` — Public handle to the WS client.
- pub `state` function L99-101 — `(&self) -> watch::Receiver<WsState>` — Watch the connection state.
- pub `outbound` function L106-108 — `(&self) -> mpsc::Sender<WsMessage>` — Sender for outbound messages (heartbeat, agent events, health,
- pub `uplink` function L113-118 — `(&self) -> WsUplink` — Cheap clonable handle bundling the outbound sender with a current
- pub `take_inbound` function L122-124 — `(&mut self) -> Option<mpsc::Receiver<WsMessage>>` — Take ownership of the inbound receiver.
- pub `WsUplink` struct L131-134 — `{ state: watch::Receiver<WsState>, outbound: mpsc::Sender<WsMessage> }` — Send-side handle for agent components that want to prefer WS but fall
- pub `is_up` function L140-142 — `(&self) -> bool` — True iff the WS state is currently `Up`.
- pub `try_send` function L150-159 — `(&self, msg: WsMessage) -> Result<(), WsMessage>` — state stays [`WsState::ForceRestOnly`] for the lifetime of the agent.
- pub `spawn` function L168-214 — `(settings: &Settings) -> WsClient` — Spawn the WS connection task and return a client handle.
- pub `ws_url_from_broker_url` function L220-232 — `(broker_url: &str) -> String` — Convert `http(s)://broker/api/v1`-style URLs into the
-  `WsState` type L58-62 — `= WsState` — state stays [`WsState::ForceRestOnly`] for the lifetime of the agent.
-  `OUTBOUND_CAPACITY` variable L68 — `: usize` — Capacity of the outbound queue from the agent's emitters to the WS task.
-  `INBOUND_CAPACITY` variable L72 — `: usize` — Capacity of the inbound queue from the WS task to in-agent consumers.
-  `BACKOFF_INITIAL` variable L75 — `: Duration` — Bounds on the reconnect backoff schedule.
-  `BACKOFF_MAX` variable L76 — `: Duration` — state stays [`WsState::ForceRestOnly`] for the lifetime of the agent.
-  `MAX_CONSECUTIVE_AUTH_REJECTIONS` variable L82 — `: u32` — Consecutive WS-upgrade auth rejections (HTTP 401/403) after which the
-  `WsClient` type L97-125 — `= WsClient` — state stays [`WsState::ForceRestOnly`] for the lifetime of the agent.
-  `WsUplink` type L136-160 — `= WsUplink` — state stays [`WsState::ForceRestOnly`] for the lifetime of the agent.
-  `reconnect_loop` function L234-285 — `( url: String, pak: String, state_tx: watch::Sender<WsState>, inbound_tx: mpsc::...` — state stays [`WsState::ForceRestOnly`] for the lifetime of the agent.
-  `is_auth_rejection` function L289-297 — `(err: &tokio_tungstenite::tungstenite::Error) -> bool` — True when a WS-upgrade error is a credential rejection (HTTP 401/403),
-  `dial` function L299-319 — `( url: &str, pak: &str, ) -> Result< tokio_tungstenite::WebSocketStream<tokio_tu...` — state stays [`WsState::ForceRestOnly`] for the lifetime of the agent.
-  `run_socket` function L321-396 — `( socket: tokio_tungstenite::WebSocketStream< tokio_tungstenite::MaybeTlsStream<...` — state stays [`WsState::ForceRestOnly`] for the lifetime of the agent.
-  `BackoffSchedule` struct L401-403 — `{ current: Duration }` — Exponential backoff with capped maximum and ±20% jitter.
-  `BackoffSchedule` type L405-423 — `= BackoffSchedule` — state stays [`WsState::ForceRestOnly`] for the lifetime of the agent.
-  `new` function L406-410 — `() -> Self` — state stays [`WsState::ForceRestOnly`] for the lifetime of the agent.
-  `reset` function L412-414 — `(&mut self)` — state stays [`WsState::ForceRestOnly`] for the lifetime of the agent.
-  `next` function L416-422 — `(&mut self) -> Duration` — state stays [`WsState::ForceRestOnly`] for the lifetime of the agent.
-  `with_jitter` function L425-434 — `(d: Duration) -> Duration` — state stays [`WsState::ForceRestOnly`] for the lifetime of the agent.
-  `tests` module L437-614 — `-` — state stays [`WsState::ForceRestOnly`] for the lifetime of the agent.
-  `ws_url_translates_scheme_and_appends_path` function L441-454 — `()` — state stays [`WsState::ForceRestOnly`] for the lifetime of the agent.
-  `auth_rejection_detects_401_and_403_only` function L457-477 — `()` — state stays [`WsState::ForceRestOnly`] for the lifetime of the agent.
-  `backoff_grows_exponentially_then_caps` function L480-497 — `()` — state stays [`WsState::ForceRestOnly`] for the lifetime of the agent.
-  `backoff_reset_restores_initial` function L500-508 — `()` — state stays [`WsState::ForceRestOnly`] for the lifetime of the agent.
-  `jitter_stays_within_twenty_percent` function L511-518 — `()` — state stays [`WsState::ForceRestOnly`] for the lifetime of the agent.
-  `uplink_with` function L528-539 — `( state: WsState, capacity: usize, ) -> (WsUplink, watch::Sender<WsState>, mpsc:...` — state stays [`WsState::ForceRestOnly`] for the lifetime of the agent.
-  `heartbeat_msg` function L541-546 — `() -> WsMessage` — state stays [`WsState::ForceRestOnly`] for the lifetime of the agent.
-  `try_send_returns_message_when_down` function L549-555 — `()` — state stays [`WsState::ForceRestOnly`] for the lifetime of the agent.
-  `try_send_returns_message_when_force_rest_only` function L558-562 — `()` — state stays [`WsState::ForceRestOnly`] for the lifetime of the agent.
-  `try_send_delivers_when_up` function L565-571 — `()` — state stays [`WsState::ForceRestOnly`] for the lifetime of the agent.
-  `try_send_returns_message_when_lane_full` function L574-581 — `()` — state stays [`WsState::ForceRestOnly`] for the lifetime of the agent.
-  `ws_is_on_by_default_per_adr_0008` function L584-593 — `()` — state stays [`WsState::ForceRestOnly`] for the lifetime of the agent.
-  `try_send_follows_state_flip_back_to_rest` function L596-613 — `()` — state stays [`WsState::ForceRestOnly`] for the lifetime of the agent.

#### crates/brokkr-agent/src/deployment_health.rs

- pub `DeploymentHealthStatus` struct L56-65 — `{ id: Uuid, status: String, summary: HealthSummary, checked_at: DateTime<Utc> }` — Health status for a deployment object
- pub `HealthSummary` struct L69-78 — `{ pods_ready: usize, pods_total: usize, conditions: Vec<String>, resources: Vec<...` — Summary of health information for a deployment
- pub `ResourceHealth` struct L82-93 — `{ kind: String, name: String, namespace: String, ready: bool, message: Option<St...` — Health status of an individual resource
- pub `HealthChecker` struct L96-101 — `{ k8s_client: Client, watch_namespace: Option<String> }` — Checks deployment health for Kubernetes resources
- pub `new` function L105-110 — `(k8s_client: Client) -> Self` — Creates a new HealthChecker instance watching the whole cluster
- pub `with_watch_namespace` function L114-117 — `(mut self, namespace: Option<String>) -> Self` — Restricts pod discovery to a single namespace when `namespace` is
- pub `check_deployment_object` function L120-127 — `( &self, deployment_object_id: Uuid, ) -> Result<DeploymentHealthStatus, Box<dyn...` — Checks the health of a specific deployment object by ID.
- pub `check_deployment_objects` function L370-398 — `( &self, deployment_object_ids: &[Uuid], ) -> Vec<DeploymentHealthStatus>` — Checks health for multiple deployment objects with one cluster-wide
- pub `HealthStatusUpdate` struct L452-455 — `{ deployment_objects: Vec<DeploymentObjectHealthUpdate> }` — Request body for sending health status updates to the broker
- pub `DeploymentObjectHealthUpdate` struct L459-468 — `{ id: Uuid, status: String, summary: Option<HealthSummary>, checked_at: DateTime...` — Health update for a single deployment object (matches broker API)
-  `MAX_OWNER_DEPTH` variable L29 — `: usize` — Maximum ownerReference hops walked when attributing a pod to a
-  `OwnerKey` type L33 — `= (String, String, String, String)` — Cache key for owner-chain resolution within one discovery pass:
-  `DEGRADED_CONDITIONS` variable L36-44 — `: &[&str]` — Known problematic waiting conditions that indicate degraded health
-  `PENDING_CONDITIONS` variable L49 — `: &[&str]` — Conditions that indicate pending state (not yet problematic but not ready)
-  `TERMINATED_ISSUES` variable L52 — `: &[&str]` — Reasons from terminated containers that indicate issues
-  `HealthChecker` type L103-399 — `= HealthChecker` — OOMKilled, and other problematic conditions.
-  `analyze_pods` function L131-248 — `(&self, deployment_object_id: Uuid, pods: &[Pod]) -> DeploymentHealthStatus` — Analyzes a set of pods attributed to one deployment object and
-  `discover_pods` function L262-296 — `( &self, deployment_object_ids: &[Uuid], ) -> Result<HashMap<Uuid, Vec<Pod>>, Bo...` — Discovers the pods belonging to each requested deployment object in a
-  `resolve_owner_doid` function L302-366 — `( &self, pod: &Pod, discovery: &mut Option<Discovery>, cache: &mut HashMap<Owner...` — Walks a pod's controller ownerReference chain upward until an object
-  `pod_direct_doid` function L404-411 — `(pod: &Pod) -> Option<Uuid>` — Extracts the deployment-object id directly carried by a pod: the
-  `annotations_doid` function L414-418 — `(annotations: Option<&BTreeMap<String, String>>) -> Option<Uuid>` — Extracts the deployment-object id from an annotation map.
-  `controller_owner` function L422-427 — `(refs: Option<&[OwnerReference]>) -> Option<&OwnerReference>` — Picks the owner to walk: the controller reference when present, otherwise
-  `gvk_of` function L430-435 — `(api_version: &str, kind: &str) -> GroupVersionKind` — Builds a GroupVersionKind from an ownerReference's apiVersion + kind.
-  `is_pod_ready` function L438-448 — `(pod: &Pod) -> bool` — Checks if a pod is in ready state
-  `DeploymentObjectHealthUpdate` type L470-479 — `= DeploymentObjectHealthUpdate` — OOMKilled, and other problematic conditions.
-  `from` function L471-478 — `(status: DeploymentHealthStatus) -> Self` — OOMKilled, and other problematic conditions.
-  `tests` module L482-632 — `-` — OOMKilled, and other problematic conditions.
-  `pod_with` function L485-493 — `( labels: Option<BTreeMap<String, String>>, annotations: Option<BTreeMap<String,...` — OOMKilled, and other problematic conditions.
-  `test_pod_direct_doid_prefers_label_then_annotation` function L496-524 — `()` — OOMKilled, and other problematic conditions.
-  `test_controller_owner_prefers_controller_ref` function L527-550 — `()` — OOMKilled, and other problematic conditions.
-  `test_gvk_of_grouped_and_core` function L553-564 — `()` — OOMKilled, and other problematic conditions.
-  `test_degraded_conditions_are_detected` function L567-573 — `()` — OOMKilled, and other problematic conditions.
-  `test_terminated_issues_include_oomkilled` function L576-579 — `()` — OOMKilled, and other problematic conditions.
-  `test_health_summary_default` function L582-588 — `()` — OOMKilled, and other problematic conditions.
-  `test_deployment_health_status_serialization` function L591-610 — `()` — OOMKilled, and other problematic conditions.
-  `test_health_update_conversion` function L613-631 — `()` — OOMKilled, and other problematic conditions.

#### crates/brokkr-agent/src/diagnostics.rs

- pub `DiagnosticRequest` struct L28-47 — `{ id: Uuid, agent_id: Uuid, deployment_object_id: Uuid, status: String, requeste...` — Diagnostic request received from the broker.
- pub `SubmitDiagnosticResult` struct L51-60 — `{ pod_statuses: String, events: String, log_tails: Option<String>, collected_at:...` — Result to submit back to the broker.
- pub `PodStatus` struct L64-75 — `{ name: String, namespace: String, phase: String, conditions: Vec<PodCondition>,...` — Pod status information for diagnostics.
- pub `PodCondition` struct L79-88 — `{ condition_type: String, status: String, reason: Option<String>, message: Optio...` — Pod condition information.
- pub `ContainerStatus` struct L92-105 — `{ name: String, ready: bool, restart_count: i32, state: String, state_reason: Op...` — Container status information.
- pub `EventInfo` struct L109-124 — `{ event_type: Option<String>, reason: Option<String>, message: Option<String>, i...` — Kubernetes event information.
- pub `DiagnosticsHandler` struct L127-130 — `{ client: Client }` — Diagnostics handler for collecting Kubernetes diagnostics.
- pub `new` function L134-136 — `(client: Client) -> Self` — Creates a new DiagnosticsHandler.
- pub `collect_diagnostics` function L146-171 — `( &self, namespace: &str, label_selector: &str, ) -> Result<SubmitDiagnosticResu...` — Collects diagnostics for resources matching the given labels in the namespace.
- pub `collect_diagnostics_in` function L184-218 — `( &self, namespaces: &[String], label_selector: &str, ) -> Result<SubmitDiagnost...` — Collects diagnostics across multiple namespaces and merges the results.
-  `MAX_LOG_LINES` variable L24 — `: i64` — Maximum number of log lines to collect per container.
-  `DiagnosticsHandler` type L132-439 — `= DiagnosticsHandler` — about Kubernetes resources, including pod statuses, events, and log tails.
-  `collect_pod_statuses` function L221-329 — `( &self, namespace: &str, label_selector: &str, ) -> Result<Vec<PodStatus>, Box<...` — Collects pod statuses for matching pods.
-  `collect_events` function L332-371 — `( &self, namespace: &str, _label_selector: &str, ) -> Result<Vec<EventInfo>, Box...` — Collects events for matching resources.
-  `collect_log_tails` function L374-415 — `( &self, namespace: &str, label_selector: &str, ) -> Result<HashMap<String, Stri...` — Collects log tails for matching pods.
-  `get_container_logs` function L418-438 — `( &self, namespace: &str, pod_name: &str, container_name: &str, ) -> Result<Stri...` — Gets logs for a specific container.
-  `tests` module L442-502 — `-` — about Kubernetes resources, including pod statuses, events, and log tails.
-  `test_pod_status_serialization` function L446-470 — `()` — about Kubernetes resources, including pod statuses, events, and log tails.
-  `test_event_info_serialization` function L473-487 — `()` — about Kubernetes resources, including pod statuses, events, and log tails.
-  `test_submit_diagnostic_result_serialization` function L490-501 — `()` — about Kubernetes resources, including pod statuses, events, and log tails.

#### crates/brokkr-agent/src/health.rs

- pub `HealthState` struct L39-43 — `{ k8s_client: Client, broker_status: Arc<RwLock<BrokerStatus>>, start_time: Syst...` — Shared state for health endpoints
- pub `BrokerStatus` struct L47-50 — `{ connected: bool, last_heartbeat: Option<String> }` — Broker connection status
- pub `configure_health_routes` function L80-87 — `(state: HealthState) -> Router` — Configures and returns the health check router
-  `HealthStatus` struct L54-61 — `{ status: String, kubernetes: KubernetesStatus, broker: BrokerStatusResponse, up...` — Health status response structure
-  `KubernetesStatus` struct L65-69 — `{ connected: bool, error: Option<String> }` — Kubernetes health status
-  `BrokerStatusResponse` struct L73-77 — `{ connected: bool, last_heartbeat: Option<String> }` — Broker health status for response
-  `healthz` function L93-95 — `() -> impl IntoResponse` — Simple liveness check endpoint
-  `readyz` function L101-113 — `(State(state): State<HealthState>) -> impl IntoResponse` — Readiness check endpoint
-  `health` function L125-184 — `(State(state): State<HealthState>) -> impl IntoResponse` — Detailed health check endpoint
-  `metrics_handler` function L190-206 — `() -> impl IntoResponse` — Prometheus metrics endpoint

#### crates/brokkr-agent/src/kube_events.rs

- pub `DEFAULT_UID_CACHE_CAP` variable L79 — `: usize` — Default entry cap.
- pub `spawn` function L126-172 — `( client: Client, uplink: WsUplink, agent_id: Uuid, uid_cache_cap: usize, watch_...` — Spawn the kube-events tailer.
-  `LOOKUP_TTL` variable L55 — `: Duration` — How long to cache a UID→stack lookup before re-querying.
-  `OUTBOUND_CAPACITY` variable L61 — `: usize` — Capacity of the bounded outbound queue we drain into the WS uplink.
-  `CacheEntry` enum L64-67 — `Owned | NotOurs` — (WS-09) under the hard 6h retention ceiling.
-  `CachedLookup` struct L69-72 — `{ value: CacheEntry, fetched_at: Instant }` — (WS-09) under the hard 6h retention ceiling.
-  `UidCache` struct L84-86 — `{ by_uid: LruCache<String, CachedLookup> }` — Bounded LRU of UID → ownership lookups, with a per-entry TTL.
-  `UidCache` type L88-122 — `= UidCache` — (WS-09) under the hard 6h retention ceiling.
-  `new` function L89-94 — `(cap: usize) -> Self` — (WS-09) under the hard 6h retention ceiling.
-  `get` function L96-106 — `(&mut self, uid: &str) -> Option<CacheEntry>` — (WS-09) under the hard 6h retention ceiling.
-  `put` function L108-116 — `(&mut self, uid: String, value: CacheEntry)` — (WS-09) under the hard 6h retention ceiling.
-  `len` function L119-121 — `(&self) -> usize` — (WS-09) under the hard 6h retention ceiling.
-  `MAX_BACKOFF` variable L153 — `: Duration` — (WS-09) under the hard 6h retention ceiling.
-  `watch_loop` function L174-194 — `( client: Client, agent_id: Uuid, tx: mpsc::Sender<WsMessage>, cache: Arc<RwLock...` — (WS-09) under the hard 6h retention ceiling.
-  `handle_event` function L196-238 — `( client: &Client, agent_id: Uuid, ev: &K8sEventResource, tx: &mpsc::Sender<WsMe...` — (WS-09) under the hard 6h retention ceiling.
-  `resolve_stack` function L240-266 — `( client: &Client, ev: &K8sEventResource, uid: &str, cache: &Arc<RwLock<UidCache...` — (WS-09) under the hard 6h retention ceiling.
-  `annotation_lookup` function L268-298 — `( client: &Client, involved: &k8s_openapi::api::core::v1::ObjectReference, ) -> ...` — (WS-09) under the hard 6h retention ceiling.
-  `tests` module L301-385 — `-` — (WS-09) under the hard 6h retention ceiling.
-  `lookup_or_miss` function L307-315 — `(cache: &mut UidCache, uid: &str, api_calls: &mut usize) -> CacheEntry` — Mirror `resolve_stack`'s cache interaction without the real API:
-  `cache_returns_owned_within_ttl` function L318-326 — `()` — (WS-09) under the hard 6h retention ceiling.
-  `cache_treats_not_ours_as_a_real_entry` function L329-333 — `()` — (WS-09) under the hard 6h retention ceiling.
-  `cache_expires_after_ttl` function L336-344 — `()` — (WS-09) under the hard 6h retention ceiling.
-  `cache_stays_bounded_under_high_unique_churn` function L347-365 — `()` — (WS-09) under the hard 6h retention ceiling.
-  `cache_serves_hot_set_without_re_hitting_the_api` function L368-384 — `()` — (WS-09) under the hard 6h retention ceiling.

#### crates/brokkr-agent/src/lib.rs

- pub `broker` module L15 — `-` — # Brokkr Agent
- pub `broker_sdk` module L16 — `-` — see the [Brokkr Documentation](https://brokkr.io/explanation/architecture).
- pub `broker_ws` module L17 — `-` — see the [Brokkr Documentation](https://brokkr.io/explanation/architecture).
- pub `cli` module L18 — `-` — see the [Brokkr Documentation](https://brokkr.io/explanation/architecture).
- pub `deployment_health` module L19 — `-` — see the [Brokkr Documentation](https://brokkr.io/explanation/architecture).
- pub `diagnostics` module L20 — `-` — see the [Brokkr Documentation](https://brokkr.io/explanation/architecture).
- pub `health` module L21 — `-` — see the [Brokkr Documentation](https://brokkr.io/explanation/architecture).
- pub `k8s` module L22 — `-` — see the [Brokkr Documentation](https://brokkr.io/explanation/architecture).
- pub `kube_events` module L23 — `-` — see the [Brokkr Documentation](https://brokkr.io/explanation/architecture).
- pub `metrics` module L24 — `-` — see the [Brokkr Documentation](https://brokkr.io/explanation/architecture).
- pub `pod_logs` module L25 — `-` — see the [Brokkr Documentation](https://brokkr.io/explanation/architecture).
- pub `utils` module L26 — `-` — see the [Brokkr Documentation](https://brokkr.io/explanation/architecture).
- pub `webhooks` module L27 — `-` — see the [Brokkr Documentation](https://brokkr.io/explanation/architecture).
- pub `work_orders` module L28 — `-` — see the [Brokkr Documentation](https://brokkr.io/explanation/architecture).

#### crates/brokkr-agent/src/metrics.rs

- pub `poll_requests_total` function L27-41 — `() -> &'static CounterVec` — Broker poll request counter
- pub `poll_duration_seconds` function L44-59 — `() -> &'static HistogramVec` — Broker poll duration histogram
- pub `kubernetes_operations_total` function L63-77 — `() -> &'static CounterVec` — Kubernetes operations counter
- pub `kubernetes_operation_duration_seconds` function L81-96 — `() -> &'static HistogramVec` — Kubernetes operation duration histogram
- pub `heartbeat_sent_total` function L99-112 — `() -> &'static IntCounter` — Heartbeat sent counter
- pub `last_successful_poll_timestamp` function L115-128 — `() -> &'static Gauge` — Last successful poll timestamp (Unix timestamp)
- pub `encode_metrics` function L135-143 — `() -> Result<String, String>` — Encodes all registered metrics in Prometheus text format
-  `REGISTRY` variable L19 — `: OnceLock<Registry>` — Global Prometheus registry for all agent metrics
-  `registry` function L21-23 — `() -> &'static Registry` — It exposes metrics about broker polling, Kubernetes operations, and agent health.
-  `COUNTER` variable L28 — `: OnceLock<CounterVec>` — It exposes metrics about broker polling, Kubernetes operations, and agent health.
-  `HISTOGRAM` variable L45 — `: OnceLock<HistogramVec>` — It exposes metrics about broker polling, Kubernetes operations, and agent health.
-  `COUNTER` variable L64 — `: OnceLock<CounterVec>` — It exposes metrics about broker polling, Kubernetes operations, and agent health.
-  `HISTOGRAM` variable L82 — `: OnceLock<HistogramVec>` — It exposes metrics about broker polling, Kubernetes operations, and agent health.
-  `COUNTER` variable L100 — `: OnceLock<IntCounter>` — It exposes metrics about broker polling, Kubernetes operations, and agent health.
-  `GAUGE` variable L116 — `: OnceLock<Gauge>` — It exposes metrics about broker polling, Kubernetes operations, and agent health.
-  `tests` module L146-156 — `-` — It exposes metrics about broker polling, Kubernetes operations, and agent health.
-  `encode_metrics_succeeds` function L150-155 — `()` — It exposes metrics about broker polling, Kubernetes operations, and agent health.

#### crates/brokkr-agent/src/pod_logs.rs

- pub `STREAM_LOGS_ANNOTATION` variable L55 — `: &str` — Annotation that opts a workload into log streaming.
- pub `spawn` function L65-95 — `( client: Client, uplink: WsUplink, agent_id: Uuid, watch_namespace: Option<Stri...` — bucket the right answer is "ship to Datadog", not "raise the limit".
-  `ActiveTails` type L52 — `= Arc<RwLock<HashMap<String, Vec<JoinHandle<()>>>>>` — Per-pod (by UID) set of running log-tail tasks.
-  `DEFAULT_LINES_PER_SEC` variable L60 — `: u64` — Default per-container line-rate ceiling.
-  `RATE_WINDOW` variable L63 — `: Duration` — Window for the token-bucket counter.
-  `MAX_BACKOFF` variable L76 — `: Duration` — bucket the right answer is "ship to Datadog", not "raise the limit".
-  `watch_pods` function L97-137 — `( client: Client, uplink: WsUplink, agent_id: Uuid, active: ActiveTails, watch_n...` — bucket the right answer is "ship to Datadog", not "raise the limit".
-  `is_opted_in` function L139-146 — `(pod: &Pod) -> bool` — bucket the right answer is "ship to Datadog", not "raise the limit".
-  `pod_stack_id` function L148-152 — `(pod: &Pod) -> Option<Uuid>` — bucket the right answer is "ship to Datadog", not "raise the limit".
-  `ensure_tails` function L157-194 — `( client: &Client, uplink: &WsUplink, agent_id: Uuid, stack_id: Uuid, pod: &Pod,...` — For a given opted-in pod, ensure one tail task per container.
-  `teardown_for` function L196-203 — `(uid: &str, active: &ActiveTails)` — bucket the right answer is "ship to Datadog", not "raise the limit".
-  `tail_container` function L205-296 — `( pods: Api<Pod>, uplink: WsUplink, agent_id: Uuid, stack_id: Uuid, namespace: S...` — bucket the right answer is "ship to Datadog", not "raise the limit".
-  `MAX_OPEN_ATTEMPTS` variable L231 — `: u32` — bucket the right answer is "ship to Datadog", not "raise the limit".
-  `OPEN_RETRY` variable L232 — `: Duration` — bucket the right answer is "ship to Datadog", not "raise the limit".
-  `RateLimiter` struct L300-305 — `{ lines_per_sec: u64, window_start: Instant, count_in_window: u64, dropped_in_wi...` — Minimal token-bucket: at most `lines_per_sec` lines per RATE_WINDOW.
-  `Allowance` enum L307-316 — `Allow | Drop | DropAndGap` — bucket the right answer is "ship to Datadog", not "raise the limit".
-  `RateLimiter` type L318-349 — `= RateLimiter` — bucket the right answer is "ship to Datadog", not "raise the limit".
-  `new` function L319-326 — `(lines_per_sec: u64) -> Self` — bucket the right answer is "ship to Datadog", not "raise the limit".
-  `consume` function L328-348 — `(&mut self) -> Allowance` — bucket the right answer is "ship to Datadog", not "raise the limit".
-  `tests` module L356-377 — `-` — bucket the right answer is "ship to Datadog", not "raise the limit".
-  `rate_limiter_allows_under_ceiling` function L360-365 — `()` — bucket the right answer is "ship to Datadog", not "raise the limit".
-  `rate_limiter_drops_above_ceiling_with_first_gap` function L368-376 — `()` — bucket the right answer is "ship to Datadog", not "raise the limit".

#### crates/brokkr-agent/src/utils.rs

- pub `multidoc_deserialize` function L18-24 — `(multi_doc_str: &str) -> Result<Vec<serde_yaml::Value>, Box<dyn Error>>` — Deserializes a multi-document YAML string into a vector of YAML values.
- pub `manifest_namespaces` function L36-58 — `(multi_doc_str: &str) -> Vec<String>` — Extracts the unique Kubernetes namespaces referenced by a multi-document
-  `tests` module L61-108 — `-`
-  `test_manifest_namespaces` function L65-70 — `()`
-  `test_multidoc_deserialize_success` function L73-92 — `()`
-  `test_multidoc_deserialize_failure` function L95-107 — `()`

#### crates/brokkr-agent/src/webhooks.rs

- pub `PendingWebhookDelivery` struct L44-63 — `{ id: Uuid, subscription_id: Uuid, event_type: String, payload: String, url: Str...` — Pending webhook delivery from the broker.
- pub `DeliveryResultRequest` struct L67-79 — `{ success: bool, status_code: Option<i32>, error: Option<String>, duration_ms: O...` — Request body for reporting delivery result to broker.
- pub `DeliveryResult` struct L83-92 — `{ success: bool, status_code: Option<i32>, error: Option<String>, duration_ms: i...` — Result of a webhook delivery attempt.
- pub `fetch_pending_webhooks` function L107-142 — `( _config: &Settings, client: &BrokkrClient, agent: &Agent, ) -> Result<Vec<Pend...` — Fetches pending webhook deliveries for this agent from the broker.
- pub `report_delivery_result` function L154-195 — `( _config: &Settings, client: &BrokkrClient, delivery_id: Uuid, result: &Deliver...` — Reports the result of a webhook delivery attempt to the broker.
- pub `deliver_webhook` function L208-295 — `(delivery: &PendingWebhookDelivery) -> DeliveryResult` — Delivers a webhook via HTTP POST.
- pub `process_pending_webhooks` function L328-385 — `( config: &Settings, client: &BrokkrClient, agent: &Agent, ) -> Result<usize, Bo...` — Processes all pending webhook deliveries for this agent.
-  `status_u16` function L20-22 — `(err: &BrokkrError) -> Option<u16>` — assigned to them, deliver them via HTTP, and report results back to the broker.
-  `convert` function L24-27 — `(value: F) -> Result<T, serde_json::Error>` — assigned to them, deliver them via HTTP, and report results back to the broker.
-  `boxed` function L29-35 — `(prefix: &str, err: BrokkrError) -> Box<dyn std::error::Error>` — assigned to them, deliver them via HTTP, and report results back to the broker.
-  `classify_error` function L298-308 — `(error: &reqwest::Error) -> String` — Classifies request errors for logging and retry decisions.
-  `tests` module L388-462 — `-` — assigned to them, deliver them via HTTP, and report results back to the broker.
-  `test_delivery_result_request_serialization` function L392-406 — `()` — assigned to them, deliver them via HTTP, and report results back to the broker.
-  `test_delivery_result_request_with_error` function L409-420 — `()` — assigned to them, deliver them via HTTP, and report results back to the broker.
-  `test_pending_webhook_delivery_deserialization` function L423-442 — `()` — assigned to them, deliver them via HTTP, and report results back to the broker.
-  `test_pending_webhook_delivery_without_auth` function L445-461 — `()` — assigned to them, deliver them via HTTP, and report results back to the broker.

### crates/brokkr-agent/src/cli

> *Semantic summary to be generated by AI agent.*

#### crates/brokkr-agent/src/cli/commands.rs

- pub `start` function L100-625 — `() -> Result<(), Box<dyn std::error::Error>>` — - Contextual information
-  `PushAction` enum L82-89 — `Reconcile | PollWorkOrders | Ignore` — What an inbound broker→agent WS push frame should trigger in the control
-  `classify_push_frame` function L92-98 — `(msg: &WsMessage) -> PushAction` — Route an inbound WS frame to the control-loop action it should trigger.
-  `tests` module L628-701 — `-` — - Contextual information
-  `stack` function L633-643 — `() -> Stack` — - Contextual information
-  `target` function L645-651 — `() -> AgentTarget` — - Contextual information
-  `work_order` function L653-671 — `() -> WorkOrder` — - Contextual information
-  `stack_and_target_changes_trigger_reconcile` function L674-683 — `()` — - Contextual information
-  `work_order_triggers_poll` function L686-691 — `()` — - Contextual information
-  `uplink_frames_are_ignored` function L694-700 — `()` — - Contextual information

#### crates/brokkr-agent/src/cli/mod.rs

- pub `commands` module L8 — `-` — Command-line interface module for the Brokkr agent.
- pub `Cli` struct L14-18 — `{ command: Commands }` — CLI configuration structure.
- pub `Commands` enum L22-25 — `Start` — Available CLI commands.
- pub `parse_cli` function L31-33 — `() -> Cli` — Parses command-line arguments into the Cli structure.

### crates/brokkr-agent/src/k8s

> *Semantic summary to be generated by AI agent.*

#### crates/brokkr-agent/src/k8s/api.rs

- pub `apply_k8s_objects` function L148-261 — `( k8s_objects: &[DynamicObject], k8s_client: K8sClient, patch_params: PatchParam...` — Applies a list of Kubernetes objects to the cluster using server-side apply.
- pub `dynamic_api` function L274-288 — `( ar: ApiResource, caps: ApiCapabilities, client: K8sClient, namespace: Option<&...` — Creates a dynamic Kubernetes API client for a specific resource type
- pub `get_all_objects_by_annotation` function L300-381 — `( k8s_client: &K8sClient, annotation_key: &str, annotation_value: &str, watch_na...` — Retrieves all Kubernetes objects with a specific annotation key-value pair.
- pub `delete_k8s_objects` function L392-485 — `( k8s_objects: &[DynamicObject], k8s_client: K8sClient, agent_id: &Uuid, ) -> Re...` — Deletes a list of Kubernetes objects from the cluster.
- pub `validate_k8s_objects` function L495-595 — `( k8s_objects: &[DynamicObject], k8s_client: K8sClient, ) -> Result<(), Box<dyn ...` — Validates Kubernetes objects against the API server without applying them.
- pub `reconcile_target_state` function L724-1012 — `( objects: &[DynamicObject], client: Client, stack_id: &str, checksum: &str, age...` — Reconciles the target state of Kubernetes objects for a stack.
- pub `create_k8s_client` function L1021-1052 — `( kubeconfig_path: Option<&str>, ) -> Result<K8sClient, Box<dyn std::error::Erro...` — Creates a Kubernetes client using either a provided kubeconfig path or default configuration.
-  `RetryConfig` struct L67-72 — `{ max_elapsed_time: Duration, initial_interval: Duration, max_interval: Duration...` — Retry configuration for Kubernetes operations
-  `RetryConfig` type L74-83 — `impl Default for RetryConfig` — 3.
-  `default` function L75-82 — `() -> Self` — 3.
-  `is_retryable_error` function L86-97 — `(error: &KubeError) -> bool` — Determines if a Kubernetes error is retryable
-  `with_retries` function L100-136 — `( operation: F, config: RetryConfig, ) -> Result<T, Box<dyn std::error::Error>>` — Executes a Kubernetes operation with retries
-  `apply_single_object` function L604-668 — `( object: &DynamicObject, client: &Client, stack_id: &str, checksum: &str, ) -> ...` — Applies a single Kubernetes object with proper annotations.
-  `rollback_namespaces` function L675-707 — `(client: &Client, namespaces: &[String])` — Rolls back namespaces that were created during a failed reconciliation.

#### crates/brokkr-agent/src/k8s/mod.rs

- pub `api` module L7 — `-`
- pub `objects` module L8 — `-`

#### crates/brokkr-agent/src/k8s/objects.rs

- pub `STACK_LABEL` variable L43 — `: &str` — Label key for identifying stack resources
- pub `CHECKSUM_ANNOTATION` variable L46 — `: &str` — Annotation key for deployment checksums
- pub `LAST_CONFIG_ANNOTATION` variable L49 — `: &str` — Annotation key for last applied configuration
- pub `DEPLOYMENT_OBJECT_ID_LABEL` variable L52 — `: &str` — Label key for deployment object IDs
- pub `BROKKR_AGENT_OWNER_ANNOTATION` variable L55 — `: &str` — Key for agent ownership
- pub `create_k8s_objects` function L64-126 — `( deployment_object: DeploymentObject, agent_id: Uuid, ) -> Result<Vec<DynamicOb...` — Creates Kubernetes objects from a brokkr deployment object's YAML content.
- pub `verify_object_ownership` function L129-137 — `(object: &DynamicObject, agent_id: &Uuid) -> bool` — - Object validation
-  `tests` module L140-469 — `-` — - Object validation
-  `create_test_object` function L153-165 — `(annotations: Option<BTreeMap<String, String>>) -> DynamicObject` — - Object validation
-  `test_create_k8s_objects_single_document` function L168-204 — `()` — - Object validation
-  `test_create_k8s_objects_multiple_documents` function L207-261 — `()` — - Object validation
-  `test_create_k8s_objects_with_crds` function L264-311 — `()` — - Object validation
-  `test_create_k8s_objects_invalid_yaml` function L314-339 — `()` — - Object validation
-  `test_create_k8s_objects_empty_yaml` function L342-359 — `()` — - Object validation
-  `test_create_k8s_objects_ordering` function L362-413 — `()` — - Object validation
-  `test_verify_object_ownership_matching_owner` function L416-427 — `()` — - Object validation
-  `test_verify_object_ownership_different_owner` function L430-441 — `()` — - Object validation
-  `test_verify_object_ownership_no_annotations` function L444-448 — `()` — - Object validation
-  `test_verify_object_ownership_empty_annotations` function L451-455 — `()` — - Object validation
-  `test_verify_object_ownership_invalid_uuid` function L458-468 — `()` — - Object validation

### crates/brokkr-agent/src/work_orders

> *Semantic summary to be generated by AI agent.*

#### crates/brokkr-agent/src/work_orders/broker.rs

- pub `fetch_pending_work_orders` function L40-83 — `( _config: &Settings, client: &BrokkrClient, agent: &Agent, work_type: Option<&s...` — Fetches pending work orders for the agent from the broker.
- pub `claim_work_order` function L86-146 — `( _config: &Settings, client: &BrokkrClient, agent: &Agent, work_order_id: Uuid,...` — Claims a work order for the agent.
- pub `complete_work_order` function L155-222 — `( _config: &Settings, client: &BrokkrClient, work_order_id: Uuid, success: bool,...` — Reports work order completion to the broker.
-  `status_u16` function L22-24 — `(err: &BrokkrError) -> Option<u16>` — types the 200 success path (T-A1 carry-over).
-  `convert` function L26-29 — `(value: F) -> Result<T, serde_json::Error>` — types the 200 success path (T-A1 carry-over).
-  `boxed` function L31-37 — `(prefix: &str, err: BrokkrError) -> Box<dyn std::error::Error + Send + Sync>` — types the 200 success path (T-A1 carry-over).

#### crates/brokkr-agent/src/work_orders/build.rs

- pub `execute_build` function L103-198 — `( k8s_client: &K8sClient, yaml_content: &str, work_order_id: &str, ) -> Result<O...` — Executes a build using Shipwright.
-  `SHIPWRIGHT_API_GROUP` variable L34 — `: &str` — Shipwright API group
-  `SHIPWRIGHT_API_VERSION` variable L35 — `: &str` — - **ClusterBuildStrategy**: Pre-installed strategy (e.g., buildah)
-  `CONDITION_SUCCEEDED` variable L38 — `: &str` — BuildRun status conditions
-  `BUILD_TIMEOUT_SECS` variable L41 — `: u64` — Maximum time to wait for a build to complete (15 minutes)
-  `STATUS_POLL_INTERVAL_SECS` variable L44 — `: u64` — Polling interval for build status checks
-  `BuildRunStatus` struct L49-56 — `{ conditions: Vec<Condition>, output: Option<BuildRunOutput>, failure_details: O...` — BuildRun status for watching completion
-  `Condition` struct L60-68 — `{ condition_type: String, status: String, reason: Option<String>, message: Optio...` — - **ClusterBuildStrategy**: Pre-installed strategy (e.g., buildah)
-  `BuildRunOutput` struct L73-76 — `{ digest: Option<String>, size: Option<i64> }` — - **ClusterBuildStrategy**: Pre-installed strategy (e.g., buildah)
-  `FailureDetails` struct L80-85 — `{ reason: Option<String>, message: Option<String> }` — - **ClusterBuildStrategy**: Pre-installed strategy (e.g., buildah)
-  `apply_shipwright_resource` function L201-213 — `( k8s_client: &K8sClient, resource: &serde_yaml::Value, ) -> Result<(), Box<dyn ...` — Applies a Shipwright resource (Build) to the cluster using the core k8s apply logic.
-  `create_buildrun` function L216-258 — `( k8s_client: &K8sClient, name: &str, build_name: &str, namespace: &str, work_or...` — Creates a BuildRun resource.
-  `watch_buildrun_completion` function L261-344 — `( k8s_client: &K8sClient, name: &str, namespace: &str, ) -> Result<Option<String...` — Watches a BuildRun until it completes (success or failure).
-  `ParsedBuildInfo` struct L349-353 — `{ build_name: String, build_namespace: String, build_docs: Vec<serde_yaml::Value...` — Result of parsing build YAML content
-  `parse_build_yaml` function L368-429 — `( yaml_content: &str, ) -> Result<ParsedBuildInfo, Box<dyn std::error::Error + S...` — Parses YAML content to extract Build resource information.
-  `interpret_buildrun_status` function L438-473 — `(status: &BuildRunStatus) -> Result<Option<String>, String>` — Interprets a BuildRun status to determine completion state.
-  `tests` module L476-885 — `-` — - **ClusterBuildStrategy**: Pre-installed strategy (e.g., buildah)
-  `test_parse_build_yaml_with_build_resource` function L482-504 — `()` — - **ClusterBuildStrategy**: Pre-installed strategy (e.g., buildah)
-  `test_parse_build_yaml_default_namespace` function L507-522 — `()` — - **ClusterBuildStrategy**: Pre-installed strategy (e.g., buildah)
-  `test_parse_build_yaml_with_work_order_buildref` function L525-540 — `()` — - **ClusterBuildStrategy**: Pre-installed strategy (e.g., buildah)
-  `test_parse_build_yaml_build_takes_precedence` function L543-569 — `()` — - **ClusterBuildStrategy**: Pre-installed strategy (e.g., buildah)
-  `test_parse_build_yaml_empty_content` function L572-583 — `()` — - **ClusterBuildStrategy**: Pre-installed strategy (e.g., buildah)
-  `test_parse_build_yaml_no_build_resource` function L586-604 — `()` — - **ClusterBuildStrategy**: Pre-installed strategy (e.g., buildah)
-  `test_parse_build_yaml_invalid_yaml` function L607-611 — `()` — - **ClusterBuildStrategy**: Pre-installed strategy (e.g., buildah)
-  `test_parse_build_yaml_multiple_builds` function L614-633 — `()` — - **ClusterBuildStrategy**: Pre-installed strategy (e.g., buildah)
-  `test_buildrun_status_deserialization_success` function L638-661 — `()` — - **ClusterBuildStrategy**: Pre-installed strategy (e.g., buildah)
-  `test_buildrun_status_deserialization_failure` function L664-683 — `()` — - **ClusterBuildStrategy**: Pre-installed strategy (e.g., buildah)
-  `test_buildrun_status_deserialization_in_progress` function L686-700 — `()` — - **ClusterBuildStrategy**: Pre-installed strategy (e.g., buildah)
-  `test_buildrun_status_deserialization_empty_conditions` function L703-709 — `()` — - **ClusterBuildStrategy**: Pre-installed strategy (e.g., buildah)
-  `test_interpret_status_succeeded_with_digest` function L714-732 — `()` — - **ClusterBuildStrategy**: Pre-installed strategy (e.g., buildah)
-  `test_interpret_status_succeeded_no_digest` function L735-750 — `()` — - **ClusterBuildStrategy**: Pre-installed strategy (e.g., buildah)
-  `test_interpret_status_failed_with_details` function L753-773 — `()` — - **ClusterBuildStrategy**: Pre-installed strategy (e.g., buildah)
-  `test_interpret_status_failed_no_details` function L776-791 — `()` — - **ClusterBuildStrategy**: Pre-installed strategy (e.g., buildah)
-  `test_interpret_status_failed_fallback_message` function L794-809 — `()` — - **ClusterBuildStrategy**: Pre-installed strategy (e.g., buildah)
-  `test_interpret_status_in_progress` function L812-827 — `()` — - **ClusterBuildStrategy**: Pre-installed strategy (e.g., buildah)
-  `test_interpret_status_no_succeeded_condition` function L830-845 — `()` — - **ClusterBuildStrategy**: Pre-installed strategy (e.g., buildah)
-  `test_interpret_status_empty_conditions` function L848-858 — `()` — - **ClusterBuildStrategy**: Pre-installed strategy (e.g., buildah)
-  `test_buildrun_name_generation_short_id` function L863-872 — `()` — - **ClusterBuildStrategy**: Pre-installed strategy (e.g., buildah)
-  `test_buildrun_name_generation_long_id` function L875-884 — `()` — - **ClusterBuildStrategy**: Pre-installed strategy (e.g., buildah)

#### crates/brokkr-agent/src/work_orders/mod.rs

- pub `broker` module L26 — `-` — # Work Orders Module
- pub `build` module L27 — `-` — ```
- pub `process_pending_work_orders` function L122-166 — `( config: &Settings, http_client: &BrokkrClient, k8s_client: &K8sClient, agent: ...` — Processes pending work orders for the agent.
-  `is_error_retryable` function L50-104 — `(error: &dyn std::error::Error) -> bool` — Determines if an error is retryable by inspecting the error message.
-  `process_single_work_order` function L169-227 — `( config: &Settings, http_client: &BrokkrClient, k8s_client: &K8sClient, agent: ...` — Processes a single work order through its complete lifecycle.
-  `execute_build_work_order` function L230-266 — `( _config: &Settings, _http_client: &BrokkrClient, k8s_client: &K8sClient, agent...` — Executes a build work order using Shipwright.
-  `execute_custom_work_order` function L269-331 — `( k8s_client: &K8sClient, agent: &Agent, work_order: &WorkOrder, ) -> Result<Opt...` — Executes a custom work order by applying YAML resources to the cluster.

### crates/brokkr-agent/tests

> *Semantic summary to be generated by AI agent.*

#### crates/brokkr-agent/tests/fixtures.rs

- pub `get_or_init_fixture` function L34-38 — `() -> Arc<Mutex<TestFixture>>` — Gets or initializes a test fixture singleton
- pub `TestFixture` struct L41-57 — `{ admin_settings: Settings, client: Client, sdk_client: BrokkrClient, agent_sett...`
- pub `new` function L67-87 — `() -> Self` — Creates a new TestFixture instance with default values
- pub `initialize` function L93-150 — `(&mut self)` — Initializes the test fixture by setting up necessary resources
- pub `wait_for_broker` function L156-158 — `(&self)` — Waits for the broker to become available
- pub `create_generator` function L168-210 — `(&mut self, name: String, description: Option<String>)` — Creates a new generator resource
- pub `create_stack` function L219-277 — `(&mut self, stack_name: &str)` — Creates a new stack resource
- pub `create_deployment` function L289-326 — `(&self, yaml_content: String) -> DeploymentObject` — Creates a new deployment from YAML content
-  `INIT` variable L15 — `: Once`
-  `FIXTURE` variable L25 — `: OnceCell<Arc<Mutex<TestFixture>>>`
-  `TestFixture` type L59-63 — `impl Default for TestFixture`
-  `default` function L60-62 — `() -> Self`
-  `TestFixture` type L65-327 — `= TestFixture`

### crates/brokkr-agent/tests/integration

> *Semantic summary to be generated by AI agent.*

#### crates/brokkr-agent/tests/integration/broker.rs

-  `TEST_NAMESPACE_YAML` variable L11-19 — `: &str`
-  `test_wait_for_broker` function L22-31 — `()`
-  `test_verify_agent_pak` function L34-46 — `()`
-  `test_fetch_agent_details` function L49-74 — `()`
-  `test_fetch_and_process_deployment_objects` function L77-103 — `()`
-  `test_successful_event_apply` function L106-156 — `()`
-  `test_failure_event_apply` function L159-214 — `()`
-  `test_send_heartbeat` function L217-254 — `()`

#### crates/brokkr-agent/tests/integration/broker_ws.rs

-  `SHORT_TIMEOUT` variable L34 — `: Duration` — the state at `ForceRestOnly`.
-  `ShutdownNotify` type L40 — `= Arc<Notify>` — Per-connection cancellation: shared with all in-flight WS handlers so a
-  `ws_accept_with_cancel` function L42-60 — `( upgrade: WebSocketUpgrade, cancel: ShutdownNotify, ) -> impl IntoResponse` — the state at `ForceRestOnly`.
-  `spawn_test_broker_on` function L66-81 — `(addr: SocketAddr) -> ShutdownNotify` — Spawn a test broker bound to a specific address.
-  `spawn_test_broker` function L83-89 — `() -> (SocketAddr, ShutdownNotify)` — the state at `ForceRestOnly`.
-  `settings_for_broker` function L91-97 — `(addr: SocketAddr, force_rest: bool) -> Settings` — the state at `ForceRestOnly`.
-  `wait_for_state` function L99-115 — `( mut watch: tokio::sync::watch::Receiver<WsState>, want: WsState, ) -> WsState` — the state at `ForceRestOnly`.
-  `client_connects_and_reaches_up_state` function L118-127 — `()` — the state at `ForceRestOnly`.
-  `client_reconnects_after_broker_restart` function L130-144 — `()` — the state at `ForceRestOnly`.
-  `force_rest_pins_state_and_skips_dial` function L147-163 — `()` — the state at `ForceRestOnly`.

#### crates/brokkr-agent/tests/integration/deployment_health.rs

-  `setup` function L15-29 — `() -> K8sClient`
-  `setup_namespace` function L31-50 — `(client: &K8sClient, namespace: &str)`
-  `cleanup` function L52-56 — `(client: &K8sClient, namespace: &str)`
-  `test_health_pod_attribution_via_owner_references` function L59-176 — `()`

#### crates/brokkr-agent/tests/integration/diagnostics.rs

-  `setup` function L16-30 — `() -> K8sClient`
-  `setup_namespace` function L32-51 — `(client: &K8sClient, namespace: &str)`
-  `cleanup` function L53-57 — `(client: &K8sClient, namespace: &str)`
-  `create_labeled_pod` function L61-105 — `( client: &K8sClient, namespace: &str, name: &str, deployment_object_id: &Uuid, ...` — Creates a bare Pod carrying the deployment-object-id label and waits until
-  `test_diagnostics_collects_pods_across_namespaces` function L108-158 — `()`

#### crates/brokkr-agent/tests/integration/health.rs

-  `create_test_health_state` function L18-34 — `() -> HealthState`
-  `test_healthz_endpoint` function L37-58 — `()`
-  `test_readyz_endpoint` function L61-81 — `()`
-  `test_health_endpoint` function L84-115 — `()`
-  `test_metrics_endpoint` function L118-142 — `()`

#### crates/brokkr-agent/tests/integration/main.rs

-  `broker` module L7 — `-`
-  `broker_ws` module L8 — `-`
-  `deployment_health` module L9 — `-`
-  `diagnostics` module L10 — `-`
-  `fixtures` module L12 — `-`
-  `health` module L13 — `-`
-  `k8s` module L14 — `-`

### crates/brokkr-agent/tests/integration/k8s

> *Semantic summary to be generated by AI agent.*

#### crates/brokkr-agent/tests/integration/k8s/api.rs

-  `INIT` variable L19 — `: Once`
-  `create_busybox_deployment_json` function L21-61 — `( name: &str, namespace: &str, agent_id: &Uuid, ) -> serde_json::Value`
-  `wait_for_configmap_value` function L63-85 — `( api: &Api<ConfigMap>, name: &str, expected_value: &str, max_attempts: u32, ) -...`
-  `create_namespace_json` function L87-98 — `(name: &str, agent_id: &Uuid) -> serde_json::Value`
-  `setup` function L100-121 — `() -> (K8sClient, Discovery)`
-  `cleanup` function L123-127 — `(client: &K8sClient, namespace: &str)`
-  `setup_namespace` function L130-143 — `(client: &K8sClient, namespace: &str, agent_id: &Uuid)`
-  `wait_for_deletion` function L145-161 — `(api: &Api<T>, name: &str, max_attempts: u32) -> bool`
-  `test_reconcile_single_object` function L164-218 — `()`
-  `test_reconcile_update_object` function L221-302 — `()`
-  `test_reconcile_invalid_object_rollback` function L305-410 — `()`
-  `test_reconcile_object_pruning` function L413-534 — `()`
-  `test_reconcile_does_not_prune_other_agents_objects` function L537-602 — `()`
-  `test_reconcile_empty_object_list` function L605-700 — `()`
-  `test_k8s_setup_and_cleanup` function L703-759 — `()`
-  `test_create_k8s_client_with_kubeconfig` function L762-775 — `()`
-  `test_create_k8s_client_with_invalid_path` function L778-784 — `()`
-  `test_create_k8s_client_default` function L787-793 — `()`
-  `test_apply_k8s_objects` function L796-867 — `()`
-  `test_validate_k8s_objects_valid` function L870-904 — `()`
-  `test_validate_k8s_objects_invalid` function L907-968 — `()`
-  `test_get_objects_by_annotation_found` function L971-1029 — `()`
-  `test_get_objects_by_annotation_not_found` function L1032-1078 — `()`
-  `test_delete_k8s_object_success` function L1081-1150 — `()`
-  `test_delete_k8s_object_not_found` function L1153-1193 — `()`
-  `test_reconcile_namespace_in_same_deployment` function L1196-1269 — `()`
-  `test_reconcile_rollback_spares_preexisting_namespace` function L1272-1341 — `()`
-  `test_reconcile_rollback_deletes_newly_created_namespace` function L1344-1430 — `()`
-  `test_reconcile_namespace_rollback_on_failure` function L1433-1506 — `()`

#### crates/brokkr-agent/tests/integration/k8s/mod.rs

-  `api` module L7 — `-`
-  `objects` module L8 — `-`

#### crates/brokkr-agent/tests/integration/k8s/objects.rs

-  `test_create_k8s_objects_single_document` function L15-51 — `()`
-  `test_create_k8s_objects_multiple_documents` function L54-108 — `()`
-  `test_create_k8s_objects_with_crds` function L111-158 — `()`
-  `test_create_k8s_objects_invalid_yaml` function L161-186 — `()`
-  `test_create_k8s_objects_empty_yaml` function L189-206 — `()`
-  `test_create_k8s_objects_ordering` function L209-260 — `()`

### crates/brokkr-broker/examples

> *Semantic summary to be generated by AI agent.*

#### crates/brokkr-broker/examples/openapi_export.rs

-  `main` function L26-43 — `() -> Result<(), Box<dyn std::error::Error>>` — Run with: `cargo run -p brokkr-broker --example openapi_export`
-  `downgrade_to_openapi_3_0` function L52-57 — `(doc: &mut Value)` — Rewrites the OpenAPI document in-place to be compatible with OpenAPI 3.0
-  `rewrite_nullable_types` function L59-128 — `(value: &mut Value)` — Run with: `cargo run -p brokkr-broker --example openapi_export`

### crates/brokkr-broker/src/api

> *Semantic summary to be generated by AI agent.*

#### crates/brokkr-broker/src/api/mod.rs

- pub `v1` module L157 — `-` — # API Module
- pub `configure_api_routes` function L194-254 — `( dal: DAL, cors_config: &Cors, reloadable_config: Option<ReloadableConfig>, ) -...` — Configures and returns the main application router with all API routes
-  `healthz` function L264-266 — `() -> impl IntoResponse` — Health check endpoint handler
-  `readyz` function L276-278 — `() -> impl IntoResponse` — Ready check endpoint handler
-  `metrics_handler` function L288-295 — `() -> impl IntoResponse` — Metrics endpoint handler
-  `metrics_middleware` function L300-316 — `(request: Request<Body>, next: Next) -> Response` — Middleware to record HTTP request metrics

### crates/brokkr-broker/src/api/v1

> *Semantic summary to be generated by AI agent.*

#### crates/brokkr-broker/src/api/v1/admin.rs

- pub `ConfigReloadResponse` struct L33-43 — `{ reloaded_at: DateTime<Utc>, changes: Vec<ConfigChangeInfo>, success: bool, mes...` — Response structure for configuration reload operations.
- pub `ConfigChangeInfo` struct L47-54 — `{ key: String, old_value: String, new_value: String }` — Information about a single configuration change.
- pub `AuditLogQueryParams` struct L58-82 — `{ actor_type: Option<String>, actor_id: Option<Uuid>, action: Option<String>, re...` — Query parameters for listing audit logs.
- pub `AuditLogListResponse` struct L100-111 — `{ logs: Vec<AuditLog>, total: i64, count: usize, limit: i64, offset: i64 }` — Response structure for audit log list operations.
- pub `routes` function L116-124 — `() -> Router<DAL>` — Constructs and returns the admin routes.
- pub `WsConnectionInfo` struct L127-132 — `{ agent_id: Uuid, connected_since: DateTime<Utc>, messages_in: u64, messages_out...` — including configuration hot-reload functionality.
- pub `WsConnectionsResponse` struct L135-140 — `{ connected_agents: usize, connections: Vec<WsConnectionInfo>, live_subscribers:...` — including configuration hot-reload functionality.
- pub `list_ws_connections` function L152-183 — `( Extension(auth): Extension<AuthPayload>, Extension(registry): Extension<std::s...` — including configuration hot-reload functionality.
-  `AuditLogFilter` type L84-96 — `= AuditLogFilter` — including configuration hot-reload functionality.
-  `from` function L85-95 — `(params: AuditLogQueryParams) -> Self` — including configuration hot-reload functionality.
-  `reload_config` function L215-285 — `( Extension(auth): Extension<AuthPayload>, Extension(config): Extension<Reloadab...` — including configuration hot-reload functionality.
-  `list_audit_logs` function L329-368 — `( State(dal): State<DAL>, Extension(auth): Extension<AuthPayload>, Query(params)...` — including configuration hot-reload functionality.
-  `tests` module L371-404 — `-` — including configuration hot-reload functionality.
-  `test_config_reload_response_serialization` function L375-391 — `()` — including configuration hot-reload functionality.
-  `test_config_change_info_serialization` function L394-403 — `()` — including configuration hot-reload functionality.

#### crates/brokkr-broker/src/api/v1/agent_events.rs

- pub `routes` function L24-28 — `() -> Router<DAL>` — Creates and returns a router for agent event-related endpoints.
-  `list_agent_events` function L44-55 — `( State(dal): State<DAL>, Extension(_auth_payload): Extension<crate::api::v1::mi...` — through HTTP endpoints.
-  `get_agent_event` function L75-94 — `( State(dal): State<DAL>, Extension(_auth_payload): Extension<crate::api::v1::mi...` — through HTTP endpoints.

#### crates/brokkr-broker/src/api/v1/agents.rs

- pub `routes` function L42-65 — `() -> Router<DAL>` — Agent management API endpoints.
- pub `CreateAgentResponse` struct L126-129 — `{ agent: Agent, initial_pak: String }` — Response body for [`create_agent`]: the newly-created agent plus the
-  `require_admin` function L67-76 — `(auth: &AuthPayload) -> Result<(), ApiError>` — Agent management API endpoints.
-  `require_admin_or_agent` function L78-87 — `(auth: &AuthPayload, id: Uuid) -> Result<(), ApiError>` — Agent management API endpoints.
-  `list_agents` function L98-121 — `( State(dal): State<DAL>, Extension(auth_payload): Extension<AuthPayload>, ) -> ...` — Agent management API endpoints.
-  `create_agent` function L141-198 — `( State(dal): State<DAL>, Extension(auth_payload): Extension<AuthPayload>, Json(...` — Agent management API endpoints.
-  `AgentQuery` struct L201-204 — `{ name: Option<String>, cluster_name: Option<String> }` — Agent management API endpoints.
-  `get_agent` function L217-234 — `( State(dal): State<DAL>, Extension(auth_payload): Extension<AuthPayload>, Path(...` — Agent management API endpoints.
-  `search_agent` function L251-284 — `( State(dal): State<DAL>, Extension(auth_payload): Extension<AuthPayload>, Query...` — Agent management API endpoints.
-  `update_agent` function L298-348 — `( State(dal): State<DAL>, Extension(auth_payload): Extension<AuthPayload>, Path(...` — Agent management API endpoints.
-  `delete_agent` function L360-394 — `( State(dal): State<DAL>, Extension(auth_payload): Extension<AuthPayload>, Exten...` — Agent management API endpoints.
-  `list_events` function L406-428 — `( State(dal): State<DAL>, Extension(auth_payload): Extension<AuthPayload>, Path(...` — Agent management API endpoints.
-  `create_event` function L441-473 — `( State(dal): State<DAL>, Extension(auth_payload): Extension<AuthPayload>, Path(...` — Agent management API endpoints.
-  `list_labels` function L487-504 — `( State(dal): State<DAL>, Extension(auth_payload): Extension<AuthPayload>, Path(...` — Agent management API endpoints.
-  `add_label` function L519-533 — `( State(dal): State<DAL>, Extension(auth_payload): Extension<AuthPayload>, Path(...` — Agent management API endpoints.
-  `remove_label` function L551-579 — `( State(dal): State<DAL>, Extension(auth_payload): Extension<AuthPayload>, Path(...` — Agent management API endpoints.
-  `list_annotations` function L593-611 — `( State(dal): State<DAL>, Extension(auth_payload): Extension<AuthPayload>, Path(...` — Agent management API endpoints.
-  `add_annotation` function L626-645 — `( State(dal): State<DAL>, Extension(auth_payload): Extension<AuthPayload>, Path(...` — Agent management API endpoints.
-  `remove_annotation` function L663-691 — `( State(dal): State<DAL>, Extension(auth_payload): Extension<AuthPayload>, Path(...` — Agent management API endpoints.
-  `list_targets` function L703-715 — `( State(dal): State<DAL>, Extension(auth_payload): Extension<AuthPayload>, Path(...` — Agent management API endpoints.
-  `add_target` function L729-747 — `( State(dal): State<DAL>, Extension(auth_payload): Extension<AuthPayload>, Exten...` — Agent management API endpoints.
-  `authorize_target_mutation` function L754-785 — `( dal: &DAL, auth: &AuthPayload, stack_id: Uuid, ) -> Result<(), ApiError>` — Authorize a target create/delete operation.
-  `remove_target` function L801-829 — `( State(dal): State<DAL>, Extension(auth_payload): Extension<AuthPayload>, Path(...` — Agent management API endpoints.
-  `record_heartbeat` function L841-867 — `( State(dal): State<DAL>, Extension(auth_payload): Extension<AuthPayload>, Path(...` — Agent management API endpoints.
-  `TargetStateParams` struct L870-872 — `{ mode: Option<String> }` — Agent management API endpoints.
-  `get_target_state` function L887-920 — `( State(dal): State<DAL>, Extension(auth_payload): Extension<AuthPayload>, Path(...` — Agent management API endpoints.
-  `get_associated_stacks` function L932-950 — `( State(dal): State<DAL>, Extension(auth_payload): Extension<AuthPayload>, Path(...` — Agent management API endpoints.
-  `rotate_agent_pak` function L963-1019 — `( State(dal): State<DAL>, Extension(auth_payload): Extension<AuthPayload>, Exten...` — Agent management API endpoints.

#### crates/brokkr-broker/src/api/v1/auth.rs

- pub `routes` function L19-21 — `() -> Router<DAL>` — Creates and returns the authentication routes for the API.
-  `verify_pak` function L40-46 — `(Extension(auth_payload): Extension<AuthPayload>) -> Json<AuthResponse>` — This module provides routes and handlers for authentication-related endpoints.

#### crates/brokkr-broker/src/api/v1/deployment_objects.rs

- pub `routes` function L59-62 — `() -> Router<DAL>` — Creates and returns the router for deployment object endpoints.
-  `accepts_yaml` function L28-42 — `(headers: &HeaderMap) -> bool` — Whether the client asked for a raw YAML representation via `Accept`
-  `deployment_object_response` function L46-56 — `(headers: &HeaderMap, object: DeploymentObject) -> Response` — Content-negotiated representation of a deployment object: the raw
-  `get_deployment_object` function L91-189 — `( State(dal): State<DAL>, Extension(auth_payload): Extension<AuthPayload>, Path(...` — including retrieval based on user authentication and authorization.

#### crates/brokkr-broker/src/api/v1/diagnostics.rs

- pub `routes` function L30-44 — `() -> Router<DAL>` — Creates and returns the router for diagnostic endpoints.
- pub `CreateDiagnosticRequest` struct L48-55 — `{ agent_id: Uuid, requested_by: Option<String>, retention_minutes: Option<i64> }` — Request body for creating a diagnostic request.
- pub `DiagnosticResponse` struct L59-64 — `{ request: DiagnosticRequest, result: Option<DiagnosticResult> }` — Response containing a diagnostic request with optional result.
- pub `SubmitDiagnosticResult` struct L68-77 — `{ pod_statuses: String, events: String, log_tails: Option<String>, collected_at:...` — Request body for submitting diagnostic results.
-  `create_diagnostic_request` function L96-160 — `( State(dal): State<DAL>, Extension(auth_payload): Extension<AuthPayload>, Path(...` — pick up and execute these requests, returning detailed diagnostic data.
-  `get_diagnostic` function L180-219 — `( State(dal): State<DAL>, Extension(auth_payload): Extension<AuthPayload>, Path(...` — pick up and execute these requests, returning detailed diagnostic data.
-  `get_pending_diagnostics` function L235-273 — `( State(dal): State<DAL>, Extension(auth_payload): Extension<AuthPayload>, Path(...` — pick up and execute these requests, returning detailed diagnostic data.
-  `claim_diagnostic` function L291-342 — `( State(dal): State<DAL>, Extension(auth_payload): Extension<AuthPayload>, Path(...` — pick up and execute these requests, returning detailed diagnostic data.
-  `submit_diagnostic_result` function L362-442 — `( State(dal): State<DAL>, Extension(auth_payload): Extension<AuthPayload>, Path(...` — pick up and execute these requests, returning detailed diagnostic data.

#### crates/brokkr-broker/src/api/v1/error.rs

- pub `ErrorResponse` struct L30-39 — `{ code: String, message: String, details: Option<BTreeMap<String, Value>> }` — Wire format for every 4xx/5xx response body in the v1 API.
- pub `ApiError` struct L44-49 — `{ status: StatusCode, code: String, message: String, details: Option<BTreeMap<St...` — Errors returned by v1 handlers.
- pub `with_details` function L63-66 — `(mut self, details: BTreeMap<String, Value>) -> Self` — Attach structured context to an error.
- pub `bad_request` function L70-72 — `(code: impl Into<String>, message: impl Into<String>) -> Self` — renamed.
- pub `unauthorized` function L74-76 — `(code: impl Into<String>, message: impl Into<String>) -> Self` — renamed.
- pub `forbidden` function L78-80 — `(code: impl Into<String>, message: impl Into<String>) -> Self` — renamed.
- pub `not_found` function L82-84 — `(code: impl Into<String>, message: impl Into<String>) -> Self` — renamed.
- pub `conflict` function L86-88 — `(code: impl Into<String>, message: impl Into<String>) -> Self` — renamed.
- pub `unprocessable` function L90-92 — `(code: impl Into<String>, message: impl Into<String>) -> Self` — renamed.
- pub `internal` function L96-98 — `(message: impl Into<String>) -> Self` — renamed.
- pub `from_diesel` function L147-197 — `(err: diesel::result::Error, internal_message: impl Into<String>) -> Self` — renamed.
-  `ApiError` type L51-99 — `= ApiError` — renamed.
-  `new` function L52-59 — `(status: StatusCode, code: impl Into<String>, message: impl Into<String>) -> Sel...` — renamed.
-  `ApiError` type L101-110 — `impl IntoResponse for ApiError` — renamed.
-  `into_response` function L102-109 — `(self) -> Response` — renamed.
-  `ApiError` type L112-122 — `= ApiError` — renamed.
-  `fmt` function L113-121 — `(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result` — renamed.
-  `ApiError` type L124 — `= ApiError` — renamed.
-  `ApiError` type L146-198 — `= ApiError` — Classify a `diesel::result::Error` into the right `ApiError` variant.
-  `ApiError` type L200-204 — `= ApiError` — renamed.
-  `from` function L201-203 — `(err: diesel::result::Error) -> Self` — renamed.

#### crates/brokkr-broker/src/api/v1/generators.rs

- pub `CreateGeneratorResponse` struct L32-37 — `{ generator: Generator, pak: String }` — Response for a successful generator creation or PAK rotation.
- pub `routes` function L39-48 — `() -> Router<DAL>` — Generators API module for Brokkr.
-  `list_generators` function L61-80 — `( State(dal): State<DAL>, Extension(auth_payload): Extension<AuthPayload>, ) -> ...` — Generators API module for Brokkr.
-  `audit_actor` function L84-90 — `(auth_payload: &AuthPayload) -> (&'static str, Option<Uuid>)` — Resolves the audit actor for generator endpoints: the admin, or the
-  `create_generator` function L106-169 — `( State(dal): State<DAL>, Extension(auth_payload): Extension<AuthPayload>, Json(...` — Generators API module for Brokkr.
-  `get_generator` function L184-212 — `( State(dal): State<DAL>, Extension(auth_payload): Extension<AuthPayload>, Path(...` — Generators API module for Brokkr.
-  `update_generator` function L228-263 — `( State(dal): State<DAL>, Extension(auth_payload): Extension<AuthPayload>, Path(...` — Generators API module for Brokkr.
-  `delete_generator` function L278-320 — `( State(dal): State<DAL>, Extension(auth_payload): Extension<AuthPayload>, Path(...` — Generators API module for Brokkr.
-  `rotate_generator_pak` function L335-409 — `( State(dal): State<DAL>, Extension(auth_payload): Extension<AuthPayload>, Path(...` — Generators API module for Brokkr.

#### crates/brokkr-broker/src/api/v1/health.rs

- pub `routes` function L30-36 — `() -> Router<DAL>` — Creates and returns the router for health-related endpoints.
- pub `HealthStatusUpdate` struct L40-43 — `{ deployment_objects: Vec<DeploymentObjectHealthUpdate> }` — Request body for updating health status from an agent.
- pub `DeploymentObjectHealthUpdate` struct L47-56 — `{ id: Uuid, status: String, summary: Option<HealthSummary>, checked_at: DateTime...` — Health update for a single deployment object.
- pub `DeploymentHealthResponse` struct L60-67 — `{ deployment_object_id: Uuid, health_records: Vec<DeploymentHealth>, overall_sta...` — Response for deployment object health query.
- pub `StackHealthResponse` struct L71-78 — `{ stack_id: Uuid, overall_status: String, deployment_objects: Vec<DeploymentObje...` — Response for stack health query.
- pub `DeploymentObjectHealthSummary` struct L82-93 — `{ id: Uuid, status: String, healthy_agents: usize, degraded_agents: usize, faili...` — Summary of health for a deployment object within a stack.
-  `update_health_status` function L114-174 — `( State(dal): State<DAL>, Extension(auth_payload): Extension<AuthPayload>, Path(...` — including endpoints for agents to report health and for operators to query health.
-  `get_deployment_health` function L194-230 — `( State(dal): State<DAL>, Extension(auth_payload): Extension<AuthPayload>, Path(...` — including endpoints for agents to report health and for operators to query health.
-  `get_stack_health` function L250-324 — `( State(dal): State<DAL>, Extension(auth_payload): Extension<AuthPayload>, Path(...` — including endpoints for agents to report health and for operators to query health.
-  `compute_overall_status` function L328-338 — `(records: &[DeploymentHealth]) -> String` — Computes the overall status from a list of health records.

#### crates/brokkr-broker/src/api/v1/middleware.rs

- pub `AuthPayload` struct L34-41 — `{ admin: bool, agent: Option<Uuid>, generator: Option<Uuid> }` — Represents the authenticated entity's payload.
- pub `AuthResponse` struct L45-52 — `{ admin: bool, agent: Option<String>, generator: Option<String> }` — Represents the response structure for authentication information.
- pub `auth_middleware` function L68-133 — `( State(dal): State<DAL>, mut request: Request<Body>, next: Next, ) -> Result<Re...` — Middleware function for authenticating requests.
-  `verify_pak` function L148-247 — `(dal: &DAL, pak: &str) -> Result<AuthPayload, StatusCode>` — Verifies the provided PAK and returns the corresponding `AuthPayload`.

#### crates/brokkr-broker/src/api/v1/mod.rs

- pub `admin` module L13 — `-` — API v1 module for the Brokkr broker.
- pub `agent_events` module L14 — `-` — with authentication middleware.
- pub `agents` module L15 — `-` — with authentication middleware.
- pub `auth` module L16 — `-` — with authentication middleware.
- pub `deployment_objects` module L17 — `-` — with authentication middleware.
- pub `diagnostics` module L18 — `-` — with authentication middleware.
- pub `error` module L19 — `-` — with authentication middleware.
- pub `generators` module L20 — `-` — with authentication middleware.
- pub `health` module L21 — `-` — with authentication middleware.
- pub `middleware` module L22 — `-` — with authentication middleware.
- pub `openapi` module L23 — `-` — with authentication middleware.
- pub `stacks` module L24 — `-` — with authentication middleware.
- pub `templates` module L25 — `-` — with authentication middleware.
- pub `webhooks` module L26 — `-` — with authentication middleware.
- pub `work_orders` module L27 — `-` — with authentication middleware.
- pub `routes` function L42-78 — `( dal: DAL, cors_config: &Cors, reloadable_config: Option<ReloadableConfig>, ) -...` — Constructs and returns the main router for API v1.
-  `build_cors_layer` function L84-121 — `(config: &Cors) -> CorsLayer` — Builds a CORS layer from configuration.

#### crates/brokkr-broker/src/api/v1/openapi.rs

- pub `ApiDoc` struct L264 — `-`
- pub `configure_openapi` function L298-302 — `() -> Router<DAL>`
-  `SecurityAddon` struct L266 — `-`
-  `SecurityAddon` type L268-285 — `= SecurityAddon`
-  `modify` function L269-284 — `(&self, openapi: &mut utoipa::openapi::OpenApi)`
-  `ServersAddon` struct L290 — `-` — Declares the API base URL.
-  `ServersAddon` type L292-296 — `= ServersAddon`
-  `modify` function L293-295 — `(&self, openapi: &mut utoipa::openapi::OpenApi)`
-  `serve_openapi` function L304-306 — `() -> Json<utoipa::openapi::OpenApi>`

#### crates/brokkr-broker/src/api/v1/stacks.rs

- pub `routes` function L37-64 — `() -> Router<DAL>`
- pub `list_deployment_objects` function L316-328 — `( State(dal): State<DAL>, Extension(auth_payload): Extension<AuthPayload>, Path(...`
- pub `CreateDeploymentObjectRequest` struct L335-341 — `{ yaml_content: String, is_deletion_marker: bool }` — Wire DTO for creating a deployment object via the public API.
- pub `CreateDeploymentObjectQuery` struct L347-352 — `{ deletion_marker: Option<bool> }` — Query parameters for the deployment-object create endpoint.
- pub `create_deployment_object` function L438-464 — `( State(dal): State<DAL>, Extension(auth_payload): Extension<AuthPayload>, Exten...`
- pub `list_labels` function L511-527 — `( State(dal): State<DAL>, Extension(auth_payload): Extension<AuthPayload>, Path(...`
- pub `add_label` function L545-563 — `( State(dal): State<DAL>, Extension(auth_payload): Extension<AuthPayload>, Path(...`
- pub `remove_label` function L582-600 — `( State(dal): State<DAL>, Extension(auth_payload): Extension<AuthPayload>, Path(...`
- pub `list_annotations` function L616-632 — `( State(dal): State<DAL>, Extension(auth_payload): Extension<AuthPayload>, Path(...`
- pub `add_annotation` function L650-668 — `( State(dal): State<DAL>, Extension(auth_payload): Extension<AuthPayload>, Path(...`
- pub `remove_annotation` function L687-705 — `( State(dal): State<DAL>, Extension(auth_payload): Extension<AuthPayload>, Path(...`
- pub `TemplateInstantiationRequest` struct L708-711 — `{ template_id: Uuid, parameters: serde_json::Value }`
- pub `TelemetryHistoryQuery` struct L878-887 — `{ since: Option<chrono::DateTime<chrono::Utc>>, limit: Option<i64> }`
- pub `RetentionInfo` struct L890-901 — `{ retention_ceiling_seconds: u64, effective_retention_seconds: u64, oldest_avail...`
- pub `K8sEventHistoryResponse` struct L904-907 — `{ retention: RetentionInfo, events: Vec<AgentK8sEvent> }`
- pub `PodLogHistoryResponse` struct L910-913 — `{ retention: RetentionInfo, lines: Vec<AgentPodLog> }`
- pub `list_telemetry_events` function L954-975 — `( State(dal): State<DAL>, Extension(auth): Extension<AuthPayload>, Path(stack_id...`
- pub `list_telemetry_logs` function L992-1013 — `( State(dal): State<DAL>, Extension(auth): Extension<AuthPayload>, Path(stack_id...`
-  `fetch_owned_stack` function L67-89 — `( dal: &DAL, auth: &AuthPayload, stack_id: Uuid, ) -> Result<Stack, ApiError>` — Fetch a stack or return 404; also enforces admin-or-generator-owner access.
-  `list_stacks` function L103-134 — `( State(dal): State<DAL>, Extension(auth_payload): Extension<AuthPayload>, ) -> ...`
-  `create_stack` function L148-190 — `( State(dal): State<DAL>, Extension(auth_payload): Extension<AuthPayload>, Json(...`
-  `get_stack` function L205-213 — `( State(dal): State<DAL>, Extension(auth_payload): Extension<AuthPayload>, Path(...`
-  `update_stack` function L230-263 — `( State(dal): State<DAL>, Extension(auth_payload): Extension<AuthPayload>, Path(...`
-  `delete_stack` function L278-301 — `( State(dal): State<DAL>, Extension(auth_payload): Extension<AuthPayload>, Path(...`
-  `content_type_is_yaml` function L358-370 — `(headers: &HeaderMap) -> bool` — Whether a `Content-Type` denotes a raw YAML body rather than the JSON
-  `resolve_create_body` function L374-390 — `( headers: &HeaderMap, query: &CreateDeploymentObjectQuery, body: &[u8], ) -> Re...` — Resolves the request body into `(yaml_content, is_deletion_marker)` based
-  `validate_manifest_yaml` function L395-421 — `(yaml_content: &str, is_deletion_marker: bool) -> Result<(), ApiError>` — Validates the manifest body at ingest so malformed YAML fails here with a
-  `is_authorized_for_stack` function L466-495 — `( dal: &DAL, auth_payload: &AuthPayload, stack_id: Uuid, ) -> Result<bool, ApiEr...`
-  `instantiate_template` function L729-857 — `( State(dal): State<DAL>, Extension(auth_payload): Extension<AuthPayload>, Path(...`
-  `TELEMETRY_DEFAULT_LIMIT` variable L873 — `: i64` — Default page size for the telemetry history endpoints.
-  `TELEMETRY_MAX_LIMIT` variable L875 — `: i64` — Maximum page size — protect the broker from "give me everything" callers.
-  `retention_info` function L915-923 — `(oldest: Option<chrono::DateTime<chrono::Utc>>) -> RetentionInfo`
-  `clamp_since` function L925-932 — `(since: Option<chrono::DateTime<chrono::Utc>>) -> chrono::DateTime<chrono::Utc>`
-  `clamp_limit` function L934-937 — `(limit: Option<i64>) -> i64`
-  `create_body_tests` module L1016-1122 — `-`
-  `headers_with` function L1020-1026 — `(ct: Option<&str>) -> HeaderMap`
-  `content_type_detection` function L1029-1038 — `()`
-  `yaml_body_uses_raw_string_and_query_flag` function L1041-1050 — `()`
-  `yaml_body_defaults_marker_false` function L1053-1059 — `()`
-  `json_body_still_parses` function L1062-1069 — `()`
-  `json_path_query_flag_ignored` function L1072-1080 — `()`
-  `malformed_json_is_rejected` function L1083-1089 — `()`
-  `validate_accepts_multidoc_yaml` function L1092-1095 — `()`
-  `validate_rejects_malformed_yaml` function L1098-1102 — `()`
-  `validate_rejects_empty_non_marker` function L1105-1108 — `()`
-  `validate_allows_empty_marker` function L1111-1114 — `()`
-  `validate_rejects_only_empty_documents` function L1117-1121 — `()`

#### crates/brokkr-broker/src/api/v1/templates.rs

- pub `CreateTemplateRequest` struct L33-38 — `{ name: String, description: Option<String>, template_content: String, parameter...` — API endpoints for stack template management.
- pub `UpdateTemplateRequest` struct L41-45 — `{ description: Option<String>, template_content: String, parameters_schema: Stri...` — API endpoints for stack template management.
- pub `AddAnnotationRequest` struct L48-51 — `{ key: String, value: String }` — API endpoints for stack template management.
- pub `routes` function L53-70 — `() -> Router<DAL>` — API endpoints for stack template management.
-  `can_modify_template` function L72-80 — `(auth: &AuthPayload, template: &StackTemplate) -> bool` — API endpoints for stack template management.
-  `check_read_access` function L82-97 — `(auth: &AuthPayload, template: &StackTemplate) -> Result<(), ApiError>` — API endpoints for stack template management.
-  `fetch_template_or_404` function L99-107 — `(dal: &DAL, template_id: Uuid) -> Result<StackTemplate, ApiError>` — API endpoints for stack template management.
-  `list_templates` function L120-156 — `( State(dal): State<DAL>, Extension(auth_payload): Extension<AuthPayload>, ) -> ...` — API endpoints for stack template management.
-  `audit_actor` function L160-166 — `(auth_payload: &AuthPayload) -> (&'static str, Option<uuid::Uuid>)` — Resolves the audit actor for template endpoints: the admin, or the
-  `create_template` function L181-235 — `( State(dal): State<DAL>, Extension(auth_payload): Extension<AuthPayload>, Json(...` — API endpoints for stack template management.
-  `get_template` function L250-260 — `( State(dal): State<DAL>, Extension(auth_payload): Extension<AuthPayload>, Path(...` — API endpoints for stack template management.
-  `update_template` function L277-329 — `( State(dal): State<DAL>, Extension(auth_payload): Extension<AuthPayload>, Path(...` — API endpoints for stack template management.
-  `delete_template` function L344-377 — `( State(dal): State<DAL>, Extension(auth_payload): Extension<AuthPayload>, Path(...` — API endpoints for stack template management.
-  `list_labels` function L394-406 — `( State(dal): State<DAL>, Extension(auth_payload): Extension<AuthPayload>, Path(...` — API endpoints for stack template management.
-  `add_label` function L425-445 — `( State(dal): State<DAL>, Extension(auth_payload): Extension<AuthPayload>, Path(...` — API endpoints for stack template management.
-  `remove_label` function L465-489 — `( State(dal): State<DAL>, Extension(auth_payload): Extension<AuthPayload>, Path(...` — API endpoints for stack template management.
-  `list_annotations` function L506-518 — `( State(dal): State<DAL>, Extension(auth_payload): Extension<AuthPayload>, Path(...` — API endpoints for stack template management.
-  `add_annotation` function L537-557 — `( State(dal): State<DAL>, Extension(auth_payload): Extension<AuthPayload>, Path(...` — API endpoints for stack template management.
-  `remove_annotation` function L577-603 — `( State(dal): State<DAL>, Extension(auth_payload): Extension<AuthPayload>, Path(...` — API endpoints for stack template management.

#### crates/brokkr-broker/src/api/v1/webhooks.rs

- pub `CreateWebhookRequest` struct L37-53 — `{ name: String, url: String, auth_header: Option<String>, event_types: Vec<Strin...` — Webhooks API module for Brokkr.
- pub `UpdateWebhookRequest` struct L56-75 — `{ name: Option<String>, url: Option<String>, auth_header: Option<Option<String>>...` — Webhooks API module for Brokkr.
- pub `WebhookResponse` struct L78-92 — `{ id: Uuid, name: String, has_url: bool, has_auth_header: bool, event_types: Vec...` — Webhooks API module for Brokkr.
- pub `ListDeliveriesQuery` struct L122-129 — `{ status: Option<String>, limit: Option<i64>, offset: Option<i64> }` — Webhooks API module for Brokkr.
- pub `PendingWebhookDelivery` struct L132-142 — `{ id: Uuid, subscription_id: Uuid, event_type: String, payload: String, url: Str...` — Webhooks API module for Brokkr.
- pub `DeliveryResultRequest` struct L145-153 — `{ success: bool, status_code: Option<i32>, error: Option<String>, duration_ms: O...` — Webhooks API module for Brokkr.
- pub `routes` function L174-193 — `() -> Router<DAL>` — Webhooks API module for Brokkr.
-  `WebhookResponse` type L94-119 — `= WebhookResponse` — Webhooks API module for Brokkr.
-  `from` function L95-118 — `(sub: WebhookSubscription) -> Self` — Webhooks API module for Brokkr.
-  `encrypt_value` function L159-164 — `(value: &str) -> Result<Vec<u8>, ApiError>` — Webhooks API module for Brokkr.
-  `decrypt_value` function L166-168 — `(encrypted: &[u8]) -> Result<String, String>` — Webhooks API module for Brokkr.
-  `list_webhooks` function L208-228 — `( State(dal): State<DAL>, Extension(auth_payload): Extension<AuthPayload>, ) -> ...` — Webhooks API module for Brokkr.
-  `list_event_types` function L238-248 — `( Extension(auth_payload): Extension<AuthPayload>, ) -> Result<Json<Vec<&'static...` — Webhooks API module for Brokkr.
-  `create_webhook` function L261-337 — `( State(dal): State<DAL>, Extension(auth_payload): Extension<AuthPayload>, Json(...` — Webhooks API module for Brokkr.
-  `get_webhook` function L350-379 — `( State(dal): State<DAL>, Extension(auth_payload): Extension<AuthPayload>, Path(...` — Webhooks API module for Brokkr.
-  `update_webhook` function L394-477 — `( State(dal): State<DAL>, Extension(auth_payload): Extension<AuthPayload>, Path(...` — Webhooks API module for Brokkr.
-  `delete_webhook` function L490-532 — `( State(dal): State<DAL>, Extension(auth_payload): Extension<AuthPayload>, Path(...` — Webhooks API module for Brokkr.
-  `list_deliveries` function L550-596 — `( State(dal): State<DAL>, Extension(auth_payload): Extension<AuthPayload>, Path(...` — Webhooks API module for Brokkr.
-  `test_webhook` function L610-716 — `( State(dal): State<DAL>, Extension(auth_payload): Extension<AuthPayload>, Path(...` — Webhooks API module for Brokkr.
-  `get_pending_agent_webhooks` function L733-835 — `( State(dal): State<DAL>, Extension(auth_payload): Extension<AuthPayload>, Path(...` — Webhooks API module for Brokkr.
-  `report_delivery_result` function L849-951 — `( State(dal): State<DAL>, Extension(auth_payload): Extension<AuthPayload>, Path(...` — Webhooks API module for Brokkr.

#### crates/brokkr-broker/src/api/v1/work_orders.rs

- pub `routes` function L33-48 — `() -> Router<DAL>` — Handles API routes and logic for work orders.
- pub `agent_routes` function L50-55 — `() -> Router<DAL>` — Handles API routes and logic for work orders.
- pub `CreateWorkOrderRequest` struct L62-75 — `{ work_type: String, yaml_content: String, max_retries: Option<i32>, backoff_sec...` — Handles API routes and logic for work orders.
- pub `WorkOrderTargeting` struct L78-85 — `{ agent_ids: Option<Vec<Uuid>>, labels: Option<Vec<String>>, annotations: Option...` — Handles API routes and logic for work orders.
- pub `ClaimWorkOrderRequest` struct L88-90 — `{ agent_id: Uuid }` — Handles API routes and logic for work orders.
- pub `CompleteWorkOrderRequest` struct L93-98 — `{ success: bool, message: Option<String>, retryable: bool }` — Handles API routes and logic for work orders.
- pub `ListWorkOrdersQuery` struct L105-108 — `{ status: Option<String>, work_type: Option<String> }` — Handles API routes and logic for work orders.
- pub `ListPendingQuery` struct L111-113 — `{ work_type: Option<String> }` — Handles API routes and logic for work orders.
- pub `ListLogQuery` struct L116-121 — `{ work_type: Option<String>, success: Option<bool>, agent_id: Option<Uuid>, limi...` — Handles API routes and logic for work orders.
-  `default_retryable` function L100-102 — `() -> bool` — Handles API routes and logic for work orders.
-  `list_work_orders` function L142-165 — `( State(dal): State<DAL>, Extension(auth_payload): Extension<AuthPayload>, Query...` — Handles API routes and logic for work orders.
-  `create_work_order` function L180-279 — `( State(dal): State<DAL>, Extension(auth_payload): Extension<AuthPayload>, Exten...` — Handles API routes and logic for work orders.
-  `get_work_order` function L294-363 — `( State(dal): State<DAL>, Extension(auth_payload): Extension<AuthPayload>, Path(...` — Handles API routes and logic for work orders.
-  `delete_work_order` function L378-406 — `( State(dal): State<DAL>, Extension(auth_payload): Extension<AuthPayload>, Path(...` — Handles API routes and logic for work orders.
-  `list_pending_for_agent` function L427-463 — `( State(dal): State<DAL>, Extension(auth_payload): Extension<AuthPayload>, Path(...` — Handles API routes and logic for work orders.
-  `claim_work_order` function L479-531 — `( State(dal): State<DAL>, Extension(auth_payload): Extension<AuthPayload>, Path(...` — Handles API routes and logic for work orders.
-  `complete_work_order` function L553-656 — `( State(dal): State<DAL>, Extension(auth_payload): Extension<AuthPayload>, Path(...` — Handles API routes and logic for work orders.
-  `list_work_order_log` function L679-710 — `( State(dal): State<DAL>, Extension(auth_payload): Extension<AuthPayload>, Query...` — Handles API routes and logic for work orders.
-  `get_work_order_log` function L725-764 — `( State(dal): State<DAL>, Extension(auth_payload): Extension<AuthPayload>, Path(...` — Handles API routes and logic for work orders.

### crates/brokkr-broker/src

> *Semantic summary to be generated by AI agent.*

#### crates/brokkr-broker/src/bin.rs

-  `main` function L24-67 — `() -> Result<(), Box<dyn std::error::Error>>` — Main function to run the Brokkr Broker application

#### crates/brokkr-broker/src/db.rs

- pub `ConnectionPool` struct L17-22 — `{ pool: Pool<ConnectionManager<PgConnection>>, schema: Option<String> }` — Represents a pool of PostgreSQL database connections.
- pub `create_shared_connection_pool` function L42-65 — `( base_url: &str, database_name: &str, max_size: u32, schema: Option<&str>, ) ->...` — Creates a shared connection pool for PostgreSQL databases.
- pub `validate_schema_name` function L78-97 — `(schema: &str) -> Result<(), String>` — Validates a PostgreSQL schema name to prevent SQL injection.
- pub `get` function L115-134 — `( &self, ) -> Result<diesel::r2d2::PooledConnection<ConnectionManager<PgConnecti...` — Gets a connection from the pool with automatic schema search_path configuration.
- pub `setup_schema` function L148-172 — `(&self, schema: &str) -> Result<(), String>` — Sets up a PostgreSQL schema for multi-tenant isolation.
-  `ConnectionPool` type L99-173 — `= ConnectionPool` — For detailed documentation, see the [Brokkr Documentation](https://brokkr.io/explanation/components#database-module).

#### crates/brokkr-broker/src/lib.rs

- pub `api` module L15 — `-` — # Brokkr Broker
- pub `cli` module L16 — `-` — see the [Brokkr Documentation](https://brokkr.io/explanation/architecture).
- pub `dal` module L17 — `-` — see the [Brokkr Documentation](https://brokkr.io/explanation/architecture).
- pub `db` module L18 — `-` — see the [Brokkr Documentation](https://brokkr.io/explanation/architecture).
- pub `metrics` module L19 — `-` — see the [Brokkr Documentation](https://brokkr.io/explanation/architecture).
- pub `utils` module L20 — `-` — see the [Brokkr Documentation](https://brokkr.io/explanation/architecture).
- pub `ws` module L21 — `-` — see the [Brokkr Documentation](https://brokkr.io/explanation/architecture).

#### crates/brokkr-broker/src/metrics.rs

- pub `REGISTRY` variable L19 — `: Lazy<Registry>` — Global Prometheus registry for all broker metrics
- pub `HTTP_REQUESTS_TOTAL` variable L23-34 — `: Lazy<CounterVec>` — HTTP request counter
- pub `HTTP_REQUEST_DURATION_SECONDS` variable L38-52 — `: Lazy<HistogramVec>` — HTTP request duration histogram
- pub `DATABASE_QUERIES_TOTAL` variable L56-67 — `: Lazy<CounterVec>` — Database query counter
- pub `DATABASE_QUERY_DURATION_SECONDS` variable L71-85 — `: Lazy<HistogramVec>` — Database query duration histogram
- pub `ACTIVE_AGENTS` variable L88-95 — `: Lazy<IntGauge>` — Number of active agents
- pub `AGENT_HEARTBEAT_AGE_SECONDS` variable L99-110 — `: Lazy<GaugeVec>` — Agent heartbeat age gauge
- pub `STACKS_TOTAL` variable L113-120 — `: Lazy<IntGauge>` — Total number of stacks
- pub `DEPLOYMENT_OBJECTS_TOTAL` variable L123-133 — `: Lazy<IntGauge>` — Total number of deployment objects
- pub `WS_CONNECTED_AGENTS` variable L145-155 — `: Lazy<IntGauge>` — Currently-connected agents on the internal WS channel.
- pub `WS_MESSAGES_TOTAL` variable L159-169 — `: Lazy<IntCounterVec>` — WS frames flowing in/out of the broker, labelled by direction and type.
- pub `WS_LIVE_SUBSCRIBERS` variable L172-182 — `: Lazy<IntGauge>` — Subscribers on the live fan-out hub (WS-11), aggregated across stacks.
- pub `WS_LOG_EVICTION_RUNS_TOTAL` variable L185-195 — `: Lazy<IntCounter>` — Eviction passes executed by the retention worker (WS-09).
- pub `WS_TELEMETRY_EVICTED_TOTAL` variable L199-209 — `: Lazy<IntCounterVec>` — Total telemetry rows evicted (events + logs).
- pub `ws_connected_agents` function L213-215 — `() -> &'static IntGauge` — Convenience accessors keep call sites short and avoid the static names
- pub `ws_messages_total` function L217-219 — `(direction: &str, variant: &str) -> prometheus::IntCounter` — It exposes metrics about HTTP requests, database queries, and system state.
- pub `ws_live_subscribers` function L221-223 — `() -> &'static IntGauge` — It exposes metrics about HTTP requests, database queries, and system state.
- pub `ws_log_eviction_runs_total` function L225-227 — `() -> &'static IntCounter` — It exposes metrics about HTTP requests, database queries, and system state.
- pub `ws_telemetry_evicted_total` function L229-231 — `(table: &str) -> prometheus::IntCounter` — It exposes metrics about HTTP requests, database queries, and system state.
- pub `init` function L237-252 — `()` — Initializes all metrics by forcing lazy static evaluation
- pub `encode_metrics` function L259-270 — `() -> String` — Encodes all registered metrics in Prometheus text format
- pub `record_http_request` function L283-295 — `(endpoint: &str, method: &str, status: u16, duration_seconds: f64)` — Records an HTTP request metric
- pub `record_db_query` function L322-330 — `(query_type: &str, duration_seconds: f64)` — Records a database query metric
- pub `set_active_agents` function L333-335 — `(count: i64)` — Updates the active agents gauge
- pub `set_stacks_total` function L338-340 — `(count: i64)` — Updates the total stacks gauge
- pub `set_deployment_objects_total` function L343-345 — `(count: i64)` — Updates the total deployment objects gauge
- pub `set_agent_heartbeat_age` function L348-352 — `(agent_id: &str, agent_name: &str, age_seconds: f64)` — Updates the heartbeat age for a specific agent
-  `normalize_endpoint` function L299-315 — `(path: &str) -> String` — Normalizes an endpoint path to reduce cardinality
-  `tests` module L355-472 — `-` — It exposes metrics about HTTP requests, database queries, and system state.
-  `test_init_registers_all_metrics` function L359-405 — `()` — It exposes metrics about HTTP requests, database queries, and system state.
-  `test_normalize_endpoint_replaces_uuids` function L408-412 — `()` — It exposes metrics about HTTP requests, database queries, and system state.
-  `test_normalize_endpoint_replaces_numeric_ids` function L415-419 — `()` — It exposes metrics about HTTP requests, database queries, and system state.
-  `test_normalize_endpoint_preserves_regular_paths` function L422-430 — `()` — It exposes metrics about HTTP requests, database queries, and system state.
-  `test_record_http_request_increments_counter` function L433-447 — `()` — It exposes metrics about HTTP requests, database queries, and system state.
-  `test_set_active_agents` function L450-459 — `()` — It exposes metrics about HTTP requests, database queries, and system state.
-  `test_set_stacks_total` function L462-471 — `()` — It exposes metrics about HTTP requests, database queries, and system state.

### crates/brokkr-broker/src/cli

> *Semantic summary to be generated by AI agent.*

#### crates/brokkr-broker/src/cli/commands.rs

- pub `MIGRATIONS` variable L29 — `: EmbeddedMigrations`
- pub `serve` function L42-192 — `(config: &Settings) -> Result<(), Box<dyn std::error::Error>>` — Function to start the Brokkr Broker server
- pub `rotate_admin` function L197-209 — `(config: &Settings) -> Result<(), Box<dyn std::error::Error>>` — Function to rotate the admin key
- pub `rotate_agent_key` function L229-260 — `( config: &Settings, uuid: Uuid, ) -> Result<String, Box<dyn std::error::Error>>`
- pub `rotate_generator_key` function L262-298 — `( config: &Settings, uuid: Uuid, ) -> Result<String, Box<dyn std::error::Error>>`
- pub `create_agent` function L300-340 — `( config: &Settings, name: String, cluster_name: String, ) -> Result<(), Box<dyn...`
- pub `create_generator` function L342-379 — `( config: &Settings, name: String, description: Option<String>, ) -> Result<(), ...`
-  `Count` struct L33-36 — `{ count: i64 }`
-  `audit_cli_pak_event` function L214-227 — `(dal: &DAL, action: &str, resource_type: &str, id: Uuid, name: &str)` — Synchronously records a PAK lifecycle event performed via the CLI.

#### crates/brokkr-broker/src/cli/mod.rs

- pub `commands` module L7 — `-`
- pub `Cli` struct L19-22 — `{ command: Commands }` — Brokkr Broker CLI
- pub `Commands` enum L25-34 — `Serve | Create | Rotate`
- pub `CreateCommands` struct L37-40 — `{ command: CreateSubcommands }`
- pub `CreateSubcommands` enum L43-63 — `Agent | Generator`
- pub `RotateCommands` struct L66-69 — `{ command: RotateSubcommands }`
- pub `RotateSubcommands` enum L72-89 — `Agent | Generator | Admin`
- pub `parse_cli` function L91-93 — `() -> Cli`

### crates/brokkr-broker/src/dal

> *Semantic summary to be generated by AI agent.*

#### crates/brokkr-broker/src/dal/agent_annotations.rs

- pub `AgentAnnotationsDAL` struct L19-22 — `{ dal: &'a DAL }` — Handles database operations for Agent Annotations.
- pub `create` function L38-46 — `( &self, new_annotation: &NewAgentAnnotation, ) -> Result<AgentAnnotation, diese...` — Creates a new agent annotation in the database.
- pub `get` function L61-70 — `( &self, annotation_id: Uuid, ) -> Result<Option<AgentAnnotation>, diesel::resul...` — Retrieves an agent annotation by its ID.
- pub `list_for_agent` function L85-93 — `( &self, agent_id: Uuid, ) -> Result<Vec<AgentAnnotation>, diesel::result::Error...` — Lists all annotations for a specific agent.
- pub `list` function L104-107 — `(&self) -> Result<Vec<AgentAnnotation>, diesel::result::Error>` — Lists all agent annotations in the database.
- pub `update` function L123-132 — `( &self, annotation_id: Uuid, updated_annotation: &AgentAnnotation, ) -> Result<...` — Updates an existing agent annotation in the database.
- pub `delete` function L147-151 — `(&self, annotation_id: Uuid) -> Result<usize, diesel::result::Error>` — Deletes an agent annotation from the database.
- pub `delete_all_for_agent` function L166-170 — `(&self, agent_id: Uuid) -> Result<usize, diesel::result::Error>` — Deletes all annotations for a specific agent.
- pub `delete_by_agent_and_key` function L188-200 — `( &self, agent_id: Uuid, key: &str, ) -> Result<usize, diesel::result::Error>` — Deletes a specific annotation for an agent using a single indexed query.

#### crates/brokkr-broker/src/dal/agent_events.rs

- pub `AgentEventsDAL` struct L22-25 — `{ dal: &'a DAL }` — Data Access Layer for AgentEvent operations.
- pub `create` function L37-42 — `(&self, new_event: &NewAgentEvent) -> Result<AgentEvent, diesel::result::Error>` — Creates a new agent event in the database.
- pub `get` function L53-60 — `(&self, event_uuid: Uuid) -> Result<Option<AgentEvent>, diesel::result::Error>` — Retrieves a non-deleted agent event by its UUID.
- pub `get_including_deleted` function L71-80 — `( &self, event_uuid: Uuid, ) -> Result<Option<AgentEvent>, diesel::result::Error...` — Retrieves an agent event by its UUID, including deleted events.
- pub `list` function L87-92 — `(&self) -> Result<Vec<AgentEvent>, diesel::result::Error>` — Lists all non-deleted agent events from the database.
- pub `list_all` function L99-102 — `(&self) -> Result<Vec<AgentEvent>, diesel::result::Error>` — Lists all agent events from the database, including deleted ones.
- pub `get_events` function L114-140 — `( &self, stack_id: Option<Uuid>, agent_id: Option<Uuid>, ) -> Result<Vec<AgentEv...` — Lists agent events from the database with optional filtering by stack and agent.
- pub `update` function L152-161 — `( &self, event_uuid: Uuid, updated_event: &AgentEvent, ) -> Result<AgentEvent, d...` — Updates an existing agent event in the database.
- pub `soft_delete` function L172-177 — `(&self, event_uuid: Uuid) -> Result<usize, diesel::result::Error>` — Soft deletes an agent event by setting its deleted_at timestamp to the current time.
- pub `hard_delete` function L188-191 — `(&self, event_uuid: Uuid) -> Result<usize, diesel::result::Error>` — Hard deletes an agent event from the database.

#### crates/brokkr-broker/src/dal/agent_k8s_events.rs

- pub `AgentK8sEventsDAL` struct L17-19 — `{ dal: &'a DAL }` — DAL for the short-lived `agent_k8s_events` telemetry table.
- pub `create` function L22-30 — `( &self, new_event: &NewAgentK8sEvent, ) -> Result<AgentK8sEvent, diesel::result...` — DAL for the short-lived `agent_k8s_events` telemetry table.
- pub `list_for_stack` function L34-47 — `( &self, stack_id: Uuid, since: DateTime<Utc>, limit: i64, ) -> Result<Vec<Agent...` — Paginated list of events for a stack within the retained window,
- pub `evict_older_than` function L50-54 — `(&self, cutoff: DateTime<Utc>) -> Result<usize, diesel::result::Error>` — Delete rows older than `cutoff`.
- pub `count` function L57-60 — `(&self) -> Result<i64, diesel::result::Error>` — Total row count (diagnostics / metrics).

#### crates/brokkr-broker/src/dal/agent_labels.rs

- pub `AgentLabelsDAL` struct L20-23 — `{ dal: &'a DAL }` — Data Access Layer for AgentLabel operations.
- pub `create` function L35-40 — `(&self, new_label: &NewAgentLabel) -> Result<AgentLabel, diesel::result::Error>` — Creates a new agent label in the database.
- pub `get` function L51-57 — `(&self, label_id: Uuid) -> Result<Option<AgentLabel>, diesel::result::Error>` — Retrieves an agent label by its ID.
- pub `list_for_agent` function L68-73 — `(&self, agent_id: Uuid) -> Result<Vec<AgentLabel>, diesel::result::Error>` — Lists all labels for a specific agent.
- pub `delete` function L84-87 — `(&self, label_id: Uuid) -> Result<usize, diesel::result::Error>` — Deletes an agent label from the database.
- pub `delete_all_for_agent` function L98-102 — `(&self, agent_id: Uuid) -> Result<usize, diesel::result::Error>` — Deletes all labels for a specific agent.
- pub `label_exists` function L115-123 — `(&self, agent_id: Uuid, label: &str) -> Result<bool, diesel::result::Error>` — Checks if a label exists for a specific agent.
- pub `delete_by_agent_and_label` function L138-150 — `( &self, agent_id: Uuid, label: &str, ) -> Result<usize, diesel::result::Error>` — Deletes a specific label for an agent using a single indexed query.

#### crates/brokkr-broker/src/dal/agent_pod_logs.rs

- pub `AgentPodLogsDAL` struct L17-19 — `{ dal: &'a DAL }` — DAL for the short-lived `agent_pod_logs` telemetry table.
- pub `create` function L22-27 — `(&self, new_line: &NewAgentPodLog) -> Result<AgentPodLog, diesel::result::Error>` — DAL for the short-lived `agent_pod_logs` telemetry table.
- pub `list_for_stack` function L29-42 — `( &self, stack_id: Uuid, since: DateTime<Utc>, limit: i64, ) -> Result<Vec<Agent...` — DAL for the short-lived `agent_pod_logs` telemetry table.
- pub `evict_older_than` function L44-48 — `(&self, cutoff: DateTime<Utc>) -> Result<usize, diesel::result::Error>` — DAL for the short-lived `agent_pod_logs` telemetry table.
- pub `count` function L50-53 — `(&self) -> Result<i64, diesel::result::Error>` — DAL for the short-lived `agent_pod_logs` telemetry table.

#### crates/brokkr-broker/src/dal/agent_targets.rs

- pub `AgentTargetsDAL` struct L19-22 — `{ dal: &'a DAL }` — Handles database operations for AgentTarget entities.
- pub `create` function L34-42 — `( &self, new_target: &NewAgentTarget, ) -> Result<AgentTarget, diesel::result::E...` — Creates a new agent target in the database.
- pub `get` function L53-59 — `(&self, target_id: Uuid) -> Result<Option<AgentTarget>, diesel::result::Error>` — Retrieves an agent target by its ID.
- pub `list` function L66-69 — `(&self) -> Result<Vec<AgentTarget>, diesel::result::Error>` — Lists all agent targets from the database.
- pub `list_for_agent` function L80-88 — `( &self, agent_id: Uuid, ) -> Result<Vec<AgentTarget>, diesel::result::Error>` — Lists all agent targets for a specific agent.
- pub `list_for_stack` function L99-107 — `( &self, stack_id: Uuid, ) -> Result<Vec<AgentTarget>, diesel::result::Error>` — Lists all agent targets for a specific stack.
- pub `delete` function L118-121 — `(&self, target_id: Uuid) -> Result<usize, diesel::result::Error>` — Deletes an agent target from the database.
- pub `delete_for_agent` function L132-136 — `(&self, agent_id: Uuid) -> Result<usize, diesel::result::Error>` — Deletes all agent targets for a specific agent.
- pub `delete_for_stack` function L147-151 — `(&self, stack_id: Uuid) -> Result<usize, diesel::result::Error>` — Deletes all agent targets for a specific stack.
- pub `delete_by_agent_and_stack` function L165-177 — `( &self, agent_id: Uuid, stack_id: Uuid, ) -> Result<usize, diesel::result::Erro...` — Deletes a specific target for an agent using a single indexed query.

#### crates/brokkr-broker/src/dal/agents.rs

- pub `AgentFilter` struct L24-29 — `{ labels: Vec<String>, annotations: Vec<(String, String)>, agent_targets: Vec<Uu...` — Struct for filtering agents based on various criteria.
- pub `AgentsDAL` struct L32-35 — `{ dal: &'a DAL }` — Data Access Layer for Agent operations.
- pub `create` function L59-79 — `(&self, new_agent: &NewAgent) -> Result<Agent, diesel::result::Error>` — Creates a new agent in the database.
- pub `get` function L91-98 — `(&self, agent_uuid: Uuid) -> Result<Option<Agent>, diesel::result::Error>` — Retrieves a non-deleted agent by its UUID.
- pub `get_including_deleted` function L110-119 — `( &self, agent_uuid: Uuid, ) -> Result<Option<Agent>, diesel::result::Error>` — Retrieves an agent by its UUID, including deleted agents.
- pub `list` function L127-132 — `(&self) -> Result<Vec<Agent>, diesel::result::Error>` — Lists all non-deleted agents from the database.
- pub `list_all` function L140-143 — `(&self) -> Result<Vec<Agent>, diesel::result::Error>` — Lists all agents from the database, including deleted ones.
- pub `update` function L156-165 — `( &self, agent_uuid: Uuid, updated_agent: &Agent, ) -> Result<Agent, diesel::res...` — Updates an existing agent in the database.
- pub `soft_delete` function L177-196 — `(&self, agent_uuid: Uuid) -> Result<usize, diesel::result::Error>` — Soft deletes an agent by setting its deleted_at timestamp to the current time.
- pub `hard_delete` function L208-211 — `(&self, agent_uuid: Uuid) -> Result<usize, diesel::result::Error>` — Hard deletes an agent from the database.
- pub `filter_by_labels` function L245-278 — `( &self, labels: Vec<String>, filter_type: FilterType, ) -> Result<Vec<Agent>, d...` — Filters agents by labels.
- pub `filter_by_annotations` function L317-380 — `( &self, annotations: Vec<(String, String)>, filter_type: FilterType, ) -> Resul...` — Filters agents by annotations.
- pub `get_agent_by_target_id` function L392-404 — `( &self, agent_target_id: Uuid, ) -> Result<Option<Agent>, diesel::result::Error...` — Retrieves an agent by its target ID.
- pub `get_agent_details` function L417-437 — `( &self, agent_id: Uuid, ) -> Result<(Vec<AgentLabel>, Vec<AgentTarget>, Vec<Age...` — Retrieves labels, targets, and annotations associated with a specific agent.
- pub `record_heartbeat` function L448-456 — `(&self, agent_id: Uuid) -> Result<(), diesel::result::Error>` — Records a heartbeat for the specified agent.
- pub `update_pak_hash` function L469-478 — `( &self, agent_uuid: Uuid, new_pak_hash: String, ) -> Result<Agent, diesel::resu...` — Updates the pak_hash for an agent.
- pub `get_by_name_and_cluster_name` function L491-503 — `( &self, name: String, cluster_name: String, ) -> Result<Option<Agent>, diesel::...` — Retrieves an agent by its name and cluster name.
- pub `get_by_pak_hash` function L518-525 — `(&self, pak_hash: &str) -> Result<Option<Agent>, diesel::result::Error>` — Retrieves an agent by its PAK hash.

#### crates/brokkr-broker/src/dal/audit_logs.rs

- pub `AuditLogsDAL` struct L20-23 — `{ dal: &'a DAL }` — Data Access Layer for AuditLog operations.
- pub `create` function L35-41 — `(&self, new_log: &NewAuditLog) -> Result<AuditLog, diesel::result::Error>` — Creates a new audit log entry.
- pub `create_batch` function L52-62 — `(&self, logs: &[NewAuditLog]) -> Result<usize, diesel::result::Error>` — Creates multiple audit log entries in a batch.
- pub `get` function L73-80 — `(&self, id: Uuid) -> Result<Option<AuditLog>, diesel::result::Error>` — Gets an audit log entry by ID.
- pub `list` function L93-143 — `( &self, filter: Option<&AuditLogFilter>, limit: Option<i64>, offset: Option<i64...` — Lists audit logs with optional filtering and pagination.
- pub `count` function L154-190 — `(&self, filter: Option<&AuditLogFilter>) -> Result<i64, diesel::result::Error>` — Counts audit logs matching the filter.
- pub `cleanup_old_logs` function L201-207 — `(&self, retention_days: i64) -> Result<usize, diesel::result::Error>` — Deletes audit logs older than the specified retention period.
- pub `get_resource_history` function L220-234 — `( &self, resource_type: &str, resource_id: Uuid, limit: i64, ) -> Result<Vec<Aud...` — Gets recent audit logs for a specific resource.
- pub `get_actor_history` function L247-261 — `( &self, actor_type: &str, actor_id: Uuid, limit: i64, ) -> Result<Vec<AuditLog>...` — Gets recent audit logs for a specific actor.
- pub `get_failed_auth_attempts` function L273-292 — `( &self, since: DateTime<Utc>, ip_address: Option<&str>, ) -> Result<Vec<AuditLo...` — Gets failed authentication attempts within a time window.

#### crates/brokkr-broker/src/dal/deployment_health.rs

- pub `DeploymentHealthDAL` struct L22-25 — `{ dal: &'a DAL }` — Data Access Layer for DeploymentHealth operations.
- pub `upsert` function L40-59 — `( &self, new_health: &NewDeploymentHealth, ) -> Result<DeploymentHealth, diesel:...` — Upserts a deployment health record.
- pub `upsert_batch` function L70-93 — `( &self, health_records: &[NewDeploymentHealth], ) -> Result<usize, diesel::resu...` — Upserts multiple deployment health records in a batch.
- pub `get_by_agent_and_deployment` function L105-117 — `( &self, agent_id: Uuid, deployment_object_id: Uuid, ) -> Result<Option<Deployme...` — Gets the health record for a specific agent and deployment object.
- pub `get` function L128-135 — `(&self, id: Uuid) -> Result<Option<DeploymentHealth>, diesel::result::Error>` — Gets the health record by its ID.
- pub `list_by_deployment_object` function L146-156 — `( &self, deployment_object_id: Uuid, ) -> Result<Vec<DeploymentHealth>, diesel::...` — Lists all health records for a specific deployment object (across all agents).
- pub `list_by_agent` function L167-177 — `( &self, agent_id: Uuid, ) -> Result<Vec<DeploymentHealth>, diesel::result::Erro...` — Lists all health records for a specific agent.
- pub `list_by_stack` function L188-201 — `( &self, stack_id: Uuid, ) -> Result<Vec<DeploymentHealth>, diesel::result::Erro...` — Lists all health records for deployment objects in a specific stack.
- pub `list_by_status` function L212-222 — `( &self, status: &str, ) -> Result<Vec<DeploymentHealth>, diesel::result::Error>` — Lists all health records with a specific status.
- pub `delete_by_agent_and_deployment` function L234-247 — `( &self, agent_id: Uuid, deployment_object_id: Uuid, ) -> Result<usize, diesel::...` — Deletes the health record for a specific agent and deployment object.
- pub `delete_by_agent` function L258-263 — `(&self, agent_id: Uuid) -> Result<usize, diesel::result::Error>` — Deletes all health records for a specific agent.

#### crates/brokkr-broker/src/dal/deployment_objects.rs

- pub `DeploymentObjectsDAL` struct L26-29 — `{ dal: &'a DAL }` — Data Access Layer for DeploymentObject operations.
- pub `create` function L41-63 — `( &self, new_deployment_object: &NewDeploymentObject, ) -> Result<DeploymentObje...` — Creates a new deployment object in the database.
- pub `get` function L74-84 — `( &self, deployment_object_uuid: Uuid, ) -> Result<Option<DeploymentObject>, die...` — Retrieves a non-deleted deployment object by its UUID.
- pub `get_including_deleted` function L95-104 — `( &self, deployment_object_uuid: Uuid, ) -> Result<Option<DeploymentObject>, die...` — Retrieves a deployment object by its UUID, including deleted objects.
- pub `list_for_stack` function L115-125 — `( &self, stack_id: Uuid, ) -> Result<Vec<DeploymentObject>, diesel::result::Erro...` — Lists all non-deleted deployment objects for a specific stack.
- pub `list_all_for_stack` function L136-145 — `( &self, stack_id: Uuid, ) -> Result<Vec<DeploymentObject>, diesel::result::Erro...` — Lists all deployment objects for a specific stack, including deleted ones.
- pub `soft_delete` function L156-188 — `( &self, deployment_object_uuid: Uuid, ) -> Result<usize, diesel::result::Error>` — Soft deletes a deployment object by setting its deleted_at timestamp to the current time.
- pub `get_latest_for_stack` function L199-210 — `( &self, stack_id: Uuid, ) -> Result<Option<DeploymentObject>, diesel::result::E...` — Retrieves the latest non-deleted deployment object for a specific stack.
- pub `get_target_state_for_agent` function L229-267 — `( &self, agent_id: Uuid, include_deployed: bool, ) -> Result<Vec<DeploymentObjec...` — Retrieves a list of undeployed objects for an agent based on its responsibilities.
- pub `search` function L279-289 — `( &self, yaml_checksum: &str, ) -> Result<Vec<DeploymentObject>, diesel::result:...` — Searches for deployment objects by checksum.
- pub `get_desired_state_for_agent` function L304-324 — `( &self, agent_id: Uuid, ) -> Result<Vec<DeploymentObject>, diesel::result::Erro...` — Retrieves applicable deployment objects for a given agent.

#### crates/brokkr-broker/src/dal/diagnostic_requests.rs

- pub `DiagnosticRequestsDAL` struct L22-25 — `{ dal: &'a DAL }` — Data Access Layer for DiagnosticRequest operations.
- pub `create` function L37-46 — `( &self, new_request: &NewDiagnosticRequest, ) -> Result<DiagnosticRequest, dies...` — Creates a new diagnostic request.
- pub `get` function L57-64 — `(&self, id: Uuid) -> Result<Option<DiagnosticRequest>, diesel::result::Error>` — Gets a diagnostic request by ID.
- pub `get_pending_for_agent` function L75-87 — `( &self, agent_id: Uuid, ) -> Result<Vec<DiagnosticRequest>, diesel::result::Err...` — Gets all pending diagnostic requests for a specific agent.
- pub `claim` function L98-110 — `(&self, id: Uuid) -> Result<DiagnosticRequest, diesel::result::Error>` — Claims a diagnostic request for processing.
- pub `complete` function L121-133 — `(&self, id: Uuid) -> Result<DiagnosticRequest, diesel::result::Error>` — Marks a diagnostic request as completed.
- pub `fail` function L144-156 — `(&self, id: Uuid) -> Result<DiagnosticRequest, diesel::result::Error>` — Marks a diagnostic request as failed.
- pub `list_by_deployment_object` function L167-177 — `( &self, deployment_object_id: Uuid, ) -> Result<Vec<DiagnosticRequest>, diesel:...` — Lists all diagnostic requests for a specific deployment object.
- pub `expire_old_requests` function L184-194 — `(&self) -> Result<usize, diesel::result::Error>` — Expires all pending requests that have passed their expiry time.
- pub `cleanup_old_requests` function L205-221 — `(&self, max_age_hours: i64) -> Result<usize, diesel::result::Error>` — Deletes expired and completed requests older than the given age in hours.
- pub `delete` function L232-237 — `(&self, id: Uuid) -> Result<usize, diesel::result::Error>` — Deletes a diagnostic request by ID.

#### crates/brokkr-broker/src/dal/diagnostic_results.rs

- pub `DiagnosticResultsDAL` struct L19-22 — `{ dal: &'a DAL }` — Data Access Layer for DiagnosticResult operations.
- pub `create` function L34-43 — `( &self, new_result: &NewDiagnosticResult, ) -> Result<DiagnosticResult, diesel:...` — Creates a new diagnostic result.
- pub `get` function L54-61 — `(&self, id: Uuid) -> Result<Option<DiagnosticResult>, diesel::result::Error>` — Gets a diagnostic result by ID.
- pub `get_by_request` function L72-82 — `( &self, request_id: Uuid, ) -> Result<Option<DiagnosticResult>, diesel::result:...` — Gets the diagnostic result for a specific request.
- pub `delete` function L93-98 — `(&self, id: Uuid) -> Result<usize, diesel::result::Error>` — Deletes a diagnostic result by ID.
- pub `delete_by_request` function L109-116 — `(&self, request_id: Uuid) -> Result<usize, diesel::result::Error>` — Deletes all diagnostic results for a specific request.

#### crates/brokkr-broker/src/dal/generators.rs

- pub `GeneratorsDAL` struct L19-22 — `{ dal: &'a DAL }` — Data Access Layer for Generator operations.
- pub `create` function L34-39 — `(&self, new_generator: &NewGenerator) -> Result<Generator, diesel::result::Error...` — Creates a new generator in the database.
- pub `get` function L50-57 — `(&self, generator_uuid: Uuid) -> Result<Option<Generator>, diesel::result::Error...` — Retrieves a non-deleted generator by its UUID.
- pub `get_including_deleted` function L68-77 — `( &self, generator_uuid: Uuid, ) -> Result<Option<Generator>, diesel::result::Er...` — Retrieves a generator by its UUID, including deleted generators.
- pub `list` function L84-89 — `(&self) -> Result<Vec<Generator>, diesel::result::Error>` — Lists all non-deleted generators from the database.
- pub `list_all` function L96-99 — `(&self) -> Result<Vec<Generator>, diesel::result::Error>` — Lists all generators from the database, including deleted ones.
- pub `update` function L111-120 — `( &self, generator_uuid: Uuid, updated_generator: &Generator, ) -> Result<Genera...` — Updates an existing generator in the database.
- pub `soft_delete` function L131-136 — `(&self, generator_id: Uuid) -> Result<usize, diesel::result::Error>` — Soft deletes a generator by setting its deleted_at timestamp to the current time.
- pub `hard_delete` function L147-150 — `(&self, generator_uuid: Uuid) -> Result<usize, diesel::result::Error>` — Hard deletes a generator from the database.
- pub `update_pak_hash` function L162-171 — `( &self, generator_uuid: Uuid, new_pak_hash: String, ) -> Result<Generator, dies...` — Updates the pak_hash for a generator.
- pub `update_last_active` function L182-193 — `( &self, generator_uuid: Uuid, ) -> Result<Generator, diesel::result::Error>` — Updates the last_active_at timestamp for a generator and sets is_active to true.
- pub `get_by_name` function L204-214 — `( &self, generator_name: &str, ) -> Result<Option<Generator>, diesel::result::Er...` — Retrieves a non-deleted generator by its name.
- pub `get_by_active_status` function L225-234 — `( &self, active: bool, ) -> Result<Vec<Generator>, diesel::result::Error>` — Retrieves non-deleted generators by their active status.
- pub `get_by_pak_hash` function L249-259 — `( &self, pak_hash: &str, ) -> Result<Option<Generator>, diesel::result::Error>` — Retrieves a generator by its PAK hash.

#### crates/brokkr-broker/src/dal/mod.rs

- pub `DalError` enum L43-50 — `ConnectionPool | Query | NotFound` — Error types for DAL operations.
- pub `agents` module L93 — `-` — ```
- pub `agent_annotations` module L96 — `-` — ```
- pub `audit_logs` module L99 — `-` — ```
- pub `agent_events` module L102 — `-` — ```
- pub `agent_k8s_events` module L105 — `-` — ```
- pub `agent_pod_logs` module L108 — `-` — ```
- pub `agent_labels` module L111 — `-` — ```
- pub `agent_targets` module L114 — `-` — ```
- pub `stacks` module L117 — `-` — ```
- pub `stack_annotations` module L120 — `-` — ```
- pub `stack_labels` module L123 — `-` — ```
- pub `deployment_health` module L126 — `-` — ```
- pub `deployment_objects` module L129 — `-` — ```
- pub `diagnostic_requests` module L132 — `-` — ```
- pub `diagnostic_results` module L135 — `-` — ```
- pub `generators` module L138 — `-` — ```
- pub `templates` module L141 — `-` — ```
- pub `template_labels` module L144 — `-` — ```
- pub `template_annotations` module L147 — `-` — ```
- pub `template_targets` module L150 — `-` — ```
- pub `rendered_deployment_objects` module L153 — `-` — ```
- pub `webhook_deliveries` module L156 — `-` — ```
- pub `webhook_subscriptions` module L159 — `-` — ```
- pub `work_orders` module L162 — `-` — ```
- pub `DAL` struct L171-176 — `{ pool: ConnectionPool, auth_cache: Option<Cache<String, AuthPayload>> }` — The main Data Access Layer struct.
- pub `new` function L188-193 — `(pool: ConnectionPool) -> Self` — Creates a new DAL instance with the given connection pool.
- pub `new_with_auth_cache` function L201-213 — `(pool: ConnectionPool, auth_cache_ttl_seconds: u64) -> Self` — Creates a new DAL instance with an auth cache.
- pub `invalidate_auth_cache` function L216-220 — `(&self, pak_hash: &str)` — Invalidates a specific entry in the auth cache by PAK hash.
- pub `invalidate_all_auth_cache` function L223-227 — `(&self)` — Invalidates all entries in the auth cache.
- pub `agents` function L234-236 — `(&self) -> AgentsDAL<'_>` — Provides access to the Agents Data Access Layer.
- pub `agent_annotations` function L243-245 — `(&self) -> AgentAnnotationsDAL<'_>` — Provides access to the Agent Annotations Data Access Layer.
- pub `agent_events` function L252-254 — `(&self) -> AgentEventsDAL<'_>` — Provides access to the Agent Events Data Access Layer.
- pub `agent_k8s_events` function L259-261 — `(&self) -> AgentK8sEventsDAL<'_>` — Provides access to the agent kube-Events telemetry buffer
- pub `agent_pod_logs` function L266-268 — `(&self) -> AgentPodLogsDAL<'_>` — Provides access to the agent pod-logs telemetry buffer
- pub `agent_labels` function L275-277 — `(&self) -> AgentLabelsDAL<'_>` — Provides access to the Agent Labels Data Access Layer.
- pub `agent_targets` function L284-286 — `(&self) -> AgentTargetsDAL<'_>` — Provides access to the Agent Targets Data Access Layer.
- pub `stack_labels` function L293-295 — `(&self) -> StackLabelsDAL<'_>` — Provides access to the Stack Labels Data Access Layer.
- pub `stack_annotations` function L302-304 — `(&self) -> StackAnnotationsDAL<'_>` — Provides access to the Stack Annotations Data Access Layer.
- pub `stacks` function L311-313 — `(&self) -> StacksDAL<'_>` — Provides access to the Stacks Data Access Layer.
- pub `deployment_health` function L320-322 — `(&self) -> DeploymentHealthDAL<'_>` — Provides access to the Deployment Health Data Access Layer.
- pub `deployment_objects` function L329-331 — `(&self) -> DeploymentObjectsDAL<'_>` — Provides access to the Deployment Objects Data Access Layer.
- pub `generators` function L338-340 — `(&self) -> GeneratorsDAL<'_>` — Provides access to the Generators Data Access Layer.
- pub `templates` function L347-349 — `(&self) -> TemplatesDAL<'_>` — Provides access to the Templates Data Access Layer.
- pub `template_labels` function L356-358 — `(&self) -> TemplateLabelsDAL<'_>` — Provides access to the Template Labels Data Access Layer.
- pub `template_annotations` function L365-367 — `(&self) -> TemplateAnnotationsDAL<'_>` — Provides access to the Template Annotations Data Access Layer.
- pub `template_targets` function L374-376 — `(&self) -> TemplateTargetsDAL<'_>` — Provides access to the Template Targets Data Access Layer.
- pub `rendered_deployment_objects` function L383-385 — `(&self) -> RenderedDeploymentObjectsDAL<'_>` — Provides access to the Rendered Deployment Objects Data Access Layer.
- pub `work_orders` function L392-394 — `(&self) -> WorkOrdersDAL<'_>` — Provides access to the Work Orders Data Access Layer.
- pub `diagnostic_requests` function L401-403 — `(&self) -> DiagnosticRequestsDAL<'_>` — Provides access to the Diagnostic Requests Data Access Layer.
- pub `diagnostic_results` function L410-412 — `(&self) -> DiagnosticResultsDAL<'_>` — Provides access to the Diagnostic Results Data Access Layer.
- pub `webhook_subscriptions` function L419-421 — `(&self) -> WebhookSubscriptionsDAL<'_>` — Provides access to the Webhook Subscriptions Data Access Layer.
- pub `webhook_deliveries` function L428-430 — `(&self) -> WebhookDeliveriesDAL<'_>` — Provides access to the Webhook Deliveries Data Access Layer.
- pub `audit_logs` function L437-439 — `(&self) -> AuditLogsDAL<'_>` — Provides access to the Audit Logs Data Access Layer.
- pub `FilterType` enum L443-446 — `And | Or` — ```
-  `DalError` type L52-60 — `= DalError` — ```
-  `fmt` function L53-59 — `(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result` — ```
-  `DalError` type L62 — `= DalError` — ```
-  `DalError` type L64-68 — `= DalError` — ```
-  `from` function L65-67 — `(e: r2d2::Error) -> Self` — ```
-  `DalError` type L70-77 — `= DalError` — ```
-  `from` function L71-76 — `(e: diesel::result::Error) -> Self` — ```
-  `DalError` type L79-91 — `impl IntoResponse for DalError` — ```
-  `into_response` function L80-90 — `(self) -> Response` — ```
-  `DAL` type L178-440 — `= DAL` — ```

#### crates/brokkr-broker/src/dal/rendered_deployment_objects.rs

- pub `RenderedDeploymentObjectsDAL` struct L22-25 — `{ dal: &'a DAL }` — Handles database operations for RenderedDeploymentObject entities.
- pub `create` function L37-45 — `( &self, new_record: &NewRenderedDeploymentObject, ) -> Result<RenderedDeploymen...` — Creates a new rendered deployment object provenance record in the database.
- pub `get` function L56-65 — `( &self, record_id: Uuid, ) -> Result<Option<RenderedDeploymentObject>, diesel::...` — Retrieves a rendered deployment object provenance record by its ID.
- pub `get_by_deployment_object` function L76-85 — `( &self, deployment_object_id: Uuid, ) -> Result<Option<RenderedDeploymentObject...` — Retrieves the provenance record for a specific deployment object.
- pub `list_by_template` function L97-115 — `( &self, template_id: Uuid, version: Option<i32>, ) -> Result<Vec<RenderedDeploy...` — Lists all provenance records for a specific template.
- pub `list` function L122-127 — `(&self) -> Result<Vec<RenderedDeploymentObject>, diesel::result::Error>` — Lists all provenance records from the database.
- pub `delete` function L138-145 — `(&self, record_id: Uuid) -> Result<usize, diesel::result::Error>` — Deletes a provenance record from the database.
- pub `delete_for_deployment_object` function L156-166 — `( &self, deployment_object_id: Uuid, ) -> Result<usize, diesel::result::Error>` — Deletes all provenance records for a specific deployment object.
- pub `delete_for_template` function L177-184 — `(&self, template_id: Uuid) -> Result<usize, diesel::result::Error>` — Deletes all provenance records for a specific template.

#### crates/brokkr-broker/src/dal/stack_annotations.rs

- pub `StackAnnotationsDAL` struct L19-22 — `{ dal: &'a DAL }` — Handles database operations for Stack Annotations.
- pub `create` function L38-46 — `( &self, new_annotation: &NewStackAnnotation, ) -> Result<StackAnnotation, diese...` — Creates a new stack annotation in the database.
- pub `get` function L61-70 — `( &self, annotation_id: Uuid, ) -> Result<Option<StackAnnotation>, diesel::resul...` — Retrieves a stack annotation by its ID.
- pub `list_for_stack` function L85-93 — `( &self, stack_id: Uuid, ) -> Result<Vec<StackAnnotation>, diesel::result::Error...` — Lists all annotations for a specific stack.
- pub `update` function L109-118 — `( &self, annotation_id: Uuid, updated_annotation: &StackAnnotation, ) -> Result<...` — Updates an existing stack annotation in the database.
- pub `delete` function L133-137 — `(&self, annotation_id: Uuid) -> Result<usize, diesel::result::Error>` — Deletes a stack annotation from the database.
- pub `delete_all_for_stack` function L152-156 — `(&self, stack_id: Uuid) -> Result<usize, diesel::result::Error>` — Deletes all annotations for a specific stack.
- pub `delete_by_stack_and_key` function L174-186 — `( &self, stack_id: Uuid, key: &str, ) -> Result<usize, diesel::result::Error>` — Deletes a specific annotation for a stack using a single indexed query.

#### crates/brokkr-broker/src/dal/stack_labels.rs

- pub `StackLabelsDAL` struct L19-22 — `{ dal: &'a DAL }` — Handles database operations for Stack Labels.
- pub `create` function L38-43 — `(&self, new_label: &NewStackLabel) -> Result<StackLabel, diesel::result::Error>` — Creates a new stack label in the database.
- pub `get` function L58-64 — `(&self, label_id: Uuid) -> Result<Option<StackLabel>, diesel::result::Error>` — Retrieves a stack label by its ID.
- pub `list_for_stack` function L79-84 — `(&self, stack_id: Uuid) -> Result<Vec<StackLabel>, diesel::result::Error>` — Lists all labels for a specific stack.
- pub `delete` function L99-102 — `(&self, label_id: Uuid) -> Result<usize, diesel::result::Error>` — Deletes a stack label from the database.
- pub `delete_all_for_stack` function L117-121 — `(&self, stack_id: Uuid) -> Result<usize, diesel::result::Error>` — Deletes all labels for a specific stack.
- pub `delete_by_stack_and_label` function L139-151 — `( &self, stack_id: Uuid, label: &str, ) -> Result<usize, diesel::result::Error>` — Deletes a specific label for a stack using a single indexed query.

#### crates/brokkr-broker/src/dal/stacks.rs

- pub `StacksDAL` struct L27-30 — `{ dal: &'a DAL }` — Data Access Layer for Stack operations.
- pub `create` function L42-58 — `(&self, new_stack: &NewStack) -> Result<Stack, diesel::result::Error>` — Creates a new stack in the database.
- pub `get` function L69-75 — `(&self, stack_uuids: Vec<Uuid>) -> Result<Vec<Stack>, diesel::result::Error>` — Retrieves non-deleted stacks by their UUIDs.
- pub `get_including_deleted` function L86-95 — `( &self, stack_uuid: Uuid, ) -> Result<Option<Stack>, diesel::result::Error>` — Retrieves a stack by its UUID, including deleted stacks.
- pub `list` function L102-107 — `(&self) -> Result<Vec<Stack>, diesel::result::Error>` — Lists all non-deleted stacks from the database.
- pub `list_for_generator` function L119-128 — `( &self, generator_id: Uuid, ) -> Result<Vec<Stack>, diesel::result::Error>` — Lists all non-deleted stacks owned by a specific generator.
- pub `list_all` function L135-138 — `(&self) -> Result<Vec<Stack>, diesel::result::Error>` — Lists all stacks from the database, including deleted ones.
- pub `update` function L150-159 — `( &self, stack_uuid: Uuid, updated_stack: &Stack, ) -> Result<Stack, diesel::res...` — Updates an existing stack in the database.
- pub `soft_delete` function L170-186 — `(&self, stack_uuid: Uuid) -> Result<usize, diesel::result::Error>` — Soft deletes a stack by setting its deleted_at timestamp to the current time.
- pub `hard_delete` function L197-200 — `(&self, stack_uuid: Uuid) -> Result<usize, diesel::result::Error>` — Hard deletes a stack from the database.
- pub `filter_by_labels` function L212-245 — `( &self, labels: Vec<String>, filter_type: FilterType, ) -> Result<Vec<Stack>, d...` — Filters stacks by labels.
- pub `filter_by_annotations` function L257-306 — `( &self, annotations: Vec<(String, String)>, filter_type: FilterType, ) -> Resul...` — Filters stacks by annotations.
- pub `get_associated_stacks` function L320-375 — `( &self, agent_id: Uuid, ) -> Result<Vec<Stack>, diesel::result::Error>` — Retrieves all stacks associated with a specific agent based on its labels, annotations, and targets.

#### crates/brokkr-broker/src/dal/template_annotations.rs

- pub `TemplateAnnotationsDAL` struct L19-22 — `{ dal: &'a DAL }` — Handles database operations for Template Annotations.
- pub `create` function L38-46 — `( &self, new_annotation: &NewTemplateAnnotation, ) -> Result<TemplateAnnotation,...` — Creates a new template annotation in the database.
- pub `get` function L61-70 — `( &self, annotation_id: Uuid, ) -> Result<Option<TemplateAnnotation>, diesel::re...` — Retrieves a template annotation by its ID.
- pub `list_for_template` function L85-93 — `( &self, template_id: Uuid, ) -> Result<Vec<TemplateAnnotation>, diesel::result:...` — Lists all annotations for a specific template.
- pub `delete` function L108-114 — `(&self, annotation_id: Uuid) -> Result<usize, diesel::result::Error>` — Deletes a template annotation from the database.
- pub `delete_all_for_template` function L129-138 — `( &self, template_id: Uuid, ) -> Result<usize, diesel::result::Error>` — Deletes all annotations for a specific template.

#### crates/brokkr-broker/src/dal/template_labels.rs

- pub `TemplateLabelsDAL` struct L19-22 — `{ dal: &'a DAL }` — Handles database operations for Template Labels.
- pub `create` function L38-46 — `( &self, new_label: &NewTemplateLabel, ) -> Result<TemplateLabel, diesel::result...` — Creates a new template label in the database.
- pub `get` function L61-67 — `(&self, label_id: Uuid) -> Result<Option<TemplateLabel>, diesel::result::Error>` — Retrieves a template label by its ID.
- pub `list_for_template` function L82-90 — `( &self, template_id: Uuid, ) -> Result<Vec<TemplateLabel>, diesel::result::Erro...` — Lists all labels for a specific template.
- pub `delete` function L105-109 — `(&self, label_id: Uuid) -> Result<usize, diesel::result::Error>` — Deletes a template label from the database.
- pub `delete_all_for_template` function L124-131 — `( &self, template_id: Uuid, ) -> Result<usize, diesel::result::Error>` — Deletes all labels for a specific template.

#### crates/brokkr-broker/src/dal/template_targets.rs

- pub `TemplateTargetsDAL` struct L19-22 — `{ dal: &'a DAL }` — Handles database operations for TemplateTarget entities.
- pub `create` function L34-42 — `( &self, new_target: &NewTemplateTarget, ) -> Result<TemplateTarget, diesel::res...` — Creates a new template target in the database.
- pub `get` function L53-59 — `(&self, target_id: Uuid) -> Result<Option<TemplateTarget>, diesel::result::Error...` — Retrieves a template target by its ID.
- pub `list` function L66-69 — `(&self) -> Result<Vec<TemplateTarget>, diesel::result::Error>` — Lists all template targets from the database.
- pub `list_for_template` function L80-88 — `( &self, template_id: Uuid, ) -> Result<Vec<TemplateTarget>, diesel::result::Err...` — Lists all template targets for a specific template.
- pub `list_for_stack` function L99-107 — `( &self, stack_id: Uuid, ) -> Result<Vec<TemplateTarget>, diesel::result::Error>` — Lists all template targets for a specific stack.
- pub `exists` function L119-127 — `(&self, template_id: Uuid, stack_id: Uuid) -> Result<bool, diesel::result::Error...` — Checks if a specific template-stack association exists.
- pub `delete` function L138-142 — `(&self, target_id: Uuid) -> Result<usize, diesel::result::Error>` — Deletes a template target from the database.
- pub `delete_for_template` function L153-159 — `(&self, template_id: Uuid) -> Result<usize, diesel::result::Error>` — Deletes all template targets for a specific template.
- pub `delete_for_stack` function L170-174 — `(&self, stack_id: Uuid) -> Result<usize, diesel::result::Error>` — Deletes all template targets for a specific stack.

#### crates/brokkr-broker/src/dal/templates.rs

- pub `TemplatesDAL` struct L22-25 — `{ dal: &'a DAL }` — Data Access Layer for Stack Template operations.
- pub `create` function L37-45 — `( &self, new_template: &NewStackTemplate, ) -> Result<StackTemplate, diesel::res...` — Creates a new stack template in the database.
- pub `create_new_version` function L63-104 — `( &self, generator_id: Option<Uuid>, name: String, description: Option<String>, ...` — Creates a new version of an existing template.
- pub `get` function L115-122 — `(&self, template_id: Uuid) -> Result<Option<StackTemplate>, diesel::result::Erro...` — Retrieves a non-deleted stack template by its UUID.
- pub `get_including_deleted` function L133-142 — `( &self, template_id: Uuid, ) -> Result<Option<StackTemplate>, diesel::result::E...` — Retrieves a stack template by its UUID, including deleted templates.
- pub `list` function L150-155 — `(&self) -> Result<Vec<StackTemplate>, diesel::result::Error>` — Lists all non-deleted stack templates from the database.
- pub `list_all` function L163-166 — `(&self) -> Result<Vec<StackTemplate>, diesel::result::Error>` — Lists all stack templates from the database, including deleted ones.
- pub `list_by_generator` function L178-187 — `( &self, generator_id: Uuid, ) -> Result<Vec<StackTemplate>, diesel::result::Err...` — Lists all non-deleted stack templates for a specific generator.
- pub `get_latest_version` function L200-222 — `( &self, generator_id: Option<Uuid>, name: &str, ) -> Result<Option<StackTemplat...` — Gets the latest version of a template by name and generator_id.
- pub `list_versions` function L235-255 — `( &self, generator_id: Option<Uuid>, name: &str, ) -> Result<Vec<StackTemplate>,...` — Lists all versions of a template by name and generator_id.
- pub `list_for_generator` function L266-275 — `( &self, generator_id: Uuid, ) -> Result<Vec<StackTemplate>, diesel::result::Err...` — Lists all non-deleted templates for a specific generator.
- pub `list_system_templates` function L282-288 — `(&self) -> Result<Vec<StackTemplate>, diesel::result::Error>` — Lists all non-deleted system templates (generator_id IS NULL).
- pub `soft_delete` function L299-304 — `(&self, template_id: Uuid) -> Result<usize, diesel::result::Error>` — Soft deletes a stack template by setting its deleted_at timestamp.
- pub `hard_delete` function L315-319 — `(&self, template_id: Uuid) -> Result<usize, diesel::result::Error>` — Hard deletes a stack template from the database.
- pub `filter_by_labels` function L331-364 — `( &self, labels: Vec<String>, filter_type: FilterType, ) -> Result<Vec<StackTemp...` — Filters templates by labels.
- pub `filter_by_annotations` function L376-440 — `( &self, annotations: Vec<(String, String)>, filter_type: FilterType, ) -> Resul...` — Filters templates by annotations.

#### crates/brokkr-broker/src/dal/webhook_deliveries.rs

- pub `WebhookDeliveriesDAL` struct L40-43 — `{ dal: &'a DAL }` — Data Access Layer for WebhookDelivery operations.
- pub `create` function L55-64 — `( &self, new_delivery: &NewWebhookDelivery, ) -> Result<WebhookDelivery, diesel:...` — Creates a new webhook delivery.
- pub `get` function L75-82 — `(&self, id: Uuid) -> Result<Option<WebhookDelivery>, diesel::result::Error>` — Gets a webhook delivery by ID.
- pub `claim_for_broker` function L101-140 — `( &self, limit: i64, ttl_seconds: Option<i64>, ) -> Result<Vec<WebhookDelivery>,...` — Claims pending deliveries for broker processing (target_labels is NULL or empty).
- pub `claim_for_agent` function L156-210 — `( &self, agent_id: Uuid, agent_labels: &[String], limit: i64, ttl_seconds: Optio...` — Claims pending deliveries for an agent based on label matching.
- pub `release_expired` function L219-234 — `(&self) -> Result<usize, diesel::result::Error>` — Releases expired acquired deliveries back to pending status.
- pub `process_retries` function L243-257 — `(&self) -> Result<usize, diesel::result::Error>` — Moves failed deliveries back to pending when retry time is reached.
- pub `mark_success` function L272-287 — `(&self, id: Uuid) -> Result<WebhookDelivery, diesel::result::Error>` — Records a successful delivery.
- pub `mark_failed` function L300-347 — `( &self, id: Uuid, error: &str, max_retries: i32, ) -> Result<WebhookDelivery, d...` — Records a failed delivery attempt and schedules retry if applicable.
- pub `list_for_subscription` function L365-387 — `( &self, subscription_id: Uuid, status_filter: Option<&str>, limit: i64, offset:...` — Lists deliveries for a subscription with optional filtering.
- pub `retry` function L398-422 — `(&self, id: Uuid) -> Result<Option<WebhookDelivery>, diesel::result::Error>` — Retries a failed or dead delivery.
- pub `cleanup_old` function L433-448 — `(&self, retention_days: i64) -> Result<usize, diesel::result::Error>` — Deletes old deliveries based on retention policy.
- pub `get_stats` function L459-479 — `(&self, subscription_id: Uuid) -> Result<DeliveryStats, diesel::result::Error>` — Gets delivery statistics for a subscription.
- pub `DeliveryStats` struct L484-495 — `{ pending: i64, acquired: i64, success: i64, failed: i64, dead: i64 }` — Statistics about webhook deliveries.
-  `DEFAULT_CLAIM_TTL_SECONDS` variable L37 — `: i64` — Default TTL for acquired deliveries (60 seconds).

#### crates/brokkr-broker/src/dal/webhook_subscriptions.rs

- pub `WebhookSubscriptionsDAL` struct L21-24 — `{ dal: &'a DAL }` — Data Access Layer for WebhookSubscription operations.
- pub `create` function L36-45 — `( &self, new_subscription: &NewWebhookSubscription, ) -> Result<WebhookSubscript...` — Creates a new webhook subscription.
- pub `get` function L56-63 — `(&self, id: Uuid) -> Result<Option<WebhookSubscription>, diesel::result::Error>` — Gets a webhook subscription by ID.
- pub `list` function L74-89 — `( &self, enabled_only: bool, ) -> Result<Vec<WebhookSubscription>, diesel::resul...` — Lists all webhook subscriptions.
- pub `get_matching_subscriptions` function L100-126 — `( &self, event_type: &str, ) -> Result<Vec<WebhookSubscription>, diesel::result:...` — Gets all enabled subscriptions that match a given event type.
- pub `update` function L138-148 — `( &self, id: Uuid, update: &UpdateWebhookSubscription, ) -> Result<WebhookSubscr...` — Updates a webhook subscription.
- pub `delete` function L159-164 — `(&self, id: Uuid) -> Result<usize, diesel::result::Error>` — Deletes a webhook subscription.
- pub `set_enabled` function L176-186 — `( &self, id: Uuid, enabled: bool, ) -> Result<WebhookSubscription, diesel::resul...` — Enables or disables a subscription.
-  `matches_event_pattern` function L195-205 — `(pattern: &str, event_type: &str) -> bool` — Matches an event type against a pattern.
-  `tests` module L208-232 — `-` — It includes methods for creating, updating, deleting, and querying webhook subscriptions.
-  `test_matches_event_pattern_exact` function L212-215 — `()` — It includes methods for creating, updating, deleting, and querying webhook subscriptions.
-  `test_matches_event_pattern_wildcard_suffix` function L218-224 — `()` — It includes methods for creating, updating, deleting, and querying webhook subscriptions.
-  `test_matches_event_pattern_full_wildcard` function L227-231 — `()` — It includes methods for creating, updating, deleting, and querying webhook subscriptions.

#### crates/brokkr-broker/src/dal/work_orders.rs

- pub `WorkOrdersDAL` struct L49-52 — `{ dal: &'a DAL }` — Data Access Layer for WorkOrder operations.
- pub `create` function L68-90 — `( &self, new_work_order: &NewWorkOrder, ) -> Result<WorkOrder, diesel::result::E...` — Creates a new work order in the database.
- pub `get` function L101-107 — `(&self, work_order_id: Uuid) -> Result<Option<WorkOrder>, diesel::result::Error>` — Retrieves a work order by its UUID.
- pub `list` function L114-119 — `(&self) -> Result<Vec<WorkOrder>, diesel::result::Error>` — Lists all work orders from the database.
- pub `list_filtered` function L131-151 — `( &self, status: Option<&str>, work_type: Option<&str>, ) -> Result<Vec<WorkOrde...` — Lists work orders filtered by status and/or work type.
- pub `delete` function L164-167 — `(&self, work_order_id: Uuid) -> Result<usize, diesel::result::Error>` — Deletes a work order by its UUID (hard delete).
- pub `list_pending_for_agent` function L190-267 — `( &self, agent_id: Uuid, work_type: Option<&str>, ) -> Result<Vec<WorkOrder>, di...` — Lists pending work orders that are claimable by a specific agent.
- pub `claim` function L288-330 — `( &self, work_order_id: Uuid, agent_id: Uuid, ) -> Result<WorkOrder, diesel::res...` — Atomically claims a work order for an agent.
- pub `release` function L408-427 — `( &self, work_order_id: Uuid, agent_id: Uuid, ) -> Result<WorkOrder, diesel::res...` — Releases a claimed work order back to PENDING status.
- pub `complete_success` function L443-473 — `( &self, work_order_id: Uuid, result_message: Option<String>, ) -> Result<WorkOr...` — Completes a work order successfully and moves it to the log.
- pub `complete_failure` function L513-573 — `( &self, work_order_id: Uuid, error_message: String, retryable: bool, ) -> Resul...` — Completes a work order with failure.
- pub `process_retry_pending` function L586-600 — `(&self) -> Result<usize, diesel::result::Error>` — Resets RETRY_PENDING work orders to PENDING if their backoff period has elapsed.
- pub `process_stale_claims` function L611-625 — `(&self) -> Result<usize, diesel::result::Error>` — Resets stale claimed work orders to PENDING.
- pub `add_target` function L640-648 — `( &self, new_target: &NewWorkOrderTarget, ) -> Result<WorkOrderTarget, diesel::r...` — Adds an agent as a target for a work order.
- pub `add_targets` function L660-675 — `( &self, work_order_id: Uuid, agent_ids: &[Uuid], ) -> Result<usize, diesel::res...` — Adds multiple agents as targets for a work order.
- pub `list_targets` function L686-694 — `( &self, work_order_id: Uuid, ) -> Result<Vec<WorkOrderTarget>, diesel::result::...` — Lists all targets for a work order.
- pub `remove_target` function L706-718 — `( &self, work_order_id: Uuid, agent_id: Uuid, ) -> Result<usize, diesel::result:...` — Removes a target from a work order.
- pub `get_log` function L733-739 — `(&self, log_id: Uuid) -> Result<Option<WorkOrderLog>, diesel::result::Error>` — Retrieves a work order log entry by its UUID.
- pub `list_log` function L753-783 — `( &self, work_type: Option<&str>, success: Option<bool>, agent_id: Option<Uuid>,...` — Lists work order log entries with optional filtering.
- pub `add_label` function L798-806 — `( &self, new_label: &NewWorkOrderLabel, ) -> Result<WorkOrderLabel, diesel::resu...` — Adds a label to a work order.
- pub `add_labels` function L818-833 — `( &self, work_order_id: Uuid, labels: &[String], ) -> Result<usize, diesel::resu...` — Adds multiple labels to a work order.
- pub `list_labels` function L844-852 — `( &self, work_order_id: Uuid, ) -> Result<Vec<WorkOrderLabel>, diesel::result::E...` — Lists all labels for a work order.
- pub `remove_label` function L864-876 — `( &self, work_order_id: Uuid, label: &str, ) -> Result<usize, diesel::result::Er...` — Removes a label from a work order.
- pub `add_annotation` function L891-899 — `( &self, new_annotation: &NewWorkOrderAnnotation, ) -> Result<WorkOrderAnnotatio...` — Adds an annotation to a work order.
- pub `add_annotations` function L911-928 — `( &self, work_order_id: Uuid, annotations: &std::collections::HashMap<String, St...` — Adds multiple annotations to a work order.
- pub `list_annotations` function L939-947 — `( &self, work_order_id: Uuid, ) -> Result<Vec<WorkOrderAnnotation>, diesel::resu...` — Lists all annotations for a work order.
- pub `remove_annotation` function L960-974 — `( &self, work_order_id: Uuid, key: &str, value: &str, ) -> Result<usize, diesel:...` — Removes an annotation from a work order.
-  `is_agent_authorized_for_work_order` function L335-396 — `( &self, conn: &mut diesel::pg::PgConnection, work_order_id: Uuid, agent_id: Uui...` — Checks if an agent is authorized to claim a work order using any targeting mechanism.
-  `emit_completion_event` function L477-494 — `(&self, log: &WorkOrderLog)` — Emits a work order completion event.

### crates/brokkr-broker/src/utils

> *Semantic summary to be generated by AI agent.*

#### crates/brokkr-broker/src/utils/audit.rs

- pub `AuditLoggerConfig` struct L54-61 — `{ channel_size: usize, batch_size: usize, flush_interval_ms: u64 }` — Configuration for the audit logger.
- pub `AuditLogger` struct L75-78 — `{ sender: mpsc::Sender<NewAuditLog> }` — The async audit logger for buffering and batching audit entries.
- pub `new` function L88-90 — `(dal: DAL) -> Self` — Creates a new audit logger and starts the background writer.
- pub `with_config` function L100-112 — `(dal: DAL, config: AuditLoggerConfig) -> Self` — Creates a new audit logger with custom configuration.
- pub `log` function L120-137 — `(&self, entry: NewAuditLog)` — Logs an audit entry asynchronously (non-blocking).
- pub `log_async` function L146-159 — `( &self, entry: NewAuditLog, ) -> Result<(), mpsc::error::SendError<NewAuditLog>...` — Logs an audit entry, waiting for it to be accepted.
- pub `try_log` function L168-180 — `(&self, entry: NewAuditLog) -> bool` — Tries to log an audit entry without blocking.
- pub `init_audit_logger` function L192-194 — `(dal: DAL) -> Result<(), String>` — Initializes the global audit logger.
- pub `init_audit_logger_with_config` function L204-209 — `(dal: DAL, config: AuditLoggerConfig) -> Result<(), String>` — Initializes the global audit logger with custom configuration.
- pub `get_audit_logger` function L215-217 — `() -> Option<Arc<AuditLogger>>` — Gets the global audit logger.
- pub `log` function L225-237 — `(entry: NewAuditLog)` — Logs an audit entry to the global audit logger.
- pub `try_log` function L246-257 — `(entry: NewAuditLog) -> bool` — Tries to log an audit entry without blocking.
- pub `log_action` function L356-383 — `( actor_type: &str, actor_id: Option<uuid::Uuid>, action: &str, resource_type: &...` — Helper to create and log an audit entry in one call.
-  `DEFAULT_CHANNEL_SIZE` variable L41 — `: usize` — Default channel buffer size for audit entries.
-  `DEFAULT_BATCH_SIZE` variable L44 — `: usize` — Default batch size for writing to database.
-  `DEFAULT_FLUSH_INTERVAL_MS` variable L47 — `: u64` — Default flush interval in milliseconds.
-  `AUDIT_LOGGER` variable L50 — `: OnceCell<Arc<AuditLogger>>` — Global audit logger storage.
-  `AuditLoggerConfig` type L63-71 — `impl Default for AuditLoggerConfig` — ```
-  `default` function L64-70 — `() -> Self` — ```
-  `AuditLogger` type L80-181 — `= AuditLogger` — ```
-  `start_audit_writer` function L263-306 — `( dal: DAL, mut receiver: mpsc::Receiver<NewAuditLog>, batch_size: usize, flush_...` — Starts the background audit writer task.
-  `flush_buffer` function L309-338 — `(dal: &DAL, buffer: &mut Vec<NewAuditLog>)` — Flushes the buffer to the database.
-  `tests` module L386-439 — `-` — ```
-  `test_audit_logger_config_default` function L394-399 — `()` — ```
-  `test_log_without_logger_does_not_panic` function L402-415 — `()` — ```
-  `test_try_log_without_logger` function L418-431 — `()` — ```
-  `test_get_audit_logger_uninitialized` function L434-438 — `()` — ```

#### crates/brokkr-broker/src/utils/background_tasks.rs

- pub `DiagnosticCleanupConfig` struct L21-26 — `{ interval_seconds: u64, max_age_hours: i64 }` — Configuration for diagnostic cleanup task.
- pub `start_diagnostic_cleanup_task` function L46-89 — `(dal: DAL, config: DiagnosticCleanupConfig)` — Starts the diagnostic cleanup background task.
- pub `WorkOrderMaintenanceConfig` struct L92-95 — `{ interval_seconds: u64 }` — Configuration for work order maintenance task.
- pub `start_work_order_maintenance_task` function L114-151 — `(dal: DAL, config: WorkOrderMaintenanceConfig)` — Starts the work order maintenance background task.
- pub `WebhookDeliveryConfig` struct L154-159 — `{ interval_seconds: u64, batch_size: i64 }` — Configuration for webhook delivery worker.
- pub `WebhookCleanupConfig` struct L171-176 — `{ interval_seconds: u64, retention_days: i64 }` — Configuration for webhook cleanup task.
- pub `start_webhook_delivery_task` function L199-391 — `(dal: DAL, config: WebhookDeliveryConfig)` — Starts the webhook delivery worker background task.
- pub `start_webhook_cleanup_task` function L435-462 — `(dal: DAL, config: WebhookCleanupConfig)` — Starts the webhook cleanup background task.
- pub `AuditLogCleanupConfig` struct L465-470 — `{ interval_seconds: u64, retention_days: i64 }` — Configuration for audit log cleanup task.
- pub `start_audit_log_cleanup_task` function L489-516 — `(dal: DAL, config: AuditLogCleanupConfig)` — Starts the audit log cleanup background task.
-  `DiagnosticCleanupConfig` type L28-35 — `impl Default for DiagnosticCleanupConfig` — system health and cleanup expired data.
-  `default` function L29-34 — `() -> Self` — system health and cleanup expired data.
-  `WorkOrderMaintenanceConfig` type L97-103 — `impl Default for WorkOrderMaintenanceConfig` — system health and cleanup expired data.
-  `default` function L98-102 — `() -> Self` — system health and cleanup expired data.
-  `WebhookDeliveryConfig` type L161-168 — `impl Default for WebhookDeliveryConfig` — system health and cleanup expired data.
-  `default` function L162-167 — `() -> Self` — system health and cleanup expired data.
-  `WebhookCleanupConfig` type L178-185 — `impl Default for WebhookCleanupConfig` — system health and cleanup expired data.
-  `default` function L179-184 — `() -> Self` — system health and cleanup expired data.
-  `attempt_delivery` function L394-425 — `( client: &reqwest::Client, url: &str, auth_header: Option<&str>, payload: &str,...` — Attempts to deliver a webhook payload via HTTP POST.
-  `AuditLogCleanupConfig` type L472-479 — `impl Default for AuditLogCleanupConfig` — system health and cleanup expired data.
-  `default` function L473-478 — `() -> Self` — system health and cleanup expired data.
-  `tests` module L519-617 — `-` — system health and cleanup expired data.
-  `test_default_diagnostic_config` function L523-527 — `()` — system health and cleanup expired data.
-  `test_custom_diagnostic_config` function L530-537 — `()` — system health and cleanup expired data.
-  `test_default_work_order_config` function L540-543 — `()` — system health and cleanup expired data.
-  `test_custom_work_order_config` function L546-551 — `()` — system health and cleanup expired data.
-  `test_default_webhook_delivery_config` function L554-558 — `()` — system health and cleanup expired data.
-  `test_custom_webhook_delivery_config` function L561-568 — `()` — system health and cleanup expired data.
-  `test_default_webhook_cleanup_config` function L571-575 — `()` — system health and cleanup expired data.
-  `test_custom_webhook_cleanup_config` function L578-585 — `()` — system health and cleanup expired data.
-  `test_attempt_delivery_invalid_url` function L588-601 — `()` — system health and cleanup expired data.
-  `test_attempt_delivery_with_auth_header_invalid_url` function L604-616 — `()` — system health and cleanup expired data.

#### crates/brokkr-broker/src/utils/config_watcher.rs

- pub `ConfigWatcherConfig` struct L21-28 — `{ config_file_path: String, debounce_duration: Duration, enabled: bool }` — Configuration for the file watcher.
- pub `from_environment` function L45-85 — `() -> Option<Self>` — Creates a new ConfigWatcherConfig from environment variables.
- pub `start_config_watcher` function L101-123 — `( config: ReloadableConfig, watcher_config: ConfigWatcherConfig, ) -> Option<tok...` — Starts the configuration file watcher as a background task.
-  `ConfigWatcherConfig` type L30-38 — `impl Default for ConfigWatcherConfig` — file and trigger configuration reloads automatically.
-  `default` function L31-37 — `() -> Self` — file and trigger configuration reloads automatically.
-  `ConfigWatcherConfig` type L40-86 — `= ConfigWatcherConfig` — file and trigger configuration reloads automatically.
-  `run_config_watcher` function L126-220 — `( config: ReloadableConfig, watcher_config: ConfigWatcherConfig, ) -> Result<(),...` — Internal function that runs the configuration file watcher loop.
-  `tests` module L223-255 — `-` — file and trigger configuration reloads automatically.
-  `test_config_watcher_config_default` function L227-232 — `()` — file and trigger configuration reloads automatically.
-  `test_config_from_environment_no_file` function L235-240 — `()` — file and trigger configuration reloads automatically.
-  `test_config_from_environment_disabled` function L243-254 — `()` — file and trigger configuration reloads automatically.

#### crates/brokkr-broker/src/utils/encryption.rs

- pub `EncryptionError` enum L47-56 — `EncryptionFailed | DecryptionFailed | InvalidData | UnsupportedVersion` — Encryption error types
- pub `EncryptionKey` struct L74-79 — `{ key: [u8; 32], cipher: Aes256Gcm }` — Encryption key wrapper with AES-256-GCM cipher.
- pub `new` function L91-94 — `(key: [u8; 32]) -> Self` — Creates a new encryption key from raw bytes.
- pub `generate` function L97-101 — `() -> Self` — Creates a new random encryption key.
- pub `from_hex` function L104-114 — `(hex: &str) -> Result<Self, String>` — Creates a key from a hex-encoded string.
- pub `fingerprint` function L117-120 — `(&self) -> String` — Returns the key as a hex string (for logging key fingerprint only).
- pub `encrypt` function L126-144 — `(&self, plaintext: &[u8]) -> Result<Vec<u8>, EncryptionError>` — Encrypts data using AES-256-GCM.
- pub `decrypt` function L151-172 — `(&self, data: &[u8]) -> Result<Vec<u8>, EncryptionError>` — Decrypts data, automatically detecting the encryption version.
- pub `init_encryption_key` function L232-252 — `(key_hex: Option<&str>) -> Result<(), String>` — Initializes the global encryption key from configuration.
- pub `get_encryption_key` function L258-263 — `() -> Arc<EncryptionKey>` — Gets the global encryption key.
- pub `encrypt_string` function L272-274 — `(value: &str) -> Result<Vec<u8>, EncryptionError>` — Encrypts a string value for storage.
- pub `decrypt_string` function L283-288 — `(encrypted: &[u8]) -> Result<String, String>` — Decrypts bytes back to a string.
-  `VERSION_AES_GCM` variable L31 — `: u8` — Version byte for AES-256-GCM encrypted data
-  `VERSION_LEGACY_XOR` variable L34 — `: u8` — Version byte for legacy XOR encrypted data (read-only)
-  `AES_GCM_NONCE_SIZE` variable L37 — `: usize` — Nonce size for AES-256-GCM (96 bits)
-  `LEGACY_XOR_NONCE_SIZE` variable L40 — `: usize` — Legacy XOR nonce size (128 bits)
-  `ENCRYPTION_KEY` variable L43 — `: OnceCell<Arc<EncryptionKey>>` — Global encryption key storage.
-  `EncryptionError` type L58-69 — `= EncryptionError` — - 0x01: AES-256-GCM encryption
-  `fmt` function L59-68 — `(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result` — - 0x01: AES-256-GCM encryption
-  `EncryptionError` type L71 — `= EncryptionError` — - 0x01: AES-256-GCM encryption
-  `EncryptionKey` type L81-87 — `= EncryptionKey` — - 0x01: AES-256-GCM encryption
-  `fmt` function L82-86 — `(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result` — - 0x01: AES-256-GCM encryption
-  `EncryptionKey` type L89-221 — `= EncryptionKey` — - 0x01: AES-256-GCM encryption
-  `decrypt_aes_gcm` function L175-188 — `(&self, data: &[u8]) -> Result<Vec<u8>, EncryptionError>` — Decrypts AES-256-GCM encrypted data.
-  `decrypt_legacy_xor` function L195-220 — `(&self, data: &[u8]) -> Result<Vec<u8>, EncryptionError>` — Decrypts legacy XOR-encrypted data (for migration support).
-  `tests` module L291-435 — `-` — - 0x01: AES-256-GCM encryption
-  `test_encryption_key_from_hex` function L295-300 — `()` — - 0x01: AES-256-GCM encryption
-  `test_encryption_key_from_hex_invalid` function L303-309 — `()` — - 0x01: AES-256-GCM encryption
-  `test_encrypt_decrypt_roundtrip` function L312-320 — `()` — - 0x01: AES-256-GCM encryption
-  `test_encrypt_decrypt_empty` function L323-331 — `()` — - 0x01: AES-256-GCM encryption
-  `test_encrypt_produces_different_output` function L334-347 — `()` — - 0x01: AES-256-GCM encryption
-  `test_decrypt_wrong_key` function L350-359 — `()` — - 0x01: AES-256-GCM encryption
-  `test_decrypt_tampered_data` function L362-375 — `()` — - 0x01: AES-256-GCM encryption
-  `test_decrypt_too_short` function L378-383 — `()` — - 0x01: AES-256-GCM encryption
-  `test_fingerprint` function L386-393 — `()` — - 0x01: AES-256-GCM encryption
-  `test_version_byte_present` function L396-404 — `()` — - 0x01: AES-256-GCM encryption
-  `test_legacy_xor_decryption` function L407-434 — `()` — - 0x01: AES-256-GCM encryption

#### crates/brokkr-broker/src/utils/event_bus.rs

- pub `emit_event` function L30-99 — `(dal: &DAL, event: &BrokkrEvent) -> usize` — Emits an event by creating webhook deliveries for all matching subscriptions.
-  `tests` module L102-123 — `-` — matching subscriptions.
-  `test_brokkr_event_creation` function L108-114 — `()` — matching subscriptions.
-  `test_brokkr_event_unique_ids` function L117-122 — `()` — matching subscriptions.

#### crates/brokkr-broker/src/utils/matching.rs

- pub `MatchResult` struct L16-23 — `{ matches: bool, missing_labels: Vec<String>, missing_annotations: Vec<(String, ...` — Result of a template-to-stack matching operation.
- pub `template_matches_stack` function L44-78 — `( template_labels: &[String], template_annotations: &[(String, String)], stack_l...` — Check if a template can be instantiated into a stack.
-  `tests` module L81-269 — `-` — annotations are compatible with a target stack before instantiation.
-  `test_template_no_labels_matches_any_stack` function L85-96 — `()` — annotations are compatible with a target stack before instantiation.
-  `test_template_no_labels_matches_empty_stack` function L99-103 — `()` — annotations are compatible with a target stack before instantiation.
-  `test_template_labels_subset_of_stack_matches` function L106-116 — `()` — annotations are compatible with a target stack before instantiation.
-  `test_template_labels_exact_match` function L119-128 — `()` — annotations are compatible with a target stack before instantiation.
-  `test_template_label_not_on_stack` function L131-141 — `()` — annotations are compatible with a target stack before instantiation.
-  `test_template_multiple_missing_labels` function L144-161 — `()` — annotations are compatible with a target stack before instantiation.
-  `test_annotation_exact_match` function L164-173 — `()` — annotations are compatible with a target stack before instantiation.
-  `test_annotation_key_matches_value_differs` function L176-189 — `()` — annotations are compatible with a target stack before instantiation.
-  `test_annotation_missing_entirely` function L192-205 — `()` — annotations are compatible with a target stack before instantiation.
-  `test_mixed_labels_and_annotations_all_match` function L208-220 — `()` — annotations are compatible with a target stack before instantiation.
-  `test_mixed_labels_match_but_annotations_dont` function L223-237 — `()` — annotations are compatible with a target stack before instantiation.
-  `test_annotations_match_but_labels_dont` function L240-251 — `()` — annotations are compatible with a target stack before instantiation.
-  `test_both_labels_and_annotations_missing` function L254-268 — `()` — annotations are compatible with a target stack before instantiation.

#### crates/brokkr-broker/src/utils/mod.rs

- pub `audit` module L20 — `-` — the broker, including admin key management and shutdown procedures.
- pub `background_tasks` module L21 — `-` — the broker, including admin key management and shutdown procedures.
- pub `config_watcher` module L22 — `-` — the broker, including admin key management and shutdown procedures.
- pub `encryption` module L23 — `-` — the broker, including admin key management and shutdown procedures.
- pub `event_bus` module L24 — `-` — the broker, including admin key management and shutdown procedures.
- pub `matching` module L25 — `-` — the broker, including admin key management and shutdown procedures.
- pub `pak` module L26 — `-` — the broker, including admin key management and shutdown procedures.
- pub `templating` module L27 — `-` — the broker, including admin key management and shutdown procedures.
- pub `shutdown` function L33-37 — `(shutdown_rx: oneshot::Receiver<()>)` — Handles the shutdown process for the broker.
- pub `AdminKey` struct L42-47 — `{ id: Uuid, created_at: chrono::DateTime<Utc>, updated_at: chrono::DateTime<Utc>...` — Represents an admin key in the database.
- pub `NewAdminKey` struct L52-54 — `{ pak_hash: String }` — Represents a new admin key to be inserted into the database.
- pub `first_startup` function L60-65 — `( conn: &mut PgConnection, config: &Settings, ) -> Result<(), Box<dyn std::error...` — Performs first-time startup operations.
- pub `upsert_admin` function L85-161 — `( conn: &mut PgConnection, config: &Settings, ) -> Result<(), Box<dyn std::error...` — Updates or inserts the admin key and related generator.
-  `create_pak` function L70-78 — `() -> Result<(String, String), Box<dyn std::error::Error>>` — Creates a new PAK (Privileged Access Key) and its hash.
-  `validate_pak_hash` function L163-167 — `(hash: &str) -> bool` — the broker, including admin key management and shutdown procedures.

#### crates/brokkr-broker/src/utils/pak.rs

- pub `create_pak_controller` function L34-48 — `( config: Option<&Settings>, ) -> Result<Arc<PrefixedApiKeyController<OsRng, Sha...` — Creates or retrieves the PAK controller.
- pub `create_pak` function L79-87 — `() -> Result<(String, String), Box<dyn std::error::Error>>` — Generates a new Prefixed API Key and its hash.
- pub `PakError` enum L94-99 — `Parse | Controller` — Errors returned by the PAK verification helpers.
- pub `verify_pak` function L106-116 — `(pak: String, stored_hash: String) -> Result<bool, PakError>` — Verifies a Prefixed API Key against a stored hash.
- pub `generate_pak_hash` function L122-126 — `(pak: String) -> Result<String, PakError>` — Generates a hash for a given Prefixed API Key.
-  `PAK_CONTROLLER` variable L23 — `: OnceCell<Arc<PrefixedApiKeyController<OsRng, Sha256>>>` — Singleton instance of the PAK controller.
-  `create_pak_controller_inner` function L59-72 — `( config: &Settings, ) -> Result<PrefixedApiKeyController<OsRng, Sha256>, Box<dy...` — Internal function to create a new PAK controller.
-  `tests` module L129-306 — `-` — Prefixed API Keys using a singleton controller pattern.
-  `test_pak_controller_singleton` function L134-183 — `()` — Prefixed API Keys using a singleton controller pattern.
-  `test_verify_pak` function L186-242 — `()` — Prefixed API Keys using a singleton controller pattern.
-  `test_generate_pak_hash` function L245-305 — `()` — Prefixed API Keys using a singleton controller pattern.

#### crates/brokkr-broker/src/utils/templating.rs

- pub `TemplateError` struct L21-24 — `{ message: String, details: Option<String> }` — Error type for templating operations.
- pub `validate_tera_syntax` function L62-73 — `(template_content: &str) -> Result<(), TemplateError>` — Validate Tera template syntax without rendering.
- pub `render_template` function L101-127 — `( template_content: &str, parameters: &Value, ) -> Result<String, TemplateError>` — Render a Tera template with the provided parameters.
- pub `validate_json_schema` function L153-165 — `(schema_str: &str) -> Result<(), TemplateError>` — Validate that a string is a valid JSON Schema.
- pub `ParameterValidationError` struct L169-172 — `{ path: String, message: String }` — Validation error details for parameter validation.
- pub `validate_parameters` function L214-249 — `( schema_str: &str, parameters: &Value, ) -> Result<(), Vec<ParameterValidationE...` — Validate parameters against a JSON Schema.
-  `TemplateError` type L26-33 — `= TemplateError` — - Validating parameters against JSON Schema at instantiation time
-  `fmt` function L27-32 — `(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result` — - Validating parameters against JSON Schema at instantiation time
-  `TemplateError` type L35 — `= TemplateError` — - Validating parameters against JSON Schema at instantiation time
-  `ParameterValidationError` type L174-182 — `= ParameterValidationError` — - Validating parameters against JSON Schema at instantiation time
-  `fmt` function L175-181 — `(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result` — - Validating parameters against JSON Schema at instantiation time
-  `tests` module L252-510 — `-` — - Validating parameters against JSON Schema at instantiation time
-  `test_valid_tera_syntax` function L259-262 — `()` — - Validating parameters against JSON Schema at instantiation time
-  `test_valid_tera_syntax_with_filters` function L265-268 — `()` — - Validating parameters against JSON Schema at instantiation time
-  `test_valid_tera_syntax_with_conditionals` function L271-280 — `()` — - Validating parameters against JSON Schema at instantiation time
-  `test_valid_tera_syntax_with_loops` function L283-290 — `()` — - Validating parameters against JSON Schema at instantiation time
-  `test_invalid_tera_syntax_unclosed_brace` function L293-299 — `()` — - Validating parameters against JSON Schema at instantiation time
-  `test_invalid_tera_syntax_unclosed_block` function L302-306 — `()` — - Validating parameters against JSON Schema at instantiation time
-  `test_valid_tera_syntax_plain_text` function L309-312 — `()` — - Validating parameters against JSON Schema at instantiation time
-  `test_valid_tera_syntax_default_filter` function L315-318 — `()` — - Validating parameters against JSON Schema at instantiation time
-  `test_render_template_simple` function L323-328 — `()` — - Validating parameters against JSON Schema at instantiation time
-  `test_render_template_multiple_vars` function L331-337 — `()` — - Validating parameters against JSON Schema at instantiation time
-  `test_render_template_with_default` function L340-345 — `()` — - Validating parameters against JSON Schema at instantiation time
-  `test_render_template_missing_required_var` function L348-355 — `()` — - Validating parameters against JSON Schema at instantiation time
-  `test_render_template_with_filter` function L358-363 — `()` — - Validating parameters against JSON Schema at instantiation time
-  `test_render_template_nested_object` function L366-371 — `()` — - Validating parameters against JSON Schema at instantiation time
-  `test_valid_json_schema_simple` function L376-379 — `()` — - Validating parameters against JSON Schema at instantiation time
-  `test_valid_json_schema_with_properties` function L382-391 — `()` — - Validating parameters against JSON Schema at instantiation time
-  `test_valid_json_schema_with_required` function L394-403 — `()` — - Validating parameters against JSON Schema at instantiation time
-  `test_invalid_json_not_json` function L406-412 — `()` — - Validating parameters against JSON Schema at instantiation time
-  `test_empty_json_schema_valid` function L415-419 — `()` — - Validating parameters against JSON Schema at instantiation time
-  `test_validate_parameters_valid` function L424-428 — `()` — - Validating parameters against JSON Schema at instantiation time
-  `test_validate_parameters_missing_required` function L431-438 — `()` — - Validating parameters against JSON Schema at instantiation time
-  `test_validate_parameters_wrong_type` function L441-446 — `()` — - Validating parameters against JSON Schema at instantiation time
-  `test_validate_parameters_pattern` function L449-459 — `()` — - Validating parameters against JSON Schema at instantiation time
-  `test_validate_parameters_minimum` function L462-473 — `()` — - Validating parameters against JSON Schema at instantiation time
-  `test_validate_parameters_empty_schema` function L476-481 — `()` — - Validating parameters against JSON Schema at instantiation time
-  `test_validate_parameters_complex_schema` function L484-509 — `()` — - Validating parameters against JSON Schema at instantiation time

### crates/brokkr-broker/src/ws

> *Semantic summary to be generated by AI agent.*

#### crates/brokkr-broker/src/ws/broadcaster.rs

- pub `LiveBroadcaster` struct L35-37 — `{ channels: RwLock<HashMap<Uuid, broadcast::Sender<WsMessage>>> }` — This is per ADR-0008: "a slow subscriber must not slow ingestion".
- pub `new` function L40-42 — `() -> Arc<Self>` — This is per ADR-0008: "a slow subscriber must not slow ingestion".
- pub `broadcast` function L47-52 — `(&self, stack_id: Uuid, msg: WsMessage)` — Send a frame to every subscriber of `stack_id`.
- pub `subscribe` function L56-62 — `(&self, stack_id: Uuid) -> broadcast::Receiver<WsMessage>` — Subscribe to all future frames for `stack_id`.
- pub `stack_count` function L65-67 — `(&self) -> usize` — Diagnostics: number of stacks with at least one live subscriber.
- pub `subscriber_count` function L70-77 — `(&self) -> usize` — Diagnostics: total subscriber count across all stacks.
-  `CHANNEL_CAPACITY` variable L32 — `: usize` — Per-stack broadcast capacity.
-  `LiveBroadcaster` type L39-78 — `= LiveBroadcaster` — This is per ADR-0008: "a slow subscriber must not slow ingestion".
-  `tests` module L81-156 — `-` — This is per ADR-0008: "a slow subscriber must not slow ingestion".
-  `evt` function L86-103 — `(stack_id: Uuid) -> WsMessage` — This is per ADR-0008: "a slow subscriber must not slow ingestion".
-  `broadcast_with_no_subscribers_is_a_noop` function L106-109 — `()` — This is per ADR-0008: "a slow subscriber must not slow ingestion".
-  `subscriber_receives_only_their_stack` function L112-126 — `()` — This is per ADR-0008: "a slow subscriber must not slow ingestion".
-  `diagnostic_counters_track_subscriptions` function L129-137 — `()` — This is per ADR-0008: "a slow subscriber must not slow ingestion".
-  `broadcaster_does_not_filter_by_message_type` function L143-155 — `()` — This is per ADR-0008: "a slow subscriber must not slow ingestion".

#### crates/brokkr-broker/src/ws/eviction.rs

- pub `HARD_RETENTION_CEILING` variable L33 — `: Duration` — Hard cap on retained telemetry — never configurable upward.
- pub `DEFAULT_EVICTION_TICK` variable L36 — `: Duration` — Default eviction tick interval.
- pub `RetentionConfig` struct L40-45 — `{ retention: Duration, tick_interval: Duration }` — Retention policy for the agent telemetry buffers.
- pub `new` function L51-61 — `(retention: Duration, tick_interval: Duration) -> Self` — Construct a policy, clamping `retention` to the hard ceiling.
- pub `default_policy` function L64-66 — `() -> Self` — Default policy: 6h retention, 60s tick.
- pub `spawn` function L78-88 — `(dal: DAL, config: RetentionConfig) -> JoinHandle<()>` — Spawn the continuous eviction worker.
- pub `run_once` function L92-113 — `(dal: &DAL, config: RetentionConfig)` — Synchronous single eviction pass — exposed for tests so they can call
-  `RetentionConfig` type L47-67 — `= RetentionConfig` — ceiling.
-  `RetentionConfig` type L69-73 — `impl Default for RetentionConfig` — ceiling.
-  `default` function L70-72 — `() -> Self` — ceiling.
-  `tests` module L116-137 — `-` — ceiling.
-  `retention_above_ceiling_is_clamped` function L120-123 — `()` — ceiling.
-  `retention_below_ceiling_is_preserved` function L126-129 — `()` — ceiling.
-  `default_policy_uses_ceiling_and_one_minute_tick` function L132-136 — `()` — ceiling.

#### crates/brokkr-broker/src/ws/handler.rs

- pub `INTERNAL_WS_PATH` variable L57 — `: &str` — Public path of the internal WS endpoint.
- pub `internal_routes` function L76-89 — `( dal: DAL, registry: Arc<ConnectionRegistry>, broadcaster: Arc<LiveBroadcaster>...` — Build the standalone router that mounts the internal WS endpoint.
-  `CONTROL_LANE_CAPACITY` variable L63 — `: usize` — Capacity of the per-connection control lane.
-  `TELEMETRY_LANE_CAPACITY` variable L68 — `: usize` — Capacity of the per-connection telemetry lane.
-  `ws_upgrade` function L91-120 — `( upgrade: WebSocketUpgrade, State(dal): State<DAL>, Extension(registry): Extens...` — entry is removed from the registry cleanly.
-  `run_connection` function L122-173 — `( socket: WebSocket, agent_id: uuid::Uuid, registry: Arc<ConnectionRegistry>, br...` — entry is removed from the registry cleanly.
-  `reader_task` function L175-206 — `( mut receiver: futures::stream::SplitStream<WebSocket>, agent_id: uuid::Uuid, m...` — entry is removed from the registry cleanly.
-  `dispatch_uplink` function L213-323 — `(msg: WsMessage, agent_id: uuid::Uuid, dal: &DAL, broadcaster: &LiveBroadcaster)` — Dispatch an inbound WS message into the same DAL operations the REST
-  `ws_variant_name` function L328-340 — `(msg: &WsMessage) -> &'static str` — Snake_case tag matching the wire enum's serde rename.
-  `writer_task` function L342-381 — `( mut sender: futures::stream::SplitSink<WebSocket, Message>, mut control_rx: mp...` — entry is removed from the registry cleanly.

#### crates/brokkr-broker/src/ws/mod.rs

- pub `broadcaster` module L18 — `-` — Internal broker↔agent WebSocket channel.
- pub `eviction` module L19 — `-` — [[BROKKR-I-0019]] in `.metis/`.
- pub `handler` module L20 — `-` — [[BROKKR-I-0019]] in `.metis/`.
- pub `push` module L21 — `-` — [[BROKKR-I-0019]] in `.metis/`.
- pub `registry` module L22 — `-` — [[BROKKR-I-0019]] in `.metis/`.
- pub `subscribe` module L23 — `-` — [[BROKKR-I-0019]] in `.metis/`.

#### crates/brokkr-broker/src/ws/push.rs

- pub `push_work_order` function L35-48 — `( registry: &Arc<ConnectionRegistry>, work_order: &WorkOrder, agent_ids: &[Uuid]...` — Push a freshly-created [`WorkOrder`] to each targeted agent.
- pub `push_target_changed` function L55-62 — `(registry: &Arc<ConnectionRegistry>, target: &AgentTarget)` — Push a [`AgentTarget`] change to the affected agent.
- pub `push_stack_changed_to_targets` function L72-92 — `(registry: &Arc<ConnectionRegistry>, dal: &DAL, stack: &Stack)` — Push a [`Stack`] change to every agent currently targeting it.
-  `deliver` function L94-104 — `(registry: &Arc<ConnectionRegistry>, agent_id: Uuid, msg: WsMessage, kind: &'sta...` — invariant and the post-commit ordering requirement.

#### crates/brokkr-broker/src/ws/registry.rs

- pub `SendError` enum L32-40 — `NotConnected | LaneUnavailable` — Errors returned when trying to push a message to a registered agent.
- pub `ConnectionHandle` struct L61-68 — `{ agent_id: Uuid, connected_since: DateTime<Utc>, messages_in: Arc<AtomicU64>, m...` — Sender-side handle for a single registered connection.
- pub `ConnectionInfo` struct L72-77 — `{ agent_id: Uuid, connected_since: DateTime<Utc>, messages_in: u64, messages_out...` — Snapshot view of one connection for diagnostics endpoints (WS-13).
- pub `ConnectionRegistry` struct L85-87 — `{ inner: RwLock<HashMap<Uuid, ConnectionHandle>> }` — Per-broker-process registry of live agent connections.
- pub `new` function L90-92 — `() -> Arc<Self>` — down cleanly.
- pub `register` function L95-99 — `(&self, handle: ConnectionHandle)` — Insert a new handle, evicting any prior connection for the same agent.
- pub `unregister_if_matches` function L104-111 — `(&self, agent_id: Uuid, connected_since: DateTime<Utc>)` — Remove the handle iff it still matches the writer's `connected_since`
- pub `is_connected` function L114-119 — `(&self, agent_id: Uuid) -> bool` — True if any handle is registered for this agent.
- pub `send_control` function L125-137 — `(&self, agent_id: Uuid, msg: WsMessage) -> Result<(), SendError>` — Send a control-plane message to a specific agent.
- pub `send_telemetry` function L143-155 — `(&self, agent_id: Uuid, msg: WsMessage) -> Result<(), SendError>` — Send a telemetry/log message to a specific agent on the low-priority
- pub `snapshot` function L159-171 — `(&self) -> Vec<ConnectionInfo>` — Snapshot every connection for diagnostics.
- pub `connected_count` function L174-176 — `(&self) -> usize` — Number of connected agents (cheap; no clone).
- pub `close_for_agent` function L189-196 — `(&self, agent_id: Uuid) -> usize` — Forcibly close any live connection for `agent_id`, returning how many
-  `SendError` type L42-51 — `= SendError` — down cleanly.
-  `fmt` function L43-50 — `(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result` — down cleanly.
-  `SendError` type L53 — `= SendError` — down cleanly.
-  `ConnectionRegistry` type L89-197 — `= ConnectionRegistry` — down cleanly.
-  `tests` module L200-332 — `-` — down cleanly.
-  `handle_for` function L204-222 — `( agent_id: Uuid, ) -> ( ConnectionHandle, mpsc::Receiver<WsMessage>, mpsc::Rece...` — down cleanly.
-  `sample_heartbeat` function L224-229 — `(agent_id: Uuid) -> WsMessage` — down cleanly.
-  `send_to_unknown_agent_errors` function L232-237 — `()` — down cleanly.
-  `register_then_send_lands_on_correct_lane` function L240-256 — `()` — down cleanly.
-  `second_register_evicts_first` function L259-278 — `()` — down cleanly.
-  `unregister_if_matches_removes_only_matching_generation` function L281-289 — `()` — down cleanly.
-  `close_for_agent_removes_handle_and_drops_senders` function L292-311 — `()` — down cleanly.
-  `lane_full_returns_lane_unavailable` function L314-331 — `()` — down cleanly.

#### crates/brokkr-broker/src/ws/subscribe.rs

- pub `LIVE_SUBSCRIPTION_PATH_TEMPLATE` variable L47 — `: &str` — Documented path template (Axum colon-style).
- pub `subscribe_routes` function L62-73 — `(dal: DAL, broadcaster: Arc<LiveBroadcaster>) -> Router<DAL>` — Build the live-subscription router.
-  `PAK_SUBPROTOCOL_PREFIX` variable L52 — `: &str` — Subprotocol that carries the PAK for browser clients that cannot set an
-  `WS_MARKER_SUBPROTOCOL` variable L57 — `: &str` — Non-secret marker subprotocol the browser also offers and the broker
-  `ws_subprotocol_auth` function L81-97 — `(mut request: Request<Body>, next: Next) -> Response` — Browser WebSocket clients can't set request headers, so they pass the PAK
-  `live_upgrade` function L99-126 — `( upgrade: WebSocketUpgrade, State(dal): State<DAL>, Extension(broadcaster): Ext...` — (per ADR-0008's "a slow subscriber must not slow ingestion").
-  `authorise` function L128-142 — `(dal: &DAL, auth: &AuthPayload, stack_id: Uuid) -> bool` — (per ADR-0008's "a slow subscriber must not slow ingestion").
-  `run_subscriber` function L144-187 — `(socket: WebSocket, stack_id: Uuid, broadcaster: Arc<LiveBroadcaster>)` — (per ADR-0008's "a slow subscriber must not slow ingestion").
-  `forward` function L189-206 — `( sink: &mut futures::stream::SplitSink<WebSocket, Message>, msg: &WsMessage, ) ...` — (per ADR-0008's "a slow subscriber must not slow ingestion").

### crates/brokkr-broker/tests

> *Semantic summary to be generated by AI agent.*

#### crates/brokkr-broker/tests/fixtures.rs

- pub `MIGRATIONS` variable L43 — `: EmbeddedMigrations` — Embedded migrations for the test database.
- pub `TestFixture` struct L48-54 — `{ dal: DAL, settings: Settings, admin_pak: String, admin_generator: Generator }` — Represents a test fixture for the Brokkr project.
- pub `create_test_router` function L71-85 — `(&self) -> Router<DAL>` — Creates and returns an Axum Router with configured API routes.
- pub `new` function L102-143 — `() -> Self` — Creates a new TestFixture instance.
- pub `create_test_stack` function L158-170 — `( &self, name: String, description: Option<String>, generator_id: Uuid, ) -> Sta...` — Creates a new stack for testing purposes.
- pub `create_test_agent` function L182-188 — `(&self, name: String, cluster_name: String) -> Agent` — Creates a new agent for testing purposes.
- pub `create_test_deployment_object` function L201-214 — `( &self, stack_id: Uuid, yaml_content: String, is_deletion_marker: bool, ) -> De...` — Creates a new deployment object for testing purposes.
- pub `create_test_stack_label` function L226-233 — `(&self, stack_id: Uuid, label: String) -> StackLabel` — Creates a new stack label for testing purposes.
- pub `create_test_stack_annotation` function L246-261 — `( &self, stack_id: Uuid, key: &str, value: &str, ) -> StackAnnotation` — Creates a new stack annotation for testing purposes.
- pub `create_test_agent_annotation` function L274-286 — `( &self, agent_id: Uuid, key: String, value: String, ) -> AgentAnnotation` — Creates a new agent annotation for testing purposes.
- pub `create_test_agent_target` function L298-305 — `(&self, agent_id: Uuid, stack_id: Uuid) -> AgentTarget` — Creates a new agent target for testing purposes.
- pub `create_test_agent_event` function L320-339 — `( &self, agent: &Agent, deployment_object: &DeploymentObject, event_type: &str, ...` — Creates a new agent event for testing purposes.
- pub `create_test_agent_label` function L351-358 — `(&self, agent_id: Uuid, label: String) -> AgentLabel` — Creates a new agent label for testing purposes.
- pub `create_test_generator` function L370-388 — `( &self, name: String, description: Option<String>, api_key_hash: String, ) -> G...` — Creates a new generator for testing purposes.
- pub `create_test_generator_with_pak` function L390-408 — `( &self, name: String, description: Option<String>, ) -> (Generator, String)` — and agent events.
- pub `create_test_agent_with_pak` function L410-427 — `( &self, name: String, cluster_name: String, ) -> (Agent, String)` — and agent events.
- pub `create_test_template` function L442-460 — `( &self, generator_id: Option<Uuid>, name: String, description: Option<String>, ...` — Creates a new stack template for testing purposes.
- pub `create_test_template_label` function L472-479 — `(&self, template_id: Uuid, label: String) -> TemplateLabel` — Creates a new template label for testing purposes.
- pub `create_test_template_annotation` function L492-505 — `( &self, template_id: Uuid, key: &str, value: &str, ) -> TemplateAnnotation` — Creates a new template annotation for testing purposes.
- pub `create_test_work_order` function L517-530 — `(&self, work_type: &str, yaml_content: &str) -> WorkOrder` — Creates a new work order for testing purposes.
- pub `create_test_work_order_target` function L542-553 — `( &self, work_order_id: Uuid, agent_id: Uuid, ) -> WorkOrderTarget` — Creates a new work order target for testing purposes.
- pub `create_test_work_order_label` function L565-572 — `(&self, work_order_id: Uuid, label: &str) -> WorkOrderLabel` — Creates a new work order label for testing purposes.
- pub `create_test_work_order_annotation` function L585-598 — `( &self, work_order_id: Uuid, key: &str, value: &str, ) -> WorkOrderAnnotation` — Creates a new work order annotation for testing purposes.
-  `TestFixture` type L56-60 — `impl Default for TestFixture` — and agent events.
-  `default` function L57-59 — `() -> Self` — and agent events.
-  `TestFixture` type L62-610 — `= TestFixture` — and agent events.
-  `reset_database` function L600-609 — `(&self)` — and agent events.
-  `TestFixture` type L612-616 — `impl Drop for TestFixture` — and agent events.
-  `drop` function L613-615 — `(&mut self)` — and agent events.

### crates/brokkr-broker/tests/integration/api

> *Semantic summary to be generated by AI agent.*

#### crates/brokkr-broker/tests/integration/api/admin.rs

-  `test_config_reload_requires_auth` function L19-37 — `()` — Test that the config reload endpoint requires authentication.
-  `test_config_reload_requires_admin` function L41-64 — `()` — Test that non-admin users cannot access config reload.
-  `test_config_reload_success_with_admin` function L68-98 — `()` — Test that admin users can successfully reload configuration.
-  `test_config_reload_no_changes` function L102-133 — `()` — Test that config reload returns no changes when config hasn't changed.
-  `test_config_reload_denied_for_generator` function L137-162 — `()` — Test that generator PAK cannot access config reload (admin only).

#### crates/brokkr-broker/tests/integration/api/agent_events.rs

-  `test_list_agent_events_success` function L17-52 — `()`
-  `test_list_agent_events_unauthorized_non_existent_pak` function L55-72 — `()`
-  `test_list_agent_events_unauthorized_no_pak` function L75-91 — `()`
-  `test_create_agent_event_unauthorized_non_existent_pak` function L94-120 — `()`
-  `test_create_agent_event_unauthorized_no_pak` function L123-148 — `()`
-  `test_get_agent_event_success` function L151-185 — `()`
-  `test_get_agent_event_unauthorized_non_existent_pak` function L188-205 — `()`
-  `test_get_agent_event_unauthorized_no_pak` function L208-224 — `()`
-  `test_get_agent_event_not_found` function L227-246 — `()`

#### crates/brokkr-broker/tests/integration/api/agents.rs

-  `make_unauthorized_request` function L25-43 — `( app: Router, method: &str, uri: &str, body: Option<String>, ) -> StatusCode`
-  `test_create_agent` function L46-75 — `()`
-  `test_get_agent` function L78-105 — `()`
-  `test_update_agent` function L108-165 — `()`
-  `test_delete_agent` function L168-189 — `()`
-  `test_list_agent_events` function L192-243 — `()`
-  `test_create_agent_event` function L246-290 — `()`
-  `test_list_agent_labels` function L293-329 — `()`
-  `test_add_agent_label` function L332-362 — `()`
-  `test_remove_agent_label` function L365-398 — `()`
-  `test_list_agent_annotations` function L401-442 — `()`
-  `test_add_agent_annotation` function L445-480 — `()`
-  `test_remove_agent_annotation` function L483-520 — `()`
-  `test_list_agent_targets` function L523-568 — `()`
-  `test_add_agent_target` function L571-610 — `()`
-  `test_remove_agent_target` function L613-655 — `()`
-  `test_unauthorized_list_agent_events` function L658-673 — `()`
-  `test_unauthorized_create_agent_event` function L676-700 — `()`
-  `test_unauthorized_list_agent_labels` function L703-718 — `()`
-  `test_unauthorized_add_agent_label` function L721-739 — `()`
-  `test_unauthorized_create_agent` function L742-758 — `()`
-  `test_unauthorized_get_agent` function L761-776 — `()`
-  `test_unauthorized_update_agent` function L779-799 — `()`
-  `test_unauthorized_delete_agent` function L802-817 — `()`
-  `test_get_agent_with_mismatched_pak` function L820-842 — `()`
-  `test_update_agent_with_mismatched_pak` function L845-871 — `()`
-  `test_create_agent_event_with_mismatched_pak` function L874-906 — `()`
-  `test_list_agent_labels_with_mismatched_pak` function L909-931 — `()`
-  `test_record_heartbeat` function L934-959 — `()`
-  `test_get_target_state_incremental` function L962-1017 — `()`
-  `test_get_target_state_full` function L1020-1089 — `()`
-  `test_get_target_state_with_invalid_mode` function L1092-1143 — `()`
-  `test_get_agent_by_name_and_cluster_name` function L1146-1175 — `()`
-  `test_get_agent_stacks` function L1178-1306 — `()`
-  `test_rotate_agent_pak_admin_success` function L1309-1347 — `()`
-  `test_rotate_agent_pak_self_success` function L1350-1383 — `()`
-  `test_rotate_agent_pak_unauthorized` function L1386-1405 — `()`
-  `test_rotate_agent_pak_forbidden` function L1408-1432 — `()`
-  `test_get_target_state_with_mismatched_auth` function L1435-1473 — `()`

#### crates/brokkr-broker/tests/integration/api/audit_logs.rs

-  `test_audit_logs_requires_auth` function L19-36 — `()` — Test that the audit logs endpoint requires authentication.
-  `test_audit_logs_requires_admin` function L40-62 — `()` — Test that non-admin users cannot access audit logs.
-  `test_audit_logs_success_with_admin` function L66-96 — `()` — Test that admin users can access audit logs.
-  `test_audit_logs_pagination` function L100-127 — `()` — Test audit logs with pagination parameters.
-  `test_audit_logs_filtering` function L131-157 — `()` — Test audit logs with filter parameters.
-  `test_audit_logs_denied_for_generator` function L161-185 — `()` — Test that generator PAK cannot access audit logs (admin only).

#### crates/brokkr-broker/tests/integration/api/auth.rs

-  `test_verify_pak_endpoint` function L19-59 — `()`
-  `test_verify_admin_pak_endpoint` function L62-91 — `()`

#### crates/brokkr-broker/tests/integration/api/deployment_objects.rs

-  `test_get_deployment_object_admin_success` function L19-48 — `()`
-  `test_get_deployment_object_agent_success` function L51-88 — `()`
-  `test_get_deployment_object_generator_success` function L91-129 — `()`
-  `test_get_deployment_object_agent_forbidden` function L132-166 — `()`
-  `test_get_deployment_object_generator_forbidden` function L169-208 — `()`
-  `test_get_deployment_object_not_found` function L211-230 — `()`
-  `test_get_deployment_object_unauthorized` function L233-257 — `()`
-  `test_update_stack_with_admin_pak` function L260-298 — `()`
-  `test_update_stack_with_generator_pak` function L301-339 — `()`
-  `test_update_stack_with_bad_pak` function L342-374 — `()`
-  `test_create_deployment_object_with_admin_pak` function L377-418 — `()`
-  `test_create_deployment_object_with_generator_pak` function L421-458 — `()`
-  `test_create_deployment_object_with_bad_pak` function L461-496 — `()`

#### crates/brokkr-broker/tests/integration/api/diagnostics.rs

-  `test_create_diagnostic_request` function L17-69 — `()`
-  `test_create_diagnostic_request_unauthorized` function L72-108 — `()`
-  `test_get_pending_diagnostics` function L111-159 — `()`
-  `test_get_pending_diagnostics_unauthorized` function L162-186 — `()`
-  `test_claim_diagnostic` function L189-237 — `()`
-  `test_claim_already_claimed` function L240-285 — `()`
-  `test_submit_diagnostic_result` function L288-356 — `()`
-  `test_submit_result_not_claimed` function L359-408 — `()`
-  `test_get_diagnostic_with_result` function L411-490 — `()`
-  `test_get_diagnostic_not_found` function L493-512 — `()`

#### crates/brokkr-broker/tests/integration/api/generators.rs

-  `test_list_generators_admin_success` function L16-43 — `()`
-  `test_list_generators_non_admin_forbidden` function L46-65 — `()`
-  `test_create_generator_admin_success` function L68-96 — `()`
-  `test_get_generator_admin_success` function L99-126 — `()`
-  `test_get_generator_self_success` function L129-151 — `()`
-  `test_update_generator_admin_success` function L154-187 — `()`
-  `test_delete_generator_admin_success` function L190-214 — `()`
-  `test_delete_generator_self_success` function L217-236 — `()`
-  `test_list_generators_unauthorized` function L239-255 — `()`
-  `test_create_generator_unauthorized` function L258-275 — `()`
-  `test_get_generator_unauthorized` function L278-299 — `()`
-  `test_update_generator_unauthorized` function L302-324 — `()`
-  `test_delete_generator_unauthorized` function L327-348 — `()`
-  `test_rotate_generator_pak_admin_success` function L351-387 — `()`
-  `test_rotate_generator_pak_self_success` function L390-423 — `()`
-  `test_rotate_generator_pak_unauthorized` function L426-444 — `()`
-  `test_rotate_generator_pak_forbidden` function L447-470 — `()`

#### crates/brokkr-broker/tests/integration/api/health.rs

-  `test_healthz_endpoint` function L16-37 — `()`
-  `test_readyz_endpoint` function L40-61 — `()`
-  `test_metrics_endpoint` function L64-88 — `()`
-  `test_metrics_records_http_requests` function L91-140 — `()`
-  `test_metrics_contains_all_defined_metrics` function L143-181 — `()`

#### crates/brokkr-broker/tests/integration/api/mod.rs

-  `admin` module L7 — `-`
-  `agent_events` module L8 — `-`
-  `agents` module L9 — `-`
-  `audit_logs` module L10 — `-`
-  `auth` module L11 — `-`
-  `deployment_objects` module L12 — `-`
-  `diagnostics` module L13 — `-`
-  `generators` module L14 — `-`
-  `health` module L15 — `-`
-  `stacks` module L16 — `-`
-  `templates` module L17 — `-`
-  `webhooks` module L18 — `-`
-  `work_orders` module L19 — `-`
-  `ws` module L20 — `-`

#### crates/brokkr-broker/tests/integration/api/stacks.rs

-  `test_create_stack` function L23-62 — `()`
-  `test_get_stack` function L65-96 — `()`
-  `test_list_stacks` function L99-130 — `()`
-  `test_list_stacks_with_generator_pak_filters_to_own` function L133-195 — `()`
-  `test_list_stacks_without_pak_forbidden` function L198-223 — `()`
-  `test_update_stack` function L226-264 — `()`
-  `test_soft_delete_stack` function L267-309 — `()`
-  `test_add_stack_annotation` function L312-350 — `()`
-  `test_remove_stack_annotation` function L353-379 — `()`
-  `test_list_stack_annotations` function L382-414 — `()`
-  `test_add_stack_label` function L417-457 — `()`
-  `test_add_stack_label_duplicate_returns_409` function L460-501 — `()`
-  `test_remove_stack_label` function L504-530 — `()`
-  `test_list_stack_labels` function L533-565 — `()`
-  `test_create_deployment_object` function L568-606 — `()`
-  `test_create_stack_with_generator_pak` function L609-647 — `()`
-  `test_create_stack_with_wrong_generator_pak` function L650-693 — `()`
-  `test_update_stack_with_wrong_generator_pak` function L696-740 — `()`
-  `test_delete_stack_with_wrong_generator_pak` function L743-780 — `()`
-  `test_add_stack_annotation_with_wrong_generator_pak` function L783-827 — `()`
-  `test_create_deployment_object_yaml_body` function L832-861 — `()`
-  `test_create_deployment_object_yaml_deletion_marker_empty` function L864-897 — `()`
-  `test_create_deployment_object_malformed_yaml_rejected` function L900-922 — `()`
-  `test_get_deployment_object_accept_yaml_roundtrip` function L925-978 — `()`
-  `test_create_deployment_object_json_still_works` function L981-1006 — `()`

#### crates/brokkr-broker/tests/integration/api/templates.rs

-  `TEST_TEMPLATE_CONTENT` variable L16-21 — `: &str`
-  `TEST_PARAMETERS_SCHEMA` variable L23-30 — `: &str`
-  `test_create_template` function L33-69 — `()`
-  `test_create_template_with_generator_pak` function L72-106 — `()`
-  `test_create_template_invalid_tera_syntax` function L109-134 — `()`
-  `test_get_template` function L137-169 — `()`
-  `test_list_templates` function L172-210 — `()`
-  `test_update_template_creates_new_version` function L213-255 — `()`
-  `test_delete_template` function L258-300 — `()`
-  `test_add_template_label` function L303-337 — `()`
-  `test_list_template_labels` function L340-374 — `()`
-  `test_remove_template_label` function L377-408 — `()`
-  `test_add_template_annotation` function L411-449 — `()`
-  `test_list_template_annotations` function L452-486 — `()`
-  `test_remove_template_annotation` function L489-520 — `()`
-  `test_instantiate_template` function L523-575 — `()`
-  `test_instantiate_template_invalid_parameters` function L578-628 — `()`
-  `test_instantiate_template_label_mismatch` function L631-674 — `()`
-  `test_instantiate_template_with_matching_labels` function L677-719 — `()`
-  `test_generator_cannot_access_other_generator_template` function L722-753 — `()`

#### crates/brokkr-broker/tests/integration/api/webhooks.rs

-  `test_list_webhooks_admin_success` function L20-41 — `()`
-  `test_list_webhooks_non_admin_forbidden` function L44-63 — `()`
-  `test_list_webhooks_unauthorized` function L66-82 — `()`
-  `test_create_webhook_admin_success` function L89-123 — `()`
-  `test_create_webhook_with_wildcard_events` function L126-151 — `()`
-  `test_create_webhook_invalid_url` function L154-179 — `()`
-  `test_create_webhook_non_admin_forbidden` function L182-208 — `()`
-  `test_get_webhook_admin_success` function L215-257 — `()`
-  `test_get_webhook_not_found` function L260-279 — `()`
-  `test_update_webhook_admin_success` function L286-335 — `()`
-  `test_delete_webhook_admin_success` function L342-389 — `()`
-  `test_delete_webhook_not_found` function L392-411 — `()`
-  `test_list_event_types_admin_success` function L418-444 — `()`
-  `test_list_deliveries_admin_success` function L451-493 — `()`
-  `test_list_deliveries_with_status_filter` function L496-554 — `()`
-  `test_list_deliveries_subscription_not_found` function L557-576 — `()`

#### crates/brokkr-broker/tests/integration/api/work_orders.rs

-  `make_request` function L22-51 — `( app: Router, method: &str, uri: &str, auth: Option<&str>, body: Option<String>...`
-  `test_create_work_order` function L58-85 — `()`
-  `test_create_work_order_empty_targets` function L88-110 — `()`
-  `test_create_work_order_unauthorized` function L113-135 — `()`
-  `test_create_work_order_forbidden_non_admin` function L138-161 — `()`
-  `test_list_work_orders` function L164-180 — `()`
-  `test_list_work_orders_filtered` function L183-205 — `()`
-  `test_get_work_order` function L208-228 — `()`
-  `test_get_work_order_not_found` function L231-246 — `()`
-  `test_delete_work_order` function L249-270 — `()`
-  `test_list_pending_for_agent` function L277-304 — `()`
-  `test_list_pending_for_agent_admin` function L307-327 — `()`
-  `test_list_pending_for_other_agent_forbidden` function L330-348 — `()`
-  `test_claim_work_order` function L351-379 — `()`
-  `test_claim_work_order_not_targeted` function L382-406 — `()`
-  `test_complete_work_order_success` function L409-445 — `()`
-  `test_complete_work_order_failure_with_retry` function L448-498 — `()`
-  `test_complete_work_order_failure_max_retries` function L501-551 — `()`
-  `test_complete_work_order_wrong_agent` function L554-588 — `()`
-  `test_list_work_order_log` function L595-620 — `()`
-  `test_get_work_order_log` function L623-652 — `()`
-  `test_get_work_order_log_not_found` function L655-670 — `()`
-  `test_list_work_order_log_forbidden` function L673-684 — `()`
-  `test_create_work_order_with_labels` function L691-729 — `()`
-  `test_create_work_order_with_annotations` function L732-770 — `()`
-  `test_create_work_order_with_combined_targeting` function L773-825 — `()`
-  `test_create_work_order_no_targeting_fails` function L828-853 — `()`
-  `test_create_work_order_empty_targeting_fails` function L856-882 — `()`
-  `test_create_work_order_legacy_target_agent_ids` function L885-910 — `()`
-  `test_list_pending_with_label_targeting` function L913-941 — `()`
-  `test_list_pending_with_annotation_targeting` function L944-972 — `()`
-  `test_claim_with_label_targeting` function L975-1007 — `()`
-  `test_claim_with_annotation_targeting` function L1010-1042 — `()`
-  `test_claim_with_no_matching_targeting` function L1045-1074 — `()`

#### crates/brokkr-broker/tests/integration/api/ws.rs

-  `spawn_broker` function L43-74 — `(fixture: &TestFixture) -> (std::net::SocketAddr, Arc<ConnectionRegistry>)` — Bind the broker on a random local port and return the bound address plus
-  `ws_url` function L76-78 — `(addr: std::net::SocketAddr) -> String` — path; this is why we bind a TCP listener for the upgrade tests.
-  `ws_upgrade_rejects_unauthenticated` function L81-105 — `()` — path; this is why we bind a TCP listener for the upgrade tests.
-  `ws_endpoint_is_not_in_openapi_spec` function L108-134 — `()` — path; this is why we bind a TCP listener for the upgrade tests.
-  `ws_request_with_pak` function L139-149 — `( url: &str, pak_value: &str, ) -> tokio_tungstenite::tungstenite::handshake::cl...` — Build a tokio-tungstenite client request with `Authorization: Bearer <pak>`.
-  `ws_upgrade_rejects_admin_pak` function L152-167 — `()` — path; this is why we bind a TCP listener for the upgrade tests.
-  `ws_upgrade_with_agent_pak_round_trips_messages` function L170-242 — `()` — path; this is why we bind a TCP listener for the upgrade tests.
-  `wait_for_connection` function L244-251 — `(registry: &Arc<ConnectionRegistry>, agent_id: Uuid) -> bool` — path; this is why we bind a TCP listener for the upgrade tests.
-  `wait_for_disconnection` function L253-260 — `(registry: &Arc<ConnectionRegistry>, agent_id: Uuid) -> bool` — path; this is why we bind a TCP listener for the upgrade tests.
-  `spawn_full_broker` function L274-317 — `( fixture: &TestFixture, ) -> (std::net::SocketAddr, Arc<ConnectionRegistry>)` — path; this is why we bind a TCP listener for the upgrade tests.
-  `await_message` function L322-346 — `( socket: &mut tokio_tungstenite::WebSocketStream< tokio_tungstenite::MaybeTlsSt...` — Read frames from `socket` until one of the requested `WsMessage` shapes
-  `rest_mutations_push_messages_over_ws` function L349-440 — `()` — path; this is why we bind a TCP listener for the upgrade tests.
-  `push_to_disconnected_agent_is_a_clean_noop` function L443-473 — `()` — path; this is why we bind a TCP listener for the upgrade tests.
-  `ws_uplink_persists_heartbeat_event_and_health` function L480-613 — `()` — path; this is why we bind a TCP listener for the upgrade tests.
-  `admin_ws_connections_endpoint_reports_live_state` function L620-670 — `()` — path; this is why we bind a TCP listener for the upgrade tests.
-  `admin_ws_connections_endpoint_rejects_non_admin` function L673-695 — `()` — path; this is why we bind a TCP listener for the upgrade tests.
-  `live_subscription_forwards_agent_telemetry_to_subscribers` function L702-794 — `()` — path; this is why we bind a TCP listener for the upgrade tests.
-  `live_subscription_authenticates_via_subprotocol` function L797-877 — `()` — path; this is why we bind a TCP listener for the upgrade tests.
-  `live_subscription_subprotocol_with_bad_pak_is_rejected` function L880-902 — `()` — path; this is why we bind a TCP listener for the upgrade tests.
-  `live_subscription_rejects_unauthorised_caller` function L905-935 — `()` — path; this is why we bind a TCP listener for the upgrade tests.
-  `rest_history_endpoints_return_retained_telemetry_with_retention_metadata` function L942-1030 — `()` — path; this is why we bind a TCP listener for the upgrade tests.
-  `rest_history_endpoints_403_for_unauthorized_callers` function L1033-1066 — `()` — path; this is why we bind a TCP listener for the upgrade tests.
-  `ws_telemetry_ingestion_lands_in_agent_telemetry_tables` function L1073-1170 — `()` — path; this is why we bind a TCP listener for the upgrade tests.
-  `eviction_worker_drops_rows_past_retention` function L1173-1244 — `()` — path; this is why we bind a TCP listener for the upgrade tests.
-  `concurrent_target_post_and_get_delivers_every_push_without_dupes` function L1268-1413 — `()` — path; this is why we bind a TCP listener for the upgrade tests.
-  `N` variable L1273 — `: usize` — path; this is why we bind a TCP listener for the upgrade tests.
-  `await_socket_close` function L1427-1444 — `( socket: &mut tokio_tungstenite::WebSocketStream< tokio_tungstenite::MaybeTlsSt...` — Drive a frame-drain until the socket closes (None / Close / Err), or the
-  `rotating_agent_pak_closes_its_open_ws` function L1447-1496 — `()` — path; this is why we bind a TCP listener for the upgrade tests.
-  `deleting_agent_closes_its_open_ws` function L1499-1543 — `()` — path; this is why we bind a TCP listener for the upgrade tests.
-  `wait_until` function L1546-1561 — `(timeout: std::time::Duration, mut predicate: F) -> bool` — Repeatedly poll `predicate` until it returns true or `timeout` elapses.

### crates/brokkr-broker/tests/integration

> *Semantic summary to be generated by AI agent.*

#### crates/brokkr-broker/tests/integration/cli.rs

-  `test_rotate_agent_key_returns_usable_pak_and_audits` function L17-59 — `()` — synchronous audit entries.
-  `test_rotate_generator_key_returns_usable_pak` function L62-86 — `()` — synchronous audit entries.

#### crates/brokkr-broker/tests/integration/main.rs

-  `api` module L7 — `-`
-  `cli` module L8 — `-`
-  `dal` module L9 — `-`
-  `db` module L10 — `-`
-  `fixtures` module L12 — `-`

### crates/brokkr-broker/tests/integration/dal

> *Semantic summary to be generated by AI agent.*

#### crates/brokkr-broker/tests/integration/dal/agent_annotations.rs

-  `test_create_agent_annotation` function L11-28 — `()`
-  `test_get_agent_annotation` function L31-48 — `()`
-  `test_list_agent_annotations` function L51-74 — `()`
-  `test_update_agent_annotation` function L77-93 — `()`
-  `test_delete_agent_annotation` function L96-116 — `()`
-  `test_delete_all_agent_annotations` function L119-139 — `()`

#### crates/brokkr-broker/tests/integration/dal/agent_events.rs

-  `test_create_agent_event` function L14-87 — `()`
-  `test_get_agent_event` function L90-159 — `()`
-  `test_get_deleted_agent_event` function L162-248 — `()`
-  `test_update_agent_event` function L251-327 — `()`
-  `test_soft_delete_agent_event` function L330-409 — `()`
-  `test_hard_delete_agent_event` function L412-490 — `()`
-  `test_list_agent_events` function L493-580 — `()`
-  `test_get_events_filtered` function L583-702 — `()`

#### crates/brokkr-broker/tests/integration/dal/agent_labels.rs

-  `test_create_agent_label` function L11-25 — `()`
-  `test_get_agent_label` function L28-47 — `()`
-  `test_list_labels_for_agent` function L50-77 — `()`
-  `test_delete_agent_label` function L80-104 — `()`
-  `test_delete_all_labels_for_agent` function L107-139 — `()`
-  `test_label_exists` function L142-167 — `()`

#### crates/brokkr-broker/tests/integration/dal/agent_targets.rs

-  `test_create_agent_target` function L11-31 — `()`
-  `test_get_agent_target` function L34-54 — `()`
-  `test_list_agent_targets` function L57-79 — `()`
-  `test_list_agent_targets_for_agent` function L82-103 — `()`
-  `test_list_agent_targets_for_stack` function L106-127 — `()`
-  `test_delete_agent_target` function L130-154 — `()`
-  `test_delete_agent_targets_for_agent` function L157-184 — `()`
-  `test_delete_agent_targets_for_stack` function L187-214 — `()`

#### crates/brokkr-broker/tests/integration/dal/agents.rs

-  `test_create_agent` function L17-33 — `()`
-  `test_get_agent` function L36-49 — `()`
-  `test_get_deleted_agent` function L52-78 — `()`
-  `test_list_agents` function L81-101 — `()`
-  `test_update_agent` function L104-121 — `()`
-  `test_soft_delete_agent` function L124-143 — `()`
-  `test_hard_delete_agent` function L146-164 — `()`
-  `test_filter_by_labels_single_label` function L167-191 — `()`
-  `test_filter_by_labels_multiple_labels_or` function L194-228 — `()`
-  `test_filter_by_labels_multiple_labels_and` function L231-263 — `()`
-  `test_filter_by_labels_no_match` function L266-283 — `()`
-  `test_filter_by_annotations` function L286-437 — `()`
-  `test_get_agent_by_target_id` function L440-490 — `()`
-  `test_get_agent_details` function L493-576 — `()`
-  `test_record_heartbeat` function L579-629 — `()`
-  `test_update_agent_pak_hash` function L632-654 — `()`
-  `test_get_agent_by_name_and_cluster_name` function L657-684 — `()`
-  `test_recreate_agent_after_soft_delete` function L687-735 — `()`

#### crates/brokkr-broker/tests/integration/dal/deployment_health.rs

-  `test_upsert_deployment_health` function L12-66 — `()`
-  `test_upsert_batch_deployment_health` function L69-129 — `()`
-  `test_get_deployment_health_by_agent_and_deployment` function L132-177 — `()`
-  `test_list_deployment_health_by_agent` function L180-231 — `()`
-  `test_list_deployment_health_by_stack` function L234-270 — `()`
-  `test_list_deployment_health_by_status` function L273-324 — `()`
-  `test_delete_deployment_health` function L327-370 — `()`
-  `test_delete_deployment_health_by_agent` function L373-434 — `()`

#### crates/brokkr-broker/tests/integration/dal/deployment_objects.rs

-  `test_create_deployment_object` function L11-33 — `()`
-  `test_get_deployment_object` function L36-59 — `()`
-  `test_get_deleted_deployment_object` function L62-94 — `()`
-  `test_list_deployment_objects_for_stack` function L97-128 — `()`
-  `test_soft_delete_deployment_object` function L131-156 — `()`
-  `test_get_latest_deployment_object_for_stack` function L159-181 — `()`
-  `test_get_target_state_for_agent_incremental` function L184-247 — `()`
-  `test_get_target_state_for_agent_full` function L250-317 — `()`
-  `test_get_target_state_for_agent_with_no_targets` function L320-334 — `()`
-  `test_get_target_state_for_agent_with_all_deployed_incremental` function L338-374 — `()`
-  `test_get_target_state_for_agent_with_all_deployed_full` function L377-428 — `()`
-  `test_get_target_state_for_agent_with_deletion_markers_incremental` function L431-500 — `()`
-  `test_get_target_state_for_agent_with_deletion_markers_full` function L503-574 — `()`
-  `test_search_deployment_objects_by_checksum` function L577-636 — `()`
-  `test_get_desired_state_for_agent` function L639-708 — `()`
-  `test_target_state_direct_targeting_after_deployment_exists` function L719-763 — `()` — Test that direct targeting (agent_targets table) works when deployment exists first.
-  `test_target_state_label_targeting_after_deployment_exists` function L769-814 — `()` — Test that label targeting works when deployment exists first.
-  `test_target_state_annotation_targeting_after_deployment_exists` function L820-866 — `()` — Test that annotation targeting works when deployment exists first.

#### crates/brokkr-broker/tests/integration/dal/diagnostic_requests.rs

-  `test_create_diagnostic_request` function L12-43 — `()`
-  `test_get_diagnostic_request` function L46-79 — `()`
-  `test_get_pending_for_agent` function L82-119 — `()`
-  `test_claim_diagnostic_request` function L122-159 — `()`
-  `test_complete_diagnostic_request` function L162-195 — `()`
-  `test_fail_diagnostic_request` function L198-228 — `()`
-  `test_list_by_deployment_object` function L231-261 — `()`
-  `test_expire_old_requests` function L264-309 — `()`
-  `test_cleanup_old_requests` function L312-358 — `()`
-  `test_delete_diagnostic_request` function L361-396 — `()`

#### crates/brokkr-broker/tests/integration/dal/diagnostic_results.rs

-  `test_create_diagnostic_result` function L13-54 — `()`
-  `test_get_diagnostic_result` function L57-104 — `()`
-  `test_get_diagnostic_result_by_request` function L107-163 — `()`
-  `test_delete_diagnostic_result` function L166-218 — `()`
-  `test_delete_diagnostic_result_by_request` function L221-273 — `()`
-  `test_cascade_delete_on_request_deletion` function L276-328 — `()`

#### crates/brokkr-broker/tests/integration/dal/event_emission.rs

-  `create_subscription_for_event` function L16-29 — `(name: &str, event_type: &str) -> NewWebhookSubscription` — webhook events and create corresponding delivery records.
-  `create_disabled_subscription` function L31-44 — `(name: &str, event_type: &str) -> NewWebhookSubscription` — webhook events and create corresponding delivery records.
-  `create_subscription_with_target_labels` function L46-63 — `( name: &str, event_type: &str, labels: Vec<String>, ) -> NewWebhookSubscription` — webhook events and create corresponding delivery records.
-  `create_subscription_with_agent_filter` function L65-83 — `( name: &str, event_type: &str, agent_id: uuid::Uuid, ) -> NewWebhookSubscriptio...` — webhook events and create corresponding delivery records.
-  `test_work_order_completion_emits_event` function L90-157 — `()` — webhook events and create corresponding delivery records.
-  `test_wildcard_subscription_matches_events` function L160-213 — `()` — webhook events and create corresponding delivery records.
-  `test_disabled_subscription_receives_no_deliveries` function L216-269 — `()` — webhook events and create corresponding delivery records.
-  `test_delivery_inherits_target_labels_from_subscription` function L272-334 — `()` — webhook events and create corresponding delivery records.
-  `test_no_delivery_when_no_matching_subscription` function L337-393 — `()` — webhook events and create corresponding delivery records.
-  `test_multiple_subscriptions_receive_same_event` function L396-471 — `()` — webhook events and create corresponding delivery records.

#### crates/brokkr-broker/tests/integration/dal/generators.rs

-  `test_create_generator` function L12-29 — `()`
-  `test_get_generator` function L32-59 — `()`
-  `test_list_generators` function L62-97 — `()`
-  `test_update_generator` function L100-120 — `()`
-  `test_soft_delete_generator` function L123-153 — `()`
-  `test_update_pak_hash` function L156-172 — `()`
-  `test_update_last_active` function L175-193 — `()`
-  `test_get_by_name` function L196-213 — `()`
-  `test_get_by_active_status` function L216-258 — `()`
-  `test_recreate_generator_after_soft_delete` function L261-322 — `()`

#### crates/brokkr-broker/tests/integration/dal/mod.rs

-  `agent_annotations` module L7 — `-`
-  `agent_events` module L8 — `-`
-  `agent_labels` module L9 — `-`
-  `agent_targets` module L10 — `-`
-  `agents` module L11 — `-`
-  `deployment_health` module L12 — `-`
-  `deployment_objects` module L13 — `-`
-  `diagnostic_requests` module L14 — `-`
-  `diagnostic_results` module L15 — `-`
-  `event_emission` module L16 — `-`
-  `generators` module L17 — `-`
-  `stack_annotations` module L18 — `-`
-  `stack_labels` module L19 — `-`
-  `stacks` module L20 — `-`
-  `templates` module L21 — `-`
-  `webhook_deliveries` module L22 — `-`
-  `webhook_subscriptions` module L23 — `-`
-  `work_orders` module L24 — `-`

#### crates/brokkr-broker/tests/integration/dal/stack_annotations.rs

-  `test_create_stack_annotation` function L11-35 — `()`
-  `test_get_stack_annotation` function L38-58 — `()`
-  `test_list_annotations_for_stack` function L61-89 — `()`
-  `test_update_stack_annotation` function L92-114 — `()`
-  `test_delete_stack_annotation` function L117-140 — `()`
-  `test_delete_all_annotations_for_stack` function L143-167 — `()`

#### crates/brokkr-broker/tests/integration/dal/stack_labels.rs

-  `test_create_stack_label` function L11-30 — `()`
-  `test_get_stack_label` function L33-51 — `()`
-  `test_list_labels_for_stack` function L54-73 — `()`
-  `test_delete_stack_label` function L76-99 — `()`
-  `test_delete_all_labels_for_stack` function L102-126 — `()`

#### crates/brokkr-broker/tests/integration/dal/stacks.rs

-  `test_create_stack` function L14-36 — `()`
-  `test_get_stack` function L38-55 — `()`
-  `test_get_deleted_stack` function L58-89 — `()`
-  `test_list_stacks` function L92-117 — `()`
-  `test_update_stack` function L120-122 — `()`
-  `test_soft_delete_stack` function L125-148 — `()`
-  `test_hard_delete_stack` function L151-192 — `()`
-  `test_hard_delete_non_existent_stack` function L195-208 — `()`
-  `test_filter_by_labels_or` function L211-236 — `()`
-  `test_filter_by_labels_and` function L239-263 — `()`
-  `test_filter_by_labels_no_match` function L266-285 — `()`
-  `test_filter_by_labels_empty_input` function L288-297 — `()`
-  `test_filter_by_labels_non_existent` function L300-309 — `()`
-  `test_filter_by_labels_duplicate` function L312-336 — `()`
-  `test_filter_by_labels_mixed_existing_and_non_existent` function L339-379 — `()`
-  `test_filter_by_annotations` function L382-450 — `()`
-  `test_get_associated_stacks` function L453-581 — `()`
-  `test_recreate_stack_after_soft_delete` function L584-637 — `()`

#### crates/brokkr-broker/tests/integration/dal/templates.rs

-  `TEST_TEMPLATE_CONTENT` variable L9-12 — `: &str`
-  `test_create_template` function L15-33 — `()`
-  `test_create_template_with_generator` function L36-55 — `()`
-  `test_get_template` function L58-78 — `()`
-  `test_list_templates` function L81-106 — `()`
-  `test_list_templates_by_generator` function L109-138 — `()`
-  `test_versioning` function L141-174 — `()`
-  `test_get_latest_version` function L177-205 — `()`
-  `test_list_versions` function L208-233 — `()`
-  `test_soft_delete` function L236-261 — `()`
-  `test_template_labels` function L264-286 — `()`
-  `test_template_annotations` function L289-311 — `()`
-  `test_delete_label` function L314-340 — `()`
-  `test_delete_annotation` function L343-369 — `()`
-  `test_checksum_generation` function L372-386 — `()`
-  `test_same_content_same_checksum` function L389-409 — `()`
-  `test_recreate_template_after_soft_delete` function L412-470 — `()`

#### crates/brokkr-broker/tests/integration/dal/webhook_deliveries.rs

-  `create_test_subscription` function L21-37 — `(name: &str) -> NewWebhookSubscription`
-  `create_test_subscription_with_labels` function L39-52 — `(name: &str, labels: Vec<String>) -> NewWebhookSubscription`
-  `create_test_event` function L54-63 — `() -> BrokkrEvent`
-  `test_create_delivery` function L67-94 — `()`
-  `test_create_delivery_with_target_labels` function L98-120 — `()`
-  `test_get_delivery` function L124-147 — `()`
-  `test_claim_for_broker` function L151-181 — `()`
-  `test_claim_for_agent_with_matching_labels` function L185-216 — `()`
-  `test_claim_for_agent_without_matching_labels` function L220-249 — `()`
-  `test_release_expired` function L253-310 — `()`
-  `test_mark_success` function L314-338 — `()`
-  `test_mark_failed_with_retry` function L342-367 — `()`
-  `test_process_retries` function L371-438 — `()`
-  `test_mark_failed_max_retries_exceeded` function L442-466 — `()`
-  `test_list_for_subscription` function L470-524 — `()`
-  `test_cleanup_old_deliveries` function L528-584 — `()`
-  `test_claim_pagination` function L588-622 — `()`
-  `test_retry_failed_delivery` function L626-657 — `()`
-  `test_get_stats` function L661-709 — `()`
-  `test_exponential_backoff_timing` function L717-811 — `()`
-  `test_claim_requires_all_labels` function L819-877 — `()`
-  `test_empty_target_labels_matches_broker` function L881-920 — `()`
-  `test_valid_acquired_until_stays_acquired` function L928-965 — `()`
-  `test_released_delivery_claimable_by_different_agent` function L969-1019 — `()`

#### crates/brokkr-broker/tests/integration/dal/webhook_subscriptions.rs

-  `create_test_subscription` function L10-23 — `(name: &str, event_types: Vec<&str>) -> NewWebhookSubscription`
-  `create_test_subscription_with_labels` function L25-42 — `( name: &str, event_types: Vec<&str>, labels: Vec<String>, ) -> NewWebhookSubscr...`
-  `test_create_subscription` function L45-63 — `()`
-  `test_create_subscription_with_target_labels` function L66-84 — `()`
-  `test_get_subscription` function L87-106 — `()`
-  `test_list_subscriptions` function L109-134 — `()`
-  `test_list_enabled_only` function L137-163 — `()`
-  `test_update_subscription` function L166-196 — `()`
-  `test_update_subscription_target_labels` function L199-231 — `()`
-  `test_delete_subscription` function L234-259 — `()`
-  `test_get_matching_subscriptions_exact` function L262-299 — `()`
-  `test_get_matching_subscriptions_wildcard` function L302-338 — `()`
-  `test_get_matching_subscriptions_star_wildcard` function L341-369 — `()`
-  `test_disabled_subscriptions_not_matched` function L372-392 — `()`

#### crates/brokkr-broker/tests/integration/dal/work_orders.rs

-  `test_create_work_order` function L19-43 — `()`
-  `test_get_work_order` function L46-60 — `()`
-  `test_get_nonexistent_work_order` function L63-73 — `()`
-  `test_list_work_orders` function L76-90 — `()`
-  `test_list_filtered_by_status` function L93-126 — `()`
-  `test_list_filtered_by_work_type` function L129-143 — `()`
-  `test_delete_work_order` function L146-166 — `()`
-  `test_list_pending_for_agent` function L173-206 — `()`
-  `test_list_pending_for_agent_with_work_type_filter` function L209-229 — `()`
-  `test_claim_work_order` function L232-248 — `()`
-  `test_claim_work_order_not_targeted` function L251-261 — `()`
-  `test_claim_already_claimed_work_order` function L264-285 — `()`
-  `test_release_work_order` function L288-311 — `()`
-  `test_release_work_order_wrong_agent` function L314-334 — `()`
-  `test_complete_success` function L341-373 — `()`
-  `test_complete_failure_with_retries` function L376-426 — `()`
-  `test_complete_failure_max_retries_exceeded` function L429-480 — `()`
-  `test_complete_failure_non_retryable` function L483-544 — `()`
-  `test_process_retry_pending` function L551-619 — `()`
-  `test_add_target` function L626-636 — `()`
-  `test_add_targets_batch` function L639-662 — `()`
-  `test_list_targets` function L665-682 — `()`
-  `test_remove_target` function L685-708 — `()`
-  `test_get_log` function L715-743 — `()`
-  `test_list_log` function L746-785 — `()`
-  `test_list_log_filtered` function L788-852 — `()`
-  `test_list_log_with_limit` function L855-883 — `()`
-  `test_add_label` function L890-898 — `()`
-  `test_add_multiple_labels` function L901-926 — `()`
-  `test_remove_label` function L929-950 — `()`
-  `test_add_annotation` function L953-962 — `()`
-  `test_add_multiple_annotations` function L965-988 — `()`
-  `test_remove_annotation` function L991-1012 — `()`
-  `test_list_pending_for_agent_with_label_match` function L1015-1035 — `()`
-  `test_list_pending_for_agent_with_annotation_match` function L1038-1058 — `()`
-  `test_list_pending_for_agent_no_match` function L1061-1080 — `()`
-  `test_list_pending_for_agent_or_logic` function L1083-1103 — `()`
-  `test_list_pending_for_agent_combined_targeting` function L1106-1142 — `()`
-  `test_claim_with_label_match` function L1145-1165 — `()`
-  `test_claim_with_annotation_match` function L1168-1188 — `()`
-  `test_claim_without_authorization` function L1191-1210 — `()`
-  `test_annotation_key_value_must_both_match` function L1213-1232 — `()`
-  `test_labels_deleted_on_work_order_delete` function L1235-1273 — `()`

### crates/brokkr-broker/tests/integration/db

> *Semantic summary to be generated by AI agent.*

#### crates/brokkr-broker/tests/integration/db/mod.rs

-  `multi_tenant` module L7 — `-`
-  `TestRecord` struct L25-30 — `{ id: i32, name: String }` — Represents a record in the test database table.
-  `test_connection_pool_integration` function L46-141 — `()` — Integration test for the connection pool functionality.

#### crates/brokkr-broker/tests/integration/db/multi_tenant.rs

-  `MIGRATIONS` variable L20 — `: EmbeddedMigrations` — Integration tests for multi-tenant schema isolation functionality
-  `create_test_database` function L23-37 — `(base_url: &str) -> String` — Helper function to create a test database
-  `drop_test_database` function L40-58 — `(base_url: &str, db_name: &str)` — Helper function to drop a test database
-  `test_schema_isolation` function L67-181 — `()` — Test complete data isolation between different schemas
-  `test_schema_auto_provisioning` function L190-237 — `()` — Test automatic schema provisioning on first connection
-  `test_backward_compatibility_no_schema` function L246-285 — `()` — Test backward compatibility with no schema (public schema)
-  `test_invalid_schema_name` function L294-331 — `()` — Test schema name validation

### crates/brokkr-cli/src

> *Semantic summary to be generated by AI agent.*

#### crates/brokkr-cli/src/config.rs

- pub `ConfigLayer` struct L20-26 — `{ broker_url: Option<String>, pak: Option<String> }` — One layer of partially-specified connection settings.
- pub `ResolvedConfig` struct L30-33 — `{ broker_url: String, pak: String }` — Fully-resolved connection settings, ready to build a client from.
- pub `resolve` function L38-65 — `( flag: &ConfigLayer, env: &ConfigLayer, file: &ConfigLayer, ) -> Result<Resolve...` — Fold the three layers in precedence order — `flag` wins over `env`, which
- pub `normalize_base_url` function L70-77 — `(url: &str) -> String` — Ensure the base URL carries the `/api/v1` prefix the SDK expects.
- pub `default_config_path` function L81-83 — `() -> Option<PathBuf>` — Default config-file location, `~/.brokkr/config`.
- pub `load_file` function L88-95 — `(path: &Path) -> Result<ConfigLayer, String>` — Read a TOML config layer from `path`.
- pub `parse_file` function L98-100 — `(contents: &str) -> Result<ConfigLayer, toml::de::Error>` — Parse a TOML config layer from a string (separated out for testing).
- pub `env_layer` function L103-108 — `() -> ConfigLayer` — Build the environment layer from `BROKKR_BROKER_URL` / `BROKKR_PAK`.
-  `tests` module L111-194 — `-` — `~/.brokkr/config` and override per-invocation without editing it.
-  `layer` function L114-119 — `(url: Option<&str>, pak: Option<&str>) -> ConfigLayer` — `~/.brokkr/config` and override per-invocation without editing it.
-  `flag_beats_env_beats_file` function L122-130 — `()` — `~/.brokkr/config` and override per-invocation without editing it.
-  `file_used_when_nothing_else_set` function L133-139 — `()` — `~/.brokkr/config` and override per-invocation without editing it.
-  `missing_broker_url_is_an_error` function L142-146 — `()` — `~/.brokkr/config` and override per-invocation without editing it.
-  `missing_pak_is_an_error` function L149-153 — `()` — `~/.brokkr/config` and override per-invocation without editing it.
-  `blank_values_are_treated_as_unset` function L156-163 — `()` — `~/.brokkr/config` and override per-invocation without editing it.
-  `normalize_adds_prefix_once` function L166-172 — `()` — `~/.brokkr/config` and override per-invocation without editing it.
-  `parse_file_reads_both_keys` function L175-178 — `()` — `~/.brokkr/config` and override per-invocation without editing it.
-  `parse_file_tolerates_partial_and_empty` function L181-187 — `()` — `~/.brokkr/config` and override per-invocation without editing it.
-  `load_file_missing_is_empty_not_error` function L190-193 — `()` — `~/.brokkr/config` and override per-invocation without editing it.

#### crates/brokkr-cli/src/main.rs

-  `config` module L15 — `-` — `brokkr` — command-line client for the Brokkr control plane.
-  `Cli` struct L26-32 — `{ command: Command, connection: ConnectionArgs }` — Brokkr control-plane CLI.
-  `ConnectionArgs` struct L37-49 — `{ broker_url: Option<String>, pak: Option<String>, config: Option<PathBuf> }` — Connection settings shared by every command.
-  `Command` enum L52-55 — `Apply` — no-op.
-  `ApplyArgs` struct L58-70 — `{ filename: PathBuf, stack: String, target_label: Vec<String> }` — no-op.
-  `main` function L73-82 — `() -> ExitCode` — no-op.
-  `run` function L84-94 — `(cli: Cli) -> Result<(), String>` — no-op.
-  `resolve_connection` function L97-108 — `(args: &ConnectionArgs) -> Result<ResolvedConfig, String>` — Layer the CLI flags over the environment and the config file.
-  `apply` function L110-130 — `(client: &BrokkrClient, args: ApplyArgs) -> Result<(), String>` — no-op.

### crates/brokkr-cli/tests

> *Semantic summary to be generated by AI agent.*

#### crates/brokkr-cli/tests/cli.rs

-  `brokkr` function L17-19 — `() -> Command` — Path to the compiled binary under test (Cargo sets `CARGO_BIN_EXE_<name>`).
-  `sandboxed` function L23-29 — `(mut cmd: Command) -> Command` — Run with a deliberately empty environment so a developer's real
-  `run` function L31-36 — `(mut cmd: Command) -> (std::process::Output, String, String)` — over the contract-tested `BrokkrClient::apply`.
-  `help_lists_apply` function L39-45 — `()` — over the contract-tested `BrokkrClient::apply`.
-  `apply_help_documents_flags` function L48-56 — `()` — over the contract-tested `BrokkrClient::apply`.
-  `version_prints` function L59-65 — `()` — over the contract-tested `BrokkrClient::apply`.
-  `apply_requires_stack_and_filename` function L68-78 — `()` — over the contract-tested `BrokkrClient::apply`.
-  `apply_without_connection_config_errors_clearly` function L81-95 — `()` — over the contract-tested `BrokkrClient::apply`.
-  `malformed_config_file_is_reported` function L98-122 — `()` — over the contract-tested `BrokkrClient::apply`.
-  `config_file_supplies_connection_then_bundle_read_runs` function L125-162 — `()` — over the contract-tested `BrokkrClient::apply`.

### crates/brokkr-client/src

> *Semantic summary to be generated by AI agent.*

#### crates/brokkr-client/src/lib.rs

-  `wrapper` module L23 — `-` — layer added by task BROKKR-T-0137 (C1).

#### crates/brokkr-client/src/wrapper.rs

- pub `BrokkrError` enum L51-65 — `Api | Transport | UnexpectedResponse | InvalidRequest` — Top-level error returned by every wrapper method.
- pub `status` function L69-76 — `(&self) -> Option<reqwest::StatusCode>` — HTTP status, when known.
- pub `code` function L80-85 — `(&self) -> Option<&str>` — Stable, machine-readable error code from the wire response, if any.
- pub `is_retryable` function L90-100 — `(&self) -> bool` — Whether this error is appropriate to retry.
- pub `BrokkrClientBuilder` struct L154-161 — `{ base_url: String, token: Option<String>, request_timeout: Duration, connect_ti...` — Builder for [`BrokkrClient`].
- pub `token` function L178-181 — `(mut self, token: impl Into<String>) -> Self` — PAK credential (admin, agent, or generator).
- pub `request_timeout` function L184-187 — `(mut self, timeout: Duration) -> Self` — Total per-request timeout.
- pub `connect_timeout` function L190-193 — `(mut self, timeout: Duration) -> Self` — TCP connect timeout.
- pub `max_retries` function L197-200 — `(mut self, max: u32) -> Self` — Maximum retry attempts for [`BrokkrClient::retry`].
- pub `initial_backoff` function L204-207 — `(mut self, initial: Duration) -> Self` — Initial backoff between retry attempts.
- pub `build` function L209-231 — `(self) -> Result<BrokkrClient, BrokkrError>` — wrapper.
- pub `BrokkrClient` struct L240-244 — `{ inner: Client, max_retries: u32, initial_backoff: Duration }` — Ergonomic client for the Brokkr broker API.
- pub `builder` function L249-251 — `(base_url: impl Into<String>) -> BrokkrClientBuilder` — Start building a client.
- pub `api` function L256-258 — `(&self) -> &Client` — Access the underlying generated client.
- pub `list_telemetry_events` function L273-288 — `( &self, stack_id: Uuid, since: Option<DateTime<Utc>>, limit: Option<i64>, ) -> ...` — Paginated kube-event history for a stack, scoped to the 6h
- pub `list_telemetry_logs` function L293-308 — `( &self, stack_id: Uuid, since: Option<DateTime<Utc>>, limit: Option<i64>, ) -> ...` — Paginated pod-log history for a stack within the 6h retention
- pub `list_ws_connections` function L314-317 — `(&self) -> Result<WsConnectionsResponse, BrokkrError>` — Snapshot of currently-connected agents on the internal WS
- pub `submit_manifests` function L337-354 — `( &self, stack_id: Uuid, path: impl AsRef<Path>, ) -> Result<DeploymentObject, B...` — Read a folder (or file/list of files) of `*.yaml`/`*.yml` manifests,
- pub `apply` function L365-461 — `( &self, stack_name: &str, path: impl AsRef<Path>, targeting: &[String], ) -> Re...` — Idempotently make a folder of manifests the desired state of the stack
- pub `retry` function L472-494 — `(&self, mut op: F) -> Result<T, BrokkrError>` — Run `op` with exponential backoff on retryable errors.
- pub `ApplyOutcome` enum L499-506 — `Created | Updated | Unchanged` — Outcome of [`BrokkrClient::apply`].
-  `BrokkrError` type L67-101 — `= BrokkrError` — wrapper.
-  `BrokkrError` type L103-117 — `= BrokkrError` — wrapper.
-  `fmt` function L104-116 — `(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result` — wrapper.
-  `BrokkrError` type L119 — `= BrokkrError` — wrapper.
-  `BrokkrError` type L121-146 — `= BrokkrError` — wrapper.
-  `from` function L122-145 — `(err: RawError<ErrorResponse>) -> Self` — wrapper.
-  `is_retryable_status` function L148-150 — `(status: reqwest::StatusCode) -> bool` — wrapper.
-  `BrokkrClientBuilder` type L163-232 — `= BrokkrClientBuilder` — wrapper.
-  `new` function L164-173 — `(base_url: impl Into<String>) -> Self` — wrapper.
-  `BrokkrClient` type L246-495 — `= BrokkrClient` — wrapper.
-  `read_manifests` function L513-530 — `(path: &Path) -> Result<String, BrokkrError>` — Read a manifest path into one validated multi-document YAML stream.
-  `collect_manifest_files` function L533-558 — `(path: &Path) -> Result<Vec<std::path::PathBuf>, BrokkrError>` — Resolve a manifest path to the concrete list of files to read.
-  `validate_manifest_documents` function L562-580 — `(content: &str, file: &Path) -> Result<(), BrokkrError>` — Validate that every non-empty document in `content` parses and carries
-  `sha256_hex` function L584-589 — `(content: &str) -> String` — Lowercase hex SHA-256, matching the broker's deployment-object checksum so
-  `tests` module L592-815 — `-` — wrapper.
-  `builder_constructs_without_token` function L596-602 — `()` — wrapper.
-  `builder_accepts_token_and_timeouts` function L605-616 — `()` — wrapper.
-  `invalid_token_header_is_rejected` function L619-624 — `()` — wrapper.
-  `error_code_extracted_from_api_response` function L627-639 — `()` — wrapper.
-  `retryable_classification` function L642-665 — `()` — wrapper.
-  `retry_stops_after_max_attempts` function L668-696 — `()` — wrapper.
-  `ws_wrapper_methods_compile_with_expected_signatures` function L707-722 — `()` — wrapper.
-  `_assert_signatures` function L708-721 — `()` — wrapper.
-  `_types_check` function L709-719 — `()` — wrapper.
-  `retry_returns_immediately_on_non_retryable` function L725-750 — `()` — wrapper.
-  `write` function L754-756 — `(dir: &std::path::Path, name: &str, content: &str)` — wrapper.
-  `read_manifests_concatenates_folder_in_sorted_order` function L759-771 — `()` — wrapper.
-  `read_manifests_accepts_single_file_and_multidoc` function L774-779 — `()` — wrapper.
-  `read_manifests_rejects_missing_apiversion_or_kind` function L782-787 — `()` — wrapper.
-  `read_manifests_rejects_malformed_yaml` function L790-794 — `()` — wrapper.
-  `read_manifests_errors_on_empty_dir_and_missing_path` function L797-801 — `()` — wrapper.
-  `sha256_hex_is_stable_and_matches_known_vector` function L804-813 — `()` — wrapper.

### crates/brokkr-client/tests

> *Semantic summary to be generated by AI agent.*

#### crates/brokkr-client/tests/surface.rs

-  `client_constructs` function L15-17 — `()` — task BROKKR-T-0137 and consume a running broker).
-  `client_exposes_baseline_operations` function L20-37 — `()` — task BROKKR-T-0137 and consume a running broker).
-  `client_surfaces_typed_error_response` function L40-49 — `()` — task BROKKR-T-0137 and consume a running broker).

### crates/brokkr-models/src

> *Semantic summary to be generated by AI agent.*

#### crates/brokkr-models/src/lib.rs

- pub `models` module L16 — `-` — Declares the models module, which likely contains the data structures representing database tables.
- pub `schema` module L19 — `-` — Declares the schema module, which likely contains the database schema definitions.
-  `establish_connection` function L39-42 — `(database_url: String) -> PgConnection` — Establishes a connection to the PostgreSQL database.

### crates/brokkr-models/src/models

> *Semantic summary to be generated by AI agent.*

#### crates/brokkr-models/src/models/agent_annotations.rs

- pub `AgentAnnotation` struct L55-64 — `{ id: Uuid, agent_id: Uuid, key: String, value: String }` — - Neither `key` nor `value` can contain whitespace.
- pub `NewAgentAnnotation` struct L69-76 — `{ agent_id: Uuid, key: String, value: String }` — Represents a new agent annotation to be inserted into the database.
- pub `new` function L90-123 — `(agent_id: Uuid, key: String, value: String) -> Result<Self, String>` — Creates a new `NewAgentAnnotation` instance.
-  `NewAgentAnnotation` type L78-124 — `= NewAgentAnnotation` — - Neither `key` nor `value` can contain whitespace.
-  `tests` module L126-262 — `-` — - Neither `key` nor `value` can contain whitespace.
-  `test_new_agent_annotation_success` function L130-151 — `()` — - Neither `key` nor `value` can contain whitespace.
-  `test_new_agent_annotation_invalid_agent_id` function L154-169 — `()` — - Neither `key` nor `value` can contain whitespace.
-  `test_new_agent_annotation_empty_key` function L172-184 — `()` — - Neither `key` nor `value` can contain whitespace.
-  `test_new_agent_annotation_empty_value` function L187-199 — `()` — - Neither `key` nor `value` can contain whitespace.
-  `test_new_agent_annotation_key_too_long` function L202-214 — `()` — - Neither `key` nor `value` can contain whitespace.
-  `test_new_agent_annotation_value_too_long` function L217-229 — `()` — - Neither `key` nor `value` can contain whitespace.
-  `test_new_agent_annotation_key_with_whitespace` function L232-245 — `()` — - Neither `key` nor `value` can contain whitespace.
-  `test_new_agent_annotation_value_with_whitespace` function L248-261 — `()` — - Neither `key` nor `value` can contain whitespace.

#### crates/brokkr-models/src/models/agent_events.rs

- pub `AgentEvent` struct L72-100 — `{ id: Uuid, created_at: DateTime<Utc>, updated_at: DateTime<Utc>, deleted_at: Op...` — - `status` must be one of: "SUCCESS", "FAILURE", "IN_PROGRESS", or "PENDING".
- pub `NewAgentEvent` struct L105-116 — `{ agent_id: Uuid, deployment_object_id: Uuid, event_type: String, status: String...` — Represents a new agent event to be inserted into the database.
- pub `new` function L132-170 — `( agent_id: Uuid, deployment_object_id: Uuid, event_type: String, status: String...` — Creates a new `NewAgentEvent` instance.
-  `NewAgentEvent` type L118-171 — `= NewAgentEvent` — - `status` must be one of: "SUCCESS", "FAILURE", "IN_PROGRESS", or "PENDING".
-  `tests` module L174-278 — `-` — - `status` must be one of: "SUCCESS", "FAILURE", "IN_PROGRESS", or "PENDING".
-  `test_new_agent_event_success` function L178-218 — `()` — - `status` must be one of: "SUCCESS", "FAILURE", "IN_PROGRESS", or "PENDING".
-  `test_new_agent_event_invalid_agent_id` function L221-238 — `()` — - `status` must be one of: "SUCCESS", "FAILURE", "IN_PROGRESS", or "PENDING".
-  `test_new_agent_event_invalid_status` function L241-257 — `()` — - `status` must be one of: "SUCCESS", "FAILURE", "IN_PROGRESS", or "PENDING".
-  `test_new_agent_event_empty_event_type` function L260-277 — `()` — - `status` must be one of: "SUCCESS", "FAILURE", "IN_PROGRESS", or "PENDING".

#### crates/brokkr-models/src/models/agent_k8s_events.rs

- pub `AgentK8sEvent` struct L22-33 — `{ id: Uuid, agent_id: Uuid, stack_id: Uuid, observed_at: DateTime<Utc>, reason: ...` — See [[BROKKR-I-0019]] and `project_log_retention_stance`.
- pub `NewAgentK8sEvent` struct L37-46 — `{ agent_id: Uuid, stack_id: Uuid, observed_at: DateTime<Utc>, reason: String, me...` — See [[BROKKR-I-0019]] and `project_log_retention_stance`.

#### crates/brokkr-models/src/models/agent_labels.rs

- pub `AgentLabel` struct L55-62 — `{ id: Uuid, agent_id: Uuid, label: String }` — - The `label` cannot contain whitespace.
- pub `NewAgentLabel` struct L67-72 — `{ agent_id: Uuid, label: String }` — Represents a new agent label to be inserted into the database.
- pub `new` function L85-103 — `(agent_id: Uuid, label: String) -> Result<Self, String>` — Creates a new `NewAgentLabel` instance.
-  `NewAgentLabel` type L74-104 — `= NewAgentLabel` — - The `label` cannot contain whitespace.
-  `tests` module L107-196 — `-` — - The `label` cannot contain whitespace.
-  `test_new_agent_label_success` function L111-127 — `()` — - The `label` cannot contain whitespace.
-  `test_new_agent_label_invalid_agent_id` function L130-141 — `()` — - The `label` cannot contain whitespace.
-  `test_new_agent_label_empty_label` function L144-155 — `()` — - The `label` cannot contain whitespace.
-  `test_new_agent_label_too_long` function L158-170 — `()` — - The `label` cannot contain whitespace.
-  `test_new_agent_label_with_whitespace` function L173-185 — `()` — - The `label` cannot contain whitespace.
-  `test_new_agent_label_max_length` function L188-195 — `()` — - The `label` cannot contain whitespace.

#### crates/brokkr-models/src/models/agent_pod_logs.rs

- pub `AgentPodLog` struct L22-32 — `{ id: Uuid, agent_id: Uuid, stack_id: Uuid, namespace: String, pod: String, cont...` — whatever the agent streams and the eviction worker keeps growth bounded.
- pub `NewAgentPodLog` struct L36-44 — `{ agent_id: Uuid, stack_id: Uuid, namespace: String, pod: String, container: Str...` — whatever the agent streams and the eviction worker keeps growth bounded.

#### crates/brokkr-models/src/models/agent_targets.rs

- pub `AgentTarget` struct L54-61 — `{ id: Uuid, agent_id: Uuid, stack_id: Uuid }` — duplicate associations.
- pub `NewAgentTarget` struct L66-71 — `{ agent_id: Uuid, stack_id: Uuid }` — Represents a new agent target to be inserted into the database.
- pub `new` function L85-97 — `(agent_id: Uuid, stack_id: Uuid) -> Result<Self, String>` — Creates a new `NewAgentTarget` instance.
-  `NewAgentTarget` type L73-98 — `= NewAgentTarget` — duplicate associations.
-  `tests` module L101-153 — `-` — duplicate associations.
-  `test_new_agent_target_success` function L105-124 — `()` — duplicate associations.
-  `test_new_agent_target_invalid_agent_id` function L127-138 — `()` — duplicate associations.
-  `test_new_agent_target_invalid_stack_id` function L141-152 — `()` — duplicate associations.

#### crates/brokkr-models/src/models/agents.rs

- pub `Agent` struct L60-80 — `{ id: Uuid, created_at: DateTime<Utc>, updated_at: DateTime<Utc>, deleted_at: Op...` — - There should be a unique constraint on the combination of `name` and `cluster_name`.
- pub `NewAgent` struct L85-90 — `{ name: String, cluster_name: String }` — Represents a new agent to be inserted into the database.
- pub `new` function L104-116 — `(name: String, cluster_name: String) -> Result<Self, String>` — Creates a new `NewAgent` instance.
-  `NewAgent` type L92-117 — `= NewAgent` — - There should be a unique constraint on the combination of `name` and `cluster_name`.
-  `tests` module L120-169 — `-` — - There should be a unique constraint on the combination of `name` and `cluster_name`.
-  `test_new_agent_success` function L124-140 — `()` — - There should be a unique constraint on the combination of `name` and `cluster_name`.
-  `test_new_agent_empty_name` function L143-154 — `()` — - There should be a unique constraint on the combination of `name` and `cluster_name`.
-  `test_new_agent_empty_cluster_name` function L157-168 — `()` — - There should be a unique constraint on the combination of `name` and `cluster_name`.

#### crates/brokkr-models/src/models/audit_logs.rs

- pub `ACTOR_TYPE_ADMIN` variable L24 — `: &str` — Actor type for admin users.
- pub `ACTOR_TYPE_AGENT` variable L26 — `: &str` — Actor type for agents.
- pub `ACTOR_TYPE_GENERATOR` variable L28 — `: &str` — Actor type for generators.
- pub `ACTOR_TYPE_SYSTEM` variable L30 — `: &str` — Actor type for system operations.
- pub `VALID_ACTOR_TYPES` variable L32-37 — `: &[&str]` — They are used for compliance, debugging, and security incident investigation.
- pub `ACTION_PAK_CREATED` variable L40 — `: &str` — They are used for compliance, debugging, and security incident investigation.
- pub `ACTION_PAK_ROTATED` variable L41 — `: &str` — They are used for compliance, debugging, and security incident investigation.
- pub `ACTION_PAK_DELETED` variable L42 — `: &str` — They are used for compliance, debugging, and security incident investigation.
- pub `ACTION_AUTH_FAILED` variable L43 — `: &str` — They are used for compliance, debugging, and security incident investigation.
- pub `ACTION_AUTH_SUCCESS` variable L44 — `: &str` — They are used for compliance, debugging, and security incident investigation.
- pub `ACTION_AGENT_CREATED` variable L47 — `: &str` — They are used for compliance, debugging, and security incident investigation.
- pub `ACTION_AGENT_UPDATED` variable L48 — `: &str` — They are used for compliance, debugging, and security incident investigation.
- pub `ACTION_AGENT_DELETED` variable L49 — `: &str` — They are used for compliance, debugging, and security incident investigation.
- pub `ACTION_STACK_CREATED` variable L50 — `: &str` — They are used for compliance, debugging, and security incident investigation.
- pub `ACTION_STACK_UPDATED` variable L51 — `: &str` — They are used for compliance, debugging, and security incident investigation.
- pub `ACTION_STACK_DELETED` variable L52 — `: &str` — They are used for compliance, debugging, and security incident investigation.
- pub `ACTION_GENERATOR_CREATED` variable L53 — `: &str` — They are used for compliance, debugging, and security incident investigation.
- pub `ACTION_GENERATOR_UPDATED` variable L54 — `: &str` — They are used for compliance, debugging, and security incident investigation.
- pub `ACTION_GENERATOR_DELETED` variable L55 — `: &str` — They are used for compliance, debugging, and security incident investigation.
- pub `ACTION_TEMPLATE_CREATED` variable L56 — `: &str` — They are used for compliance, debugging, and security incident investigation.
- pub `ACTION_TEMPLATE_UPDATED` variable L57 — `: &str` — They are used for compliance, debugging, and security incident investigation.
- pub `ACTION_TEMPLATE_DELETED` variable L58 — `: &str` — They are used for compliance, debugging, and security incident investigation.
- pub `ACTION_WEBHOOK_CREATED` variable L61 — `: &str` — They are used for compliance, debugging, and security incident investigation.
- pub `ACTION_WEBHOOK_UPDATED` variable L62 — `: &str` — They are used for compliance, debugging, and security incident investigation.
- pub `ACTION_WEBHOOK_DELETED` variable L63 — `: &str` — They are used for compliance, debugging, and security incident investigation.
- pub `ACTION_WEBHOOK_DELIVERY_FAILED` variable L64 — `: &str` — They are used for compliance, debugging, and security incident investigation.
- pub `ACTION_WORKORDER_CREATED` variable L67 — `: &str` — They are used for compliance, debugging, and security incident investigation.
- pub `ACTION_WORKORDER_CLAIMED` variable L68 — `: &str` — They are used for compliance, debugging, and security incident investigation.
- pub `ACTION_WORKORDER_COMPLETED` variable L69 — `: &str` — They are used for compliance, debugging, and security incident investigation.
- pub `ACTION_WORKORDER_FAILED` variable L70 — `: &str` — They are used for compliance, debugging, and security incident investigation.
- pub `ACTION_WORKORDER_RETRY` variable L71 — `: &str` — They are used for compliance, debugging, and security incident investigation.
- pub `ACTION_CONFIG_RELOADED` variable L74 — `: &str` — They are used for compliance, debugging, and security incident investigation.
- pub `RESOURCE_TYPE_AGENT` variable L77 — `: &str` — They are used for compliance, debugging, and security incident investigation.
- pub `RESOURCE_TYPE_STACK` variable L78 — `: &str` — They are used for compliance, debugging, and security incident investigation.
- pub `RESOURCE_TYPE_GENERATOR` variable L79 — `: &str` — They are used for compliance, debugging, and security incident investigation.
- pub `RESOURCE_TYPE_TEMPLATE` variable L80 — `: &str` — They are used for compliance, debugging, and security incident investigation.
- pub `RESOURCE_TYPE_WEBHOOK` variable L81 — `: &str` — They are used for compliance, debugging, and security incident investigation.
- pub `RESOURCE_TYPE_WORKORDER` variable L82 — `: &str` — They are used for compliance, debugging, and security incident investigation.
- pub `RESOURCE_TYPE_PAK` variable L83 — `: &str` — They are used for compliance, debugging, and security incident investigation.
- pub `RESOURCE_TYPE_CONFIG` variable L84 — `: &str` — They are used for compliance, debugging, and security incident investigation.
- pub `RESOURCE_TYPE_SYSTEM` variable L85 — `: &str` — They are used for compliance, debugging, and security incident investigation.
- pub `AuditLog` struct L94-120 — `{ id: Uuid, timestamp: DateTime<Utc>, actor_type: String, actor_id: Option<Uuid>...` — An audit log record from the database.
- pub `NewAuditLog` struct L125-142 — `{ actor_type: String, actor_id: Option<Uuid>, action: String, resource_type: Str...` — A new audit log entry to be inserted.
- pub `new` function L153-188 — `( actor_type: &str, actor_id: Option<Uuid>, action: &str, resource_type: &str, r...` — Creates a new audit log entry.
- pub `with_details` function L191-194 — `(mut self, details: serde_json::Value) -> Self` — Adds details to the audit log entry.
- pub `with_ip_address` function L197-200 — `(mut self, ip: impl Into<String>) -> Self` — Adds client IP address to the audit log entry.
- pub `with_user_agent` function L203-206 — `(mut self, user_agent: String) -> Self` — Adds user agent to the audit log entry.
- pub `AuditLogFilter` struct L215-237 — `{ actor_type: Option<String>, actor_id: Option<Uuid>, action: Option<String>, re...` — Filters for querying audit logs.
-  `NewAuditLog` type L144-207 — `= NewAuditLog` — They are used for compliance, debugging, and security incident investigation.
-  `tests` module L244-332 — `-` — They are used for compliance, debugging, and security incident investigation.
-  `test_new_audit_log_success` function L248-261 — `()` — They are used for compliance, debugging, and security incident investigation.
-  `test_new_audit_log_invalid_actor_type` function L264-275 — `()` — They are used for compliance, debugging, and security incident investigation.
-  `test_new_audit_log_empty_action` function L278-283 — `()` — They are used for compliance, debugging, and security incident investigation.
-  `test_audit_log_with_details` function L286-299 — `()` — They are used for compliance, debugging, and security incident investigation.
-  `test_audit_log_with_ip_address` function L302-315 — `()` — They are used for compliance, debugging, and security incident investigation.
-  `test_audit_log_system_action` function L318-331 — `()` — They are used for compliance, debugging, and security incident investigation.

#### crates/brokkr-models/src/models/deployment_health.rs

- pub `HEALTH_STATUS_HEALTHY` variable L39 — `: &str` — Valid health status values
- pub `HEALTH_STATUS_DEGRADED` variable L40 — `: &str` — cluster access.
- pub `HEALTH_STATUS_FAILING` variable L41 — `: &str` — cluster access.
- pub `HEALTH_STATUS_UNKNOWN` variable L42 — `: &str` — cluster access.
- pub `DeploymentHealth` struct L78-103 — `{ id: Uuid, agent_id: Uuid, deployment_object_id: Uuid, status: String, summary:...` — cluster access.
- pub `NewDeploymentHealth` struct L108-119 — `{ agent_id: Uuid, deployment_object_id: Uuid, status: String, summary: Option<St...` — Represents a new deployment health record to be inserted into the database.
- pub `new` function L136-168 — `( agent_id: Uuid, deployment_object_id: Uuid, status: String, summary: Option<St...` — Creates a new `NewDeploymentHealth` instance.
- pub `UpdateDeploymentHealth` struct L174-181 — `{ status: String, summary: Option<String>, checked_at: DateTime<Utc> }` — Represents an update to an existing deployment health record.
- pub `HealthSummary` struct L185-195 — `{ pods_ready: i32, pods_total: i32, conditions: Vec<String>, resources: Option<V...` — Structured health summary for serialization/deserialization.
- pub `ResourceHealth` struct L199-211 — `{ kind: String, name: String, namespace: String, ready: bool, message: Option<St...` — Health status for an individual Kubernetes resource.
-  `VALID_HEALTH_STATUSES` variable L44-49 — `: [&str; 4]` — cluster access.
-  `NewDeploymentHealth` type L121-169 — `= NewDeploymentHealth` — cluster access.
-  `tests` module L214-305 — `-` — cluster access.
-  `test_new_deployment_health_success` function L218-242 — `()` — cluster access.
-  `test_new_deployment_health_invalid_agent_id` function L245-262 — `()` — cluster access.
-  `test_new_deployment_health_invalid_status` function L265-281 — `()` — cluster access.
-  `test_health_summary_serialization` function L284-304 — `()` — cluster access.

#### crates/brokkr-models/src/models/deployment_objects.rs

- pub `DeploymentObject` struct L64-85 — `{ id: Uuid, created_at: DateTime<Utc>, updated_at: DateTime<Utc>, deleted_at: Op...` — - Deployment objects are designed to be immutable after creation, with exceptions for soft deletion.
- pub `NewDeploymentObject` struct L90-99 — `{ stack_id: Uuid, yaml_content: String, yaml_checksum: String, is_deletion_marke...` — Represents a new deployment object to be inserted into the database.
- pub `new` function L115-141 — `( stack_id: Uuid, yaml_content: String, is_deletion_marker: bool, ) -> Result<Se...` — Creates a new `NewDeploymentObject` instance.
-  `NewDeploymentObject` type L101-142 — `= NewDeploymentObject` — - Deployment objects are designed to be immutable after creation, with exceptions for soft deletion.
-  `generate_checksum` function L145-150 — `(content: &str) -> String` — Helper function to generate SHA-256 checksum for YAML content.
-  `tests` module L153-216 — `-` — - Deployment objects are designed to be immutable after creation, with exceptions for soft deletion.
-  `test_new_deployment_object_success` function L157-172 — `()` — - Deployment objects are designed to be immutable after creation, with exceptions for soft deletion.
-  `test_new_deployment_object_invalid_stack_id` function L175-186 — `()` — - Deployment objects are designed to be immutable after creation, with exceptions for soft deletion.
-  `test_new_deployment_object_empty_yaml` function L189-200 — `()` — - Deployment objects are designed to be immutable after creation, with exceptions for soft deletion.
-  `test_new_deployment_object_empty_deletion_marker_allowed` function L203-215 — `()` — - Deployment objects are designed to be immutable after creation, with exceptions for soft deletion.

#### crates/brokkr-models/src/models/diagnostic_requests.rs

- pub `VALID_STATUSES` variable L20 — `: &[&str]` — Valid diagnostic request statuses
- pub `DiagnosticRequest` struct L25-44 — `{ id: Uuid, agent_id: Uuid, deployment_object_id: Uuid, status: String, requeste...` — A diagnostic request record from the database.
- pub `NewDiagnosticRequest` struct L49-60 — `{ agent_id: Uuid, deployment_object_id: Uuid, status: String, requested_by: Opti...` — A new diagnostic request to be inserted.
- pub `new` function L73-101 — `( agent_id: Uuid, deployment_object_id: Uuid, requested_by: Option<String>, rete...` — Creates a new diagnostic request.
- pub `UpdateDiagnosticRequest` struct L107-114 — `{ status: Option<String>, claimed_at: Option<DateTime<Utc>>, completed_at: Optio...` — Changeset for updating a diagnostic request.
-  `NewDiagnosticRequest` type L62-102 — `= NewDiagnosticRequest` — information from agents about specific deployment objects.
-  `tests` module L117-179 — `-` — information from agents about specific deployment objects.
-  `test_new_diagnostic_request_success` function L121-139 — `()` — information from agents about specific deployment objects.
-  `test_new_diagnostic_request_nil_agent_id` function L142-147 — `()` — information from agents about specific deployment objects.
-  `test_new_diagnostic_request_nil_deployment_object_id` function L150-155 — `()` — information from agents about specific deployment objects.
-  `test_new_diagnostic_request_invalid_retention` function L158-163 — `()` — information from agents about specific deployment objects.
-  `test_new_diagnostic_request_default_retention` function L166-178 — `()` — information from agents about specific deployment objects.

#### crates/brokkr-models/src/models/diagnostic_results.rs

- pub `DiagnosticResult` struct L22-37 — `{ id: Uuid, request_id: Uuid, pod_statuses: String, events: String, log_tails: O...` — A diagnostic result record from the database.
- pub `NewDiagnosticResult` struct L42-53 — `{ request_id: Uuid, pod_statuses: String, events: String, log_tails: Option<Stri...` — A new diagnostic result to be inserted.
- pub `new` function L67-96 — `( request_id: Uuid, pod_statuses: String, events: String, log_tails: Option<Stri...` — Creates a new diagnostic result.
-  `NewDiagnosticResult` type L55-97 — `= NewDiagnosticResult` — collected by agents in response to diagnostic requests.
-  `tests` module L100-183 — `-` — collected by agents in response to diagnostic requests.
-  `test_new_diagnostic_result_success` function L104-123 — `()` — collected by agents in response to diagnostic requests.
-  `test_new_diagnostic_result_nil_request_id` function L126-137 — `()` — collected by agents in response to diagnostic requests.
-  `test_new_diagnostic_result_empty_pod_statuses` function L140-151 — `()` — collected by agents in response to diagnostic requests.
-  `test_new_diagnostic_result_empty_events` function L154-165 — `()` — collected by agents in response to diagnostic requests.
-  `test_new_diagnostic_result_no_log_tails` function L168-182 — `()` — collected by agents in response to diagnostic requests.

#### crates/brokkr-models/src/models/generator.rs

- pub `Generator` struct L60-80 — `{ id: Uuid, created_at: DateTime<Utc>, updated_at: DateTime<Utc>, deleted_at: Op...` — - The `is_active` flag determines whether the generator can perform operations.
- pub `NewGenerator` struct L85-90 — `{ name: String, description: Option<String> }` — Represents the data required to create a new generator.
- pub `new` function L103-113 — `(name: String, description: Option<String>) -> Result<Self, String>` — Creates a new `NewGenerator` instance.
-  `NewGenerator` type L92-114 — `= NewGenerator` — - The `is_active` flag determines whether the generator can perform operations.
-  `tests` module L117-151 — `-` — - The `is_active` flag determines whether the generator can perform operations.
-  `test_new_generator_success` function L122-135 — `()` — Tests successful creation of a new generator.
-  `test_new_generator_empty_name` function L139-150 — `()` — Tests failure when creating a new generator with an empty name.

#### crates/brokkr-models/src/models/mod.rs

- pub `agent_annotations` module L7 — `-`
- pub `agent_events` module L8 — `-`
- pub `agent_k8s_events` module L9 — `-`
- pub `agent_labels` module L10 — `-`
- pub `agent_pod_logs` module L11 — `-`
- pub `agent_targets` module L12 — `-`
- pub `agents` module L13 — `-`
- pub `audit_logs` module L14 — `-`
- pub `deployment_health` module L15 — `-`
- pub `deployment_objects` module L16 — `-`
- pub `diagnostic_requests` module L17 — `-`
- pub `diagnostic_results` module L18 — `-`
- pub `generator` module L19 — `-`
- pub `rendered_deployment_objects` module L20 — `-`
- pub `stack_annotations` module L21 — `-`
- pub `stack_labels` module L22 — `-`
- pub `stack_templates` module L23 — `-`
- pub `stacks` module L24 — `-`
- pub `template_annotations` module L25 — `-`
- pub `template_labels` module L26 — `-`
- pub `template_targets` module L27 — `-`
- pub `webhooks` module L28 — `-`
- pub `work_order_annotations` module L29 — `-`
- pub `work_order_labels` module L30 — `-`
- pub `work_orders` module L31 — `-`

#### crates/brokkr-models/src/models/rendered_deployment_objects.rs

- pub `RenderedDeploymentObject` struct L66-79 — `{ id: Uuid, deployment_object_id: Uuid, template_id: Uuid, template_version: i32...` — - `template_parameters` must be a valid JSON string.
- pub `NewRenderedDeploymentObject` struct L84-93 — `{ deployment_object_id: Uuid, template_id: Uuid, template_version: i32, template...` — Represents a new rendered deployment object provenance record to be inserted.
- pub `new` function L109-141 — `( deployment_object_id: Uuid, template_id: Uuid, template_version: i32, template...` — Creates a new `NewRenderedDeploymentObject` instance.
-  `NewRenderedDeploymentObject` type L95-142 — `= NewRenderedDeploymentObject` — - `template_parameters` must be a valid JSON string.
-  `tests` module L145-218 — `-` — - `template_parameters` must be a valid JSON string.
-  `test_new_rendered_deployment_object_success` function L149-171 — `()` — - `template_parameters` must be a valid JSON string.
-  `test_new_rendered_deployment_object_invalid_deployment_object_id` function L174-179 — `()` — - `template_parameters` must be a valid JSON string.
-  `test_new_rendered_deployment_object_invalid_template_id` function L182-187 — `()` — - `template_parameters` must be a valid JSON string.
-  `test_new_rendered_deployment_object_invalid_version` function L190-195 — `()` — - `template_parameters` must be a valid JSON string.
-  `test_new_rendered_deployment_object_invalid_json` function L198-210 — `()` — - `template_parameters` must be a valid JSON string.
-  `test_new_rendered_deployment_object_empty_json_object` function L213-217 — `()` — - `template_parameters` must be a valid JSON string.

#### crates/brokkr-models/src/models/stack_annotations.rs

- pub `StackAnnotation` struct L56-65 — `{ id: Uuid, stack_id: Uuid, key: String, value: String }` — - Neither `key` nor `value` can contain whitespace.
- pub `NewStackAnnotation` struct L70-77 — `{ stack_id: Uuid, key: String, value: String }` — Represents a new stack annotation to be inserted into the database.
- pub `new` function L92-125 — `(stack_id: Uuid, key: String, value: String) -> Result<Self, String>` — Creates a new `NewStackAnnotation` instance.
-  `NewStackAnnotation` type L79-126 — `= NewStackAnnotation` — - Neither `key` nor `value` can contain whitespace.
-  `tests` module L129-265 — `-` — - Neither `key` nor `value` can contain whitespace.
-  `test_new_stack_annotation_success` function L133-154 — `()` — - Neither `key` nor `value` can contain whitespace.
-  `test_new_stack_annotation_invalid_stack_id` function L157-172 — `()` — - Neither `key` nor `value` can contain whitespace.
-  `test_new_stack_annotation_empty_key` function L175-187 — `()` — - Neither `key` nor `value` can contain whitespace.
-  `test_new_stack_annotation_empty_value` function L190-202 — `()` — - Neither `key` nor `value` can contain whitespace.
-  `test_new_stack_annotation_key_too_long` function L205-217 — `()` — - Neither `key` nor `value` can contain whitespace.
-  `test_new_stack_annotation_value_too_long` function L220-232 — `()` — - Neither `key` nor `value` can contain whitespace.
-  `test_new_stack_annotation_key_with_whitespace` function L235-248 — `()` — - Neither `key` nor `value` can contain whitespace.
-  `test_new_stack_annotation_value_with_whitespace` function L251-264 — `()` — - Neither `key` nor `value` can contain whitespace.

#### crates/brokkr-models/src/models/stack_labels.rs

- pub `StackLabel` struct L55-62 — `{ id: Uuid, stack_id: Uuid, label: String }` — - The `label` cannot contain whitespace.
- pub `NewStackLabel` struct L67-72 — `{ stack_id: Uuid, label: String }` — Represents a new stack label to be inserted into the database.
- pub `new` function L86-108 — `(stack_id: Uuid, label: String) -> Result<Self, String>` — Creates a new `NewStackLabel` instance.
-  `NewStackLabel` type L74-109 — `= NewStackLabel` — - The `label` cannot contain whitespace.
-  `tests` module L112-200 — `-` — - The `label` cannot contain whitespace.
-  `test_new_stack_label_success` function L116-132 — `()` — - The `label` cannot contain whitespace.
-  `test_new_stack_label_invalid_stack_id` function L135-146 — `()` — - The `label` cannot contain whitespace.
-  `test_new_stack_label_empty_label` function L149-160 — `()` — - The `label` cannot contain whitespace.
-  `test_new_stack_label_whitespace_label` function L163-174 — `()` — - The `label` cannot contain whitespace.
-  `test_new_stack_label_too_long` function L177-189 — `()` — - The `label` cannot contain whitespace.
-  `test_new_stack_label_max_length` function L192-199 — `()` — - The `label` cannot contain whitespace.

#### crates/brokkr-models/src/models/stack_templates.rs

- pub `StackTemplate` struct L59-82 — `{ id: Uuid, created_at: DateTime<Utc>, updated_at: DateTime<Utc>, deleted_at: Op...` — - Unique constraint on (generator_id, name, version).
- pub `NewStackTemplate` struct L87-102 — `{ generator_id: Option<Uuid>, name: String, description: Option<String>, version...` — Represents a new stack template to be inserted into the database.
- pub `new` function L125-172 — `( generator_id: Option<Uuid>, name: String, description: Option<String>, version...` — Creates a new `NewStackTemplate` instance.
- pub `generate_checksum` function L176-180 — `(content: &str) -> String` — Generates a SHA-256 checksum for the given content.
-  `NewStackTemplate` type L104-173 — `= NewStackTemplate` — - Unique constraint on (generator_id, name, version).
-  `tests` module L183-281 — `-` — - Unique constraint on (generator_id, name, version).
-  `test_new_stack_template_success` function L187-202 — `()` — - Unique constraint on (generator_id, name, version).
-  `test_new_stack_template_system_template` function L205-218 — `()` — - Unique constraint on (generator_id, name, version).
-  `test_new_stack_template_empty_name` function L221-233 — `()` — - Unique constraint on (generator_id, name, version).
-  `test_new_stack_template_empty_content` function L236-248 — `()` — - Unique constraint on (generator_id, name, version).
-  `test_new_stack_template_invalid_version` function L251-263 — `()` — - Unique constraint on (generator_id, name, version).
-  `test_generate_checksum` function L266-280 — `()` — - Unique constraint on (generator_id, name, version).

#### crates/brokkr-models/src/models/stacks.rs

- pub `Stack` struct L57-72 — `{ id: Uuid, created_at: DateTime<Utc>, updated_at: DateTime<Utc>, deleted_at: Op...` — - There should be a unique constraint on the `name` field.
- pub `NewStack` struct L77-84 — `{ name: String, description: Option<String>, generator_id: Uuid }` — Represents a new stack to be inserted into the database.
- pub `new` function L99-121 — `( name: String, description: Option<String>, generator_id: Uuid, ) -> Result<Sel...` — Creates a new `NewStack` instance.
-  `NewStack` type L86-122 — `= NewStack` — - There should be a unique constraint on the `name` field.
-  `tests` module L125-173 — `-` — - There should be a unique constraint on the `name` field.
-  `test_new_stack_success` function L129-144 — `()` — - There should be a unique constraint on the `name` field.
-  `test_new_stack_empty_name` function L147-158 — `()` — - There should be a unique constraint on the `name` field.
-  `test_new_stack_empty_description` function L161-172 — `()` — - There should be a unique constraint on the `name` field.

#### crates/brokkr-models/src/models/template_annotations.rs

- pub `TemplateAnnotation` struct L41-52 — `{ id: Uuid, template_id: Uuid, key: String, value: String, created_at: DateTime<...` — - The `value` must be a non-empty string, max 64 characters, no whitespace.
- pub `NewTemplateAnnotation` struct L57-64 — `{ template_id: Uuid, key: String, value: String }` — Represents a new template annotation to be inserted into the database.
- pub `new` function L79-112 — `(template_id: Uuid, key: String, value: String) -> Result<Self, String>` — Creates a new `NewTemplateAnnotation` instance.
-  `NewTemplateAnnotation` type L66-113 — `= NewTemplateAnnotation` — - The `value` must be a non-empty string, max 64 characters, no whitespace.
-  `tests` module L116-203 — `-` — - The `value` must be a non-empty string, max 64 characters, no whitespace.
-  `test_new_template_annotation_success` function L120-132 — `()` — - The `value` must be a non-empty string, max 64 characters, no whitespace.
-  `test_new_template_annotation_invalid_template_id` function L135-140 — `()` — - The `value` must be a non-empty string, max 64 characters, no whitespace.
-  `test_new_template_annotation_empty_key` function L143-148 — `()` — - The `value` must be a non-empty string, max 64 characters, no whitespace.
-  `test_new_template_annotation_empty_value` function L151-155 — `()` — - The `value` must be a non-empty string, max 64 characters, no whitespace.
-  `test_new_template_annotation_key_with_whitespace` function L158-169 — `()` — - The `value` must be a non-empty string, max 64 characters, no whitespace.
-  `test_new_template_annotation_value_with_whitespace` function L172-180 — `()` — - The `value` must be a non-empty string, max 64 characters, no whitespace.
-  `test_new_template_annotation_key_too_long` function L183-191 — `()` — - The `value` must be a non-empty string, max 64 characters, no whitespace.
-  `test_new_template_annotation_value_too_long` function L194-202 — `()` — - The `value` must be a non-empty string, max 64 characters, no whitespace.

#### crates/brokkr-models/src/models/template_labels.rs

- pub `TemplateLabel` struct L43-52 — `{ id: Uuid, template_id: Uuid, label: String, created_at: DateTime<Utc> }` — - The `label` cannot contain whitespace.
- pub `NewTemplateLabel` struct L57-62 — `{ template_id: Uuid, label: String }` — Represents a new template label to be inserted into the database.
- pub `new` function L76-98 — `(template_id: Uuid, label: String) -> Result<Self, String>` — Creates a new `NewTemplateLabel` instance.
-  `NewTemplateLabel` type L64-99 — `= NewTemplateLabel` — - The `label` cannot contain whitespace.
-  `tests` module L102-153 — `-` — - The `label` cannot contain whitespace.
-  `test_new_template_label_success` function L106-116 — `()` — - The `label` cannot contain whitespace.
-  `test_new_template_label_invalid_template_id` function L119-123 — `()` — - The `label` cannot contain whitespace.
-  `test_new_template_label_empty_label` function L126-130 — `()` — - The `label` cannot contain whitespace.
-  `test_new_template_label_whitespace_label` function L133-137 — `()` — - The `label` cannot contain whitespace.
-  `test_new_template_label_too_long` function L140-145 — `()` — - The `label` cannot contain whitespace.
-  `test_new_template_label_max_length` function L148-152 — `()` — - The `label` cannot contain whitespace.

#### crates/brokkr-models/src/models/template_targets.rs

- pub `TemplateTarget` struct L58-67 — `{ id: Uuid, template_id: Uuid, stack_id: Uuid, created_at: DateTime<Utc> }` — duplicate associations.
- pub `NewTemplateTarget` struct L72-77 — `{ template_id: Uuid, stack_id: Uuid }` — Represents a new template target to be inserted into the database.
- pub `new` function L91-106 — `(template_id: Uuid, stack_id: Uuid) -> Result<Self, String>` — Creates a new `NewTemplateTarget` instance.
-  `NewTemplateTarget` type L79-107 — `= NewTemplateTarget` — duplicate associations.
-  `tests` module L110-162 — `-` — duplicate associations.
-  `test_new_template_target_success` function L114-133 — `()` — duplicate associations.
-  `test_new_template_target_invalid_template_id` function L136-147 — `()` — duplicate associations.
-  `test_new_template_target_invalid_stack_id` function L150-161 — `()` — duplicate associations.

#### crates/brokkr-models/src/models/webhooks.rs

- pub `DELIVERY_STATUS_PENDING` variable L24 — `: &str` — Valid delivery statuses
- pub `DELIVERY_STATUS_ACQUIRED` variable L25 — `: &str` — enabling external systems to receive notifications when events occur in Brokkr.
- pub `DELIVERY_STATUS_SUCCESS` variable L26 — `: &str` — enabling external systems to receive notifications when events occur in Brokkr.
- pub `DELIVERY_STATUS_FAILED` variable L27 — `: &str` — enabling external systems to receive notifications when events occur in Brokkr.
- pub `DELIVERY_STATUS_DEAD` variable L28 — `: &str` — enabling external systems to receive notifications when events occur in Brokkr.
- pub `VALID_DELIVERY_STATUSES` variable L30-36 — `: &[&str]` — enabling external systems to receive notifications when events occur in Brokkr.
- pub `EVENT_AGENT_REGISTERED` variable L43 — `: &str` — enabling external systems to receive notifications when events occur in Brokkr.
- pub `EVENT_AGENT_DEREGISTERED` variable L44 — `: &str` — enabling external systems to receive notifications when events occur in Brokkr.
- pub `EVENT_STACK_CREATED` variable L47 — `: &str` — enabling external systems to receive notifications when events occur in Brokkr.
- pub `EVENT_STACK_DELETED` variable L48 — `: &str` — enabling external systems to receive notifications when events occur in Brokkr.
- pub `EVENT_DEPLOYMENT_CREATED` variable L51 — `: &str` — enabling external systems to receive notifications when events occur in Brokkr.
- pub `EVENT_DEPLOYMENT_APPLIED` variable L52 — `: &str` — enabling external systems to receive notifications when events occur in Brokkr.
- pub `EVENT_DEPLOYMENT_FAILED` variable L53 — `: &str` — enabling external systems to receive notifications when events occur in Brokkr.
- pub `EVENT_DEPLOYMENT_DELETED` variable L54 — `: &str` — enabling external systems to receive notifications when events occur in Brokkr.
- pub `EVENT_WORKORDER_CREATED` variable L57 — `: &str` — enabling external systems to receive notifications when events occur in Brokkr.
- pub `EVENT_WORKORDER_CLAIMED` variable L58 — `: &str` — enabling external systems to receive notifications when events occur in Brokkr.
- pub `EVENT_WORKORDER_COMPLETED` variable L59 — `: &str` — enabling external systems to receive notifications when events occur in Brokkr.
- pub `EVENT_WORKORDER_FAILED` variable L60 — `: &str` — enabling external systems to receive notifications when events occur in Brokkr.
- pub `VALID_EVENT_TYPES` variable L62-79 — `: &[&str]` — enabling external systems to receive notifications when events occur in Brokkr.
- pub `BrokkrEvent` struct L87-96 — `{ id: Uuid, event_type: String, timestamp: DateTime<Utc>, data: serde_json::Valu...` — A Brokkr event that can trigger webhook deliveries.
- pub `new` function L100-107 — `(event_type: &str, data: serde_json::Value) -> Self` — Creates a new event.
- pub `WebhookFilters` struct L112-122 — `{ agent_id: Option<Uuid>, stack_id: Option<Uuid>, labels: Option<std::collection...` — Filters for webhook subscriptions.
- pub `WebhookSubscription` struct L131-160 — `{ id: Uuid, name: String, url_encrypted: Vec<u8>, auth_header_encrypted: Option<...` — A webhook subscription record from the database.
- pub `NewWebhookSubscription` struct L165-186 — `{ name: String, url_encrypted: Vec<u8>, auth_header_encrypted: Option<Vec<u8>>, ...` — A new webhook subscription to be inserted.
- pub `new` function L202-242 — `( name: String, url_encrypted: Vec<u8>, auth_header_encrypted: Option<Vec<u8>>, ...` — Creates a new webhook subscription.
- pub `UpdateWebhookSubscription` struct L248-267 — `{ name: Option<String>, url_encrypted: Option<Vec<u8>>, auth_header_encrypted: O...` — Changeset for updating a webhook subscription.
- pub `WebhookDelivery` struct L276-307 — `{ id: Uuid, subscription_id: Uuid, event_type: String, event_id: Uuid, payload: ...` — A webhook delivery record from the database.
- pub `NewWebhookDelivery` struct L312-325 — `{ subscription_id: Uuid, event_type: String, event_id: Uuid, payload: String, ta...` — A new webhook delivery to be inserted.
- pub `new` function L337-357 — `( subscription_id: Uuid, event: &BrokkrEvent, target_labels: Option<Vec<Option<S...` — Creates a new webhook delivery.
- pub `UpdateWebhookDelivery` struct L363-380 — `{ status: Option<String>, acquired_by: Option<Option<Uuid>>, acquired_until: Opt...` — Changeset for updating a webhook delivery.
-  `BrokkrEvent` type L98-108 — `= BrokkrEvent` — enabling external systems to receive notifications when events occur in Brokkr.
-  `NewWebhookSubscription` type L188-243 — `= NewWebhookSubscription` — enabling external systems to receive notifications when events occur in Brokkr.
-  `NewWebhookDelivery` type L327-358 — `= NewWebhookDelivery` — enabling external systems to receive notifications when events occur in Brokkr.
-  `tests` module L387-556 — `-` — enabling external systems to receive notifications when events occur in Brokkr.
-  `test_brokkr_event_new` function L391-398 — `()` — enabling external systems to receive notifications when events occur in Brokkr.
-  `test_new_webhook_subscription_success` function L401-418 — `()` — enabling external systems to receive notifications when events occur in Brokkr.
-  `test_new_webhook_subscription_with_target_labels` function L421-436 — `()` — enabling external systems to receive notifications when events occur in Brokkr.
-  `test_new_webhook_subscription_empty_name` function L439-452 — `()` — enabling external systems to receive notifications when events occur in Brokkr.
-  `test_new_webhook_subscription_no_event_types` function L455-468 — `()` — enabling external systems to receive notifications when events occur in Brokkr.
-  `test_new_webhook_delivery_success` function L471-484 — `()` — enabling external systems to receive notifications when events occur in Brokkr.
-  `test_new_webhook_delivery_with_target_labels` function L487-498 — `()` — enabling external systems to receive notifications when events occur in Brokkr.
-  `test_new_webhook_delivery_nil_subscription` function L501-511 — `()` — enabling external systems to receive notifications when events occur in Brokkr.
-  `test_webhook_filters_serialization` function L514-529 — `()` — enabling external systems to receive notifications when events occur in Brokkr.
-  `test_valid_event_types` function L532-546 — `()` — enabling external systems to receive notifications when events occur in Brokkr.
-  `test_valid_delivery_statuses` function L549-555 — `()` — enabling external systems to receive notifications when events occur in Brokkr.

#### crates/brokkr-models/src/models/work_order_annotations.rs

- pub `WorkOrderAnnotation` struct L56-67 — `{ id: Uuid, work_order_id: Uuid, key: String, value: String, created_at: chrono:...` — - Neither `key` nor `value` can contain whitespace.
- pub `NewWorkOrderAnnotation` struct L72-79 — `{ work_order_id: Uuid, key: String, value: String }` — Represents a new work order annotation to be inserted into the database.
- pub `new` function L94-127 — `(work_order_id: Uuid, key: String, value: String) -> Result<Self, String>` — Creates a new `NewWorkOrderAnnotation` instance.
-  `NewWorkOrderAnnotation` type L81-128 — `= NewWorkOrderAnnotation` — - Neither `key` nor `value` can contain whitespace.
-  `tests` module L131-280 — `-` — - Neither `key` nor `value` can contain whitespace.
-  `test_new_work_order_annotation_success` function L135-156 — `()` — - Neither `key` nor `value` can contain whitespace.
-  `test_new_work_order_annotation_invalid_work_order_id` function L159-174 — `()` — - Neither `key` nor `value` can contain whitespace.
-  `test_new_work_order_annotation_empty_key` function L177-189 — `()` — - Neither `key` nor `value` can contain whitespace.
-  `test_new_work_order_annotation_empty_value` function L192-204 — `()` — - Neither `key` nor `value` can contain whitespace.
-  `test_new_work_order_annotation_key_too_long` function L207-220 — `()` — - Neither `key` nor `value` can contain whitespace.
-  `test_new_work_order_annotation_value_too_long` function L223-236 — `()` — - Neither `key` nor `value` can contain whitespace.
-  `test_new_work_order_annotation_key_with_whitespace` function L239-252 — `()` — - Neither `key` nor `value` can contain whitespace.
-  `test_new_work_order_annotation_value_with_whitespace` function L255-268 — `()` — - Neither `key` nor `value` can contain whitespace.
-  `test_new_work_order_annotation_max_length` function L271-279 — `()` — - Neither `key` nor `value` can contain whitespace.

#### crates/brokkr-models/src/models/work_order_labels.rs

- pub `WorkOrderLabel` struct L54-63 — `{ id: Uuid, work_order_id: Uuid, label: String, created_at: chrono::DateTime<chr...` — - The `label` cannot contain whitespace.
- pub `NewWorkOrderLabel` struct L68-73 — `{ work_order_id: Uuid, label: String }` — Represents a new work order label to be inserted into the database.
- pub `new` function L87-112 — `(work_order_id: Uuid, label: String) -> Result<Self, String>` — Creates a new `NewWorkOrderLabel` instance.
-  `NewWorkOrderLabel` type L75-113 — `= NewWorkOrderLabel` — - The `label` cannot contain whitespace.
-  `tests` module L116-218 — `-` — - The `label` cannot contain whitespace.
-  `test_new_work_order_label_success` function L120-136 — `()` — - The `label` cannot contain whitespace.
-  `test_new_work_order_label_invalid_work_order_id` function L139-150 — `()` — - The `label` cannot contain whitespace.
-  `test_new_work_order_label_empty_label` function L153-164 — `()` — - The `label` cannot contain whitespace.
-  `test_new_work_order_label_whitespace_label` function L167-178 — `()` — - The `label` cannot contain whitespace.
-  `test_new_work_order_label_too_long` function L181-193 — `()` — - The `label` cannot contain whitespace.
-  `test_new_work_order_label_max_length` function L196-203 — `()` — - The `label` cannot contain whitespace.
-  `test_new_work_order_label_with_whitespace` function L206-217 — `()` — - The `label` cannot contain whitespace.

#### crates/brokkr-models/src/models/work_orders.rs

- pub `WORK_ORDER_STATUS_PENDING` variable L35 — `: &str` — Valid work order statuses
- pub `WORK_ORDER_STATUS_CLAIMED` variable L36 — `: &str` — On completion (success or max retries exceeded), records move to `work_order_log`.
- pub `WORK_ORDER_STATUS_RETRY_PENDING` variable L37 — `: &str` — On completion (success or max retries exceeded), records move to `work_order_log`.
- pub `WORK_TYPE_BUILD` variable L40 — `: &str` — Valid work types
- pub `WorkOrder` struct L76-122 — `{ id: Uuid, created_at: DateTime<Utc>, updated_at: DateTime<Utc>, work_type: Str...` — On completion (success or max retries exceeded), records move to `work_order_log`.
- pub `NewWorkOrder` struct L134-148 — `{ work_type: String, yaml_content: String, max_retries: i32, backoff_seconds: i3...` — On completion (success or max retries exceeded), records move to `work_order_log`.
- pub `new` function L176-216 — `( work_type: String, yaml_content: String, max_retries: Option<i32>, backoff_sec...` — Creates a new `NewWorkOrder` instance with validation.
- pub `WorkOrderLog` struct L247-278 — `{ id: Uuid, work_type: String, created_at: DateTime<Utc>, claimed_at: Option<Dat...` — On completion (success or max retries exceeded), records move to `work_order_log`.
- pub `NewWorkOrderLog` struct L283-302 — `{ id: Uuid, work_type: String, created_at: DateTime<Utc>, claimed_at: Option<Dat...` — Represents a new work order log entry to be inserted.
- pub `from_work_order` function L306-322 — `( work_order: &WorkOrder, success: bool, result_message: Option<String>, ) -> Se...` — Creates a new log entry from a completed work order.
- pub `WorkOrderTarget` struct L349-362 — `{ id: Uuid, work_order_id: Uuid, agent_id: Uuid, created_at: DateTime<Utc> }` — On completion (success or max retries exceeded), records move to `work_order_log`.
- pub `NewWorkOrderTarget` struct L367-372 — `{ work_order_id: Uuid, agent_id: Uuid }` — Represents a new work order target to be inserted.
- pub `new` function L376-387 — `(work_order_id: Uuid, agent_id: Uuid) -> Result<Self, String>` — Creates a new work order target.
-  `default_max_retries` function L150-152 — `() -> i32` — On completion (success or max retries exceeded), records move to `work_order_log`.
-  `default_backoff_seconds` function L154-156 — `() -> i32` — On completion (success or max retries exceeded), records move to `work_order_log`.
-  `default_claim_timeout_seconds` function L158-160 — `() -> i32` — On completion (success or max retries exceeded), records move to `work_order_log`.
-  `NewWorkOrder` type L162-217 — `= NewWorkOrder` — On completion (success or max retries exceeded), records move to `work_order_log`.
-  `NewWorkOrderLog` type L304-323 — `= NewWorkOrderLog` — On completion (success or max retries exceeded), records move to `work_order_log`.
-  `NewWorkOrderTarget` type L374-388 — `= NewWorkOrderTarget` — On completion (success or max retries exceeded), records move to `work_order_log`.
-  `tests` module L391-455 — `-` — On completion (success or max retries exceeded), records move to `work_order_log`.
-  `test_new_work_order_success` function L395-409 — `()` — On completion (success or max retries exceeded), records move to `work_order_log`.
-  `test_new_work_order_empty_work_type` function L412-417 — `()` — On completion (success or max retries exceeded), records move to `work_order_log`.
-  `test_new_work_order_empty_yaml` function L420-424 — `()` — On completion (success or max retries exceeded), records move to `work_order_log`.
-  `test_new_work_order_invalid_max_retries` function L427-437 — `()` — On completion (success or max retries exceeded), records move to `work_order_log`.
-  `test_new_work_order_target_success` function L440-443 — `()` — On completion (success or max retries exceeded), records move to `work_order_log`.
-  `test_new_work_order_target_invalid_ids` function L446-454 — `()` — On completion (success or max retries exceeded), records move to `work_order_log`.

### crates/brokkr-utils/src

> *Semantic summary to be generated by AI agent.*

#### crates/brokkr-utils/src/config.rs

- pub `Settings` struct L121-136 — `{ database: Database, log: Log, pak: PAK, agent: Agent, broker: Broker, cors: Co...` — Represents the main settings structure for the application
- pub `Cors` struct L140-156 — `{ allowed_origins: Vec<String>, allowed_methods: Vec<String>, allowed_headers: V...` — Represents the CORS configuration
- pub `Broker` struct L159-179 — `{ pak_hash: Option<String>, diagnostic_cleanup_interval_seconds: Option<u64>, di...` — Default: 60 (set to 0 to disable caching)
- pub `Agent` struct L184-240 — `{ broker_url: String, polling_interval: u64, kubeconfig_path: Option<String>, ma...` — Represents the agent configuration
- pub `Database` struct L245-250 — `{ url: String, schema: Option<String> }` — Represents the database configuration
- pub `Log` struct L254-260 — `{ level: String, format: String }` — Represents the logging configuration
- pub `Telemetry` struct L268-287 — `{ enabled: bool, otlp_endpoint: String, service_name: String, sampling_rate: f64...` — Represents the telemetry (OpenTelemetry) configuration with hierarchical overrides
- pub `TelemetryOverride` struct L291-300 — `{ enabled: Option<bool>, otlp_endpoint: Option<String>, service_name: Option<Str...` — Component-specific telemetry overrides (all fields optional)
- pub `ResolvedTelemetry` struct L304-309 — `{ enabled: bool, otlp_endpoint: String, service_name: String, sampling_rate: f64...` — Resolved telemetry configuration after merging base with overrides
- pub `for_broker` function L313-328 — `(&self) -> ResolvedTelemetry` — Get resolved telemetry config for broker (base merged with broker overrides)
- pub `for_agent` function L331-346 — `(&self) -> ResolvedTelemetry` — Get resolved telemetry config for agent (base merged with agent overrides)
- pub `PAK` struct L363-380 — `{ prefix: Option<String>, digest: Option<String>, rng: Option<String>, short_tok...` — Represents the PAK configuration
- pub `short_length_as_str` function L384-386 — `(&mut self)` — Convert short token length to string
- pub `long_length_as_str` function L389-391 — `(&mut self)` — Convert long token length to string
- pub `new` function L404-423 — `(file: Option<String>) -> Result<Self, ConfigError>` — Creates a new `Settings` instance
- pub `DynamicConfig` struct L431-448 — `{ log_level: String, diagnostic_cleanup_interval_seconds: u64, diagnostic_max_ag...` — Dynamic configuration values that can be hot-reloaded at runtime.
- pub `from_settings` function L452-472 — `(settings: &Settings) -> Self` — Create DynamicConfig from Settings
- pub `ConfigChange` struct L477-484 — `{ key: String, old_value: String, new_value: String }` — Represents a configuration change detected during reload
- pub `ReloadableConfig` struct L510-517 — `{ static_config: Settings, dynamic: Arc<RwLock<DynamicConfig>>, config_file: Opt...` — Configuration wrapper that separates static (restart-required) settings
- pub `new` function L529-538 — `(file: Option<String>) -> Result<Self, ConfigError>` — Creates a new ReloadableConfig instance
- pub `from_settings` function L550-558 — `(settings: Settings, config_file: Option<String>) -> Self` — Creates a ReloadableConfig from an existing Settings instance
- pub `static_config` function L563-565 — `(&self) -> &Settings` — Get a reference to the static (immutable) settings
- pub `reload` function L571-650 — `(&self) -> Result<Vec<ConfigChange>, ConfigError>` — Reload dynamic configuration from sources (file + environment)
- pub `log_level` function L657-662 — `(&self) -> String` — Get current log level
- pub `diagnostic_cleanup_interval_seconds` function L665-670 — `(&self) -> u64` — Get diagnostic cleanup interval in seconds
- pub `diagnostic_max_age_hours` function L673-678 — `(&self) -> i64` — Get diagnostic max age in hours
- pub `webhook_delivery_interval_seconds` function L681-686 — `(&self) -> u64` — Get webhook delivery interval in seconds
- pub `webhook_delivery_batch_size` function L689-694 — `(&self) -> i64` — Get webhook delivery batch size
- pub `webhook_cleanup_retention_days` function L697-702 — `(&self) -> i64` — Get webhook cleanup retention in days
- pub `cors_allowed_origins` function L705-710 — `(&self) -> Vec<String>` — Get CORS allowed origins
- pub `cors_max_age_seconds` function L713-718 — `(&self) -> u64` — Get CORS max age in seconds
- pub `dynamic_snapshot` function L721-723 — `(&self) -> Option<DynamicConfig>` — Get a snapshot of all dynamic config values
-  `deserialize_string_or_vec` function L76-113 — `(deserializer: D) -> Result<Vec<String>, D::Error>` — Deserializes a comma-separated string or array into Vec<String>
-  `StringOrVec` struct L83 — `-` — Default: 60 (set to 0 to disable caching)
-  `StringOrVec` type L85-110 — `= StringOrVec` — Default: 60 (set to 0 to disable caching)
-  `Value` type L86 — `= Vec<String>` — Default: 60 (set to 0 to disable caching)
-  `expecting` function L88-90 — `(&self, formatter: &mut fmt::Formatter) -> fmt::Result` — Default: 60 (set to 0 to disable caching)
-  `visit_str` function L92-98 — `(self, value: &str) -> Result<Self::Value, E>` — Default: 60 (set to 0 to disable caching)
-  `visit_seq` function L100-109 — `(self, mut seq: A) -> Result<Self::Value, A::Error>` — Default: 60 (set to 0 to disable caching)
-  `DEFAULT_SETTINGS` variable L116 — `: &str` — Default: 60 (set to 0 to disable caching)
-  `default_log_format` function L262-264 — `() -> String` — Default: 60 (set to 0 to disable caching)
-  `Telemetry` type L311-347 — `= Telemetry` — Default: 60 (set to 0 to disable caching)
-  `default_otlp_endpoint` function L349-351 — `() -> String` — Default: 60 (set to 0 to disable caching)
-  `default_service_name` function L353-355 — `() -> String` — Default: 60 (set to 0 to disable caching)
-  `default_sampling_rate` function L357-359 — `() -> f64` — Default: 60 (set to 0 to disable caching)
-  `PAK` type L382-392 — `= PAK` — Default: 60 (set to 0 to disable caching)
-  `Settings` type L394-424 — `= Settings` — Default: 60 (set to 0 to disable caching)
-  `DynamicConfig` type L450-473 — `= DynamicConfig` — Default: 60 (set to 0 to disable caching)
-  `ReloadableConfig` type L519-724 — `= ReloadableConfig` — Default: 60 (set to 0 to disable caching)
-  `tests` module L727-1072 — `-` — Default: 60 (set to 0 to disable caching)
-  `test_settings_default_values` function L737-746 — `()` — Test the creation of Settings with default values
-  `test_telemetry_default_values` function L749-757 — `()` — Default: 60 (set to 0 to disable caching)
-  `test_telemetry_for_broker_no_overrides` function L760-777 — `()` — Default: 60 (set to 0 to disable caching)
-  `test_telemetry_for_broker_full_overrides` function L780-802 — `()` — Default: 60 (set to 0 to disable caching)
-  `test_telemetry_for_broker_partial_overrides` function L805-827 — `()` — Default: 60 (set to 0 to disable caching)
-  `test_telemetry_for_agent_no_overrides` function L830-847 — `()` — Default: 60 (set to 0 to disable caching)
-  `test_telemetry_for_agent_full_overrides` function L850-872 — `()` — Default: 60 (set to 0 to disable caching)
-  `test_telemetry_broker_and_agent_independent` function L875-912 — `()` — Default: 60 (set to 0 to disable caching)
-  `test_telemetry_override_enabled_false_overrides_base_true` function L915-936 — `()` — Default: 60 (set to 0 to disable caching)
-  `test_telemetry_sampling_rate_extremes` function L939-961 — `()` — Default: 60 (set to 0 to disable caching)
-  `test_reloadable_config_creation` function L968-981 — `()` — Default: 60 (set to 0 to disable caching)
-  `test_dynamic_config_from_settings` function L984-995 — `()` — Default: 60 (set to 0 to disable caching)
-  `test_reloadable_config_accessors_with_defaults` function L998-1008 — `()` — Default: 60 (set to 0 to disable caching)
-  `test_reloadable_config_dynamic_snapshot` function L1011-1023 — `()` — Default: 60 (set to 0 to disable caching)
-  `test_reloadable_config_reload_no_changes` function L1026-1036 — `()` — Default: 60 (set to 0 to disable caching)
-  `test_reloadable_config_is_clone` function L1039-1045 — `()` — Default: 60 (set to 0 to disable caching)
-  `test_reloadable_config_thread_safety` function L1048-1071 — `()` — Default: 60 (set to 0 to disable caching)

#### crates/brokkr-utils/src/lib.rs

- pub `config` module L7 — `-`
- pub `logging` module L8 — `-`
- pub `telemetry` module L9 — `-`

#### crates/brokkr-utils/src/logging.rs

- pub `BrokkrLogger` struct L63 — `-` — Custom logger for the Brokkr application
- pub `init` function L131-133 — `(level: &str) -> Result<(), SetLoggerError>` — Initializes the Brokkr logging system with the specified log level.
- pub `init_with_format` function L143-157 — `(level: &str, format: &str) -> Result<(), SetLoggerError>` — Initializes the Brokkr logging system with the specified log level and format.
- pub `update_log_level` function L182-187 — `(level: &str) -> Result<(), String>` — Updates the current log level.
- pub `prelude` module L213-215 — `-` — operations and log level changes from multiple threads.
-  `LOGGER` variable L57 — `: BrokkrLogger` — operations and log level changes from multiple threads.
-  `CURRENT_LEVEL` variable L58 — `: AtomicUsize` — operations and log level changes from multiple threads.
-  `JSON_FORMAT` variable L59 — `: AtomicBool` — operations and log level changes from multiple threads.
-  `INIT` variable L60 — `: OnceCell<()>` — operations and log level changes from multiple threads.
-  `BrokkrLogger` type L65-98 — `= BrokkrLogger` — operations and log level changes from multiple threads.
-  `enabled` function L66-69 — `(&self, metadata: &Metadata) -> bool` — operations and log level changes from multiple threads.
-  `log` function L71-95 — `(&self, record: &Record)` — operations and log level changes from multiple threads.
-  `flush` function L97 — `(&self)` — operations and log level changes from multiple threads.
-  `str_to_level_filter` function L189-199 — `(level: &str) -> LevelFilter` — operations and log level changes from multiple threads.
-  `level_filter_from_u8` function L201-211 — `(v: u8) -> LevelFilter` — operations and log level changes from multiple threads.
-  `tests` module L217-384 — `-` — operations and log level changes from multiple threads.
-  `test_init` function L232-238 — `()` — Verifies that the logger initializes correctly with the specified log level.
-  `test_update_log_level` function L247-261 — `()` — Tests the ability to update the log level after initialization.
-  `test_invalid_log_level` function L269-281 — `()` — Checks the logger's behavior when given invalid log levels.
-  `test_log_macros` function L289-298 — `()` — Ensures that all log macros can be called without errors.
-  `test_thread_safety_and_performance` function L308-383 — `()` — Tests thread safety and performance of the logger under concurrent usage.

#### crates/brokkr-utils/src/telemetry.rs

- pub `TelemetryError` enum L47-54 — `ExporterError | TracerError | SubscriberError` — Error type for telemetry initialization
- pub `init` function L81-167 — `( config: &ResolvedTelemetry, log_level: &str, log_format: &str, ) -> Result<(),...` — Initialize OpenTelemetry tracing with the given configuration.
- pub `shutdown` function L172-174 — `()` — Shutdown OpenTelemetry, flushing any pending traces.
- pub `prelude` module L177-181 — `-` — Re-export tracing macros for convenience
-  `TelemetryError` type L56-64 — `= TelemetryError` — ```
-  `fmt` function L57-63 — `(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result` — ```
-  `TelemetryError` type L66 — `= TelemetryError` — ```
-  `tests` module L184-219 — `-` — ```
-  `test_disabled_telemetry_config` function L188-198 — `()` — ```
-  `test_sampling_rate_bounds` function L201-218 — `()` — ```

### crates/brokkr-utils/tests

> *Semantic summary to be generated by AI agent.*

#### crates/brokkr-utils/tests/integration.rs

-  `test_settings_from_file_and_env` function L22-61 — `()` — Tests the loading of settings from both a file and environment variables.
-  `test_settings_default` function L73-86 — `()` — Tests the loading of default settings when no configuration file is provided.
-  `test_settings_via_brokkr_config_file_env` function L94-119 — `()` — Tests the `BROKKR_CONFIG_FILE` wiring used by the shipped binaries

### crates/brokkr-wire/src

> *Semantic summary to be generated by AI agent.*

#### crates/brokkr-wire/src/lib.rs

- pub `Heartbeat` struct L39-42 — `{ agent_id: Uuid, sent_at: DateTime<Utc> }` — Heartbeat from agent to broker.
- pub `ObjectRef` struct L48-54 — `{ api_version: String, kind: String, namespace: Option<String>, name: String, ui...` — Kubernetes object reference for events and log lines.
- pub `K8sEvent` struct L60-71 — `{ agent_id: Uuid, stack_id: Uuid, observed_at: DateTime<Utc>, reason: String, me...` — A Kubernetes `Event` for an object the agent manages, forwarded upstream
- pub `PodLogLine` struct L76-84 — `{ agent_id: Uuid, stack_id: Uuid, namespace: String, pod: String, container: Str...` — A single line of pod log output forwarded upstream.
- pub `GapReason` enum L89-93 — `RateLimit | BufferFull | Disconnected` — Reason a sequence of log lines was dropped before reaching the broker.
- pub `LogGap` struct L98-104 — `{ agent_id: Uuid, stack_id: Uuid, since_ts: DateTime<Utc>, dropped_count: u64, r...` — Marker emitted when log lines were dropped so consumers can render a
- pub `WsMessage` enum L110-125 — `WorkOrder | TargetChanged | StackChanged | Heartbeat | AgentEvent | AgentHealth ...` — The canonical message envelope on the broker↔agent WebSocket.
- pub `WIRE_VERSION` variable L129 — `: &str` — Wire-protocol version.

### crates/brokkr-wire/tests

> *Semantic summary to be generated by AI agent.*

#### crates/brokkr-wire/tests/golden.rs

-  `sample_messages` function L21-122 — `() -> Vec<WsMessage>` — Build a deterministic sample of every `WsMessage` variant.
-  `every_variant_roundtrips` function L125-136 — `()` — or a tag rename) will fail this test.
-  `variant_tags_are_snake_case` function L139-166 — `()` — or a tag rename) will fail this test.
-  `golden_fixture_matches_current_serialization` function L169-184 — `()` — or a tag rename) will fail this test.
-  `wire_version_is_pinned` function L187-192 — `()` — or a tag rename) will fail this test.

### docs

> *Semantic summary to be generated by AI agent.*

#### docs/mermaid.min.js

- pub `constructor` method L716 — `constructor()`
- pub `visitEndAnchor` method L716 — `visitEndAnchor(a)`
- pub `constructor` method L718 — `constructor()`
- pub `visitStartAnchor` method L718 — `visitStartAnchor(a)`
- pub `constructor` method L806 — `constructor(k,L,R,O,M,B=!1)`
- pub `file` method L806 — `file(k)`
- pub `from` method L806 — `from(k)`
- pub `fsPath` method L806 — `fsPath()`
- pub `isUri` method L806 — `isUri(k)`
- pub `parse` method L806 — `parse(k,L=!1)`
- pub `revive` method L806 — `revive(k)`
- pub `toJSON` method L806 — `toJSON()`
- pub `toString` method L806 — `toString(k=!1)`
- pub `with` method L806 — `with(k)`
-  `$F` function L3 — `function $F(t,e,r)`
-  `B4` function L3-8 — `function B4(t)`
-  `DC` function L3 — `function DC(t,e)`
-  `I4` function L3 — `function I4(t,e,r,n)`
-  `IF` function L3 — `function IF(t,e)`
-  `LC` function L3 — `function LC(t,e,r)`
-  `MC` function L3 — `function MC(t,e)`
-  `NC` function L3 — `function NC(t,e=Q2e)`
-  `PF` function L3 — `function PF(t,e,r)`
-  `RC` function L3 — `function RC(t,e)`
-  `RF` function L3 — `function RF(t,e)`
-  `X2e` function L3 — `function X2e(t,e)`
-  `Y2e` function L3 — `function Y2e(t,e="defs")`
-  `_C` function L3 — `function _C(t,e)`
-  `a` function L3 — `function a(s)`
-  `axe` function L3 — `function axe()`
-  `cxe` function L3 — `function cxe()`
-  `e` function L3 — `function e(f)`
-  `h` function L3 — `function h()`
-  `i` function L3 — `function i(f)`
-  `ixe` function L3 — `function ixe(t)`
-  `k` function L3 — `function k(R)`
-  `l` function L3 — `function l(f)`
-  `lxe` function L3 — `function lxe()`
-  `n` function L3 — `function n(f,d)`
-  `nxe` function L3 — `function nxe(t)`
-  `oxe` function L3 — `function oxe(t)`
-  `r` function L3 — `function r(f)`
-  `rxe` function L3 — `function rxe(t)`
-  `s` function L3 — `function s(l)`
-  `sxe` function L3 — `function sxe(t)`
-  `txe` function L3 — `function txe(t)`
-  `u` function L3 — `function u(f)`
-  `y` function L3 — `function y(...v)`
-  `Rt` function L9 — `function Rt(nt)`
-  `st` function L9 — `function st()`
-  `$xe` function L14 — `function $xe(t)`
-  `Cr` function L14 — `function Cr(t,e)`
-  `He` function L14 — `function He()`
-  `Iy` function L14 — `function Iy(t,e)`
-  `Ka` function L14 — `function Ka(t)`
-  `Qf` function L14 — `function Qf(t)`
-  `Tt` function L14 — `function Tt(At,Ce,tt)`
-  `r` function L14 — `function r()`
-  `sz` function L14-15 — `function sz()`
-  `zxe` function L14 — `function zxe(t)`
-  `$z` function L15 — `function $z(t)`
-  `Abe` function L15 — `function Abe(t,e)`
-  `C` function L15 — `function C()`
-  `G` function L15 — `function G(t,e,r,n,i,a)`
-  `Nbe` function L15 — `function Nbe(t)`
-  `Nt` function L15 — `function Nt(t)`
-  `P7` function L15 — `function P7(t,e,r)`
-  `Ql` function L15 — `function Ql(t)`
-  `Qz` function L15 — `function Qz(t)`
-  `SG` function L15 — `function SG(t,e,r)`
-  `W7` function L15 — `function W7(t)`
-  `_7` function L15 — `function _7(t,e)`
-  `_be` function L15 — `function _be(t)`
-  `a3` function L15 — `function a3(t,e)`
-  `abe` function L15 — `function abe(t)`
-  `bz` function L15 — `function bz(t)`
-  `cbe` function L15 — `function cbe(t)`
-  `d4e` function L15 — `function d4e(t,e,r)`
-  `fe` function L15 — `function fe(t,e)`
-  `gbe` function L15 — `function gbe(t)`
-  `gz` function L15 — `function gz(t,e,r,n,i)`
-  `hz` function L15 — `function hz(t)`
-  `k3` function L15 — `function k3(t,e)`
-  `p4e` function L15 — `function p4e(t)`
-  `ph` function L15 — `function ph(t,e,r)`
-  `q7` function L15 — `function q7(t)`
-  `rG` function L15 — `function rG(t,e)`
-  `rd` function L15 — `function rd(t)`
-  `tG` function L15 — `function tG(t,e)`
-  `w` function L15 — `function w()`
-  `w3` function L15 — `function w3(t)`
-  `wz` function L15 — `function wz(t)`
-  `xr` function L15 — `function xr(t,e)`
-  `z7` function L15 — `function z7(t)`
-  `n` function L269-270 — `function n()`
-  `C` function L270 — `function C(ae)`
-  `j4e` function L275 — `function j4e()`
-  `$` function L352 — `function $(K)`
-  `$0` function L352 — `function $0(t)`
-  `$5` function L352 — `function $5(t,e)`
-  `$5e` function L352 — `function $5e()`
-  `$6e` function L352 — `function $6e(t)`
-  `$A` function L352 — `function $A(t,e,r)`
-  `$Ee` function L352 — `function $Ee(t,e)`
-  `$Se` function L352 — `function $Se(t,e,r,n)`
-  `$Te` function L352 — `function $Te(t)`
-  `$ke` function L352 — `function $ke(t)`
-  `$n` function L352 — `function $n(t)`
-  `$we` function L352 — `function $we(t)`
-  `A` function L352 — `function A(K,X)`
-  `A5e` function L352 — `function A5e(t,e)`
-  `A8` function L352 — `function A8(t)`
-  `AA` function L352 — `function AA(t)`
-  `AH` function L352 — `function AH()`
-  `AW` function L352 — `function AW(t)`
-  `Ake` function L352 — `function Ake(t,e)`
-  `Av` function L352 — `function Av(t)`
-  `Awe` function L352 — `function Awe(t,e,r)`
-  `B` function L352 — `function B(K)`
-  `B0` function L352 — `function B0(t,e,r)`
-  `B3` function L352 — `function B3(t)`
-  `B5` function L352 — `function B5(t)`
-  `B5e` function L352 — `function B5e(t)`
-  `B6e` function L352 — `function B6e(t)`
-  `BA` function L352 — `function BA(t,e)`
-  `BEe` function L352 — `function BEe(t)`
-  `BW` function L352 — `function BW(t,e,r,n,i,a,s,l,u,h)`
-  `Bi` function L352 — `function Bi(t,e)`
-  `Bke` function L352 — `function Bke(t)`
-  `Bn` function L352 — `function Bn(t)`
-  `Bwe` function L352 — `function Bwe(t,e)`
-  `C0` function L352 — `function C0(t,e)`
-  `C5e` function L352 — `function C5e(t)`
-  `CA` function L352 — `function CA(t)`
-  `CEe` function L352 — `function CEe(t)`
-  `CH` function L352 — `function CH(t)`
-  `CU` function L352 — `function CU(t,e)`
-  `CW` function L352 — `function CW(t)`
-  `C_` function L352 — `function C_(t)`
-  `Cke` function L352 — `function Cke(t)`
-  `Cv` function L352 — `function Cv(t,e,r)`
-  `Cwe` function L352 — `function Cwe(t,e,r)`
-  `D` function L352 — `function D(K,X,te)`
-  `D0` function L352 — `function D0(t,e,r)`
-  `D5e` function L352 — `function D5e(t)`
-  `DA` function L352 — `function DA(t)`
-  `DEe` function L352 — `function DEe(t)`
-  `DW` function L352 — `function DW(t)`
-  `Dh` function L352 — `function Dh(t,e,r)`
-  `Dke` function L352 — `function Dke(t)`
-  `Do` function L352 — `function Do(t)`
-  `Dwe` function L352 — `function Dwe(t,e)`
-  `E` function L352 — `function E(K,X)`
-  `E5e` function L352 — `function E5e(t,e,r)`
-  `E6e` function L352 — `function E6e(t,e,r)`
-  `E8` function L352 — `function E8(t,e,r,n,i)`
-  `E9` function L352 — `function E9(t,e)`
-  `ECe` function L352 — `function ECe(t)`
-  `EEe` function L352 — `function EEe(t)`
-  `EU` function L352 — `function EU(t)`
-  `EW` function L352 — `function EW(t,e)`
-  `Eh` function L352 — `function Eh(t)`
-  `Eke` function L352 — `function Eke(t,e)`
-  `Ewe` function L352 — `function Ewe(t)`
-  `F` function L352 — `function F(K)`
-  `F0` function L352 — `function F0(t)`
-  `F3e` function L352 — `function F3e()`
-  `F5` function L352 — `function F5(t,e)`
-  `F5e` function L352 — `function F5e()`
-  `F6e` function L352 — `function F6e(t)`
-  `FA` function L352 — `function FA()`
-  `FEe` function L352 — `function FEe(t,e)`
-  `FSe` function L352 — `function FSe(t,e,r)`
-  `FU` function L352 — `function FU(t)`
-  `Fi` function L352 — `function Fi(t,e)`
-  `Fke` function L352 — `function Fke(t)`
-  `Fv` function L352 — `function Fv(t,e,r)`
-  `Fwe` function L352 — `function Fwe(t,e)`
-  `G0` function L352 — `function G0(t)`
-  `G3` function L352 — `function G3(t)`
-  `G3e` function L352 — `function G3e({_intern:t,_key:e},r)`
-  `G5e` function L352 — `function G5e()`
-  `G6e` function L352 — `function G6e(t)`
-  `GA` function L352 — `function GA()`
-  `GEe` function L352 — `function GEe()`
-  `GTe` function L352 — `function GTe(t,e,r)`
-  `Ge` function L352 — `function Ge(t)`
-  `Gke` function L352 — `function Gke(t)`
-  `Gv` function L352 — `function Gv(t)`
-  `Gwe` function L352 — `function Gwe(t)`
-  `H` function L352 — `function H(K)`
-  `H5` function L352 — `function H5(t,e)`
-  `H5e` function L352 — `function H5e(t)`
-  `HA` function L352 — `function HA()`
-  `HTe` function L352 — `function HTe(t,e,r)`
-  `HW` function L352 — `function HW(t)`
-  `Hke` function L352 — `function Hke(t)`
-  `Hwe` function L352 — `function Hwe(t,e,r)`
-  `I` function L352 — `function I(D)`
-  `I3` function L352 — `function I3(t,e,r)`
-  `I5` function L352 — `function I5()`
-  `I5e` function L352 — `function I5e(t)`
-  `I6e` function L352 — `function I6e(t,e)`
-  `IA` function L352 — `function IA()`
-  `IU` function L352 — `function IU(t)`
-  `IV` function L352 — `function IV(t,e)`
-  `I_` function L352 — `function I_(t,e,r,n)`
-  `Ike` function L352 — `function Ike(t,e)`
-  `Iv` function L352 — `function Iv(t)`
-  `Iwe` function L352 — `function Iwe(t,e)`
-  `J3e` function L352 — `function J3e(t,e)`
-  `J8` function L352 — `function J8(t,e)`
-  `JA` function L352 — `function JA(t,e)`
-  `JEe` function L352 — `function JEe(t,e)`
-  `JSe` function L352 — `function JSe(t,e,r,n,i,a,s)`
-  `JTe` function L352 — `function JTe(t,e,r)`
-  `JV` function L352 — `function JV()`
-  `J_` function L352 — `function J_(t)`
-  `Jy` function L352 — `function Jy(t)`
-  `K3` function L352 — `function K3(t,e,r,n,i,a)`
-  `K3e` function L352 — `function K3e(t,e)`
-  `K5e` function L352 — `function K5e()`
-  `K6e` function L352 — `function K6e()`
-  `K8` function L352 — `function K8(t)`
-  `KG` function L352 — `function KG({_intern:t,_key:e},r)`
-  `KSe` function L352 — `function KSe(t)`
-  `KTe` function L352 — `function KTe(t,e,r)`
-  `K_` function L352 — `function K_(t)`
-  `Ki` function L352 — `function Ki(t,e)`
-  `Kwe` function L352 — `function Kwe(t)`
-  `L` function L352 — `function L(K,X,te)`
-  `L0` function L352 — `function L0()`
-  `L3` function L352 — `function L3(t,e,r)`
-  `L5e` function L352 — `function L5e(t)`
-  `L6e` function L352 — `function L6e(t)`
-  `LA` function L352 — `function LA(t)`
-  `LSe` function L352 — `function LSe(t)`
-  `LW` function L352 — `function LW(t,e)`
-  `L_` function L352 — `function L_(t)`
-  `Lke` function L352 — `function Lke(t,e)`
-  `Lv` function L352 — `function Lv(t)`
-  `Lwe` function L352 — `function Lwe(t,e)`
-  `M` function L352 — `function M(K,X,te)`
-  `M3` function L352 — `function M3(t,e)`
-  `M5` function L352 — `function M5(t,e,r,n,i,a,s)`
-  `M5e` function L352 — `function M5e(t)`
-  `M8` function L352 — `function M8(t,e)`
-  `MU` function L352 — `function MU(t)`
-  `MV` function L352 — `function MV(t,e)`
-  `Mke` function L352 — `function Mke(t,e)`
-  `Mwe` function L352 — `function Mwe(t,e)`
-  `N3` function L352 — `function N3(t,e)`
-  `N5` function L352 — `function N5(t)`
-  `N5e` function L352 — `function N5e()`
-  `N6e` function L352 — `function N6e(t,e)`
-  `N8` function L352 — `function N8(t,e)`
-  `NA` function L352 — `function NA(t)`
-  `NV` function L352 — `function NV(t)`
-  `N_` function L352 — `function N_(t,e)`
-  `Nke` function L352 — `function Nke(t,e)`
-  `Nv` function L352 — `function Nv(t)`
-  `Nwe` function L352 — `function Nwe(t,e)`
-  `O` function L352 — `function O(K,X,te)`
-  `O5e` function L352 — `function O5e()`
-  `OSe` function L352 — `function OSe(t,e)`
-  `OX` function L352 — `function OX(t,e)`
-  `O_` function L352 — `function O_(t,e)`
-  `Oke` function L352 — `function Oke()`
-  `Owe` function L352 — `function Owe(t,e)`
-  `P` function L352 — `function P(K)`
-  `P0` function L352 — `function P0(t,e,r)`
-  `P5` function L352 — `function P5(t)`
-  `P5e` function L352 — `function P5e(t)`
-  `PA` function L352 — `function PA(t)`
-  `PU` function L352 — `function PU(t,e,r,n)`
-  `Pke` function L352 — `function Pke(t)`
-  `Pwe` function L352 — `function Pwe(t,e)`
-  `Q` function L352 — `function Q(K)`
-  `Q3e` function L352 — `function Q3e()`
-  `Q8` function L352 — `function Q8(t)`
-  `QA` function L352 — `function QA(t,e,r)`
-  `QEe` function L352 — `function QEe(t,e)`
-  `QSe` function L352 — `function QSe(t)`
-  `QTe` function L352 — `function QTe(t,e,r)`
-  `Q_` function L352 — `function Q_(t)`
-  `Qwe` function L352 — `function Qwe(t)`
-  `R` function L352 — `function R(K,X,te)`
-  `R0` function L352 — `function R0(t)`
-  `R3` function L352 — `function R3(t,e,r)`
-  `R5e` function L352 — `function R5e(t,e)`
-  `R6e` function L352 — `function R6e(t,e)`
-  `R8` function L352 — `function R8(t,e)`
-  `RA` function L352 — `function RA(t)`
-  `RSe` function L352 — `function RSe(t)`
-  `RV` function L352 — `function RV(t)`
-  `RW` function L352 — `function RW(t)`
-  `R_` function L352 — `function R_(t)`
-  `Rke` function L352 — `function Rke(t,e)`
-  `Rq` function L352 — `function Rq(t)`
-  `Rv` function L352 — `function Rv(t)`
-  `Rwe` function L352 — `function Rwe(t,e)`
-  `S` function L352 — `function S(K,X,te,J)`
-  `S5e` function L352 — `function S5e(t,e,r)`
-  `S6e` function L352 — `function S6e(t)`
-  `S8` function L352 — `function S8(t)`
-  `SW` function L352 — `function SW()`
-  `S_` function L352 — `function S_(t)`
-  `Sh` function L352 — `function Sh(t,e)`
-  `Ske` function L352 — `function Ske(t,e)`
-  `Swe` function L352 — `function Swe(t,e,r)`
-  `T` function L352 — `function T(E)`
-  `T0` function L352 — `function T0(t,e,r,n)`
-  `T5e` function L352 — `function T5e(t,e)`
-  `T8` function L352 — `function T8(t)`
-  `TU` function L352 — `function TU(t)`
-  `TW` function L352 — `function TW(t,e,r)`
-  `T_` function L352 — `function T_(t,e)`
-  `Th` function L352 — `function Th()`
-  `Tke` function L352 — `function Tke(t,e)`
-  `Twe` function L352 — `function Twe(t,e,r)`
-  `U0` function L352 — `function U0(t)`
-  `U3` function L352 — `function U3(t)`
-  `U3e` function L352 — `function U3e(t)`
-  `U5` function L352 — `function U5(t)`
-  `U5e` function L352 — `function U5e()`
-  `U8` function L352 — `function U8(t,e)`
-  `UA` function L352 — `function UA()`
-  `USe` function L352 — `function USe(t,e)`
-  `UTe` function L352 — `function UTe(t,e,r)`
-  `UU` function L352 — `function UU()`
-  `Uke` function L352 — `function Uke(t,e,r,n,i,a,s,l)`
-  `Uv` function L352 — `function Uv(t)`
-  `Uwe` function L352 — `function Uwe(t,e)`
-  `V0` function L352 — `function V0(t)`
-  `V3` function L352 — `function V3(t,e,r,n)`
-  `V3e` function L352 — `function V3e({_intern:t,_key:e},r)`
-  `V5` function L352 — `function V5(t)`
-  `V5e` function L352 — `function V5e()`
-  `V8` function L352 — `function V8(t)`
-  `VA` function L352 — `function VA(t)`
-  `VEe` function L352 — `function VEe(t)`
-  `VTe` function L352 — `function VTe(t,e,r)`
-  `Vke` function L352 — `function Vke(t)`
-  `Vv` function L352 — `function Vv(t)`
-  `W0` function L352 — `function W0(t)`
-  `W3` function L352 — `function W3(t)`
-  `W5e` function L352 — `function W5e(t)`
-  `W8` function L352 — `function W8(t,e)`
-  `W9` function L352 — `function W9(t,e)`
-  `WA` function L352 — `function WA()`
-  `WTe` function L352 — `function WTe(t,e,r)`
-  `Wke` function L352 — `function Wke(t)`
-  `Wr` function L352 — `function Wr(t,e,r)`
-  `Wwe` function L352 — `function Wwe(t,e,r)`
-  `X3` function L352 — `function X3(t,e)`
-  `X3e` function L352 — `function X3e(t)`
-  `X5e` function L352 — `function X5e(t,e)`
-  `X6e` function L352 — `function X6e(t)`
-  `X8` function L352 — `function X8(t)`
-  `XA` function L352 — `function XA(t)`
-  `XTe` function L352 — `function XTe(t,e,r)`
-  `XW` function L352 — `function XW(t)`
-  `X_` function L352 — `function X_(t)`
-  `Xwe` function L352 — `function Xwe(t,e,r)`
-  `Y3e` function L352 — `function Y3e(t)`
-  `Y5e` function L352 — `function Y5e(t,e,r)`
-  `Y6e` function L352 — `function Y6e(t)`
-  `Y8` function L352 — `function Y8(t)`
-  `Y9` function L352 — `function Y9(t)`
-  `YA` function L352 — `function YA()`
-  `YSe` function L352 — `function YSe(t)`
-  `YTe` function L352 — `function YTe(t,e,r)`
-  `YW` function L352 — `function YW(t)`
-  `Yke` function L352 — `function Yke(t)`
-  `Ywe` function L352 — `function Ywe(t,e,r)`
-  `Z8` function L352 — `function Z8(t)`
-  `Z9` function L352 — `function Z9(t)`
-  `ZEe` function L352 — `function ZEe()`
-  `ZSe` function L352 — `function ZSe(t)`
-  `ZTe` function L352 — `function ZTe(t,e,r)`
-  `Zs` function L352 — `function Zs()`
-  `Zwe` function L352 — `function Zwe(t)`
-  `Zy` function L352 — `function Zy(t,e,r)`
-  `_` function L352 — `function _(K,X,te)`
-  `_5` function L352 — `function _5()`
-  `_5e` function L352 — `function _5e(t,e)`
-  `_6e` function L352 — `function _6e(t,e)`
-  `_W` function L352 — `function _W(t,e)`
-  `_ke` function L352 — `function _ke(t,e)`
-  `_v` function L352 — `function _v(t)`
-  `_we` function L352 — `function _we(t,e,r)`
-  `a` function L352 — `function a(l,u,h=0,f=l.length)`
-  `a5e` function L352 — `function a5e(t)`
-  `a6e` function L352 — `function a6e(t)`
-  `a8` function L352 — `function a8()`
-  `aSe` function L352 — `function aSe(t)`
-  `aU` function L352 — `function aU(t)`
-  `a_` function L352 — `function a_(t,e,r)`
-  `ake` function L352 — `function ake(t,e)`
-  `aq` function L352 — `function aq(t,e)`
-  `awe` function L352 — `function awe()`
-  `b0` function L352 — `function b0(t,e)`
-  `b5e` function L352 — `function b5e(t,e)`
-  `b8` function L352 — `function b8(t)`
-  `bA` function L352 — `function bA(t)`
-  `bCe` function L352 — `function bCe(t)`
-  `bU` function L352 — `function bU(t,e)`
-  `bW` function L352 — `function bW(t,e,r)`
-  `bX` function L352 — `function bX(t,e,r,n,i)`
-  `b_` function L352 — `function b_(t)`
-  `bh` function L352 — `function bh(t,e)`
-  `bke` function L352 — `function bke(t,e)`
-  `bl` function L352 — `function bl()`
-  `bq` function L352 — `function bq(t)`
-  `bwe` function L352 — `function bwe(t,e,r)`
-  `c5` function L352 — `function c5(t)`
-  `c6e` function L352 — `function c6e(t)`
-  `c8` function L352 — `function c8(t)`
-  `cH` function L352 — `function cH(t)`
-  `cTe` function L352 — `function cTe()`
-  `c_` function L352 — `function c_()`
-  `cd` function L352 — `function cd(t)`
-  `cke` function L352 — `function cke(t)`
-  `cwe` function L352 — `function cwe(t,e,r)`
-  `d` function L352 — `function d()`
-  `d5e` function L352 — `function d5e(t,e,r,n,i,a,s)`
-  `d6e` function L352 — `function d6e(t)`
-  `dke` function L352 — `function dke(t,e)`
-  `dl` function L352 — `function dl(t,e,r,n)`
-  `du` function L352 — `function du(t,e)`
-  `dv` function L352 — `function dv()`
-  `dw` function L352 — `function dw(t,e)`
-  `dwe` function L352 — `function dwe(t)`
-  `e` function L352 — `function e(d,p)`
-  `e5e` function L352 — `function e5e(t,e)`
-  `e6e` function L352 — `function e6e(t,e)`
-  `e8` function L352 — `function e8(t)`
-  `eCe` function L352 — `function eCe(t)`
-  `eU` function L352 — `function eU()`
-  `e_` function L352 — `function e_()`
-  `eke` function L352 — `function eke(t,e,r)`
-  `es` function L352 — `function es(t,e,r,n)`
-  `f` function L352 — `function f(d,p)`
-  `f5e` function L352 — `function f5e(t,e,r,n,i,a)`
-  `f6e` function L352 — `function f6e(t)`
-  `f8` function L352 — `function f8(t,e)`
-  `fA` function L352 — `function fA(t,e)`
-  `fCe` function L352 — `function fCe(t)`
-  `fEe` function L352 — `function fEe(t)`
-  `f_` function L352 — `function f_(t)`
-  `fke` function L352 — `function fke(t)`
-  `fq` function L352 — `function fq(t)`
-  `fu` function L352 — `function fu(t,e,r,n)`
-  `fv` function L352 — `function fv()`
-  `fwe` function L352 — `function fwe(t)`
-  `g5` function L352 — `function g5(t,e)`
-  `g5e` function L352 — `function g5e(t,e)`
-  `g6e` function L352 — `function g6e()`
-  `g8` function L352 — `function g8(t,e,r)`
-  `gA` function L352 — `function gA(t)`
-  `gH` function L352 — `function gH(t)`
-  `gU` function L352 — `function gU(t)`
-  `gW` function L352 — `function gW(t,e,r,n,i,a)`
-  `g_` function L352 — `function g_(t)`
-  `gke` function L352 — `function gke(t,e)`
-  `gl` function L352 — `function gl()`
-  `gq` function L352 — `function gq(t,e)`
-  `gu` function L352 — `function gu()`
-  `gv` function L352 — `function gv(t,e)`
-  `gwe` function L352 — `function gwe()`
-  `h` function L352 — `function h(d)`
-  `h5` function L352 — `function h5(t)`
-  `h5e` function L352 — `function h5e(t)`
-  `h6e` function L352 — `function h6e(t)`
-  `h8` function L352 — `function h8(t,e,r)`
-  `hCe` function L352 — `function hCe(t,e,r)`
-  `h_` function L352 — `function h_(t)`
-  `ha` function L352 — `function ha(t,e)`
-  `he` function L352 — `function he(K)`
-  `hke` function L352 — `function hke(t,e)`
-  `i` function L352 — `function i(l,u,h=0,f=l.length)`
-  `i5e` function L352 — `function i5e()`
-  `i6e` function L352 — `function i6e(t)`
-  `i8` function L352 — `function i8()`
-  `iTe` function L352 — `function iTe(t)`
-  `iU` function L352 — `function iU(t,e,r,n)`
-  `iV` function L352 — `function iV(t,e)`
-  `i_` function L352 — `function i_(t,e,r)`
-  `ic` function L352 — `function ic(t,e,r,n)`
-  `ie` function L352 — `function ie(K)`
-  `ike` function L352 — `function ike(t,e)`
-  `iwe` function L352 — `function iwe()`
-  `j` function L352 — `function j(K)`
-  `j3e` function L352 — `function j3e(t)`
-  `j5e` function L352 — `function j5e(t,e)`
-  `j6e` function L352 — `function j6e(t)`
-  `j8` function L352 — `function j8(t)`
-  `jA` function L352 — `function jA(t,e)`
-  `jEe` function L352 — `function jEe(t)`
-  `jTe` function L352 — `function jTe(t,e,r)`
-  `jV` function L352 — `function jV()`
-  `j_` function L352 — `function j_(t)`
-  `jwe` function L352 — `function jwe(t)`
-  `k` function L352 — `function k(K,X,te)`
-  `k5e` function L352 — `function k5e(t)`
-  `k6e` function L352 — `function k6e(t,e,r)`
-  `k8` function L352 — `function k8(t,e,r,n)`
-  `kCe` function L352 — `function kCe(t,e,r)`
-  `kH` function L352 — `function kH(t)`
-  `kU` function L352 — `function kU(t)`
-  `kW` function L352 — `function kW(t,e)`
-  `kh` function L352 — `function kh(t,e)`
-  `kke` function L352 — `function kke(t,e)`
-  `kv` function L352 — `function kv()`
-  `kwe` function L352 — `function kwe(t)`
-  `l` function L352 — `function l(h,f,d,p)`
-  `l5e` function L352 — `function l5e()`
-  `l6e` function L352 — `function l6e()`
-  `l8` function L352 — `function l8()`
-  `lCe` function L352 — `function lCe(t)`
-  `lTe` function L352 — `function lTe(t,e,r)`
-  `lV` function L352 — `function lV()`
-  `l_` function L352 — `function l_()`
-  `ld` function L352 — `function ld(t)`
-  `le` function L352 — `function le(K)`
-  `lke` function L352 — `function lke(t,e)`
-  `lq` function L352 — `function lq(t,e)`
-  `lwe` function L352 — `function lwe(t)`
-  `m5` function L352 — `function m5(t)`
-  `m5e` function L352 — `function m5e(t)`
-  `m9` function L352 — `function m9(t,e,r)`
-  `mEe` function L352 — `function mEe(t)`
-  `md` function L352 — `function md(t,e)`
-  `mke` function L352 — `function mke(t,e)`
-  `ml` function L352 — `function ml(t)`
-  `mq` function L352 — `function mq(t,e,r)`
-  `mwe` function L352 — `function mwe()`
-  `n` function L352 — `function n(i,a)`
-  `n5e` function L352 — `function n5e()`
-  `n6e` function L352 — `function n6e(t)`
-  `n8` function L352 — `function n8(t)`
-  `nCe` function L352 — `function nCe(t)`
-  `nU` function L352 — `function nU()`
-  `n_` function L352 — `function n_()`
-  `ne` function L352 — `function ne(K)`
-  `nke` function L352 — `function nke(t,e)`
-  `nq` function L352 — `function nq(t)`
-  `nv` function L352 — `function nv(t)`
-  `o5` function L352 — `function o5(t)`
-  `o5e` function L352 — `function o5e(t)`
-  `o8` function L352 — `function o8(t,e)`
-  `o9` function L352 — `function o9(t)`
-  `oTe` function L352 — `function oTe(t,e,r)`
-  `oU` function L352 — `function oU(t,e,r,n)`
-  `oV` function L352 — `function oV(t,e,r)`
-  `o_` function L352 — `function o_(t)`
-  `od` function L352 — `function od(t)`
-  `oi` function L352 — `function oi(t,e)`
-  `oke` function L352 — `function oke(t,e)`
-  `oq` function L352 — `function oq(t,e)`
-  `p` function L352 — `function p(m)`
-  `p5e` function L352 — `function p5e(t)`
-  `p6e` function L352 — `function p6e(t,e)`
-  `pA` function L352 — `function pA(t)`
-  `pke` function L352 — `function pke(t,e)`
-  `pl` function L352 — `function pl(t)`
-  `pq` function L352 — `function pq(t)`
-  `pu` function L352 — `function pu(t,e,r,n,i,a)`
-  `pv` function L352 — `function pv(t,e,r)`
-  `q5e` function L352 — `function q5e(t)`
-  `q8` function L352 — `function q8(t,e)`
-  `qA` function L352 — `function qA()`
-  `qEe` function L352 — `function qEe(t)`
-  `qSe` function L352 — `function qSe(t,e)`
-  `qTe` function L352 — `function qTe(t,e,r)`
-  `qV` function L352 — `function qV(t,e,r)`
-  `qW` function L352 — `function qW(t)`
-  `qke` function L352 — `function qke(t)`
-  `qwe` function L352 — `function qwe(t,e)`
-  `r` function L352 — `function r(n)`
-  `r5e` function L352 — `function r5e(t)`
-  `r8` function L352 — `function r8(t)`
-  `r9` function L352 — `function r9(t)`
-  `rCe` function L352 — `function rCe(t,e,r)`
-  `rTe` function L352 — `function rTe(t)`
-  `rU` function L352 — `function rU()`
-  `r_` function L352 — `function r_(t)`
-  `ra` function L352 — `function ra(t,e)`
-  `rc` function L352 — `function rc(t)`
-  `rke` function L352 — `function rke(t,e)`
-  `rv` function L352 — `function rv(t,e)`
-  `s` function L352 — `function s(l,u,h=0,f=l.length)`
-  `s5` function L352 — `function s5()`
-  `s6e` function L352 — `function s6e(t,e)`
-  `s8` function L352 — `function s8(t)`
-  `s9` function L352 — `function s9(t,e)`
-  `sTe` function L352 — `function sTe(t,e)`
-  `sU` function L352 — `function sU(t)`
-  `s_` function L352 — `function s_(t)`
-  `sd` function L352 — `function sd(t,e)`
-  `ske` function L352 — `function ske(t,e)`
-  `sv` function L352 — `function sv(t,e,r,n)`
-  `swe` function L352 — `function swe()`
-  `t` function L352 — `function t(i,a,s,l,u,h,f)`
-  `t5` function L352 — `function t5(t,e,r)`
-  `t5e` function L352 — `function t5e(t)`
-  `t8` function L352 — `function t8(t,e)`
-  `tCe` function L352 — `function tCe(t,e,r)`
-  `tTe` function L352 — `function tTe(t,e)`
-  `tU` function L352 — `function tU(t)`
-  `t_` function L352 — `function t_(t)`
-  `tke` function L352 — `function tke(t,e,r)`
-  `tq` function L352 — `function tq(t)`
-  `tv` function L352 — `function tv(t)`
-  `u` function L352 — `function u(h,f,d,p,m,g)`
-  `u5` function L352 — `function u5(t,e)`
-  `u5e` function L352 — `function u5e()`
-  `u6e` function L352 — `function u6e(t,e)`
-  `u8` function L352 — `function u8(t)`
-  `uCe` function L352 — `function uCe(t,e)`
-  `uTe` function L352 — `function uTe(t)`
-  `ua` function L352 — `function ua(t,e,r,n)`
-  `uke` function L352 — `function uke(t,e)`
-  `uq` function L352 — `function uq(t,e)`
-  `v0` function L352 — `function v0(t)`
-  `v5` function L352 — `function v5(t,e)`
-  `v5e` function L352 — `function v5e(t)`
-  `v6e` function L352 — `function v6e(t)`
-  `vCe` function L352 — `function vCe(t,e)`
-  `vEe` function L352 — `function vEe(t)`
-  `v_` function L352 — `function v_(t)`
-  `vd` function L352 — `function vd(t)`
-  `vke` function L352 — `function vke(t)`
-  `vq` function L352 — `function vq(t)`
-  `w5e` function L352 — `function w5e(t,e)`
-  `w6e` function L352 — `function w6e(t,e)`
-  `w8` function L352 — `function w8(t)`
-  `wCe` function L352 — `function wCe(t)`
-  `wU` function L352 — `function wU(t,e)`
-  `wW` function L352 — `function wW(t,e,r)`
-  `wh` function L352 — `function wh(t,e,r)`
-  `wke` function L352 — `function wke(t,e)`
-  `wl` function L352 — `function wl(t,e)`
-  `wq` function L352 — `function wq(t)`
-  `wu` function L352 — `function wu(t)`
-  `wwe` function L352 — `function wwe(t,e)`
-  `x0` function L352 — `function x0(t)`
-  `x5e` function L352 — `function x5e(t,e)`
-  `x6e` function L352 — `function x6e(t)`
-  `x8` function L352 — `function x8(t)`
-  `xA` function L352 — `function xA(t)`
-  `xEe` function L352 — `function xEe(t)`
-  `xW` function L352 — `function xW(t,e,r)`
-  `xd` function L352 — `function xd(t)`
-  `xh` function L352 — `function xh(t)`
-  `xke` function L352 — `function xke(t,e)`
-  `xn` function L352 — `function xn(t,e,r,n)`
-  `xv` function L352 — `function xv(t,e)`
-  `y0` function L352 — `function y0(t,e,r)`
-  `y5` function L352 — `function y5(t,e)`
-  `y5e` function L352 — `function y5e(t)`
-  `y6e` function L352 — `function y6e(t)`
-  `y8` function L352 — `function y8(t)`
-  `y9` function L352 — `function y9(t)`
-  `yU` function L352 — `function yU(t)`
-  `y_` function L352 — `function y_(t,e)`
-  `yke` function L352 — `function yke(t,e)`
-  `yq` function L352 — `function yq(t)`
-  `yv` function L352 — `function yv(t,e)`
-  `ywe` function L352 — `function ywe()`
-  `z` function L352 — `function z(K)`
-  `z5` function L352 — `function z5(t,e)`
-  `z5e` function L352 — `function z5e()`
-  `z8` function L352 — `function z8(t)`
-  `zA` function L352 — `function zA(t)`
-  `zSe` function L352 — `function zSe(t,e)`
-  `zTe` function L352 — `function zTe(t,e,r)`
-  `zU` function L352 — `function zU()`
-  `zke` function L352 — `function zke(t)`
-  `zv` function L352 — `function zv(t)`
-  `zwe` function L352 — `function zwe(t,e,r)`
-  `Cl` function L353 — `function Cl(t,e,r,n,i)`
-  `GX` function L353 — `function GX(t,e,r,n,i)`
-  `$Ce` function L357 — `function $Ce(t)`
-  `GCe` function L357 — `function GCe(t,e)`
-  `UCe` function L357 — `function UCe(t)`
-  `VCe` function L357 — `function VCe(t,e)`
-  `hj` function L357-359 — `function hj(t,e)`
-  `uj` function L357 — `function uj(t)`
-  `zCe` function L357 — `function zCe(t)`
-  `KCe` function L359-363 — `function KCe(t,e)`
-  `nD` function L359 — `function nD(t,e)`
-  `o2` function L359 — `function o2(t,e)`
-  `rD` function L359 — `function rD(t,e,r,n,i)`
-  `$7e` function L363 — `function $7e(t)`
-  `A7e` function L363 — `function A7e(t,e)`
-  `B7e` function L363 — `function B7e(t)`
-  `E7e` function L363 — `function E7e(t)`
-  `F7e` function L363 — `function F7e(t)`
-  `H7e` function L363 — `function H7e(t)`
-  `J7e` function L363 — `function J7e(t)`
-  `Ls` function L363 — `function Ls(t)`
-  `M7e` function L363 — `function M7e(t)`
-  `N7e` function L363 — `function N7e(t)`
-  `Nd` function L363 — `function Nd(t)`
-  `O7e` function L363 — `function O7e(t)`
-  `QX` function L363 — `function QX(t)`
-  `R7e` function L363 — `function R7e(t)`
-  `S7e` function L363 — `function S7e(t)`
-  `W7e` function L363 — `function W7e(t)`
-  `X7e` function L363 — `function X7e(t)`
-  `Z7e` function L363 — `function Z7e(t)`
-  `ZX` function L363-364 — `function ZX(t)`
-  `_7e` function L363 — `function _7e(t)`
-  `aAe` function L363 — `function aAe(t)`
-  `aD` function L363 — `function aD(t)`
-  `am` function L363 — `function am(t)`
-  `b7e` function L363 — `function b7e(t)`
-  `c7e` function L363 — `function c7e()`
-  `d7e` function L363 — `function d7e(t)`
-  `dc` function L363 — `function dc(t)`
-  `e7e` function L363 — `function e7e(t)`
-  `f7e` function L363 — `function f7e(t)`
-  `g7e` function L363 — `function g7e(t)`
-  `j7e` function L363 — `function j7e(t)`
-  `jX` function L363 — `function jX(t,e)`
-  `l7e` function L363 — `function l7e(t)`
-  `n` function L363 — `function n(i)`
-  `oAe` function L363 — `function oAe(t)`
-  `p7e` function L363 — `function p7e(t)`
-  `r7e` function L363 — `function r7e()`
-  `sAe` function L363 — `function sAe(t)`
-  `t7e` function L363 — `function t7e(t,e)`
-  `u7e` function L363 — `function u7e(t)`
-  `v7e` function L363 — `function v7e(t)`
-  `w7e` function L363 — `function w7e(t)`
-  `x7e` function L363 — `function x7e(t)`
-  `y7e` function L363 — `function y7e(t)`
-  `z7e` function L363 — `function z7e(t)`
-  `Ci` function L364 — `function Ci(t,e,r)`
-  `Qt` function L364 — `function Qt(t,e)`
-  `Tj` function L364 — `function Tj(t,e)`
-  `bw` function L364 — `function bw(t,e)`
-  `cAe` function L364 — `function cAe(t,e)`
-  `ej` function L364 — `function ej(t,e,r,n)`
-  `fD` function L364-365 — `function fD(t,e)`
-  `hD` function L364 — `function hD(t)`
-  `kw` function L364 — `function kw(t)`
-  `lAe` function L364 — `function lAe(t)`
-  `sm` function L364 — `function sm(t,e,r,n,i,a,s,l,u)`
-  `zh` function L364 — `function zh(t,e,r,n)`
-  `dAe` function L365 — `function dAe(t,e)`
-  `fAe` function L365 — `function fAe(t,e)`
-  `hAe` function L365 — `function hAe(t,e)`
-  `pAe` function L365-371 — `function pAe(t,e)`
-  `uAe` function L365 — `function uAe(t,e,r)`
-  `gAe` function L371 — `function gAe(t)`
-  `kj` function L371-372 — `function kj(t,e)`
-  `mAe` function L371 — `function mAe(t,e,r)`
-  `om` function L371 — `function om(t,e,r,n,i)`
-  `tj` function L371 — `function tj(t,e)`
-  `vAe` function L371 — `function vAe(t)`
-  `xAe` function L371 — `function xAe(t)`
-  `yAe` function L371 — `function yAe(t)`
-  `UAe` function L372 — `function UAe(t)`
-  `VAe` function L372 — `function VAe(t,e)`
-  `WAe` function L372 — `function WAe(t)`
-  `bAe` function L372 — `function bAe(t,e,r)`
-  `rj` function L372-374 — `function rj(t,e)`
-  `wAe` function L372 — `function wAe(t,e)`
-  `oD` function L374-375 — `function oD(t,e)`
-  `KAe` function L375 — `function KAe(t,e,r,n,i)`
-  `Nj` function L375 — `function Nj(t)`
-  `Tw` function L375 — `function Tw(t)`
-  `XAe` function L375 — `function XAe(t)`
-  `YAe` function L375 — `function YAe(t)`
-  `aj` function L375-379 — `function aj(t,e)`
-  `ij` function L375 — `function ij(t,e,r)`
-  `jAe` function L375 — `function jAe(t,e,r,n,i,a,s,l)`
-  `nj` function L375 — `function nj(t)`
-  `qAe` function L375 — `function qAe(t,e)`
-  `s2` function L375 — `function s2(t,e)`
-  `u` function L375 — `function u(h)`
-  `u2` function L375 — `function u2(t)`
-  `sj` function L379-380 — `function sj(t)`
-  `QAe` function L380-383 — `function QAe(t,e)`
-  `oj` function L383-386 — `function oj(t,e)`
-  `Au` function L386 — `function Au(t,e,r,n,i,a,s)`
-  `JAe` function L386 — `function JAe(t,e,r)`
-  `ZAe` function L386 — `function ZAe(t)`
-  `cD` function L386 — `function cD(t,e,r)`
-  `cj` function L386 — `function cj(t,e,r)`
-  `e8e` function L386 — `function e8e(t,e,r)`
-  `lj` function L386 — `function lj(t,e,r,n)`
-  `n8e` function L386-387 — `function n8e(t,e)`
-  `r8e` function L386 — `function r8e(t,e)`
-  `t8e` function L386 — `function t8e(t,e,r,n)`
-  `pD` function L387 — `function pD(t,e)`
-  `$8e` function L388 — `function $8e(t,e)`
-  `$j` function L388 — `function $j(t,e)`
-  `Fj` function L388 — `function Fj(t)`
-  `Gj` function L388 — `function Gj(t)`
-  `f2` function L388 — `function f2(t,e,r)`
-  `nn` function L388 — `function nn(t,e="")`
-  `pc` function L388 — `function pc(t,e)`
-  `vD` function L388 — `function vD()`
-  `z8e` function L388-390 — `function z8e(t,e,r)`
-  `zj` function L388 — `function zj(t,e,r,n,i)`
-  `Jr` function L390 — `function Jr(t,e)`
-  `G8e` function L443-445 — `function G8e(t,{markdownAutoWrap:e})`
-  `Jj` function L445-446 — `function Jj(t,e={})`
-  `s` function L445-446 — `function s(l,u="normal")`
-  `U8e` function L446 — `function U8e(t,e)`
-  `V8e` function L446 — `function V8e(t)`
-  `eK` function L446 — `function eK(t,{markdownAutoWrap:e}={})`
-  `n` function L446 — `function n(i)`
-  `nK` function L446-447 — `function nK(t,e)`
-  `rK` function L446 — `function rK(t,e,r,n)`
-  `CD` function L447 — `function CD(t,e,r=[],n=[])`
-  `H8e` function L447-448 — `function H8e(t,e,r,n,i=!1)`
-  `aK` function L447 — `function aK(t,e)`
-  `$w` function L448 — `function $w(t,e,r,n,i)`
-  `AD` function L448 — `function AD(t,e,r)`
-  `AK` function L448 — `function AK(t,e,r,n,i,a,s,l,u,h)`
-  `BD` function L448 — `function BD(t,e=.15,r)`
-  `Bw` function L448 — `function Bw(t,e,r,n=1)`
-  `CK` function L448 — `function CK(t)`
-  `DD` function L448 — `function DD(t)`
-  `DK` function L448 — `function DK(t,e,r)`
-  `EK` function L448 — `function EK(t,e=0)`
-  `FK` function L448 — `function FK(t,e)`
-  `Fo` function L448 — `function Fo(t,e,r,n,i,a)`
-  `Fw` function L448 — `function Fw(t,e,r)`
-  `HD` function L448 — `function HD(t,e,r,n)`
-  `IK` function L448 — `function IK(t,e)`
-  `K8e` function L448 — `function K8e(t,e,r,n)`
-  `LK` function L448 — `function LK(t)`
-  `La` function L448 — `function La(t,e,r,n)`
-  `Lw` function L448 — `function Lw(t,e,r,n,i,a)`
-  `MD` function L448 — `function MD(t,e,r)`
-  `MK` function L448 — `function MK(t,e,r,n,i,a,s)`
-  `Mw` function L448 — `function Mw(t,e,r)`
-  `OD` function L448 — `function OD(t,e)`
-  `Od` function L448 — `function Od(t,e,r)`
-  `Ow` function L448 — `function Ow(t,e,r)`
-  `PD` function L448 — `function PD(t,e)`
-  `PK` function L448 — `function PK(t,e)`
-  `Pw` function L448 — `function Pw(t,e)`
-  `Q8e` function L448 — `function Q8e(t,e,r)`
-  `RK` function L448 — `function RK(t,e)`
-  `SK` function L448 — `function SK(t)`
-  `TK` function L448 — `function TK(t,e,r,n,i,a,s,l)`
-  `Uh` function L448 — `function Uh(t,e,r,n,i,a=!1)`
-  `W8e` function L448 — `function W8e(t,e,r)`
-  `WD` function L448 — `function WD(t,e,r,n,i,a,s)`
-  `X8e` function L448 — `function X8e(t,e,r,n)`
-  `Xt` function L448 — `function Xt(t)`
-  `Y8e` function L448 — `function Y8e(t,e)`
-  `_D` function L448 — `function _D(t,e)`
-  `_K` function L448 — `function _K(t,e,r,n,i)`
-  `a_e` function L448 — `function a_e(t,e,r,n,i,a,s,l)`
-  `bK` function L448 — `function bK(t,e)`
-  `dm` function L448 — `function dm(t,e)`
-  `e_e` function L448 — `function e_e(t,e)`
-  `g2` function L448 — `function g2(t,e,r)`
-  `hK` function L448 — `function hK(t,e)`
-  `i_e` function L448 — `function i_e(t,e,r,n,i)`
-  `j8e` function L448 — `function j8e(t,e,r)`
-  `jD` function L448 — `function jD(t)`
-  `kK` function L448 — `function kK(t,e,r,n,i,a,s,l,u)`
-  `l_e` function L448-449 — `function l_e(t)`
-  `nr` function L448 — `function nr(t,e,r=1)`
-  `o_e` function L448 — `function o_e(t,e)`
-  `q8e` function L448 — `function q8e(t,e,r,n=!1)`
-  `qD` function L448 — `function qD(t,e,r,n)`
-  `sK` function L448 — `function sK(t,e,r)`
-  `s_e` function L448 — `function s_e(t,e,r)`
-  `t_e` function L448 — `function t_e(t,e,r,n=1)`
-  `vK` function L448 — `function vK(t,e)`
-  `wK` function L448 — `function wK(t)`
-  `x` function L448 — `function x()`
-  `x2` function L448 — `function x2(t,e)`
-  `xK` function L448 — `function xK(t,e,r,n,i,a,s,l,u)`
-  `y2` function L448 — `function y2(t)`
-  `zK` function L448 — `function zK(t,e)`
-  `zw` function L448 — `function zw(t)`
-  `CQ` function L449 — `function CQ(t,e)`
-  `Du` function L449 — `function Du(t,e,r)`
-  `EQ` function L449 — `function EQ(t,e,{config:{flowchart:r}})`
-  `FQ` function L449 — `function FQ(t,e)`
-  `HK` function L449 — `function HK(t,e)`
-  `HQ` function L449 — `function HQ(t,e,{config:{themeVariables:r}})`
-  `Hh` function L449 — `function Hh(t,e,r,n=100,i=0,a=180)`
-  `IQ` function L449 — `function IQ(t,e)`
-  `KK` function L449 — `function KK(t,e)`
-  `LQ` function L449 — `function LQ(t,e)`
-  `NQ` function L449 — `function NQ(t,e)`
-  `PQ` function L449 — `function PQ(t,e)`
-  `Ra` function L449 — `function Ra(t,e,r,n=100,i=0,a=180)`
-  `TQ` function L449 — `function TQ(t,e,{config:{themeVariables:r,flowchart:n}})`
-  `VK` function L449 — `function VK(t,e)`
-  `VQ` function L449 — `function VQ(t,e)`
-  `Wh` function L449 — `function Wh(t,e,r,n=100,i=0,a=180)`
-  `XK` function L449 — `function XK(t,e)`
-  `ZK` function L449 — `function ZK(t,e)`
-  `_Q` function L449 — `function _Q(t,e)`
-  `bQ` function L449 — `function bQ(t,e,{config:{themeVariables:r,flowchart:n}})`
-  `eQ` function L449 — `function eQ(t,e)`
-  `fQ` function L449 — `function fQ(t,e)`
-  `gQ` function L449 — `function gQ(t,e,{config:{themeVariables:r,flowchart:n}})`
-  `iQ` function L449 — `function iQ(t,e,{config:{themeVariables:r}})`
-  `lQ` function L449 — `function lQ(t,e,{dir:r,config:{state:n,themeVariables:i}})`
-  `pQ` function L449 — `function pQ(t,e)`
-  `qK` function L449 — `function qK(t,e)`
-  `qQ` function L449-453 — `function qQ(t,e)`
-  `rQ` function L449 — `function rQ(t,e)`
-  `sQ` function L449 — `function sQ(t,e)`
-  `uQ` function L449 — `function uQ(t,e)`
-  `vQ` function L449 — `function vQ(t,e,{config:{themeVariables:r,flowchart:n}})`
-  `zQ` function L449 — `function zQ(t,e)`
-  `XQ` function L453 — `function XQ(t,e)`
-  `v_e` function L453-454 — `function v_e(t)`
-  `y_e` function L453 — `function y_e(t,e)`
-  `KQ` function L454 — `function KQ(t,e)`
-  `TZ` function L454 — `function TZ(t,e)`
-  `ZQ` function L454 — `function ZQ(t,e)`
-  `bZ` function L454 — `function bZ(t,e)`
-  `eZ` function L454 — `function eZ(t,e)`
-  `fZ` function L454 — `function fZ(t,e,{config:{themeVariables:r}})`
-  `gZ` function L454 — `function gZ(t,e)`
-  `iZ` function L454 — `function iZ(t,e)`
-  `lZ` function L454 — `function lZ(t,e)`
-  `pZ` function L454 — `function pZ(t,e)`
-  `rZ` function L454 — `function rZ(t,e)`
-  `sZ` function L454 — `function sZ(t,e)`
-  `uZ` function L454 — `function uZ(t,e,{config:{themeVariables:r}})`
-  `vZ` function L454 — `function vZ(t,e)`
-  `CZ` function L460 — `function CZ(t,e)`
-  `EZ` function L460 — `function EZ(t,e)`
-  `IZ` function L460-462 — `function IZ(t,e)`
-  `LZ` function L460 — `function LZ(t,e)`
-  `NZ` function L460 — `function NZ(t,e)`
-  `_Z` function L460 — `function _Z(t,e)`
-  `$Z` function L462 — `function $Z(t,e)`
-  `BZ` function L462 — `function BZ(t,e,r,n,i=r.class.padding??12)`
-  `GZ` function L462 — `function GZ(t,e)`
-  `KD` function L462 — `function KD(t,e)`
-  `Lu` function L462 — `function Lu(t,e,r,n="")`
-  `UZ` function L462 — `function UZ(t,e,{config:r})`
-  `Vw` function L462 — `function Vw(t,e,r,n=[])`
-  `WZ` function L462 — `function WZ(t)`
-  `b2` function L462 — `function b2(t,e,r,n=0,i=0,a=[],s="")`
-  `x` function L462 — `function x()`
-  `P_e` function L470 — `function P_e(t)`
-  `Ww` function L470 — `function Ww(t,e)`
-  `Yw` function L470 — `function Yw(t,e)`
-  `$Me` function L476 — `function $Me(t)`
-  `$Ne` function L476 — `function $Ne(t)`
-  `AIe` function L476 — `function AIe(t,e,r)`
-  `AMe` function L476 — `function AMe(t)`
-  `ARe` function L476 — `function ARe(t,e,r)`
-  `Ane` function L476 — `function Ane(t)`
-  `B9e` function L476 — `function B9e(t)`
-  `BDe` function L476 — `function BDe(t)`
-  `BIe` function L476 — `function BIe(t,e)`
-  `BLe` function L476 — `function BLe(t)`
-  `BMe` function L476 — `function BMe(t,e)`
-  `BNe` function L476 — `function BNe(t)`
-  `BRe` function L476 — `function BRe(t,e)`
-  `CIe` function L476 — `function CIe(t,e)`
-  `CNe` function L476 — `function CNe(t)`
-  `CRe` function L476 — `function CRe(t,e)`
-  `D2` function L476 — `function D2(t)`
-  `D9e` function L476 — `function D9e(t)`
-  `DIe` function L476 — `function DIe(t,e,r,n,i)`
-  `DMe` function L476 — `function DMe(t,e,r)`
-  `E9e` function L476 — `function E9e(t,e)`
-  `EIe` function L476 — `function EIe(t,e)`
-  `EMe` function L476 — `function EMe(t,e)`
-  `ENe` function L476 — `function ENe(t)`
-  `ERe` function L476 — `function ERe(t,e)`
-  `Ec` function L476 — `function Ec(t,e,r,n)`
-  `Ene` function L476 — `function Ene(t)`
-  `FIe` function L476 — `function FIe(t,e)`
-  `FRe` function L476 — `function FRe(t,e)`
-  `G9e` function L476 — `function G9e(t)`
-  `GLe` function L476 — `function GLe(t)`
-  `GMe` function L476 — `function GMe(t,e)`
-  `GNe` function L476 — `function GNe(t,e)`
-  `GRe` function L476 — `function GRe(t,e)`
-  `Gne` function L476 — `function Gne(t,e,r)`
-  `H9e` function L476 — `function H9e(t)`
-  `HL` function L476 — `function HL(t,e)`
-  `HLe` function L476 — `function HLe(t)`
-  `HMe` function L476 — `function HMe(t,e)`
-  `HNe` function L476 — `function HNe(t)`
-  `Hne` function L476 — `function Hne(t,e,r)`
-  `IDe` function L476 — `function IDe(t,e)`
-  `IIe` function L476 — `function IIe(t,e,r)`
-  `IMe` function L476 — `function IMe(t,e,r)`
-  `IRe` function L476 — `function IRe(t)`
-  `JIe` function L476 — `function JIe(t)`
-  `JMe` function L476 — `function JMe(t,e)`
-  `JNe` function L476 — `function JNe(t,e,r)`
-  `JRe` function L476 — `function JRe(t,e)`
-  `JT` function L476 — `function JT(t)`
-  `K9e` function L476 — `function K9e(t)`
-  `KIe` function L476 — `function KIe(t)`
-  `KL` function L476 — `function KL(t)`
-  `KLe` function L476 — `function KLe(t)`
-  `KMe` function L476 — `function KMe(t)`
-  `KNe` function L476 — `function KNe(t,e)`
-  `KRe` function L476 — `function KRe(t,e,r)`
-  `Kne` function L476 — `function Kne(t)`
-  `L2` function L476 — `function L2()`
-  `LIe` function L476 — `function LIe(t,e,r,n)`
-  `LNe` function L476 — `function LNe(t,e,r)`
-  `MDe` function L476 — `function MDe(t)`
-  `MIe` function L476 — `function MIe(t,e)`
-  `MRe` function L476 — `function MRe(t)`
-  `NIe` function L476 — `function NIe(t,e)`
-  `NRe` function L476 — `function NRe(t)`
-  `O9e` function L476 — `function O9e(t,e)`
-  `OIe` function L476 — `function OIe(t,e)`
-  `ONe` function L476 — `function ONe(t)`
-  `ORe` function L476 — `function ORe(t)`
-  `One` function L476 — `function One(t,e,r,n,i,a)`
-  `PDe` function L476 — `function PDe(t)`
-  `PIe` function L476 — `function PIe(t)`
-  `PJ` function L476 — `function PJ(t,e,r,n,i)`
-  `PMe` function L476 — `function PMe(t)`
-  `PRe` function L476 — `function PRe(t,e,r,n)`
-  `Q9e` function L476 — `function Q9e(t,e,r)`
-  `QIe` function L476 — `function QIe(t)`
-  `QL` function L476 — `function QL(t)`
-  `QLe` function L476 — `function QLe(t)`
-  `QNe` function L476 — `function QNe(t,e)`
-  `QRe` function L476 — `function QRe(t,e,r)`
-  `R2` function L476 — `function R2(t,e)`
-  `RIe` function L476 — `function RIe(t,e)`
-  `RRe` function L476 — `function RRe(t,e)`
-  `SIe` function L476 — `function SIe(t,e)`
-  `SNe` function L476 — `function SNe(t,e)`
-  `SRe` function L476 — `function SRe(t)`
-  `Sne` function L476 — `function Sne(t)`
-  `T9e` function L476 — `function T9e(t,e,r)`
-  `TIe` function L476 — `function TIe(t,e,r,n)`
-  `TMe` function L476 — `function TMe(t,e,r,n,i)`
-  `TNe` function L476 — `function TNe(t,e)`
-  `TRe` function L476 — `function TRe(t)`
-  `TT` function L476 — `function TT(t)`
-  `Tne` function L476 — `function Tne(t)`
-  `ULe` function L476 — `function ULe(t)`
-  `UMe` function L476 — `function UMe(t,e,r,n)`
-  `UNe` function L476 — `function UNe(t)`
-  `Une` function L476 — `function Une(t)`
-  `Uo` function L476 — `function Uo(t)`
-  `V9e` function L476 — `function V9e(t,e)`
-  `VNe` function L476 — `function VNe(t,e,r)`
-  `Vne` function L476 — `function Vne(t,e,r,n,i)`
-  `W9e` function L476 — `function W9e(t,e)`
-  `WL` function L476 — `function WL(t,e,r,n,i)`
-  `WLe` function L476 — `function WLe(t,e)`
-  `WNe` function L476 — `function WNe(t,e)`
-  `WRe` function L476 — `function WRe(t,e,r)`
-  `Wd` function L476 — `function Wd(t,e)`
-  `Wne` function L476 — `function Wne(t,e,r,n)`
-  `X9e` function L476 — `function X9e(t)`
-  `XIe` function L476 — `function XIe(t)`
-  `XL` function L476 — `function XL(t,e)`
-  `XMe` function L476 — `function XMe(t)`
-  `XNe` function L476 — `function XNe(t,e,r,n)`
-  `Xee` function L476 — `function Xee(t,e,r,n,i)`
-  `Xne` function L476 — `function Xne(t)`
-  `Y9e` function L476 — `function Y9e(t,e)`
-  `YIe` function L476 — `function YIe(t)`
-  `YMe` function L476 — `function YMe(t,e)`
-  `YNe` function L476 — `function YNe(t)`
-  `YRe` function L476 — `function YRe(t,e,r,n)`
-  `Yne` function L476 — `function Yne(t)`
-  `ZIe` function L476 — `function ZIe(t)`
-  `ZL` function L476 — `function ZL(t)`
-  `ZMe` function L476 — `function ZMe(t,e)`
-  `ZNe` function L476 — `function ZNe(t,e)`
-  `ZRe` function L476 — `function ZRe(t)`
-  `Zne` function L476 — `function Zne(t,e,r)`
-  `_2` function L476 — `function _2(t,e,r,n)`
-  `_9e` function L476 — `function _9e(t)`
-  `_Ie` function L476 — `function _Ie(t,e,r,n)`
-  `_Me` function L476 — `function _Me(t,e)`
-  `_Ne` function L476 — `function _Ne(t,e,r,n)`
-  `_Re` function L476 — `function _Re(t,e)`
-  `_ne` function L476 — `function _ne(t)`
-  `a` function L476 — `function a(s)`
-  `aLe` function L476 — `function aLe(t)`
-  `aNe` function L476 — `function aNe(t)`
-  `aOe` function L476 — `function aOe(t)`
-  `ane` function L476 — `function ane(t,e)`
-  `b9e` function L476 — `function b9e(t,e,r,n)`
-  `bDe` function L476 — `function bDe(t)`
-  `bIe` function L476 — `function bIe(t,e)`
-  `bMe` function L476 — `function bMe(t)`
-  `bT` function L476 — `function bT(t,e,r,n,i,a)`
-  `bie` function L476 — `function bie(t,e,r)`
-  `bne` function L476 — `function bne(t)`
-  `cDe` function L476 — `function cDe(t,e)`
-  `cNe` function L476 — `function cNe(t)`
-  `cR` function L476 — `function cR(t,e)`
-  `d9e` function L476 — `function d9e(t)`
-  `dMe` function L476 — `function dMe(t)`
-  `dNe` function L476 — `function dNe(t,e)`
-  `dR` function L476 — `function dR(t,e,r,n)`
-  `dne` function L476 — `function dne(t)`
-  `e` function L476 — `function e(r)`
-  `eIe` function L476 — `function eIe(t,e)`
-  `eMe` function L476 — `function eMe(t,e,r)`
-  `eNe` function L476 — `function eNe(t,e)`
-  `eOe` function L476 — `function eOe(t)`
-  `ef` function L476 — `function ef(t)`
-  `eie` function L476 — `function eie(t,e,r)`
-  `f` function L476 — `function f(d)`
-  `fIe` function L476 — `function fIe(t)`
-  `fNe` function L476 — `function fNe(t,e)`
-  `g9e` function L476 — `function g9e(t)`
-  `gIe` function L476 — `function gIe(t)`
-  `gie` function L476 — `function gie(t)`
-  `gne` function L476 — `function gne(t,e)`
-  `h` function L476 — `function h(d)`
-  `hDe` function L476 — `function hDe()`
-  `hIe` function L476 — `function hIe(t)`
-  `hNe` function L476 — `function hNe(t,e)`
-  `hR` function L476 — `function hR(t)`
-  `hRe` function L476 — `function hRe(t,e,r,n,i,a,s)`
-  `hie` function L476 — `function hie(t,e)`
-  `hne` function L476 — `function hne(t)`
-  `i` function L476 — `function i(a)`
-  `iNe` function L476 — `function iNe(t,e)`
-  `iOe` function L476 — `function iOe(t)`
-  `iR` function L476 — `function iR(t)`
-  `iie` function L476 — `function iie(t)`
-  `ine` function L476 — `function ine(t,e)`
-  `j9e` function L476 — `function j9e(t)`
-  `jIe` function L476 — `function jIe(t)`
-  `jL` function L476 — `function jL(t,e,r,n)`
-  `jLe` function L476 — `function jLe(t,e,r,n,i,a)`
-  `jMe` function L476 — `function jMe(t)`
-  `jNe` function L476 — `function jNe(t,e,r)`
-  `jRe` function L476 — `function jRe(t)`
-  `jne` function L476 — `function jne(t,e,r,n,i,a,s)`
-  `k9e` function L476 — `function k9e(t,e,r)`
-  `kIe` function L476 — `function kIe(t)`
-  `kMe` function L476 — `function kMe(t,e,r)`
-  `kRe` function L476 — `function kRe(t)`
-  `kie` function L476 — `function kie(t)`
-  `kne` function L476 — `function kne(t)`
-  `l9e` function L476 — `function l9e(t)`
-  `lDe` function L476 — `function lDe(t,e)`
-  `lIe` function L476 — `function lIe(t,e,r)`
-  `lJ` function L476 — `function lJ(t)`
-  `lLe` function L476 — `function lLe(t)`
-  `lR` function L476 — `function lR(t,e)`
-  `lie` function L476 — `function lie(t,e)`
-  `lne` function L476 — `function lne(t,e)`
-  `m9e` function L476 — `function m9e(t)`
-  `mDe` function L476 — `function mDe(t,e)`
-  `mIe` function L476 — `function mIe(t,e,r)`
-  `mMe` function L476 — `function mMe(t,e)`
-  `mR` function L476 — `function mR(t)`
-  `mRe` function L476 — `function mRe(t,e,r,n,i,a)`
-  `mie` function L476 — `function mie(t,e)`
-  `mne` function L476 — `function mne(t)`
-  `n` function L476 — `function n(a)`
-  `n9e` function L476 — `function n9e(t,e)`
-  `nLe` function L476 — `function nLe(t,e,r)`
-  `nNe` function L476 — `function nNe(t,e)`
-  `nOe` function L476 — `function nOe(t)`
-  `oDe` function L476 — `function oDe(t,e,r,n)`
-  `oIe` function L476 — `function oIe(t,e)`
-  `oNe` function L476 — `function oNe(t,e,r)`
-  `oOe` function L476 — `function oOe(t)`
-  `oR` function L476 — `function oR(t,e)`
-  `pIe` function L476 — `function pIe(t)`
-  `pMe` function L476 — `function pMe(t)`
-  `pNe` function L476 — `function pNe(t,e)`
-  `pR` function L476 — `function pR(t,e)`
-  `pie` function L476 — `function pie(t,e,r)`
-  `pne` function L476 — `function pne(t)`
-  `q9e` function L476 — `function q9e(t,e,r)`
-  `qL` function L476 — `function qL(t,e,r)`
-  `qLe` function L476 — `function qLe(t,e)`
-  `qMe` function L476 — `function qMe(t,e,r)`
-  `r` function L476 — `function r(n)`
-  `r9e` function L476 — `function r9e(t)`
-  `rNe` function L476 — `function rNe(t,e,r)`
-  `rOe` function L476 — `function rOe(t)`
-  `rf` function L476 — `function rf(t)`
-  `rie` function L476 — `function rie(t,e)`
-  `rk` function L476 — `function rk(t,e,r)`
-  `s9e` function L476 — `function s9e(t)`
-  `sDe` function L476 — `function sDe(t)`
-  `sIe` function L476 — `function sIe(t,e,r)`
-  `sOe` function L476 — `function sOe(t)`
-  `sie` function L476 — `function sie(t,e)`
-  `sne` function L476 — `function sne(t)`
-  `tIe` function L476 — `function tIe(t,e,r)`
-  `tNe` function L476 — `function tNe(t,e)`
-  `tOe` function L476 — `function tOe(t)`
-  `tk` function L476 — `function tk(t)`
-  `u` function L476 — `function u(d,p)`
-  `uDe` function L476 — `function uDe(t,e)`
-  `uIe` function L476 — `function uIe(t)`
-  `uNe` function L476 — `function uNe(t,e)`
-  `uR` function L476 — `function uR(t,e)`
-  `uie` function L476 — `function uie(t,e,r)`
-  `une` function L476 — `function une(t)`
-  `v9e` function L476 — `function v9e()`
-  `vDe` function L476 — `function vDe(t,e)`
-  `vIe` function L476 — `function vIe(t)`
-  `vNe` function L476 — `function vNe(t,e)`
-  `vRe` function L476 — `function vRe(t,e,r,n,i,a)`
-  `vie` function L476 — `function vie(t)`
-  `vm` function L476 — `function vm(t,e,r)`
-  `vne` function L476 — `function vne(t,e)`
-  `w9e` function L476 — `function w9e(t)`
-  `wDe` function L476 — `function wDe(t)`
-  `wIe` function L476 — `function wIe(t,e)`
-  `wNe` function L476 — `function wNe(t,e)`
-  `wRe` function L476 — `function wRe(t,e,r,n)`
-  `wie` function L476 — `function wie(t)`
-  `x9e` function L476 — `function x9e(t,e)`
-  `xDe` function L476 — `function xDe(t,e,r)`
-  `xIe` function L476 — `function xIe(t,e)`
-  `xMe` function L476 — `function xMe(t,e,r,n)`
-  `xne` function L476 — `function xne(t,e,r,n,i,a)`
-  `yIe` function L476 — `function yIe(t,e)`
-  `yne` function L476 — `function yne(t,e)`
-  `zLe` function L476 — `function zLe(t)`
-  `zMe` function L476 — `function zMe(t,e,r)`
-  `zNe` function L476 — `function zNe(t,e)`
-  `zRe` function L476 — `function zRe(t,e,r,n)`
-  `zne` function L476 — `function zne(t,e,r)`
-  `P2e` function L479 — `function P2e()`
-  `wnt` function L479 — `function wnt(Ws)`
-  `Sn` function L484 — `function Sn()`
-  `q` function L631 — `function q()`
-  `xe` function L631 — `function xe(ct)`
-  `te` function L636 — `function te()`
-  `$R` function L690 — `function $R(t)`
-  `AR` function L690 — `function AR(t,e)`
-  `Ag` function L690 — `function Ag(t)`
-  `BOe` function L690 — `function BOe(t,e)`
-  `BR` function L690 — `function BR(t)`
-  `CR` function L690 — `function CR(t,e)`
-  `DR` function L690 — `function DR(t)`
-  `ER` function L690 — `function ER(t,e)`
-  `FOe` function L690 — `function FOe(t,e)`
-  `FR` function L690 — `function FR(t)`
-  `G2` function L690 — `function G2(t)`
-  `GOe` function L690 — `function GOe(t)`
-  `GR` function L690 — `function GR(t)`
-  `Gm` function L690 — `function Gm(t)`
-  `H2` function L690 — `function H2(t)`
-  `HOe` function L690 — `function HOe(t)`
-  `HR` function L690 — `function HR(t)`
-  `Ho` function L690 — `function Ho(t)`
-  `IOe` function L690 — `function IOe(t,e,r=lk)`
-  `IR` function L690 — `function IR(t)`
-  `Il` function L690 — `function Il(t)`
-  `JOe` function L690 — `function JOe(t)`
-  `KOe` function L690 — `function KOe(t)`
-  `Kd` function L690 — `function Kd(t)`
-  `LR` function L690 — `function LR(t)`
-  `Lc` function L690 — `function Lc(t)`
-  `Ll` function L690 — `function Ll(t)`
-  `M2` function L690 — `function M2(t)`
-  `MOe` function L690 — `function MOe(t)`
-  `MR` function L690 — `function MR(t)`
-  `Ml` function L690 — `function Ml(t)`
-  `Mu` function L690 — `function Mu(t)`
-  `NOe` function L690 — `function NOe(t)`
-  `NR` function L690 — `function NR(t)`
-  `Nc` function L690 — `function Nc(t,e)`
-  `OOe` function L690 — `function OOe(t,e=!0)`
-  `OR` function L690 — `function OR(t)`
-  `Oa` function L690 — `function Oa(t)`
-  `POe` function L690 — `function POe(t)`
-  `PR` function L690 — `function PR(t)`
-  `Pa` function L690 — `function Pa(t)`
-  `QOe` function L690 — `function QOe(t)`
-  `Qd` function L690 — `function Qd(t)`
-  `RR` function L690 — `function RR(t)`
-  `SR` function L690 — `function SR(t,e)`
-  `UOe` function L690 — `function UOe(t)`
-  `UR` function L690 — `function UR(t)`
-  `V2` function L690 — `function V2(t)`
-  `VOe` function L690 — `function VOe(t)`
-  `VR` function L690 — `function VR(t)`
-  `W2` function L690 — `function W2(t,e)`
-  `WOe` function L690 — `function WOe(t)`
-  `WR` function L690 — `function WR(t)`
-  `Wo` function L690 — `function Wo(t,e)`
-  `XOe` function L690 — `function XOe(t)`
-  `XR` function L690 — `function XR(t,e)`
-  `YOe` function L690 — `function YOe(t)`
-  `YR` function L690 — `function YR(t,e)`
-  `ZOe` function L690 — `function ZOe(t)`
-  `_R` function L690 — `function _R(t,e)`
-  `_g` function L690 — `function _g(t,e)`
-  `af` function L690 — `function af(t)`
-  `ar` function L690 — `function ar(t)`
-  `bk` function L690 — `function bk(t,e)`
-  `cae` function L690 — `function cae(t,e)`
-  `dae` function L690 — `function dae(t)`
-  `dk` function L690 — `function dk(t)`
-  `e` function L690 — `function e(a)`
-  `ePe` function L690 — `function ePe(t,e)`
-  `en` function L690 — `function en(...t)`
-  `ep` function L690 — `function ep(t)`
-  `fae` function L690 — `function fae(t,e=!0)`
-  `fk` function L690 — `function fk(t)`
-  `gk` function L690 — `function gk(t)`
-  `hae` function L690 — `function hae(t,e,r)`
-  `i` function L690 — `function i(a)`
-  `ii` function L690 — `function ii(t)`
-  `jOe` function L690 — `function jOe(t)`
-  `jR` function L690 — `function jR(t)`
-  `jd` function L690 — `function jd(t)`
-  `kR` function L690 — `function kR(t)`
-  `lae` function L690 — `function lae(t)`
-  `mk` function L690 — `function mk(t)`
-  `n` function L690 — `function n(a)`
-  `ok` function L690 — `function ok(t)`
-  `pae` function L690 — `function pae(t)`
-  `pk` function L690 — `function pk(t)`
-  `q2` function L690 — `function q2()`
-  `qOe` function L690 — `function qOe(t)`
-  `qR` function L690 — `function qR(t,e)`
-  `r` function L690 — `function r(a)`
-  `rp` function L690 — `function rp(t)`
-  `sf` function L690 — `function sf(t)`
-  `so` function L690 — `function so(t)`
-  `tPe` function L690 — `function tPe(t,e=Pa(t).parseResult.value)`
-  `tp` function L690 — `function tp(t,e)`
-  `uae` function L690 — `function uae(t,e)`
-  `va` function L690 — `function va(t)`
-  `vk` function L690 — `function vk(t)`
-  `yk` function L690 — `function yk(t)`
-  `zOe` function L690 — `function zOe(t)`
-  `zR` function L690 — `function zR(t)`
-  `Dg` function L698 — `function Dg(t)`
-  `ap` function L698 — `function ap(t)`
-  `bae` function L698 — `function bae(t)`
-  `eN` function L698 — `function eN(t)`
-  `i` function L698 — `function i()`
-  `iPe` function L698 — `function iPe(t)`
-  `l` function L698 — `function l(h)`
-  `rN` function L698 — `function rN(t,e)`
-  `tN` function L698 — `function tN(t)`
-  `u` function L698 — `function u(h)`
-  `$ae` function L701-709 — `function $ae(t,e=!1)`
-  `Aae` function L701 — `function Aae(t)`
-  `Bg` function L701 — `function Bg(t)`
-  `Bs` function L701 — `function Bs(t)`
-  `Cae` function L701 — `function Cae(t,e)`
-  `Eae` function L701 — `function Eae(t,e,r)`
-  `Iae` function L701 — `function Iae(t)`
-  `Ig` function L701 — `function Ig(t)`
-  `Iu` function L701 — `function Iu(t,e)`
-  `J2` function L701 — `function J2(t)`
-  `K2` function L701 — `function K2(t,e)`
-  `Mae` function L701 — `function Mae(t,e,r)`
-  `Mg` function L701 — `function Mg(t,e)`
-  `Ng` function L701 — `function Ng(t)`
-  `Pae` function L701 — `function Pae()`
-  `Pg` function L701 — `function Pg(t)`
-  `Q2` function L701 — `function Q2(t,e,r)`
-  `Rg` function L701 — `function Rg(t)`
-  `Sae` function L701 — `function Sae(t)`
-  `Sk` function L701 — `function Sk(t)`
-  `Tae` function L701 — `function Tae(t)`
-  `Z2` function L701 — `function Z2(t)`
-  `aN` function L701 — `function aN(t)`
-  `aPe` function L701 — `function aPe(t,e)`
-  `bPe` function L701 — `function bPe(t)`
-  `cN` function L701 — `function cN(t,e,r)`
-  `cPe` function L701 — `function cPe(t)`
-  `dPe` function L701 — `function dPe(t)`
-  `e` function L701 — `function e()`
-  `ex` function L701 — `function ex(t)`
-  `fN` function L701 — `function fN(t)`
-  `fPe` function L701 — `function fPe(t)`
-  `gPe` function L701 — `function gPe(t)`
-  `hN` function L701 — `function hN(t)`
-  `hPe` function L701 — `function hPe(t)`
-  `i` function L701 — `function i(a,s)`
-  `iN` function L701 — `function iN(t,e)`
-  `kae` function L701 — `function kae(t,e,r)`
-  `kk` function L701 — `function kk(t)`
-  `lN` function L701 — `function lN(t,e,r,n)`
-  `lPe` function L701 — `function lPe(t)`
-  `mPe` function L701 — `function mPe(t)`
-  `n` function L701 — `function n()`
-  `nN` function L701 — `function nN(t)`
-  `oN` function L701 — `function oN(t,e)`
-  `oPe` function L701 — `function oPe(t)`
-  `op` function L701 — `function op(t)`
-  `pN` function L701 — `function pN(t)`
-  `pPe` function L701 — `function pPe(t)`
-  `rx` function L701 — `function rx(t)`
-  `sN` function L701 — `function sN(t)`
-  `sPe` function L701 — `function sPe(t,e)`
-  `sp` function L701 — `function sp(t,e=[])`
-  `tx` function L701 — `function tx(t)`
-  `uN` function L701 — `function uN(t,e,r)`
-  `uPe` function L701 — `function uPe(t)`
-  `vN` function L701 — `function vN(t)`
-  `vPe` function L701 — `function vPe(t)`
-  `wPe` function L701 — `function wPe(t,e)`
-  `wae` function L701 — `function wae(t)`
-  `xPe` function L701 — `function xPe(t)`
-  `yN` function L701 — `function yN(t)`
-  `yPe` function L701 — `function yPe(t)`
-  `Bae` function L709 — `function Bae(t,e)`
-  `Nk` function L709 — `function Nk(t,e)`
-  `Rk` function L709 — `function Rk(t,e,r)`
-  `TN` function L709 — `function TN(t,e,r)`
-  `kN` function L709 — `function kN(t)`
-  `kPe` function L709 — `function kPe(t,e)`
-  `Uae` function L710-716 — `function Uae(t,e)`
-  `CPe` function L716 — `function CPe(t)`
-  `EPe` function L716 — `function EPe(t)`
-  `Hae` function L716 — `function Hae(t,e)`
-  `SPe` function L716 — `function SPe(t)`
-  `_Pe` function L716-718 — `function _Pe(t)`
-  `e` class L716 — `-`
-  `DPe` function L718 — `function DPe(t)`
-  `RPe` function L718-720 — `function RPe(t)`
-  `e` class L718 — `-`
-  `IPe` function L720 — `function IPe(t)`
-  `MPe` function L720 — `function MPe(t)`
-  `NPe` function L720 — `function NPe(t)`
-  `OPe` function L720 — `function OPe(t,e)`
-  `PPe` function L720-722 — `function PPe(t)`
-  `BPe` function L722 — `function BPe(t,e)`
-  `FPe` function L722 — `function FPe(t)`
-  `Gae` function L722 — `function Gae(t)`
-  `Vae` function L722 — `function Vae(t)`
-  `Wae` function L722-727 — `function Wae(t,e,r)`
-  `qae` function L727-731 — `function qae(t,e,r)`
-  `$Pe` function L731 — `function $Pe(t)`
-  `Kae` function L731 — `function Kae(t,e)`
-  `Xae` function L731 — `function Xae(t)`
-  `Yae` function L731 — `function Yae(t)`
-  `zPe` function L731-736 — `function zPe(t,e)`
-  `Bu` function L736 — `function Bu(t)`
-  `CN` function L736 — `function CN(t,e,r)`
-  `GPe` function L736 — `function GPe()`
-  `HPe` function L736 — `function HPe(t)`
-  `Ic` function L736 — `function Ic(t)`
-  `Jae` function L736 — `function Jae(t)`
-  `Pu` function L736 — `function Pu(t,e)`
-  `Qae` function L736 — `function Qae(t)`
-  `UPe` function L736 — `function UPe(t)`
-  `VPe` function L736 — `function VPe(t)`
-  `WPe` function L736 — `function WPe(t)`
-  `YPe` function L736 — `function YPe(t)`
-  `_N` function L736 — `function _N(t)`
-  `qPe` function L736 — `function qPe(t)`
-  `rse` function L736 — `function rse(t)`
-  `tse` function L736 — `function tse(t,e)`
-  `zg` function L736 — `function zg(t,e)`
-  `$` function L745 — `function $()`
-  `Fu` function L745 — `function Fu(t)`
-  `H` function L745 — `function H(le)`
-  `LN` function L745 — `function LN(t)`
-  `j` function L745 — `function j(le)`
-  `jPe` function L745-746 — `function jPe(t)`
-  `of` function L745 — `function of(t)`
-  `$u` function L746 — `function $u(t,e,r,n,i,a,s,l)`
-  `sx` function L746 — `function sx(t,e)`
-  `r` function L752 — `function r(f)`
-  `$k` function L776 — `function $k(t,e,r,n)`
-  `$se` function L776 — `function $se(t,e,r)`
-  `Ase` function L776 — `function Ase(t,e,r)`
-  `BN` function L776 — `function BN(t,e,r,n=[])`
-  `Cse` function L776 — `function Cse(t,e)`
-  `Dse` function L776 — `function Dse(t)`
-  `Ese` function L776 — `function Ese(t)`
-  `Fk` function L776 — `function Fk(t,e,r=[])`
-  `Fse` function L776 — `function Fse(t,e,r)`
-  `Gk` function L776 — `function Gk(t)`
-  `Hg` function L776 — `function Hg(t,e,r,n)`
-  `JPe` function L776 — `function JPe(t)`
-  `KPe` function L776 — `function KPe(t,e,r,n)`
-  `Lse` function L776 — `function Lse(t)`
-  `MN` function L776 — `function MN(t)`
-  `QPe` function L776 — `function QPe(t,e,r)`
-  `Sse` function L776 — `function Sse(t,e,r)`
-  `Tse` function L776 — `function Tse(t)`
-  `Uk` function L776 — `function Uk(t)`
-  `Vk` function L776 — `function Vk(t,e)`
-  `Vse` function L776-780 — `function Vse(t,e)`
-  `Wg` function L776 — `function Wg(t,e,r,n)`
-  `Wk` function L776 — `function Wk(t,e,r)`
-  `XN` function L776 — `function XN(t,e)`
-  `YN` function L776 — `function YN(t,e)`
-  `ZPe` function L776 — `function ZPe(t,e)`
-  `_se` function L776 — `function _se(t,e,r)`
-  `a` function L776 — `function a(l)`
-  `aBe` function L776 — `function aBe(t,e,r,n,i,a,s)`
-  `bse` function L776 — `function bse(t,e)`
-  `eBe` function L776 — `function eBe(t,e,r,n)`
-  `fse` function L776 — `function fse(t,e)`
-  `gse` function L776 — `function gse(t,e,r,n,i,a)`
-  `iBe` function L776 — `function iBe(t,e,r)`
-  `kse` function L776 — `function kse(t,e,r,n)`
-  `lBe` function L776 — `function lBe(t,e)`
-  `lf` function L776 — `function lf(t)`
-  `mse` function L776 — `function mse(t,e,r,n,i,a)`
-  `nBe` function L776 — `function nBe(t,e,r,n)`
-  `pse` function L776 — `function pse(t)`
-  `qN` function L776 — `function qN(t,e)`
-  `rBe` function L776 — `function rBe(t,e,r,n)`
-  `s` function L776 — `function s(l)`
-  `sBe` function L776 — `function sBe(t)`
-  `tBe` function L776 — `function tBe(t,e)`
-  `ux` function L776 — `function ux(t)`
-  `vse` function L776 — `function vse(t,e,r)`
-  `wse` function L776 — `function wse(t)`
-  `xse` function L776 — `function xse(t,e)`
-  `yse` function L776 — `function yse(t,e,r,n)`
-  `Use` function L780 — `function Use(t,e,r)`
-  `cBe` function L780 — `function cBe(t,e)`
-  `uBe` function L780 — `function uBe(t,e)`
-  `Jse` function L787 — `function Jse(t)`
-  `aE` function L787-788 — `function aE(t)`
-  `dBe` function L787 — `function dBe(t,e)`
-  `yx` function L787 — `function yx(t,e,r,n=!1)`
-  `ioe` function L794 — `function ioe(t,e)`
-  `lE` function L794 — `function lE(t=void 0)`
-  `ABe` function L800 — `function ABe(t,e,r,n)`
-  `Ai` function L800 — `function Ai(t,e)`
-  `BBe` function L800 — `function BBe(t,e,r,n)`
-  `CBe` function L800 — `function CBe(t,e,r)`
-  `DBe` function L800 — `function DBe(t,e)`
-  `EBe` function L800 — `function EBe(t,e,r)`
-  `FBe` function L800-803 — `function FBe(t)`
-  `IBe` function L800 — `function IBe(t)`
-  `JN` function L800 — `function JN(t,e=!0)`
-  `LBe` function L800 — `function LBe(t,e,r)`
-  `MBe` function L800 — `function MBe(t,e)`
-  `NBe` function L800 — `function NBe(t,e)`
-  `OBe` function L800 — `function OBe(t,e,r,n)`
-  `PBe` function L800 — `function PBe(t,e,r,n,i,a)`
-  `QN` function L800 — `function QN(t,e,r,n)`
-  `RBe` function L800 — `function RBe(t,e,r)`
-  `SBe` function L800 — `function SBe(t,e,r)`
-  `TBe` function L800 — `function TBe(t,e,r)`
-  `ZN` function L800 — `function ZN(t,e)`
-  `_Be` function L800 — `function _Be(t)`
-  `aa` function L800 — `function aa(t,e,r,n)`
-  `bBe` function L800 — `function bBe(t,e)`
-  `boe` function L800 — `function boe(t,e=!0)`
-  `doe` function L800 — `function doe(t)`
-  `dp` function L800 — `function dp(t,e,r)`
-  `e1` function L800 — `function e1(t,e,r,n,...i)`
-  `eM` function L800 — `function eM(t,e,r,n)`
-  `fp` function L800 — `function fp(t,e,r)`
-  `goe` function L800 — `function goe(t,e,r,n,i)`
-  `hf` function L800 — `function hf(t,e)`
-  `kBe` function L800 — `function kBe(t,e,r)`
-  `moe` function L800 — `function moe(t,e,r,n,i)`
-  `poe` function L800 — `function poe(t,e,r)`
-  `wBe` function L800 — `function wBe(t,e,r)`
-  `$Be` function L803 — `function $Be(t)`
-  `GBe` function L803 — `function GBe(t,e)`
-  `HBe` function L803 — `function HBe(t,e)`
-  `KBe` function L803 — `function KBe(t)`
-  `QBe` function L803 — `function QBe(t)`
-  `Toe` function L803 — `function Toe(t)`
-  `UBe` function L803 — `function UBe(t,e)`
-  `VBe` function L803 — `function VBe(t,e,r)`
-  `WBe` function L803 — `function WBe(t)`
-  `XBe` function L803 — `function XBe(t)`
-  `YBe` function L803 — `function YBe(t)`
-  `ZBe` function L803 — `function ZBe(t)`
-  `e` function L803 — `function e(r)`
-  `i` function L803 — `function i(a)`
-  `jBe` function L803 — `function jBe(t)`
-  `koe` function L803 — `function koe(t,e)`
-  `n` function L803 — `function n(a)`
-  `qBe` function L803 — `function qBe(t,e)`
-  `r` function L803 — `function r(n)`
-  `uE` function L803 — `function uE(t,e)`
-  `woe` function L803 — `function woe(t,e,r,n)`
-  `zBe` function L803 — `function zBe(t,e,r)`
-  `Dle` function L805 — `function Dle(t,e,r=e.terminal)`
-  `Lle` function L805 — `function Lle(t,e,r,n)`
-  `Nle` function L805 — `function Nle(t)`
-  `Rle` function L805 — `function Rle(t,e)`
-  `Rx` function L805 — `function Rx(t,e,r)`
-  `TM` function L805 — `function TM(t)`
-  `Vu` function L805 — `function Vu(t)`
-  `a` function L805 — `function a(m)`
-  `aFe` function L805 — `function aFe(t,e)`
-  `bM` function L805 — `function bM(t)`
-  `d` function L805 — `function d(m)`
-  `e` function L805 — `function e(h)`
-  `eFe` function L805 — `function eFe(t,e)`
-  `f` function L805 — `function f(m)`
-  `gp` function L805 — `function gp(t,e,r=!1)`
-  `h` function L805 — `function h(m)`
-  `i` function L805 — `function i(m)`
-  `iFe` function L805 — `function iFe(t,e)`
-  `l` function L805 — `function l(m,g,y)`
-  `lFe` function L805 — `function lFe(t,e)`
-  `n` function L805 — `function n(m)`
-  `nFe` function L805 — `function nFe(t,e)`
-  `oFe` function L805 — `function oFe(t,e)`
-  `p` function L805 — `function p(m,g)`
-  `r` function L805 — `function r(m)`
-  `rFe` function L805 — `function rFe(t,e)`
-  `s` function L805 — `function s(m)`
-  `sFe` function L805 — `function sFe(t,e)`
-  `tFe` function L805 — `function tFe(t,e)`
-  `u` function L805 — `function u(m)`
-  `xE` function L805 — `function xE(t)`
-  `xM` function L805 — `function xM(t,e)`
-  `yM` function L805 — `function yM(t)`
-  `$le` function L806 — `function $le(t)`
-  `AFe` function L806 — `function AFe(t,e)`
-  `Bc` function L806 — `function Bc(t)`
-  `C` function L806 — `function C(D)`
-  `CE` function L806 — `function CE()`
-  `CFe` function L806 — `function CFe(t)`
-  `CM` function L806 — `function CM()`
-  `DFe` function L806 — `function DFe(t)`
-  `E` function L806 — `function E(D)`
-  `Gle` function L806 — `function Gle(t)`
-  `IE` function L806 — `function IE(t)`
-  `IM` function L806 — `function IM(t,e)`
-  `Ile` function L806 — `function Ile(t)`
-  `JM` function L806 — `function JM(t)`
-  `Jle` function L806 — `function Jle(t,e)`
-  `Kle` function L806-810 — `function Kle(t)`
-  `LFe` function L806 — `function LFe(t,e,r)`
-  `MM` function L806 — `function MM()`
-  `Mle` function L806 — `function Mle(t)`
-  `Qle` function L806 — `function Qle(t)`
-  `RE` function L806 — `function RE(t)`
-  `RFe` function L806 — `function RFe(t,e)`
-  `SFe` function L806 — `function SFe(t)`
-  `TFe` function L806 — `function TFe(t,e,r,n)`
-  `UM` function L806 — `function UM(t)`
-  `Vle` function L806 — `function Vle(t)`
-  `Wle` function L806 — `function Wle(t)`
-  `Xle` function L806 — `function Xle(t)`
-  `Yle` function L806 — `function Yle(t)`
-  `Zle` function L806 — `function Zle(t)`
-  `_Fe` function L806 — `function _Fe(t,e)`
-  `a` function L806 — `function a(h)`
-  `aI` function L806 — `function aI(t)`
-  `b` function L806 — `function b(D,k)`
-  `bp` function L806 — `function bp(t)`
-  `cFe` function L806 — `function cFe(t)`
-  `dFe` function L806 — `function dFe(t)`
-  `e` function L806 — `function e(r)`
-  `eI` function L806 — `function eI(t)`
-  `ece` function L806 — `function ece(t)`
-  `fFe` function L806 — `function fFe(t)`
-  `g` class L806 — `-`
-  `hFe` function L806 — `function hFe(t)`
-  `i` function L806 — `function i(h)`
-  `iI` function L806 — `function iI(t,e)`
-  `l` function L806 — `function l(h)`
-  `n` function L806 — `function n(h)`
-  `nI` function L806 — `function nI(t,e,r)`
-  `p` class L806 — `-`
-  `qle` function L806 — `function qle(t)`
-  `r` function L806 — `function r(i,a,s)`
-  `rI` function L806 — `function rI(t,e)`
-  `s` function L806 — `function s(h)`
-  `tI` function L806 — `function tI(t,e)`
-  `u` function L806 — `function u(h)`
-  `uFe` function L806 — `function uFe(t)`
-  `v` function L806 — `function v(D,k,L)`
-  `vFe` function L806 — `function vFe(t)`
-  `w` function L806 — `function w(D,k)`
-  `wFe` function L806 — `function wFe(t)`
-  `x` function L806 — `function x(D)`
-  `xi` function L806 — `function xi(t)`
-  `zle` function L806 — `function zle(t,e,r=0)`
-  `FE` function L814 — `function FE(t,e)`
-  `ace` function L814 — `function ace(t,e)`
-  `ds` function L814 — `function ds(t)`
-  `fs` function L814 — `function fs(t)`
-  `ice` function L814 — `function ice(t)`
-  `rce` function L814 — `function rce(t,e,r,n)`
-  `ui` function L814 — `function ui(t,e,r,n,i,a,s,l,u)`
-  `Ace` function L815 — `function Ace(t)`
-  `Cce` function L815 — `function Cce(t)`
-  `Dce` function L815 — `function Dce(t)`
-  `Hu` function L815 — `function Hu(t)`
-  `IFe` function L815 — `function IFe()`
-  `Ice` function L815 — `function Ice(t)`
-  `Lce` function L815 — `function Lce(t)`
-  `Mce` function L815 — `function Mce(t)`
-  `Nce` function L815 — `function Nce(t)`
-  `Oce` function L815 — `function Oce(t)`
-  `Rce` function L815 — `function Rce(t)`
-  `Sce` function L815 — `function Sce(t)`
-  `_ce` function L815 — `function _ce(t)`
-  `KE` function L816 — `function KE(t=ps)`
-  `XE` function L816 — `function XE(t=ps)`
-  `ZE` function L816 — `function ZE(t=ps)`
-  `e6` function L816 — `function e6(t=ps)`
-  `i6` function L816 — `function i6(t=ps)`
-  `r6` function L816 — `function r6(t=ps)`
-  `uo` function L816 — `function uo(t,e)`
-  `$c` function L818 — `function $c(t,e)`
-  `Qce` function L818 — `function Qce(t)`
-  `jce` function L818 — `function jce(t,e,r)`
-  `u$e` function L818 — `function u$e(t,e)`
-  `zI` function L818 — `function zI()`
-  `se` function L882 — `function se(W)`
-  `ue` function L882 — `function ue()`
-  `A` function L887 — `function A(S)`
-  `C` function L887 — `function C(S,_,I,D)`
-  `E` function L887 — `function E(S,_,I,D)`
-  `Nue` function L887 — `function Nue(t,e,r)`
-  `T` function L887 — `function T(S,_)`
-  `b` function L887 — `function b(S,_,I,D,k,L,R)`
-  `k` function L887 — `function k()`
-  `m` function L887 — `function m(g)`
-  `v` function L887 — `function v(S,_)`
-  `w` function L887 — `function w(S,_,I,D,k,L,R,O)`
-  `x` function L887 — `function x(S,_,I)`
-  `ut` function L1168 — `function ut()`
-  `xt` function L1168 — `function xt(Ce)`
-  `AGe` function L1173 — `function AGe()`
-  `CGe` function L1173 — `function CGe(t)`
-  `EGe` function L1173 — `function EGe(t,e)`
-  `SGe` function L1173 — `function SGe(t)`
-  `TGe` function L1173 — `function TGe(t)`
-  `V` function L1173 — `function V()`
-  `Xu` function L1173 — `function Xu(t)`
-  `a` function L1173 — `function a(S)`
-  `bGe` function L1173 — `function bGe(t)`
-  `dO` function L1173 — `function dO(t)`
-  `dhe` function L1173 — `function dhe(t)`
-  `fO` function L1173 — `function fO(t)`
-  `gGe` function L1173 — `function gGe(t)`
-  `i` function L1173 — `function i(S)`
-  `kGe` function L1173 — `function kGe(t,e,r,n,i)`
-  `mGe` function L1173 — `function mGe(t)`
-  `phe` function L1173 — `function phe(t)`
-  `s` function L1173 — `function s(S)`
-  `vGe` function L1173 — `function vGe(t)`
-  `wGe` function L1173 — `function wGe(t)`
-  `xGe` function L1173 — `function xGe(t)`
-  `yGe` function L1173 — `function yGe(t)`
-  `Se` function L1174 — `function Se()`
-  `Z` function L1174 — `function Z(re)`
-  `$Ge` function L1179 — `function $Ge(t,e)`
-  `$he` function L1179 — `function $he()`
-  `BGe` function L1179 — `function BGe(t)`
-  `Dhe` function L1179 — `function Dhe(t,e,r,n)`
-  `FGe` function L1179 — `function FGe(t)`
-  `Fhe` function L1179 — `function Fhe()`
-  `GGe` function L1179 — `function GGe(t,e)`
-  `Ghe` function L1179 — `function Ghe(t,e)`
-  `HGe` function L1179 — `function HGe()`
-  `IGe` function L1179 — `function IGe(t)`
-  `Mhe` function L1179 — `function Mhe(t,e,r)`
-  `OGe` function L1179 — `function OGe(t)`
-  `PGe` function L1179 — `function PGe(t)`
-  `R` function L1179 — `function R()`
-  `S1` function L1179 — `function S1(t)`
-  `UGe` function L1179 — `function UGe()`
-  `Uhe` function L1179 — `function Uhe(t)`
-  `VGe` function L1179 — `function VGe(t,e)`
-  `Vhe` function L1179 — `function Vhe(t)`
-  `WGe` function L1179 — `function WGe()`
-  `h` function L1179 — `function h(v)`
-  `kO` function L1179 — `function kO(t)`
-  `l` function L1179 — `function l(v)`
-  `mO` function L1179 — `function mO(t)`
-  `u` function L1179 — `function u(v)`
-  `v6` function L1179 — `function v6(t)`
-  `vO` function L1179 — `function vO(t,e,r,n)`
-  `zGe` function L1179 — `function zGe(t)`
-  `zhe` function L1179 — `function zhe()`
-  `Yt` function L1180 — `function Yt(Dr)`
-  `bt` function L1180 — `function bt()`
-  `y` function L1180 — `function y(v)`
-  `de` function L1185 — `function de()`
-  `oe` function L1243 — `function oe()`
-  `re` function L1243 — `function re(Rt)`
-  `he` function L1248 — `function he()`
-  `Hc` function L1364 — `function Hc(t,e,r,n,i)`
-  `MVe` function L1364 — `function MVe(t,e,r)`
-  `OVe` function L1364 — `function OVe(t,e,r)`
-  `RVe` function L1364 — `function RVe(t,e,r,n,i,a,s)`
-  `T` function L1364 — `function T(F,P)`
-  `Ue` function L1364 — `function Ue(Tt)`
-  `_Ve` function L1364 — `function _Ve(t,e)`
-  `ct` function L1364 — `function ct()`
-  `e` function L1364 — `function e(a,s,l,u,h,f,d,p)`
-  `i` function L1364 — `function i(a,s)`
-  `l` function L1364 — `function l(h,f)`
-  `n` function L1364 — `function n(a,s,l,u,h,f,d,p)`
-  `r` function L1364 — `function r(i,a,s,l,u)`
-  `s` function L1364 — `function s(l)`
-  `t` function L1364 — `function t(a,s,l,u,h,f,d)`
-  `u` function L1364 — `function u(h,f)`
-  `Re` function L1369 — `function Re()`
-  `Oe` function L1527 — `function Oe()`
-  `ae` function L1527 — `function ae(xe)`
-  `B` function L1532 — `function B()`
-  `G6` function L1532 — `function G6(t,e,r)`
-  `ZO` function L1532 — `function ZO(t="",e=0,r="",n=$6)`
-  `lUe` function L1532 — `function lUe(t)`
-  `oUe` function L1532 — `function oUe(t)`
-  `ude` function L1532 — `function ude()`
-  `M` function L1746 — `function M()`
-  `O` function L1746 — `function O(K)`
-  `f` function L1751 — `function f()`
-  `B` function L1883 — `function B()`
-  `JUe` function L1883 — `function JUe(t)`
-  `M` function L1883 — `function M(X)`
-  `a` function L1883 — `function a(u)`
-  `e` function L1883 — `function e(i,a,s,l,u,h,f,d,p)`
-  `l` function L1883 — `function l(u)`
-  `n` function L1883 — `function n(i,a)`
-  `r` function L1883 — `function r(i,a,s,l,u)`
-  `s` function L1883 — `function s(u)`
-  `t` function L1883 — `function t(i,a,s,l,u,h,f,d)`
-  `a` function L1888 — `function a(u)`
-  `d` function L1888 — `function d()`
-  `dpe` function L1888 — `function dpe(t,e)`
-  `e` function L1888 — `function e(i,a,s,l,u,h,f,d,p)`
-  `l` function L1888 — `function l(u)`
-  `n` function L1888 — `function n(i,a)`
-  `r` function L1888 — `function r(i,a,s,l,u)`
-  `s` function L1888 — `function s(u)`
-  `t` function L1888 — `function t(i,a,s,l,u,h,f,d)`
-  `ie` function L1943 — `function ie()`
-  `j` function L1943 — `function j(ae)`
-  `Dpe` function L1948 — `function Dpe(t,e)`
-  `HHe` function L1948 — `function HHe(t)`
-  `If` function L1948 — `function If(t,e,r)`
-  `Mf` function L1948 — `function Mf(t,e)`
-  `OP` function L1948 — `function OP(t,e)`
-  `UHe` function L1948 — `function UHe(t)`
-  `WHe` function L1948 — `function WHe(t)`
-  `Wi` function L1948 — `function Wi(t)`
-  `X0e` function L1948 — `function X0e(t,e,r)`
-  `YHe` function L1948-1949 — `function YHe()`
-  `ZP` function L1948 — `function ZP(t,e)`
-  `_i` function L1948 — `function _i(t,e)`
-  `j0e` function L1948 — `function j0e(t)`
-  `qHe` function L1948 — `function qHe(t,e)`
-  `w` function L1948 — `function w()`
-  `XHe` function L1949-1950 — `function XHe()`
-  `mo` function L1950-1951 — `function mo(t,e)`
-  `$0e` function L1951 — `function $0e(t,e,r)`
-  `$1` function L1951 — `function $1(t,e,r,n,i)`
-  `$S` function L1951 — `function $S()`
-  `$Xe` function L1951 — `function $Xe(t,e)`
-  `$Ze` function L1951 — `function $Ze(t,e)`
-  `$ge` function L1951 — `function $ge(t,e)`
-  `$me` function L1951 — `function $me(t,e,r)`
-  `A` function L1951 — `function A()`
-  `AP` function L1951 — `function AP(t,e)`
-  `AXe` function L1951 — `function AXe(t)`
-  `Age` function L1951 — `function Age(t)`
-  `Aje` function L1951 — `function Aje(t,e)`
-  `BWe` function L1951 — `function BWe(t)`
-  `BZe` function L1951 — `function BZe(t,e,r,n,i)`
-  `C` function L1951 — `function C()`
-  `Cge` function L1951 — `function Cge(t,e,r)`
-  `Cje` function L1951 — `function Cje(t)`
-  `DS` function L1951 — `function DS(t,e,r)`
-  `DZe` function L1951 — `function DZe(t)`
-  `Dje` function L1951 — `function Dje(t,e,r)`
-  `Eje` function L1951 — `function Eje(t,e)`
-  `F1` function L1951 — `function F1(t,e)`
-  `FXe` function L1951 — `function FXe(t,e)`
-  `FZe` function L1951 — `function FZe(t,e,r)`
-  `Fge` function L1951 — `function Fge(t,e,r)`
-  `Fme` function L1951 — `function Fme(t,e,r)`
-  `GS` function L1951 — `function GS(t,e)`
-  `GWe` function L1951 — `function GWe(t)`
-  `Gb` function L1951 — `function Gb()`
-  `H0e` function L1951 — `function H0e(t,e)`
-  `HS` function L1951 — `function HS(t)`
-  `HXe` function L1951 — `function HXe(t)`
-  `HZe` function L1951 — `function HZe(t,e,r,n,i)`
-  `Hje` function L1951 — `function Hje(t,e,r)`
-  `IS` function L1951 — `function IS(t,e,r,n,i,a)`
-  `IXe` function L1951 — `function IXe(t,e)`
-  `IZe` function L1951 — `function IZe(t,e)`
-  `Ime` function L1951 — `function Ime(t)`
-  `JXe` function L1951 — `function JXe()`
-  `Jme` function L1951 — `function Jme(t,e,r)`
-  `KWe` function L1951 — `function KWe(t)`
-  `LZe` function L1951 — `function LZe(t)`
-  `Lb` function L1951 — `function Lb()`
-  `Lge` function L1951 — `function Lge(t,e)`
-  `MWe` function L1951 — `function MWe(t)`
-  `MZe` function L1951 — `function MZe(t)`
-  `Mge` function L1951 — `function Mge(t,e,r)`
-  `NP` function L1951 — `function NP(t,e,r,n,i)`
-  `NZe` function L1951 — `function NZe(t)`
-  `Nje` function L1951 — `function Nje(t,e,r)`
-  `OZe` function L1951 — `function OZe(t)`
-  `Oge` function L1951 — `function Oge(t)`
-  `PXe` function L1951 — `function PXe()`
-  `PZe` function L1951 — `function PZe(t)`
-  `Pje` function L1951 — `function Pje(t,e,r)`
-  `RP` function L1951 — `function RP(t,e,r,n,i)`
-  `RXe` function L1951 — `function RXe(t)`
-  `RZe` function L1951 — `function RZe(t)`
-  `Rb` function L1951 — `function Rb(t,e)`
-  `Rge` function L1951 — `function Rge(t,e,r)`
-  `SP` function L1951 — `function SP(t)`
-  `SWe` function L1951 — `function SWe(t)`
-  `SZe` function L1951 — `function SZe(t,e)`
-  `Sge` function L1951 — `function Sge(t,e)`
-  `T` function L1951 — `function T()`
-  `TB` function L1951 — `function TB(t,e,r)`
-  `TXe` function L1951 — `function TXe(t)`
-  `TZe` function L1951 — `function TZe(t,e,r)`
-  `UKe` function L1951 — `function UKe(t,e,r,n)`
-  `VWe` function L1951 — `function VWe(t)`
-  `VXe` function L1951 — `function VXe(t)`
-  `Vje` function L1951 — `function Vje(t,e,r,n)`
-  `WKe` function L1951 — `function WKe(t,e,r,n)`
-  `WWe` function L1951 — `function WWe(t)`
-  `WZe` function L1951 — `function WZe(t,e,r,n)`
-  `XXe` function L1951 — `function XXe(t,e)`
-  `Xje` function L1951 — `function Xje(t)`
-  `Y0e` function L1951 — `function Y0e(t)`
-  `ZYe` function L1951 — `function ZYe(t)`
-  `_Qe` function L1951 — `function _Qe(t,e,r)`
-  `_We` function L1951 — `function _We(t)`
-  `_ge` function L1951 — `function _ge(t,e,r)`
-  `a` function L1951 — `function a(f)`
-  `aXe` function L1951 — `function aXe(t)`
-  `aZe` function L1951 — `function aZe(t,e,r)`
-  `age` function L1951 — `function age(t)`
-  `ay` function L1951 — `function ay(t)`
-  `b` function L1951 — `function b(E)`
-  `b0e` function L1951 — `function b0e(t,e)`
-  `bXe` function L1951 — `function bXe()`
-  `bZe` function L1951 — `function bZe(t)`
-  `bje` function L1951 — `function bje(t,e)`
-  `cB` function L1951 — `function cB(t,e)`
-  `dB` function L1951 — `function dB(t)`
-  `dKe` function L1951 — `function dKe(t,e,r)`
-  `e` function L1951 — `function e(n,i,a)`
-  `eqe` function L1951 — `function eqe(t,e,r)`
-  `f` function L1951 — `function f(g,y,v)`
-  `f0e` function L1951 — `function f0e(t)`
-  `fB` function L1951 — `function fB(t,e,r,n)`
-  `fge` function L1951 — `function fge(t)`
-  `g` function L1951 — `function g(E)`
-  `g0e` function L1951 — `function g0e(t)`
-  `gB` function L1951 — `function gB(t)`
-  `gXe` function L1951 — `function gXe(t,e)`
-  `gZe` function L1951 — `function gZe(t)`
-  `hge` function L1951 — `function hge(t,e)`
-  `i` function L1951 — `function i(f,d)`
-  `iZe` function L1951 — `function iZe(t,e)`
-  `ije` function L1951 — `function ije(t)`
-  `kje` function L1951 — `function kje(t)`
-  `l` function L1951 — `function l(f)`
-  `lS` function L1951 — `function lS(t,e)`
-  `lge` function L1951 — `function lge(t)`
-  `lje` function L1951 — `function lje(t)`
-  `m` function L1951 — `function m(E)`
-  `mZe` function L1951 — `function mZe(t,e,r)`
-  `mje` function L1951 — `function mje(t)`
-  `n` function L1951 — `function n(i)`
-  `nge` function L1951 — `function nge(t)`
-  `nje` function L1951 — `function nje(t,e)`
-  `ny` function L1951 — `function ny(t)`
-  `oS` function L1951 — `function oS(t,e,r)`
-  `oZe` function L1951 — `function oZe(t,e,r,n)`
-  `oge` function L1951 — `function oge(t)`
-  `p` function L1951 — `function p(R,O)`
-  `pXe` function L1951 — `function pXe(t)`
-  `pZe` function L1951 — `function pZe(t,e,r)`
-  `po` function L1951 — `function po(t,e,r,n)`
-  `qKe` function L1951 — `function qKe(t,e,r,n)`
-  `qXe` function L1951 — `function qXe(t)`
-  `qYe` function L1951 — `function qYe(t,e)`
-  `qje` function L1951 — `function qje(t,e)`
-  `r` function L1951 — `function r(s)`
-  `rS` function L1951 — `function rS()`
-  `rge` function L1951 — `function rge(t)`
-  `ry` function L1951 — `function ry(t)`
-  `s` function L1951 — `function s(R)`
-  `sZe` function L1951 — `function sZe(t,e,r,n,i)`
-  `sge` function L1951 — `function sge(t)`
-  `sje` function L1951 — `function sje(t)`
-  `t` function L1951 — `function t()`
-  `tXe` function L1951 — `function tXe(t)`
-  `tge` function L1951 — `function tge(t)`
-  `tje` function L1951 — `function tje(t)`
-  `ty` function L1951 — `function ty(t)`
-  `u` function L1951 — `function u(R)`
-  `uje` function L1951 — `function uje(t,e)`
-  `v` function L1951 — `function v(E)`
-  `v0e` function L1951 — `function v0e(t)`
-  `vB` function L1951 — `function vB(t,e,r,n)`
-  `vWe` function L1951 — `function vWe(t,e)`
-  `vXe` function L1951 — `function vXe(t,e)`
-  `vZe` function L1951 — `function vZe(t,e,r,n,i,a)`
-  `w` function L1951 — `function w()`
-  `w0e` function L1951 — `function w0e(t,e)`
-  `wB` function L1951 — `function wB(t)`
-  `wZe` function L1951 — `function wZe(t,e,r)`
-  `x` function L1951 — `function x()`
-  `x0e` function L1951 — `function x0e(t,e,r,n,i)`
-  `xZe` function L1951 — `function xZe(t,e,r,n)`
-  `y` function L1951 — `function y(E)`
-  `y0e` function L1951 — `function y0e(t)`
-  `yWe` function L1951 — `function yWe(t)`
-  `yZe` function L1951 — `function yZe(t,e)`
-  `z0e` function L1951 — `function z0e(t,e,r)`
-  `zS` function L1951 — `function zS(t,e)`
-  `zje` function L1951 — `function zje(t,e)`
-  `D` function L1954 — `function D(K,X,te,J,se)`
-  `k` function L1954 — `function k(K,X)`
-  `r` function L1954 — `function r(n)`
-  `t` function L1954 — `function t(e)`
-  `JZe` function L2155 — `function JZe(t,e,r,n,i)`
-  `a` function L2155 — `function a(s)`
-  `h` function L2155 — `function h(d,p,m,g)`
-  `i` function L2155 — `function i()`
-  `l` function L2155 — `function l(u,h)`
-  `m` function L2155 — `function m(v)`
-  `n` function L2155 — `function n()`
-  `p` function L2155 — `function p(g,y,v)`
-  `r` function L2155 — `function r(n)`
-  `s` function L2155 — `function s(u,h,f)`
-  `t` function L2155 — `function t(e)`
-  `w` function L2155 — `function w()`
-  `Yge` function L2188 — `function Yge(t,e,r,n)`
-  `iJe` function L2188 — `function iJe(t,e)`
-  `nJe` function L2188 — `function nJe(t,e)`
-  `qge` function L2188 — `function qge(t,e,r,n,i)`
-  `rJe` function L2188 — `function rJe(t,e)`
-  `he` function L2244 — `function he()`
-  `le` function L2244 — `function le(ze)`
-  `E` function L2249 — `function E()`
-  `D` function L2331 — `function D(ie)`
-  `k` function L2331 — `function k()`
-  `$B` function L2336 — `function $B(t)`
-  `$Je` function L2336 — `function $Je(t)`
-  `A` function L2336 — `function A(O,M,B)`
-  `A1e` function L2336 — `function A1e()`
-  `BB` function L2336 — `function BB(t)`
-  `BJe` function L2336 — `function BJe(t)`
-  `C` function L2336 — `function C(O)`
-  `D` function L2336 — `function D({sourceLinks:O,targetLinks:M})`
-  `E` function L2336 — `function E(O,M,B)`
-  `E1e` function L2336 — `function E1e(t,e)`
-  `FB` function L2336 — `function FB(t,e)`
-  `FJe` function L2336 — `function FJe(t)`
-  `GB` function L2336 — `function GB(t)`
-  `GJe` function L2336 — `function GJe(t)`
-  `HB` function L2336 — `function HB()`
-  `HJe` function L2336 — `function HJe(t,e,r,n,i)`
-  `I` function L2336 — `function I(O,M,B,F)`
-  `JS` function L2336 — `function JS()`
-  `KS` function L2336 — `function KS(t,e)`
-  `L` function L2336 — `function L(O,M)`
-  `N1e` function L2336 — `function N1e(t)`
-  `PJe` function L2336 — `function PJe(t)`
-  `QS` function L2336 — `function QS()`
-  `R` function L2336 — `function R(O,M)`
-  `R1e` function L2336 — `function R1e(t)`
-  `S` function L2336 — `function S(O,M)`
-  `S1e` function L2336 — `function S1e({nodes:t})`
-  `T` function L2336 — `function T(O)`
-  `T1e` function L2336 — `function T1e(t,e)`
-  `UJe` function L2336 — `function UJe(t)`
-  `VJe` function L2336 — `function VJe(t)`
-  `WJe` function L2336 — `function WJe(t)`
-  `ZS` function L2336 — `function ZS(t)`
-  `_` function L2336 — `function _(O,M,B,F)`
-  `b` function L2336 — `function b({nodes:O})`
-  `cy` function L2336 — `function cy(t,e)`
-  `g` function L2336 — `function g()`
-  `hy` function L2336 — `function hy(t)`
-  `k` function L2336 — `function k(O)`
-  `k1e` function L2336 — `function k1e(t,e)`
-  `m4` function L2336 — `function m4(t,e)`
-  `p4` function L2336 — `function p4(t,e)`
-  `qB` function L2336 — `function qB()`
-  `qJe` function L2336 — `function qJe(t)`
-  `s` function L2336 — `function s()`
-  `uy` function L2336 — `function uy(t,e)`
-  `v` function L2336 — `function v({nodes:O})`
-  `w` function L2336 — `function w({nodes:O})`
-  `x` function L2336 — `function x({nodes:O})`
-  `y` function L2336 — `function y({nodes:O,links:M})`
-  `Iet` function L2363 — `function Iet(t,e,r,n)`
-  `Met` function L2363 — `function Met(t,e)`
-  `Net` function L2363 — `function Net(t,e,r,n)`
-  `Ret` function L2363 — `function Ret(t,e,r,n,i,a,s)`
-  `Q` function L2405 — `function Q(ce)`
-  `j` function L2405 — `function j()`
-  `Xet` function L2410 — `function Xet(t)`
-  `Yet` function L2410 — `function Yet(t)`
-  `b` function L2410 — `function b()`
-  `jet` function L2410 — `function jet(t)`
-  `Bye` function L2522 — `function Bye(t,e)`
-  `Iye` function L2522 — `function Iye(t,{minX:e,minY:r,maxX:n,maxY:i}={minX:0,minY:0,maxX:0,maxY:0})`
-  `Mye` function L2522 — `function Mye(t,e)`
-  `Oye` function L2522 — `function Oye(t)`
-  `eF` function L2522 — `function eF(t,e,r=0,n=0)`
-  `nC` function L2522 — `function nC(t,e)`
-  `wtt` function L2522 — `function wtt(t)`
-  `xtt` function L2522 — `function xtt(t,e)`
-  `Att` function L2525 — `function Att(t,e)`
-  `C` function L2525 — `function C()`
-  `Dtt` function L2525 — `function Dtt(t,e,r)`
-  `Hl` function L2525 — `function Hl(t,e,r,n)`
-  `Li` function L2525 — `function Li(t)`
-  `Ltt` function L2525 — `function Ltt(t,e,r,n)`
-  `Rtt` function L2525 — `function Rtt(t,e,r)`
-  `Zye` function L2525 — `function Zye(t,e)`
-  `_tt` function L2525 — `function _tt(t,e,r,n)`
-  `aF` function L2525 — `function aF(t,e,r,n)`
-  `dve` function L2525 — `function dve(t,e,r=!1)`
-  `gve` function L2525 — `function gve(t,e,r,n,i)`
-  `mve` function L2525 — `function mve(t,e,r)`
-  `oF` function L2525 — `function oF(t,e,r,n)`
-  `pve` function L2525 — `function pve(t,e,r)`
-  `rrt` function L2525 — `function rrt(t,e,r)`
-  `trt` function L2525 — `function trt(t,e,r)`
-  `I` function L2556 — `function I()`
-  `Pe` function L2556 — `function Pe(st,Ue)`
-  `a` function L2556 — `function a(s)`
-  `b` function L2556 — `function b(w,C)`
-  `d` function L2556 — `function d(p,m)`
-  `f` function L2556 — `function f(p,m,g,y)`
-  `h` function L2556 — `function h(d,p,m,g)`
-  `i` function L2556 — `function i()`
-  `l` function L2556 — `function l(u,h)`
-  `m` function L2556 — `function m()`
-  `n` function L2556 — `function n()`
-  `p` function L2556 — `function p(g,y,v)`
-  `r` function L2556 — `function r(n)`
-  `s` function L2556 — `function s(u,h,f)`
-  `u` function L2556 — `function u(g)`
-  `Art` function L2560 — `function Art(t,e)`
-  `Crt` function L2560 — `function Crt(t,e)`
-  `Drt` function L2560 — `function Drt(t,e)`
-  `Lrt` function L2560 — `function Lrt(t,e,r)`
-  `Nrt` function L2560 — `function Nrt(t,e,r,n,i,{spatialMaps:a,groupAlignments:s})`
-  `Rrt` function L2560 — `function Rrt(t)`
-  `Srt` function L2560 — `function Srt(t,e)`
-  `_rt` function L2560 — `function _rt(t,e)`
-  `m` function L2560 — `function m(g,y,v,x)`
-  `$f` function L2561 — `function $f(t,e)`
-  `A4` function L2561 — `function A4()`
-  `C4` function L2561 — `function C4(t,e,r)`
-  `Frt` function L2561 — `function Frt(t,e,r,n)`
-  `a2e` function L2561 — `function a2e()`
-  `al` function L2561 — `function al()`
-  `c2e` function L2561 — `function c2e(t,e)`
-  `d2e` function L2561 — `function d2e(t,e,r,n,i)`
-  `dC` function L2561 — `function dC(t,e,r,n,i,a,s,l)`
-  `f2e` function L2561 — `function f2e(t,e,r,n,i,a,s,l,u,h,f,d)`
-  `g2e` function L2561 — `function g2e(t,e)`
-  `gC` function L2561 — `function gC(t,e,r,n,i,a,s,l,u)`
-  `h2e` function L2561 — `function h2e(t)`
-  `hC` function L2561 — `function hC(t)`
-  `i2e` function L2561 — `function i2e()`
-  `l2e` function L2561 — `function l2e(t)`
-  `m2e` function L2561 — `function m2e(t,e,r,n)`
-  `mC` function L2561 — `function mC(t)`
-  `my` function L2561 — `function my(t,e)`
-  `o2e` function L2561 — `function o2e(t)`
-  `p2e` function L2561 — `function p2e(t)`
-  `pC` function L2561 — `function pC(t,e)`
-  `r2e` function L2561 — `function r2e(t)`
-  `rh` function L2561 — `function rh()`
-  `s2e` function L2561 — `function s2e(t)`
-  `t2e` function L2561 — `function t2e(t,e,r)`
-  `u2e` function L2561 — `function u2e(t,e)`
-  `vo` function L2561 — `function vo(t)`
-  `xF` function L2561 — `function xF(t)`
-  `y2e` function L2561 — `function y2e(t,e,r,n)`
-  `yC` function L2561 — `function yC(t,e)`
-  `yy` function L2561 — `function yy(t)`
-  `zf` function L2561 — `function zf(t,e,r)`
-  `w2e` function L2562 — `function w2e(t)`
-  `C2e` function L2563 — `function C2e(t)`
-  `T2e` function L2563 — `function T2e(t)`
-  `bF` function L2563 — `function bF(t)`
-  `int` function L2563 — `function int(t,e)`
-  `S2e` function L2569 — `function S2e(t,e)`
-  `fnt` function L2569 — `function fnt(t,e,r,n)`
-  `hnt` function L2569 — `function hnt(t={})`

### examples/ui-slim/src

> *Semantic summary to be generated by AI agent.*

#### examples/ui-slim/src/App.js

- pub `App` function L3384-3390 — `function App()`
-  `StackTelemetrySection` function L32-189 — `const StackTelemetrySection = ({ stackId })`
-  `toggleLive` function L112 — `const toggleLive = ()`
-  `AgentsPanel` function L192-486 — `const AgentsPanel = ({ stacks, onRefresh })`
-  `tick` function L238-247 — `const tick = ()`
-  `selectAgent` function L253-262 — `const selectAgent = (agent)`
-  `addLabel` function L264-273 — `const addLabel = (label)`
-  `removeLabel` function L275-283 — `const removeLabel = (label)`
-  `addAnnotation` function L285-294 — `const addAnnotation = (key, value)`
-  `removeAnnotation` function L296-304 — `const removeAnnotation = (key)`
-  `addTarget` function L306-315 — `const addTarget = (stackId)`
-  `removeTarget` function L317-325 — `const removeTarget = (stackId)`
-  `toggleStatus` function L327-338 — `const toggleStatus = ()`
-  `StacksPanel` function L489-853 — `const StacksPanel = ({ generators, agents, onRefresh })`
-  `selectStack` function L526-539 — `const selectStack = (stack)`
-  `create` function L541-552 — `const create = (e)`
-  `deploy` function L554-567 — `const deploy = (e)`
-  `addLabel` function L569-578 — `const addLabel = (label)`
-  `removeLabel` function L580-588 — `const removeLabel = (label)`
-  `addAnnotation` function L590-599 — `const addAnnotation = (key, value)`
-  `removeAnnotation` function L601-609 — `const removeAnnotation = (key)`
-  `copyDeployment` function L611-620 — `const copyDeployment = (depId)`
-  `requestDiagnostic` function L622-647 — `const requestDiagnostic = (depId, agentId)`
-  `pollResult` function L628-642 — `const pollResult = ()`
-  `TemplatesPanel` function L856-1094 — `const TemplatesPanel = ({ stacks })`
-  `create` function L895-906 — `const create = (e)`
-  `instantiate` function L908-919 — `const instantiate = (e)`
-  `remove` function L921-932 — `const remove = (id)`
-  `addLabel` function L934-943 — `const addLabel = (label)`
-  `removeLabel` function L945-953 — `const removeLabel = (label)`
-  `JobsPanel` function L1097-1467 — `const JobsPanel = ({ agents })`
-  `create` function L1133-1151 — `const create = (e)`
-  `cancel` function L1153-1163 — `const cancel = (id)`
-  `runBuildDemo` function L1166-1239 — `const runBuildDemo = ()`
-  `prefillBuildDemo` function L1242-1249 — `const prefillBuildDemo = ()`
-  `AdminPanel` function L1470-1617 — `const AdminPanel = ({ onGeneratorsChange, onAgentsChange })`
-  `create` function L1496-1513 — `const create = (e)`
-  `rotate` function L1515-1525 — `const rotate = (type, id)`
-  `copy` function L1527-1530 — `const copy = ()`
-  `closeCreate` function L1532-1538 — `const closeCreate = ()`
-  `WebhooksPanel` function L1620-1950 — `const WebhooksPanel = ()`
-  `selectWebhook` function L1653-1662 — `const selectWebhook = (webhook)`
-  `create` function L1664-1681 — `const create = (e)`
-  `toggleEnabled` function L1683-1694 — `const toggleEnabled = (webhook)`
-  `remove` function L1696-1707 — `const remove = (id)`
-  `toggleEventType` function L1709-1715 — `const toggleEventType = (type)`
-  `MetricsPanel` function L1953-2117 — `const MetricsPanel = ()`
-  `getMetricValue` function L1980-1986 — `const getMetricValue = (name, labels = {})`
-  `getMetricValues` function L1989 — `const getMetricValues = (name)`
-  `sumMetric` function L1992-1995 — `const sumMetric = (name)`
-  `DemoPanel` function L2120-3333 — `const DemoPanel = ()`
-  `startEventPolling` function L2148-2167 — `const startEventPolling = ()`
-  `poll` function L2151-2164 — `const poll = ()`
-  `stopEventPolling` function L2170-2176 — `const stopEventPolling = ()`
-  `clearWebhookEvents` function L2179-2186 — `const clearWebhookEvents = ()`
-  `getEventTypeClass` function L2198-2205 — `const getEventTypeClass = (eventType)`
-  `getEventStatusClass` function L2208-2220 — `const getEventStatusClass = (event)`
-  `formatEventPayload` function L2223-2231 — `const formatEventPayload = (event)`
-  `EventLogPanel` function L2234-2285 — `const EventLogPanel = ()`
-  `updatePhase` function L2288-2296 — `const updatePhase = (phaseNum, updates)`
-  `addStep` function L2299-2310 — `const addStep = (phaseNum, step)`
-  `formatDuration` function L2313-2319 — `const formatDuration = (ms)`
-  `resetDemo` function L2322-2352 — `const resetDemo = ()`
-  `canStartPhase` function L2357-2384 — `const canStartPhase = (phaseNum)`
-  `runPhase` function L2387-2418 — `const runPhase = (phaseNum)`
-  `runPhase1` function L2421-2495 — `const runPhase1 = ()`
-  `runPhase2` function L2498-2551 — `const runPhase2 = ()`
-  `runPhase3` function L2554-2635 — `const runPhase3 = ()`
-  `runPhase4` function L2638-2736 — `const runPhase4 = ()`
-  `runPhase5` function L2739-2831 — `const runPhase5 = ()`
-  `runPhase6` function L2834-2963 — `const runPhase6 = ()`
-  `runPhase7` function L2966-3043 — `const runPhase7 = ()`
-  `runPhase8` function L3046-3090 — `const runPhase8 = ()`
-  `runCleanup` function L3093-3181 — `const runCleanup = ()`
-  `PhaseCard` function L3187-3261 — `const PhaseCard = ({ num, phase })`
-  `AppContent` function L3337-3381 — `const AppContent = ()`

#### examples/ui-slim/src/api.js

- pub `ApiError` class L41-49 — `-`
- pub `constructor` method L42-48 — `constructor({ message, code, status, response })`
- pub `getAgents` function L82 — `const getAgents = ()`
- pub `getAgentLabels` function L83-84 — `const getAgentLabels = (id)`
- pub `getAgentAnnotations` function L85-88 — `const getAgentAnnotations = (id)`
- pub `getAgentTargets` function L89-90 — `const getAgentTargets = (id)`
- pub `getAgentEvents` function L91-92 — `const getAgentEvents = (id)`
- pub `getAgentStacks` function L93-94 — `const getAgentStacks = (id)`
- pub `addAgentLabel` function L95-101 — `const addAgentLabel = (id, label)`
- pub `removeAgentLabel` function L102-107 — `const removeAgentLabel = (id, label)`
- pub `addAgentAnnotation` function L108-114 — `const addAgentAnnotation = (id, key, value)`
- pub `removeAgentAnnotation` function L115-120 — `const removeAgentAnnotation = (id, key)`
- pub `addAgentTarget` function L121-127 — `const addAgentTarget = (id, stackId)`
- pub `removeAgentTarget` function L128-133 — `const removeAgentTarget = (id, stackId)`
- pub `createAgent` function L134-135 — `const createAgent = (name, cluster)`
- pub `updateAgent` function L136-139 — `const updateAgent = (id, updates)`
- pub `rotateAgentPak` function L140-143 — `const rotateAgentPak = (id)`
- pub `getStacks` function L149 — `const getStacks = ()`
- pub `getStackLabels` function L150-151 — `const getStackLabels = (id)`
- pub `getStackAnnotations` function L152-155 — `const getStackAnnotations = (id)`
- pub `getStackDeployments` function L156-161 — `const getStackDeployments = (id)`
- pub `getStackEvents` function L176-177 — `const getStackEvents = (id, query = {})`
- pub `getStackLogs` function L184-185 — `const getStackLogs = (id, query = {})`
- pub `getWsConnections` function L191 — `const getWsConnections = ()`
- pub `openStackLiveStream` function L203-206 — `const openStackLiveStream = (id)`
- pub `createStack` function L207-212 — `const createStack = (name, description, generatorId)`
- pub `addStackLabel` function L213-219 — `const addStackLabel = (id, label)`
- pub `removeStackLabel` function L220-225 — `const removeStackLabel = (id, label)`
- pub `addStackAnnotation` function L226-232 — `const addStackAnnotation = (id, key, value)`
- pub `removeStackAnnotation` function L233-238 — `const removeStackAnnotation = (id, key)`
- pub `createDeployment` function L239-252 — `const createDeployment = (stackId, yaml, isDeletion = false)`
- pub `getDeployment` function L253-256 — `const getDeployment = (id)`
- pub `getTemplates` function L262 — `const getTemplates = ()`
- pub `getTemplateLabels` function L263-266 — `const getTemplateLabels = (id)`
- pub `getTemplateAnnotations` function L267-270 — `const getTemplateAnnotations = (id)`
- pub `createTemplate` function L271-281 — `const createTemplate = (name, description, content, schema)`
- pub `updateTemplate` function L282-292 — `const updateTemplate = (id, description, content, schema)`
- pub `deleteTemplate` function L293-294 — `const deleteTemplate = (id)`
- pub `addTemplateLabel` function L295-301 — `const addTemplateLabel = (id, label)`
- pub `removeTemplateLabel` function L302-307 — `const removeTemplateLabel = (id, label)`
- pub `instantiateTemplate` function L308-314 — `const instantiateTemplate = (stackId, templateId, params)`
- pub `getGenerators` function L320 — `const getGenerators = ()`
- pub `createGenerator` function L321-326 — `const createGenerator = (name, description)`
- pub `rotateGeneratorPak` function L327-332 — `const rotateGeneratorPak = (id)`
- pub `getWorkOrders` function L338-343 — `const getWorkOrders = (status, workType)`
- pub `getWorkOrder` function L344-347 — `const getWorkOrder = (id)`
- pub `createWorkOrder` function L348-360 — `const createWorkOrder = (workType, yamlContent, targeting, options = {})`
- pub `deleteWorkOrder` function L361-362 — `const deleteWorkOrder = (id)`
- pub `getWorkOrderLog` function L363-370 — `const getWorkOrderLog = (workType, success, agentId, limit)`
- pub `createDiagnostic` function L376-391 — `const createDiagnostic = ( deploymentObjectId, agentId, requestedBy, retentionMi...`
- pub `getDiagnostic` function L392-393 — `const getDiagnostic = (id)`
- pub `getDeploymentHealth` function L399-404 — `const getDeploymentHealth = (id)`
- pub `getStackHealth` function L405-406 — `const getStackHealth = (id)`
- pub `getWebhooks` function L412 — `const getWebhooks = ()`
- pub `getWebhook` function L413-414 — `const getWebhook = (id)`
- pub `createWebhook` function L415-428 — `const createWebhook = (name, url, eventTypes, authHeader, options = {})`
- pub `updateWebhook` function L429-435 — `const updateWebhook = (id, updates)`
- pub `deleteWebhook` function L436-437 — `const deleteWebhook = (id)`
- pub `getWebhookEventTypes` function L438-439 — `const getWebhookEventTypes = ()`
- pub `getWebhookDeliveries` function L440-449 — `const getWebhookDeliveries = (id, status, limit)`
- pub `getMetrics` function L456-460 — `const getMetrics = ()`
- pub `getWebhookCatcherStats` function L466-470 — `const getWebhookCatcherStats = ()`
- pub `clearWebhookCatcher` function L472-478 — `const clearWebhookCatcher = ()`
- pub `getDemoBuildYaml` function L491-509 — `const getDemoBuildYaml = ()`
- pub `deleteStack` function L515-516 — `const deleteStack = (id)`
- pub `deleteAgent` function L517-518 — `const deleteAgent = (id)`
- pub `deleteGenerator` function L519-520 — `const deleteGenerator = (id)`
- pub `createBuildWorkOrder` function L526-536 — `const createBuildWorkOrder = ( imageTag = "latest", agentId = null, )`
- pub `getWebhookCatcherDeploymentYaml` function L538-586 — `const getWebhookCatcherDeploymentYaml = (imageTag = "latest")`
- pub `parseMetrics` function L589-611 — `const parseMetrics = (metricsText)`
- pub `checkEnvironment` function L615-645 — `const checkEnvironment = ()`
- pub `getWebhookCatcherEvents` function L647-655 — `const getWebhookCatcherEvents = ()`
- pub `pollForCondition` function L657-669 — `const pollForCondition = ( checkFn, intervalMs = 2000, timeoutMs = 60000, )`
- pub `pollAgentStatus` function L671-693 — `const pollAgentStatus = (agentId, timeoutMs = 120000)`
- pub `pollWorkOrderStatus` function L695-712 — `const pollWorkOrderStatus = (workOrderId, timeoutMs = 300000)`
- pub `cleanupDemo` function L715-800 — `const cleanupDemo = (resources, onProgress)`
-  `sha256` function L30-36 — `const sha256 = (str)`
-  `unwrap` function L54-76 — `const unwrap = (callPromise)`
-  `log` function L716 — `const log = (step, status)`

#### examples/ui-slim/src/components.js

- pub `useToast` function L14 — `const useToast = ()`
- pub `ToastProvider` function L24-38 — `const ToastProvider = ({ children })`
- pub `getErrorMessage` function L43-48 — `const getErrorMessage = (error)`
- pub `Tag` function L52-57 — `const Tag = ({ children, onRemove, variant = 'default' })`
- pub `Section` function L61-74 — `const Section = ({ title, icon, children, defaultOpen = false, count })`
- pub `InlineAdd` function L78-103 — `const InlineAdd = ({ placeholder, onAdd, fields = 1 })`
- pub `Status` function L107-113 — `const Status = ({ status })`
- pub `HeartbeatIndicator` function L119-135 — `const HeartbeatIndicator = ({ lastHeartbeat })`
- pub `Pagination` function L139-158 — `const Pagination = ({ page, totalPages, onPageChange, pageSize, onPageSizeChange...`
- pub `usePagination` function L161-183 — `const usePagination = (items, defaultPageSize = 25)`
- pub `Modal` function L187-197 — `const Modal = ({ title, onClose, children })`
-  `Toast` function L17-22 — `const Toast = ({ message, type = 'info', onClose })`
-  `showToast` function L27-30 — `const showToast = (message, type = 'success')`
-  `handleSubmit` function L80-89 — `const handleSubmit = (e)`

### examples/webhook-catcher

> *Semantic summary to be generated by AI agent.*

#### examples/webhook-catcher/main.py

- pub `WebhookHandler` class L23-117 — `(BaseHTTPRequestHandler) { log_message, send_cors_headers, send_json, do_OPTIONS...`
- pub `log_message` method L24-25 — `def log_message(self, format: str, *args) -> None`
- pub `send_cors_headers` method L27-31 — `def send_cors_headers(self) -> None` — Add CORS headers for browser access.
- pub `send_json` method L33-40 — `def send_json(self, status: int, data: dict) -> None`
- pub `do_OPTIONS` method L42-46 — `def do_OPTIONS(self) -> None` — Handle CORS preflight requests.
- pub `do_GET` method L48-78 — `def do_GET(self) -> None`
- pub `do_POST` method L80-108 — `def do_POST(self) -> None`
- pub `do_DELETE` method L110-117 — `def do_DELETE(self) -> None`
- pub `main` function L120-124 — `def main() -> None`

### sdks/python/brokkr/brokkr

> *Semantic summary to be generated by AI agent.*

#### sdks/python/brokkr/brokkr/client.py

- pub `BrokkrClient` class L31-222 — `{ __init__, max_retries, initial_backoff, retry, submit_manifests, apply }` — Ergonomic Brokkr broker client.
- pub `__init__` method L40-69 — `def __init__( self, base_url: str, *, token: str | None = None, request_timeout:...`
- pub `retry` method L79-115 — `def retry(self, op: Callable[[Any], Awaitable[T]]) -> T` — Run ``op(client)`` with exponential backoff on retryable failures.
- pub `submit_manifests` method L117-136 — `def submit_manifests(self, stack_id: UUID, path: Any) -> Any` — Read a folder (or file) of manifests, concatenate into one
- pub `apply` method L138-222 — `def apply( self, stack_name: str, path: Any, targeting: Optional[Sequence[str]] ...` — Idempotently make a folder of manifests the desired state of the
- pub `ApplyResult` class L227-231 — `-` — Outcome of :meth:`BrokkrClient.apply`.
-  `_expect` function L234-240 — `def _expect(result: Any, what: str) -> Any` — Unwrap a generated ``.asyncio`` result, raising on error/None.
-  `_read_manifests` function L243-277 — `def _read_manifests(path: Any) -> str` — Read a manifest path into one validated multi-document YAML stream.
-  `_sha256_hex` function L280-282 — `def _sha256_hex(content: str) -> str` — Lowercase hex SHA-256, matching the broker's deployment-object checksum.

#### sdks/python/brokkr/brokkr/errors.py

- pub `BrokkrError` class L16-62 — `(Exception) { is_retryable, from_response, from_transport }` — Single exception type surfaced by the wrapper.
- pub `__post_init__` method L30-31 — `def __post_init__(self) -> None`
- pub `__str__` method L33-38 — `def __str__(self) -> str`
- pub `is_retryable` method L40-45 — `def is_retryable(self) -> bool` — Whether this error qualifies for the wrapper's retry helper.

### sdks/python/brokkr/tests

> *Semantic summary to be generated by AI agent.*

#### sdks/python/brokkr/tests/test_wrapper.py

- pub `test_constructs_authenticated_when_token_supplied` function L15-17 — `def test_constructs_authenticated_when_token_supplied() -> None`
- pub `test_constructs_unauthenticated_when_token_omitted` function L20-22 — `def test_constructs_unauthenticated_when_token_omitted() -> None`
- pub `test_rejects_invalid_max_retries` function L25-27 — `def test_rejects_invalid_max_retries() -> None`
- pub `test_rejects_invalid_initial_backoff` function L30-32 — `def test_rejects_invalid_initial_backoff() -> None`
- pub `test_error_code_and_status_round_trip` function L35-41 — `def test_error_code_and_status_round_trip() -> None`
- pub `test_transport_error_default_retryable` function L56-59 — `def test_transport_error_default_retryable() -> None`
- pub `test_retry_returns_on_first_success` function L62-73 — `def test_retry_returns_on_first_success() -> None`
- pub `test_retry_stops_after_max_attempts_on_transport_error` function L76-88 — `def test_retry_stops_after_max_attempts_on_transport_error() -> None`
- pub `test_retry_short_circuits_on_non_retryable_status` function L91-111 — `def test_retry_short_circuits_on_non_retryable_status() -> None`
- pub `test_retry_backoff_doubles` function L114-135 — `def test_retry_backoff_doubles(monkeypatch: pytest.MonkeyPatch) -> None`
- pub `test_template_generator_reexport_resolves_to_generated_type` function L138-141 — `def test_template_generator_reexport_resolves_to_generated_type() -> None`
- pub `test_read_manifests_concatenates_folder_sorted` function L155-162 — `def test_read_manifests_concatenates_folder_sorted(tmp_path: Path) -> None`
- pub `test_read_manifests_single_file_multidoc` function L165-172 — `def test_read_manifests_single_file_multidoc(tmp_path: Path) -> None`
- pub `test_read_manifests_rejects_missing_apiversion_or_kind` function L175-178 — `def test_read_manifests_rejects_missing_apiversion_or_kind(tmp_path: Path) -> No...`
- pub `test_read_manifests_rejects_malformed_yaml` function L181-184 — `def test_read_manifests_rejects_malformed_yaml(tmp_path: Path) -> None`
- pub `test_read_manifests_errors_on_empty_and_missing` function L187-191 — `def test_read_manifests_errors_on_empty_and_missing(tmp_path: Path) -> None`
- pub `test_sha256_hex_matches_known_vector` function L194-198 — `def test_sha256_hex_matches_known_vector() -> None`
-  `_write` function L151-152 — `def _write(d: Path, name: str, content: str) -> None`

### sdks/python/brokkr-client/brokkr_broker_client

> *Semantic summary to be generated by AI agent.*

#### sdks/python/brokkr-client/brokkr_broker_client/client.py

- pub `Client` class L9-132 — `{ with_headers, with_cookies, with_timeout, set_httpx_client, get_httpx_client, ...` — A class for keeping track of data related to the API
- pub `with_headers` method L48-54 — `def with_headers(self, headers: dict[str, str]) -> "Client"` — Get a new client matching this one with additional headers
- pub `with_cookies` method L56-62 — `def with_cookies(self, cookies: dict[str, str]) -> "Client"` — Get a new client matching this one with additional cookies
- pub `with_timeout` method L64-70 — `def with_timeout(self, timeout: httpx.Timeout) -> "Client"` — Get a new client matching this one with a new timeout configuration
- pub `set_httpx_client` method L72-78 — `def set_httpx_client(self, client: httpx.Client) -> "Client"` — Manually set the underlying httpx.Client
- pub `get_httpx_client` method L80-92 — `def get_httpx_client(self) -> httpx.Client` — Get the underlying httpx.Client, constructing a new one if not previously set
- pub `__enter__` method L94-97 — `def __enter__(self) -> "Client"` — Enter a context manager for self.client—you cannot enter twice (see httpx docs)
- pub `__exit__` method L99-101 — `def __exit__(self, *args: Any, **kwargs: Any) -> None` — Exit a context manager for internal httpx.Client (see httpx docs)
- pub `set_async_httpx_client` method L103-109 — `def set_async_httpx_client(self, async_client: httpx.AsyncClient) -> "Client"` — Manually set the underlying httpx.AsyncClient
- pub `get_async_httpx_client` method L111-123 — `def get_async_httpx_client(self) -> httpx.AsyncClient` — Get the underlying httpx.AsyncClient, constructing a new one if not previously set
- pub `__aenter__` method L125-128 — `def __aenter__(self) -> "Client"` — Enter a context manager for underlying httpx.AsyncClient—you cannot enter twice (see httpx docs)
- pub `__aexit__` method L130-132 — `def __aexit__(self, *args: Any, **kwargs: Any) -> None` — Exit a context manager for underlying httpx.AsyncClient (see httpx docs)
- pub `AuthenticatedClient` class L136-268 — `{ with_headers, with_cookies, with_timeout, set_httpx_client, get_httpx_client, ...` — A Client which has been authenticated for use on secured endpoints
- pub `with_headers` method L182-188 — `def with_headers(self, headers: dict[str, str]) -> "AuthenticatedClient"` — Get a new client matching this one with additional headers
- pub `with_cookies` method L190-196 — `def with_cookies(self, cookies: dict[str, str]) -> "AuthenticatedClient"` — Get a new client matching this one with additional cookies
- pub `with_timeout` method L198-204 — `def with_timeout(self, timeout: httpx.Timeout) -> "AuthenticatedClient"` — Get a new client matching this one with a new timeout configuration
- pub `set_httpx_client` method L206-212 — `def set_httpx_client(self, client: httpx.Client) -> "AuthenticatedClient"` — Manually set the underlying httpx.Client
- pub `get_httpx_client` method L214-227 — `def get_httpx_client(self) -> httpx.Client` — Get the underlying httpx.Client, constructing a new one if not previously set
- pub `__enter__` method L229-232 — `def __enter__(self) -> "AuthenticatedClient"` — Enter a context manager for self.client—you cannot enter twice (see httpx docs)
- pub `__exit__` method L234-236 — `def __exit__(self, *args: Any, **kwargs: Any) -> None` — Exit a context manager for internal httpx.Client (see httpx docs)
- pub `set_async_httpx_client` method L238-244 — `def set_async_httpx_client(self, async_client: httpx.AsyncClient) -> "Authentica...` — Manually set the underlying httpx.AsyncClient
- pub `get_async_httpx_client` method L246-259 — `def get_async_httpx_client(self) -> httpx.AsyncClient` — Get the underlying httpx.AsyncClient, constructing a new one if not previously set
- pub `__aenter__` method L261-264 — `def __aenter__(self) -> "AuthenticatedClient"` — Enter a context manager for underlying httpx.AsyncClient—you cannot enter twice (see httpx docs)
- pub `__aexit__` method L266-268 — `def __aexit__(self, *args: Any, **kwargs: Any) -> None` — Exit a context manager for underlying httpx.AsyncClient (see httpx docs)

#### sdks/python/brokkr-client/brokkr_broker_client/errors.py

- pub `UnexpectedStatus` class L4-13 — `(Exception) { __init__ }` — Raised by api functions when the response status an undocumented status and Client.raise_on_unexpected_status is True
- pub `__init__` method L7-13 — `def __init__(self, status_code: int, content: bytes)`

#### sdks/python/brokkr-client/brokkr_broker_client/helpers.py

- pub `list_telemetry_events` function L35-54 — `def list_telemetry_events( client: AuthenticatedClient, stack_id: UUID, *, since...` — Paginated kube-event history for a stack within the 6h retention window.
- pub `list_telemetry_events_async` function L57-76 — `def list_telemetry_events_async( client: AuthenticatedClient, stack_id: UUID, *,...` — Async variant of :func:`list_telemetry_events`.
- pub `list_telemetry_logs` function L79-98 — `def list_telemetry_logs( client: AuthenticatedClient, stack_id: UUID, *, since: ...` — Paginated pod-log history for a stack within the 6h retention window.
- pub `list_telemetry_logs_async` function L101-120 — `def list_telemetry_logs_async( client: AuthenticatedClient, stack_id: UUID, *, s...` — Async variant of :func:`list_telemetry_logs`.
- pub `list_ws_connections` function L123-135 — `def list_ws_connections(client: AuthenticatedClient) -> WsConnectionsResponse` — Admin-only snapshot of currently-connected agents on the internal WS channel.
- pub `list_ws_connections_async` function L138-147 — `def list_ws_connections_async( client: AuthenticatedClient, ) -> WsConnectionsRe...` — Async variant of :func:`list_ws_connections`.
- pub `live_subscription_url` function L150-167 — `def live_subscription_url(base_url: str, stack_id: UUID) -> str` — Compute the WebSocket URL for a stack's live event + log tail.

#### sdks/python/brokkr-client/brokkr_broker_client/types.py

- pub `Unset` class L10-12 — `-`
- pub `__bool__` method L11-12 — `def __bool__(self) -> Literal[False]`
- pub `File` class L29-38 — `{ to_tuple }` — Contains information for file uploads
- pub `to_tuple` method L36-38 — `def to_tuple(self) -> FileTypes` — Return a tuple representation that httpx will accept for multipart/form-data
- pub `Response` class L45-51 — `(Generic[T])` — A response from an endpoint

### sdks/python/brokkr-client/brokkr_broker_client/api/admin

> *Semantic summary to be generated by AI agent.*

#### sdks/python/brokkr-client/brokkr_broker_client/api/admin/list_audit_logs.py

- pub `sync_detailed` function L152-228 — `def sync_detailed( *, client: AuthenticatedClient, actor_type: None | str | Unse...` — Lists audit logs with optional filtering and pagination.
- pub `sync` function L231-302 — `def sync( *, client: AuthenticatedClient, actor_type: None | str | Unset = UNSET...` — Lists audit logs with optional filtering and pagination.
- pub `asyncio_detailed` function L305-379 — `def asyncio_detailed( *, client: AuthenticatedClient, actor_type: None | str | U...` — Lists audit logs with optional filtering and pagination.
- pub `asyncio` function L382-455 — `def asyncio( *, client: AuthenticatedClient, actor_type: None | str | Unset = UN...` — Lists audit logs with optional filtering and pagination.
-  `_get_kwargs` function L15-109 — `def _get_kwargs( *, actor_type: None | str | Unset = UNSET, actor_id: None | Uns...`
-  `_parse_response` function L112-138 — `def _parse_response( *, client: AuthenticatedClient | Client, response: httpx.Re...`
-  `_build_response` function L141-149 — `def _build_response( *, client: AuthenticatedClient | Client, response: httpx.Re...`

#### sdks/python/brokkr-client/brokkr_broker_client/api/admin/list_ws_connections.py

- pub `sync_detailed` function L53-72 — `def sync_detailed( *, client: AuthenticatedClient, ) -> Response[ErrorResponse |...` — Raises:
- pub `sync` function L75-90 — `def sync( *, client: AuthenticatedClient, ) -> ErrorResponse | WsConnectionsResp...` — Raises:
- pub `asyncio_detailed` function L93-110 — `def asyncio_detailed( *, client: AuthenticatedClient, ) -> Response[ErrorRespons...` — Raises:
- pub `asyncio` function L113-130 — `def asyncio( *, client: AuthenticatedClient, ) -> ErrorResponse | WsConnectionsR...` — Raises:
-  `_get_kwargs` function L13-20 — `def _get_kwargs() -> dict[str, Any]`
-  `_parse_response` function L23-39 — `def _parse_response( *, client: AuthenticatedClient | Client, response: httpx.Re...`
-  `_build_response` function L42-50 — `def _build_response( *, client: AuthenticatedClient | Client, response: httpx.Re...`

#### sdks/python/brokkr-client/brokkr_broker_client/api/admin/reload_config.py

- pub `sync_detailed` function L63-98 — `def sync_detailed( *, client: AuthenticatedClient, ) -> Response[ConfigReloadRes...` — r"""Reloads the broker configuration from disk.
- pub `sync` function L101-132 — `def sync( *, client: AuthenticatedClient, ) -> ConfigReloadResponse | ErrorRespo...` — r"""Reloads the broker configuration from disk.
- pub `asyncio_detailed` function L135-168 — `def asyncio_detailed( *, client: AuthenticatedClient, ) -> Response[ConfigReload...` — r"""Reloads the broker configuration from disk.
- pub `asyncio` function L171-204 — `def asyncio( *, client: AuthenticatedClient, ) -> ConfigReloadResponse | ErrorRe...` — r"""Reloads the broker configuration from disk.
-  `_get_kwargs` function L13-20 — `def _get_kwargs() -> dict[str, Any]`
-  `_parse_response` function L23-49 — `def _parse_response( *, client: AuthenticatedClient | Client, response: httpx.Re...`
-  `_build_response` function L52-60 — `def _build_response( *, client: AuthenticatedClient | Client, response: httpx.Re...`

### sdks/python/brokkr-client/brokkr_broker_client/api/agent_annotations

> *Semantic summary to be generated by AI agent.*

#### sdks/python/brokkr-client/brokkr_broker_client/api/agent_annotations/agents_add_annotation.py

- pub `sync_detailed` function L73-102 — `def sync_detailed( id: UUID, *, client: AuthenticatedClient, body: NewAgentAnnot...` — Args:
- pub `sync` function L105-129 — `def sync( id: UUID, *, client: AuthenticatedClient, body: NewAgentAnnotation, ) ...` — Args:
- pub `asyncio_detailed` function L132-159 — `def asyncio_detailed( id: UUID, *, client: AuthenticatedClient, body: NewAgentAn...` — Args:
- pub `asyncio` function L162-188 — `def asyncio( id: UUID, *, client: AuthenticatedClient, body: NewAgentAnnotation,...` — Args:
-  `_get_kwargs` function L16-35 — `def _get_kwargs( id: UUID, *, body: NewAgentAnnotation, ) -> dict[str, Any]`
-  `_parse_response` function L38-59 — `def _parse_response( *, client: AuthenticatedClient | Client, response: httpx.Re...`
-  `_build_response` function L62-70 — `def _build_response( *, client: AuthenticatedClient | Client, response: httpx.Re...`

#### sdks/python/brokkr-client/brokkr_broker_client/api/agent_annotations/agents_list_annotations.py

- pub `sync_detailed` function L69-94 — `def sync_detailed( id: UUID, *, client: AuthenticatedClient, ) -> Response[Error...` — Args:
- pub `sync` function L97-117 — `def sync( id: UUID, *, client: AuthenticatedClient, ) -> ErrorResponse | list[Ag...` — Args:
- pub `asyncio_detailed` function L120-143 — `def asyncio_detailed( id: UUID, *, client: AuthenticatedClient, ) -> Response[Er...` — Args:
- pub `asyncio` function L146-168 — `def asyncio( id: UUID, *, client: AuthenticatedClient, ) -> ErrorResponse | list...` — Args:
-  `_get_kwargs` function L15-26 — `def _get_kwargs( id: UUID, ) -> dict[str, Any]`
-  `_parse_response` function L29-55 — `def _parse_response( *, client: AuthenticatedClient | Client, response: httpx.Re...`
-  `_build_response` function L58-66 — `def _build_response( *, client: AuthenticatedClient | Client, response: httpx.Re...`

#### sdks/python/brokkr-client/brokkr_broker_client/api/agent_annotations/agents_remove_annotation.py

- pub `sync_detailed` function L65-93 — `def sync_detailed( id: UUID, key: str, *, client: AuthenticatedClient, ) -> Resp...` — Args:
- pub `sync` function L96-119 — `def sync( id: UUID, key: str, *, client: AuthenticatedClient, ) -> Any | ErrorRe...` — Args:
- pub `asyncio_detailed` function L122-148 — `def asyncio_detailed( id: UUID, key: str, *, client: AuthenticatedClient, ) -> R...` — Args:
- pub `asyncio` function L151-176 — `def asyncio( id: UUID, key: str, *, client: AuthenticatedClient, ) -> Any | Erro...` — Args:
-  `_get_kwargs` function L14-27 — `def _get_kwargs( id: UUID, key: str, ) -> dict[str, Any]`
-  `_parse_response` function L30-53 — `def _parse_response(*, client: AuthenticatedClient | Client, response: httpx.Res...`
-  `_build_response` function L56-62 — `def _build_response(*, client: AuthenticatedClient | Client, response: httpx.Res...`

### sdks/python/brokkr-client/brokkr_broker_client/api/agent_events

> *Semantic summary to be generated by AI agent.*

#### sdks/python/brokkr-client/brokkr_broker_client/api/agent_events/create_event.py

- pub `sync_detailed` function L73-101 — `def sync_detailed( id: UUID, *, client: AuthenticatedClient, body: NewAgentEvent...` — Args:
- pub `sync` function L104-127 — `def sync( id: UUID, *, client: AuthenticatedClient, body: NewAgentEvent, ) -> Ag...` — Args:
- pub `asyncio_detailed` function L130-156 — `def asyncio_detailed( id: UUID, *, client: AuthenticatedClient, body: NewAgentEv...` — Args:
- pub `asyncio` function L159-184 — `def asyncio( id: UUID, *, client: AuthenticatedClient, body: NewAgentEvent, ) ->...` — Args:
-  `_get_kwargs` function L16-35 — `def _get_kwargs( id: UUID, *, body: NewAgentEvent, ) -> dict[str, Any]`
-  `_parse_response` function L38-59 — `def _parse_response( *, client: AuthenticatedClient | Client, response: httpx.Re...`
-  `_build_response` function L62-70 — `def _build_response( *, client: AuthenticatedClient | Client, response: httpx.Re...`

#### sdks/python/brokkr-client/brokkr_broker_client/api/agent_events/get_agent_event.py

- pub `sync_detailed` function L64-89 — `def sync_detailed( id: UUID, *, client: AuthenticatedClient, ) -> Response[Agent...` — Args:
- pub `sync` function L92-112 — `def sync( id: UUID, *, client: AuthenticatedClient, ) -> AgentEvent | ErrorRespo...` — Args:
- pub `asyncio_detailed` function L115-138 — `def asyncio_detailed( id: UUID, *, client: AuthenticatedClient, ) -> Response[Ag...` — Args:
- pub `asyncio` function L141-163 — `def asyncio( id: UUID, *, client: AuthenticatedClient, ) -> AgentEvent | ErrorRe...` — Args:
-  `_get_kwargs` function L15-26 — `def _get_kwargs( id: UUID, ) -> dict[str, Any]`
-  `_parse_response` function L29-50 — `def _parse_response( *, client: AuthenticatedClient | Client, response: httpx.Re...`
-  `_build_response` function L53-61 — `def _build_response( *, client: AuthenticatedClient | Client, response: httpx.Re...`

#### sdks/python/brokkr-client/brokkr_broker_client/api/agent_events/list_agent_events.py

- pub `sync_detailed` function L58-77 — `def sync_detailed( *, client: AuthenticatedClient, ) -> Response[ErrorResponse |...` — Raises:
- pub `sync` function L80-95 — `def sync( *, client: AuthenticatedClient, ) -> ErrorResponse | list[AgentEvent] ...` — Raises:
- pub `asyncio_detailed` function L98-115 — `def asyncio_detailed( *, client: AuthenticatedClient, ) -> Response[ErrorRespons...` — Raises:
- pub `asyncio` function L118-135 — `def asyncio( *, client: AuthenticatedClient, ) -> ErrorResponse | list[AgentEven...` — Raises:
-  `_get_kwargs` function L13-20 — `def _get_kwargs() -> dict[str, Any]`
-  `_parse_response` function L23-44 — `def _parse_response( *, client: AuthenticatedClient | Client, response: httpx.Re...`
-  `_build_response` function L47-55 — `def _build_response( *, client: AuthenticatedClient | Client, response: httpx.Re...`

#### sdks/python/brokkr-client/brokkr_broker_client/api/agent_events/list_events.py

- pub `sync_detailed` function L69-94 — `def sync_detailed( id: UUID, *, client: AuthenticatedClient, ) -> Response[Error...` — Args:
- pub `sync` function L97-117 — `def sync( id: UUID, *, client: AuthenticatedClient, ) -> ErrorResponse | list[Ag...` — Args:
- pub `asyncio_detailed` function L120-143 — `def asyncio_detailed( id: UUID, *, client: AuthenticatedClient, ) -> Response[Er...` — Args:
- pub `asyncio` function L146-168 — `def asyncio( id: UUID, *, client: AuthenticatedClient, ) -> ErrorResponse | list...` — Args:
-  `_get_kwargs` function L15-26 — `def _get_kwargs( id: UUID, ) -> dict[str, Any]`
-  `_parse_response` function L29-55 — `def _parse_response( *, client: AuthenticatedClient | Client, response: httpx.Re...`
-  `_build_response` function L58-66 — `def _build_response( *, client: AuthenticatedClient | Client, response: httpx.Re...`

### sdks/python/brokkr-client/brokkr_broker_client/api/agent_labels

> *Semantic summary to be generated by AI agent.*

#### sdks/python/brokkr-client/brokkr_broker_client/api/agent_labels/agents_add_label.py

- pub `sync_detailed` function L73-101 — `def sync_detailed( id: UUID, *, client: AuthenticatedClient, body: NewAgentLabel...` — Args:
- pub `sync` function L104-127 — `def sync( id: UUID, *, client: AuthenticatedClient, body: NewAgentLabel, ) -> Ag...` — Args:
- pub `asyncio_detailed` function L130-156 — `def asyncio_detailed( id: UUID, *, client: AuthenticatedClient, body: NewAgentLa...` — Args:
- pub `asyncio` function L159-184 — `def asyncio( id: UUID, *, client: AuthenticatedClient, body: NewAgentLabel, ) ->...` — Args:
-  `_get_kwargs` function L16-35 — `def _get_kwargs( id: UUID, *, body: NewAgentLabel, ) -> dict[str, Any]`
-  `_parse_response` function L38-59 — `def _parse_response( *, client: AuthenticatedClient | Client, response: httpx.Re...`
-  `_build_response` function L62-70 — `def _build_response( *, client: AuthenticatedClient | Client, response: httpx.Re...`

#### sdks/python/brokkr-client/brokkr_broker_client/api/agent_labels/agents_list_labels.py

- pub `sync_detailed` function L69-94 — `def sync_detailed( id: UUID, *, client: AuthenticatedClient, ) -> Response[Error...` — Args:
- pub `sync` function L97-117 — `def sync( id: UUID, *, client: AuthenticatedClient, ) -> ErrorResponse | list[Ag...` — Args:
- pub `asyncio_detailed` function L120-143 — `def asyncio_detailed( id: UUID, *, client: AuthenticatedClient, ) -> Response[Er...` — Args:
- pub `asyncio` function L146-168 — `def asyncio( id: UUID, *, client: AuthenticatedClient, ) -> ErrorResponse | list...` — Args:
-  `_get_kwargs` function L15-26 — `def _get_kwargs( id: UUID, ) -> dict[str, Any]`
-  `_parse_response` function L29-55 — `def _parse_response( *, client: AuthenticatedClient | Client, response: httpx.Re...`
-  `_build_response` function L58-66 — `def _build_response( *, client: AuthenticatedClient | Client, response: httpx.Re...`

#### sdks/python/brokkr-client/brokkr_broker_client/api/agent_labels/agents_remove_label.py

- pub `sync_detailed` function L65-93 — `def sync_detailed( id: UUID, label: str, *, client: AuthenticatedClient, ) -> Re...` — Args:
- pub `sync` function L96-119 — `def sync( id: UUID, label: str, *, client: AuthenticatedClient, ) -> Any | Error...` — Args:
- pub `asyncio_detailed` function L122-148 — `def asyncio_detailed( id: UUID, label: str, *, client: AuthenticatedClient, ) ->...` — Args:
- pub `asyncio` function L151-176 — `def asyncio( id: UUID, label: str, *, client: AuthenticatedClient, ) -> Any | Er...` — Args:
-  `_get_kwargs` function L14-27 — `def _get_kwargs( id: UUID, label: str, ) -> dict[str, Any]`
-  `_parse_response` function L30-53 — `def _parse_response(*, client: AuthenticatedClient | Client, response: httpx.Res...`
-  `_build_response` function L56-62 — `def _build_response(*, client: AuthenticatedClient | Client, response: httpx.Res...`

### sdks/python/brokkr-client/brokkr_broker_client/api/agent_targets

> *Semantic summary to be generated by AI agent.*

#### sdks/python/brokkr-client/brokkr_broker_client/api/agent_targets/add_target.py

- pub `sync_detailed` function L78-106 — `def sync_detailed( id: UUID, *, client: AuthenticatedClient, body: NewAgentTarge...` — Args:
- pub `sync` function L109-132 — `def sync( id: UUID, *, client: AuthenticatedClient, body: NewAgentTarget, ) -> A...` — Args:
- pub `asyncio_detailed` function L135-161 — `def asyncio_detailed( id: UUID, *, client: AuthenticatedClient, body: NewAgentTa...` — Args:
- pub `asyncio` function L164-189 — `def asyncio( id: UUID, *, client: AuthenticatedClient, body: NewAgentTarget, ) -...` — Args:
-  `_get_kwargs` function L16-35 — `def _get_kwargs( id: UUID, *, body: NewAgentTarget, ) -> dict[str, Any]`
-  `_parse_response` function L38-64 — `def _parse_response( *, client: AuthenticatedClient | Client, response: httpx.Re...`
-  `_build_response` function L67-75 — `def _build_response( *, client: AuthenticatedClient | Client, response: httpx.Re...`

#### sdks/python/brokkr-client/brokkr_broker_client/api/agent_targets/list_targets.py

- pub `sync_detailed` function L69-94 — `def sync_detailed( id: UUID, *, client: AuthenticatedClient, ) -> Response[Error...` — Args:
- pub `sync` function L97-117 — `def sync( id: UUID, *, client: AuthenticatedClient, ) -> ErrorResponse | list[Ag...` — Args:
- pub `asyncio_detailed` function L120-143 — `def asyncio_detailed( id: UUID, *, client: AuthenticatedClient, ) -> Response[Er...` — Args:
- pub `asyncio` function L146-168 — `def asyncio( id: UUID, *, client: AuthenticatedClient, ) -> ErrorResponse | list...` — Args:
-  `_get_kwargs` function L15-26 — `def _get_kwargs( id: UUID, ) -> dict[str, Any]`
-  `_parse_response` function L29-55 — `def _parse_response( *, client: AuthenticatedClient | Client, response: httpx.Re...`
-  `_build_response` function L58-66 — `def _build_response( *, client: AuthenticatedClient | Client, response: httpx.Re...`

#### sdks/python/brokkr-client/brokkr_broker_client/api/agent_targets/remove_target.py

- pub `sync_detailed` function L65-93 — `def sync_detailed( id: UUID, stack_id: UUID, *, client: AuthenticatedClient, ) -...` — Args:
- pub `sync` function L96-119 — `def sync( id: UUID, stack_id: UUID, *, client: AuthenticatedClient, ) -> Any | E...` — Args:
- pub `asyncio_detailed` function L122-148 — `def asyncio_detailed( id: UUID, stack_id: UUID, *, client: AuthenticatedClient, ...` — Args:
- pub `asyncio` function L151-176 — `def asyncio( id: UUID, stack_id: UUID, *, client: AuthenticatedClient, ) -> Any ...` — Args:
-  `_get_kwargs` function L14-27 — `def _get_kwargs( id: UUID, stack_id: UUID, ) -> dict[str, Any]`
-  `_parse_response` function L30-53 — `def _parse_response(*, client: AuthenticatedClient | Client, response: httpx.Res...`
-  `_build_response` function L56-62 — `def _build_response(*, client: AuthenticatedClient | Client, response: httpx.Res...`

### sdks/python/brokkr-client/brokkr_broker_client/api/agents

> *Semantic summary to be generated by AI agent.*

#### sdks/python/brokkr-client/brokkr_broker_client/api/agents/create_agent.py

- pub `sync_detailed` function L68-93 — `def sync_detailed( *, client: AuthenticatedClient, body: NewAgent, ) -> Response...` — Args:
- pub `sync` function L96-116 — `def sync( *, client: AuthenticatedClient, body: NewAgent, ) -> CreateAgentRespon...` — Args:
- pub `asyncio_detailed` function L119-142 — `def asyncio_detailed( *, client: AuthenticatedClient, body: NewAgent, ) -> Respo...` — Args:
- pub `asyncio` function L145-167 — `def asyncio( *, client: AuthenticatedClient, body: NewAgent, ) -> CreateAgentRes...` — Args:
-  `_get_kwargs` function L14-30 — `def _get_kwargs( *, body: NewAgent, ) -> dict[str, Any]`
-  `_parse_response` function L33-54 — `def _parse_response( *, client: AuthenticatedClient | Client, response: httpx.Re...`
-  `_build_response` function L57-65 — `def _build_response( *, client: AuthenticatedClient | Client, response: httpx.Re...`

#### sdks/python/brokkr-client/brokkr_broker_client/api/agents/delete_agent.py

- pub `sync_detailed` function L58-83 — `def sync_detailed( id: UUID, *, client: AuthenticatedClient, ) -> Response[Any |...` — Args:
- pub `sync` function L86-106 — `def sync( id: UUID, *, client: AuthenticatedClient, ) -> Any | ErrorResponse | N...` — Args:
- pub `asyncio_detailed` function L109-132 — `def asyncio_detailed( id: UUID, *, client: AuthenticatedClient, ) -> Response[An...` — Args:
- pub `asyncio` function L135-157 — `def asyncio( id: UUID, *, client: AuthenticatedClient, ) -> Any | ErrorResponse ...` — Args:
-  `_get_kwargs` function L14-25 — `def _get_kwargs( id: UUID, ) -> dict[str, Any]`
-  `_parse_response` function L28-46 — `def _parse_response(*, client: AuthenticatedClient | Client, response: httpx.Res...`
-  `_build_response` function L49-55 — `def _build_response(*, client: AuthenticatedClient | Client, response: httpx.Res...`

#### sdks/python/brokkr-client/brokkr_broker_client/api/agents/get_agent.py

- pub `sync_detailed` function L67-92 — `def sync_detailed( id: UUID, *, client: AuthenticatedClient, ) -> Response[Agent...` — Args:
- pub `sync` function L95-115 — `def sync( id: UUID, *, client: AuthenticatedClient, ) -> Agent | ErrorResponse |...` — Args:
- pub `asyncio_detailed` function L118-141 — `def asyncio_detailed( id: UUID, *, client: AuthenticatedClient, ) -> Response[Ag...` — Args:
- pub `asyncio` function L144-166 — `def asyncio( id: UUID, *, client: AuthenticatedClient, ) -> Agent | ErrorRespons...` — Args:
-  `_get_kwargs` function L15-26 — `def _get_kwargs( id: UUID, ) -> dict[str, Any]`
-  `_parse_response` function L29-53 — `def _parse_response(*, client: AuthenticatedClient | Client, response: httpx.Res...`
-  `_build_response` function L56-64 — `def _build_response( *, client: AuthenticatedClient | Client, response: httpx.Re...`

#### sdks/python/brokkr-client/brokkr_broker_client/api/agents/get_associated_stacks.py

- pub `sync_detailed` function L69-94 — `def sync_detailed( id: UUID, *, client: AuthenticatedClient, ) -> Response[Error...` — Args:
- pub `sync` function L97-117 — `def sync( id: UUID, *, client: AuthenticatedClient, ) -> ErrorResponse | list[St...` — Args:
- pub `asyncio_detailed` function L120-143 — `def asyncio_detailed( id: UUID, *, client: AuthenticatedClient, ) -> Response[Er...` — Args:
- pub `asyncio` function L146-168 — `def asyncio( id: UUID, *, client: AuthenticatedClient, ) -> ErrorResponse | list...` — Args:
-  `_get_kwargs` function L15-26 — `def _get_kwargs( id: UUID, ) -> dict[str, Any]`
-  `_parse_response` function L29-55 — `def _parse_response( *, client: AuthenticatedClient | Client, response: httpx.Re...`
-  `_build_response` function L58-66 — `def _build_response( *, client: AuthenticatedClient | Client, response: httpx.Re...`

#### sdks/python/brokkr-client/brokkr_broker_client/api/agents/get_target_state.py

- pub `sync_detailed` function L78-106 — `def sync_detailed( id: UUID, *, client: AuthenticatedClient, mode: str | Unset =...` — Args:
- pub `sync` function L109-132 — `def sync( id: UUID, *, client: AuthenticatedClient, mode: str | Unset = UNSET, )...` — Args:
- pub `asyncio_detailed` function L135-161 — `def asyncio_detailed( id: UUID, *, client: AuthenticatedClient, mode: str | Unse...` — Args:
- pub `asyncio` function L164-189 — `def asyncio( id: UUID, *, client: AuthenticatedClient, mode: str | Unset = UNSET...` — Args:
-  `_get_kwargs` function L15-35 — `def _get_kwargs( id: UUID, *, mode: str | Unset = UNSET, ) -> dict[str, Any]`
-  `_parse_response` function L38-64 — `def _parse_response( *, client: AuthenticatedClient | Client, response: httpx.Re...`
-  `_build_response` function L67-75 — `def _build_response( *, client: AuthenticatedClient | Client, response: httpx.Re...`

#### sdks/python/brokkr-client/brokkr_broker_client/api/agents/list_agents.py

- pub `sync_detailed` function L63-82 — `def sync_detailed( *, client: AuthenticatedClient, ) -> Response[ErrorResponse |...` — Raises:
- pub `sync` function L85-100 — `def sync( *, client: AuthenticatedClient, ) -> ErrorResponse | list[Agent] | Non...` — Raises:
- pub `asyncio_detailed` function L103-120 — `def asyncio_detailed( *, client: AuthenticatedClient, ) -> Response[ErrorRespons...` — Raises:
- pub `asyncio` function L123-140 — `def asyncio( *, client: AuthenticatedClient, ) -> ErrorResponse | list[Agent] | ...` — Raises:
-  `_get_kwargs` function L13-20 — `def _get_kwargs() -> dict[str, Any]`
-  `_parse_response` function L23-49 — `def _parse_response( *, client: AuthenticatedClient | Client, response: httpx.Re...`
-  `_build_response` function L52-60 — `def _build_response( *, client: AuthenticatedClient | Client, response: httpx.Re...`

#### sdks/python/brokkr-client/brokkr_broker_client/api/agents/record_heartbeat.py

- pub `sync_detailed` function L58-83 — `def sync_detailed( id: UUID, *, client: AuthenticatedClient, ) -> Response[Any |...` — Args:
- pub `sync` function L86-106 — `def sync( id: UUID, *, client: AuthenticatedClient, ) -> Any | ErrorResponse | N...` — Args:
- pub `asyncio_detailed` function L109-132 — `def asyncio_detailed( id: UUID, *, client: AuthenticatedClient, ) -> Response[An...` — Args:
- pub `asyncio` function L135-157 — `def asyncio( id: UUID, *, client: AuthenticatedClient, ) -> Any | ErrorResponse ...` — Args:
-  `_get_kwargs` function L14-25 — `def _get_kwargs( id: UUID, ) -> dict[str, Any]`
-  `_parse_response` function L28-46 — `def _parse_response(*, client: AuthenticatedClient | Client, response: httpx.Res...`
-  `_build_response` function L49-55 — `def _build_response(*, client: AuthenticatedClient | Client, response: httpx.Res...`

#### sdks/python/brokkr-client/brokkr_broker_client/api/agents/rotate_agent_pak.py

- pub `sync_detailed` function L63-88 — `def sync_detailed( id: UUID, *, client: AuthenticatedClient, ) -> Response[Any |...` — Args:
- pub `sync` function L91-111 — `def sync( id: UUID, *, client: AuthenticatedClient, ) -> Any | ErrorResponse | N...` — Args:
- pub `asyncio_detailed` function L114-137 — `def asyncio_detailed( id: UUID, *, client: AuthenticatedClient, ) -> Response[An...` — Args:
- pub `asyncio` function L140-162 — `def asyncio( id: UUID, *, client: AuthenticatedClient, ) -> Any | ErrorResponse ...` — Args:
-  `_get_kwargs` function L14-25 — `def _get_kwargs( id: UUID, ) -> dict[str, Any]`
-  `_parse_response` function L28-51 — `def _parse_response(*, client: AuthenticatedClient | Client, response: httpx.Res...`
-  `_build_response` function L54-60 — `def _build_response(*, client: AuthenticatedClient | Client, response: httpx.Res...`

#### sdks/python/brokkr-client/brokkr_broker_client/api/agents/search_agent.py

- pub `sync_detailed` function L79-107 — `def sync_detailed( *, client: AuthenticatedClient, name: str | Unset = UNSET, cl...` — Args:
- pub `sync` function L110-133 — `def sync( *, client: AuthenticatedClient, name: str | Unset = UNSET, cluster_nam...` — Args:
- pub `asyncio_detailed` function L136-162 — `def asyncio_detailed( *, client: AuthenticatedClient, name: str | Unset = UNSET,...` — Args:
- pub `asyncio` function L165-190 — `def asyncio( *, client: AuthenticatedClient, name: str | Unset = UNSET, cluster_...` — Args:
-  `_get_kwargs` function L13-33 — `def _get_kwargs( *, name: str | Unset = UNSET, cluster_name: str | Unset = UNSET...`
-  `_parse_response` function L36-65 — `def _parse_response(*, client: AuthenticatedClient | Client, response: httpx.Res...`
-  `_build_response` function L68-76 — `def _build_response( *, client: AuthenticatedClient | Client, response: httpx.Re...`

#### sdks/python/brokkr-client/brokkr_broker_client/api/agents/update_agent.py

- pub `sync_detailed` function L75-103 — `def sync_detailed( id: UUID, *, client: AuthenticatedClient, body: Any, ) -> Res...` — Args:
- pub `sync` function L106-129 — `def sync( id: UUID, *, client: AuthenticatedClient, body: Any, ) -> Agent | Erro...` — Args:
- pub `asyncio_detailed` function L132-158 — `def asyncio_detailed( id: UUID, *, client: AuthenticatedClient, body: Any, ) -> ...` — Args:
- pub `asyncio` function L161-186 — `def asyncio( id: UUID, *, client: AuthenticatedClient, body: Any, ) -> Agent | E...` — Args:
-  `_get_kwargs` function L15-34 — `def _get_kwargs( id: UUID, *, body: Any, ) -> dict[str, Any]`
-  `_parse_response` function L37-61 — `def _parse_response(*, client: AuthenticatedClient | Client, response: httpx.Res...`
-  `_build_response` function L64-72 — `def _build_response( *, client: AuthenticatedClient | Client, response: httpx.Re...`

### sdks/python/brokkr-client/brokkr_broker_client/api/auth

> *Semantic summary to be generated by AI agent.*

#### sdks/python/brokkr-client/brokkr_broker_client/api/auth/verify_pak.py

- pub `sync_detailed` function L53-75 — `def sync_detailed( *, client: AuthenticatedClient, ) -> Response[AuthResponse | ...` — Verifies a PAK (Personal Access Key) and returns an AuthResponse.
- pub `sync` function L78-96 — `def sync( *, client: AuthenticatedClient, ) -> AuthResponse | ErrorResponse | No...` — Verifies a PAK (Personal Access Key) and returns an AuthResponse.
- pub `asyncio_detailed` function L99-119 — `def asyncio_detailed( *, client: AuthenticatedClient, ) -> Response[AuthResponse...` — Verifies a PAK (Personal Access Key) and returns an AuthResponse.
- pub `asyncio` function L122-142 — `def asyncio( *, client: AuthenticatedClient, ) -> AuthResponse | ErrorResponse |...` — Verifies a PAK (Personal Access Key) and returns an AuthResponse.
-  `_get_kwargs` function L13-20 — `def _get_kwargs() -> dict[str, Any]`
-  `_parse_response` function L23-39 — `def _parse_response( *, client: AuthenticatedClient | Client, response: httpx.Re...`
-  `_build_response` function L42-50 — `def _build_response( *, client: AuthenticatedClient | Client, response: httpx.Re...`

### sdks/python/brokkr-client/brokkr_broker_client/api/deployment_objects

> *Semantic summary to be generated by AI agent.*

#### sdks/python/brokkr-client/brokkr_broker_client/api/deployment_objects/get_deployment_object.py

- pub `sync_detailed` function L74-106 — `def sync_detailed( id: UUID, *, client: AuthenticatedClient, ) -> Response[Deplo...` — Retrieves a deployment object by ID, with access control based on user role.
- pub `sync` function L109-136 — `def sync( id: UUID, *, client: AuthenticatedClient, ) -> DeploymentObject | Erro...` — Retrieves a deployment object by ID, with access control based on user role.
- pub `asyncio_detailed` function L139-169 — `def asyncio_detailed( id: UUID, *, client: AuthenticatedClient, ) -> Response[De...` — Retrieves a deployment object by ID, with access control based on user role.
- pub `asyncio` function L172-201 — `def asyncio( id: UUID, *, client: AuthenticatedClient, ) -> DeploymentObject | E...` — Retrieves a deployment object by ID, with access control based on user role.
-  `_get_kwargs` function L15-26 — `def _get_kwargs( id: UUID, ) -> dict[str, Any]`
-  `_parse_response` function L29-60 — `def _parse_response( *, client: AuthenticatedClient | Client, response: httpx.Re...`
-  `_build_response` function L63-71 — `def _build_response( *, client: AuthenticatedClient | Client, response: httpx.Re...`

### sdks/python/brokkr-client/brokkr_broker_client/api/diagnostics

> *Semantic summary to be generated by AI agent.*

#### sdks/python/brokkr-client/brokkr_broker_client/api/diagnostics/claim_diagnostic.py

- pub `sync_detailed` function L74-99 — `def sync_detailed( id: UUID, *, client: AuthenticatedClient, ) -> Response[Diagn...` — Args:
- pub `sync` function L102-122 — `def sync( id: UUID, *, client: AuthenticatedClient, ) -> DiagnosticRequest | Err...` — Args:
- pub `asyncio_detailed` function L125-148 — `def asyncio_detailed( id: UUID, *, client: AuthenticatedClient, ) -> Response[Di...` — Args:
- pub `asyncio` function L151-173 — `def asyncio( id: UUID, *, client: AuthenticatedClient, ) -> DiagnosticRequest | ...` — Args:
-  `_get_kwargs` function L15-26 — `def _get_kwargs( id: UUID, ) -> dict[str, Any]`
-  `_parse_response` function L29-60 — `def _parse_response( *, client: AuthenticatedClient | Client, response: httpx.Re...`
-  `_build_response` function L63-71 — `def _build_response( *, client: AuthenticatedClient | Client, response: httpx.Re...`

#### sdks/python/brokkr-client/brokkr_broker_client/api/diagnostics/create_diagnostic_request.py

- pub `sync_detailed` function L83-111 — `def sync_detailed( id: UUID, *, client: AuthenticatedClient, body: CreateDiagnos...` — Args:
- pub `sync` function L114-137 — `def sync( id: UUID, *, client: AuthenticatedClient, body: CreateDiagnosticReques...` — Args:
- pub `asyncio_detailed` function L140-166 — `def asyncio_detailed( id: UUID, *, client: AuthenticatedClient, body: CreateDiag...` — Args:
- pub `asyncio` function L169-194 — `def asyncio( id: UUID, *, client: AuthenticatedClient, body: CreateDiagnosticReq...` — Args:
-  `_get_kwargs` function L16-35 — `def _get_kwargs( id: UUID, *, body: CreateDiagnosticRequest, ) -> dict[str, Any]`
-  `_parse_response` function L38-69 — `def _parse_response( *, client: AuthenticatedClient | Client, response: httpx.Re...`
-  `_build_response` function L72-80 — `def _build_response( *, client: AuthenticatedClient | Client, response: httpx.Re...`

#### sdks/python/brokkr-client/brokkr_broker_client/api/diagnostics/get_diagnostic.py

- pub `sync_detailed` function L69-94 — `def sync_detailed( id: UUID, *, client: AuthenticatedClient, ) -> Response[Diagn...` — Args:
- pub `sync` function L97-117 — `def sync( id: UUID, *, client: AuthenticatedClient, ) -> DiagnosticResponse | Er...` — Args:
- pub `asyncio_detailed` function L120-143 — `def asyncio_detailed( id: UUID, *, client: AuthenticatedClient, ) -> Response[Di...` — Args:
- pub `asyncio` function L146-168 — `def asyncio( id: UUID, *, client: AuthenticatedClient, ) -> DiagnosticResponse |...` — Args:
-  `_get_kwargs` function L15-26 — `def _get_kwargs( id: UUID, ) -> dict[str, Any]`
-  `_parse_response` function L29-55 — `def _parse_response( *, client: AuthenticatedClient | Client, response: httpx.Re...`
-  `_build_response` function L58-66 — `def _build_response( *, client: AuthenticatedClient | Client, response: httpx.Re...`

#### sdks/python/brokkr-client/brokkr_broker_client/api/diagnostics/get_pending_diagnostics.py

- pub `sync_detailed` function L69-94 — `def sync_detailed( id: UUID, *, client: AuthenticatedClient, ) -> Response[Error...` — Args:
- pub `sync` function L97-117 — `def sync( id: UUID, *, client: AuthenticatedClient, ) -> ErrorResponse | list[Di...` — Args:
- pub `asyncio_detailed` function L120-143 — `def asyncio_detailed( id: UUID, *, client: AuthenticatedClient, ) -> Response[Er...` — Args:
- pub `asyncio` function L146-168 — `def asyncio( id: UUID, *, client: AuthenticatedClient, ) -> ErrorResponse | list...` — Args:
-  `_get_kwargs` function L15-26 — `def _get_kwargs( id: UUID, ) -> dict[str, Any]`
-  `_parse_response` function L29-55 — `def _parse_response( *, client: AuthenticatedClient | Client, response: httpx.Re...`
-  `_build_response` function L58-66 — `def _build_response( *, client: AuthenticatedClient | Client, response: httpx.Re...`

#### sdks/python/brokkr-client/brokkr_broker_client/api/diagnostics/submit_diagnostic_result.py

- pub `sync_detailed` function L88-116 — `def sync_detailed( id: UUID, *, client: AuthenticatedClient, body: SubmitDiagnos...` — Args:
- pub `sync` function L119-142 — `def sync( id: UUID, *, client: AuthenticatedClient, body: SubmitDiagnosticResult...` — Args:
- pub `asyncio_detailed` function L145-171 — `def asyncio_detailed( id: UUID, *, client: AuthenticatedClient, body: SubmitDiag...` — Args:
- pub `asyncio` function L174-199 — `def asyncio( id: UUID, *, client: AuthenticatedClient, body: SubmitDiagnosticRes...` — Args:
-  `_get_kwargs` function L16-35 — `def _get_kwargs( id: UUID, *, body: SubmitDiagnosticResult, ) -> dict[str, Any]`
-  `_parse_response` function L38-74 — `def _parse_response( *, client: AuthenticatedClient | Client, response: httpx.Re...`
-  `_build_response` function L77-85 — `def _build_response( *, client: AuthenticatedClient | Client, response: httpx.Re...`

### sdks/python/brokkr-client/brokkr_broker_client/api/generators

> *Semantic summary to be generated by AI agent.*

#### sdks/python/brokkr-client/brokkr_broker_client/api/generators/create_generator.py

- pub `sync_detailed` function L78-103 — `def sync_detailed( *, client: AuthenticatedClient, body: NewGenerator, ) -> Resp...` — Args:
- pub `sync` function L106-126 — `def sync( *, client: AuthenticatedClient, body: NewGenerator, ) -> CreateGenerat...` — Args:
- pub `asyncio_detailed` function L129-152 — `def asyncio_detailed( *, client: AuthenticatedClient, body: NewGenerator, ) -> R...` — Args:
- pub `asyncio` function L155-177 — `def asyncio( *, client: AuthenticatedClient, body: NewGenerator, ) -> CreateGene...` — Args:
-  `_get_kwargs` function L14-30 — `def _get_kwargs( *, body: NewGenerator, ) -> dict[str, Any]`
-  `_parse_response` function L33-64 — `def _parse_response( *, client: AuthenticatedClient | Client, response: httpx.Re...`
-  `_build_response` function L67-75 — `def _build_response( *, client: AuthenticatedClient | Client, response: httpx.Re...`

#### sdks/python/brokkr-client/brokkr_broker_client/api/generators/delete_generator.py

- pub `sync_detailed` function L63-88 — `def sync_detailed( id: UUID, *, client: AuthenticatedClient, ) -> Response[Any |...` — Args:
- pub `sync` function L91-111 — `def sync( id: UUID, *, client: AuthenticatedClient, ) -> Any | ErrorResponse | N...` — Args:
- pub `asyncio_detailed` function L114-137 — `def asyncio_detailed( id: UUID, *, client: AuthenticatedClient, ) -> Response[An...` — Args:
- pub `asyncio` function L140-162 — `def asyncio( id: UUID, *, client: AuthenticatedClient, ) -> Any | ErrorResponse ...` — Args:
-  `_get_kwargs` function L14-25 — `def _get_kwargs( id: UUID, ) -> dict[str, Any]`
-  `_parse_response` function L28-51 — `def _parse_response(*, client: AuthenticatedClient | Client, response: httpx.Res...`
-  `_build_response` function L54-60 — `def _build_response(*, client: AuthenticatedClient | Client, response: httpx.Res...`

#### sdks/python/brokkr-client/brokkr_broker_client/api/generators/get_generator.py

- pub `sync_detailed` function L69-94 — `def sync_detailed( id: UUID, *, client: AuthenticatedClient, ) -> Response[Error...` — Args:
- pub `sync` function L97-117 — `def sync( id: UUID, *, client: AuthenticatedClient, ) -> ErrorResponse | Generat...` — Args:
- pub `asyncio_detailed` function L120-143 — `def asyncio_detailed( id: UUID, *, client: AuthenticatedClient, ) -> Response[Er...` — Args:
- pub `asyncio` function L146-168 — `def asyncio( id: UUID, *, client: AuthenticatedClient, ) -> ErrorResponse | Gene...` — Args:
-  `_get_kwargs` function L15-26 — `def _get_kwargs( id: UUID, ) -> dict[str, Any]`
-  `_parse_response` function L29-55 — `def _parse_response( *, client: AuthenticatedClient | Client, response: httpx.Re...`
-  `_build_response` function L58-66 — `def _build_response( *, client: AuthenticatedClient | Client, response: httpx.Re...`

#### sdks/python/brokkr-client/brokkr_broker_client/api/generators/list_generators.py

- pub `sync_detailed` function L63-82 — `def sync_detailed( *, client: AuthenticatedClient, ) -> Response[ErrorResponse |...` — Raises:
- pub `sync` function L85-100 — `def sync( *, client: AuthenticatedClient, ) -> ErrorResponse | list[Generator] |...` — Raises:
- pub `asyncio_detailed` function L103-120 — `def asyncio_detailed( *, client: AuthenticatedClient, ) -> Response[ErrorRespons...` — Raises:
- pub `asyncio` function L123-140 — `def asyncio( *, client: AuthenticatedClient, ) -> ErrorResponse | list[Generator...` — Raises:
-  `_get_kwargs` function L13-20 — `def _get_kwargs() -> dict[str, Any]`
-  `_parse_response` function L23-49 — `def _parse_response( *, client: AuthenticatedClient | Client, response: httpx.Re...`
-  `_build_response` function L52-60 — `def _build_response( *, client: AuthenticatedClient | Client, response: httpx.Re...`

#### sdks/python/brokkr-client/brokkr_broker_client/api/generators/rotate_generator_pak.py

- pub `sync_detailed` function L69-94 — `def sync_detailed( id: UUID, *, client: AuthenticatedClient, ) -> Response[Creat...` — Args:
- pub `sync` function L97-117 — `def sync( id: UUID, *, client: AuthenticatedClient, ) -> CreateGeneratorResponse...` — Args:
- pub `asyncio_detailed` function L120-143 — `def asyncio_detailed( id: UUID, *, client: AuthenticatedClient, ) -> Response[Cr...` — Args:
- pub `asyncio` function L146-168 — `def asyncio( id: UUID, *, client: AuthenticatedClient, ) -> CreateGeneratorRespo...` — Args:
-  `_get_kwargs` function L15-26 — `def _get_kwargs( id: UUID, ) -> dict[str, Any]`
-  `_parse_response` function L29-55 — `def _parse_response( *, client: AuthenticatedClient | Client, response: httpx.Re...`
-  `_build_response` function L58-66 — `def _build_response( *, client: AuthenticatedClient | Client, response: httpx.Re...`

#### sdks/python/brokkr-client/brokkr_broker_client/api/generators/update_generator.py

- pub `sync_detailed` function L77-105 — `def sync_detailed( id: UUID, *, client: AuthenticatedClient, body: Generator, ) ...` — Args:
- pub `sync` function L108-131 — `def sync( id: UUID, *, client: AuthenticatedClient, body: Generator, ) -> ErrorR...` — Args:
- pub `asyncio_detailed` function L134-160 — `def asyncio_detailed( id: UUID, *, client: AuthenticatedClient, body: Generator,...` — Args:
- pub `asyncio` function L163-188 — `def asyncio( id: UUID, *, client: AuthenticatedClient, body: Generator, ) -> Err...` — Args:
-  `_get_kwargs` function L15-34 — `def _get_kwargs( id: UUID, *, body: Generator, ) -> dict[str, Any]`
-  `_parse_response` function L37-63 — `def _parse_response( *, client: AuthenticatedClient | Client, response: httpx.Re...`
-  `_build_response` function L66-74 — `def _build_response( *, client: AuthenticatedClient | Client, response: httpx.Re...`

### sdks/python/brokkr-client/brokkr_broker_client/api/health

> *Semantic summary to be generated by AI agent.*

#### sdks/python/brokkr-client/brokkr_broker_client/api/health/get_deployment_health.py

- pub `sync_detailed` function L64-93 — `def sync_detailed( id: UUID, *, client: AuthenticatedClient, ) -> Response[Deplo...` — Gets health status for a specific deployment object.
- pub `sync` function L96-120 — `def sync( id: UUID, *, client: AuthenticatedClient, ) -> DeploymentHealthRespons...` — Gets health status for a specific deployment object.
- pub `asyncio_detailed` function L123-150 — `def asyncio_detailed( id: UUID, *, client: AuthenticatedClient, ) -> Response[De...` — Gets health status for a specific deployment object.
- pub `asyncio` function L153-179 — `def asyncio( id: UUID, *, client: AuthenticatedClient, ) -> DeploymentHealthResp...` — Gets health status for a specific deployment object.
-  `_get_kwargs` function L15-26 — `def _get_kwargs( id: UUID, ) -> dict[str, Any]`
-  `_parse_response` function L29-50 — `def _parse_response( *, client: AuthenticatedClient | Client, response: httpx.Re...`
-  `_build_response` function L53-61 — `def _build_response( *, client: AuthenticatedClient | Client, response: httpx.Re...`

#### sdks/python/brokkr-client/brokkr_broker_client/api/health/get_stack_health.py

- pub `sync_detailed` function L64-93 — `def sync_detailed( id: UUID, *, client: AuthenticatedClient, ) -> Response[Error...` — Gets health status for all deployment objects in a stack.
- pub `sync` function L96-120 — `def sync( id: UUID, *, client: AuthenticatedClient, ) -> ErrorResponse | StackHe...` — Gets health status for all deployment objects in a stack.
- pub `asyncio_detailed` function L123-150 — `def asyncio_detailed( id: UUID, *, client: AuthenticatedClient, ) -> Response[Er...` — Gets health status for all deployment objects in a stack.
- pub `asyncio` function L153-179 — `def asyncio( id: UUID, *, client: AuthenticatedClient, ) -> ErrorResponse | Stac...` — Gets health status for all deployment objects in a stack.
-  `_get_kwargs` function L15-26 — `def _get_kwargs( id: UUID, ) -> dict[str, Any]`
-  `_parse_response` function L29-50 — `def _parse_response( *, client: AuthenticatedClient | Client, response: httpx.Re...`
-  `_build_response` function L53-61 — `def _build_response( *, client: AuthenticatedClient | Client, response: httpx.Re...`

#### sdks/python/brokkr-client/brokkr_broker_client/api/health/update_health_status.py

- pub `sync_detailed` function L67-99 — `def sync_detailed( id: UUID, *, client: AuthenticatedClient, body: HealthStatusU...` — Updates health status for deployment objects from an agent.
- pub `sync` function L102-129 — `def sync( id: UUID, *, client: AuthenticatedClient, body: HealthStatusUpdate, ) ...` — Updates health status for deployment objects from an agent.
- pub `asyncio_detailed` function L132-162 — `def asyncio_detailed( id: UUID, *, client: AuthenticatedClient, body: HealthStat...` — Updates health status for deployment objects from an agent.
- pub `asyncio` function L165-194 — `def asyncio( id: UUID, *, client: AuthenticatedClient, body: HealthStatusUpdate,...` — Updates health status for deployment objects from an agent.
-  `_get_kwargs` function L15-34 — `def _get_kwargs( id: UUID, *, body: HealthStatusUpdate, ) -> dict[str, Any]`
-  `_parse_response` function L37-55 — `def _parse_response(*, client: AuthenticatedClient | Client, response: httpx.Res...`
-  `_build_response` function L58-64 — `def _build_response(*, client: AuthenticatedClient | Client, response: httpx.Res...`

### sdks/python/brokkr-client/brokkr_broker_client/api/stack_telemetry

> *Semantic summary to be generated by AI agent.*

#### sdks/python/brokkr-client/brokkr_broker_client/api/stack_telemetry/list_telemetry_events.py

- pub `sync_detailed` function L89-120 — `def sync_detailed( id: UUID, *, client: AuthenticatedClient, since: datetime.dat...` — Args:
- pub `sync` function L123-149 — `def sync( id: UUID, *, client: AuthenticatedClient, since: datetime.datetime | N...` — Args:
- pub `asyncio_detailed` function L152-181 — `def asyncio_detailed( id: UUID, *, client: AuthenticatedClient, since: datetime....` — Args:
- pub `asyncio` function L184-212 — `def asyncio( id: UUID, *, client: AuthenticatedClient, since: datetime.datetime ...` — Args:
-  `_get_kwargs` function L16-51 — `def _get_kwargs( id: UUID, *, since: datetime.datetime | None | Unset = UNSET, l...`
-  `_parse_response` function L54-75 — `def _parse_response( *, client: AuthenticatedClient | Client, response: httpx.Re...`
-  `_build_response` function L78-86 — `def _build_response( *, client: AuthenticatedClient | Client, response: httpx.Re...`

#### sdks/python/brokkr-client/brokkr_broker_client/api/stack_telemetry/list_telemetry_logs.py

- pub `sync_detailed` function L89-120 — `def sync_detailed( id: UUID, *, client: AuthenticatedClient, since: datetime.dat...` — Args:
- pub `sync` function L123-149 — `def sync( id: UUID, *, client: AuthenticatedClient, since: datetime.datetime | N...` — Args:
- pub `asyncio_detailed` function L152-181 — `def asyncio_detailed( id: UUID, *, client: AuthenticatedClient, since: datetime....` — Args:
- pub `asyncio` function L184-212 — `def asyncio( id: UUID, *, client: AuthenticatedClient, since: datetime.datetime ...` — Args:
-  `_get_kwargs` function L16-51 — `def _get_kwargs( id: UUID, *, since: datetime.datetime | None | Unset = UNSET, l...`
-  `_parse_response` function L54-75 — `def _parse_response( *, client: AuthenticatedClient | Client, response: httpx.Re...`
-  `_build_response` function L78-86 — `def _build_response( *, client: AuthenticatedClient | Client, response: httpx.Re...`

### sdks/python/brokkr-client/brokkr_broker_client/api/stacks

> *Semantic summary to be generated by AI agent.*

#### sdks/python/brokkr-client/brokkr_broker_client/api/stacks/create_deployment_object.py

- pub `sync_detailed` function L96-131 — `def sync_detailed( id: UUID, *, client: AuthenticatedClient, body: CreateDeploym...` — Args:
- pub `sync` function L134-164 — `def sync( id: UUID, *, client: AuthenticatedClient, body: CreateDeploymentObject...` — Args:
- pub `asyncio_detailed` function L167-200 — `def asyncio_detailed( id: UUID, *, client: AuthenticatedClient, body: CreateDepl...` — Args:
- pub `asyncio` function L203-235 — `def asyncio( id: UUID, *, client: AuthenticatedClient, body: CreateDeploymentObj...` — Args:
-  `_get_kwargs` function L16-48 — `def _get_kwargs( id: UUID, *, body: CreateDeploymentObjectRequest, deletion_mark...`
-  `_parse_response` function L51-82 — `def _parse_response( *, client: AuthenticatedClient | Client, response: httpx.Re...`
-  `_build_response` function L85-93 — `def _build_response( *, client: AuthenticatedClient | Client, response: httpx.Re...`

#### sdks/python/brokkr-client/brokkr_broker_client/api/stacks/create_stack.py

- pub `sync_detailed` function L66-91 — `def sync_detailed( *, client: AuthenticatedClient, body: NewStack, ) -> Response...` — Args:
- pub `sync` function L94-114 — `def sync( *, client: AuthenticatedClient, body: NewStack, ) -> ErrorResponse | S...` — Args:
- pub `asyncio_detailed` function L117-140 — `def asyncio_detailed( *, client: AuthenticatedClient, body: NewStack, ) -> Respo...` — Args:
- pub `asyncio` function L143-165 — `def asyncio( *, client: AuthenticatedClient, body: NewStack, ) -> ErrorResponse ...` — Args:
-  `_get_kwargs` function L14-30 — `def _get_kwargs( *, body: NewStack, ) -> dict[str, Any]`
-  `_parse_response` function L33-52 — `def _parse_response(*, client: AuthenticatedClient | Client, response: httpx.Res...`
-  `_build_response` function L55-63 — `def _build_response( *, client: AuthenticatedClient | Client, response: httpx.Re...`

#### sdks/python/brokkr-client/brokkr_broker_client/api/stacks/delete_stack.py

- pub `sync_detailed` function L63-88 — `def sync_detailed( id: UUID, *, client: AuthenticatedClient, ) -> Response[Any |...` — Args:
- pub `sync` function L91-111 — `def sync( id: UUID, *, client: AuthenticatedClient, ) -> Any | ErrorResponse | N...` — Args:
- pub `asyncio_detailed` function L114-137 — `def asyncio_detailed( id: UUID, *, client: AuthenticatedClient, ) -> Response[An...` — Args:
- pub `asyncio` function L140-162 — `def asyncio( id: UUID, *, client: AuthenticatedClient, ) -> Any | ErrorResponse ...` — Args:
-  `_get_kwargs` function L14-25 — `def _get_kwargs( id: UUID, ) -> dict[str, Any]`
-  `_parse_response` function L28-51 — `def _parse_response(*, client: AuthenticatedClient | Client, response: httpx.Res...`
-  `_build_response` function L54-60 — `def _build_response(*, client: AuthenticatedClient | Client, response: httpx.Res...`

#### sdks/python/brokkr-client/brokkr_broker_client/api/stacks/get_stack.py

- pub `sync_detailed` function L67-92 — `def sync_detailed( id: UUID, *, client: AuthenticatedClient, ) -> Response[Error...` — Args:
- pub `sync` function L95-115 — `def sync( id: UUID, *, client: AuthenticatedClient, ) -> ErrorResponse | Stack |...` — Args:
- pub `asyncio_detailed` function L118-141 — `def asyncio_detailed( id: UUID, *, client: AuthenticatedClient, ) -> Response[Er...` — Args:
- pub `asyncio` function L144-166 — `def asyncio( id: UUID, *, client: AuthenticatedClient, ) -> ErrorResponse | Stac...` — Args:
-  `_get_kwargs` function L15-26 — `def _get_kwargs( id: UUID, ) -> dict[str, Any]`
-  `_parse_response` function L29-53 — `def _parse_response(*, client: AuthenticatedClient | Client, response: httpx.Res...`
-  `_build_response` function L56-64 — `def _build_response( *, client: AuthenticatedClient | Client, response: httpx.Re...`

#### sdks/python/brokkr-client/brokkr_broker_client/api/stacks/instantiate_template.py

- pub `sync_detailed` function L88-116 — `def sync_detailed( stack_id: UUID, *, client: AuthenticatedClient, body: Templat...` — Args:
- pub `sync` function L119-142 — `def sync( stack_id: UUID, *, client: AuthenticatedClient, body: TemplateInstanti...` — Args:
- pub `asyncio_detailed` function L145-171 — `def asyncio_detailed( stack_id: UUID, *, client: AuthenticatedClient, body: Temp...` — Args:
- pub `asyncio` function L174-199 — `def asyncio( stack_id: UUID, *, client: AuthenticatedClient, body: TemplateInsta...` — Args:
-  `_get_kwargs` function L16-35 — `def _get_kwargs( stack_id: UUID, *, body: TemplateInstantiationRequest, ) -> dic...`
-  `_parse_response` function L38-74 — `def _parse_response( *, client: AuthenticatedClient | Client, response: httpx.Re...`
-  `_build_response` function L77-85 — `def _build_response( *, client: AuthenticatedClient | Client, response: httpx.Re...`

#### sdks/python/brokkr-client/brokkr_broker_client/api/stacks/list_deployment_objects.py

- pub `sync_detailed` function L74-99 — `def sync_detailed( id: UUID, *, client: AuthenticatedClient, ) -> Response[Error...` — Args:
- pub `sync` function L102-122 — `def sync( id: UUID, *, client: AuthenticatedClient, ) -> ErrorResponse | list[De...` — Args:
- pub `asyncio_detailed` function L125-148 — `def asyncio_detailed( id: UUID, *, client: AuthenticatedClient, ) -> Response[Er...` — Args:
- pub `asyncio` function L151-173 — `def asyncio( id: UUID, *, client: AuthenticatedClient, ) -> ErrorResponse | list...` — Args:
-  `_get_kwargs` function L15-26 — `def _get_kwargs( id: UUID, ) -> dict[str, Any]`
-  `_parse_response` function L29-60 — `def _parse_response( *, client: AuthenticatedClient | Client, response: httpx.Re...`
-  `_build_response` function L63-71 — `def _build_response( *, client: AuthenticatedClient | Client, response: httpx.Re...`

#### sdks/python/brokkr-client/brokkr_broker_client/api/stacks/list_stacks.py

- pub `sync_detailed` function L63-82 — `def sync_detailed( *, client: AuthenticatedClient, ) -> Response[ErrorResponse |...` — Raises:
- pub `sync` function L85-100 — `def sync( *, client: AuthenticatedClient, ) -> ErrorResponse | list[Stack] | Non...` — Raises:
- pub `asyncio_detailed` function L103-120 — `def asyncio_detailed( *, client: AuthenticatedClient, ) -> Response[ErrorRespons...` — Raises:
- pub `asyncio` function L123-140 — `def asyncio( *, client: AuthenticatedClient, ) -> ErrorResponse | list[Stack] | ...` — Raises:
-  `_get_kwargs` function L13-20 — `def _get_kwargs() -> dict[str, Any]`
-  `_parse_response` function L23-49 — `def _parse_response( *, client: AuthenticatedClient | Client, response: httpx.Re...`
-  `_build_response` function L52-60 — `def _build_response( *, client: AuthenticatedClient | Client, response: httpx.Re...`

#### sdks/python/brokkr-client/brokkr_broker_client/api/stacks/stacks_add_annotation.py

- pub `sync_detailed` function L83-112 — `def sync_detailed( id: UUID, *, client: AuthenticatedClient, body: NewStackAnnot...` — Args:
- pub `sync` function L115-139 — `def sync( id: UUID, *, client: AuthenticatedClient, body: NewStackAnnotation, ) ...` — Args:
- pub `asyncio_detailed` function L142-169 — `def asyncio_detailed( id: UUID, *, client: AuthenticatedClient, body: NewStackAn...` — Args:
- pub `asyncio` function L172-198 — `def asyncio( id: UUID, *, client: AuthenticatedClient, body: NewStackAnnotation,...` — Args:
-  `_get_kwargs` function L16-35 — `def _get_kwargs( id: UUID, *, body: NewStackAnnotation, ) -> dict[str, Any]`
-  `_parse_response` function L38-69 — `def _parse_response( *, client: AuthenticatedClient | Client, response: httpx.Re...`
-  `_build_response` function L72-80 — `def _build_response( *, client: AuthenticatedClient | Client, response: httpx.Re...`

#### sdks/python/brokkr-client/brokkr_broker_client/api/stacks/stacks_add_label.py

- pub `sync_detailed` function L82-110 — `def sync_detailed( id: UUID, *, client: AuthenticatedClient, body: str, ) -> Res...` — Args:
- pub `sync` function L113-136 — `def sync( id: UUID, *, client: AuthenticatedClient, body: str, ) -> ErrorRespons...` — Args:
- pub `asyncio_detailed` function L139-165 — `def asyncio_detailed( id: UUID, *, client: AuthenticatedClient, body: str, ) -> ...` — Args:
- pub `asyncio` function L168-193 — `def asyncio( id: UUID, *, client: AuthenticatedClient, body: str, ) -> ErrorResp...` — Args:
-  `_get_kwargs` function L15-34 — `def _get_kwargs( id: UUID, *, body: str, ) -> dict[str, Any]`
-  `_parse_response` function L37-68 — `def _parse_response( *, client: AuthenticatedClient | Client, response: httpx.Re...`
-  `_build_response` function L71-79 — `def _build_response( *, client: AuthenticatedClient | Client, response: httpx.Re...`

#### sdks/python/brokkr-client/brokkr_broker_client/api/stacks/stacks_list_annotations.py

- pub `sync_detailed` function L74-99 — `def sync_detailed( id: UUID, *, client: AuthenticatedClient, ) -> Response[Error...` — Args:
- pub `sync` function L102-122 — `def sync( id: UUID, *, client: AuthenticatedClient, ) -> ErrorResponse | list[St...` — Args:
- pub `asyncio_detailed` function L125-148 — `def asyncio_detailed( id: UUID, *, client: AuthenticatedClient, ) -> Response[Er...` — Args:
- pub `asyncio` function L151-173 — `def asyncio( id: UUID, *, client: AuthenticatedClient, ) -> ErrorResponse | list...` — Args:
-  `_get_kwargs` function L15-26 — `def _get_kwargs( id: UUID, ) -> dict[str, Any]`
-  `_parse_response` function L29-60 — `def _parse_response( *, client: AuthenticatedClient | Client, response: httpx.Re...`
-  `_build_response` function L63-71 — `def _build_response( *, client: AuthenticatedClient | Client, response: httpx.Re...`

#### sdks/python/brokkr-client/brokkr_broker_client/api/stacks/stacks_list_labels.py

- pub `sync_detailed` function L74-99 — `def sync_detailed( id: UUID, *, client: AuthenticatedClient, ) -> Response[Error...` — Args:
- pub `sync` function L102-122 — `def sync( id: UUID, *, client: AuthenticatedClient, ) -> ErrorResponse | list[St...` — Args:
- pub `asyncio_detailed` function L125-148 — `def asyncio_detailed( id: UUID, *, client: AuthenticatedClient, ) -> Response[Er...` — Args:
- pub `asyncio` function L151-173 — `def asyncio( id: UUID, *, client: AuthenticatedClient, ) -> ErrorResponse | list...` — Args:
-  `_get_kwargs` function L15-26 — `def _get_kwargs( id: UUID, ) -> dict[str, Any]`
-  `_parse_response` function L29-60 — `def _parse_response( *, client: AuthenticatedClient | Client, response: httpx.Re...`
-  `_build_response` function L63-71 — `def _build_response( *, client: AuthenticatedClient | Client, response: httpx.Re...`

#### sdks/python/brokkr-client/brokkr_broker_client/api/stacks/stacks_remove_annotation.py

- pub `sync_detailed` function L65-93 — `def sync_detailed( id: UUID, key: str, *, client: AuthenticatedClient, ) -> Resp...` — Args:
- pub `sync` function L96-119 — `def sync( id: UUID, key: str, *, client: AuthenticatedClient, ) -> Any | ErrorRe...` — Args:
- pub `asyncio_detailed` function L122-148 — `def asyncio_detailed( id: UUID, key: str, *, client: AuthenticatedClient, ) -> R...` — Args:
- pub `asyncio` function L151-176 — `def asyncio( id: UUID, key: str, *, client: AuthenticatedClient, ) -> Any | Erro...` — Args:
-  `_get_kwargs` function L14-27 — `def _get_kwargs( id: UUID, key: str, ) -> dict[str, Any]`
-  `_parse_response` function L30-53 — `def _parse_response(*, client: AuthenticatedClient | Client, response: httpx.Res...`
-  `_build_response` function L56-62 — `def _build_response(*, client: AuthenticatedClient | Client, response: httpx.Res...`

#### sdks/python/brokkr-client/brokkr_broker_client/api/stacks/stacks_remove_label.py

- pub `sync_detailed` function L65-93 — `def sync_detailed( id: UUID, label: str, *, client: AuthenticatedClient, ) -> Re...` — Args:
- pub `sync` function L96-119 — `def sync( id: UUID, label: str, *, client: AuthenticatedClient, ) -> Any | Error...` — Args:
- pub `asyncio_detailed` function L122-148 — `def asyncio_detailed( id: UUID, label: str, *, client: AuthenticatedClient, ) ->...` — Args:
- pub `asyncio` function L151-176 — `def asyncio( id: UUID, label: str, *, client: AuthenticatedClient, ) -> Any | Er...` — Args:
-  `_get_kwargs` function L14-27 — `def _get_kwargs( id: UUID, label: str, ) -> dict[str, Any]`
-  `_parse_response` function L30-53 — `def _parse_response(*, client: AuthenticatedClient | Client, response: httpx.Res...`
-  `_build_response` function L56-62 — `def _build_response(*, client: AuthenticatedClient | Client, response: httpx.Res...`

#### sdks/python/brokkr-client/brokkr_broker_client/api/stacks/update_stack.py

- pub `sync_detailed` function L80-108 — `def sync_detailed( id: UUID, *, client: AuthenticatedClient, body: Stack, ) -> R...` — Args:
- pub `sync` function L111-134 — `def sync( id: UUID, *, client: AuthenticatedClient, body: Stack, ) -> ErrorRespo...` — Args:
- pub `asyncio_detailed` function L137-163 — `def asyncio_detailed( id: UUID, *, client: AuthenticatedClient, body: Stack, ) -...` — Args:
- pub `asyncio` function L166-191 — `def asyncio( id: UUID, *, client: AuthenticatedClient, body: Stack, ) -> ErrorRe...` — Args:
-  `_get_kwargs` function L15-34 — `def _get_kwargs( id: UUID, *, body: Stack, ) -> dict[str, Any]`
-  `_parse_response` function L37-66 — `def _parse_response(*, client: AuthenticatedClient | Client, response: httpx.Res...`
-  `_build_response` function L69-77 — `def _build_response( *, client: AuthenticatedClient | Client, response: httpx.Re...`

### sdks/python/brokkr-client/brokkr_broker_client/api/templates

> *Semantic summary to be generated by AI agent.*

#### sdks/python/brokkr-client/brokkr_broker_client/api/templates/create_template.py

- pub `sync_detailed` function L73-98 — `def sync_detailed( *, client: AuthenticatedClient, body: CreateTemplateRequest, ...` — Args:
- pub `sync` function L101-121 — `def sync( *, client: AuthenticatedClient, body: CreateTemplateRequest, ) -> Erro...` — Args:
- pub `asyncio_detailed` function L124-147 — `def asyncio_detailed( *, client: AuthenticatedClient, body: CreateTemplateReques...` — Args:
- pub `asyncio` function L150-172 — `def asyncio( *, client: AuthenticatedClient, body: CreateTemplateRequest, ) -> E...` — Args:
-  `_get_kwargs` function L14-30 — `def _get_kwargs( *, body: CreateTemplateRequest, ) -> dict[str, Any]`
-  `_parse_response` function L33-59 — `def _parse_response( *, client: AuthenticatedClient | Client, response: httpx.Re...`
-  `_build_response` function L62-70 — `def _build_response( *, client: AuthenticatedClient | Client, response: httpx.Re...`

#### sdks/python/brokkr-client/brokkr_broker_client/api/templates/delete_template.py

- pub `sync_detailed` function L63-88 — `def sync_detailed( id: UUID, *, client: AuthenticatedClient, ) -> Response[Any |...` — Args:
- pub `sync` function L91-111 — `def sync( id: UUID, *, client: AuthenticatedClient, ) -> Any | ErrorResponse | N...` — Args:
- pub `asyncio_detailed` function L114-137 — `def asyncio_detailed( id: UUID, *, client: AuthenticatedClient, ) -> Response[An...` — Args:
- pub `asyncio` function L140-162 — `def asyncio( id: UUID, *, client: AuthenticatedClient, ) -> Any | ErrorResponse ...` — Args:
-  `_get_kwargs` function L14-25 — `def _get_kwargs( id: UUID, ) -> dict[str, Any]`
-  `_parse_response` function L28-51 — `def _parse_response(*, client: AuthenticatedClient | Client, response: httpx.Res...`
-  `_build_response` function L54-60 — `def _build_response(*, client: AuthenticatedClient | Client, response: httpx.Res...`

#### sdks/python/brokkr-client/brokkr_broker_client/api/templates/get_template.py

- pub `sync_detailed` function L69-94 — `def sync_detailed( id: UUID, *, client: AuthenticatedClient, ) -> Response[Error...` — Args:
- pub `sync` function L97-117 — `def sync( id: UUID, *, client: AuthenticatedClient, ) -> ErrorResponse | StackTe...` — Args:
- pub `asyncio_detailed` function L120-143 — `def asyncio_detailed( id: UUID, *, client: AuthenticatedClient, ) -> Response[Er...` — Args:
- pub `asyncio` function L146-168 — `def asyncio( id: UUID, *, client: AuthenticatedClient, ) -> ErrorResponse | Stac...` — Args:
-  `_get_kwargs` function L15-26 — `def _get_kwargs( id: UUID, ) -> dict[str, Any]`
-  `_parse_response` function L29-55 — `def _parse_response( *, client: AuthenticatedClient | Client, response: httpx.Re...`
-  `_build_response` function L58-66 — `def _build_response( *, client: AuthenticatedClient | Client, response: httpx.Re...`

#### sdks/python/brokkr-client/brokkr_broker_client/api/templates/list_templates.py

- pub `sync_detailed` function L63-82 — `def sync_detailed( *, client: AuthenticatedClient, ) -> Response[ErrorResponse |...` — Raises:
- pub `sync` function L85-100 — `def sync( *, client: AuthenticatedClient, ) -> ErrorResponse | list[StackTemplat...` — Raises:
- pub `asyncio_detailed` function L103-120 — `def asyncio_detailed( *, client: AuthenticatedClient, ) -> Response[ErrorRespons...` — Raises:
- pub `asyncio` function L123-140 — `def asyncio( *, client: AuthenticatedClient, ) -> ErrorResponse | list[StackTemp...` — Raises:
-  `_get_kwargs` function L13-20 — `def _get_kwargs() -> dict[str, Any]`
-  `_parse_response` function L23-49 — `def _parse_response( *, client: AuthenticatedClient | Client, response: httpx.Re...`
-  `_build_response` function L52-60 — `def _build_response( *, client: AuthenticatedClient | Client, response: httpx.Re...`

#### sdks/python/brokkr-client/brokkr_broker_client/api/templates/templates_add_annotation.py

- pub `sync_detailed` function L83-111 — `def sync_detailed( id: UUID, *, client: AuthenticatedClient, body: AddAnnotation...` — Args:
- pub `sync` function L114-137 — `def sync( id: UUID, *, client: AuthenticatedClient, body: AddAnnotationRequest, ...` — Args:
- pub `asyncio_detailed` function L140-166 — `def asyncio_detailed( id: UUID, *, client: AuthenticatedClient, body: AddAnnotat...` — Args:
- pub `asyncio` function L169-194 — `def asyncio( id: UUID, *, client: AuthenticatedClient, body: AddAnnotationReques...` — Args:
-  `_get_kwargs` function L16-35 — `def _get_kwargs( id: UUID, *, body: AddAnnotationRequest, ) -> dict[str, Any]`
-  `_parse_response` function L38-69 — `def _parse_response( *, client: AuthenticatedClient | Client, response: httpx.Re...`
-  `_build_response` function L72-80 — `def _build_response( *, client: AuthenticatedClient | Client, response: httpx.Re...`

#### sdks/python/brokkr-client/brokkr_broker_client/api/templates/templates_add_label.py

- pub `sync_detailed` function L82-110 — `def sync_detailed( id: UUID, *, client: AuthenticatedClient, body: str, ) -> Res...` — Args:
- pub `sync` function L113-136 — `def sync( id: UUID, *, client: AuthenticatedClient, body: str, ) -> ErrorRespons...` — Args:
- pub `asyncio_detailed` function L139-165 — `def asyncio_detailed( id: UUID, *, client: AuthenticatedClient, body: str, ) -> ...` — Args:
- pub `asyncio` function L168-193 — `def asyncio( id: UUID, *, client: AuthenticatedClient, body: str, ) -> ErrorResp...` — Args:
-  `_get_kwargs` function L15-34 — `def _get_kwargs( id: UUID, *, body: str, ) -> dict[str, Any]`
-  `_parse_response` function L37-68 — `def _parse_response( *, client: AuthenticatedClient | Client, response: httpx.Re...`
-  `_build_response` function L71-79 — `def _build_response( *, client: AuthenticatedClient | Client, response: httpx.Re...`

#### sdks/python/brokkr-client/brokkr_broker_client/api/templates/templates_list_annotations.py

- pub `sync_detailed` function L74-99 — `def sync_detailed( id: UUID, *, client: AuthenticatedClient, ) -> Response[Error...` — Args:
- pub `sync` function L102-122 — `def sync( id: UUID, *, client: AuthenticatedClient, ) -> ErrorResponse | list[Te...` — Args:
- pub `asyncio_detailed` function L125-148 — `def asyncio_detailed( id: UUID, *, client: AuthenticatedClient, ) -> Response[Er...` — Args:
- pub `asyncio` function L151-173 — `def asyncio( id: UUID, *, client: AuthenticatedClient, ) -> ErrorResponse | list...` — Args:
-  `_get_kwargs` function L15-26 — `def _get_kwargs( id: UUID, ) -> dict[str, Any]`
-  `_parse_response` function L29-60 — `def _parse_response( *, client: AuthenticatedClient | Client, response: httpx.Re...`
-  `_build_response` function L63-71 — `def _build_response( *, client: AuthenticatedClient | Client, response: httpx.Re...`

#### sdks/python/brokkr-client/brokkr_broker_client/api/templates/templates_list_labels.py

- pub `sync_detailed` function L74-99 — `def sync_detailed( id: UUID, *, client: AuthenticatedClient, ) -> Response[Error...` — Args:
- pub `sync` function L102-122 — `def sync( id: UUID, *, client: AuthenticatedClient, ) -> ErrorResponse | list[Te...` — Args:
- pub `asyncio_detailed` function L125-148 — `def asyncio_detailed( id: UUID, *, client: AuthenticatedClient, ) -> Response[Er...` — Args:
- pub `asyncio` function L151-173 — `def asyncio( id: UUID, *, client: AuthenticatedClient, ) -> ErrorResponse | list...` — Args:
-  `_get_kwargs` function L15-26 — `def _get_kwargs( id: UUID, ) -> dict[str, Any]`
-  `_parse_response` function L29-60 — `def _parse_response( *, client: AuthenticatedClient | Client, response: httpx.Re...`
-  `_build_response` function L63-71 — `def _build_response( *, client: AuthenticatedClient | Client, response: httpx.Re...`

#### sdks/python/brokkr-client/brokkr_broker_client/api/templates/templates_remove_annotation.py

- pub `sync_detailed` function L65-93 — `def sync_detailed( id: UUID, key: str, *, client: AuthenticatedClient, ) -> Resp...` — Args:
- pub `sync` function L96-119 — `def sync( id: UUID, key: str, *, client: AuthenticatedClient, ) -> Any | ErrorRe...` — Args:
- pub `asyncio_detailed` function L122-148 — `def asyncio_detailed( id: UUID, key: str, *, client: AuthenticatedClient, ) -> R...` — Args:
- pub `asyncio` function L151-176 — `def asyncio( id: UUID, key: str, *, client: AuthenticatedClient, ) -> Any | Erro...` — Args:
-  `_get_kwargs` function L14-27 — `def _get_kwargs( id: UUID, key: str, ) -> dict[str, Any]`
-  `_parse_response` function L30-53 — `def _parse_response(*, client: AuthenticatedClient | Client, response: httpx.Res...`
-  `_build_response` function L56-62 — `def _build_response(*, client: AuthenticatedClient | Client, response: httpx.Res...`

#### sdks/python/brokkr-client/brokkr_broker_client/api/templates/templates_remove_label.py

- pub `sync_detailed` function L65-93 — `def sync_detailed( id: UUID, label: str, *, client: AuthenticatedClient, ) -> Re...` — Args:
- pub `sync` function L96-119 — `def sync( id: UUID, label: str, *, client: AuthenticatedClient, ) -> Any | Error...` — Args:
- pub `asyncio_detailed` function L122-148 — `def asyncio_detailed( id: UUID, label: str, *, client: AuthenticatedClient, ) ->...` — Args:
- pub `asyncio` function L151-176 — `def asyncio( id: UUID, label: str, *, client: AuthenticatedClient, ) -> Any | Er...` — Args:
-  `_get_kwargs` function L14-27 — `def _get_kwargs( id: UUID, label: str, ) -> dict[str, Any]`
-  `_parse_response` function L30-53 — `def _parse_response(*, client: AuthenticatedClient | Client, response: httpx.Res...`
-  `_build_response` function L56-62 — `def _build_response(*, client: AuthenticatedClient | Client, response: httpx.Res...`

#### sdks/python/brokkr-client/brokkr_broker_client/api/templates/update_template.py

- pub `sync_detailed` function L83-111 — `def sync_detailed( id: UUID, *, client: AuthenticatedClient, body: UpdateTemplat...` — Args:
- pub `sync` function L114-137 — `def sync( id: UUID, *, client: AuthenticatedClient, body: UpdateTemplateRequest,...` — Args:
- pub `asyncio_detailed` function L140-166 — `def asyncio_detailed( id: UUID, *, client: AuthenticatedClient, body: UpdateTemp...` — Args:
- pub `asyncio` function L169-194 — `def asyncio( id: UUID, *, client: AuthenticatedClient, body: UpdateTemplateReque...` — Args:
-  `_get_kwargs` function L16-35 — `def _get_kwargs( id: UUID, *, body: UpdateTemplateRequest, ) -> dict[str, Any]`
-  `_parse_response` function L38-69 — `def _parse_response( *, client: AuthenticatedClient | Client, response: httpx.Re...`
-  `_build_response` function L72-80 — `def _build_response( *, client: AuthenticatedClient | Client, response: httpx.Re...`

### sdks/python/brokkr-client/brokkr_broker_client/api/webhooks

> *Semantic summary to be generated by AI agent.*

#### sdks/python/brokkr-client/brokkr_broker_client/api/webhooks/create_webhook.py

- pub `sync_detailed` function L73-98 — `def sync_detailed( *, client: AuthenticatedClient, body: CreateWebhookRequest, )...` — Args:
- pub `sync` function L101-121 — `def sync( *, client: AuthenticatedClient, body: CreateWebhookRequest, ) -> Error...` — Args:
- pub `asyncio_detailed` function L124-147 — `def asyncio_detailed( *, client: AuthenticatedClient, body: CreateWebhookRequest...` — Args:
- pub `asyncio` function L150-172 — `def asyncio( *, client: AuthenticatedClient, body: CreateWebhookRequest, ) -> Er...` — Args:
-  `_get_kwargs` function L14-30 — `def _get_kwargs( *, body: CreateWebhookRequest, ) -> dict[str, Any]`
-  `_parse_response` function L33-59 — `def _parse_response( *, client: AuthenticatedClient | Client, response: httpx.Re...`
-  `_build_response` function L62-70 — `def _build_response( *, client: AuthenticatedClient | Client, response: httpx.Re...`

#### sdks/python/brokkr-client/brokkr_broker_client/api/webhooks/delete_webhook.py

- pub `sync_detailed` function L63-88 — `def sync_detailed( id: UUID, *, client: AuthenticatedClient, ) -> Response[Any |...` — Args:
- pub `sync` function L91-111 — `def sync( id: UUID, *, client: AuthenticatedClient, ) -> Any | ErrorResponse | N...` — Args:
- pub `asyncio_detailed` function L114-137 — `def asyncio_detailed( id: UUID, *, client: AuthenticatedClient, ) -> Response[An...` — Args:
- pub `asyncio` function L140-162 — `def asyncio( id: UUID, *, client: AuthenticatedClient, ) -> Any | ErrorResponse ...` — Args:
-  `_get_kwargs` function L14-25 — `def _get_kwargs( id: UUID, ) -> dict[str, Any]`
-  `_parse_response` function L28-51 — `def _parse_response(*, client: AuthenticatedClient | Client, response: httpx.Res...`
-  `_build_response` function L54-60 — `def _build_response(*, client: AuthenticatedClient | Client, response: httpx.Res...`

#### sdks/python/brokkr-client/brokkr_broker_client/api/webhooks/get_pending_agent_webhooks.py

- pub `sync_detailed` function L74-99 — `def sync_detailed( agent_id: UUID, *, client: AuthenticatedClient, ) -> Response...` — Args:
- pub `sync` function L102-122 — `def sync( agent_id: UUID, *, client: AuthenticatedClient, ) -> ErrorResponse | l...` — Args:
- pub `asyncio_detailed` function L125-148 — `def asyncio_detailed( agent_id: UUID, *, client: AuthenticatedClient, ) -> Respo...` — Args:
- pub `asyncio` function L151-173 — `def asyncio( agent_id: UUID, *, client: AuthenticatedClient, ) -> ErrorResponse ...` — Args:
-  `_get_kwargs` function L15-26 — `def _get_kwargs( agent_id: UUID, ) -> dict[str, Any]`
-  `_parse_response` function L29-60 — `def _parse_response( *, client: AuthenticatedClient | Client, response: httpx.Re...`
-  `_build_response` function L63-71 — `def _build_response( *, client: AuthenticatedClient | Client, response: httpx.Re...`

#### sdks/python/brokkr-client/brokkr_broker_client/api/webhooks/get_webhook.py

- pub `sync_detailed` function L69-94 — `def sync_detailed( id: UUID, *, client: AuthenticatedClient, ) -> Response[Error...` — Args:
- pub `sync` function L97-117 — `def sync( id: UUID, *, client: AuthenticatedClient, ) -> ErrorResponse | Webhook...` — Args:
- pub `asyncio_detailed` function L120-143 — `def asyncio_detailed( id: UUID, *, client: AuthenticatedClient, ) -> Response[Er...` — Args:
- pub `asyncio` function L146-168 — `def asyncio( id: UUID, *, client: AuthenticatedClient, ) -> ErrorResponse | Webh...` — Args:
-  `_get_kwargs` function L15-26 — `def _get_kwargs( id: UUID, ) -> dict[str, Any]`
-  `_parse_response` function L29-55 — `def _parse_response( *, client: AuthenticatedClient | Client, response: httpx.Re...`
-  `_build_response` function L58-66 — `def _build_response( *, client: AuthenticatedClient | Client, response: httpx.Re...`

#### sdks/python/brokkr-client/brokkr_broker_client/api/webhooks/list_deliveries.py

- pub `sync_detailed` function L89-123 — `def sync_detailed( id: UUID, *, client: AuthenticatedClient, status: str | Unset...` — Args:
- pub `sync` function L126-155 — `def sync( id: UUID, *, client: AuthenticatedClient, status: str | Unset = UNSET,...` — Args:
- pub `asyncio_detailed` function L158-190 — `def asyncio_detailed( id: UUID, *, client: AuthenticatedClient, status: str | Un...` — Args:
- pub `asyncio` function L193-224 — `def asyncio( id: UUID, *, client: AuthenticatedClient, status: str | Unset = UNS...` — Args:
-  `_get_kwargs` function L15-41 — `def _get_kwargs( id: UUID, *, status: str | Unset = UNSET, limit: int | Unset = ...`
-  `_parse_response` function L44-75 — `def _parse_response( *, client: AuthenticatedClient | Client, response: httpx.Re...`
-  `_build_response` function L78-86 — `def _build_response( *, client: AuthenticatedClient | Client, response: httpx.Re...`

#### sdks/python/brokkr-client/brokkr_broker_client/api/webhooks/list_event_types.py

- pub `sync_detailed` function L52-71 — `def sync_detailed( *, client: AuthenticatedClient, ) -> Response[ErrorResponse |...` — Raises:
- pub `sync` function L74-89 — `def sync( *, client: AuthenticatedClient, ) -> ErrorResponse | list[str] | None` — Raises:
- pub `asyncio_detailed` function L92-109 — `def asyncio_detailed( *, client: AuthenticatedClient, ) -> Response[ErrorRespons...` — Raises:
- pub `asyncio` function L112-129 — `def asyncio( *, client: AuthenticatedClient, ) -> ErrorResponse | list[str] | No...` — Raises:
-  `_get_kwargs` function L12-19 — `def _get_kwargs() -> dict[str, Any]`
-  `_parse_response` function L22-38 — `def _parse_response( *, client: AuthenticatedClient | Client, response: httpx.Re...`
-  `_build_response` function L41-49 — `def _build_response( *, client: AuthenticatedClient | Client, response: httpx.Re...`

#### sdks/python/brokkr-client/brokkr_broker_client/api/webhooks/list_webhooks.py

- pub `sync_detailed` function L63-82 — `def sync_detailed( *, client: AuthenticatedClient, ) -> Response[ErrorResponse |...` — Raises:
- pub `sync` function L85-100 — `def sync( *, client: AuthenticatedClient, ) -> ErrorResponse | list[WebhookRespo...` — Raises:
- pub `asyncio_detailed` function L103-120 — `def asyncio_detailed( *, client: AuthenticatedClient, ) -> Response[ErrorRespons...` — Raises:
- pub `asyncio` function L123-140 — `def asyncio( *, client: AuthenticatedClient, ) -> ErrorResponse | list[WebhookRe...` — Raises:
-  `_get_kwargs` function L13-20 — `def _get_kwargs() -> dict[str, Any]`
-  `_parse_response` function L23-49 — `def _parse_response( *, client: AuthenticatedClient | Client, response: httpx.Re...`
-  `_build_response` function L52-60 — `def _build_response( *, client: AuthenticatedClient | Client, response: httpx.Re...`

#### sdks/python/brokkr-client/brokkr_broker_client/api/webhooks/report_delivery_result.py

- pub `sync_detailed` function L72-100 — `def sync_detailed( id: UUID, *, client: AuthenticatedClient, body: DeliveryResul...` — Args:
- pub `sync` function L103-126 — `def sync( id: UUID, *, client: AuthenticatedClient, body: DeliveryResultRequest,...` — Args:
- pub `asyncio_detailed` function L129-155 — `def asyncio_detailed( id: UUID, *, client: AuthenticatedClient, body: DeliveryRe...` — Args:
- pub `asyncio` function L158-183 — `def asyncio( id: UUID, *, client: AuthenticatedClient, body: DeliveryResultReque...` — Args:
-  `_get_kwargs` function L15-34 — `def _get_kwargs( id: UUID, *, body: DeliveryResultRequest, ) -> dict[str, Any]`
-  `_parse_response` function L37-60 — `def _parse_response(*, client: AuthenticatedClient | Client, response: httpx.Res...`
-  `_build_response` function L63-69 — `def _build_response(*, client: AuthenticatedClient | Client, response: httpx.Res...`

#### sdks/python/brokkr-client/brokkr_broker_client/api/webhooks/test_webhook.py

- pub `sync_detailed` function L68-93 — `def sync_detailed( id: UUID, *, client: AuthenticatedClient, ) -> Response[Any |...` — Args:
- pub `sync` function L96-116 — `def sync( id: UUID, *, client: AuthenticatedClient, ) -> Any | ErrorResponse | N...` — Args:
- pub `asyncio_detailed` function L119-142 — `def asyncio_detailed( id: UUID, *, client: AuthenticatedClient, ) -> Response[An...` — Args:
- pub `asyncio` function L145-167 — `def asyncio( id: UUID, *, client: AuthenticatedClient, ) -> Any | ErrorResponse ...` — Args:
-  `_get_kwargs` function L14-25 — `def _get_kwargs( id: UUID, ) -> dict[str, Any]`
-  `_parse_response` function L28-56 — `def _parse_response(*, client: AuthenticatedClient | Client, response: httpx.Res...`
-  `_build_response` function L59-65 — `def _build_response(*, client: AuthenticatedClient | Client, response: httpx.Res...`

#### sdks/python/brokkr-client/brokkr_broker_client/api/webhooks/update_webhook.py

- pub `sync_detailed` function L83-111 — `def sync_detailed( id: UUID, *, client: AuthenticatedClient, body: UpdateWebhook...` — Args:
- pub `sync` function L114-137 — `def sync( id: UUID, *, client: AuthenticatedClient, body: UpdateWebhookRequest, ...` — Args:
- pub `asyncio_detailed` function L140-166 — `def asyncio_detailed( id: UUID, *, client: AuthenticatedClient, body: UpdateWebh...` — Args:
- pub `asyncio` function L169-194 — `def asyncio( id: UUID, *, client: AuthenticatedClient, body: UpdateWebhookReques...` — Args:
-  `_get_kwargs` function L16-35 — `def _get_kwargs( id: UUID, *, body: UpdateWebhookRequest, ) -> dict[str, Any]`
-  `_parse_response` function L38-69 — `def _parse_response( *, client: AuthenticatedClient | Client, response: httpx.Re...`
-  `_build_response` function L72-80 — `def _build_response( *, client: AuthenticatedClient | Client, response: httpx.Re...`

### sdks/python/brokkr-client/brokkr_broker_client/api/work_orders

> *Semantic summary to be generated by AI agent.*

#### sdks/python/brokkr-client/brokkr_broker_client/api/work_orders/claim_work_order.py

- pub `sync_detailed` function L78-106 — `def sync_detailed( id: UUID, *, client: AuthenticatedClient, body: ClaimWorkOrde...` — Args:
- pub `sync` function L109-132 — `def sync( id: UUID, *, client: AuthenticatedClient, body: ClaimWorkOrderRequest,...` — Args:
- pub `asyncio_detailed` function L135-161 — `def asyncio_detailed( id: UUID, *, client: AuthenticatedClient, body: ClaimWorkO...` — Args:
- pub `asyncio` function L164-189 — `def asyncio( id: UUID, *, client: AuthenticatedClient, body: ClaimWorkOrderReque...` — Args:
-  `_get_kwargs` function L16-35 — `def _get_kwargs( id: UUID, *, body: ClaimWorkOrderRequest, ) -> dict[str, Any]`
-  `_parse_response` function L38-64 — `def _parse_response( *, client: AuthenticatedClient | Client, response: httpx.Re...`
-  `_build_response` function L67-75 — `def _build_response( *, client: AuthenticatedClient | Client, response: httpx.Re...`

#### sdks/python/brokkr-client/brokkr_broker_client/api/work_orders/complete_work_order.py

- pub `sync_detailed` function L78-106 — `def sync_detailed( id: UUID, *, client: AuthenticatedClient, body: CompleteWorkO...` — Args:
- pub `sync` function L109-132 — `def sync( id: UUID, *, client: AuthenticatedClient, body: CompleteWorkOrderReque...` — Args:
- pub `asyncio_detailed` function L135-161 — `def asyncio_detailed( id: UUID, *, client: AuthenticatedClient, body: CompleteWo...` — Args:
- pub `asyncio` function L164-189 — `def asyncio( id: UUID, *, client: AuthenticatedClient, body: CompleteWorkOrderRe...` — Args:
-  `_get_kwargs` function L16-35 — `def _get_kwargs( id: UUID, *, body: CompleteWorkOrderRequest, ) -> dict[str, Any...`
-  `_parse_response` function L38-64 — `def _parse_response( *, client: AuthenticatedClient | Client, response: httpx.Re...`
-  `_build_response` function L67-75 — `def _build_response( *, client: AuthenticatedClient | Client, response: httpx.Re...`

#### sdks/python/brokkr-client/brokkr_broker_client/api/work_orders/create_work_order.py

- pub `sync_detailed` function L73-98 — `def sync_detailed( *, client: AuthenticatedClient, body: CreateWorkOrderRequest,...` — Args:
- pub `sync` function L101-121 — `def sync( *, client: AuthenticatedClient, body: CreateWorkOrderRequest, ) -> Err...` — Args:
- pub `asyncio_detailed` function L124-147 — `def asyncio_detailed( *, client: AuthenticatedClient, body: CreateWorkOrderReque...` — Args:
- pub `asyncio` function L150-172 — `def asyncio( *, client: AuthenticatedClient, body: CreateWorkOrderRequest, ) -> ...` — Args:
-  `_get_kwargs` function L14-30 — `def _get_kwargs( *, body: CreateWorkOrderRequest, ) -> dict[str, Any]`
-  `_parse_response` function L33-59 — `def _parse_response( *, client: AuthenticatedClient | Client, response: httpx.Re...`
-  `_build_response` function L62-70 — `def _build_response( *, client: AuthenticatedClient | Client, response: httpx.Re...`

#### sdks/python/brokkr-client/brokkr_broker_client/api/work_orders/delete_work_order.py

- pub `sync_detailed` function L63-88 — `def sync_detailed( id: UUID, *, client: AuthenticatedClient, ) -> Response[Any |...` — Args:
- pub `sync` function L91-111 — `def sync( id: UUID, *, client: AuthenticatedClient, ) -> Any | ErrorResponse | N...` — Args:
- pub `asyncio_detailed` function L114-137 — `def asyncio_detailed( id: UUID, *, client: AuthenticatedClient, ) -> Response[An...` — Args:
- pub `asyncio` function L140-162 — `def asyncio( id: UUID, *, client: AuthenticatedClient, ) -> Any | ErrorResponse ...` — Args:
-  `_get_kwargs` function L14-25 — `def _get_kwargs( id: UUID, ) -> dict[str, Any]`
-  `_parse_response` function L28-51 — `def _parse_response(*, client: AuthenticatedClient | Client, response: httpx.Res...`
-  `_build_response` function L54-60 — `def _build_response(*, client: AuthenticatedClient | Client, response: httpx.Res...`

#### sdks/python/brokkr-client/brokkr_broker_client/api/work_orders/get_work_order.py

- pub `sync_detailed` function L69-94 — `def sync_detailed( id: UUID, *, client: AuthenticatedClient, ) -> Response[Error...` — Args:
- pub `sync` function L97-117 — `def sync( id: UUID, *, client: AuthenticatedClient, ) -> ErrorResponse | WorkOrd...` — Args:
- pub `asyncio_detailed` function L120-143 — `def asyncio_detailed( id: UUID, *, client: AuthenticatedClient, ) -> Response[Er...` — Args:
- pub `asyncio` function L146-168 — `def asyncio( id: UUID, *, client: AuthenticatedClient, ) -> ErrorResponse | Work...` — Args:
-  `_get_kwargs` function L15-26 — `def _get_kwargs( id: UUID, ) -> dict[str, Any]`
-  `_parse_response` function L29-55 — `def _parse_response( *, client: AuthenticatedClient | Client, response: httpx.Re...`
-  `_build_response` function L58-66 — `def _build_response( *, client: AuthenticatedClient | Client, response: httpx.Re...`

#### sdks/python/brokkr-client/brokkr_broker_client/api/work_orders/get_work_order_log.py

- pub `sync_detailed` function L69-94 — `def sync_detailed( id: UUID, *, client: AuthenticatedClient, ) -> Response[Error...` — Args:
- pub `sync` function L97-117 — `def sync( id: UUID, *, client: AuthenticatedClient, ) -> ErrorResponse | WorkOrd...` — Args:
- pub `asyncio_detailed` function L120-143 — `def asyncio_detailed( id: UUID, *, client: AuthenticatedClient, ) -> Response[Er...` — Args:
- pub `asyncio` function L146-168 — `def asyncio( id: UUID, *, client: AuthenticatedClient, ) -> ErrorResponse | Work...` — Args:
-  `_get_kwargs` function L15-26 — `def _get_kwargs( id: UUID, ) -> dict[str, Any]`
-  `_parse_response` function L29-55 — `def _parse_response( *, client: AuthenticatedClient | Client, response: httpx.Re...`
-  `_build_response` function L58-66 — `def _build_response( *, client: AuthenticatedClient | Client, response: httpx.Re...`

#### sdks/python/brokkr-client/brokkr_broker_client/api/work_orders/list_pending_for_agent.py

- pub `sync_detailed` function L78-106 — `def sync_detailed( agent_id: UUID, *, client: AuthenticatedClient, work_type: st...` — Args:
- pub `sync` function L109-132 — `def sync( agent_id: UUID, *, client: AuthenticatedClient, work_type: str | Unset...` — Args:
- pub `asyncio_detailed` function L135-161 — `def asyncio_detailed( agent_id: UUID, *, client: AuthenticatedClient, work_type:...` — Args:
- pub `asyncio` function L164-189 — `def asyncio( agent_id: UUID, *, client: AuthenticatedClient, work_type: str | Un...` — Args:
-  `_get_kwargs` function L15-35 — `def _get_kwargs( agent_id: UUID, *, work_type: str | Unset = UNSET, ) -> dict[st...`
-  `_parse_response` function L38-64 — `def _parse_response( *, client: AuthenticatedClient | Client, response: httpx.Re...`
-  `_build_response` function L67-75 — `def _build_response( *, client: AuthenticatedClient | Client, response: httpx.Re...`

#### sdks/python/brokkr-client/brokkr_broker_client/api/work_orders/list_work_order_log.py

- pub `sync_detailed` function L86-120 — `def sync_detailed( *, client: AuthenticatedClient, work_type: str | Unset = UNSE...` — Args:
- pub `sync` function L123-152 — `def sync( *, client: AuthenticatedClient, work_type: str | Unset = UNSET, succes...` — Args:
- pub `asyncio_detailed` function L155-187 — `def asyncio_detailed( *, client: AuthenticatedClient, work_type: str | Unset = U...` — Args:
- pub `asyncio` function L190-221 — `def asyncio( *, client: AuthenticatedClient, work_type: str | Unset = UNSET, suc...` — Args:
-  `_get_kwargs` function L14-43 — `def _get_kwargs( *, work_type: str | Unset = UNSET, success: bool | Unset = UNSE...`
-  `_parse_response` function L46-72 — `def _parse_response( *, client: AuthenticatedClient | Client, response: httpx.Re...`
-  `_build_response` function L75-83 — `def _build_response( *, client: AuthenticatedClient | Client, response: httpx.Re...`

#### sdks/python/brokkr-client/brokkr_broker_client/api/work_orders/list_work_orders.py

- pub `sync_detailed` function L76-104 — `def sync_detailed( *, client: AuthenticatedClient, status: str | Unset = UNSET, ...` — Args:
- pub `sync` function L107-130 — `def sync( *, client: AuthenticatedClient, status: str | Unset = UNSET, work_type...` — Args:
- pub `asyncio_detailed` function L133-159 — `def asyncio_detailed( *, client: AuthenticatedClient, status: str | Unset = UNSE...` — Args:
- pub `asyncio` function L162-187 — `def asyncio( *, client: AuthenticatedClient, status: str | Unset = UNSET, work_t...` — Args:
-  `_get_kwargs` function L13-33 — `def _get_kwargs( *, status: str | Unset = UNSET, work_type: str | Unset = UNSET,...`
-  `_parse_response` function L36-62 — `def _parse_response( *, client: AuthenticatedClient | Client, response: httpx.Re...`
-  `_build_response` function L65-73 — `def _build_response( *, client: AuthenticatedClient | Client, response: httpx.Re...`

### sdks/python/brokkr-client/brokkr_broker_client/models

> *Semantic summary to be generated by AI agent.*

#### sdks/python/brokkr-client/brokkr_broker_client/models/add_annotation_request.py

- pub `AddAnnotationRequest` class L13-69 — `{ to_dict, from_dict, additional_keys }` — Attributes:
- pub `to_dict` method L24-38 — `def to_dict(self) -> dict[str, Any]`
- pub `__getitem__` method L59-60 — `def __getitem__(self, key: str) -> Any`
- pub `__setitem__` method L62-63 — `def __setitem__(self, key: str, value: Any) -> None`
- pub `__delitem__` method L65-66 — `def __delitem__(self, key: str) -> None`
- pub `__contains__` method L68-69 — `def __contains__(self, key: str) -> bool`

#### sdks/python/brokkr-client/brokkr_broker_client/models/agent.py

- pub `Agent` class L18-167 — `{ to_dict, from_dict, additional_keys }` — Represents an agent in the database.
- pub `to_dict` method L42-88 — `def to_dict(self) -> dict[str, Any]`
- pub `__getitem__` method L157-158 — `def __getitem__(self, key: str) -> Any`
- pub `__setitem__` method L160-161 — `def __setitem__(self, key: str, value: Any) -> None`
- pub `__delitem__` method L163-164 — `def __delitem__(self, key: str) -> None`
- pub `__contains__` method L166-167 — `def __contains__(self, key: str) -> bool`

#### sdks/python/brokkr-client/brokkr_broker_client/models/agent_annotation.py

- pub `AgentAnnotation` class L14-87 — `{ to_dict, from_dict, additional_keys }` — Represents an agent annotation in the database.
- pub `to_dict` method L30-50 — `def to_dict(self) -> dict[str, Any]`
- pub `__getitem__` method L77-78 — `def __getitem__(self, key: str) -> Any`
- pub `__setitem__` method L80-81 — `def __setitem__(self, key: str, value: Any) -> None`
- pub `__delitem__` method L83-84 — `def __delitem__(self, key: str) -> None`
- pub `__contains__` method L86-87 — `def __contains__(self, key: str) -> bool`

#### sdks/python/brokkr-client/brokkr_broker_client/models/agent_event.py

- pub `AgentEvent` class L18-173 — `{ to_dict, from_dict, additional_keys }` — Represents an agent event in the database.
- pub `to_dict` method L52-99 — `def to_dict(self) -> dict[str, Any]`
- pub `__getitem__` method L163-164 — `def __getitem__(self, key: str) -> Any`
- pub `__setitem__` method L166-167 — `def __setitem__(self, key: str, value: Any) -> None`
- pub `__delitem__` method L169-170 — `def __delitem__(self, key: str) -> None`
- pub `__contains__` method L172-173 — `def __contains__(self, key: str) -> bool`

#### sdks/python/brokkr-client/brokkr_broker_client/models/agent_k8s_event.py

- pub `AgentK8SEvent` class L18-150 — `{ to_dict, from_dict, additional_keys }` — Attributes:
- pub `to_dict` method L45-88 — `def to_dict(self) -> dict[str, Any]`
- pub `__getitem__` method L140-141 — `def __getitem__(self, key: str) -> Any`
- pub `__setitem__` method L143-144 — `def __setitem__(self, key: str, value: Any) -> None`
- pub `__delitem__` method L146-147 — `def __delitem__(self, key: str) -> None`
- pub `__contains__` method L149-150 — `def __contains__(self, key: str) -> bool`

#### sdks/python/brokkr-client/brokkr_broker_client/models/agent_label.py

- pub `AgentLabel` class L14-79 — `{ to_dict, from_dict, additional_keys }` — Represents an agent label in the database.
- pub `to_dict` method L28-45 — `def to_dict(self) -> dict[str, Any]`
- pub `__getitem__` method L69-70 — `def __getitem__(self, key: str) -> Any`
- pub `__setitem__` method L72-73 — `def __setitem__(self, key: str, value: Any) -> None`
- pub `__delitem__` method L75-76 — `def __delitem__(self, key: str) -> None`
- pub `__contains__` method L78-79 — `def __contains__(self, key: str) -> bool`

#### sdks/python/brokkr-client/brokkr_broker_client/models/agent_pod_log.py

- pub `AgentPodLog` class L16-128 — `{ to_dict, from_dict, additional_keys }` — Attributes:
- pub `to_dict` method L41-76 — `def to_dict(self) -> dict[str, Any]`
- pub `__getitem__` method L118-119 — `def __getitem__(self, key: str) -> Any`
- pub `__setitem__` method L121-122 — `def __setitem__(self, key: str, value: Any) -> None`
- pub `__delitem__` method L124-125 — `def __delitem__(self, key: str) -> None`
- pub `__contains__` method L127-128 — `def __contains__(self, key: str) -> bool`

#### sdks/python/brokkr-client/brokkr_broker_client/models/agent_target.py

- pub `AgentTarget` class L14-79 — `{ to_dict, from_dict, additional_keys }` — Represents an agent target in the database.
- pub `to_dict` method L28-45 — `def to_dict(self) -> dict[str, Any]`
- pub `__getitem__` method L69-70 — `def __getitem__(self, key: str) -> Any`
- pub `__setitem__` method L72-73 — `def __setitem__(self, key: str, value: Any) -> None`
- pub `__delitem__` method L75-76 — `def __delitem__(self, key: str) -> None`
- pub `__contains__` method L78-79 — `def __contains__(self, key: str) -> bool`

#### sdks/python/brokkr-client/brokkr_broker_client/models/audit_log.py

- pub `AuditLog` class L18-216 — `{ to_dict, from_dict, additional_keys }` — An audit log record from the database.
- pub `to_dict` method L48-114 — `def to_dict(self) -> dict[str, Any]`
- pub `__getitem__` method L206-207 — `def __getitem__(self, key: str) -> Any`
- pub `__setitem__` method L209-210 — `def __setitem__(self, key: str, value: Any) -> None`
- pub `__delitem__` method L212-213 — `def __delitem__(self, key: str) -> None`
- pub `__contains__` method L215-216 — `def __contains__(self, key: str) -> bool`

#### sdks/python/brokkr-client/brokkr_broker_client/models/audit_log_list_response.py

- pub `AuditLogListResponse` class L17-108 — `{ to_dict, from_dict, additional_keys }` — Response structure for audit log list operations.
- pub `to_dict` method L35-61 — `def to_dict(self) -> dict[str, Any]`
- pub `__getitem__` method L98-99 — `def __getitem__(self, key: str) -> Any`
- pub `__setitem__` method L101-102 — `def __setitem__(self, key: str, value: Any) -> None`
- pub `__delitem__` method L104-105 — `def __delitem__(self, key: str) -> None`
- pub `__contains__` method L107-108 — `def __contains__(self, key: str) -> bool`

#### sdks/python/brokkr-client/brokkr_broker_client/models/auth_response.py

- pub `AuthResponse` class L15-104 — `{ to_dict, from_dict, additional_keys }` — Represents the response structure for authentication information.
- pub `to_dict` method L29-56 — `def to_dict(self) -> dict[str, Any]`
- pub `__getitem__` method L94-95 — `def __getitem__(self, key: str) -> Any`
- pub `__setitem__` method L97-98 — `def __setitem__(self, key: str, value: Any) -> None`
- pub `__delitem__` method L100-101 — `def __delitem__(self, key: str) -> None`
- pub `__contains__` method L103-104 — `def __contains__(self, key: str) -> bool`

#### sdks/python/brokkr-client/brokkr_broker_client/models/claim_work_order_request.py

- pub `ClaimWorkOrderRequest` class L14-62 — `{ to_dict, from_dict, additional_keys }` — Attributes:
- pub `to_dict` method L23-34 — `def to_dict(self) -> dict[str, Any]`
- pub `__getitem__` method L52-53 — `def __getitem__(self, key: str) -> Any`
- pub `__setitem__` method L55-56 — `def __setitem__(self, key: str, value: Any) -> None`
- pub `__delitem__` method L58-59 — `def __delitem__(self, key: str) -> None`
- pub `__contains__` method L61-62 — `def __contains__(self, key: str) -> bool`

#### sdks/python/brokkr-client/brokkr_broker_client/models/complete_work_order_request.py

- pub `CompleteWorkOrderRequest` class L15-92 — `{ to_dict, from_dict, additional_keys }` — Attributes:
- pub `to_dict` method L28-51 — `def to_dict(self) -> dict[str, Any]`
- pub `__getitem__` method L82-83 — `def __getitem__(self, key: str) -> Any`
- pub `__setitem__` method L85-86 — `def __setitem__(self, key: str, value: Any) -> None`
- pub `__delitem__` method L88-89 — `def __delitem__(self, key: str) -> None`
- pub `__contains__` method L91-92 — `def __contains__(self, key: str) -> bool`

#### sdks/python/brokkr-client/brokkr_broker_client/models/config_change_info.py

- pub `ConfigChangeInfo` class L13-78 — `{ to_dict, from_dict, additional_keys }` — Information about a single configuration change.
- pub `to_dict` method L27-44 — `def to_dict(self) -> dict[str, Any]`
- pub `__getitem__` method L68-69 — `def __getitem__(self, key: str) -> Any`
- pub `__setitem__` method L71-72 — `def __setitem__(self, key: str, value: Any) -> None`
- pub `__delitem__` method L74-75 — `def __delitem__(self, key: str) -> None`
- pub `__contains__` method L77-78 — `def __contains__(self, key: str) -> bool`

#### sdks/python/brokkr-client/brokkr_broker_client/models/config_reload_response.py

- pub `ConfigReloadResponse` class L21-116 — `{ to_dict, from_dict, additional_keys }` — Response structure for configuration reload operations.
- pub `to_dict` method L37-65 — `def to_dict(self) -> dict[str, Any]`
- pub `__getitem__` method L106-107 — `def __getitem__(self, key: str) -> Any`
- pub `__setitem__` method L109-110 — `def __setitem__(self, key: str, value: Any) -> None`
- pub `__delitem__` method L112-113 — `def __delitem__(self, key: str) -> None`
- pub `__contains__` method L115-116 — `def __contains__(self, key: str) -> bool`

#### sdks/python/brokkr-client/brokkr_broker_client/models/create_agent_response.py

- pub `CreateAgentResponse` class L17-77 — `{ to_dict, from_dict, additional_keys }` — Response body for [`create_agent`]: the newly-created agent plus the
- pub `to_dict` method L30-44 — `def to_dict(self) -> dict[str, Any]`
- pub `__getitem__` method L67-68 — `def __getitem__(self, key: str) -> Any`
- pub `__setitem__` method L70-71 — `def __setitem__(self, key: str, value: Any) -> None`
- pub `__delitem__` method L73-74 — `def __delitem__(self, key: str) -> None`
- pub `__contains__` method L76-77 — `def __contains__(self, key: str) -> bool`

#### sdks/python/brokkr-client/brokkr_broker_client/models/create_deployment_object_request.py

- pub `CreateDeploymentObjectRequest` class L15-76 — `{ to_dict, from_dict, additional_keys }` — Wire DTO for creating a deployment object via the public API.
- pub `to_dict` method L30-45 — `def to_dict(self) -> dict[str, Any]`
- pub `__getitem__` method L66-67 — `def __getitem__(self, key: str) -> Any`
- pub `__setitem__` method L69-70 — `def __setitem__(self, key: str, value: Any) -> None`
- pub `__delitem__` method L72-73 — `def __delitem__(self, key: str) -> None`
- pub `__contains__` method L75-76 — `def __contains__(self, key: str) -> bool`

#### sdks/python/brokkr-client/brokkr_broker_client/models/create_diagnostic_request.py

- pub `CreateDiagnosticRequest` class L16-106 — `{ to_dict, from_dict, additional_keys }` — Request body for creating a diagnostic request.
- pub `to_dict` method L31-58 — `def to_dict(self) -> dict[str, Any]`
- pub `__getitem__` method L96-97 — `def __getitem__(self, key: str) -> Any`
- pub `__setitem__` method L99-100 — `def __setitem__(self, key: str, value: Any) -> None`
- pub `__delitem__` method L102-103 — `def __delitem__(self, key: str) -> None`
- pub `__contains__` method L105-106 — `def __contains__(self, key: str) -> bool`

#### sdks/python/brokkr-client/brokkr_broker_client/models/create_generator_response.py

- pub `CreateGeneratorResponse` class L17-76 — `{ to_dict, from_dict, additional_keys }` — Response for a successful generator creation or PAK rotation.
- pub `to_dict` method L29-43 — `def to_dict(self) -> dict[str, Any]`
- pub `__getitem__` method L66-67 — `def __getitem__(self, key: str) -> Any`
- pub `__setitem__` method L69-70 — `def __setitem__(self, key: str, value: Any) -> None`
- pub `__delitem__` method L72-73 — `def __delitem__(self, key: str) -> None`
- pub `__contains__` method L75-76 — `def __contains__(self, key: str) -> bool`

#### sdks/python/brokkr-client/brokkr_broker_client/models/create_template_request.py

- pub `CreateTemplateRequest` class L15-99 — `{ to_dict, from_dict, additional_keys }` — Attributes:
- pub `to_dict` method L30-55 — `def to_dict(self) -> dict[str, Any]`
- pub `__getitem__` method L89-90 — `def __getitem__(self, key: str) -> Any`
- pub `__setitem__` method L92-93 — `def __setitem__(self, key: str, value: Any) -> None`
- pub `__delitem__` method L95-96 — `def __delitem__(self, key: str) -> None`
- pub `__contains__` method L98-99 — `def __contains__(self, key: str) -> bool`

#### sdks/python/brokkr-client/brokkr_broker_client/models/create_webhook_request.py

- pub `CreateWebhookRequest` class L19-201 — `{ to_dict, from_dict, additional_keys }` — Attributes:
- pub `to_dict` method L44-106 — `def to_dict(self) -> dict[str, Any]`
- pub `__getitem__` method L191-192 — `def __getitem__(self, key: str) -> Any`
- pub `__setitem__` method L194-195 — `def __setitem__(self, key: str, value: Any) -> None`
- pub `__delitem__` method L197-198 — `def __delitem__(self, key: str) -> None`
- pub `__contains__` method L200-201 — `def __contains__(self, key: str) -> bool`

#### sdks/python/brokkr-client/brokkr_broker_client/models/create_work_order_request.py

- pub `CreateWorkOrderRequest` class L20-193 — `{ to_dict, from_dict, additional_keys }` — Attributes:
- pub `to_dict` method L41-99 — `def to_dict(self) -> dict[str, Any]`
- pub `__getitem__` method L183-184 — `def __getitem__(self, key: str) -> Any`
- pub `__setitem__` method L186-187 — `def __setitem__(self, key: str, value: Any) -> None`
- pub `__delitem__` method L189-190 — `def __delitem__(self, key: str) -> None`
- pub `__contains__` method L192-193 — `def __contains__(self, key: str) -> bool`

#### sdks/python/brokkr-client/brokkr_broker_client/models/delivery_result_request.py

- pub `DeliveryResultRequest` class L15-123 — `{ to_dict, from_dict, additional_keys }` — Attributes:
- pub `to_dict` method L30-65 — `def to_dict(self) -> dict[str, Any]`
- pub `__getitem__` method L113-114 — `def __getitem__(self, key: str) -> Any`
- pub `__setitem__` method L116-117 — `def __setitem__(self, key: str, value: Any) -> None`
- pub `__delitem__` method L119-120 — `def __delitem__(self, key: str) -> None`
- pub `__contains__` method L122-123 — `def __contains__(self, key: str) -> bool`

#### sdks/python/brokkr-client/brokkr_broker_client/models/deployment_health.py

- pub `DeploymentHealth` class L18-144 — `{ to_dict, from_dict, additional_keys }` — Represents a deployment health record in the database.
- pub `to_dict` method L51-88 — `def to_dict(self) -> dict[str, Any]`
- pub `__getitem__` method L134-135 — `def __getitem__(self, key: str) -> Any`
- pub `__setitem__` method L137-138 — `def __setitem__(self, key: str, value: Any) -> None`
- pub `__delitem__` method L140-141 — `def __delitem__(self, key: str) -> None`
- pub `__contains__` method L143-144 — `def __contains__(self, key: str) -> bool`

#### sdks/python/brokkr-client/brokkr_broker_client/models/deployment_health_response.py

- pub `DeploymentHealthResponse` class L18-93 — `{ to_dict, from_dict, additional_keys }` — Response for deployment object health query.
- pub `to_dict` method L32-52 — `def to_dict(self) -> dict[str, Any]`
- pub `__getitem__` method L83-84 — `def __getitem__(self, key: str) -> Any`
- pub `__setitem__` method L86-87 — `def __setitem__(self, key: str, value: Any) -> None`
- pub `__delitem__` method L89-90 — `def __delitem__(self, key: str) -> None`
- pub `__contains__` method L92-93 — `def __contains__(self, key: str) -> bool`

#### sdks/python/brokkr-client/brokkr_broker_client/models/deployment_object.py

- pub `DeploymentObject` class L18-161 — `{ to_dict, from_dict, additional_keys }` — Represents a deployment object in the database.
- pub `to_dict` method L46-91 — `def to_dict(self) -> dict[str, Any]`
- pub `__getitem__` method L151-152 — `def __getitem__(self, key: str) -> Any`
- pub `__setitem__` method L154-155 — `def __setitem__(self, key: str, value: Any) -> None`
- pub `__delitem__` method L157-158 — `def __delitem__(self, key: str) -> None`
- pub `__contains__` method L160-161 — `def __contains__(self, key: str) -> bool`

#### sdks/python/brokkr-client/brokkr_broker_client/models/deployment_object_health_summary.py

- pub `DeploymentObjectHealthSummary` class L14-95 — `{ to_dict, from_dict, additional_keys }` — Summary of health for a deployment object within a stack.
- pub `to_dict` method L32-55 — `def to_dict(self) -> dict[str, Any]`
- pub `__getitem__` method L85-86 — `def __getitem__(self, key: str) -> Any`
- pub `__setitem__` method L88-89 — `def __setitem__(self, key: str, value: Any) -> None`
- pub `__delitem__` method L91-92 — `def __delitem__(self, key: str) -> None`
- pub `__contains__` method L94-95 — `def __contains__(self, key: str) -> bool`

#### sdks/python/brokkr-client/brokkr_broker_client/models/deployment_object_health_update.py

- pub `DeploymentObjectHealthUpdate` class L22-105 — `{ to_dict, from_dict, additional_keys }` — Health update for a single deployment object.
- pub `to_dict` method L38-61 — `def to_dict(self) -> dict[str, Any]`
- pub `__getitem__` method L95-96 — `def __getitem__(self, key: str) -> Any`
- pub `__setitem__` method L98-99 — `def __setitem__(self, key: str, value: Any) -> None`
- pub `__delitem__` method L101-102 — `def __delitem__(self, key: str) -> None`
- pub `__contains__` method L104-105 — `def __contains__(self, key: str) -> bool`

#### sdks/python/brokkr-client/brokkr_broker_client/models/diagnostic_request.py

- pub `DiagnosticRequest` class L18-187 — `{ to_dict, from_dict, additional_keys }` — A diagnostic request record from the database.
- pub `to_dict` method L44-98 — `def to_dict(self) -> dict[str, Any]`
- pub `__getitem__` method L177-178 — `def __getitem__(self, key: str) -> Any`
- pub `__setitem__` method L180-181 — `def __setitem__(self, key: str, value: Any) -> None`
- pub `__delitem__` method L183-184 — `def __delitem__(self, key: str) -> None`
- pub `__contains__` method L186-187 — `def __contains__(self, key: str) -> bool`

#### sdks/python/brokkr-client/brokkr_broker_client/models/diagnostic_response.py

- pub `DiagnosticResponse` class L20-88 — `{ to_dict, from_dict, additional_keys }` — Response containing a diagnostic request with optional result.
- pub `to_dict` method L32-49 — `def to_dict(self) -> dict[str, Any]`
- pub `__getitem__` method L78-79 — `def __getitem__(self, key: str) -> Any`
- pub `__setitem__` method L81-82 — `def __setitem__(self, key: str, value: Any) -> None`
- pub `__delitem__` method L84-85 — `def __delitem__(self, key: str) -> None`
- pub `__contains__` method L87-88 — `def __contains__(self, key: str) -> bool`

#### sdks/python/brokkr-client/brokkr_broker_client/models/diagnostic_result.py

- pub `DiagnosticResult` class L18-127 — `{ to_dict, from_dict, additional_keys }` — A diagnostic result record from the database.
- pub `to_dict` method L40-74 — `def to_dict(self) -> dict[str, Any]`
- pub `__getitem__` method L117-118 — `def __getitem__(self, key: str) -> Any`
- pub `__setitem__` method L120-121 — `def __setitem__(self, key: str, value: Any) -> None`
- pub `__delitem__` method L123-124 — `def __delitem__(self, key: str) -> None`
- pub `__contains__` method L126-127 — `def __contains__(self, key: str) -> bool`

#### sdks/python/brokkr-client/brokkr_broker_client/models/error_response.py

- pub `ErrorResponse` class L19-112 — `{ to_dict, from_dict, additional_keys }` — Wire format for every 4xx/5xx response body in the v1 API.
- pub `to_dict` method L35-61 — `def to_dict(self) -> dict[str, Any]`
- pub `__getitem__` method L102-103 — `def __getitem__(self, key: str) -> Any`
- pub `__setitem__` method L105-106 — `def __setitem__(self, key: str, value: Any) -> None`
- pub `__delitem__` method L108-109 — `def __delitem__(self, key: str) -> None`
- pub `__contains__` method L111-112 — `def __contains__(self, key: str) -> bool`

#### sdks/python/brokkr-client/brokkr_broker_client/models/error_response_details_type_0.py

- pub `ErrorResponseDetailsType0` class L13-50 — `{ to_dict, from_dict, additional_keys }` — Optional structured context.
- pub `to_dict` method L21-26 — `def to_dict(self) -> dict[str, Any]`
- pub `__getitem__` method L40-41 — `def __getitem__(self, key: str) -> Any`
- pub `__setitem__` method L43-44 — `def __setitem__(self, key: str, value: Any) -> None`
- pub `__delitem__` method L46-47 — `def __delitem__(self, key: str) -> None`
- pub `__contains__` method L49-50 — `def __contains__(self, key: str) -> bool`

#### sdks/python/brokkr-client/brokkr_broker_client/models/generator.py

- pub `Generator` class L18-179 — `{ to_dict, from_dict, additional_keys }` — Represents a generator in the Brokkr system.
- pub `to_dict` method L42-93 — `def to_dict(self) -> dict[str, Any]`
- pub `__getitem__` method L169-170 — `def __getitem__(self, key: str) -> Any`
- pub `__setitem__` method L172-173 — `def __setitem__(self, key: str, value: Any) -> None`
- pub `__delitem__` method L175-176 — `def __delitem__(self, key: str) -> None`
- pub `__contains__` method L178-179 — `def __contains__(self, key: str) -> bool`

#### sdks/python/brokkr-client/brokkr_broker_client/models/health_status_update.py

- pub `HealthStatusUpdate` class L17-76 — `{ to_dict, from_dict, additional_keys }` — Request body for updating health status from an agent.
- pub `to_dict` method L27-41 — `def to_dict(self) -> dict[str, Any]`
- pub `__getitem__` method L66-67 — `def __getitem__(self, key: str) -> Any`
- pub `__setitem__` method L69-70 — `def __setitem__(self, key: str, value: Any) -> None`
- pub `__delitem__` method L72-73 — `def __delitem__(self, key: str) -> None`
- pub `__contains__` method L75-76 — `def __contains__(self, key: str) -> bool`

#### sdks/python/brokkr-client/brokkr_broker_client/models/health_summary.py

- pub `HealthSummary` class L19-125 — `{ to_dict, from_dict, additional_keys }` — Structured health summary for serialization/deserialization.
- pub `to_dict` method L35-66 — `def to_dict(self) -> dict[str, Any]`
- pub `__getitem__` method L115-116 — `def __getitem__(self, key: str) -> Any`
- pub `__setitem__` method L118-119 — `def __setitem__(self, key: str, value: Any) -> None`
- pub `__delitem__` method L121-122 — `def __delitem__(self, key: str) -> None`
- pub `__contains__` method L124-125 — `def __contains__(self, key: str) -> bool`

#### sdks/python/brokkr-client/brokkr_broker_client/models/k8s_event_history_response.py

- pub `K8SEventHistoryResponse` class L18-85 — `{ to_dict, from_dict, additional_keys }` — Attributes:
- pub `to_dict` method L29-46 — `def to_dict(self) -> dict[str, Any]`
- pub `__getitem__` method L75-76 — `def __getitem__(self, key: str) -> Any`
- pub `__setitem__` method L78-79 — `def __setitem__(self, key: str, value: Any) -> None`
- pub `__delitem__` method L81-82 — `def __delitem__(self, key: str) -> None`
- pub `__contains__` method L84-85 — `def __contains__(self, key: str) -> bool`

#### sdks/python/brokkr-client/brokkr_broker_client/models/list_deliveries_query.py

- pub `ListDeliveriesQuery` class L15-113 — `{ to_dict, from_dict, additional_keys }` — Attributes:
- pub `to_dict` method L28-57 — `def to_dict(self) -> dict[str, Any]`
- pub `__getitem__` method L103-104 — `def __getitem__(self, key: str) -> Any`
- pub `__setitem__` method L106-107 — `def __setitem__(self, key: str, value: Any) -> None`
- pub `__delitem__` method L109-110 — `def __delitem__(self, key: str) -> None`
- pub `__contains__` method L112-113 — `def __contains__(self, key: str) -> bool`

#### sdks/python/brokkr-client/brokkr_broker_client/models/new_agent.py

- pub `NewAgent` class L13-70 — `{ to_dict, from_dict, additional_keys }` — Represents a new agent to be inserted into the database.
- pub `to_dict` method L25-39 — `def to_dict(self) -> dict[str, Any]`
- pub `__getitem__` method L60-61 — `def __getitem__(self, key: str) -> Any`
- pub `__setitem__` method L63-64 — `def __setitem__(self, key: str, value: Any) -> None`
- pub `__delitem__` method L66-67 — `def __delitem__(self, key: str) -> None`
- pub `__contains__` method L69-70 — `def __contains__(self, key: str) -> bool`

#### sdks/python/brokkr-client/brokkr_broker_client/models/new_agent_annotation.py

- pub `NewAgentAnnotation` class L14-79 — `{ to_dict, from_dict, additional_keys }` — Represents a new agent annotation to be inserted into the database.
- pub `to_dict` method L28-45 — `def to_dict(self) -> dict[str, Any]`
- pub `__getitem__` method L69-70 — `def __getitem__(self, key: str) -> Any`
- pub `__setitem__` method L72-73 — `def __setitem__(self, key: str, value: Any) -> None`
- pub `__delitem__` method L75-76 — `def __delitem__(self, key: str) -> None`
- pub `__contains__` method L78-79 — `def __contains__(self, key: str) -> bool`

#### sdks/python/brokkr-client/brokkr_broker_client/models/new_agent_event.py

- pub `NewAgentEvent` class L16-109 — `{ to_dict, from_dict, additional_keys }` — Represents a new agent event to be inserted into the database.
- pub `to_dict` method L34-62 — `def to_dict(self) -> dict[str, Any]`
- pub `__getitem__` method L99-100 — `def __getitem__(self, key: str) -> Any`
- pub `__setitem__` method L102-103 — `def __setitem__(self, key: str, value: Any) -> None`
- pub `__delitem__` method L105-106 — `def __delitem__(self, key: str) -> None`
- pub `__contains__` method L108-109 — `def __contains__(self, key: str) -> bool`

#### sdks/python/brokkr-client/brokkr_broker_client/models/new_agent_label.py

- pub `NewAgentLabel` class L14-71 — `{ to_dict, from_dict, additional_keys }` — Represents a new agent label to be inserted into the database.
- pub `to_dict` method L26-40 — `def to_dict(self) -> dict[str, Any]`
- pub `__getitem__` method L61-62 — `def __getitem__(self, key: str) -> Any`
- pub `__setitem__` method L64-65 — `def __setitem__(self, key: str, value: Any) -> None`
- pub `__delitem__` method L67-68 — `def __delitem__(self, key: str) -> None`
- pub `__contains__` method L70-71 — `def __contains__(self, key: str) -> bool`

#### sdks/python/brokkr-client/brokkr_broker_client/models/new_agent_target.py

- pub `NewAgentTarget` class L14-71 — `{ to_dict, from_dict, additional_keys }` — Represents a new agent target to be inserted into the database.
- pub `to_dict` method L26-40 — `def to_dict(self) -> dict[str, Any]`
- pub `__getitem__` method L61-62 — `def __getitem__(self, key: str) -> Any`
- pub `__setitem__` method L64-65 — `def __setitem__(self, key: str, value: Any) -> None`
- pub `__delitem__` method L67-68 — `def __delitem__(self, key: str) -> None`
- pub `__contains__` method L70-71 — `def __contains__(self, key: str) -> bool`

#### sdks/python/brokkr-client/brokkr_broker_client/models/new_deployment_object.py

- pub `NewDeploymentObject` class L14-87 — `{ to_dict, from_dict, additional_keys }` — Represents a new deployment object to be inserted into the database.
- pub `to_dict` method L30-50 — `def to_dict(self) -> dict[str, Any]`
- pub `__getitem__` method L77-78 — `def __getitem__(self, key: str) -> Any`
- pub `__setitem__` method L80-81 — `def __setitem__(self, key: str, value: Any) -> None`
- pub `__delitem__` method L83-84 — `def __delitem__(self, key: str) -> None`
- pub `__contains__` method L86-87 — `def __contains__(self, key: str) -> bool`

#### sdks/python/brokkr-client/brokkr_broker_client/models/new_generator.py

- pub `NewGenerator` class L15-84 — `{ to_dict, from_dict, additional_keys }` — Represents the data required to create a new generator.
- pub `to_dict` method L27-46 — `def to_dict(self) -> dict[str, Any]`
- pub `__getitem__` method L74-75 — `def __getitem__(self, key: str) -> Any`
- pub `__setitem__` method L77-78 — `def __setitem__(self, key: str, value: Any) -> None`
- pub `__delitem__` method L80-81 — `def __delitem__(self, key: str) -> None`
- pub `__contains__` method L83-84 — `def __contains__(self, key: str) -> bool`

#### sdks/python/brokkr-client/brokkr_broker_client/models/new_stack.py

- pub `NewStack` class L16-93 — `{ to_dict, from_dict, additional_keys }` — Represents a new stack to be inserted into the database.
- pub `to_dict` method L30-52 — `def to_dict(self) -> dict[str, Any]`
- pub `__getitem__` method L83-84 — `def __getitem__(self, key: str) -> Any`
- pub `__setitem__` method L86-87 — `def __setitem__(self, key: str, value: Any) -> None`
- pub `__delitem__` method L89-90 — `def __delitem__(self, key: str) -> None`
- pub `__contains__` method L92-93 — `def __contains__(self, key: str) -> bool`

#### sdks/python/brokkr-client/brokkr_broker_client/models/new_stack_annotation.py

- pub `NewStackAnnotation` class L14-79 — `{ to_dict, from_dict, additional_keys }` — Represents a new stack annotation to be inserted into the database.
- pub `to_dict` method L28-45 — `def to_dict(self) -> dict[str, Any]`
- pub `__getitem__` method L69-70 — `def __getitem__(self, key: str) -> Any`
- pub `__setitem__` method L72-73 — `def __setitem__(self, key: str, value: Any) -> None`
- pub `__delitem__` method L75-76 — `def __delitem__(self, key: str) -> None`
- pub `__contains__` method L78-79 — `def __contains__(self, key: str) -> bool`

#### sdks/python/brokkr-client/brokkr_broker_client/models/new_stack_label.py

- pub `NewStackLabel` class L14-71 — `{ to_dict, from_dict, additional_keys }` — Represents a new stack label to be inserted into the database.
- pub `to_dict` method L26-40 — `def to_dict(self) -> dict[str, Any]`
- pub `__getitem__` method L61-62 — `def __getitem__(self, key: str) -> Any`
- pub `__setitem__` method L64-65 — `def __setitem__(self, key: str, value: Any) -> None`
- pub `__delitem__` method L67-68 — `def __delitem__(self, key: str) -> None`
- pub `__contains__` method L70-71 — `def __contains__(self, key: str) -> bool`

#### sdks/python/brokkr-client/brokkr_broker_client/models/new_stack_template.py

- pub `NewStackTemplate` class L16-147 — `{ to_dict, from_dict, additional_keys }` — Represents a new stack template to be inserted into the database.
- pub `to_dict` method L38-79 — `def to_dict(self) -> dict[str, Any]`
- pub `__getitem__` method L137-138 — `def __getitem__(self, key: str) -> Any`
- pub `__setitem__` method L140-141 — `def __setitem__(self, key: str, value: Any) -> None`
- pub `__delitem__` method L143-144 — `def __delitem__(self, key: str) -> None`
- pub `__contains__` method L146-147 — `def __contains__(self, key: str) -> bool`

#### sdks/python/brokkr-client/brokkr_broker_client/models/new_template_annotation.py

- pub `NewTemplateAnnotation` class L14-79 — `{ to_dict, from_dict, additional_keys }` — Represents a new template annotation to be inserted into the database.
- pub `to_dict` method L28-45 — `def to_dict(self) -> dict[str, Any]`
- pub `__getitem__` method L69-70 — `def __getitem__(self, key: str) -> Any`
- pub `__setitem__` method L72-73 — `def __setitem__(self, key: str, value: Any) -> None`
- pub `__delitem__` method L75-76 — `def __delitem__(self, key: str) -> None`
- pub `__contains__` method L78-79 — `def __contains__(self, key: str) -> bool`

#### sdks/python/brokkr-client/brokkr_broker_client/models/new_template_label.py

- pub `NewTemplateLabel` class L14-71 — `{ to_dict, from_dict, additional_keys }` — Represents a new template label to be inserted into the database.
- pub `to_dict` method L26-40 — `def to_dict(self) -> dict[str, Any]`
- pub `__getitem__` method L61-62 — `def __getitem__(self, key: str) -> Any`
- pub `__setitem__` method L64-65 — `def __setitem__(self, key: str, value: Any) -> None`
- pub `__delitem__` method L67-68 — `def __delitem__(self, key: str) -> None`
- pub `__contains__` method L70-71 — `def __contains__(self, key: str) -> bool`

#### sdks/python/brokkr-client/brokkr_broker_client/models/pending_webhook_delivery.py

- pub `PendingWebhookDelivery` class L16-140 — `{ to_dict, from_dict, additional_keys }` — Attributes:
- pub `to_dict` method L41-81 — `def to_dict(self) -> dict[str, Any]`
- pub `__getitem__` method L130-131 — `def __getitem__(self, key: str) -> Any`
- pub `__setitem__` method L133-134 — `def __setitem__(self, key: str, value: Any) -> None`
- pub `__delitem__` method L136-137 — `def __delitem__(self, key: str) -> None`
- pub `__contains__` method L139-140 — `def __contains__(self, key: str) -> bool`

#### sdks/python/brokkr-client/brokkr_broker_client/models/pod_log_history_response.py

- pub `PodLogHistoryResponse` class L18-85 — `{ to_dict, from_dict, additional_keys }` — Attributes:
- pub `to_dict` method L29-46 — `def to_dict(self) -> dict[str, Any]`
- pub `__getitem__` method L75-76 — `def __getitem__(self, key: str) -> Any`
- pub `__setitem__` method L78-79 — `def __setitem__(self, key: str, value: Any) -> None`
- pub `__delitem__` method L81-82 — `def __delitem__(self, key: str) -> None`
- pub `__contains__` method L84-85 — `def __contains__(self, key: str) -> bool`

#### sdks/python/brokkr-client/brokkr_broker_client/models/resource_health.py

- pub `ResourceHealth` class L15-108 — `{ to_dict, from_dict, additional_keys }` — Health status for an individual Kubernetes resource.
- pub `to_dict` method L33-61 — `def to_dict(self) -> dict[str, Any]`
- pub `__getitem__` method L98-99 — `def __getitem__(self, key: str) -> Any`
- pub `__setitem__` method L101-102 — `def __setitem__(self, key: str, value: Any) -> None`
- pub `__delitem__` method L104-105 — `def __delitem__(self, key: str) -> None`
- pub `__contains__` method L107-108 — `def __contains__(self, key: str) -> bool`

#### sdks/python/brokkr-client/brokkr_broker_client/models/retention_info.py

- pub `RetentionInfo` class L17-114 — `{ to_dict, from_dict, additional_keys }` — Attributes:
- pub `to_dict` method L35-62 — `def to_dict(self) -> dict[str, Any]`
- pub `__getitem__` method L104-105 — `def __getitem__(self, key: str) -> Any`
- pub `__setitem__` method L107-108 — `def __setitem__(self, key: str, value: Any) -> None`
- pub `__delitem__` method L110-111 — `def __delitem__(self, key: str) -> None`
- pub `__contains__` method L113-114 — `def __contains__(self, key: str) -> bool`

#### sdks/python/brokkr-client/brokkr_broker_client/models/stack.py

- pub `Stack` class L18-149 — `{ to_dict, from_dict, additional_keys }` — Represents a stack in the database.
- pub `to_dict` method L40-81 — `def to_dict(self) -> dict[str, Any]`
- pub `__getitem__` method L139-140 — `def __getitem__(self, key: str) -> Any`
- pub `__setitem__` method L142-143 — `def __setitem__(self, key: str, value: Any) -> None`
- pub `__delitem__` method L145-146 — `def __delitem__(self, key: str) -> None`
- pub `__contains__` method L148-149 — `def __contains__(self, key: str) -> bool`

#### sdks/python/brokkr-client/brokkr_broker_client/models/stack_annotation.py

- pub `StackAnnotation` class L14-87 — `{ to_dict, from_dict, additional_keys }` — Represents a stack annotation in the database.
- pub `to_dict` method L30-50 — `def to_dict(self) -> dict[str, Any]`
- pub `__getitem__` method L77-78 — `def __getitem__(self, key: str) -> Any`
- pub `__setitem__` method L80-81 — `def __setitem__(self, key: str, value: Any) -> None`
- pub `__delitem__` method L83-84 — `def __delitem__(self, key: str) -> None`
- pub `__contains__` method L86-87 — `def __contains__(self, key: str) -> bool`

#### sdks/python/brokkr-client/brokkr_broker_client/models/stack_health_response.py

- pub `StackHealthResponse` class L18-93 — `{ to_dict, from_dict, additional_keys }` — Response for stack health query.
- pub `to_dict` method L32-52 — `def to_dict(self) -> dict[str, Any]`
- pub `__getitem__` method L83-84 — `def __getitem__(self, key: str) -> Any`
- pub `__setitem__` method L86-87 — `def __setitem__(self, key: str, value: Any) -> None`
- pub `__delitem__` method L89-90 — `def __delitem__(self, key: str) -> None`
- pub `__contains__` method L92-93 — `def __contains__(self, key: str) -> bool`

#### sdks/python/brokkr-client/brokkr_broker_client/models/stack_label.py

- pub `StackLabel` class L14-79 — `{ to_dict, from_dict, additional_keys }` — Represents a stack label in the database.
- pub `to_dict` method L28-45 — `def to_dict(self) -> dict[str, Any]`
- pub `__getitem__` method L69-70 — `def __getitem__(self, key: str) -> Any`
- pub `__setitem__` method L72-73 — `def __setitem__(self, key: str, value: Any) -> None`
- pub `__delitem__` method L75-76 — `def __delitem__(self, key: str) -> None`
- pub `__contains__` method L78-79 — `def __contains__(self, key: str) -> bool`

#### sdks/python/brokkr-client/brokkr_broker_client/models/stack_template.py

- pub `StackTemplate` class L18-203 — `{ to_dict, from_dict, additional_keys }` — Represents a stack template in the database.
- pub `to_dict` method L48-108 — `def to_dict(self) -> dict[str, Any]`
- pub `__getitem__` method L193-194 — `def __getitem__(self, key: str) -> Any`
- pub `__setitem__` method L196-197 — `def __setitem__(self, key: str, value: Any) -> None`
- pub `__delitem__` method L199-200 — `def __delitem__(self, key: str) -> None`
- pub `__contains__` method L202-203 — `def __contains__(self, key: str) -> bool`

#### sdks/python/brokkr-client/brokkr_broker_client/models/submit_diagnostic_result.py

- pub `SubmitDiagnosticResult` class L17-102 — `{ to_dict, from_dict, additional_keys }` — Request body for submitting diagnostic results.
- pub `to_dict` method L33-58 — `def to_dict(self) -> dict[str, Any]`
- pub `__getitem__` method L92-93 — `def __getitem__(self, key: str) -> Any`
- pub `__setitem__` method L95-96 — `def __setitem__(self, key: str, value: Any) -> None`
- pub `__delitem__` method L98-99 — `def __delitem__(self, key: str) -> None`
- pub `__contains__` method L101-102 — `def __contains__(self, key: str) -> bool`

#### sdks/python/brokkr-client/brokkr_broker_client/models/template_annotation.py

- pub `TemplateAnnotation` class L16-97 — `{ to_dict, from_dict, additional_keys }` — Represents a template annotation in the database.
- pub `to_dict` method L34-57 — `def to_dict(self) -> dict[str, Any]`
- pub `__getitem__` method L87-88 — `def __getitem__(self, key: str) -> Any`
- pub `__setitem__` method L90-91 — `def __setitem__(self, key: str, value: Any) -> None`
- pub `__delitem__` method L93-94 — `def __delitem__(self, key: str) -> None`
- pub `__contains__` method L96-97 — `def __contains__(self, key: str) -> bool`

#### sdks/python/brokkr-client/brokkr_broker_client/models/template_instantiation_request.py

- pub `TemplateInstantiationRequest` class L14-70 — `{ to_dict, from_dict, additional_keys }` — Attributes:
- pub `to_dict` method L25-39 — `def to_dict(self) -> dict[str, Any]`
- pub `__getitem__` method L60-61 — `def __getitem__(self, key: str) -> Any`
- pub `__setitem__` method L63-64 — `def __setitem__(self, key: str, value: Any) -> None`
- pub `__delitem__` method L66-67 — `def __delitem__(self, key: str) -> None`
- pub `__contains__` method L69-70 — `def __contains__(self, key: str) -> bool`

#### sdks/python/brokkr-client/brokkr_broker_client/models/template_label.py

- pub `TemplateLabel` class L16-89 — `{ to_dict, from_dict, additional_keys }` — Represents a template label in the database.
- pub `to_dict` method L32-52 — `def to_dict(self) -> dict[str, Any]`
- pub `__getitem__` method L79-80 — `def __getitem__(self, key: str) -> Any`
- pub `__setitem__` method L82-83 — `def __setitem__(self, key: str, value: Any) -> None`
- pub `__delitem__` method L85-86 — `def __delitem__(self, key: str) -> None`
- pub `__contains__` method L88-89 — `def __contains__(self, key: str) -> bool`

#### sdks/python/brokkr-client/brokkr_broker_client/models/update_template_request.py

- pub `UpdateTemplateRequest` class L15-91 — `{ to_dict, from_dict, additional_keys }` — Attributes:
- pub `to_dict` method L28-50 — `def to_dict(self) -> dict[str, Any]`
- pub `__getitem__` method L81-82 — `def __getitem__(self, key: str) -> Any`
- pub `__setitem__` method L84-85 — `def __setitem__(self, key: str, value: Any) -> None`
- pub `__delitem__` method L87-88 — `def __delitem__(self, key: str) -> None`
- pub `__contains__` method L90-91 — `def __contains__(self, key: str) -> bool`

#### sdks/python/brokkr-client/brokkr_broker_client/models/update_webhook_request.py

- pub `UpdateWebhookRequest` class L19-257 — `{ to_dict, from_dict, additional_keys }` — Attributes:
- pub `to_dict` method L44-125 — `def to_dict(self) -> dict[str, Any]`
- pub `__getitem__` method L247-248 — `def __getitem__(self, key: str) -> Any`
- pub `__setitem__` method L250-251 — `def __setitem__(self, key: str, value: Any) -> None`
- pub `__delitem__` method L253-254 — `def __delitem__(self, key: str) -> None`
- pub `__contains__` method L256-257 — `def __contains__(self, key: str) -> bool`

#### sdks/python/brokkr-client/brokkr_broker_client/models/webhook_delivery.py

- pub `WebhookDelivery` class L18-339 — `{ to_dict, from_dict, additional_keys }` — A webhook delivery record from the database.
- pub `to_dict` method L56-161 — `def to_dict(self) -> dict[str, Any]`
- pub `__getitem__` method L329-330 — `def __getitem__(self, key: str) -> Any`
- pub `__setitem__` method L332-333 — `def __setitem__(self, key: str, value: Any) -> None`
- pub `__delitem__` method L335-336 — `def __delitem__(self, key: str) -> None`
- pub `__contains__` method L338-339 — `def __contains__(self, key: str) -> bool`

#### sdks/python/brokkr-client/brokkr_broker_client/models/webhook_filters.py

- pub `WebhookFilters` class L20-153 — `{ to_dict, from_dict, additional_keys }` — Filters for webhook subscriptions.
- pub `to_dict` method L34-71 — `def to_dict(self) -> dict[str, Any]`
- pub `__getitem__` method L143-144 — `def __getitem__(self, key: str) -> Any`
- pub `__setitem__` method L146-147 — `def __setitem__(self, key: str, value: Any) -> None`
- pub `__delitem__` method L149-150 — `def __delitem__(self, key: str) -> None`
- pub `__contains__` method L152-153 — `def __contains__(self, key: str) -> bool`

#### sdks/python/brokkr-client/brokkr_broker_client/models/webhook_filters_labels_type_0.py

- pub `WebhookFiltersLabelsType0` class L13-47 — `{ to_dict, from_dict, additional_keys }` — Filter by labels (all must match).
- pub `to_dict` method L18-23 — `def to_dict(self) -> dict[str, Any]`
- pub `__getitem__` method L37-38 — `def __getitem__(self, key: str) -> str`
- pub `__setitem__` method L40-41 — `def __setitem__(self, key: str, value: str) -> None`
- pub `__delitem__` method L43-44 — `def __delitem__(self, key: str) -> None`
- pub `__contains__` method L46-47 — `def __contains__(self, key: str) -> bool`

#### sdks/python/brokkr-client/brokkr_broker_client/models/webhook_response.py

- pub `WebhookResponse` class L22-211 — `{ to_dict, from_dict, additional_keys }` — Attributes:
- pub `to_dict` method L55-118 — `def to_dict(self) -> dict[str, Any]`
- pub `__getitem__` method L201-202 — `def __getitem__(self, key: str) -> Any`
- pub `__setitem__` method L204-205 — `def __setitem__(self, key: str, value: Any) -> None`
- pub `__delitem__` method L207-208 — `def __delitem__(self, key: str) -> None`
- pub `__contains__` method L210-211 — `def __contains__(self, key: str) -> bool`

#### sdks/python/brokkr-client/brokkr_broker_client/models/webhook_subscription.py

- pub `WebhookSubscription` class L18-224 — `{ to_dict, from_dict, additional_keys }` — A webhook subscription record from the database.
- pub `to_dict` method L48-115 — `def to_dict(self) -> dict[str, Any]`
- pub `__getitem__` method L214-215 — `def __getitem__(self, key: str) -> Any`
- pub `__setitem__` method L217-218 — `def __setitem__(self, key: str, value: Any) -> None`
- pub `__delitem__` method L220-221 — `def __delitem__(self, key: str) -> None`
- pub `__contains__` method L223-224 — `def __contains__(self, key: str) -> bool`

#### sdks/python/brokkr-client/brokkr_broker_client/models/work_order.py

- pub `WorkOrder` class L18-290 — `{ to_dict, from_dict, additional_keys }` — r"""Represents an active work order in the queue.
- pub `to_dict` method L67-153 — `def to_dict(self) -> dict[str, Any]`
- pub `__getitem__` method L280-281 — `def __getitem__(self, key: str) -> Any`
- pub `__setitem__` method L283-284 — `def __setitem__(self, key: str, value: Any) -> None`
- pub `__delitem__` method L286-287 — `def __delitem__(self, key: str) -> None`
- pub `__contains__` method L289-290 — `def __contains__(self, key: str) -> bool`

#### sdks/python/brokkr-client/brokkr_broker_client/models/work_order_log.py

- pub `WorkOrderLog` class L18-207 — `{ to_dict, from_dict, additional_keys }` — r"""Represents a completed work order in the audit log.
- pub `to_dict` method L58-115 — `def to_dict(self) -> dict[str, Any]`
- pub `__getitem__` method L197-198 — `def __getitem__(self, key: str) -> Any`
- pub `__setitem__` method L200-201 — `def __setitem__(self, key: str, value: Any) -> None`
- pub `__delitem__` method L203-204 — `def __delitem__(self, key: str) -> None`
- pub `__contains__` method L206-207 — `def __contains__(self, key: str) -> bool`

#### sdks/python/brokkr-client/brokkr_broker_client/models/work_order_targeting.py

- pub `WorkOrderTargeting` class L20-162 — `{ to_dict, from_dict, additional_keys }` — Attributes:
- pub `to_dict` method L33-75 — `def to_dict(self) -> dict[str, Any]`
- pub `__getitem__` method L152-153 — `def __getitem__(self, key: str) -> Any`
- pub `__setitem__` method L155-156 — `def __setitem__(self, key: str, value: Any) -> None`
- pub `__delitem__` method L158-159 — `def __delitem__(self, key: str) -> None`
- pub `__contains__` method L161-162 — `def __contains__(self, key: str) -> bool`

#### sdks/python/brokkr-client/brokkr_broker_client/models/work_order_targeting_annotations_type_0.py

- pub `WorkOrderTargetingAnnotationsType0` class L13-47 — `{ to_dict, from_dict, additional_keys }`
- pub `to_dict` method L18-23 — `def to_dict(self) -> dict[str, Any]`
- pub `__getitem__` method L37-38 — `def __getitem__(self, key: str) -> str`
- pub `__setitem__` method L40-41 — `def __setitem__(self, key: str, value: str) -> None`
- pub `__delitem__` method L43-44 — `def __delitem__(self, key: str) -> None`
- pub `__contains__` method L46-47 — `def __contains__(self, key: str) -> bool`

#### sdks/python/brokkr-client/brokkr_broker_client/models/ws_connection_info.py

- pub `WsConnectionInfo` class L16-88 — `{ to_dict, from_dict, additional_keys }` — Attributes:
- pub `to_dict` method L31-51 — `def to_dict(self) -> dict[str, Any]`
- pub `__getitem__` method L78-79 — `def __getitem__(self, key: str) -> Any`
- pub `__setitem__` method L81-82 — `def __setitem__(self, key: str, value: Any) -> None`
- pub `__delitem__` method L84-85 — `def __delitem__(self, key: str) -> None`
- pub `__contains__` method L87-88 — `def __contains__(self, key: str) -> bool`

#### sdks/python/brokkr-client/brokkr_broker_client/models/ws_connections_response.py

- pub `WsConnectionsResponse` class L17-91 — `{ to_dict, from_dict, additional_keys }` — Attributes:
- pub `to_dict` method L30-50 — `def to_dict(self) -> dict[str, Any]`
- pub `__getitem__` method L81-82 — `def __getitem__(self, key: str) -> Any`
- pub `__setitem__` method L84-85 — `def __setitem__(self, key: str, value: Any) -> None`
- pub `__delitem__` method L87-88 — `def __delitem__(self, key: str) -> None`
- pub `__contains__` method L90-91 — `def __contains__(self, key: str) -> bool`

### sdks/python/brokkr-client/tests

> *Semantic summary to be generated by AI agent.*

#### sdks/python/brokkr-client/tests/test_helpers.py

- pub `test_helpers_module_exposes_expected_surface` function L19-31 — `def test_helpers_module_exposes_expected_surface() -> None`
- pub `test_live_subscription_url_http_to_ws` function L34-39 — `def test_live_subscription_url_http_to_ws() -> None`
- pub `test_live_subscription_url_https_to_wss` function L42-47 — `def test_live_subscription_url_https_to_wss() -> None`
- pub `test_live_subscription_url_strips_trailing_slash` function L50-55 — `def test_live_subscription_url_strips_trailing_slash() -> None`
- pub `test_history_helper_signatures_include_keyword_filters` function L58-65 — `def test_history_helper_signatures_include_keyword_filters() -> None`
- pub `test_list_ws_connections_takes_only_a_client` function L68-70 — `def test_list_ws_connections_takes_only_a_client() -> None`

#### sdks/python/brokkr-client/tests/test_surface.py

- pub `test_clients_construct` function L38-40 — `def test_clients_construct() -> None`
- pub `test_endpoints_expose_sync_and_async` function L43-60 — `def test_endpoints_expose_sync_and_async() -> None`
- pub `test_error_response_shape` function L63-68 — `def test_error_response_shape() -> None`
- pub `test_list_agents_return_type_includes_error_response` function L71-78 — `def test_list_agents_return_type_includes_error_response() -> None`

### sdks/typescript/brokkr-client/src

> *Semantic summary to be generated by AI agent.*

#### sdks/typescript/brokkr-client/src/client.ts

- pub `ApplyResult` type L30-33 — `= | { status: "created"; deploymentObject: DeploymentObject } | { status: "updat...`
- pub `TelemetryHistoryQuery` interface L35-41 — `{ since: : string, limit: : number }`
- pub `BrokkrClientOptions` interface L43-53 — `{ baseUrl: : string, token: : string, requestTimeoutMs: : number, maxRetries: : ...`
- pub `BrokkrClient` class L69-317 — `-`
- pub `constructor` method L75-111 — `constructor(options: BrokkrClientOptions)`
- pub `listTelemetryEvents` method L122-131 — `listTelemetryEvents( stackId: string, query: TelemetryHistoryQuery = {}, ): Prom...`
- pub `listTelemetryLogs` method L134-143 — `listTelemetryLogs( stackId: string, query: TelemetryHistoryQuery = {}, ): Promis...`
- pub `listWsConnections` method L148-152 — `listWsConnections(): Promise<WsConnectionsResponse>`
- pub `submitManifests` method L164-175 — `submitManifests( stackId: string, path: string, ): Promise<DeploymentObject>`
- pub `apply` method L183-247 — `apply( stackName: string, path: string, targeting: string[] = [], ): Promise<App...`
- pub `liveSubscriptionUrl` method L260-272 — `liveSubscriptionUrl(stackId: string): string`
- pub `retry` method L286-316 — `retry(op: (api: BrokkrApi) => Promise<FetchResult<T>>): Promise<T>`
- pub `readManifests` function L381-422 — `function readManifests(path: string): Promise<string>`
- pub `sha256Hex` function L428-431 — `function sha256Hex(content: string): Promise<string>`
-  `FetchResult` type L63-67 — `= { data?: T; error?: unknown; response: Response; }`
-  `customFetch` function L93-102 — `const customFetch = (input, init)`
-  `classify` function L319-347 — `function classify( result: FetchResult<T> | undefined, transportErr: unknown, ):...`
-  `sleep` function L349-351 — `function sleep(ms: number): Promise<void>`
-  `mergeSignals` function L354-366 — `function mergeSignals(signals: AbortSignal[]): AbortSignal`

#### sdks/typescript/brokkr-client/src/error.ts

- pub `BrokkrError` class L18-85 — `extends Error`
- pub `constructor` method L23-36 — `constructor(args: { message: string; code?: string; status?: number; response?: ...`
- pub `isRetryable` method L41-44 — `isRetryable(): boolean`
- pub `fromResponse` method L47-54 — `fromResponse(response: ErrorResponse, status: number): BrokkrError`
- pub `fromTransport` method L57-61 — `fromTransport(cause: unknown): BrokkrError`
- pub `fromOpenapiFetch` method L65-84 — `fromOpenapiFetch( error: unknown, response: Response, ): BrokkrError`

#### sdks/typescript/brokkr-client/src/index.ts

- pub `ErrorResponse` type L30 — `= components["schemas"]["ErrorResponse"]`
- pub `Agent` type L31 — `= components["schemas"]["Agent"]`
- pub `Stack` type L32 — `= components["schemas"]["Stack"]`
- pub `WorkOrder` type L33 — `= components["schemas"]["WorkOrder"]`
- pub `WorkOrderLog` type L34 — `= components["schemas"]["WorkOrderLog"]`
- pub `DeploymentObject` type L35 — `= components["schemas"]["DeploymentObject"]`
- pub `StackTemplate` type L36 — `= components["schemas"]["StackTemplate"]`
- pub `AuthResponse` type L37 — `= components["schemas"]["AuthResponse"]`
- pub `WebhookResponse` type L38 — `= components["schemas"]["WebhookResponse"]`
- pub `PendingWebhookDelivery` type L39 — `= components["schemas"]["PendingWebhookDelivery"]`
- pub `K8sEventHistoryResponse` type L42-43 — `= components["schemas"]["K8sEventHistoryResponse"]`
- pub `PodLogHistoryResponse` type L44-45 — `= components["schemas"]["PodLogHistoryResponse"]`
- pub `RetentionInfo` type L46 — `= components["schemas"]["RetentionInfo"]`
- pub `WsConnectionsResponse` type L47-48 — `= components["schemas"]["WsConnectionsResponse"]`
- pub `WsConnectionInfo` type L49 — `= components["schemas"]["WsConnectionInfo"]`
- pub `createBrokkrClient` function L58-60 — `function createBrokkrClient(options: ClientOptions = {})`
- pub `BrokkrApi` type L63 — `= ReturnType<typeof createBrokkrClient>`

#### sdks/typescript/brokkr-client/src/manifests.test.ts

-  `tmp` function L8-10 — `function tmp(): string`

#### sdks/typescript/brokkr-client/src/schema.d.ts

- pub `paths` interface L6-1071 — `{ "/admin/audit-logs": : { parameters: { query?: never; header?: never; path?: n...`
- pub `webhooks` type L1072 — `= Record<string, never>`
- pub `components` interface L1073-2525 — `{ schemas: : { AddAnnotationRequest: { key: string; value: string; }; /** @descr...`
- pub `$defs` type L2526 — `= Record<string, never>`
- pub `operations` interface L2527-6886 — `{ list_audit_logs: : { parameters: { query?: { /** * @description Filter by acto...`

#### sdks/typescript/brokkr-client/src/wrapper.test.ts

-  `scriptedFetch` function L11-47 — `function scriptedFetch( steps: Array<{ status: number; body?: object } | { throw...`
-  `impl` function L16-45 — `const impl = (input, init)`

### tests/e2e/src

> *Semantic summary to be generated by AI agent.*

#### tests/e2e/src/api.rs

- pub `Result` type L17 — `= std::result::Result<T, Box<dyn std::error::Error>>` — HTTP API client for the Brokkr broker.
- pub `Client` struct L20-24 — `{ http: reqwest::Client, base_url: String, admin_pak: String }` — API client for the Brokkr broker
- pub `new` function L27-33 — `(base_url: &str, admin_pak: &str) -> Self` — HTTP API client for the Brokkr broker.
- pub `wait_for_ready` function L36-54 — `(&self, timeout_secs: u64) -> Result<()>` — Wait for the broker to be ready
- pub `get_json` function L96-98 — `(&self, path: &str) -> Result<Value>` — Public GET that returns a raw `serde_json::Value`.
- pub `list_agents` function L128-130 — `(&self) -> Result<Vec<Value>>` — HTTP API client for the Brokkr broker.
- pub `create_agent` function L132-141 — `(&self, name: &str, cluster: &str) -> Result<Value>` — HTTP API client for the Brokkr broker.
- pub `get_agent` function L143-145 — `(&self, id: Uuid) -> Result<Value>` — HTTP API client for the Brokkr broker.
- pub `update_agent` function L147-149 — `(&self, id: Uuid, updates: Value) -> Result<Value>` — HTTP API client for the Brokkr broker.
- pub `add_agent_label` function L151-160 — `(&self, id: Uuid, label: &str) -> Result<Value>` — HTTP API client for the Brokkr broker.
- pub `get_agent_labels` function L162-164 — `(&self, id: Uuid) -> Result<Vec<Value>>` — HTTP API client for the Brokkr broker.
- pub `add_agent_annotation` function L166-176 — `(&self, id: Uuid, key: &str, value: &str) -> Result<Value>` — HTTP API client for the Brokkr broker.
- pub `get_agent_annotations` function L178-181 — `(&self, id: Uuid) -> Result<Vec<Value>>` — HTTP API client for the Brokkr broker.
- pub `add_agent_target` function L183-192 — `(&self, agent_id: Uuid, stack_id: Uuid) -> Result<Value>` — HTTP API client for the Brokkr broker.
- pub `get_agent_targets` function L194-196 — `(&self, id: Uuid) -> Result<Vec<Value>>` — HTTP API client for the Brokkr broker.
- pub `get_agent_stacks` function L198-200 — `(&self, id: Uuid) -> Result<Vec<Value>>` — HTTP API client for the Brokkr broker.
- pub `get_agent_target_state` function L202-208 — `(&self, id: Uuid, mode: Option<&str>) -> Result<Vec<Value>>` — HTTP API client for the Brokkr broker.
- pub `create_generator` function L214-223 — `(&self, name: &str, description: Option<&str>) -> Result<Value>` — HTTP API client for the Brokkr broker.
- pub `list_generators` function L225-227 — `(&self) -> Result<Vec<Value>>` — HTTP API client for the Brokkr broker.
- pub `create_stack` function L233-248 — `( &self, name: &str, description: Option<&str>, generator_id: Uuid, ) -> Result<...` — HTTP API client for the Brokkr broker.
- pub `list_stacks` function L250-252 — `(&self) -> Result<Vec<Value>>` — HTTP API client for the Brokkr broker.
- pub `get_stack` function L254-256 — `(&self, id: Uuid) -> Result<Value>` — HTTP API client for the Brokkr broker.
- pub `add_stack_label` function L258-262 — `(&self, id: Uuid, label: &str) -> Result<Value>` — HTTP API client for the Brokkr broker.
- pub `get_stack_labels` function L264-266 — `(&self, id: Uuid) -> Result<Vec<Value>>` — HTTP API client for the Brokkr broker.
- pub `add_stack_annotation` function L268-278 — `(&self, id: Uuid, key: &str, value: &str) -> Result<Value>` — HTTP API client for the Brokkr broker.
- pub `create_deployment` function L284-301 — `( &self, stack_id: Uuid, yaml: &str, is_deletion: bool, ) -> Result<Value>` — HTTP API client for the Brokkr broker.
- pub `list_deployments` function L303-306 — `(&self, stack_id: Uuid) -> Result<Vec<Value>>` — HTTP API client for the Brokkr broker.
- pub `get_deployment` function L308-311 — `(&self, id: Uuid) -> Result<Value>` — HTTP API client for the Brokkr broker.
- pub `get_deployment_health` function L313-316 — `(&self, id: Uuid) -> Result<Value>` — HTTP API client for the Brokkr broker.
- pub `get_stack_health` function L318-320 — `(&self, id: Uuid) -> Result<Value>` — HTTP API client for the Brokkr broker.
- pub `create_template` function L326-343 — `( &self, name: &str, description: Option<&str>, content: &str, schema: &str, ) -...` — HTTP API client for the Brokkr broker.
- pub `list_templates` function L345-347 — `(&self) -> Result<Vec<Value>>` — HTTP API client for the Brokkr broker.
- pub `instantiate_template` function L349-366 — `( &self, stack_id: Uuid, template_id: Uuid, parameters: Value, ) -> Result<Value...` — HTTP API client for the Brokkr broker.
- pub `delete_template` function L368-370 — `(&self, id: Uuid) -> Result<()>` — HTTP API client for the Brokkr broker.
- pub `create_work_order` function L376-399 — `( &self, work_type: &str, yaml: &str, target_agent_ids: Option<Vec<Uuid>>, targe...` — HTTP API client for the Brokkr broker.
- pub `list_work_orders` function L401-403 — `(&self) -> Result<Vec<Value>>` — HTTP API client for the Brokkr broker.
- pub `get_work_order` function L405-407 — `(&self, id: Uuid) -> Result<Value>` — HTTP API client for the Brokkr broker.
- pub `get_work_order_log` function L409-411 — `(&self, id: Uuid) -> Result<Value>` — HTTP API client for the Brokkr broker.
- pub `delete_work_order` function L413-415 — `(&self, id: Uuid) -> Result<()>` — HTTP API client for the Brokkr broker.
- pub `create_diagnostic` function L421-431 — `(&self, deployment_id: Uuid, agent_id: Uuid) -> Result<Value>` — HTTP API client for the Brokkr broker.
- pub `get_diagnostic` function L433-435 — `(&self, id: Uuid) -> Result<Value>` — HTTP API client for the Brokkr broker.
- pub `create_webhook` function L441-450 — `( &self, name: &str, url: &str, event_types: Vec<&str>, auth_header: Option<&str...` — HTTP API client for the Brokkr broker.
- pub `create_webhook_with_options` function L452-476 — `( &self, name: &str, url: &str, event_types: Vec<&str>, auth_header: Option<&str...` — HTTP API client for the Brokkr broker.
- pub `list_webhooks` function L478-480 — `(&self) -> Result<Vec<Value>>` — HTTP API client for the Brokkr broker.
- pub `get_webhook` function L482-484 — `(&self, id: Uuid) -> Result<Value>` — HTTP API client for the Brokkr broker.
- pub `update_webhook` function L486-488 — `(&self, id: Uuid, updates: Value) -> Result<Value>` — HTTP API client for the Brokkr broker.
- pub `delete_webhook` function L490-492 — `(&self, id: Uuid) -> Result<()>` — HTTP API client for the Brokkr broker.
- pub `list_webhook_deliveries` function L494-497 — `(&self, webhook_id: Uuid) -> Result<Vec<Value>>` — HTTP API client for the Brokkr broker.
- pub `test_webhook` function L499-502 — `(&self, id: Uuid) -> Result<Value>` — HTTP API client for the Brokkr broker.
- pub `list_audit_logs` function L508-514 — `(&self, limit: Option<i32>) -> Result<Value>` — HTTP API client for the Brokkr broker.
- pub `get_metrics` function L521-532 — `(&self) -> Result<String>` — Fetch Prometheus metrics from the broker
- pub `metric_value` function L543-568 — `(&self, name: &str, labels: &[(&str, &str)]) -> Result<f64>` — Parse a single Prometheus metric value from the broker's `/metrics`
- pub `wait_for_metric` function L573-603 — `( &self, name: &str, labels: &[(&str, &str)], timeout_secs: u64, predicate: F, )...` — Poll `metric_value` until `predicate` is true or `timeout_secs` elapses.
- pub `get_healthz` function L606-617 — `(&self) -> Result<String>` — Fetch health check endpoint
- pub `WebhookCatcher` struct L621-624 — `{ http: reqwest::Client, base_url: String }` — Client for webhook-catcher test service
- pub `new` function L627-632 — `(base_url: &str) -> Self` — HTTP API client for the Brokkr broker.
- pub `get_messages` function L635-646 — `(&self) -> Result<Value>` — Get all messages received by webhook-catcher
- pub `clear_messages` function L649-659 — `(&self) -> Result<()>` — Clear all messages from webhook-catcher
- pub `wait_for_messages` function L662-682 — `(&self, count: usize, timeout_secs: u64) -> Result<Value>` — Wait for at least N messages to arrive, with timeout
-  `Client` type L26-618 — `= Client` — HTTP API client for the Brokkr broker.
-  `request` function L56-87 — `( &self, method: reqwest::Method, path: &str, body: Option<Value>, ) -> Result<T...` — HTTP API client for the Brokkr broker.
-  `get` function L89-91 — `(&self, path: &str) -> Result<T>` — HTTP API client for the Brokkr broker.
-  `post` function L100-102 — `(&self, path: &str, body: Value) -> Result<T>` — HTTP API client for the Brokkr broker.
-  `put` function L104-106 — `(&self, path: &str, body: Value) -> Result<T>` — HTTP API client for the Brokkr broker.
-  `delete` function L108-122 — `(&self, path: &str) -> Result<()>` — HTTP API client for the Brokkr broker.
-  `WebhookCatcher` type L626-683 — `= WebhookCatcher` — HTTP API client for the Brokkr broker.
-  `sha256_hex` function L685-689 — `(data: &str) -> String` — HTTP API client for the Brokkr broker.

#### tests/e2e/src/main.rs

-  `api` module L18 — `-` — Brokkr End-to-End Test Suite
-  `scenarios` module L19 — `-` — Run with: angreal tests e2e
-  `main` function L25-204 — `() -> ExitCode` — Run with: angreal tests e2e
-  `run_scenario` macro L58-75 — `-` — Run with: angreal tests e2e
-  `run_scenario_allow_fail` macro L82-102 — `-` — Run with: angreal tests e2e

#### tests/e2e/src/scenarios.rs

- pub `test_agent_management` function L133-184 — `(client: &Client) -> Result<()>` — Each scenario tests a complete user workflow through the system.
- pub `test_stack_deployment` function L190-230 — `(client: &Client) -> Result<()>` — Each scenario tests a complete user workflow through the system.
- pub `test_targeting` function L236-290 — `(client: &Client) -> Result<()>` — Each scenario tests a complete user workflow through the system.
- pub `test_templates` function L296-357 — `(client: &Client) -> Result<()>` — Each scenario tests a complete user workflow through the system.
- pub `test_work_orders` function L363-410 — `(client: &Client) -> Result<()>` — Each scenario tests a complete user workflow through the system.
- pub `test_build_work_orders` function L423-574 — `(client: &Client) -> Result<()>` — Test build work orders using Shipwright.
- pub `test_health_diagnostics` function L580-617 — `(client: &Client) -> Result<()>` — Each scenario tests a complete user workflow through the system.
- pub `test_webhooks` function L623-801 — `(client: &Client, webhook_catcher_url: Option<&str>) -> Result<()>` — Each scenario tests a complete user workflow through the system.
- pub `test_agent_reconciliation_existing_deployments` function L812-956 — `(client: &Client) -> Result<()>` — Test that agents can reconcile pre-existing deployments when targeted to a stack.
- pub `test_audit_logs` function L962-1014 — `(client: &Client) -> Result<()>` — Each scenario tests a complete user workflow through the system.
- pub `test_metrics` function L1020-1096 — `(client: &Client) -> Result<()>` — Each scenario tests a complete user workflow through the system.
- pub `test_ws_smoke` function L1116-1263 — `(client: &Client) -> Result<()>` — I-0019 / I-0020 A1 smoke test.
- pub `test_ws_chaos` function L1314-1473 — `(client: &Client) -> Result<()>` — I-0019 / I-0020 A2 chaos test — Pass 1 (infrastructure validation).
- pub `test_ws_workorders` function L1485-1621 — `(client: &Client) -> Result<()>` — Prove the full work-order lifecycle survives a WS outage: with the WS
- pub `test_ws_telemetry` function L1786-2037 — `(client: &Client) -> Result<()>` — I-0019 / I-0020 A3 telemetry-tailer test against real k3s.
-  `DEMO_DEPLOYMENT_YAML` variable L16-53 — `: &str` — Sample deployment YAML for testing
-  `MICROSERVICE_TEMPLATE` variable L56-76 — `: &str` — Microservice template for testing
-  `MICROSERVICE_SCHEMA` variable L78-88 — `: &str` — Each scenario tests a complete user workflow through the system.
-  `JOB_YAML` variable L91-105 — `: &str` — Job YAML for work order testing
-  `BUILD_YAML` variable L110-127 — `: &str` — Shipwright Build YAML for build work order testing
-  `toxiproxy_set_enabled` function L1274-1298 — `( toxiproxy_url: &str, proxy_name: &str, enabled: bool, ) -> Result<()>` — Toggle a toxiproxy proxy's `enabled` flag via the admin API.
-  `N` variable L1486 — `: usize` — Each scenario tests a complete user workflow through the system.
-  `k3s_apply` function L1631-1668 — `(compose_file: &str, manifest: &str) -> Result<()>` — Apply a Kubernetes manifest by piping it through `docker compose exec k3s
-  `dump_diagnostics` function L1672-1739 — `(compose_file: &str, pod_name: &str)` — On A3 Pass 2 failure, dump pod status + agent logs so the next iteration
-  `k3s_delete_best_effort` function L1743-1765 — `(compose_file: &str, args: &[&str])` — Run `kubectl delete` against the k3s cluster.

### tests/sdk-contract/python

> *Semantic summary to be generated by AI agent.*

#### tests/sdk-contract/python/conftest.py

- pub `make_client` function L63-65 — `def make_client(base_url: str, pak: str) -> AuthenticatedClient` — Build an AuthenticatedClient that sends `Authorization: <pak>` (no prefix).
- pub `unique` function L68-69 — `def unique(prefix: str) -> str`

#### tests/sdk-contract/python/test_manifest_apply.py

- pub `test_manifest_apply` function L20-73 — `def test_manifest_apply(admin_client, base_url, tmp_path)`

#### tests/sdk-contract/python/test_telemetry_and_ws.py

- pub `test_list_telemetry_events_returns_retention_metadata` function L39-49 — `def test_list_telemetry_events_returns_retention_metadata( admin_client: Authent...`
- pub `test_list_telemetry_logs_returns_retention_metadata` function L52-59 — `def test_list_telemetry_logs_returns_retention_metadata( admin_client: Authentic...`
- pub `test_list_ws_connections_returns_snapshot` function L62-70 — `def test_list_ws_connections_returns_snapshot( admin_client: AuthenticatedClient...`
- pub `test_live_subscription_url_helper_round_trips_through_format` function L73-83 — `def test_live_subscription_url_helper_round_trips_through_format( broker_url: st...`
-  `_seed_stack` function L23-36 — `def _seed_stack(admin_client: AuthenticatedClient, base_url: str)`

#### tests/sdk-contract/python/test_uat_walkthrough.py

- pub `test_uat_walkthrough` function L45-159 — `def test_uat_walkthrough(admin_client, base_url)` — Full UAT walkthrough: admin bootstrap + generator-driven flow.
- pub `test_target_generator_mismatch_returns_typed_403` function L162-218 — `def test_target_generator_mismatch_returns_typed_403(admin_client, base_url)` — Generator A cannot target a stack owned by generator B → typed 403.

### tests/sdk-contract/rust/src

> *Semantic summary to be generated by AI agent.*

#### tests/sdk-contract/rust/src/main.rs

-  `berr` function L34-36 — `(e: progenitor_client::Error<ErrorResponse>) -> BrokkrError` — Convert a progenitor `Error<ErrorResponse>` into our typed [`BrokkrError`].
-  `DEMO_YAML` variable L38-52 — `: &str` — Run with: `angreal tests sdk-contract rust`
-  `main` function L55-127 — `() -> ExitCode` — Run with: `angreal tests sdk-contract rust`
-  `run` macro L79-95 — `-` — Run with: `angreal tests sdk-contract rust`
-  `wait_for_ready` function L129-147 — `(broker_url: &str, timeout_secs: u64) -> Result<()>` — Run with: `angreal tests sdk-contract rust`
-  `client` function L150-155 — `(base_url: &str, pak: &str) -> Result<BrokkrClient>` — Build a [`BrokkrClient`] for a given PAK.
-  `unique` function L158-161 — `(prefix: &str) -> String` — Suffix used to keep names unique across reruns.
-  `scenario_uat_walkthrough` function L164-361 — `(base_url: &str, admin_pak: &str) -> Result<()>` — Full UAT walkthrough using a generator PAK after admin bootstrap.
-  `scenario_target_mismatch` function L365-468 — `(base_url: &str, admin_pak: &str) -> Result<()>` — A generator must not be able to target a stack it does not own — the
-  `scenario_raw_progenitor_surface` function L473-504 — `(base_url: &str, admin_pak: &str) -> Result<()>` — Smoke-check the raw progenitor [`brokkr_client::Client`] surface.
-  `scenario_telemetry_and_ws_diagnostics` function L511-596 — `(base_url: &str, admin_pak: &str) -> Result<()>` — WS-10 + WS-13 surface: ergonomic-wrapper methods for the telemetry
-  `last4` function L598-605 — `(s: &str) -> String` — Run with: `angreal tests sdk-contract rust`
-  `scenario_manifest_apply` function L610-695 — `(base_url: &str, admin_pak: &str) -> Result<()>` — BROKKR-T-0195: the manifest folder helpers — `submit_manifests` on an

### tests/sdk-contract/typescript/src

> *Semantic summary to be generated by AI agent.*

#### tests/sdk-contract/typescript/src/manifest-apply.test.ts

-  `unique` function L20-22 — `function unique(prefix: string): string`
-  `waitForBroker` function L24-35 — `function waitForBroker(timeoutMs = 30_000): Promise<void>`

#### tests/sdk-contract/typescript/src/telemetry-and-ws.test.ts

-  `waitForReady` function L35-47 — `function waitForReady(): Promise<void>`
-  `seedStack` function L53-77 — `function seedStack(): Promise<string>`

#### tests/sdk-contract/typescript/src/uat-walkthrough.test.ts

-  `unique` function L48-50 — `function unique(prefix: string): string`
-  `clientFor` function L56-61 — `function clientFor(pak: string): BrokkrApi`
-  `waitForBroker` function L63-76 — `function waitForBroker(timeoutMs = 30_000): Promise<void>`

### tools/ws-loadtest/src

> *Semantic summary to be generated by AI agent.*

#### tools/ws-loadtest/src/main.rs

-  `Config` struct L58-69 — `{ broker_url: String, admin_pak: String, agents: usize, stacks: usize, subscribe...` — LT_PG_CONTAINER     default brokkr-dev-postgres-1
-  `Config` type L71-93 — `= Config` — LT_PG_CONTAINER     default brokkr-dev-postgres-1
-  `from_env` function L72-92 — `() -> Self` — LT_PG_CONTAINER     default brokkr-dev-postgres-1
-  `Stats` struct L97-105 — `{ connected: AtomicU64, conn_errors: AtomicU64, sent: AtomicU64, send_errors: At...` — Shared counters across all synthetic clients.
-  `main` function L108-188 — `()` — LT_PG_CONTAINER     default brokkr-dev-postgres-1
-  `agent_loop` function L193-262 — `( url: String, pak: String, agent_id: String, idx: usize, stats: Arc<Stats>, sta...` — One synthetic agent: connect, then heartbeat every 5s + telemetry at the
-  `subscriber_loop` function L265-295 — `(url: String, pak: String, stats: Arc<Stats>, deadline: Instant)` — One live subscriber: drain frames until the deadline, counting receipts.
-  `Sample` struct L297-304 — `{ at: Instant, connected_gauge: Option<f64>, cpu_pct: Option<f64>, rss_mib: Opti...` — LT_PG_CONTAINER     default brokkr-dev-postgres-1
-  `sample_loop` function L306-343 — `( http: &reqwest::Client, cfg: &Config, stats: &Arc<Stats>, deadline: Instant, )...` — LT_PG_CONTAINER     default brokkr-dev-postgres-1
-  `print_summary` function L345-392 — `(cfg: &Config, stats: &Stats, samples: &[Sample])` — LT_PG_CONTAINER     default brokkr-dev-postgres-1
-  `create_generator` function L396-406 — `(http: &reqwest::Client, cfg: &Config, run: &str) -> String` — LT_PG_CONTAINER     default brokkr-dev-postgres-1
-  `create_stacks` function L408-430 — `(http: &reqwest::Client, cfg: &Config, generator_id: &str, run: &str) -> Vec<Str...` — LT_PG_CONTAINER     default brokkr-dev-postgres-1
-  `create_agents` function L432-462 — `(http: &reqwest::Client, cfg: &Config, run: &str) -> Vec<(String, String)>` — LT_PG_CONTAINER     default brokkr-dev-postgres-1
-  `scrape_gauge` function L466-488 — `(http: &reqwest::Client, cfg: &Config, name: &str) -> Option<f64>` — LT_PG_CONTAINER     default brokkr-dev-postgres-1
-  `docker_stats` function L491-517 — `(container: &str) -> (Option<f64>, Option<f64>)` — `docker stats --no-stream` for one container → (cpu%, rss MiB).
-  `parse_mem_mib` function L519-533 — `(s: &str) -> Option<f64>` — LT_PG_CONTAINER     default brokkr-dev-postgres-1
-  `pg_counts` function L536-561 — `(container: &str) -> (Option<u64>, Option<u64>)` — Two `select count(*)` via `docker exec ...
-  `auth_request` function L565-573 — `( url: &str, pak: &str, ) -> Option<tokio_tungstenite::tungstenite::handshake::c...` — LT_PG_CONTAINER     default brokkr-dev-postgres-1
-  `now_rfc3339` function L575-577 — `() -> String` — LT_PG_CONTAINER     default brokkr-dev-postgres-1
-  `heartbeat_json` function L579-585 — `(agent_id: &str) -> String` — LT_PG_CONTAINER     default brokkr-dev-postgres-1
-  `k8s_event_json` function L587-608 — `(agent_id: &str, stack_id: &str, idx: usize, tick: u64) -> String` — LT_PG_CONTAINER     default brokkr-dev-postgres-1
-  `pod_log_json` function L610-624 — `(agent_id: &str, stack_id: &str, idx: usize, tick: u64) -> String` — LT_PG_CONTAINER     default brokkr-dev-postgres-1
-  `ws_url` function L626-634 — `(broker_url: &str) -> String` — LT_PG_CONTAINER     default brokkr-dev-postgres-1

