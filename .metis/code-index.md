# Code Index

> Generated: 2026-03-10T13:18:26Z | 197 files | JavaScript, Python, Rust

## Project Structure

```
├── crates/
│   ├── brokkr-agent/
│   │   ├── src/
│   │   │   ├── bin.rs
│   │   │   ├── broker.rs
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
│   │   │   ├── lib.rs
│   │   │   ├── metrics.rs
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
│   │           ├── health.rs
│   │           ├── k8s/
│   │           │   ├── api.rs
│   │           │   ├── mod.rs
│   │           │   └── objects.rs
│   │           └── main.rs
│   ├── brokkr-broker/
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
│   │   │   │   ├── agent_labels.rs
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
│   │   │   └── utils/
│   │   │       ├── audit.rs
│   │   │       ├── background_tasks.rs
│   │   │       ├── config_watcher.rs
│   │   │       ├── encryption.rs
│   │   │       ├── event_bus.rs
│   │   │       ├── matching.rs
│   │   │       ├── mod.rs
│   │   │       ├── pak.rs
│   │   │       └── templating.rs
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
│   │           │   └── work_orders.rs
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
│   ├── brokkr-models/
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── models/
│   │       │   ├── agent_annotations.rs
│   │       │   ├── agent_events.rs
│   │       │   ├── agent_labels.rs
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
│   └── brokkr-utils/
│       ├── src/
│       │   ├── config.rs
│       │   ├── lib.rs
│       │   ├── logging.rs
│       │   └── telemetry.rs
│       └── tests/
│           └── integration.rs
├── docs/
│   └── themes/
│       └── hugo-geekdoc/
│           ├── eslint.config.js
│           └── static/
│               └── js/
│                   ├── 130-395cb664.chunk.min.js
│                   ├── 155-155e0581.chunk.min.js
│                   ├── 164-c7b61128.chunk.min.js
│                   ├── 165-4df74207.chunk.min.js
│                   ├── 174-5ff0286f.chunk.min.js
│                   ├── 178-3e4e928c.chunk.min.js
│                   ├── 186-df634c5c.chunk.min.js
│                   ├── 247-34fff2e1.chunk.min.js
│                   ├── 32-f6b664cc.chunk.min.js
│                   ├── 357-2a926bc9.chunk.min.js
│                   ├── 364-fd5df3dd.chunk.min.js
│                   ├── 379-233b54d3.chunk.min.js
│                   ├── 387-3546ecdc.chunk.min.js
│                   ├── 445-99c1ba44.chunk.min.js
│                   ├── 449-121db0c2.chunk.min.js
│                   ├── 452-e65d6d68.chunk.min.js
│                   ├── 484-77a146f6.chunk.min.js
│                   ├── 496-1979476f.chunk.min.js
│                   ├── 525-abc802a0.chunk.min.js
│                   ├── 567-4fef9a1a.chunk.min.js
│                   ├── 573-5fb26808.chunk.min.js
│                   ├── 606-72346440.chunk.min.js
│                   ├── 664-723fc55c.chunk.min.js
│                   ├── 689-3cbd5ea9.chunk.min.js
│                   ├── 711-c5eeef68.chunk.min.js
│                   ├── 720-970f726e.chunk.min.js
│                   ├── 723-47eb515a.chunk.min.js
│                   ├── 731-70ea2831.chunk.min.js
│                   ├── 763-66119f34.chunk.min.js
│                   ├── 790-2b300153.chunk.min.js
│                   ├── 802-4ae1987e.chunk.min.js
│                   ├── 840-6b7093bb.chunk.min.js
│                   ├── 875-6da97aae.chunk.min.js
│                   ├── 890-c9907c95.chunk.min.js
│                   ├── 921-8d080722.chunk.min.js
│                   ├── 998-ac49fa4c.chunk.min.js
│                   ├── colortheme-662de488.bundle.min.js
│                   ├── katex-81adfa46.bundle.min.js
│                   ├── main-2e274343.bundle.min.js
│                   ├── mermaid-16393d09.bundle.min.js
│                   └── search-d0afef64.bundle.min.js
├── examples/
│   ├── ui-slim/
│   │   └── src/
│   │       ├── App.js
│   │       ├── api.js
│   │       ├── components.js
│   │       └── index.js
│   └── webhook-catcher/
│       └── main.py
├── tests/
│   └── e2e/
│       └── src/
│           ├── api.rs
│           ├── main.rs
│           └── scenarios.rs
└── tools/
    └── webhook-catcher/
        └── app.py
```

## Modules

### crates/brokkr-agent/src

> *Semantic summary to be generated by AI agent.*

#### crates/brokkr-agent/src/bin.rs

-  `main` function L11-21 — `() -> Result<(), Box<dyn std::error::Error>>`

#### crates/brokkr-agent/src/broker.rs

- pub `wait_for_broker_ready` function L29-66 — `(config: &Settings)` — Waits for the broker service to become ready.
- pub `verify_agent_pak` function L76-114 — `(config: &Settings) -> Result<(), Box<dyn std::error::Error>>` — Verifies the agent's Personal Access Key (PAK) with the broker.
- pub `fetch_agent_details` function L124-178 — `( config: &Settings, client: &Client, ) -> Result<Agent, Box<dyn std::error::Err...` — Fetches the details of the agent from the broker.
- pub `fetch_and_process_deployment_objects` function L194-259 — `( config: &Settings, client: &Client, agent: &Agent, ) -> Result<Vec<DeploymentO...` — Fetches and processes deployment objects from the Kubernetes cluster
- pub `send_success_event` function L272-328 — `( config: &Settings, client: &Client, agent: &Agent, deployment_object_id: Uuid,...` — Sends a success event to the broker for the given deployment object.
- pub `send_failure_event` function L341-400 — `( config: &Settings, client: &Client, agent: &Agent, deployment_object_id: Uuid,...` — Sends a failure event to the broker for the given deployment object.
- pub `send_heartbeat` function L411-458 — `( config: &Settings, client: &Client, agent: &Agent, ) -> Result<(), Box<dyn std...` — Sends a heartbeat event to the broker for the given agent.
- pub `send_health_status` function L470-531 — `( config: &Settings, client: &Client, agent: &Agent, health_updates: Vec<Deploym...` — Sends health status updates for deployment objects to the broker.
- pub `fetch_pending_diagnostics` function L542-594 — `( config: &Settings, client: &Client, agent: &Agent, ) -> Result<Vec<DiagnosticR...` — Fetches pending diagnostic requests for the agent.
- pub `claim_diagnostic_request` function L605-654 — `( config: &Settings, client: &Client, request_id: Uuid, ) -> Result<DiagnosticRe...` — Claims a diagnostic request for processing.
- pub `submit_diagnostic_result` function L666-714 — `( config: &Settings, client: &Client, request_id: Uuid, result: SubmitDiagnostic...` — Submits diagnostic results for a request.

#### crates/brokkr-agent/src/deployment_health.rs

- pub `DeploymentHealthStatus` struct L50-59 — `{ id: Uuid, status: String, summary: HealthSummary, checked_at: DateTime<Utc> }` — Health status for a deployment object
- pub `HealthSummary` struct L63-72 — `{ pods_ready: usize, pods_total: usize, conditions: Vec<String>, resources: Vec<...` — Summary of health information for a deployment
- pub `ResourceHealth` struct L76-87 — `{ kind: String, name: String, namespace: String, ready: bool, message: Option<St...` — Health status of an individual resource
- pub `HealthChecker` struct L90-92 — `{ k8s_client: Client }` — Checks deployment health for Kubernetes resources
- pub `new` function L96-98 — `(k8s_client: Client) -> Self` — Creates a new HealthChecker instance
- pub `check_deployment_object` function L104-232 — `( &self, deployment_object_id: Uuid, ) -> Result<DeploymentHealthStatus, Box<dyn...` — Checks the health of a specific deployment object by ID
- pub `check_deployment_objects` function L250-273 — `( &self, deployment_object_ids: &[Uuid], ) -> Vec<DeploymentHealthStatus>` — Checks health for multiple deployment objects
- pub `HealthStatusUpdate` struct L291-294 — `{ deployment_objects: Vec<DeploymentObjectHealthUpdate> }` — Request body for sending health status updates to the broker
- pub `DeploymentObjectHealthUpdate` struct L298-307 — `{ id: Uuid, status: String, summary: Option<HealthSummary>, checked_at: DateTime...` — Health update for a single deployment object (matches broker API)
-  `DEGRADED_CONDITIONS` variable L22-30 — `: &[&str]` — Known problematic waiting conditions that indicate degraded health
-  `PENDING_CONDITIONS` variable L35-39 — `: &[&str]` — Conditions that indicate pending state (not yet problematic but not ready)
-  `TERMINATED_ISSUES` variable L42-46 — `: &[&str]` — Reasons from terminated containers that indicate issues
-  `HealthChecker` type L94-274 — `= HealthChecker` — OOMKilled, and other problematic conditions.
-  `find_pods_for_deployment` function L235-247 — `( &self, deployment_object_id: Uuid, ) -> Result<Vec<Pod>, Box<dyn std::error::E...` — Finds all pods labeled with the given deployment object ID
-  `is_pod_ready` function L277-287 — `(pod: &Pod) -> bool` — Checks if a pod is in ready state
-  `DeploymentObjectHealthUpdate` type L309-318 — `= DeploymentObjectHealthUpdate` — OOMKilled, and other problematic conditions.
-  `from` function L310-317 — `(status: DeploymentHealthStatus) -> Self` — OOMKilled, and other problematic conditions.
-  `tests` module L321-390 — `-` — OOMKilled, and other problematic conditions.
-  `test_degraded_conditions_are_detected` function L325-331 — `()` — OOMKilled, and other problematic conditions.
-  `test_terminated_issues_include_oomkilled` function L334-337 — `()` — OOMKilled, and other problematic conditions.
-  `test_health_summary_default` function L340-346 — `()` — OOMKilled, and other problematic conditions.
-  `test_deployment_health_status_serialization` function L349-368 — `()` — OOMKilled, and other problematic conditions.
-  `test_health_update_conversion` function L371-389 — `()` — OOMKilled, and other problematic conditions.

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
-  `MAX_LOG_LINES` variable L24 — `: i64` — Maximum number of log lines to collect per container.
-  `DiagnosticsHandler` type L132-389 — `= DiagnosticsHandler` — about Kubernetes resources, including pod statuses, events, and log tails.
-  `collect_pod_statuses` function L174-279 — `( &self, namespace: &str, label_selector: &str, ) -> Result<Vec<PodStatus>, Box<...` — Collects pod statuses for matching pods.
-  `collect_events` function L282-321 — `( &self, namespace: &str, _label_selector: &str, ) -> Result<Vec<EventInfo>, Box...` — Collects events for matching resources.
-  `collect_log_tails` function L324-365 — `( &self, namespace: &str, label_selector: &str, ) -> Result<HashMap<String, Stri...` — Collects log tails for matching pods.
-  `get_container_logs` function L368-388 — `( &self, namespace: &str, pod_name: &str, container_name: &str, ) -> Result<Stri...` — Gets logs for a specific container.
-  `tests` module L392-452 — `-` — about Kubernetes resources, including pod statuses, events, and log tails.
-  `test_pod_status_serialization` function L396-420 — `()` — about Kubernetes resources, including pod statuses, events, and log tails.
-  `test_event_info_serialization` function L423-437 — `()` — about Kubernetes resources, including pod statuses, events, and log tails.
-  `test_submit_diagnostic_result_serialization` function L440-451 — `()` — about Kubernetes resources, including pod statuses, events, and log tails.

#### crates/brokkr-agent/src/health.rs

- pub `HealthState` struct L39-43 — `{ k8s_client: Client, broker_status: Arc<RwLock<BrokerStatus>>, start_time: Syst...` — Shared state for health endpoints
- pub `BrokerStatus` struct L47-50 — `{ connected: bool, last_heartbeat: Option<String> }` — Broker connection status
- pub `configure_health_routes` function L80-87 — `(state: HealthState) -> Router` — Configures and returns the health check router
-  `HealthStatus` struct L54-61 — `{ status: String, kubernetes: KubernetesStatus, broker: BrokerStatusResponse, up...` — Health status response structure
-  `KubernetesStatus` struct L65-69 — `{ connected: bool, error: Option<String> }` — Kubernetes health status
-  `BrokerStatusResponse` struct L73-77 — `{ connected: bool, last_heartbeat: Option<String> }` — Broker health status for response
-  `healthz` function L93-95 — `() -> impl IntoResponse` — Simple liveness check endpoint
-  `readyz` function L101-113 — `(State(state): State<HealthState>) -> impl IntoResponse` — Readiness check endpoint
-  `health` function L125-183 — `(State(state): State<HealthState>) -> impl IntoResponse` — Detailed health check endpoint
-  `metrics_handler` function L189-196 — `() -> impl IntoResponse` — Prometheus metrics endpoint

#### crates/brokkr-agent/src/lib.rs

- pub `broker` module L15 — `-` — # Brokkr Agent
- pub `cli` module L16 — `-` — see the [Brokkr Documentation](https://brokkr.io/explanation/architecture).
- pub `deployment_health` module L17 — `-` — see the [Brokkr Documentation](https://brokkr.io/explanation/architecture).
- pub `diagnostics` module L18 — `-` — see the [Brokkr Documentation](https://brokkr.io/explanation/architecture).
- pub `health` module L19 — `-` — see the [Brokkr Documentation](https://brokkr.io/explanation/architecture).
- pub `k8s` module L20 — `-` — see the [Brokkr Documentation](https://brokkr.io/explanation/architecture).
- pub `metrics` module L21 — `-` — see the [Brokkr Documentation](https://brokkr.io/explanation/architecture).
- pub `utils` module L22 — `-` — see the [Brokkr Documentation](https://brokkr.io/explanation/architecture).
- pub `webhooks` module L23 — `-` — see the [Brokkr Documentation](https://brokkr.io/explanation/architecture).
- pub `work_orders` module L24 — `-` — see the [Brokkr Documentation](https://brokkr.io/explanation/architecture).

#### crates/brokkr-agent/src/metrics.rs

- pub `poll_requests_total` function L27-41 — `() -> &'static CounterVec` — Broker poll request counter
- pub `poll_duration_seconds` function L44-59 — `() -> &'static HistogramVec` — Broker poll duration histogram
- pub `kubernetes_operations_total` function L63-77 — `() -> &'static CounterVec` — Kubernetes operations counter
- pub `kubernetes_operation_duration_seconds` function L81-96 — `() -> &'static HistogramVec` — Kubernetes operation duration histogram
- pub `heartbeat_sent_total` function L99-112 — `() -> &'static IntCounter` — Heartbeat sent counter
- pub `last_successful_poll_timestamp` function L115-128 — `() -> &'static Gauge` — Last successful poll timestamp (Unix timestamp)
- pub `encode_metrics` function L135-143 — `() -> String` — Encodes all registered metrics in Prometheus text format
-  `REGISTRY` variable L19 — `: OnceLock<Registry>` — Global Prometheus registry for all agent metrics
-  `registry` function L21-23 — `() -> &'static Registry` — It exposes metrics about broker polling, Kubernetes operations, and agent health.
-  `COUNTER` variable L28 — `: OnceLock<CounterVec>` — It exposes metrics about broker polling, Kubernetes operations, and agent health.
-  `HISTOGRAM` variable L45 — `: OnceLock<HistogramVec>` — It exposes metrics about broker polling, Kubernetes operations, and agent health.
-  `COUNTER` variable L64 — `: OnceLock<CounterVec>` — It exposes metrics about broker polling, Kubernetes operations, and agent health.
-  `HISTOGRAM` variable L82 — `: OnceLock<HistogramVec>` — It exposes metrics about broker polling, Kubernetes operations, and agent health.
-  `COUNTER` variable L100 — `: OnceLock<IntCounter>` — It exposes metrics about broker polling, Kubernetes operations, and agent health.
-  `GAUGE` variable L116 — `: OnceLock<Gauge>` — It exposes metrics about broker polling, Kubernetes operations, and agent health.

#### crates/brokkr-agent/src/utils.rs

- pub `multidoc_deserialize` function L18-24 — `(multi_doc_str: &str) -> Result<Vec<serde_yaml::Value>, Box<dyn Error>>` — Deserializes a multi-document YAML string into a vector of YAML values.
-  `tests` module L27-66 — `-`
-  `test_multidoc_deserialize_success` function L31-50 — `()`
-  `test_multidoc_deserialize_failure` function L53-65 — `()`

#### crates/brokkr-agent/src/webhooks.rs

- pub `PendingWebhookDelivery` struct L27-46 — `{ id: Uuid, subscription_id: Uuid, event_type: String, payload: String, url: Str...` — Pending webhook delivery from the broker.
- pub `DeliveryResultRequest` struct L50-62 — `{ success: bool, status_code: Option<i32>, error: Option<String>, duration_ms: O...` — Request body for reporting delivery result to broker.
- pub `DeliveryResult` struct L66-75 — `{ success: bool, status_code: Option<i32>, error: Option<String>, duration_ms: i...` — Result of a webhook delivery attempt.
- pub `fetch_pending_webhooks` function L90-142 — `( config: &Settings, client: &Client, agent: &Agent, ) -> Result<Vec<PendingWebh...` — Fetches pending webhook deliveries for this agent from the broker.
- pub `report_delivery_result` function L154-203 — `( config: &Settings, client: &Client, delivery_id: Uuid, result: &DeliveryResult...` — Reports the result of a webhook delivery attempt to the broker.
- pub `deliver_webhook` function L216-303 — `(delivery: &PendingWebhookDelivery) -> DeliveryResult` — Delivers a webhook via HTTP POST.
- pub `process_pending_webhooks` function L336-391 — `( config: &Settings, client: &Client, agent: &Agent, ) -> Result<usize, Box<dyn ...` — Processes all pending webhook deliveries for this agent.
-  `classify_error` function L306-316 — `(error: &reqwest::Error) -> String` — Classifies request errors for logging and retry decisions.
-  `tests` module L394-468 — `-` — assigned to them, deliver them via HTTP, and report results back to the broker.
-  `test_delivery_result_request_serialization` function L398-412 — `()` — assigned to them, deliver them via HTTP, and report results back to the broker.
-  `test_delivery_result_request_with_error` function L415-426 — `()` — assigned to them, deliver them via HTTP, and report results back to the broker.
-  `test_pending_webhook_delivery_deserialization` function L429-448 — `()` — assigned to them, deliver them via HTTP, and report results back to the broker.
-  `test_pending_webhook_delivery_without_auth` function L451-467 — `()` — assigned to them, deliver them via HTTP, and report results back to the broker.

### crates/brokkr-agent/src/cli

> *Semantic summary to be generated by AI agent.*

#### crates/brokkr-agent/src/cli/commands.rs

- pub `start` function L75-441 — `() -> Result<(), Box<dyn std::error::Error>>` — - Contextual information

#### crates/brokkr-agent/src/cli/mod.rs

- pub `commands` module L8 — `-` — Command-line interface module for the Brokkr agent.
- pub `Cli` struct L14-18 — `{ command: Commands }` — CLI configuration structure.
- pub `Commands` enum L22-25 — `Start` — Available CLI commands.
- pub `parse_cli` function L31-33 — `() -> Cli` — Parses command-line arguments into the Cli structure.

### crates/brokkr-agent/src/k8s

> *Semantic summary to be generated by AI agent.*

#### crates/brokkr-agent/src/k8s/api.rs

- pub `apply_k8s_objects` function L148-253 — `( k8s_objects: &[DynamicObject], k8s_client: K8sClient, patch_params: PatchParam...` — Applies a list of Kubernetes objects to the cluster using server-side apply.
- pub `dynamic_api` function L266-280 — `( ar: ApiResource, caps: ApiCapabilities, client: K8sClient, namespace: Option<&...` — Creates a dynamic Kubernetes API client for a specific resource type
- pub `get_all_objects_by_annotation` function L292-342 — `( k8s_client: &K8sClient, annotation_key: &str, annotation_value: &str, ) -> Res...` — Retrieves all Kubernetes objects with a specific annotation key-value pair.
- pub `delete_k8s_objects` function L353-443 — `( k8s_objects: &[DynamicObject], k8s_client: K8sClient, agent_id: &Uuid, ) -> Re...` — Deletes a list of Kubernetes objects from the cluster.
- pub `validate_k8s_objects` function L453-550 — `( k8s_objects: &[DynamicObject], k8s_client: K8sClient, ) -> Result<(), Box<dyn ...` — Validates Kubernetes objects against the API server without applying them.
- pub `reconcile_target_state` function L675-877 — `( objects: &[DynamicObject], client: Client, stack_id: &str, checksum: &str, ) -...` — Reconciles the target state of Kubernetes objects for a stack.
- pub `create_k8s_client` function L886-916 — `( kubeconfig_path: Option<&str>, ) -> Result<K8sClient, Box<dyn std::error::Erro...` — Creates a Kubernetes client using either a provided kubeconfig path or default configuration.
-  `RetryConfig` struct L67-72 — `{ max_elapsed_time: Duration, initial_interval: Duration, max_interval: Duration...` — Retry configuration for Kubernetes operations
-  `RetryConfig` type L74-83 — `impl Default for RetryConfig` — 3.
-  `default` function L75-82 — `() -> Self` — 3.
-  `is_retryable_error` function L86-97 — `(error: &KubeError) -> bool` — Determines if a Kubernetes error is retryable
-  `with_retries` function L100-136 — `( operation: F, config: RetryConfig, ) -> Result<T, Box<dyn std::error::Error>>` — Executes a Kubernetes operation with retries
-  `apply_single_object` function L559-622 — `( object: &DynamicObject, client: &Client, stack_id: &str, checksum: &str, ) -> ...` — Applies a single Kubernetes object with proper annotations.
-  `rollback_namespaces` function L629-658 — `(client: &Client, namespaces: &[String])` — Rolls back namespaces that were created during a failed reconciliation.

#### crates/brokkr-agent/src/k8s/mod.rs

- pub `api` module L7 — `-`
- pub `objects` module L8 — `-`

#### crates/brokkr-agent/src/k8s/objects.rs

- pub `STACK_LABEL` variable L44 — `: &str` — Label key for identifying stack resources
- pub `CHECKSUM_ANNOTATION` variable L47 — `: &str` — Annotation key for deployment checksums
- pub `LAST_CONFIG_ANNOTATION` variable L50 — `: &str` — Annotation key for last applied configuration
- pub `DEPLOYMENT_OBJECT_ID_LABEL` variable L53 — `: &str` — Label key for deployment object IDs
- pub `BROKKR_AGENT_OWNER_ANNOTATION` variable L56 — `: &str` — Key for agent ownership
- pub `create_k8s_objects` function L65-117 — `( deployment_object: DeploymentObject, agent_id: Uuid, ) -> Result<Vec<DynamicOb...` — Creates Kubernetes objects from a brokkr deployment object's YAML content.
- pub `verify_object_ownership` function L120-128 — `(object: &DynamicObject, agent_id: &Uuid) -> bool` — - Object validation
-  `tests` module L131-460 — `-` — - Object validation
-  `create_test_object` function L144-156 — `(annotations: Option<BTreeMap<String, String>>) -> DynamicObject` — - Object validation
-  `test_create_k8s_objects_single_document` function L159-195 — `()` — - Object validation
-  `test_create_k8s_objects_multiple_documents` function L198-252 — `()` — - Object validation
-  `test_create_k8s_objects_with_crds` function L255-302 — `()` — - Object validation
-  `test_create_k8s_objects_invalid_yaml` function L305-330 — `()` — - Object validation
-  `test_create_k8s_objects_empty_yaml` function L333-350 — `()` — - Object validation
-  `test_create_k8s_objects_ordering` function L353-404 — `()` — - Object validation
-  `test_verify_object_ownership_matching_owner` function L407-418 — `()` — - Object validation
-  `test_verify_object_ownership_different_owner` function L421-432 — `()` — - Object validation
-  `test_verify_object_ownership_no_annotations` function L435-439 — `()` — - Object validation
-  `test_verify_object_ownership_empty_annotations` function L442-446 — `()` — - Object validation
-  `test_verify_object_ownership_invalid_uuid` function L449-459 — `()` — - Object validation

### crates/brokkr-agent/src/work_orders

> *Semantic summary to be generated by AI agent.*

#### crates/brokkr-agent/src/work_orders/broker.rs

- pub `fetch_pending_work_orders` function L54-113 — `( config: &Settings, client: &Client, agent: &Agent, work_type: Option<&str>, ) ...` — Fetches pending work orders for the agent from the broker.
- pub `claim_work_order` function L125-196 — `( config: &Settings, client: &Client, agent: &Agent, work_order_id: Uuid, ) -> R...` — Claims a work order for the agent.
- pub `complete_work_order` function L210-285 — `( config: &Settings, client: &Client, work_order_id: Uuid, success: bool, messag...` — Reports work order completion to the broker.
-  `ClaimRequest` struct L24-26 — `{ agent_id: Uuid }` — Request body for claiming a work order.
-  `CompleteRequest` struct L30-36 — `{ success: bool, message: Option<String>, retryable: bool }` — Request body for completing a work order.
-  `RetryResponse` struct L40-42 — `{ status: String }` — Response for retry scheduling.

#### crates/brokkr-agent/src/work_orders/build.rs

- pub `execute_build` function L103-180 — `( k8s_client: &K8sClient, yaml_content: &str, work_order_id: &str, ) -> Result<O...` — Executes a build using Shipwright.
-  `SHIPWRIGHT_API_GROUP` variable L34 — `: &str` — Shipwright API group
-  `SHIPWRIGHT_API_VERSION` variable L35 — `: &str` — - **ClusterBuildStrategy**: Pre-installed strategy (e.g., buildah)
-  `CONDITION_SUCCEEDED` variable L38 — `: &str` — BuildRun status conditions
-  `BUILD_TIMEOUT_SECS` variable L41 — `: u64` — Maximum time to wait for a build to complete (15 minutes)
-  `STATUS_POLL_INTERVAL_SECS` variable L44 — `: u64` — Polling interval for build status checks
-  `BuildRunStatus` struct L49-56 — `{ conditions: Vec<Condition>, output: Option<BuildRunOutput>, failure_details: O...` — BuildRun status for watching completion
-  `Condition` struct L60-68 — `{ condition_type: String, status: String, reason: Option<String>, message: Optio...` — - **ClusterBuildStrategy**: Pre-installed strategy (e.g., buildah)
-  `BuildRunOutput` struct L73-76 — `{ digest: Option<String>, size: Option<i64> }` — - **ClusterBuildStrategy**: Pre-installed strategy (e.g., buildah)
-  `FailureDetails` struct L80-85 — `{ reason: Option<String>, message: Option<String> }` — - **ClusterBuildStrategy**: Pre-installed strategy (e.g., buildah)
-  `apply_shipwright_resource` function L183-193 — `( k8s_client: &K8sClient, resource: &serde_yaml::Value, ) -> Result<(), Box<dyn ...` — Applies a Shipwright resource (Build) to the cluster using the core k8s apply logic.
-  `create_buildrun` function L196-238 — `( k8s_client: &K8sClient, name: &str, build_name: &str, namespace: &str, work_or...` — Creates a BuildRun resource.
-  `watch_buildrun_completion` function L241-328 — `( k8s_client: &K8sClient, name: &str, namespace: &str, ) -> Result<Option<String...` — Watches a BuildRun until it completes (success or failure).
-  `ParsedBuildInfo` struct L332-336 — `{ build_name: String, build_namespace: String, build_docs: Vec<serde_yaml::Value...` — Result of parsing build YAML content
-  `parse_build_yaml` function L350-401 — `(yaml_content: &str) -> Result<ParsedBuildInfo, Box<dyn std::error::Error>>` — Parses YAML content to extract Build resource information.
-  `interpret_buildrun_status` function L409-444 — `(status: &BuildRunStatus) -> Result<Option<String>, String>` — Interprets a BuildRun status to determine completion state.
-  `tests` module L447-835 — `-` — - **ClusterBuildStrategy**: Pre-installed strategy (e.g., buildah)
-  `test_parse_build_yaml_with_build_resource` function L453-475 — `()` — - **ClusterBuildStrategy**: Pre-installed strategy (e.g., buildah)
-  `test_parse_build_yaml_default_namespace` function L478-493 — `()` — - **ClusterBuildStrategy**: Pre-installed strategy (e.g., buildah)
-  `test_parse_build_yaml_with_work_order_buildref` function L496-511 — `()` — - **ClusterBuildStrategy**: Pre-installed strategy (e.g., buildah)
-  `test_parse_build_yaml_build_takes_precedence` function L514-540 — `()` — - **ClusterBuildStrategy**: Pre-installed strategy (e.g., buildah)
-  `test_parse_build_yaml_empty_content` function L543-549 — `()` — - **ClusterBuildStrategy**: Pre-installed strategy (e.g., buildah)
-  `test_parse_build_yaml_no_build_resource` function L552-565 — `()` — - **ClusterBuildStrategy**: Pre-installed strategy (e.g., buildah)
-  `test_parse_build_yaml_invalid_yaml` function L568-572 — `()` — - **ClusterBuildStrategy**: Pre-installed strategy (e.g., buildah)
-  `test_parse_build_yaml_multiple_builds` function L575-594 — `()` — - **ClusterBuildStrategy**: Pre-installed strategy (e.g., buildah)
-  `test_buildrun_status_deserialization_success` function L599-619 — `()` — - **ClusterBuildStrategy**: Pre-installed strategy (e.g., buildah)
-  `test_buildrun_status_deserialization_failure` function L622-641 — `()` — - **ClusterBuildStrategy**: Pre-installed strategy (e.g., buildah)
-  `test_buildrun_status_deserialization_in_progress` function L644-658 — `()` — - **ClusterBuildStrategy**: Pre-installed strategy (e.g., buildah)
-  `test_buildrun_status_deserialization_empty_conditions` function L661-667 — `()` — - **ClusterBuildStrategy**: Pre-installed strategy (e.g., buildah)
-  `test_interpret_status_succeeded_with_digest` function L672-690 — `()` — - **ClusterBuildStrategy**: Pre-installed strategy (e.g., buildah)
-  `test_interpret_status_succeeded_no_digest` function L693-708 — `()` — - **ClusterBuildStrategy**: Pre-installed strategy (e.g., buildah)
-  `test_interpret_status_failed_with_details` function L711-731 — `()` — - **ClusterBuildStrategy**: Pre-installed strategy (e.g., buildah)
-  `test_interpret_status_failed_no_details` function L734-749 — `()` — - **ClusterBuildStrategy**: Pre-installed strategy (e.g., buildah)
-  `test_interpret_status_failed_fallback_message` function L752-767 — `()` — - **ClusterBuildStrategy**: Pre-installed strategy (e.g., buildah)
-  `test_interpret_status_in_progress` function L770-785 — `()` — - **ClusterBuildStrategy**: Pre-installed strategy (e.g., buildah)
-  `test_interpret_status_no_succeeded_condition` function L788-803 — `()` — - **ClusterBuildStrategy**: Pre-installed strategy (e.g., buildah)
-  `test_interpret_status_empty_conditions` function L806-816 — `()` — - **ClusterBuildStrategy**: Pre-installed strategy (e.g., buildah)
-  `test_buildrun_name_generation_short_id` function L821-826 — `()` — - **ClusterBuildStrategy**: Pre-installed strategy (e.g., buildah)
-  `test_buildrun_name_generation_long_id` function L829-834 — `()` — - **ClusterBuildStrategy**: Pre-installed strategy (e.g., buildah)

#### crates/brokkr-agent/src/work_orders/mod.rs

- pub `broker` module L26 — `-` — # Work Orders Module
- pub `build` module L27 — `-` — ```
- pub `process_pending_work_orders` function L113-157 — `( config: &Settings, http_client: &Client, k8s_client: &K8sClient, agent: &Agent...` — Processes pending work orders for the agent.
-  `is_error_retryable` function L50-95 — `(error: &dyn std::error::Error) -> bool` — Determines if an error is retryable by inspecting the error message.
-  `process_single_work_order` function L160-224 — `( config: &Settings, http_client: &Client, k8s_client: &K8sClient, agent: &Agent...` — Processes a single work order through its complete lifecycle.
-  `execute_build_work_order` function L227-257 — `( _config: &Settings, _http_client: &Client, k8s_client: &K8sClient, agent: &Age...` — Executes a build work order using Shipwright.
-  `execute_custom_work_order` function L260-316 — `( k8s_client: &K8sClient, agent: &Agent, work_order: &WorkOrder, ) -> Result<Opt...` — Executes a custom work order by applying YAML resources to the cluster.

### crates/brokkr-agent/tests

> *Semantic summary to be generated by AI agent.*

#### crates/brokkr-agent/tests/fixtures.rs

- pub `get_or_init_fixture` function L33-37 — `() -> Arc<Mutex<TestFixture>>` — Gets or initializes a test fixture singleton
- pub `TestFixture` struct L40-49 — `{ admin_settings: Settings, client: Client, agent_settings: Settings, initialize...`
- pub `new` function L53-71 — `() -> Self` — Creates a new TestFixture instance with default values
- pub `initialize` function L77-128 — `(&mut self)` — Initializes the test fixture by setting up necessary resources
- pub `wait_for_broker` function L134-136 — `(&self)` — Waits for the broker to become available
- pub `create_generator` function L146-188 — `(&mut self, name: String, description: Option<String>)` — Creates a new generator resource
- pub `create_stack` function L197-255 — `(&mut self, stack_name: &str)` — Creates a new stack resource
- pub `create_deployment` function L267-304 — `(&self, yaml_content: String) -> DeploymentObject` — Creates a new deployment from YAML content
-  `INIT` variable L14 — `: Once`
-  `FIXTURE` variable L24 — `: OnceCell<Arc<Mutex<TestFixture>>>`
-  `TestFixture` type L51-305 — `= TestFixture`

### crates/brokkr-agent/tests/integration

> *Semantic summary to be generated by AI agent.*

#### crates/brokkr-agent/tests/integration/broker.rs

-  `TEST_NAMESPACE_YAML` variable L11-19 — `: &str`
-  `test_wait_for_broker` function L22-31 — `()`
-  `test_verify_agent_pak` function L34-46 — `()`
-  `test_fetch_agent_details` function L49-74 — `()`
-  `test_fetch_and_process_deployment_objects` function L77-103 — `()`
-  `test_successful_event_apply` function L106-155 — `()`
-  `test_failure_event_apply` function L158-212 — `()`
-  `test_send_heartbeat` function L215-251 — `()`

#### crates/brokkr-agent/tests/integration/health.rs

-  `create_test_health_state` function L18-34 — `() -> HealthState`
-  `test_healthz_endpoint` function L37-58 — `()`
-  `test_readyz_endpoint` function L61-81 — `()`
-  `test_health_endpoint` function L84-115 — `()`
-  `test_metrics_endpoint` function L118-142 — `()`

#### crates/brokkr-agent/tests/integration/main.rs

-  `broker` module L7 — `-`
-  `fixtures` module L9 — `-`
-  `health` module L10 — `-`
-  `k8s` module L11 — `-`

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
-  `test_reconcile_invalid_object_rollback` function L305-406 — `()`
-  `test_reconcile_object_pruning` function L409-526 — `()`
-  `test_reconcile_empty_object_list` function L529-624 — `()`
-  `test_k8s_setup_and_cleanup` function L627-681 — `()`
-  `test_create_k8s_client_with_kubeconfig` function L684-697 — `()`
-  `test_create_k8s_client_with_invalid_path` function L700-706 — `()`
-  `test_create_k8s_client_default` function L709-715 — `()`
-  `test_apply_k8s_objects` function L718-789 — `()`
-  `test_validate_k8s_objects_valid` function L792-826 — `()`
-  `test_validate_k8s_objects_invalid` function L829-890 — `()`
-  `test_get_objects_by_annotation_found` function L893-951 — `()`
-  `test_get_objects_by_annotation_not_found` function L954-1000 — `()`
-  `test_delete_k8s_object_success` function L1003-1072 — `()`
-  `test_delete_k8s_object_not_found` function L1075-1115 — `()`
-  `test_reconcile_namespace_in_same_deployment` function L1118-1190 — `()`
-  `test_reconcile_namespace_rollback_on_failure` function L1193-1265 — `()`

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

### crates/brokkr-broker/src/api

> *Semantic summary to be generated by AI agent.*

#### crates/brokkr-broker/src/api/mod.rs

- pub `v1` module L157 — `-` — # API Module
- pub `configure_api_routes` function L189-228 — `( dal: DAL, cors_config: &Cors, reloadable_config: Option<ReloadableConfig>, ) -...` — Configures and returns the main application router with all API routes
-  `healthz` function L238-240 — `() -> impl IntoResponse` — Health check endpoint handler
-  `readyz` function L250-252 — `() -> impl IntoResponse` — Ready check endpoint handler
-  `metrics_handler` function L262-269 — `() -> impl IntoResponse` — Metrics endpoint handler
-  `metrics_middleware` function L274-290 — `(request: Request<Body>, next: Next) -> Response` — Middleware to record HTTP request metrics

### crates/brokkr-broker/src/api/v1

> *Semantic summary to be generated by AI agent.*

#### crates/brokkr-broker/src/api/v1/admin.rs

- pub `ConfigReloadResponse` struct L31-41 — `{ reloaded_at: DateTime<Utc>, changes: Vec<ConfigChangeInfo>, success: bool, mes...` — Response structure for configuration reload operations.
- pub `ConfigChangeInfo` struct L45-52 — `{ key: String, old_value: String, new_value: String }` — Information about a single configuration change.
- pub `AuditLogQueryParams` struct L56-80 — `{ actor_type: Option<String>, actor_id: Option<Uuid>, action: Option<String>, re...` — Query parameters for listing audit logs.
- pub `AuditLogListResponse` struct L98-109 — `{ logs: Vec<AuditLog>, total: i64, count: usize, limit: i64, offset: i64 }` — Response structure for audit log list operations.
- pub `routes` function L114-119 — `() -> Router<DAL>` — Constructs and returns the admin routes.
-  `AuditLogFilter` type L82-94 — `= AuditLogFilter` — including configuration hot-reload functionality.
-  `from` function L83-93 — `(params: AuditLogQueryParams) -> Self` — including configuration hot-reload functionality.
-  `reload_config` function L151-202 — `( Extension(auth): Extension<AuthPayload>, Extension(config): Extension<Reloadab...` — including configuration hot-reload functionality.
-  `list_audit_logs` function L246-297 — `( State(dal): State<DAL>, Extension(auth): Extension<AuthPayload>, Query(params)...` — including configuration hot-reload functionality.
-  `tests` module L300-333 — `-` — including configuration hot-reload functionality.
-  `test_config_reload_response_serialization` function L304-320 — `()` — including configuration hot-reload functionality.
-  `test_config_change_info_serialization` function L323-332 — `()` — including configuration hot-reload functionality.

#### crates/brokkr-broker/src/api/v1/agent_events.rs

- pub `routes` function L23-27 — `() -> Router<DAL>` — Creates and returns a router for agent event-related endpoints.
-  `list_agent_events` function L46-64 — `( State(dal): State<DAL>, Extension(_auth_payload): Extension<crate::api::v1::mi...` — Retrieves a list of all agent events.
-  `get_agent_event` function L88-114 — `( State(dal): State<DAL>, Extension(_auth_payload): Extension<crate::api::v1::mi...` — Retrieves a specific agent event by its ID.

#### crates/brokkr-broker/src/api/v1/agents.rs

- pub `routes` function L42-65 — `() -> Router<DAL>` — Creates and returns the router for agent-related endpoints.
-  `list_agents` function L84-123 — `( State(dal): State<DAL>, Extension(auth_payload): Extension<AuthPayload>, ) -> ...` — event logging, label management, annotation management, target management, and heartbeat recording.
-  `create_agent` function L143-211 — `( State(dal): State<DAL>, Extension(auth_payload): Extension<AuthPayload>, Json(...` — event logging, label management, annotation management, target management, and heartbeat recording.
-  `AgentQuery` struct L214-217 — `{ name: Option<String>, cluster_name: Option<String> }` — event logging, label management, annotation management, target management, and heartbeat recording.
-  `get_agent` function L241-275 — `( State(dal): State<DAL>, Extension(auth_payload): Extension<AuthPayload>, Path(...` — event logging, label management, annotation management, target management, and heartbeat recording.
-  `search_agent` function L300-349 — `( State(dal): State<DAL>, Extension(auth_payload): Extension<AuthPayload>, Query...` — event logging, label management, annotation management, target management, and heartbeat recording.
-  `update_agent` function L374-447 — `( State(dal): State<DAL>, Extension(auth_payload): Extension<AuthPayload>, Path(...` — event logging, label management, annotation management, target management, and heartbeat recording.
-  `delete_agent` function L469-509 — `( State(dal): State<DAL>, Extension(auth_payload): Extension<AuthPayload>, Path(...` — event logging, label management, annotation management, target management, and heartbeat recording.
-  `list_events` function L532-571 — `( State(dal): State<DAL>, Extension(auth_payload): Extension<AuthPayload>, Path(...` — event logging, label management, annotation management, target management, and heartbeat recording.
-  `create_event` function L595-645 — `( State(dal): State<DAL>, Extension(auth_payload): Extension<AuthPayload>, Path(...` — event logging, label management, annotation management, target management, and heartbeat recording.
-  `list_labels` function L668-702 — `( State(dal): State<DAL>, Extension(auth_payload): Extension<AuthPayload>, Path(...` — event logging, label management, annotation management, target management, and heartbeat recording.
-  `add_label` function L725-756 — `( State(dal): State<DAL>, Extension(auth_payload): Extension<AuthPayload>, Path(...` — event logging, label management, annotation management, target management, and heartbeat recording.
-  `remove_label` function L780-827 — `( State(dal): State<DAL>, Extension(auth_payload): Extension<AuthPayload>, Path(...` — event logging, label management, annotation management, target management, and heartbeat recording.
-  `list_annotations` function L850-890 — `( State(dal): State<DAL>, Extension(auth_payload): Extension<AuthPayload>, Path(...` — event logging, label management, annotation management, target management, and heartbeat recording.
-  `add_annotation` function L913-947 — `( State(dal): State<DAL>, Extension(auth_payload): Extension<AuthPayload>, Path(...` — event logging, label management, annotation management, target management, and heartbeat recording.
-  `remove_annotation` function L971-1018 — `( State(dal): State<DAL>, Extension(auth_payload): Extension<AuthPayload>, Path(...` — event logging, label management, annotation management, target management, and heartbeat recording.
-  `list_targets` function L1041-1075 — `( State(dal): State<DAL>, Extension(auth_payload): Extension<AuthPayload>, Path(...` — event logging, label management, annotation management, target management, and heartbeat recording.
-  `add_target` function L1099-1130 — `( State(dal): State<DAL>, Extension(auth_payload): Extension<AuthPayload>, Path(...` — event logging, label management, annotation management, target management, and heartbeat recording.
-  `remove_target` function L1155-1205 — `( State(dal): State<DAL>, Extension(auth_payload): Extension<AuthPayload>, Path(...` — event logging, label management, annotation management, target management, and heartbeat recording.
-  `record_heartbeat` function L1227-1270 — `( State(dal): State<DAL>, Extension(auth_payload): Extension<AuthPayload>, Path(...` — event logging, label management, annotation management, target management, and heartbeat recording.
-  `TargetStateParams` struct L1274-1277 — `{ mode: Option<String> }` — Defines query parameters for the target state endpoint
-  `get_target_state` function L1306-1358 — `( State(dal): State<DAL>, Extension(auth_payload): Extension<AuthPayload>, Path(...` — event logging, label management, annotation management, target management, and heartbeat recording.
-  `get_associated_stacks` function L1381-1421 — `( State(dal): State<DAL>, Extension(auth_payload): Extension<AuthPayload>, Path(...` — event logging, label management, annotation management, target management, and heartbeat recording.
-  `rotate_agent_pak` function L1445-1517 — `( State(dal): State<DAL>, Extension(auth_payload): Extension<AuthPayload>, Path(...` — event logging, label management, annotation management, target management, and heartbeat recording.

#### crates/brokkr-broker/src/api/v1/auth.rs

- pub `routes` function L19-21 — `() -> Router<DAL>` — Creates and returns the authentication routes for the API.
-  `verify_pak` function L38-44 — `(Extension(auth_payload): Extension<AuthPayload>) -> Json<AuthResponse>` — This module provides routes and handlers for authentication-related endpoints.

#### crates/brokkr-broker/src/api/v1/deployment_objects.rs

- pub `routes` function L28-31 — `() -> Router<DAL>` — Creates and returns the router for deployment object endpoints.
-  `get_deployment_object` function L60-184 — `( State(dal): State<DAL>, Extension(auth_payload): Extension<AuthPayload>, Path(...` — including retrieval based on user authentication and authorization.

#### crates/brokkr-broker/src/api/v1/diagnostics.rs

- pub `routes` function L29-43 — `() -> Router<DAL>` — Creates and returns the router for diagnostic endpoints.
- pub `CreateDiagnosticRequest` struct L47-54 — `{ agent_id: Uuid, requested_by: Option<String>, retention_minutes: Option<i64> }` — Request body for creating a diagnostic request.
- pub `DiagnosticResponse` struct L58-63 — `{ request: DiagnosticRequest, result: Option<DiagnosticResult> }` — Response containing a diagnostic request with optional result.
- pub `SubmitDiagnosticResult` struct L67-76 — `{ pod_statuses: String, events: String, log_tails: Option<String>, collected_at:...` — Request body for submitting diagnostic results.
-  `create_diagnostic_request` function L98-184 — `( State(dal): State<DAL>, Extension(auth_payload): Extension<AuthPayload>, Path(...` — pick up and execute these requests, returning detailed diagnostic data.
-  `get_diagnostic` function L208-252 — `( State(dal): State<DAL>, Extension(auth_payload): Extension<AuthPayload>, Path(...` — pick up and execute these requests, returning detailed diagnostic data.
-  `get_pending_diagnostics` function L272-313 — `( State(dal): State<DAL>, Extension(auth_payload): Extension<AuthPayload>, Path(...` — pick up and execute these requests, returning detailed diagnostic data.
-  `claim_diagnostic` function L335-404 — `( State(dal): State<DAL>, Extension(auth_payload): Extension<AuthPayload>, Path(...` — pick up and execute these requests, returning detailed diagnostic data.
-  `submit_diagnostic_result` function L428-527 — `( State(dal): State<DAL>, Extension(auth_payload): Extension<AuthPayload>, Path(...` — pick up and execute these requests, returning detailed diagnostic data.

#### crates/brokkr-broker/src/api/v1/generators.rs

- pub `CreateGeneratorResponse` struct L29-34 — `{ generator: Generator, pak: String }` — Response for a successful generator creation
- pub `routes` function L41-50 — `() -> Router<DAL>` — Creates and returns the router for generator endpoints.
-  `list_generators` function L75-101 — `( State(dal): State<DAL>, Extension(auth_payload): Extension<AuthPayload>, ) -> ...` — Lists all generators.
-  `create_generator` function L129-179 — `( State(dal): State<DAL>, Extension(auth_payload): Extension<AuthPayload>, Json(...` — Creates a new generator.
-  `get_generator` function L210-244 — `( State(dal): State<DAL>, Extension(auth_payload): Extension<AuthPayload>, Path(...` — Retrieves a specific generator by ID.
-  `update_generator` function L277-305 — `( State(dal): State<DAL>, Extension(auth_payload): Extension<AuthPayload>, Path(...` — Updates an existing generator.
-  `delete_generator` function L336-363 — `( State(dal): State<DAL>, Extension(auth_payload): Extension<AuthPayload>, Path(...` — Deletes a generator.
-  `rotate_generator_pak` function L394-454 — `( State(dal): State<DAL>, Extension(auth_payload): Extension<AuthPayload>, Path(...` — Rotates the PAK for a specific generator.

#### crates/brokkr-broker/src/api/v1/health.rs

- pub `routes` function L29-35 — `() -> Router<DAL>` — Creates and returns the router for health-related endpoints.
- pub `HealthStatusUpdate` struct L39-42 — `{ deployment_objects: Vec<DeploymentObjectHealthUpdate> }` — Request body for updating health status from an agent.
- pub `DeploymentObjectHealthUpdate` struct L46-55 — `{ id: Uuid, status: String, summary: Option<HealthSummary>, checked_at: DateTime...` — Health update for a single deployment object.
- pub `DeploymentHealthResponse` struct L59-66 — `{ deployment_object_id: Uuid, health_records: Vec<DeploymentHealth>, overall_sta...` — Response for deployment object health query.
- pub `StackHealthResponse` struct L70-77 — `{ stack_id: Uuid, overall_status: String, deployment_objects: Vec<DeploymentObje...` — Response for stack health query.
- pub `DeploymentObjectHealthSummary` struct L81-92 — `{ id: Uuid, status: String, healthy_agents: usize, degraded_agents: usize, faili...` — Summary of health for a deployment object within a stack.
-  `update_health_status` function L113-181 — `( State(dal): State<DAL>, Extension(auth_payload): Extension<AuthPayload>, Path(...` — including endpoints for agents to report health and for operators to query health.
-  `get_deployment_health` function L201-243 — `( State(dal): State<DAL>, Extension(auth_payload): Extension<AuthPayload>, Path(...` — including endpoints for agents to report health and for operators to query health.
-  `get_stack_health` function L263-344 — `( State(dal): State<DAL>, Extension(auth_payload): Extension<AuthPayload>, Path(...` — including endpoints for agents to report health and for operators to query health.
-  `compute_overall_status` function L348-358 — `(records: &[DeploymentHealth]) -> String` — Computes the overall status from a list of health records.

#### crates/brokkr-broker/src/api/v1/middleware.rs

- pub `AuthPayload` struct L30-37 — `{ admin: bool, agent: Option<Uuid>, generator: Option<Uuid> }` — Represents the authenticated entity's payload.
- pub `AuthResponse` struct L41-48 — `{ admin: bool, agent: Option<String>, generator: Option<String> }` — Represents the response structure for authentication information.
- pub `auth_middleware` function L64-93 — `( State(dal): State<DAL>, mut request: Request<Body>, next: Next, ) -> Result<Re...` — Middleware function for authenticating requests.
-  `verify_pak` function L108-173 — `(dal: &DAL, pak: &str) -> Result<AuthPayload, StatusCode>` — Verifies the provided PAK and returns the corresponding `AuthPayload`.

#### crates/brokkr-broker/src/api/v1/mod.rs

- pub `admin` module L13 — `-` — API v1 module for the Brokkr broker.
- pub `agent_events` module L14 — `-` — with authentication middleware.
- pub `agents` module L15 — `-` — with authentication middleware.
- pub `auth` module L16 — `-` — with authentication middleware.
- pub `deployment_objects` module L17 — `-` — with authentication middleware.
- pub `diagnostics` module L18 — `-` — with authentication middleware.
- pub `generators` module L19 — `-` — with authentication middleware.
- pub `health` module L20 — `-` — with authentication middleware.
- pub `middleware` module L21 — `-` — with authentication middleware.
- pub `openapi` module L22 — `-` — with authentication middleware.
- pub `stacks` module L23 — `-` — with authentication middleware.
- pub `templates` module L24 — `-` — with authentication middleware.
- pub `webhooks` module L25 — `-` — with authentication middleware.
- pub `work_orders` module L26 — `-` — with authentication middleware.
- pub `routes` function L41-73 — `(dal: DAL, cors_config: &Cors, reloadable_config: Option<ReloadableConfig>) -> R...` — Constructs and returns the main router for API v1.
-  `build_cors_layer` function L79-116 — `(config: &Cors) -> CorsLayer` — Builds a CORS layer from configuration.

#### crates/brokkr-broker/src/api/v1/openapi.rs

- pub `ApiDoc` struct L190 — `-`
- pub `configure_openapi` function L213-217 — `() -> Router<DAL>`
-  `SecurityAddon` struct L192 — `-`
-  `SecurityAddon` type L194-211 — `= SecurityAddon`
-  `modify` function L195-210 — `(&self, openapi: &mut utoipa::openapi::OpenApi)`
-  `serve_openapi` function L219-221 — `() -> Json<utoipa::openapi::OpenApi>`

#### crates/brokkr-broker/src/api/v1/stacks.rs

- pub `routes` function L34-57 — `() -> Router<DAL>`
- pub `TemplateInstantiationRequest` struct L811-816 — `{ template_id: Uuid, parameters: serde_json::Value }` — Request body for template instantiation.
-  `list_stacks` function L77-105 — `( State(dal): State<DAL>, Extension(auth_payload): Extension<AuthPayload>, ) -> ...`
-  `create_stack` function L125-178 — `( State(dal): State<DAL>, Extension(auth_payload): Extension<AuthPayload>, Json(...`
-  `get_stack` function L201-235 — `( State(dal): State<DAL>, Extension(auth_payload): Extension<AuthPayload>, Path(...`
-  `update_stack` function L259-328 — `( State(dal): State<DAL>, Extension(auth_payload): Extension<AuthPayload>, Path(...`
-  `delete_stack` function L351-409 — `( State(dal): State<DAL>, Extension(auth_payload): Extension<AuthPayload>, Path(...`
-  `list_deployment_objects` function L411-453 — `( State(dal): State<DAL>, Extension(auth_payload): Extension<AuthPayload>, Path(...`
-  `create_deployment_object` function L455-514 — `( State(dal): State<DAL>, Extension(auth_payload): Extension<AuthPayload>, Path(...`
-  `list_labels` function L516-537 — `( State(dal): State<DAL>, Extension(auth_payload): Extension<AuthPayload>, Path(...`
-  `add_label` function L539-589 — `( State(dal): State<DAL>, Extension(auth_payload): Extension<AuthPayload>, Path(...`
-  `remove_label` function L591-638 — `( State(dal): State<DAL>, Extension(auth_payload): Extension<AuthPayload>, Path(...`
-  `is_authorized_for_stack` function L640-686 — `( dal: &DAL, auth_payload: &AuthPayload, stack_id: Uuid, ) -> Result<bool, (Stat...`
-  `list_annotations` function L688-709 — `( State(dal): State<DAL>, Extension(auth_payload): Extension<AuthPayload>, Path(...`
-  `add_annotation` function L711-758 — `( State(dal): State<DAL>, Extension(auth_payload): Extension<AuthPayload>, Path(...`
-  `remove_annotation` function L760-807 — `( State(dal): State<DAL>, Extension(auth_payload): Extension<AuthPayload>, Path(...`
-  `instantiate_template` function L845-1070 — `( State(dal): State<DAL>, Extension(auth_payload): Extension<AuthPayload>, Path(...`

#### crates/brokkr-broker/src/api/v1/templates.rs

- pub `CreateTemplateRequest` struct L31-40 — `{ name: String, description: Option<String>, template_content: String, parameter...` — Request body for creating a new template.
- pub `UpdateTemplateRequest` struct L44-51 — `{ description: Option<String>, template_content: String, parameters_schema: Stri...` — Request body for updating a template (creates new version).
- pub `routes` function L54-72 — `() -> Router<DAL>` — Sets up the routes for template management.
- pub `AddAnnotationRequest` struct L770-775 — `{ key: String, value: String }` — Request body for adding an annotation.
-  `can_modify_template` function L78-86 — `(auth: &AuthPayload, template: &StackTemplate) -> bool` — Checks if the authenticated user can modify the given template.
-  `list_templates` function L107-158 — `( State(dal): State<DAL>, Extension(auth_payload): Extension<AuthPayload>, ) -> ...` — stack templates, as well as managing template labels and annotations.
-  `create_template` function L180-242 — `( State(dal): State<DAL>, Extension(auth_payload): Extension<AuthPayload>, Json(...` — stack templates, as well as managing template labels and annotations.
-  `get_template` function L262-312 — `( State(dal): State<DAL>, Extension(auth_payload): Extension<AuthPayload>, Path(...` — stack templates, as well as managing template labels and annotations.
-  `update_template` function L336-414 — `( State(dal): State<DAL>, Extension(auth_payload): Extension<AuthPayload>, Path(...` — stack templates, as well as managing template labels and annotations.
-  `delete_template` function L434-483 — `( State(dal): State<DAL>, Extension(auth_payload): Extension<AuthPayload>, Path(...` — stack templates, as well as managing template labels and annotations.
-  `list_labels` function L503-553 — `( State(dal): State<DAL>, Extension(auth_payload): Extension<AuthPayload>, Path(...` — stack templates, as well as managing template labels and annotations.
-  `add_label` function L575-621 — `( State(dal): State<DAL>, Extension(auth_payload): Extension<AuthPayload>, Path(...` — stack templates, as well as managing template labels and annotations.
-  `remove_label` function L642-696 — `( State(dal): State<DAL>, Extension(auth_payload): Extension<AuthPayload>, Path(...` — stack templates, as well as managing template labels and annotations.
-  `list_annotations` function L716-766 — `( State(dal): State<DAL>, Extension(auth_payload): Extension<AuthPayload>, Path(...` — stack templates, as well as managing template labels and annotations.
-  `add_annotation` function L797-844 — `( State(dal): State<DAL>, Extension(auth_payload): Extension<AuthPayload>, Path(...` — stack templates, as well as managing template labels and annotations.
-  `remove_annotation` function L865-919 — `( State(dal): State<DAL>, Extension(auth_payload): Extension<AuthPayload>, Path(...` — stack templates, as well as managing template labels and annotations.

#### crates/brokkr-broker/src/api/v1/webhooks.rs

- pub `CreateWebhookRequest` struct L40-66 — `{ name: String, url: String, auth_header: Option<String>, event_types: Vec<Strin...` — Request body for creating a webhook subscription.
- pub `UpdateWebhookRequest` struct L70-100 — `{ name: Option<String>, url: Option<String>, auth_header: Option<Option<String>>...` — Request body for updating a webhook subscription.
- pub `WebhookResponse` struct L104-131 — `{ id: Uuid, name: String, has_url: bool, has_auth_header: bool, event_types: Vec...` — Response for a webhook subscription (safe view without encrypted fields).
- pub `ListDeliveriesQuery` struct L164-174 — `{ status: Option<String>, limit: Option<i64>, offset: Option<i64> }` — Query parameters for listing deliveries.
- pub `routes` function L203-217 — `() -> Router<DAL>` — Creates and returns the router for webhook endpoints.
- pub `PendingWebhookDelivery` struct L877-896 — `{ id: Uuid, subscription_id: Uuid, event_type: String, payload: String, url: Str...` — Pending webhook delivery for an agent (includes decrypted secrets).
- pub `DeliveryResultRequest` struct L900-912 — `{ success: bool, status_code: Option<i32>, error: Option<String>, duration_ms: O...` — Request body for reporting delivery result.
-  `WebhookResponse` type L133-160 — `= WebhookResponse` — including CRUD operations and delivery status inspection.
-  `from` function L134-159 — `(sub: WebhookSubscription) -> Self` — including CRUD operations and delivery status inspection.
-  `encrypt_value` function L183-191 — `(value: &str) -> Result<Vec<u8>, (StatusCode, Json<serde_json::Value>)>` — Encrypts a value for storage.
-  `decrypt_value` function L194-196 — `(encrypted: &[u8]) -> Result<String, String>` — Decrypts a stored value back to a string.
-  `list_webhooks` function L237-265 — `( State(dal): State<DAL>, Extension(auth_payload): Extension<AuthPayload>, ) -> ...` — Lists all webhook subscriptions.
-  `list_event_types` function L280-291 — `( Extension(auth_payload): Extension<AuthPayload>, ) -> Result<Json<Vec<&'static...` — Lists all available event types for webhook subscriptions.
-  `create_webhook` function L309-412 — `( State(dal): State<DAL>, Extension(auth_payload): Extension<AuthPayload>, Json(...` — Creates a new webhook subscription.
-  `get_webhook` function L432-467 — `( State(dal): State<DAL>, Extension(auth_payload): Extension<AuthPayload>, Path(...` — Retrieves a specific webhook subscription by ID.
-  `update_webhook` function L489-582 — `( State(dal): State<DAL>, Extension(auth_payload): Extension<AuthPayload>, Path(...` — Updates an existing webhook subscription.
-  `delete_webhook` function L602-650 — `( State(dal): State<DAL>, Extension(auth_payload): Extension<AuthPayload>, Path(...` — Deletes a webhook subscription.
-  `list_deliveries` function L673-723 — `( State(dal): State<DAL>, Extension(auth_payload): Extension<AuthPayload>, Path(...` — Lists deliveries for a specific webhook subscription.
-  `test_webhook` function L744-869 — `( State(dal): State<DAL>, Extension(auth_payload): Extension<AuthPayload>, Path(...` — Sends a test event to the webhook endpoint.
-  `get_pending_agent_webhooks` function L933-1040 — `( State(dal): State<DAL>, Extension(auth_payload): Extension<AuthPayload>, Path(...` — Gets pending webhook deliveries for an agent to process.
-  `report_delivery_result` function L1061-1176 — `( State(dal): State<DAL>, Extension(auth_payload): Extension<AuthPayload>, Path(...` — Reports the result of a webhook delivery attempt by an agent.

#### crates/brokkr-broker/src/api/v1/work_orders.rs

- pub `routes` function L47-62 — `() -> Router<DAL>` — Creates and returns a router for work order-related endpoints.
- pub `agent_routes` function L66-71 — `() -> Router<DAL>` — Creates agent-specific routes for work order operations.
- pub `CreateWorkOrderRequest` struct L79-99 — `{ work_type: String, yaml_content: String, max_retries: Option<i32>, backoff_sec...` — Request body for creating a new work order.
- pub `WorkOrderTargeting` struct L106-116 — `{ agent_ids: Option<Vec<Uuid>>, labels: Option<Vec<String>>, annotations: Option...` — Targeting configuration for work orders.
- pub `ClaimWorkOrderRequest` struct L120-123 — `{ agent_id: Uuid }` — Request body for claiming a work order.
- pub `CompleteWorkOrderRequest` struct L127-136 — `{ success: bool, message: Option<String>, retryable: bool }` — Request body for completing a work order.
- pub `ListWorkOrdersQuery` struct L144-149 — `{ status: Option<String>, work_type: Option<String> }` — Query parameters for listing work orders.
- pub `ListPendingQuery` struct L153-156 — `{ work_type: Option<String> }` — Query parameters for listing pending work orders for an agent.
- pub `ListLogQuery` struct L160-169 — `{ work_type: Option<String>, success: Option<bool>, agent_id: Option<Uuid>, limi...` — Query parameters for listing work order log.
-  `default_retryable` function L138-140 — `() -> bool` — - `GET /api/v1/work-order-log/:id` - Get completed work order by ID
-  `list_work_orders` function L194-225 — `( State(dal): State<DAL>, Extension(auth_payload): Extension<AuthPayload>, Query...` — - `GET /api/v1/work-order-log/:id` - Get completed work order by ID
-  `create_work_order` function L244-353 — `( State(dal): State<DAL>, Extension(auth_payload): Extension<AuthPayload>, Json(...` — - `GET /api/v1/work-order-log/:id` - Get completed work order by ID
-  `get_work_order` function L372-459 — `( State(dal): State<DAL>, Extension(auth_payload): Extension<AuthPayload>, Path(...` — - `GET /api/v1/work-order-log/:id` - Get completed work order by ID
-  `delete_work_order` function L478-513 — `( State(dal): State<DAL>, Extension(auth_payload): Extension<AuthPayload>, Path(...` — - `GET /api/v1/work-order-log/:id` - Get completed work order by ID
-  `list_pending_for_agent` function L538-581 — `( State(dal): State<DAL>, Extension(auth_payload): Extension<AuthPayload>, Path(...` — - `GET /api/v1/work-order-log/:id` - Get completed work order by ID
-  `claim_work_order` function L603-649 — `( State(dal): State<DAL>, Extension(auth_payload): Extension<AuthPayload>, Path(...` — - `GET /api/v1/work-order-log/:id` - Get completed work order by ID
-  `complete_work_order` function L675-763 — `( State(dal): State<DAL>, Extension(auth_payload): Extension<AuthPayload>, Path(...` — - `GET /api/v1/work-order-log/:id` - Get completed work order by ID
-  `list_work_order_log` function L790-826 — `( State(dal): State<DAL>, Extension(auth_payload): Extension<AuthPayload>, Query...` — - `GET /api/v1/work-order-log/:id` - Get completed work order by ID
-  `get_work_order_log` function L845-883 — `( State(dal): State<DAL>, Extension(auth_payload): Extension<AuthPayload>, Path(...` — - `GET /api/v1/work-order-log/:id` - Get completed work order by ID

### crates/brokkr-broker/src

> *Semantic summary to be generated by AI agent.*

#### crates/brokkr-broker/src/bin.rs

-  `main` function L24-61 — `() -> Result<(), Box<dyn std::error::Error>>` — Main function to run the Brokkr Broker application

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
- pub `init` function L139-149 — `()` — Initializes all metrics by forcing lazy static evaluation
- pub `encode_metrics` function L156-167 — `() -> String` — Encodes all registered metrics in Prometheus text format
- pub `record_http_request` function L180-192 — `(endpoint: &str, method: &str, status: u16, duration_seconds: f64)` — Records an HTTP request metric
- pub `record_db_query` function L220-228 — `(query_type: &str, duration_seconds: f64)` — Records a database query metric
- pub `set_active_agents` function L231-233 — `(count: i64)` — Updates the active agents gauge
- pub `set_stacks_total` function L236-238 — `(count: i64)` — Updates the total stacks gauge
- pub `set_deployment_objects_total` function L241-243 — `(count: i64)` — Updates the total deployment objects gauge
- pub `set_agent_heartbeat_age` function L246-250 — `(agent_id: &str, agent_name: &str, age_seconds: f64)` — Updates the heartbeat age for a specific agent
-  `normalize_endpoint` function L196-213 — `(path: &str) -> String` — Normalizes an endpoint path to reduce cardinality
-  `tests` module L253-370 — `-` — It exposes metrics about HTTP requests, database queries, and system state.
-  `test_init_registers_all_metrics` function L257-303 — `()` — It exposes metrics about HTTP requests, database queries, and system state.
-  `test_normalize_endpoint_replaces_uuids` function L306-310 — `()` — It exposes metrics about HTTP requests, database queries, and system state.
-  `test_normalize_endpoint_replaces_numeric_ids` function L313-317 — `()` — It exposes metrics about HTTP requests, database queries, and system state.
-  `test_normalize_endpoint_preserves_regular_paths` function L320-328 — `()` — It exposes metrics about HTTP requests, database queries, and system state.
-  `test_record_http_request_increments_counter` function L331-345 — `()` — It exposes metrics about HTTP requests, database queries, and system state.
-  `test_set_active_agents` function L348-357 — `()` — It exposes metrics about HTTP requests, database queries, and system state.
-  `test_set_stacks_total` function L360-369 — `()` — It exposes metrics about HTTP requests, database queries, and system state.

### crates/brokkr-broker/src/cli

> *Semantic summary to be generated by AI agent.*

#### crates/brokkr-broker/src/cli/commands.rs

- pub `MIGRATIONS` variable L25 — `: EmbeddedMigrations`
- pub `serve` function L38-181 — `(config: &Settings) -> Result<(), Box<dyn std::error::Error>>` — Function to start the Brokkr Broker server
- pub `rotate_admin` function L186-198 — `(config: &Settings) -> Result<(), Box<dyn std::error::Error>>` — Function to rotate the admin key
- pub `rotate_agent_key` function L200-217 — `(config: &Settings, uuid: Uuid) -> Result<(), Box<dyn std::error::Error>>`
- pub `rotate_generator_key` function L219-244 — `( config: &Settings, uuid: Uuid, ) -> Result<(), Box<dyn std::error::Error>>`
- pub `create_agent` function L246-279 — `( config: &Settings, name: String, cluster_name: String, ) -> Result<(), Box<dyn...`
- pub `create_generator` function L281-311 — `( config: &Settings, name: String, description: Option<String>, ) -> Result<(), ...`
-  `Count` struct L29-32 — `{ count: i64 }`

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

#### crates/brokkr-broker/src/dal/agent_labels.rs

- pub `AgentLabelsDAL` struct L20-23 — `{ dal: &'a DAL }` — Data Access Layer for AgentLabel operations.
- pub `create` function L35-40 — `(&self, new_label: &NewAgentLabel) -> Result<AgentLabel, diesel::result::Error>` — Creates a new agent label in the database.
- pub `get` function L51-57 — `(&self, label_id: Uuid) -> Result<Option<AgentLabel>, diesel::result::Error>` — Retrieves an agent label by its ID.
- pub `list_for_agent` function L68-73 — `(&self, agent_id: Uuid) -> Result<Vec<AgentLabel>, diesel::result::Error>` — Lists all labels for a specific agent.
- pub `delete` function L84-87 — `(&self, label_id: Uuid) -> Result<usize, diesel::result::Error>` — Deletes an agent label from the database.
- pub `delete_all_for_agent` function L98-102 — `(&self, agent_id: Uuid) -> Result<usize, diesel::result::Error>` — Deletes all labels for a specific agent.
- pub `label_exists` function L115-123 — `(&self, agent_id: Uuid, label: &str) -> Result<bool, diesel::result::Error>` — Checks if a label exists for a specific agent.
- pub `delete_by_agent_and_label` function L138-150 — `( &self, agent_id: Uuid, label: &str, ) -> Result<usize, diesel::result::Error>` — Deletes a specific label for an agent using a single indexed query.

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

- pub `AgentFilter` struct L22-27 — `{ labels: Vec<String>, annotations: Vec<(String, String)>, agent_targets: Vec<Uu...` — Struct for filtering agents based on various criteria.
- pub `AgentsDAL` struct L30-33 — `{ dal: &'a DAL }` — Data Access Layer for Agent operations.
- pub `create` function L57-74 — `(&self, new_agent: &NewAgent) -> Result<Agent, diesel::result::Error>` — Creates a new agent in the database.
- pub `get` function L86-93 — `(&self, agent_uuid: Uuid) -> Result<Option<Agent>, diesel::result::Error>` — Retrieves a non-deleted agent by its UUID.
- pub `get_including_deleted` function L105-114 — `( &self, agent_uuid: Uuid, ) -> Result<Option<Agent>, diesel::result::Error>` — Retrieves an agent by its UUID, including deleted agents.
- pub `list` function L122-127 — `(&self) -> Result<Vec<Agent>, diesel::result::Error>` — Lists all non-deleted agents from the database.
- pub `list_all` function L135-138 — `(&self) -> Result<Vec<Agent>, diesel::result::Error>` — Lists all agents from the database, including deleted ones.
- pub `update` function L151-160 — `( &self, agent_uuid: Uuid, updated_agent: &Agent, ) -> Result<Agent, diesel::res...` — Updates an existing agent in the database.
- pub `soft_delete` function L172-188 — `(&self, agent_uuid: Uuid) -> Result<usize, diesel::result::Error>` — Soft deletes an agent by setting its deleted_at timestamp to the current time.
- pub `hard_delete` function L200-203 — `(&self, agent_uuid: Uuid) -> Result<usize, diesel::result::Error>` — Hard deletes an agent from the database.
- pub `filter_by_labels` function L237-270 — `( &self, labels: Vec<String>, filter_type: FilterType, ) -> Result<Vec<Agent>, d...` — Filters agents by labels.
- pub `filter_by_annotations` function L309-372 — `( &self, annotations: Vec<(String, String)>, filter_type: FilterType, ) -> Resul...` — Filters agents by annotations.
- pub `get_agent_by_target_id` function L384-396 — `( &self, agent_target_id: Uuid, ) -> Result<Option<Agent>, diesel::result::Error...` — Retrieves an agent by its target ID.
- pub `get_agent_details` function L409-429 — `( &self, agent_id: Uuid, ) -> Result<(Vec<AgentLabel>, Vec<AgentTarget>, Vec<Age...` — Retrieves labels, targets, and annotations associated with a specific agent.
- pub `record_heartbeat` function L440-448 — `(&self, agent_id: Uuid) -> Result<(), diesel::result::Error>` — Records a heartbeat for the specified agent.
- pub `update_pak_hash` function L461-470 — `( &self, agent_uuid: Uuid, new_pak_hash: String, ) -> Result<Agent, diesel::resu...` — Updates the pak_hash for an agent.
- pub `get_by_name_and_cluster_name` function L483-495 — `( &self, name: String, cluster_name: String, ) -> Result<Option<Agent>, diesel::...` — Retrieves an agent by its name and cluster name.
- pub `get_by_pak_hash` function L510-517 — `(&self, pak_hash: &str) -> Result<Option<Agent>, diesel::result::Error>` — Retrieves an agent by its PAK hash.

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
- pub `list_by_agent` function L167-174 — `(&self, agent_id: Uuid) -> Result<Vec<DeploymentHealth>, diesel::result::Error>` — Lists all health records for a specific agent.
- pub `list_by_stack` function L185-195 — `(&self, stack_id: Uuid) -> Result<Vec<DeploymentHealth>, diesel::result::Error>` — Lists all health records for deployment objects in a specific stack.
- pub `list_by_status` function L206-213 — `(&self, status: &str) -> Result<Vec<DeploymentHealth>, diesel::result::Error>` — Lists all health records with a specific status.
- pub `delete_by_agent_and_deployment` function L225-238 — `( &self, agent_id: Uuid, deployment_object_id: Uuid, ) -> Result<usize, diesel::...` — Deletes the health record for a specific agent and deployment object.
- pub `delete_by_agent` function L249-254 — `(&self, agent_id: Uuid) -> Result<usize, diesel::result::Error>` — Deletes all health records for a specific agent.

#### crates/brokkr-broker/src/dal/deployment_objects.rs

- pub `DeploymentObjectsDAL` struct L24-27 — `{ dal: &'a DAL }` — Data Access Layer for DeploymentObject operations.
- pub `create` function L39-58 — `( &self, new_deployment_object: &NewDeploymentObject, ) -> Result<DeploymentObje...` — Creates a new deployment object in the database.
- pub `get` function L69-79 — `( &self, deployment_object_uuid: Uuid, ) -> Result<Option<DeploymentObject>, die...` — Retrieves a non-deleted deployment object by its UUID.
- pub `get_including_deleted` function L90-99 — `( &self, deployment_object_uuid: Uuid, ) -> Result<Option<DeploymentObject>, die...` — Retrieves a deployment object by its UUID, including deleted objects.
- pub `list_for_stack` function L110-120 — `( &self, stack_id: Uuid, ) -> Result<Vec<DeploymentObject>, diesel::result::Erro...` — Lists all non-deleted deployment objects for a specific stack.
- pub `list_all_for_stack` function L131-140 — `( &self, stack_id: Uuid, ) -> Result<Vec<DeploymentObject>, diesel::result::Erro...` — Lists all deployment objects for a specific stack, including deleted ones.
- pub `soft_delete` function L151-180 — `( &self, deployment_object_uuid: Uuid, ) -> Result<usize, diesel::result::Error>` — Soft deletes a deployment object by setting its deleted_at timestamp to the current time.
- pub `get_latest_for_stack` function L191-202 — `( &self, stack_id: Uuid, ) -> Result<Option<DeploymentObject>, diesel::result::E...` — Retrieves the latest non-deleted deployment object for a specific stack.
- pub `get_target_state_for_agent` function L221-259 — `( &self, agent_id: Uuid, include_deployed: bool, ) -> Result<Vec<DeploymentObjec...` — Retrieves a list of undeployed objects for an agent based on its responsibilities.
- pub `search` function L271-281 — `( &self, yaml_checksum: &str, ) -> Result<Vec<DeploymentObject>, diesel::result:...` — Searches for deployment objects by checksum.
- pub `get_desired_state_for_agent` function L296-316 — `( &self, agent_id: Uuid, ) -> Result<Vec<DeploymentObject>, diesel::result::Erro...` — Retrieves applicable deployment objects for a given agent.

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

- pub `DalError` enum L40-47 — `ConnectionPool | Query | NotFound` — Error types for DAL operations.
- pub `agents` module L93 — `-` — ```
- pub `agent_annotations` module L96 — `-` — ```
- pub `audit_logs` module L99 — `-` — ```
- pub `agent_events` module L102 — `-` — ```
- pub `agent_labels` module L105 — `-` — ```
- pub `agent_targets` module L108 — `-` — ```
- pub `stacks` module L111 — `-` — ```
- pub `stack_annotations` module L114 — `-` — ```
- pub `stack_labels` module L117 — `-` — ```
- pub `deployment_health` module L120 — `-` — ```
- pub `deployment_objects` module L123 — `-` — ```
- pub `diagnostic_requests` module L126 — `-` — ```
- pub `diagnostic_results` module L129 — `-` — ```
- pub `generators` module L132 — `-` — ```
- pub `templates` module L135 — `-` — ```
- pub `template_labels` module L138 — `-` — ```
- pub `template_annotations` module L141 — `-` — ```
- pub `template_targets` module L144 — `-` — ```
- pub `rendered_deployment_objects` module L147 — `-` — ```
- pub `webhook_deliveries` module L150 — `-` — ```
- pub `webhook_subscriptions` module L153 — `-` — ```
- pub `work_orders` module L156 — `-` — ```
- pub `DAL` struct L165-168 — `{ pool: ConnectionPool }` — The main Data Access Layer struct.
- pub `new` function L180-182 — `(pool: ConnectionPool) -> Self` — Creates a new DAL instance with the given connection pool.
- pub `agents` function L189-191 — `(&self) -> AgentsDAL` — Provides access to the Agents Data Access Layer.
- pub `agent_annotations` function L198-200 — `(&self) -> AgentAnnotationsDAL` — Provides access to the Agent Annotations Data Access Layer.
- pub `agent_events` function L207-209 — `(&self) -> AgentEventsDAL` — Provides access to the Agent Events Data Access Layer.
- pub `agent_labels` function L216-218 — `(&self) -> AgentLabelsDAL` — Provides access to the Agent Labels Data Access Layer.
- pub `agent_targets` function L225-227 — `(&self) -> AgentTargetsDAL` — Provides access to the Agent Targets Data Access Layer.
- pub `stack_labels` function L234-236 — `(&self) -> StackLabelsDAL` — Provides access to the Stack Labels Data Access Layer.
- pub `stack_annotations` function L243-245 — `(&self) -> StackAnnotationsDAL` — Provides access to the Stack Annotations Data Access Layer.
- pub `stacks` function L252-254 — `(&self) -> StacksDAL` — Provides access to the Stacks Data Access Layer.
- pub `deployment_health` function L261-263 — `(&self) -> DeploymentHealthDAL` — Provides access to the Deployment Health Data Access Layer.
- pub `deployment_objects` function L270-272 — `(&self) -> DeploymentObjectsDAL` — Provides access to the Deployment Objects Data Access Layer.
- pub `generators` function L279-281 — `(&self) -> GeneratorsDAL` — Provides access to the Generators Data Access Layer.
- pub `templates` function L288-290 — `(&self) -> TemplatesDAL` — Provides access to the Templates Data Access Layer.
- pub `template_labels` function L297-299 — `(&self) -> TemplateLabelsDAL` — Provides access to the Template Labels Data Access Layer.
- pub `template_annotations` function L306-308 — `(&self) -> TemplateAnnotationsDAL` — Provides access to the Template Annotations Data Access Layer.
- pub `template_targets` function L315-317 — `(&self) -> TemplateTargetsDAL` — Provides access to the Template Targets Data Access Layer.
- pub `rendered_deployment_objects` function L324-326 — `(&self) -> RenderedDeploymentObjectsDAL` — Provides access to the Rendered Deployment Objects Data Access Layer.
- pub `work_orders` function L333-335 — `(&self) -> WorkOrdersDAL` — Provides access to the Work Orders Data Access Layer.
- pub `diagnostic_requests` function L342-344 — `(&self) -> DiagnosticRequestsDAL` — Provides access to the Diagnostic Requests Data Access Layer.
- pub `diagnostic_results` function L351-353 — `(&self) -> DiagnosticResultsDAL` — Provides access to the Diagnostic Results Data Access Layer.
- pub `webhook_subscriptions` function L360-362 — `(&self) -> WebhookSubscriptionsDAL` — Provides access to the Webhook Subscriptions Data Access Layer.
- pub `webhook_deliveries` function L369-371 — `(&self) -> WebhookDeliveriesDAL` — Provides access to the Webhook Deliveries Data Access Layer.
- pub `audit_logs` function L378-380 — `(&self) -> AuditLogsDAL` — Provides access to the Audit Logs Data Access Layer.
- pub `FilterType` enum L384-387 — `And | Or` — ```
-  `DalError` type L49-57 — `= DalError` — ```
-  `fmt` function L50-56 — `(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result` — ```
-  `DalError` type L59 — `= DalError` — ```
-  `DalError` type L61-65 — `= DalError` — ```
-  `from` function L62-64 — `(e: r2d2::Error) -> Self` — ```
-  `DalError` type L67-74 — `= DalError` — ```
-  `from` function L68-73 — `(e: diesel::result::Error) -> Self` — ```
-  `DalError` type L76-91 — `impl IntoResponse for DalError` — ```
-  `into_response` function L77-90 — `(self) -> Response` — ```
-  `DAL` type L170-381 — `= DAL` — ```

#### crates/brokkr-broker/src/dal/rendered_deployment_objects.rs

- pub `RenderedDeploymentObjectsDAL` struct L22-25 — `{ dal: &'a DAL }` — Handles database operations for RenderedDeploymentObject entities.
- pub `create` function L37-45 — `( &self, new_record: &NewRenderedDeploymentObject, ) -> Result<RenderedDeploymen...` — Creates a new rendered deployment object provenance record in the database.
- pub `get` function L56-65 — `( &self, record_id: Uuid, ) -> Result<Option<RenderedDeploymentObject>, diesel::...` — Retrieves a rendered deployment object provenance record by its ID.
- pub `get_by_deployment_object` function L76-85 — `( &self, deployment_object_id: Uuid, ) -> Result<Option<RenderedDeploymentObject...` — Retrieves the provenance record for a specific deployment object.
- pub `list_by_template` function L97-115 — `( &self, template_id: Uuid, version: Option<i32>, ) -> Result<Vec<RenderedDeploy...` — Lists all provenance records for a specific template.
- pub `list` function L122-127 — `(&self) -> Result<Vec<RenderedDeploymentObject>, diesel::result::Error>` — Lists all provenance records from the database.
- pub `delete` function L138-144 — `(&self, record_id: Uuid) -> Result<usize, diesel::result::Error>` — Deletes a provenance record from the database.
- pub `delete_for_deployment_object` function L155-165 — `( &self, deployment_object_id: Uuid, ) -> Result<usize, diesel::result::Error>` — Deletes all provenance records for a specific deployment object.
- pub `delete_for_template` function L176-183 — `(&self, template_id: Uuid) -> Result<usize, diesel::result::Error>` — Deletes all provenance records for a specific template.

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
- pub `list_all` function L114-117 — `(&self) -> Result<Vec<Stack>, diesel::result::Error>` — Lists all stacks from the database, including deleted ones.
- pub `update` function L129-138 — `( &self, stack_uuid: Uuid, updated_stack: &Stack, ) -> Result<Stack, diesel::res...` — Updates an existing stack in the database.
- pub `soft_delete` function L149-165 — `(&self, stack_uuid: Uuid) -> Result<usize, diesel::result::Error>` — Soft deletes a stack by setting its deleted_at timestamp to the current time.
- pub `hard_delete` function L176-179 — `(&self, stack_uuid: Uuid) -> Result<usize, diesel::result::Error>` — Hard deletes a stack from the database.
- pub `filter_by_labels` function L191-224 — `( &self, labels: Vec<String>, filter_type: FilterType, ) -> Result<Vec<Stack>, d...` — Filters stacks by labels.
- pub `filter_by_annotations` function L236-285 — `( &self, annotations: Vec<(String, String)>, filter_type: FilterType, ) -> Resul...` — Filters stacks by annotations.
- pub `get_associated_stacks` function L299-354 — `( &self, agent_id: Uuid, ) -> Result<Vec<Stack>, diesel::result::Error>` — Retrieves all stacks associated with a specific agent based on its labels, annotations, and targets.

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
- pub `exists` function L119-131 — `( &self, template_id: Uuid, stack_id: Uuid, ) -> Result<bool, diesel::result::Er...` — Checks if a specific template-stack association exists.
- pub `delete` function L142-146 — `(&self, target_id: Uuid) -> Result<usize, diesel::result::Error>` — Deletes a template target from the database.
- pub `delete_for_template` function L157-163 — `(&self, template_id: Uuid) -> Result<usize, diesel::result::Error>` — Deletes all template targets for a specific template.
- pub `delete_for_stack` function L174-178 — `(&self, stack_id: Uuid) -> Result<usize, diesel::result::Error>` — Deletes all template targets for a specific stack.

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
- pub `filter_by_annotations` function L376-437 — `( &self, annotations: Vec<(String, String)>, filter_type: FilterType, ) -> Resul...` — Filters templates by annotations.

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
- pub `get_stats` function L459-482 — `( &self, subscription_id: Uuid, ) -> Result<DeliveryStats, diesel::result::Error...` — Gets delivery statistics for a subscription.
- pub `DeliveryStats` struct L487-498 — `{ pending: i64, acquired: i64, success: i64, failed: i64, dead: i64 }` — Statistics about webhook deliveries.
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
-  `matches_event_pattern` function L195-206 — `(pattern: &str, event_type: &str) -> bool` — Matches an event type against a pattern.
-  `tests` module L209-233 — `-` — It includes methods for creating, updating, deleting, and querying webhook subscriptions.
-  `test_matches_event_pattern_exact` function L213-216 — `()` — It includes methods for creating, updating, deleting, and querying webhook subscriptions.
-  `test_matches_event_pattern_wildcard_suffix` function L219-225 — `()` — It includes methods for creating, updating, deleting, and querying webhook subscriptions.
-  `test_matches_event_pattern_full_wildcard` function L228-232 — `()` — It includes methods for creating, updating, deleting, and querying webhook subscriptions.

#### crates/brokkr-broker/src/dal/work_orders.rs

- pub `WorkOrdersDAL` struct L48-51 — `{ dal: &'a DAL }` — Data Access Layer for WorkOrder operations.
- pub `create` function L67-83 — `(&self, new_work_order: &NewWorkOrder) -> Result<WorkOrder, diesel::result::Erro...` — Creates a new work order in the database.
- pub `get` function L94-100 — `(&self, work_order_id: Uuid) -> Result<Option<WorkOrder>, diesel::result::Error>` — Retrieves a work order by its UUID.
- pub `list` function L107-112 — `(&self) -> Result<Vec<WorkOrder>, diesel::result::Error>` — Lists all work orders from the database.
- pub `list_filtered` function L124-144 — `( &self, status: Option<&str>, work_type: Option<&str>, ) -> Result<Vec<WorkOrde...` — Lists work orders filtered by status and/or work type.
- pub `delete` function L157-160 — `(&self, work_order_id: Uuid) -> Result<usize, diesel::result::Error>` — Deletes a work order by its UUID (hard delete).
- pub `list_pending_for_agent` function L183-260 — `( &self, agent_id: Uuid, work_type: Option<&str>, ) -> Result<Vec<WorkOrder>, di...` — Lists pending work orders that are claimable by a specific agent.
- pub `claim` function L281-319 — `( &self, work_order_id: Uuid, agent_id: Uuid, ) -> Result<WorkOrder, diesel::res...` — Atomically claims a work order for an agent.
- pub `release` function L397-416 — `( &self, work_order_id: Uuid, agent_id: Uuid, ) -> Result<WorkOrder, diesel::res...` — Releases a claimed work order back to PENDING status.
- pub `complete_success` function L432-462 — `( &self, work_order_id: Uuid, result_message: Option<String>, ) -> Result<WorkOr...` — Completes a work order successfully and moves it to the log.
- pub `complete_failure` function L502-562 — `( &self, work_order_id: Uuid, error_message: String, retryable: bool, ) -> Resul...` — Completes a work order with failure.
- pub `process_retry_pending` function L575-589 — `(&self) -> Result<usize, diesel::result::Error>` — Resets RETRY_PENDING work orders to PENDING if their backoff period has elapsed.
- pub `process_stale_claims` function L600-614 — `(&self) -> Result<usize, diesel::result::Error>` — Resets stale claimed work orders to PENDING.
- pub `add_target` function L629-637 — `( &self, new_target: &NewWorkOrderTarget, ) -> Result<WorkOrderTarget, diesel::r...` — Adds an agent as a target for a work order.
- pub `add_targets` function L649-664 — `( &self, work_order_id: Uuid, agent_ids: &[Uuid], ) -> Result<usize, diesel::res...` — Adds multiple agents as targets for a work order.
- pub `list_targets` function L675-683 — `( &self, work_order_id: Uuid, ) -> Result<Vec<WorkOrderTarget>, diesel::result::...` — Lists all targets for a work order.
- pub `remove_target` function L695-707 — `( &self, work_order_id: Uuid, agent_id: Uuid, ) -> Result<usize, diesel::result:...` — Removes a target from a work order.
- pub `get_log` function L722-728 — `(&self, log_id: Uuid) -> Result<Option<WorkOrderLog>, diesel::result::Error>` — Retrieves a work order log entry by its UUID.
- pub `list_log` function L742-772 — `( &self, work_type: Option<&str>, success: Option<bool>, agent_id: Option<Uuid>,...` — Lists work order log entries with optional filtering.
- pub `add_label` function L787-795 — `( &self, new_label: &NewWorkOrderLabel, ) -> Result<WorkOrderLabel, diesel::resu...` — Adds a label to a work order.
- pub `add_labels` function L807-822 — `( &self, work_order_id: Uuid, labels: &[String], ) -> Result<usize, diesel::resu...` — Adds multiple labels to a work order.
- pub `list_labels` function L833-841 — `( &self, work_order_id: Uuid, ) -> Result<Vec<WorkOrderLabel>, diesel::result::E...` — Lists all labels for a work order.
- pub `remove_label` function L853-865 — `( &self, work_order_id: Uuid, label: &str, ) -> Result<usize, diesel::result::Er...` — Removes a label from a work order.
- pub `add_annotation` function L880-888 — `( &self, new_annotation: &NewWorkOrderAnnotation, ) -> Result<WorkOrderAnnotatio...` — Adds an annotation to a work order.
- pub `add_annotations` function L900-917 — `( &self, work_order_id: Uuid, annotations: &std::collections::HashMap<String, St...` — Adds multiple annotations to a work order.
- pub `list_annotations` function L928-936 — `( &self, work_order_id: Uuid, ) -> Result<Vec<WorkOrderAnnotation>, diesel::resu...` — Lists all annotations for a work order.
- pub `remove_annotation` function L949-963 — `( &self, work_order_id: Uuid, key: &str, value: &str, ) -> Result<usize, diesel:...` — Removes an annotation from a work order.
-  `is_agent_authorized_for_work_order` function L324-385 — `( &self, conn: &mut diesel::pg::PgConnection, work_order_id: Uuid, agent_id: Uui...` — Checks if an agent is authorized to claim a work order using any targeting mechanism.
-  `emit_completion_event` function L466-483 — `(&self, log: &WorkOrderLog)` — Emits a work order completion event.

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
- pub `log` function L225-234 — `(entry: NewAuditLog)` — Logs an audit entry to the global audit logger.
- pub `try_log` function L243-253 — `(entry: NewAuditLog) -> bool` — Tries to log an audit entry without blocking.
- pub `log_action` function L348-375 — `( actor_type: &str, actor_id: Option<uuid::Uuid>, action: &str, resource_type: &...` — Helper to create and log an audit entry in one call.
-  `DEFAULT_CHANNEL_SIZE` variable L41 — `: usize` — Default channel buffer size for audit entries.
-  `DEFAULT_BATCH_SIZE` variable L44 — `: usize` — Default batch size for writing to database.
-  `DEFAULT_FLUSH_INTERVAL_MS` variable L47 — `: u64` — Default flush interval in milliseconds.
-  `AUDIT_LOGGER` variable L50 — `: OnceCell<Arc<AuditLogger>>` — Global audit logger storage.
-  `AuditLoggerConfig` type L63-71 — `impl Default for AuditLoggerConfig` — ```
-  `default` function L64-70 — `() -> Self` — ```
-  `AuditLogger` type L80-181 — `= AuditLogger` — ```
-  `start_audit_writer` function L259-302 — `( dal: DAL, mut receiver: mpsc::Receiver<NewAuditLog>, batch_size: usize, flush_...` — Starts the background audit writer task.
-  `flush_buffer` function L305-331 — `(dal: &DAL, buffer: &mut Vec<NewAuditLog>)` — Flushes the buffer to the database.
-  `tests` module L378-431 — `-` — ```
-  `test_audit_logger_config_default` function L386-391 — `()` — ```
-  `test_log_without_logger_does_not_panic` function L394-407 — `()` — ```
-  `test_try_log_without_logger` function L410-423 — `()` — ```
-  `test_get_audit_logger_uninitialized` function L426-430 — `()` — ```

#### crates/brokkr-broker/src/utils/background_tasks.rs

- pub `DiagnosticCleanupConfig` struct L18-23 — `{ interval_seconds: u64, max_age_hours: i64 }` — Configuration for diagnostic cleanup task.
- pub `start_diagnostic_cleanup_task` function L43-86 — `(dal: DAL, config: DiagnosticCleanupConfig)` — Starts the diagnostic cleanup background task.
- pub `WorkOrderMaintenanceConfig` struct L89-92 — `{ interval_seconds: u64 }` — Configuration for work order maintenance task.
- pub `start_work_order_maintenance_task` function L111-148 — `(dal: DAL, config: WorkOrderMaintenanceConfig)` — Starts the work order maintenance background task.
- pub `WebhookDeliveryConfig` struct L151-156 — `{ interval_seconds: u64, batch_size: i64 }` — Configuration for webhook delivery worker.
- pub `WebhookCleanupConfig` struct L168-173 — `{ interval_seconds: u64, retention_days: i64 }` — Configuration for webhook cleanup task.
- pub `start_webhook_delivery_task` function L196-367 — `(dal: DAL, config: WebhookDeliveryConfig)` — Starts the webhook delivery worker background task.
- pub `start_webhook_cleanup_task` function L405-432 — `(dal: DAL, config: WebhookCleanupConfig)` — Starts the webhook cleanup background task.
- pub `AuditLogCleanupConfig` struct L435-440 — `{ interval_seconds: u64, retention_days: i64 }` — Configuration for audit log cleanup task.
- pub `start_audit_log_cleanup_task` function L459-486 — `(dal: DAL, config: AuditLogCleanupConfig)` — Starts the audit log cleanup background task.
-  `DiagnosticCleanupConfig` type L25-32 — `impl Default for DiagnosticCleanupConfig` — system health and cleanup expired data.
-  `default` function L26-31 — `() -> Self` — system health and cleanup expired data.
-  `WorkOrderMaintenanceConfig` type L94-100 — `impl Default for WorkOrderMaintenanceConfig` — system health and cleanup expired data.
-  `default` function L95-99 — `() -> Self` — system health and cleanup expired data.
-  `WebhookDeliveryConfig` type L158-165 — `impl Default for WebhookDeliveryConfig` — system health and cleanup expired data.
-  `default` function L159-164 — `() -> Self` — system health and cleanup expired data.
-  `WebhookCleanupConfig` type L175-182 — `impl Default for WebhookCleanupConfig` — system health and cleanup expired data.
-  `default` function L176-181 — `() -> Self` — system health and cleanup expired data.
-  `attempt_delivery` function L370-394 — `( client: &reqwest::Client, url: &str, auth_header: Option<&str>, payload: &str,...` — Attempts to deliver a webhook payload via HTTP POST.
-  `AuditLogCleanupConfig` type L442-449 — `impl Default for AuditLogCleanupConfig` — system health and cleanup expired data.
-  `default` function L443-448 — `() -> Self` — system health and cleanup expired data.
-  `tests` module L489-587 — `-` — system health and cleanup expired data.
-  `test_default_diagnostic_config` function L493-497 — `()` — system health and cleanup expired data.
-  `test_custom_diagnostic_config` function L500-507 — `()` — system health and cleanup expired data.
-  `test_default_work_order_config` function L510-513 — `()` — system health and cleanup expired data.
-  `test_custom_work_order_config` function L516-521 — `()` — system health and cleanup expired data.
-  `test_default_webhook_delivery_config` function L524-528 — `()` — system health and cleanup expired data.
-  `test_custom_webhook_delivery_config` function L531-538 — `()` — system health and cleanup expired data.
-  `test_default_webhook_cleanup_config` function L541-545 — `()` — system health and cleanup expired data.
-  `test_custom_webhook_cleanup_config` function L548-555 — `()` — system health and cleanup expired data.
-  `test_attempt_delivery_invalid_url` function L558-571 — `()` — system health and cleanup expired data.
-  `test_attempt_delivery_with_auth_header_invalid_url` function L574-586 — `()` — system health and cleanup expired data.

#### crates/brokkr-broker/src/utils/config_watcher.rs

- pub `ConfigWatcherConfig` struct L21-28 — `{ config_file_path: String, debounce_duration: Duration, enabled: bool }` — Configuration for the file watcher.
- pub `from_environment` function L45-85 — `() -> Option<Self>` — Creates a new ConfigWatcherConfig from environment variables.
- pub `start_config_watcher` function L101-123 — `( config: ReloadableConfig, watcher_config: ConfigWatcherConfig, ) -> Option<tok...` — Starts the configuration file watcher as a background task.
-  `ConfigWatcherConfig` type L30-38 — `impl Default for ConfigWatcherConfig` — file and trigger configuration reloads automatically.
-  `default` function L31-37 — `() -> Self` — file and trigger configuration reloads automatically.
-  `ConfigWatcherConfig` type L40-86 — `= ConfigWatcherConfig` — file and trigger configuration reloads automatically.
-  `run_config_watcher` function L126-224 — `( config: ReloadableConfig, watcher_config: ConfigWatcherConfig, ) -> Result<(),...` — Internal function that runs the configuration file watcher loop.
-  `tests` module L227-254 — `-` — file and trigger configuration reloads automatically.
-  `test_config_watcher_config_default` function L231-236 — `()` — file and trigger configuration reloads automatically.
-  `test_config_from_environment_no_file` function L239-243 — `()` — file and trigger configuration reloads automatically.
-  `test_config_from_environment_disabled` function L246-253 — `()` — file and trigger configuration reloads automatically.

#### crates/brokkr-broker/src/utils/encryption.rs

- pub `EncryptionError` enum L47-56 — `EncryptionFailed | DecryptionFailed | InvalidData | UnsupportedVersion` — Encryption error types
- pub `EncryptionKey` struct L72-77 — `{ key: [u8; 32], cipher: Aes256Gcm }` — Encryption key wrapper with AES-256-GCM cipher.
- pub `new` function L89-92 — `(key: [u8; 32]) -> Self` — Creates a new encryption key from raw bytes.
- pub `generate` function L95-99 — `() -> Self` — Creates a new random encryption key.
- pub `from_hex` function L102-112 — `(hex: &str) -> Result<Self, String>` — Creates a key from a hex-encoded string.
- pub `fingerprint` function L115-118 — `(&self) -> String` — Returns the key as a hex string (for logging key fingerprint only).
- pub `encrypt` function L124-142 — `(&self, plaintext: &[u8]) -> Result<Vec<u8>, EncryptionError>` — Encrypts data using AES-256-GCM.
- pub `decrypt` function L149-170 — `(&self, data: &[u8]) -> Result<Vec<u8>, EncryptionError>` — Decrypts data, automatically detecting the encryption version.
- pub `init_encryption_key` function L230-250 — `(key_hex: Option<&str>) -> Result<(), String>` — Initializes the global encryption key from configuration.
- pub `get_encryption_key` function L256-261 — `() -> Arc<EncryptionKey>` — Gets the global encryption key.
- pub `encrypt_string` function L270-272 — `(value: &str) -> Result<Vec<u8>, EncryptionError>` — Encrypts a string value for storage.
- pub `decrypt_string` function L281-286 — `(encrypted: &[u8]) -> Result<String, String>` — Decrypts bytes back to a string.
-  `VERSION_AES_GCM` variable L31 — `: u8` — Version byte for AES-256-GCM encrypted data
-  `VERSION_LEGACY_XOR` variable L34 — `: u8` — Version byte for legacy XOR encrypted data (read-only)
-  `AES_GCM_NONCE_SIZE` variable L37 — `: usize` — Nonce size for AES-256-GCM (96 bits)
-  `LEGACY_XOR_NONCE_SIZE` variable L40 — `: usize` — Legacy XOR nonce size (128 bits)
-  `ENCRYPTION_KEY` variable L43 — `: OnceCell<Arc<EncryptionKey>>` — Global encryption key storage.
-  `EncryptionError` type L58-67 — `= EncryptionError` — - 0x01: AES-256-GCM encryption
-  `fmt` function L59-66 — `(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result` — - 0x01: AES-256-GCM encryption
-  `EncryptionError` type L69 — `= EncryptionError` — - 0x01: AES-256-GCM encryption
-  `EncryptionKey` type L79-85 — `= EncryptionKey` — - 0x01: AES-256-GCM encryption
-  `fmt` function L80-84 — `(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result` — - 0x01: AES-256-GCM encryption
-  `EncryptionKey` type L87-219 — `= EncryptionKey` — - 0x01: AES-256-GCM encryption
-  `decrypt_aes_gcm` function L173-186 — `(&self, data: &[u8]) -> Result<Vec<u8>, EncryptionError>` — Decrypts AES-256-GCM encrypted data.
-  `decrypt_legacy_xor` function L193-218 — `(&self, data: &[u8]) -> Result<Vec<u8>, EncryptionError>` — Decrypts legacy XOR-encrypted data (for migration support).
-  `tests` module L289-433 — `-` — - 0x01: AES-256-GCM encryption
-  `test_encryption_key_from_hex` function L293-298 — `()` — - 0x01: AES-256-GCM encryption
-  `test_encryption_key_from_hex_invalid` function L301-307 — `()` — - 0x01: AES-256-GCM encryption
-  `test_encrypt_decrypt_roundtrip` function L310-318 — `()` — - 0x01: AES-256-GCM encryption
-  `test_encrypt_decrypt_empty` function L321-329 — `()` — - 0x01: AES-256-GCM encryption
-  `test_encrypt_produces_different_output` function L332-345 — `()` — - 0x01: AES-256-GCM encryption
-  `test_decrypt_wrong_key` function L348-357 — `()` — - 0x01: AES-256-GCM encryption
-  `test_decrypt_tampered_data` function L360-373 — `()` — - 0x01: AES-256-GCM encryption
-  `test_decrypt_too_short` function L376-381 — `()` — - 0x01: AES-256-GCM encryption
-  `test_fingerprint` function L384-391 — `()` — - 0x01: AES-256-GCM encryption
-  `test_version_byte_present` function L394-402 — `()` — - 0x01: AES-256-GCM encryption
-  `test_legacy_xor_decryption` function L405-432 — `()` — - 0x01: AES-256-GCM encryption

#### crates/brokkr-broker/src/utils/event_bus.rs

- pub `emit_event` function L30-101 — `(dal: &DAL, event: &BrokkrEvent) -> usize` — Emits an event by creating webhook deliveries for all matching subscriptions.
-  `tests` module L104-125 — `-` — matching subscriptions.
-  `test_brokkr_event_creation` function L110-116 — `()` — matching subscriptions.
-  `test_brokkr_event_unique_ids` function L119-124 — `()` — matching subscriptions.

#### crates/brokkr-broker/src/utils/matching.rs

- pub `MatchResult` struct L16-23 — `{ matches: bool, missing_labels: Vec<String>, missing_annotations: Vec<(String, ...` — Result of a template-to-stack matching operation.
- pub `template_matches_stack` function L44-78 — `( template_labels: &[String], template_annotations: &[(String, String)], stack_l...` — Check if a template can be instantiated into a stack.
-  `tests` module L81-265 — `-` — annotations are compatible with a target stack before instantiation.
-  `test_template_no_labels_matches_any_stack` function L85-96 — `()` — annotations are compatible with a target stack before instantiation.
-  `test_template_no_labels_matches_empty_stack` function L99-103 — `()` — annotations are compatible with a target stack before instantiation.
-  `test_template_labels_subset_of_stack_matches` function L106-116 — `()` — annotations are compatible with a target stack before instantiation.
-  `test_template_labels_exact_match` function L119-128 — `()` — annotations are compatible with a target stack before instantiation.
-  `test_template_label_not_on_stack` function L131-141 — `()` — annotations are compatible with a target stack before instantiation.
-  `test_template_multiple_missing_labels` function L144-157 — `()` — annotations are compatible with a target stack before instantiation.
-  `test_annotation_exact_match` function L160-169 — `()` — annotations are compatible with a target stack before instantiation.
-  `test_annotation_key_matches_value_differs` function L172-185 — `()` — annotations are compatible with a target stack before instantiation.
-  `test_annotation_missing_entirely` function L188-201 — `()` — annotations are compatible with a target stack before instantiation.
-  `test_mixed_labels_and_annotations_all_match` function L204-216 — `()` — annotations are compatible with a target stack before instantiation.
-  `test_mixed_labels_match_but_annotations_dont` function L219-233 — `()` — annotations are compatible with a target stack before instantiation.
-  `test_annotations_match_but_labels_dont` function L236-247 — `()` — annotations are compatible with a target stack before instantiation.
-  `test_both_labels_and_annotations_missing` function L250-264 — `()` — annotations are compatible with a target stack before instantiation.

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

- pub `create_pak_controller` function L33-47 — `( config: Option<&Settings>, ) -> Result<Arc<PrefixedApiKeyController<OsRng, Sha...` — Creates or retrieves the PAK controller.
- pub `create_pak` function L78-86 — `() -> Result<(String, String), Box<dyn std::error::Error>>` — Generates a new Prefixed API Key and its hash.
- pub `verify_pak` function L98-103 — `(pak: String, stored_hash: String) -> bool` — Verifies a Prefixed API Key against a stored hash.
- pub `generate_pak_hash` function L114-118 — `(pak: String) -> String` — Generates a hash for a given Prefixed API Key.
-  `PAK_CONTROLLER` variable L22 — `: OnceCell<Arc<PrefixedApiKeyController<OsRng, Sha256>>>` — Singleton instance of the PAK controller.
-  `create_pak_controller_inner` function L58-71 — `( config: &Settings, ) -> Result<PrefixedApiKeyController<OsRng, Sha256>, Box<dy...` — Internal function to create a new PAK controller.
-  `tests` module L121-279 — `-` — Prefixed API Keys using a singleton controller pattern.
-  `test_pak_controller_singleton` function L126-175 — `()` — Prefixed API Keys using a singleton controller pattern.
-  `test_verify_pak` function L178-224 — `()` — Prefixed API Keys using a singleton controller pattern.
-  `test_generate_pak_hash` function L227-278 — `()` — Prefixed API Keys using a singleton controller pattern.

#### crates/brokkr-broker/src/utils/templating.rs

- pub `TemplateError` struct L21-24 — `{ message: String, details: Option<String> }` — Error type for templating operations.
- pub `validate_tera_syntax` function L62-73 — `(template_content: &str) -> Result<(), TemplateError>` — Validate Tera template syntax without rendering.
- pub `render_template` function L101-123 — `(template_content: &str, parameters: &Value) -> Result<String, TemplateError>` — Render a Tera template with the provided parameters.
- pub `validate_json_schema` function L149-161 — `(schema_str: &str) -> Result<(), TemplateError>` — Validate that a string is a valid JSON Schema.
- pub `ParameterValidationError` struct L165-168 — `{ path: String, message: String }` — Validation error details for parameter validation.
- pub `validate_parameters` function L210-245 — `( schema_str: &str, parameters: &Value, ) -> Result<(), Vec<ParameterValidationE...` — Validate parameters against a JSON Schema.
-  `TemplateError` type L26-33 — `= TemplateError` — - Validating parameters against JSON Schema at instantiation time
-  `fmt` function L27-32 — `(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result` — - Validating parameters against JSON Schema at instantiation time
-  `TemplateError` type L35 — `= TemplateError` — - Validating parameters against JSON Schema at instantiation time
-  `ParameterValidationError` type L170-178 — `= ParameterValidationError` — - Validating parameters against JSON Schema at instantiation time
-  `fmt` function L171-177 — `(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result` — - Validating parameters against JSON Schema at instantiation time
-  `tests` module L248-505 — `-` — - Validating parameters against JSON Schema at instantiation time
-  `test_valid_tera_syntax` function L255-258 — `()` — - Validating parameters against JSON Schema at instantiation time
-  `test_valid_tera_syntax_with_filters` function L261-264 — `()` — - Validating parameters against JSON Schema at instantiation time
-  `test_valid_tera_syntax_with_conditionals` function L267-276 — `()` — - Validating parameters against JSON Schema at instantiation time
-  `test_valid_tera_syntax_with_loops` function L279-286 — `()` — - Validating parameters against JSON Schema at instantiation time
-  `test_invalid_tera_syntax_unclosed_brace` function L289-295 — `()` — - Validating parameters against JSON Schema at instantiation time
-  `test_invalid_tera_syntax_unclosed_block` function L298-302 — `()` — - Validating parameters against JSON Schema at instantiation time
-  `test_valid_tera_syntax_plain_text` function L305-308 — `()` — - Validating parameters against JSON Schema at instantiation time
-  `test_valid_tera_syntax_default_filter` function L311-314 — `()` — - Validating parameters against JSON Schema at instantiation time
-  `test_render_template_simple` function L319-324 — `()` — - Validating parameters against JSON Schema at instantiation time
-  `test_render_template_multiple_vars` function L327-333 — `()` — - Validating parameters against JSON Schema at instantiation time
-  `test_render_template_with_default` function L336-341 — `()` — - Validating parameters against JSON Schema at instantiation time
-  `test_render_template_missing_required_var` function L344-351 — `()` — - Validating parameters against JSON Schema at instantiation time
-  `test_render_template_with_filter` function L354-359 — `()` — - Validating parameters against JSON Schema at instantiation time
-  `test_render_template_nested_object` function L362-367 — `()` — - Validating parameters against JSON Schema at instantiation time
-  `test_valid_json_schema_simple` function L372-375 — `()` — - Validating parameters against JSON Schema at instantiation time
-  `test_valid_json_schema_with_properties` function L378-387 — `()` — - Validating parameters against JSON Schema at instantiation time
-  `test_valid_json_schema_with_required` function L390-399 — `()` — - Validating parameters against JSON Schema at instantiation time
-  `test_invalid_json_not_json` function L402-408 — `()` — - Validating parameters against JSON Schema at instantiation time
-  `test_empty_json_schema_valid` function L411-415 — `()` — - Validating parameters against JSON Schema at instantiation time
-  `test_validate_parameters_valid` function L420-424 — `()` — - Validating parameters against JSON Schema at instantiation time
-  `test_validate_parameters_missing_required` function L427-434 — `()` — - Validating parameters against JSON Schema at instantiation time
-  `test_validate_parameters_wrong_type` function L437-442 — `()` — - Validating parameters against JSON Schema at instantiation time
-  `test_validate_parameters_pattern` function L445-455 — `()` — - Validating parameters against JSON Schema at instantiation time
-  `test_validate_parameters_minimum` function L458-468 — `()` — - Validating parameters against JSON Schema at instantiation time
-  `test_validate_parameters_empty_schema` function L471-476 — `()` — - Validating parameters against JSON Schema at instantiation time
-  `test_validate_parameters_complex_schema` function L479-504 — `()` — - Validating parameters against JSON Schema at instantiation time

### crates/brokkr-broker/tests

> *Semantic summary to be generated by AI agent.*

#### crates/brokkr-broker/tests/fixtures.rs

- pub `MIGRATIONS` variable L43 — `: EmbeddedMigrations` — Embedded migrations for the test database.
- pub `TestFixture` struct L49-55 — `{ dal: DAL, settings: Settings, admin_pak: String, admin_generator: Generator }` — Represents a test fixture for the Brokkr project.
- pub `create_test_router` function L72-81 — `(&self) -> Router<DAL>` — Creates and returns an Axum Router with configured API routes.
- pub `new` function L98-139 — `() -> Self` — Creates a new TestFixture instance.
- pub `create_test_stack` function L154-166 — `( &self, name: String, description: Option<String>, generator_id: Uuid, ) -> Sta...` — Creates a new stack for testing purposes.
- pub `create_test_agent` function L178-184 — `(&self, name: String, cluster_name: String) -> Agent` — Creates a new agent for testing purposes.
- pub `create_test_deployment_object` function L197-210 — `( &self, stack_id: Uuid, yaml_content: String, is_deletion_marker: bool, ) -> De...` — Creates a new deployment object for testing purposes.
- pub `create_test_stack_label` function L222-229 — `(&self, stack_id: Uuid, label: String) -> StackLabel` — Creates a new stack label for testing purposes.
- pub `create_test_stack_annotation` function L242-257 — `( &self, stack_id: Uuid, key: &str, value: &str, ) -> StackAnnotation` — Creates a new stack annotation for testing purposes.
- pub `create_test_agent_annotation` function L270-282 — `( &self, agent_id: Uuid, key: String, value: String, ) -> AgentAnnotation` — Creates a new agent annotation for testing purposes.
- pub `create_test_agent_target` function L294-301 — `(&self, agent_id: Uuid, stack_id: Uuid) -> AgentTarget` — Creates a new agent target for testing purposes.
- pub `create_test_agent_event` function L316-335 — `( &self, agent: &Agent, deployment_object: &DeploymentObject, event_type: &str, ...` — Creates a new agent event for testing purposes.
- pub `create_test_agent_label` function L347-354 — `(&self, agent_id: Uuid, label: String) -> AgentLabel` — Creates a new agent label for testing purposes.
- pub `create_test_generator` function L366-384 — `( &self, name: String, description: Option<String>, api_key_hash: String, ) -> G...` — Creates a new generator for testing purposes.
- pub `create_test_generator_with_pak` function L386-404 — `( &self, name: String, description: Option<String>, ) -> (Generator, String)` — and agent events.
- pub `create_test_agent_with_pak` function L406-423 — `( &self, name: String, cluster_name: String, ) -> (Agent, String)` — and agent events.
- pub `create_test_template` function L438-450 — `( &self, generator_id: Option<Uuid>, name: String, description: Option<String>, ...` — Creates a new stack template for testing purposes.
- pub `create_test_template_label` function L462-469 — `(&self, template_id: Uuid, label: String) -> TemplateLabel` — Creates a new template label for testing purposes.
- pub `create_test_template_annotation` function L482-494 — `( &self, template_id: Uuid, key: &str, value: &str, ) -> TemplateAnnotation` — Creates a new template annotation for testing purposes.
- pub `create_test_work_order` function L506-519 — `(&self, work_type: &str, yaml_content: &str) -> WorkOrder` — Creates a new work order for testing purposes.
- pub `create_test_work_order_target` function L531-542 — `( &self, work_order_id: Uuid, agent_id: Uuid, ) -> WorkOrderTarget` — Creates a new work order target for testing purposes.
- pub `create_test_work_order_label` function L554-565 — `( &self, work_order_id: Uuid, label: &str, ) -> WorkOrderLabel` — Creates a new work order label for testing purposes.
- pub `create_test_work_order_annotation` function L578-590 — `( &self, work_order_id: Uuid, key: &str, value: &str, ) -> WorkOrderAnnotation` — Creates a new work order annotation for testing purposes.
-  `TestFixture` type L57-61 — `impl Default for TestFixture` — and agent events.
-  `default` function L58-60 — `() -> Self` — and agent events.
-  `TestFixture` type L63-602 — `= TestFixture` — and agent events.
-  `reset_database` function L592-601 — `(&self)` — and agent events.
-  `TestFixture` type L604-608 — `impl Drop for TestFixture` — and agent events.
-  `drop` function L605-607 — `(&mut self)` — and agent events.

### crates/brokkr-broker/tests/integration/api

> *Semantic summary to be generated by AI agent.*

#### crates/brokkr-broker/tests/integration/api/admin.rs

-  `test_config_reload_requires_auth` function L19-39 — `()` — Test that the config reload endpoint requires authentication.
-  `test_config_reload_requires_admin` function L43-70 — `()` — Test that non-admin users cannot access config reload.
-  `test_config_reload_success_with_admin` function L74-106 — `()` — Test that admin users can successfully reload configuration.
-  `test_config_reload_no_changes` function L110-140 — `()` — Test that config reload returns no changes when config hasn't changed.
-  `test_config_reload_denied_for_generator` function L144-171 — `()` — Test that generator PAK cannot access config reload (admin only).

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
-  `test_update_agent` function L108-163 — `()`
-  `test_delete_agent` function L166-187 — `()`
-  `test_list_agent_events` function L190-241 — `()`
-  `test_create_agent_event` function L244-288 — `()`
-  `test_list_agent_labels` function L291-327 — `()`
-  `test_add_agent_label` function L330-360 — `()`
-  `test_remove_agent_label` function L363-396 — `()`
-  `test_list_agent_annotations` function L399-440 — `()`
-  `test_add_agent_annotation` function L443-478 — `()`
-  `test_remove_agent_annotation` function L481-518 — `()`
-  `test_list_agent_targets` function L521-566 — `()`
-  `test_add_agent_target` function L569-608 — `()`
-  `test_remove_agent_target` function L611-653 — `()`
-  `test_unauthorized_list_agent_events` function L656-671 — `()`
-  `test_unauthorized_create_agent_event` function L674-698 — `()`
-  `test_unauthorized_list_agent_labels` function L701-716 — `()`
-  `test_unauthorized_add_agent_label` function L719-737 — `()`
-  `test_unauthorized_create_agent` function L740-756 — `()`
-  `test_unauthorized_get_agent` function L759-774 — `()`
-  `test_unauthorized_update_agent` function L777-797 — `()`
-  `test_unauthorized_delete_agent` function L800-815 — `()`
-  `test_get_agent_with_mismatched_pak` function L818-840 — `()`
-  `test_update_agent_with_mismatched_pak` function L843-869 — `()`
-  `test_create_agent_event_with_mismatched_pak` function L872-904 — `()`
-  `test_list_agent_labels_with_mismatched_pak` function L907-929 — `()`
-  `test_record_heartbeat` function L932-957 — `()`
-  `test_get_target_state_incremental` function L960-1018 — `()`
-  `test_get_target_state_full` function L1021-1090 — `()`
-  `test_get_target_state_with_invalid_mode` function L1093-1144 — `()`
-  `test_get_agent_by_name_and_cluster_name` function L1147-1176 — `()`
-  `test_get_agent_stacks` function L1179-1307 — `()`
-  `test_rotate_agent_pak_admin_success` function L1310-1348 — `()`
-  `test_rotate_agent_pak_self_success` function L1351-1384 — `()`
-  `test_rotate_agent_pak_unauthorized` function L1387-1406 — `()`
-  `test_rotate_agent_pak_forbidden` function L1409-1433 — `()`
-  `test_get_target_state_with_mismatched_auth` function L1436-1474 — `()`

#### crates/brokkr-broker/tests/integration/api/audit_logs.rs

-  `test_audit_logs_requires_auth` function L19-38 — `()` — Test that the audit logs endpoint requires authentication.
-  `test_audit_logs_requires_admin` function L42-68 — `()` — Test that non-admin users cannot access audit logs.
-  `test_audit_logs_success_with_admin` function L72-104 — `()` — Test that admin users can access audit logs.
-  `test_audit_logs_pagination` function L108-137 — `()` — Test audit logs with pagination parameters.
-  `test_audit_logs_filtering` function L141-169 — `()` — Test audit logs with filter parameters.
-  `test_audit_logs_denied_for_generator` function L173-199 — `()` — Test that generator PAK cannot access audit logs (admin only).

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

-  `test_create_diagnostic_request` function L17-71 — `()`
-  `test_create_diagnostic_request_unauthorized` function L74-115 — `()`
-  `test_get_pending_diagnostics` function L118-163 — `()`
-  `test_get_pending_diagnostics_unauthorized` function L166-194 — `()`
-  `test_claim_diagnostic` function L197-242 — `()`
-  `test_claim_already_claimed` function L245-285 — `()`
-  `test_submit_diagnostic_result` function L288-351 — `()`
-  `test_submit_result_not_claimed` function L354-400 — `()`
-  `test_get_diagnostic_with_result` function L403-467 — `()`
-  `test_get_diagnostic_not_found` function L470-489 — `()`

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

#### crates/brokkr-broker/tests/integration/api/stacks.rs

-  `test_create_stack` function L23-62 — `()`
-  `test_get_stack` function L65-96 — `()`
-  `test_list_stacks` function L99-130 — `()`
-  `test_update_stack` function L133-171 — `()`
-  `test_soft_delete_stack` function L174-216 — `()`
-  `test_add_stack_annotation` function L219-257 — `()`
-  `test_remove_stack_annotation` function L260-286 — `()`
-  `test_list_stack_annotations` function L289-321 — `()`
-  `test_add_stack_label` function L324-364 — `()`
-  `test_remove_stack_label` function L367-393 — `()`
-  `test_list_stack_labels` function L396-428 — `()`
-  `test_create_deployment_object` function L431-469 — `()`
-  `test_create_stack_with_generator_pak` function L472-510 — `()`
-  `test_create_stack_with_wrong_generator_pak` function L513-556 — `()`
-  `test_update_stack_with_wrong_generator_pak` function L559-603 — `()`
-  `test_delete_stack_with_wrong_generator_pak` function L606-643 — `()`
-  `test_add_stack_annotation_with_wrong_generator_pak` function L646-690 — `()`

#### crates/brokkr-broker/tests/integration/api/templates.rs

-  `TEST_TEMPLATE_CONTENT` variable L16-21 — `: &str`
-  `TEST_PARAMETERS_SCHEMA` variable L23-30 — `: &str`
-  `test_create_template` function L33-68 — `()`
-  `test_create_template_with_generator_pak` function L71-105 — `()`
-  `test_create_template_invalid_tera_syntax` function L108-133 — `()`
-  `test_get_template` function L136-168 — `()`
-  `test_list_templates` function L171-209 — `()`
-  `test_update_template_creates_new_version` function L212-254 — `()`
-  `test_delete_template` function L257-299 — `()`
-  `test_add_template_label` function L302-336 — `()`
-  `test_list_template_labels` function L339-373 — `()`
-  `test_remove_template_label` function L376-404 — `()`
-  `test_add_template_annotation` function L407-445 — `()`
-  `test_list_template_annotations` function L448-482 — `()`
-  `test_remove_template_annotation` function L485-516 — `()`
-  `test_instantiate_template` function L519-571 — `()`
-  `test_instantiate_template_invalid_parameters` function L574-622 — `()`
-  `test_instantiate_template_label_mismatch` function L625-668 — `()`
-  `test_instantiate_template_with_matching_labels` function L671-713 — `()`
-  `test_generator_cannot_access_other_generator_template` function L716-748 — `()`

#### crates/brokkr-broker/tests/integration/api/webhooks.rs

-  `test_list_webhooks_admin_success` function L20-41 — `()`
-  `test_list_webhooks_non_admin_forbidden` function L44-63 — `()`
-  `test_list_webhooks_unauthorized` function L66-82 — `()`
-  `test_create_webhook_admin_success` function L89-123 — `()`
-  `test_create_webhook_with_wildcard_events` function L126-151 — `()`
-  `test_create_webhook_invalid_url` function L154-179 — `()`
-  `test_create_webhook_non_admin_forbidden` function L182-208 — `()`
-  `test_get_webhook_admin_success` function L215-253 — `()`
-  `test_get_webhook_not_found` function L256-275 — `()`
-  `test_update_webhook_admin_success` function L282-327 — `()`
-  `test_delete_webhook_admin_success` function L334-373 — `()`
-  `test_delete_webhook_not_found` function L376-395 — `()`
-  `test_list_event_types_admin_success` function L402-428 — `()`
-  `test_list_deliveries_admin_success` function L435-473 — `()`
-  `test_list_deliveries_with_status_filter` function L476-521 — `()`
-  `test_list_deliveries_subscription_not_found` function L524-543 — `()`

#### crates/brokkr-broker/tests/integration/api/work_orders.rs

-  `make_request` function L22-51 — `( app: Router, method: &str, uri: &str, auth: Option<&str>, body: Option<String>...`
-  `test_create_work_order` function L58-85 — `()`
-  `test_create_work_order_empty_targets` function L88-110 — `()`
-  `test_create_work_order_unauthorized` function L113-135 — `()`
-  `test_create_work_order_forbidden_non_admin` function L138-161 — `()`
-  `test_list_work_orders` function L164-179 — `()`
-  `test_list_work_orders_filtered` function L182-204 — `()`
-  `test_get_work_order` function L207-227 — `()`
-  `test_get_work_order_not_found` function L230-245 — `()`
-  `test_delete_work_order` function L248-269 — `()`
-  `test_list_pending_for_agent` function L276-303 — `()`
-  `test_list_pending_for_agent_admin` function L306-326 — `()`
-  `test_list_pending_for_other_agent_forbidden` function L329-347 — `()`
-  `test_claim_work_order` function L350-378 — `()`
-  `test_claim_work_order_not_targeted` function L381-405 — `()`
-  `test_complete_work_order_success` function L408-444 — `()`
-  `test_complete_work_order_failure_with_retry` function L447-493 — `()`
-  `test_complete_work_order_failure_max_retries` function L496-542 — `()`
-  `test_complete_work_order_wrong_agent` function L545-579 — `()`
-  `test_list_work_order_log` function L586-617 — `()`
-  `test_get_work_order_log` function L620-649 — `()`
-  `test_get_work_order_log_not_found` function L652-667 — `()`
-  `test_list_work_order_log_forbidden` function L670-687 — `()`
-  `test_create_work_order_with_labels` function L694-732 — `()`
-  `test_create_work_order_with_annotations` function L735-773 — `()`
-  `test_create_work_order_with_combined_targeting` function L776-828 — `()`
-  `test_create_work_order_no_targeting_fails` function L831-858 — `()`
-  `test_create_work_order_empty_targeting_fails` function L861-889 — `()`
-  `test_create_work_order_legacy_target_agent_ids` function L892-917 — `()`
-  `test_list_pending_with_label_targeting` function L920-948 — `()`
-  `test_list_pending_with_annotation_targeting` function L951-979 — `()`
-  `test_claim_with_label_targeting` function L982-1014 — `()`
-  `test_claim_with_annotation_targeting` function L1017-1049 — `()`
-  `test_claim_with_no_matching_targeting` function L1052-1081 — `()`

### crates/brokkr-broker/tests/integration/dal

> *Semantic summary to be generated by AI agent.*

#### crates/brokkr-broker/tests/integration/dal/agent_annotations.rs

-  `test_create_agent_annotation` function L11-28 — `()`
-  `test_get_agent_annotation` function L31-48 — `()`
-  `test_list_agent_annotations` function L51-70 — `()`
-  `test_update_agent_annotation` function L73-89 — `()`
-  `test_delete_agent_annotation` function L92-112 — `()`
-  `test_delete_all_agent_annotations` function L115-135 — `()`

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
-  `test_get_agent_details` function L493-572 — `()`
-  `test_record_heartbeat` function L575-625 — `()`
-  `test_update_agent_pak_hash` function L628-650 — `()`
-  `test_get_agent_by_name_and_cluster_name` function L653-680 — `()`
-  `test_recreate_agent_after_soft_delete` function L683-728 — `()`

#### crates/brokkr-broker/tests/integration/dal/deployment_health.rs

-  `test_upsert_deployment_health` function L12-67 — `()`
-  `test_upsert_batch_deployment_health` function L70-133 — `()`
-  `test_get_deployment_health_by_agent_and_deployment` function L136-180 — `()`
-  `test_list_deployment_health_by_agent` function L183-219 — `()`
-  `test_list_deployment_health_by_stack` function L222-261 — `()`
-  `test_list_deployment_health_by_status` function L264-304 — `()`
-  `test_delete_deployment_health` function L307-353 — `()`
-  `test_delete_deployment_health_by_agent` function L356-402 — `()`

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
-  `test_target_state_annotation_targeting_after_deployment_exists` function L820-865 — `()` — Test that annotation targeting works when deployment exists first.

#### crates/brokkr-broker/tests/integration/dal/diagnostic_requests.rs

-  `test_create_diagnostic_request` function L12-42 — `()`
-  `test_get_diagnostic_request` function L45-73 — `()`
-  `test_get_pending_for_agent` function L76-104 — `()`
-  `test_claim_diagnostic_request` function L107-138 — `()`
-  `test_complete_diagnostic_request` function L141-165 — `()`
-  `test_fail_diagnostic_request` function L168-192 — `()`
-  `test_list_by_deployment_object` function L195-219 — `()`
-  `test_expire_old_requests` function L222-262 — `()`
-  `test_cleanup_old_requests` function L265-298 — `()`
-  `test_delete_diagnostic_request` function L301-330 — `()`

#### crates/brokkr-broker/tests/integration/dal/diagnostic_results.rs

-  `test_create_diagnostic_result` function L13-48 — `()`
-  `test_get_diagnostic_result` function L51-85 — `()`
-  `test_get_diagnostic_result_by_request` function L88-131 — `()`
-  `test_delete_diagnostic_result` function L134-173 — `()`
-  `test_delete_diagnostic_result_by_request` function L176-215 — `()`
-  `test_cascade_delete_on_request_deletion` function L218-257 — `()`

#### crates/brokkr-broker/tests/integration/dal/event_emission.rs

-  `create_subscription_for_event` function L16-29 — `(name: &str, event_type: &str) -> NewWebhookSubscription` — webhook events and create corresponding delivery records.
-  `create_disabled_subscription` function L31-44 — `(name: &str, event_type: &str) -> NewWebhookSubscription` — webhook events and create corresponding delivery records.
-  `create_subscription_with_target_labels` function L46-63 — `( name: &str, event_type: &str, labels: Vec<String>, ) -> NewWebhookSubscription` — webhook events and create corresponding delivery records.
-  `create_subscription_with_agent_filter` function L65-83 — `( name: &str, event_type: &str, agent_id: uuid::Uuid, ) -> NewWebhookSubscriptio...` — webhook events and create corresponding delivery records.
-  `test_work_order_completion_emits_event` function L90-140 — `()` — webhook events and create corresponding delivery records.
-  `test_wildcard_subscription_matches_events` function L143-185 — `()` — webhook events and create corresponding delivery records.
-  `test_disabled_subscription_receives_no_deliveries` function L188-228 — `()` — webhook events and create corresponding delivery records.
-  `test_delivery_inherits_target_labels_from_subscription` function L231-282 — `()` — webhook events and create corresponding delivery records.
-  `test_no_delivery_when_no_matching_subscription` function L285-327 — `()` — webhook events and create corresponding delivery records.
-  `test_multiple_subscriptions_receive_same_event` function L330-383 — `()` — webhook events and create corresponding delivery records.

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
-  `test_recreate_generator_after_soft_delete` function L261-319 — `()`

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
-  `test_list_annotations_for_stack` function L61-85 — `()`
-  `test_update_stack_annotation` function L88-110 — `()`
-  `test_delete_stack_annotation` function L113-136 — `()`
-  `test_delete_all_annotations_for_stack` function L139-163 — `()`

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
-  `test_recreate_stack_after_soft_delete` function L584-634 — `()`

#### crates/brokkr-broker/tests/integration/dal/templates.rs

-  `TEST_TEMPLATE_CONTENT` variable L9-12 — `: &str`
-  `test_create_template` function L15-33 — `()`
-  `test_create_template_with_generator` function L36-58 — `()`
-  `test_get_template` function L61-81 — `()`
-  `test_list_templates` function L84-109 — `()`
-  `test_list_templates_by_generator` function L112-144 — `()`
-  `test_versioning` function L147-180 — `()`
-  `test_get_latest_version` function L183-211 — `()`
-  `test_list_versions` function L214-239 — `()`
-  `test_soft_delete` function L242-267 — `()`
-  `test_template_labels` function L270-292 — `()`
-  `test_template_annotations` function L295-317 — `()`
-  `test_delete_label` function L320-346 — `()`
-  `test_delete_annotation` function L349-375 — `()`
-  `test_checksum_generation` function L378-392 — `()`
-  `test_same_content_same_checksum` function L395-415 — `()`
-  `test_recreate_template_after_soft_delete` function L418-473 — `()`

#### crates/brokkr-broker/tests/integration/dal/webhook_deliveries.rs

-  `create_test_subscription` function L11-24 — `(name: &str) -> NewWebhookSubscription`
-  `create_test_subscription_with_labels` function L26-39 — `(name: &str, labels: Vec<String>) -> NewWebhookSubscription`
-  `create_test_event` function L41-50 — `() -> BrokkrEvent`
-  `test_create_delivery` function L53-80 — `()`
-  `test_create_delivery_with_target_labels` function L83-101 — `()`
-  `test_get_delivery` function L104-123 — `()`
-  `test_claim_for_broker` function L126-152 — `()`
-  `test_claim_for_agent_with_matching_labels` function L155-184 — `()`
-  `test_claim_for_agent_without_matching_labels` function L187-214 — `()`
-  `test_release_expired` function L217-259 — `()`
-  `test_mark_success` function L262-282 — `()`
-  `test_mark_failed_with_retry` function L285-306 — `()`
-  `test_process_retries` function L309-339 — `()`
-  `test_mark_failed_max_retries_exceeded` function L342-362 — `()`
-  `test_list_for_subscription` function L365-407 — `()`
-  `test_cleanup_old_deliveries` function L410-448 — `()`
-  `test_claim_pagination` function L451-481 — `()`
-  `test_retry_failed_delivery` function L484-507 — `()`
-  `test_get_stats` function L510-542 — `()`
-  `test_exponential_backoff_timing` function L549-610 — `()`
-  `test_claim_requires_all_labels` function L617-663 — `()`
-  `test_empty_target_labels_matches_broker` function L666-695 — `()`
-  `test_valid_acquired_until_stays_acquired` function L702-730 — `()`
-  `test_released_delivery_claimable_by_different_agent` function L733-769 — `()`

#### crates/brokkr-broker/tests/integration/dal/webhook_subscriptions.rs

-  `create_test_subscription` function L10-23 — `(name: &str, event_types: Vec<&str>) -> NewWebhookSubscription`
-  `create_test_subscription_with_labels` function L25-38 — `(name: &str, event_types: Vec<&str>, labels: Vec<String>) -> NewWebhookSubscript...`
-  `test_create_subscription` function L41-56 — `()`
-  `test_create_subscription_with_target_labels` function L59-77 — `()`
-  `test_get_subscription` function L80-99 — `()`
-  `test_list_subscriptions` function L102-119 — `()`
-  `test_list_enabled_only` function L122-140 — `()`
-  `test_update_subscription` function L143-173 — `()`
-  `test_update_subscription_target_labels` function L176-208 — `()`
-  `test_delete_subscription` function L211-236 — `()`
-  `test_get_matching_subscriptions_exact` function L239-265 — `()`
-  `test_get_matching_subscriptions_wildcard` function L268-296 — `()`
-  `test_get_matching_subscriptions_star_wildcard` function L299-323 — `()`
-  `test_disabled_subscriptions_not_matched` function L326-342 — `()`

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
-  `test_complete_failure_non_retryable` function L483-540 — `()`
-  `test_process_retry_pending` function L547-604 — `()`
-  `test_add_target` function L611-621 — `()`
-  `test_add_targets_batch` function L624-647 — `()`
-  `test_list_targets` function L650-667 — `()`
-  `test_remove_target` function L670-693 — `()`
-  `test_get_log` function L700-728 — `()`
-  `test_list_log` function L731-770 — `()`
-  `test_list_log_filtered` function L773-837 — `()`
-  `test_list_log_with_limit` function L840-868 — `()`
-  `test_add_label` function L875-883 — `()`
-  `test_add_multiple_labels` function L886-907 — `()`
-  `test_remove_label` function L910-931 — `()`
-  `test_add_annotation` function L934-943 — `()`
-  `test_add_multiple_annotations` function L946-969 — `()`
-  `test_remove_annotation` function L972-993 — `()`
-  `test_list_pending_for_agent_with_label_match` function L996-1016 — `()`
-  `test_list_pending_for_agent_with_annotation_match` function L1019-1039 — `()`
-  `test_list_pending_for_agent_no_match` function L1042-1061 — `()`
-  `test_list_pending_for_agent_or_logic` function L1064-1084 — `()`
-  `test_list_pending_for_agent_combined_targeting` function L1087-1118 — `()`
-  `test_claim_with_label_match` function L1121-1141 — `()`
-  `test_claim_with_annotation_match` function L1144-1164 — `()`
-  `test_claim_without_authorization` function L1167-1186 — `()`
-  `test_annotation_key_value_must_both_match` function L1189-1208 — `()`
-  `test_labels_deleted_on_work_order_delete` function L1211-1249 — `()`

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

### crates/brokkr-broker/tests/integration

> *Semantic summary to be generated by AI agent.*

#### crates/brokkr-broker/tests/integration/main.rs

-  `api` module L7 — `-`
-  `dal` module L8 — `-`
-  `db` module L9 — `-`
-  `fixtures` module L11 — `-`

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
- pub `new` function L115-139 — `( stack_id: Uuid, yaml_content: String, is_deletion_marker: bool, ) -> Result<Se...` — Creates a new `NewDeploymentObject` instance.
-  `NewDeploymentObject` type L101-140 — `= NewDeploymentObject` — - Deployment objects are designed to be immutable after creation, with exceptions for soft deletion.
-  `generate_checksum` function L143-148 — `(content: &str) -> String` — Helper function to generate SHA-256 checksum for YAML content.
-  `tests` module L151-199 — `-` — - Deployment objects are designed to be immutable after creation, with exceptions for soft deletion.
-  `test_new_deployment_object_success` function L155-170 — `()` — - Deployment objects are designed to be immutable after creation, with exceptions for soft deletion.
-  `test_new_deployment_object_invalid_stack_id` function L173-184 — `()` — - Deployment objects are designed to be immutable after creation, with exceptions for soft deletion.
-  `test_new_deployment_object_empty_yaml` function L187-198 — `()` — - Deployment objects are designed to be immutable after creation, with exceptions for soft deletion.

#### crates/brokkr-models/src/models/diagnostic_requests.rs

- pub `VALID_STATUSES` variable L20 — `: &[&str]` — Valid diagnostic request statuses
- pub `DiagnosticRequest` struct L27-46 — `{ id: Uuid, agent_id: Uuid, deployment_object_id: Uuid, status: String, requeste...` — information from agents about specific deployment objects.
- pub `NewDiagnosticRequest` struct L51-62 — `{ agent_id: Uuid, deployment_object_id: Uuid, status: String, requested_by: Opti...` — A new diagnostic request to be inserted.
- pub `new` function L75-103 — `( agent_id: Uuid, deployment_object_id: Uuid, requested_by: Option<String>, rete...` — Creates a new diagnostic request.
- pub `UpdateDiagnosticRequest` struct L109-116 — `{ status: Option<String>, claimed_at: Option<DateTime<Utc>>, completed_at: Optio...` — Changeset for updating a diagnostic request.
-  `NewDiagnosticRequest` type L64-104 — `= NewDiagnosticRequest` — information from agents about specific deployment objects.
-  `tests` module L119-201 — `-` — information from agents about specific deployment objects.
-  `test_new_diagnostic_request_success` function L123-141 — `()` — information from agents about specific deployment objects.
-  `test_new_diagnostic_request_nil_agent_id` function L144-154 — `()` — information from agents about specific deployment objects.
-  `test_new_diagnostic_request_nil_deployment_object_id` function L157-167 — `()` — information from agents about specific deployment objects.
-  `test_new_diagnostic_request_invalid_retention` function L170-180 — `()` — information from agents about specific deployment objects.
-  `test_new_diagnostic_request_default_retention` function L183-200 — `()` — information from agents about specific deployment objects.

#### crates/brokkr-models/src/models/diagnostic_results.rs

- pub `DiagnosticResult` struct L24-39 — `{ id: Uuid, request_id: Uuid, pod_statuses: String, events: String, log_tails: O...` — collected by agents in response to diagnostic requests.
- pub `NewDiagnosticResult` struct L44-55 — `{ request_id: Uuid, pod_statuses: String, events: String, log_tails: Option<Stri...` — A new diagnostic result to be inserted.
- pub `new` function L69-98 — `( request_id: Uuid, pod_statuses: String, events: String, log_tails: Option<Stri...` — Creates a new diagnostic result.
-  `NewDiagnosticResult` type L57-99 — `= NewDiagnosticResult` — collected by agents in response to diagnostic requests.
-  `tests` module L102-185 — `-` — collected by agents in response to diagnostic requests.
-  `test_new_diagnostic_result_success` function L106-125 — `()` — collected by agents in response to diagnostic requests.
-  `test_new_diagnostic_result_nil_request_id` function L128-139 — `()` — collected by agents in response to diagnostic requests.
-  `test_new_diagnostic_result_empty_pod_statuses` function L142-153 — `()` — collected by agents in response to diagnostic requests.
-  `test_new_diagnostic_result_empty_events` function L156-167 — `()` — collected by agents in response to diagnostic requests.
-  `test_new_diagnostic_result_no_log_tails` function L170-184 — `()` — collected by agents in response to diagnostic requests.

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
- pub `agent_labels` module L9 — `-`
- pub `agent_targets` module L10 — `-`
- pub `agents` module L11 — `-`
- pub `audit_logs` module L12 — `-`
- pub `deployment_health` module L13 — `-`
- pub `deployment_objects` module L14 — `-`
- pub `diagnostic_requests` module L15 — `-`
- pub `diagnostic_results` module L16 — `-`
- pub `generator` module L17 — `-`
- pub `rendered_deployment_objects` module L18 — `-`
- pub `stack_annotations` module L19 — `-`
- pub `stack_labels` module L20 — `-`
- pub `stack_templates` module L21 — `-`
- pub `stacks` module L22 — `-`
- pub `template_annotations` module L23 — `-`
- pub `template_labels` module L24 — `-`
- pub `template_targets` module L25 — `-`
- pub `webhooks` module L26 — `-`
- pub `work_order_annotations` module L27 — `-`
- pub `work_order_labels` module L28 — `-`
- pub `work_orders` module L29 — `-`

#### crates/brokkr-models/src/models/rendered_deployment_objects.rs

- pub `RenderedDeploymentObject` struct L66-79 — `{ id: Uuid, deployment_object_id: Uuid, template_id: Uuid, template_version: i32...` — - `template_parameters` must be a valid JSON string.
- pub `NewRenderedDeploymentObject` struct L84-93 — `{ deployment_object_id: Uuid, template_id: Uuid, template_version: i32, template...` — Represents a new rendered deployment object provenance record to be inserted.
- pub `new` function L109-141 — `( deployment_object_id: Uuid, template_id: Uuid, template_version: i32, template...` — Creates a new `NewRenderedDeploymentObject` instance.
-  `NewRenderedDeploymentObject` type L95-142 — `= NewRenderedDeploymentObject` — - `template_parameters` must be a valid JSON string.
-  `tests` module L145-234 — `-` — - `template_parameters` must be a valid JSON string.
-  `test_new_rendered_deployment_object_success` function L149-171 — `()` — - `template_parameters` must be a valid JSON string.
-  `test_new_rendered_deployment_object_invalid_deployment_object_id` function L174-186 — `()` — - `template_parameters` must be a valid JSON string.
-  `test_new_rendered_deployment_object_invalid_template_id` function L189-198 — `()` — - `template_parameters` must be a valid JSON string.
-  `test_new_rendered_deployment_object_invalid_version` function L201-210 — `()` — - `template_parameters` must be a valid JSON string.
-  `test_new_rendered_deployment_object_invalid_json` function L213-222 — `()` — - `template_parameters` must be a valid JSON string.
-  `test_new_rendered_deployment_object_empty_json_object` function L225-233 — `()` — - `template_parameters` must be a valid JSON string.

#### crates/brokkr-models/src/models/stack_annotations.rs

- pub `StackAnnotation` struct L54-63 — `{ id: Uuid, stack_id: Uuid, key: String, value: String }` — - Neither `key` nor `value` can contain whitespace.
- pub `NewStackAnnotation` struct L68-75 — `{ stack_id: Uuid, key: String, value: String }` — Represents a new stack annotation to be inserted into the database.
- pub `new` function L90-123 — `(stack_id: Uuid, key: String, value: String) -> Result<Self, String>` — Creates a new `NewStackAnnotation` instance.
-  `NewStackAnnotation` type L77-124 — `= NewStackAnnotation` — - Neither `key` nor `value` can contain whitespace.
-  `tests` module L127-263 — `-` — - Neither `key` nor `value` can contain whitespace.
-  `test_new_stack_annotation_success` function L131-152 — `()` — - Neither `key` nor `value` can contain whitespace.
-  `test_new_stack_annotation_invalid_stack_id` function L155-170 — `()` — - Neither `key` nor `value` can contain whitespace.
-  `test_new_stack_annotation_empty_key` function L173-185 — `()` — - Neither `key` nor `value` can contain whitespace.
-  `test_new_stack_annotation_empty_value` function L188-200 — `()` — - Neither `key` nor `value` can contain whitespace.
-  `test_new_stack_annotation_key_too_long` function L203-215 — `()` — - Neither `key` nor `value` can contain whitespace.
-  `test_new_stack_annotation_value_too_long` function L218-230 — `()` — - Neither `key` nor `value` can contain whitespace.
-  `test_new_stack_annotation_key_with_whitespace` function L233-246 — `()` — - Neither `key` nor `value` can contain whitespace.
-  `test_new_stack_annotation_value_with_whitespace` function L249-262 — `()` — - Neither `key` nor `value` can contain whitespace.

#### crates/brokkr-models/src/models/stack_labels.rs

- pub `StackLabel` struct L53-60 — `{ id: Uuid, stack_id: Uuid, label: String }` — - The `label` cannot contain whitespace.
- pub `NewStackLabel` struct L65-70 — `{ stack_id: Uuid, label: String }` — Represents a new stack label to be inserted into the database.
- pub `new` function L84-106 — `(stack_id: Uuid, label: String) -> Result<Self, String>` — Creates a new `NewStackLabel` instance.
-  `NewStackLabel` type L72-107 — `= NewStackLabel` — - The `label` cannot contain whitespace.
-  `tests` module L110-198 — `-` — - The `label` cannot contain whitespace.
-  `test_new_stack_label_success` function L114-130 — `()` — - The `label` cannot contain whitespace.
-  `test_new_stack_label_invalid_stack_id` function L133-144 — `()` — - The `label` cannot contain whitespace.
-  `test_new_stack_label_empty_label` function L147-158 — `()` — - The `label` cannot contain whitespace.
-  `test_new_stack_label_whitespace_label` function L161-172 — `()` — - The `label` cannot contain whitespace.
-  `test_new_stack_label_too_long` function L175-187 — `()` — - The `label` cannot contain whitespace.
-  `test_new_stack_label_max_length` function L190-197 — `()` — - The `label` cannot contain whitespace.

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
-  `tests` module L116-201 — `-` — - The `value` must be a non-empty string, max 64 characters, no whitespace.
-  `test_new_template_annotation_success` function L120-132 — `()` — - The `value` must be a non-empty string, max 64 characters, no whitespace.
-  `test_new_template_annotation_invalid_template_id` function L135-139 — `()` — - The `value` must be a non-empty string, max 64 characters, no whitespace.
-  `test_new_template_annotation_empty_key` function L142-147 — `()` — - The `value` must be a non-empty string, max 64 characters, no whitespace.
-  `test_new_template_annotation_empty_value` function L150-154 — `()` — - The `value` must be a non-empty string, max 64 characters, no whitespace.
-  `test_new_template_annotation_key_with_whitespace` function L157-165 — `()` — - The `value` must be a non-empty string, max 64 characters, no whitespace.
-  `test_new_template_annotation_value_with_whitespace` function L168-176 — `()` — - The `value` must be a non-empty string, max 64 characters, no whitespace.
-  `test_new_template_annotation_key_too_long` function L179-188 — `()` — - The `value` must be a non-empty string, max 64 characters, no whitespace.
-  `test_new_template_annotation_value_too_long` function L191-200 — `()` — - The `value` must be a non-empty string, max 64 characters, no whitespace.

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
-  `tests` module L387-551 — `-` — enabling external systems to receive notifications when events occur in Brokkr.
-  `test_brokkr_event_new` function L391-398 — `()` — enabling external systems to receive notifications when events occur in Brokkr.
-  `test_new_webhook_subscription_success` function L401-418 — `()` — enabling external systems to receive notifications when events occur in Brokkr.
-  `test_new_webhook_subscription_with_target_labels` function L421-436 — `()` — enabling external systems to receive notifications when events occur in Brokkr.
-  `test_new_webhook_subscription_empty_name` function L439-452 — `()` — enabling external systems to receive notifications when events occur in Brokkr.
-  `test_new_webhook_subscription_no_event_types` function L455-468 — `()` — enabling external systems to receive notifications when events occur in Brokkr.
-  `test_new_webhook_delivery_success` function L471-484 — `()` — enabling external systems to receive notifications when events occur in Brokkr.
-  `test_new_webhook_delivery_with_target_labels` function L487-498 — `()` — enabling external systems to receive notifications when events occur in Brokkr.
-  `test_new_webhook_delivery_nil_subscription` function L501-507 — `()` — enabling external systems to receive notifications when events occur in Brokkr.
-  `test_webhook_filters_serialization` function L510-524 — `()` — enabling external systems to receive notifications when events occur in Brokkr.
-  `test_valid_event_types` function L527-541 — `()` — enabling external systems to receive notifications when events occur in Brokkr.
-  `test_valid_delivery_statuses` function L544-550 — `()` — enabling external systems to receive notifications when events occur in Brokkr.

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
- pub `from_work_order` function L306-318 — `(work_order: &WorkOrder, success: bool, result_message: Option<String>) -> Self` — Creates a new log entry from a completed work order.
- pub `WorkOrderTarget` struct L345-358 — `{ id: Uuid, work_order_id: Uuid, agent_id: Uuid, created_at: DateTime<Utc> }` — On completion (success or max retries exceeded), records move to `work_order_log`.
- pub `NewWorkOrderTarget` struct L363-368 — `{ work_order_id: Uuid, agent_id: Uuid }` — Represents a new work order target to be inserted.
- pub `new` function L372-383 — `(work_order_id: Uuid, agent_id: Uuid) -> Result<Self, String>` — Creates a new work order target.
-  `default_max_retries` function L150-152 — `() -> i32` — On completion (success or max retries exceeded), records move to `work_order_log`.
-  `default_backoff_seconds` function L154-156 — `() -> i32` — On completion (success or max retries exceeded), records move to `work_order_log`.
-  `default_claim_timeout_seconds` function L158-160 — `() -> i32` — On completion (success or max retries exceeded), records move to `work_order_log`.
-  `NewWorkOrder` type L162-217 — `= NewWorkOrder` — On completion (success or max retries exceeded), records move to `work_order_log`.
-  `NewWorkOrderLog` type L304-319 — `= NewWorkOrderLog` — On completion (success or max retries exceeded), records move to `work_order_log`.
-  `NewWorkOrderTarget` type L370-384 — `= NewWorkOrderTarget` — On completion (success or max retries exceeded), records move to `work_order_log`.
-  `tests` module L387-462 — `-` — On completion (success or max retries exceeded), records move to `work_order_log`.
-  `test_new_work_order_success` function L391-405 — `()` — On completion (success or max retries exceeded), records move to `work_order_log`.
-  `test_new_work_order_empty_work_type` function L408-418 — `()` — On completion (success or max retries exceeded), records move to `work_order_log`.
-  `test_new_work_order_empty_yaml` function L421-431 — `()` — On completion (success or max retries exceeded), records move to `work_order_log`.
-  `test_new_work_order_invalid_max_retries` function L434-444 — `()` — On completion (success or max retries exceeded), records move to `work_order_log`.
-  `test_new_work_order_target_success` function L447-450 — `()` — On completion (success or max retries exceeded), records move to `work_order_log`.
-  `test_new_work_order_target_invalid_ids` function L453-461 — `()` — On completion (success or max retries exceeded), records move to `work_order_log`.

### crates/brokkr-utils/src

> *Semantic summary to be generated by AI agent.*

#### crates/brokkr-utils/src/config.rs

- pub `Settings` struct L118-133 — `{ database: Database, log: Log, pak: PAK, agent: Agent, broker: Broker, cors: Co...` — Represents the main settings structure for the application
- pub `Cors` struct L137-153 — `{ allowed_origins: Vec<String>, allowed_methods: Vec<String>, allowed_headers: V...` — Represents the CORS configuration
- pub `Broker` struct L156-174 — `{ pak_hash: Option<String>, diagnostic_cleanup_interval_seconds: Option<u64>, di...` — Default: "BR"
- pub `Agent` struct L179-204 — `{ broker_url: String, polling_interval: u64, kubeconfig_path: Option<String>, ma...` — Represents the agent configuration
- pub `Database` struct L209-214 — `{ url: String, schema: Option<String> }` — Represents the database configuration
- pub `Log` struct L218-224 — `{ level: String, format: String }` — Represents the logging configuration
- pub `Telemetry` struct L232-251 — `{ enabled: bool, otlp_endpoint: String, service_name: String, sampling_rate: f64...` — Represents the telemetry (OpenTelemetry) configuration with hierarchical overrides
- pub `TelemetryOverride` struct L255-264 — `{ enabled: Option<bool>, otlp_endpoint: Option<String>, service_name: Option<Str...` — Component-specific telemetry overrides (all fields optional)
- pub `ResolvedTelemetry` struct L268-273 — `{ enabled: bool, otlp_endpoint: String, service_name: String, sampling_rate: f64...` — Resolved telemetry configuration after merging base with overrides
- pub `for_broker` function L277-292 — `(&self) -> ResolvedTelemetry` — Get resolved telemetry config for broker (base merged with broker overrides)
- pub `for_agent` function L295-310 — `(&self) -> ResolvedTelemetry` — Get resolved telemetry config for agent (base merged with agent overrides)
- pub `PAK` struct L327-344 — `{ prefix: Option<String>, digest: Option<String>, rng: Option<String>, short_tok...` — Represents the PAK configuration
- pub `short_length_as_str` function L348-350 — `(&mut self)` — Convert short token length to string
- pub `long_length_as_str` function L353-355 — `(&mut self)` — Convert long token length to string
- pub `new` function L368-387 — `(file: Option<String>) -> Result<Self, ConfigError>` — Creates a new `Settings` instance
- pub `DynamicConfig` struct L395-412 — `{ log_level: String, diagnostic_cleanup_interval_seconds: u64, diagnostic_max_ag...` — Dynamic configuration values that can be hot-reloaded at runtime.
- pub `from_settings` function L416-436 — `(settings: &Settings) -> Self` — Create DynamicConfig from Settings
- pub `ConfigChange` struct L441-448 — `{ key: String, old_value: String, new_value: String }` — Represents a configuration change detected during reload
- pub `ReloadableConfig` struct L474-481 — `{ static_config: Settings, dynamic: Arc<RwLock<DynamicConfig>>, config_file: Opt...` — Configuration wrapper that separates static (restart-required) settings
- pub `new` function L493-502 — `(file: Option<String>) -> Result<Self, ConfigError>` — Creates a new ReloadableConfig instance
- pub `from_settings` function L514-522 — `(settings: Settings, config_file: Option<String>) -> Self` — Creates a ReloadableConfig from an existing Settings instance
- pub `static_config` function L527-529 — `(&self) -> &Settings` — Get a reference to the static (immutable) settings
- pub `reload` function L535-614 — `(&self) -> Result<Vec<ConfigChange>, ConfigError>` — Reload dynamic configuration from sources (file + environment)
- pub `log_level` function L621-626 — `(&self) -> String` — Get current log level
- pub `diagnostic_cleanup_interval_seconds` function L629-634 — `(&self) -> u64` — Get diagnostic cleanup interval in seconds
- pub `diagnostic_max_age_hours` function L637-642 — `(&self) -> i64` — Get diagnostic max age in hours
- pub `webhook_delivery_interval_seconds` function L645-650 — `(&self) -> u64` — Get webhook delivery interval in seconds
- pub `webhook_delivery_batch_size` function L653-658 — `(&self) -> i64` — Get webhook delivery batch size
- pub `webhook_cleanup_retention_days` function L661-666 — `(&self) -> i64` — Get webhook cleanup retention in days
- pub `cors_allowed_origins` function L669-674 — `(&self) -> Vec<String>` — Get CORS allowed origins
- pub `cors_max_age_seconds` function L677-679 — `(&self) -> u64` — Get CORS max age in seconds
- pub `dynamic_snapshot` function L682-684 — `(&self) -> Option<DynamicConfig>` — Get a snapshot of all dynamic config values
-  `deserialize_string_or_vec` function L73-110 — `(deserializer: D) -> Result<Vec<String>, D::Error>` — Deserializes a comma-separated string or array into Vec<String>
-  `StringOrVec` struct L80 — `-` — Default: "BR"
-  `StringOrVec` type L82-107 — `= StringOrVec` — Default: "BR"
-  `Value` type L83 — `= Vec<String>` — Default: "BR"
-  `expecting` function L85-87 — `(&self, formatter: &mut fmt::Formatter) -> fmt::Result` — Default: "BR"
-  `visit_str` function L89-95 — `(self, value: &str) -> Result<Self::Value, E>` — Default: "BR"
-  `visit_seq` function L97-106 — `(self, mut seq: A) -> Result<Self::Value, A::Error>` — Default: "BR"
-  `DEFAULT_SETTINGS` variable L113 — `: &str` — Default: "BR"
-  `default_log_format` function L226-228 — `() -> String` — Default: "BR"
-  `Telemetry` type L275-311 — `= Telemetry` — Default: "BR"
-  `default_otlp_endpoint` function L313-315 — `() -> String` — Default: "BR"
-  `default_service_name` function L317-319 — `() -> String` — Default: "BR"
-  `default_sampling_rate` function L321-323 — `() -> f64` — Default: "BR"
-  `PAK` type L346-356 — `= PAK` — Default: "BR"
-  `Settings` type L358-388 — `= Settings` — Default: "BR"
-  `DynamicConfig` type L414-437 — `= DynamicConfig` — Default: "BR"
-  `ReloadableConfig` type L483-685 — `= ReloadableConfig` — Default: "BR"
-  `tests` module L688-1030 — `-` — Default: "BR"
-  `test_settings_default_values` function L698-707 — `()` — Test the creation of Settings with default values
-  `test_telemetry_default_values` function L710-718 — `()` — Default: "BR"
-  `test_telemetry_for_broker_no_overrides` function L721-738 — `()` — Default: "BR"
-  `test_telemetry_for_broker_full_overrides` function L741-763 — `()` — Default: "BR"
-  `test_telemetry_for_broker_partial_overrides` function L766-788 — `()` — Default: "BR"
-  `test_telemetry_for_agent_no_overrides` function L791-808 — `()` — Default: "BR"
-  `test_telemetry_for_agent_full_overrides` function L811-833 — `()` — Default: "BR"
-  `test_telemetry_broker_and_agent_independent` function L836-870 — `()` — Default: "BR"
-  `test_telemetry_override_enabled_false_overrides_base_true` function L873-894 — `()` — Default: "BR"
-  `test_telemetry_sampling_rate_extremes` function L897-919 — `()` — Default: "BR"
-  `test_reloadable_config_creation` function L926-939 — `()` — Default: "BR"
-  `test_dynamic_config_from_settings` function L942-953 — `()` — Default: "BR"
-  `test_reloadable_config_accessors_with_defaults` function L956-966 — `()` — Default: "BR"
-  `test_reloadable_config_dynamic_snapshot` function L969-981 — `()` — Default: "BR"
-  `test_reloadable_config_reload_no_changes` function L984-994 — `()` — Default: "BR"
-  `test_reloadable_config_is_clone` function L997-1003 — `()` — Default: "BR"
-  `test_reloadable_config_thread_safety` function L1006-1029 — `()` — Default: "BR"

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

-  `test_settings_from_file_and_env` function L22-59 — `()` — Tests the loading of settings from both a file and environment variables.
-  `test_settings_default` function L71-84 — `()` — Tests the loading of default settings when no configuration file is provided.

### docs/themes/hugo-geekdoc/static/js

> *Semantic summary to be generated by AI agent.*

#### docs/themes/hugo-geekdoc/static/js/130-395cb664.chunk.min.js

- pub `_getExpansion` method L1 — `_getExpansion(e)`
- pub `baseSizingClasses` method L1 — `baseSizingClasses()`
- pub `beginGroup` method L1 — `beginGroup()`
- pub `callFunction` method L1 — `callFunction(e,t,r,a,n)`
- pub `constructor` method L1 — `constructor(e,t,r)`
- pub `consume` method L1 — `consume()`
- pub `consumeArg` method L1 — `consumeArg(e)`
- pub `consumeArgs` method L1 — `consumeArgs(e,t)`
- pub `consumeSpaces` method L1 — `consumeSpaces()`
- pub `countExpansion` method L1 — `countExpansion(e)`
- pub `cramp` method L1 — `cramp()`
- pub `endGroup` method L1 — `endGroup()`
- pub `endGroups` method L1 — `endGroups()`
- pub `expandAfterFuture` method L1 — `expandAfterFuture()`
- pub `expandMacro` method L1 — `expandMacro(e)`
- pub `expandMacroAsText` method L1 — `expandMacroAsText(e)`
- pub `expandNextToken` method L1 — `expandNextToken()`
- pub `expandOnce` method L1 — `expandOnce(e)`
- pub `expandTokens` method L1 — `expandTokens(e)`
- pub `expect` method L1 — `expect(e,t)`
- pub `extend` method L1 — `extend(e)`
- pub `feed` method L1 — `feed(e)`
- pub `fetch` method L1 — `fetch()`
- pub `fontMetrics` method L1 — `fontMetrics()`
- pub `formLigatures` method L1 — `formLigatures(e)`
- pub `formatUnsupportedCmd` method L1 — `formatUnsupportedCmd(e)`
- pub `fracDen` method L1 — `fracDen()`
- pub `fracNum` method L1 — `fracNum()`
- pub `future` method L1 — `future()`
- pub `get` method L1 — `get(e)`
- pub `getAttribute` method L1 — `getAttribute(e)`
- pub `getColor` method L1 — `getColor()`
- pub `handleInfixNodes` method L1 — `handleInfixNodes(e)`
- pub `handleSupSubscript` method L1 — `handleSupSubscript(e)`
- pub `has` method L1 — `has(e)`
- pub `hasClass` method L1 — `hasClass(e)`
- pub `havingBaseSizing` method L1 — `havingBaseSizing()`
- pub `havingBaseStyle` method L1 — `havingBaseStyle(e)`
- pub `havingCrampedStyle` method L1 — `havingCrampedStyle()`
- pub `havingSize` method L1 — `havingSize(e)`
- pub `havingStyle` method L1 — `havingStyle(e)`
- pub `isDefined` method L1 — `isDefined(e)`
- pub `isExpandable` method L1 — `isExpandable(e)`
- pub `isTight` method L1 — `isTight()`
- pub `isTrusted` method L1 — `isTrusted(e)`
- pub `lex` method L1 — `lex()`
- pub `parse` method L1 — `parse()`
- pub `parseArgumentGroup` method L1 — `parseArgumentGroup(e,t)`
- pub `parseArguments` method L1 — `parseArguments(e,t)`
- pub `parseAtom` method L1 — `parseAtom(e)`
- pub `parseColorGroup` method L1 — `parseColorGroup(e)`
- pub `parseExpression` method L1 — `parseExpression(e,t)`
- pub `parseFunction` method L1 — `parseFunction(e,t)`
- pub `parseGroup` method L1 — `parseGroup(e,t)`
- pub `parseGroupOfType` method L1 — `parseGroupOfType(e,t,r)`
- pub `parseRegexGroup` method L1 — `parseRegexGroup(e,t)`
- pub `parseSizeGroup` method L1 — `parseSizeGroup(e)`
- pub `parseStringGroup` method L1 — `parseStringGroup(e,t)`
- pub `parseSymbol` method L1 — `parseSymbol()`
- pub `parseUrlGroup` method L1 — `parseUrlGroup(e)`
- pub `popToken` method L1 — `popToken()`
- pub `pushToken` method L1 — `pushToken(e)`
- pub `pushTokens` method L1 — `pushTokens(e)`
- pub `range` method L1 — `range(e,t)`
- pub `reportNonstrict` method L1 — `reportNonstrict(e,t,r)`
- pub `scanArgument` method L1 — `scanArgument(e)`
- pub `set` method L1 — `set(e,t,r)`
- pub `setAttribute` method L1 — `setAttribute(e,t)`
- pub `setCatcode` method L1 — `setCatcode(e,t)`
- pub `sizingClasses` method L1 — `sizingClasses(e)`
- pub `sub` method L1 — `sub()`
- pub `subparse` method L1 — `subparse(e)`
- pub `sup` method L1 — `sup()`
- pub `switchMode` method L1 — `switchMode(e)`
- pub `text` method L1 — `text()`
- pub `toMarkup` method L1 — `toMarkup()`
- pub `toNode` method L1 — `toNode()`
- pub `toText` method L1 — `toText()`
- pub `useStrictBehavior` method L1 — `useStrictBehavior(e,t,r)`
- pub `withColor` method L1 — `withColor(e)`
- pub `withFont` method L1 — `withFont(e)`
- pub `withPhantom` method L1 — `withPhantom()`
- pub `withTextFontFamily` method L1 — `withTextFontFamily(e)`
- pub `withTextFontShape` method L1 — `withTextFontShape(e)`
- pub `withTextFontWeight` method L1 — `withTextFontWeight(e)`
-  `At` function L1 — `function At(e)`
-  `Bt` class L1 — `-`
-  `C` function L1 — `function C(e)`
-  `Dr` function L1 — `function Dr(e)`
-  `Et` function L1 — `function Et(e,t,r,a,n)`
-  `Fr` function L1 — `function Fr(e)`
-  `G` class L1 — `-`
-  `Ha` class L1 — `-`
-  `Hr` function L1 — `function Hr(e)`
-  `J` class L1 — `-`
-  `Jt` function L1 — `function Jt(e,t)`
-  `Kt` function L1 — `function Kt(e,t)`
-  `L` function L1 — `function L(e,t,r)`
-  `Mt` function L1 — `function Mt(e,t)`
-  `Pr` function L1 — `function Pr(e,t)`
-  `Q` class L1 — `-`
-  `R` class L1 — `-`
-  `Ra` class L1 — `-`
-  `Rr` function L1 — `function Rr(e,t)`
-  `Tt` class L1 — `-`
-  `Ua` class L1 — `-`
-  `Ur` function L1 — `function Ur(e)`
-  `Ut` function L1 — `function Ut(e,t)`
-  `Xr` function L1 — `function Xr(e)`
-  `Xt` function L1 — `function Xt(e)`
-  `Yr` function L1 — `function Yr(e,t,r)`
-  `Yt` function L1 — `function Yt(e)`
-  `a` class L1 — `-`
-  `ae` class L1 — `-`
-  `b` function L1 — `function b(e)`
-  `ee` class L1 — `-`
-  `ga` function L1 — `function ga(e,t,r)`
-  `ht` function L1 — `function ht(e)`
-  `i` class L1 — `-`
-  `ie` class L1 — `-`
-  `ja` class L1 — `-`
-  `k` function L1 — `function k()`
-  `lt` function L1 — `function lt(e)`
-  `me` function L1 — `function me(e,t,r,a,n,i)`
-  `n` class L1 — `-`
-  `ne` class L1 — `-`
-  `oe` function L1 — `function oe(e)`
-  `re` class L1 — `-`
-  `rr` function L1 — `function rr(e,t,r)`
-  `w` function L1 — `function w()`
-  `x` class L1 — `-`
-  `x` function L1 — `function x(e)`
-  `y` class L1 — `-`
-  `zt` function L1 — `function zt(e,t)`

#### docs/themes/hugo-geekdoc/static/js/155-155e0581.chunk.min.js

-  `At` function L1 — `function At(t,e,a,n,s)`
-  `Rt` function L1 — `function Rt(t,e,a,n,i)`
-  `a` function L1 — `function a(t,a,i,r,s,l,o,c)`
-  `e` function L1 — `function e(t,e,a,i,s,l,o,c)`
-  `n` function L1 — `function n(t,e)`
-  `st` function L1 — `function st()`
-  `t` function L1 — `function t(t,e,a,i,r,s,l)`
-  `x` function L1 — `function x()`

#### docs/themes/hugo-geekdoc/static/js/164-c7b61128.chunk.min.js

-  `o` function L1 — `function o(t)`

#### docs/themes/hugo-geekdoc/static/js/165-4df74207.chunk.min.js

-  `$a` function L2 — `function $a(e,t,n)`
-  `$i` function L2 — `function $i(e,t,n)`
-  `$l` function L2 — `function $l(e,t)`
-  `Au` function L2 — `function Au(e,t)`
-  `Bi` function L2 — `function Bi()`
-  `Bo` function L2 — `function Bo(e,t,n,r)`
-  `D` function L2 — `function D(e,r)`
-  `Do` function L2 — `function Do(e,t)`
-  `Eo` function L2 — `function Eo(e,t,n,r)`
-  `Iu` function L2 — `function Iu(e,t,n)`
-  `Ji` function L2 — `function Ji(e,t)`
-  `Jl` function L2 — `function Jl(e,t,n)`
-  `Jr` function L2 — `function Jr(e)`
-  `Lu` function L2 — `function Lu(e,t,n,r,a)`
-  `Mo` function L2 — `function Mo(e,t)`
-  `Ni` function L2 — `function Ni()`
-  `No` function L2 — `function No(e,t)`
-  `Oi` function L2 — `function Oi()`
-  `On` function L2 — `function On(e,t,n,r,a,i)`
-  `Os` function L2 — `function Os(e)`
-  `Ou` function L2 — `function Ou(e,t,n,r)`
-  `P` function L2 — `function P(e,t,n,r,a)`
-  `Po` function L2 — `function Po(e,t,n,r,a)`
-  `Pu` function L2 — `function Pu(e,t,n)`
-  `Qa` function L2 — `function Qa(e,t,n)`
-  `Qi` function L2 — `function Qi(e,t)`
-  `Ql` function L2 — `function Ql(e,t,n)`
-  `Su` function L2 — `function Su(e)`
-  `To` function L2 — `function To(e,t,n,r,a)`
-  `Ua` function L2 — `function Ua(e,t,n,r)`
-  `Ur` function L2 — `function Ur(e)`
-  `Vs` function L2 — `function Vs(e)`
-  `Ws` function L2 — `function Ws(e)`
-  `Xr` function L2 — `function Xr(e)`
-  `Ys` function L2 — `function Ys(e)`
-  `Za` function L2 — `function Za(e,t,n)`
-  `_o` function L2 — `function _o(e,t,n,r)`
-  `_u` function L2 — `function _u(e)`
-  `a` function L2 — `function a(e,t)`
-  `ai` function L2 — `function ai(e,t)`
-  `b` function L2 — `function b(e,t,n)`
-  `c` function L2 — `function c(e,t)`
-  `co` function L2 — `function co(e)`
-  `d` function L2 — `function d(e,t)`
-  `ds` function L2 — `function ds(e)`
-  `e` function L2 — `function e()`
-  `eu` function L2 — `function eu(e,t,n,r,a)`
-  `f` function L2 — `function f(e)`
-  `fs` function L2 — `function fs(e)`
-  `g` function L2 — `function g()`
-  `h` function L2 — `function h(e,t)`
-  `i` function L2 — `function i(e,t)`
-  `js` function L2 — `function js(e)`
-  `kl` function L2 — `function kl(e,t,n)`
-  `l` function L2 — `function l(e,t)`
-  `m` function L2 — `function m(n)`
-  `ml` function L2 — `function ml(e,t,n,r)`
-  `mo` function L2 — `function mo(e)`
-  `n` function L2 — `function n(e)`
-  `no` function L2 — `function no(e)`
-  `o` function L2 — `function o(e,t,n)`
-  `p` function L2 — `function p(t)`
-  `pu` function L2 — `function pu(e,t,n,r,a)`
-  `r` function L2 — `function r(e)`
-  `ri` function L2 — `function ri(e)`
-  `s` function L2 — `function s(e,t,n)`
-  `t` function L2 — `function t(t,n,r)`
-  `ta` function L2 — `function ta(e,t)`
-  `to` function L2 — `function to(e,t)`
-  `tu` function L2 — `function tu(e,t,n,r)`
-  `u` function L2 — `function u(e,t)`
-  `uo` function L2 — `function uo(e)`
-  `us` function L2 — `function us(e)`
-  `v` function L2 — `function v(e)`
-  `vo` function L2 — `function vo(e)`
-  `vs` function L2 — `function vs(e)`
-  `x` function L2 — `function x(n,r)`
-  `xl` function L2 — `function xl(e)`
-  `y` function L2 — `function y()`
-  `yl` function L2 — `function yl(e,t)`
-  `yo` function L2 — `function yo(e)`

#### docs/themes/hugo-geekdoc/static/js/174-5ff0286f.chunk.min.js

-  `E` function L1 — `function E(t,r,e)`
-  `g` function L1 — `function g(t,r)`
-  `k` function L1 — `function k(t)`
-  `l` function L1 — `function l()`
-  `n` function L1 — `function n(t,r)`

#### docs/themes/hugo-geekdoc/static/js/178-3e4e928c.chunk.min.js

-  `f` function L1 — `function f(e)`
-  `h` function L1 — `function h(e)`
-  `l` function L1 — `function l(e)`

#### docs/themes/hugo-geekdoc/static/js/186-df634c5c.chunk.min.js

-  `r` function L1 — `function r(t,e)`

#### docs/themes/hugo-geekdoc/static/js/247-34fff2e1.chunk.min.js

-  `T` function L1 — `function T()`
-  `at` function L1 — `function at(t,e)`
-  `ct` function L1 — `function ct(t,e,i,r,{spatialMaps:n})`
-  `g` function L1 — `function g(t,e,i)`
-  `h` function L1 — `function h()`
-  `ht` function L1 — `function ht(t)`
-  `i` function L1 — `function i(r)`
-  `j` function L1 — `function j(t)`
-  `l` function L1 — `function l(t,e,i,s)`
-  `lt` function L1 — `function lt(t)`
-  `n` function L1 — `function n()`
-  `nt` function L1 — `function nt(t,e)`
-  `o` function L1 — `function o(t,e,i,n)`
-  `ot` function L1 — `function ot(t,e)`
-  `r` function L1 — `function r(t)`
-  `rt` function L1 — `function rt(t,e)`
-  `s` function L1 — `function s(t,e,i)`
-  `st` function L1 — `function st(t,e)`
-  `t` function L1 — `function t(t,e)`

#### docs/themes/hugo-geekdoc/static/js/32-f6b664cc.chunk.min.js

-  `$t` function L1 — `function $t(t,e,a,r,s,i,o)`
-  `B` function L1 — `function B(t,e)`
-  `Bt` function L1 — `function Bt(t,e,a,r,s)`
-  `F` function L1 — `function F()`
-  `Ft` function L1 — `function Ft(t,e,a)`
-  `I` function L1 — `function I(t,e)`
-  `Mt` function L1 — `function Mt(t,e)`
-  `V` function L1 — `function V(t,e)`
-  `a` function L1 — `function a(t,e,a,r,s)`
-  `b` function L1 — `function b()`
-  `c` function L1 — `function c(a,r)`
-  `e` function L1 — `function e(t,e,a,r,o,c,l,d)`
-  `l` function L1 — `function l(a,r)`
-  `o` function L1 — `function o(o)`
-  `qt` function L1 — `function qt(t,e,a)`
-  `r` function L1 — `function r(t,a,r,i,o,c,l,d)`
-  `s` function L1 — `function s(t,e)`
-  `t` function L1 — `function t(t,e,a,r,i,n,o)`
-  `z` function L1 — `function z()`

#### docs/themes/hugo-geekdoc/static/js/357-2a926bc9.chunk.min.js

-  `a` function L1 — `function a(t)`
-  `c` function L1 — `function c(t)`
-  `e` function L1 — `function e(t,e,n,i,r,a,o,c,l)`
-  `j` function L1 — `function j(t,e)`
-  `n` function L1 — `function n(t,e,n,i,s)`
-  `o` function L1 — `function o(t)`
-  `s` function L1 — `function s(t,e)`
-  `t` function L1 — `function t(t,e,n,i,r,a,o,c)`
-  `u` function L1 — `function u()`
-  `x` function L1 — `function x()`

#### docs/themes/hugo-geekdoc/static/js/364-fd5df3dd.chunk.min.js

-  `A` function L1 — `function A(t)`
-  `C` function L1 — `function C(t)`
-  `F` function L1 — `function F(t,e)`
-  `L` function L1 — `function L()`
-  `P` function L1 — `function P(t)`
-  `S` function L1 — `function S(t)`
-  `T` function L1 — `function T()`
-  `_` function L1 — `function _(t)`
-  `b` function L1 — `function b(t)`
-  `c` function L1 — `function c(t)`
-  `d` function L1 — `function d(t)`
-  `g` function L1 — `function g(t)`
-  `gt` function L1 — `function gt()`
-  `k` function L1 — `function k(t,e,i,a,n)`
-  `l` function L1 — `function l(t)`
-  `m` function L1 — `function m(t)`
-  `o` function L1 — `function o(t)`
-  `p` function L1 — `function p(t)`
-  `q` function L1 — `function q(t)`
-  `r` function L1 — `function r(t)`
-  `u` function L1 — `function u(t)`
-  `y` function L1 — `function y(t)`

#### docs/themes/hugo-geekdoc/static/js/379-233b54d3.chunk.min.js

-  `L` function L1 — `function L(t)`
-  `a` function L1 — `function a(t)`
-  `c` function L1 — `function c(t)`
-  `e` function L1 — `function e(t,e,n,s,r,a,o,c,l)`
-  `i` function L1 — `function i(t,e)`
-  `m` function L1 — `function m()`
-  `n` function L1 — `function n(t,e,n,i,s)`
-  `o` function L1 — `function o(t)`
-  `t` function L1 — `function t(t,e,n,s,r,a,o,c)`
-  `u` function L1 — `function u()`

#### docs/themes/hugo-geekdoc/static/js/445-99c1ba44.chunk.min.js

-  `N` function L1 — `function N()`
-  `b` function L1 — `function b()`

#### docs/themes/hugo-geekdoc/static/js/449-121db0c2.chunk.min.js

-  `F` function L1 — `function F(t,e,i,n)`
-  `P` function L1 — `function P(t,e)`
-  `R` function L1 — `function R(t,e,i,n,r)`
-  `S` function L1 — `function S(t,e,i,n,r)`
-  `U` function L1 — `function U(t,e)`
-  `_` function L1 — `function _()`
-  `b` function L1 — `function b(t,e)`
-  `g` function L1 — `function g(t)`
-  `h` function L1 — `function h()`
-  `i` function L1 — `function i(n)`
-  `l` function L1 — `function l(t,e,i,s)`
-  `n` function L1 — `function n()`
-  `o` function L1 — `function o(t,e,i,r)`
-  `r` function L1 — `function r()`
-  `s` function L1 — `function s(t,e,i)`
-  `t` function L1 — `function t(t,e)`
-  `u` function L1 — `function u(t,e,i)`
-  `v` function L1 — `function v()`

#### docs/themes/hugo-geekdoc/static/js/496-1979476f.chunk.min.js

-  `R` function L1 — `function R()`
-  `V` function L1 — `function V()`

#### docs/themes/hugo-geekdoc/static/js/525-abc802a0.chunk.min.js

-  `$` function L1 — `function $(n,e)`
-  `C` function L1 — `function C(t)`
-  `D` function L1 — `function D(t)`
-  `I` function L1 — `function I(t)`
-  `K` function L1 — `function K(t)`
-  `L` function L1 — `function L()`
-  `M` function L1 — `function M(t)`
-  `N` function L1 — `function N(t,n,e,i,s)`
-  `O` function L1 — `function O()`
-  `P` function L1 — `function P(t)`
-  `S` function L1 — `function S()`
-  `T` function L1 — `function T(t)`
-  `_` function L1 — `function _(t)`
-  `a` function L1 — `function a(t,n)`
-  `c` function L1 — `function c(t,n)`
-  `d` function L1 — `function d(t)`
-  `f` function L1 — `function f(t,n)`
-  `g` function L1 — `function g(t)`
-  `h` function L1 — `function h(t)`
-  `k` function L1 — `function k(t,n)`
-  `l` function L1 — `function l(t,n)`
-  `o` function L1 — `function o(t)`
-  `p` function L1 — `function p(t)`
-  `r` function L1 — `function r(t,n)`
-  `u` function L1 — `function u(t,n)`
-  `x` function L1 — `function x({nodes:t})`
-  `y` function L1 — `function y(t,n)`

#### docs/themes/hugo-geekdoc/static/js/567-4fef9a1a.chunk.min.js

- pub `_removeFromParentsChildList` method L1 — `_removeFromParentsChildList(e)`
- pub `children` method L1 — `children(e)`
- pub `constructor` method L1 — `constructor()`
- pub `dequeue` method L1 — `dequeue()`
- pub `edge` method L1 — `edge(e,n,t)`
- pub `edgeCount` method L1 — `edgeCount()`
- pub `edges` method L1 — `edges()`
- pub `enqueue` method L1 — `enqueue(e)`
- pub `filterNodes` method L1 — `filterNodes(e)`
- pub `graph` method L1 — `graph()`
- pub `hasEdge` method L1 — `hasEdge(e,n,t)`
- pub `hasNode` method L1 — `hasNode(e)`
- pub `inEdges` method L1 — `inEdges(e,n)`
- pub `isCompound` method L1 — `isCompound()`
- pub `isDirected` method L1 — `isDirected()`
- pub `isLeaf` method L1 — `isLeaf(e)`
- pub `isMultigraph` method L1 — `isMultigraph()`
- pub `neighbors` method L1 — `neighbors(e)`
- pub `node` method L1 — `node(e)`
- pub `nodeCount` method L1 — `nodeCount()`
- pub `nodeEdges` method L1 — `nodeEdges(e,n)`
- pub `nodes` method L1 — `nodes()`
- pub `outEdges` method L1 — `outEdges(e,n)`
- pub `parent` method L1 — `parent(e)`
- pub `predecessors` method L1 — `predecessors(e)`
- pub `removeEdge` method L1 — `removeEdge(e,n,t)`
- pub `removeNode` method L1 — `removeNode(e)`
- pub `setDefaultEdgeLabel` method L1 — `setDefaultEdgeLabel(e)`
- pub `setDefaultNodeLabel` method L1 — `setDefaultNodeLabel(e)`
- pub `setEdge` method L1 — `setEdge()`
- pub `setGraph` method L1 — `setGraph(e)`
- pub `setNode` method L1 — `setNode(e,n)`
- pub `setNodes` method L1 — `setNodes(e,n)`
- pub `setParent` method L1 — `setParent(e,n)`
- pub `setPath` method L1 — `setPath(e,n)`
- pub `sinks` method L1 — `sinks()`
- pub `sources` method L1 — `sources()`
- pub `successors` method L1 — `successors(e)`
- pub `toString` method L1 — `toString()`
-  `$` function L1 — `function $(e,n,t,r)`
-  `A` function L1 — `function A(e)`
-  `An` function L1 — `function An(e,n,t)`
-  `Be` function L1 — `function Be(e)`
-  `Ce` function L1 — `function Ce(e,n,t)`
-  `De` function L1 — `function De(e)`
-  `Fe` function L1 — `function Fe(e,n,t)`
-  `H` function L1 — `function H(e)`
-  `Ie` function L1 — `function Ie(e,n)`
-  `J` function L1 — `function J(e)`
-  `K` function L1 — `function K(e,n,t,r)`
-  `Le` function L1 — `function Le(e,n,t,o,i)`
-  `Me` function L1 — `function Me(e,n,t)`
-  `Ne` function L1 — `function Ne(e,n,t,o,i,u)`
-  `Oe` function L1 — `function Oe(e,n,t)`
-  `Pe` function L1 — `function Pe(e)`
-  `Pn` function L1 — `function Pn(e,n)`
-  `Q` function L1 — `function Q(e)`
-  `Re` function L1 — `function Re(e,n,t,o)`
-  `Te` function L1 — `function Te(e)`
-  `U` function L1 — `function U(e,n)`
-  `W` function L1 — `function W(e,n)`
-  `X` function L1 — `function X(e,n,t,r,o,i)`
-  `Z` function L1 — `function Z(e,n)`
-  `_` function L1 — `function _(e)`
-  `ae` function L1 — `function ae(e,n)`
-  `an` function L1 — `function an(e,n,t,o)`
-  `b` function L1 — `function b(e,n)`
-  `bn` function L1 — `function bn(e,n)`
-  `ce` function L1 — `function ce(e,n)`
-  `cn` function L1 — `function cn(e,n)`
-  `d` function L1 — `function d(e,n)`
-  `de` function L1 — `function de(e,n)`
-  `dn` function L1 — `function dn(e,n)`
-  `ee` function L1 — `function ee(e)`
-  `he` function L1 — `function he(e,n,t)`
-  `je` function L1 — `function je(e,n)`
-  `jn` function L1 — `function jn(e)`
-  `m` function L1 — `function m(e,n,t,o,i)`
-  `ne` function L1 — `function ne(e)`
-  `o` function L1 — `function o(n)`
-  `on` function L1 — `function on(e,n)`
-  `p` class L1 — `-`
-  `pn` function L1 — `function pn(e,n,t)`
-  `qe` function L1 — `function qe(e,n,t,o,i,u,a)`
-  `re` function L1 — `function re(e)`
-  `rn` function L1 — `function rn(e,n,t)`
-  `se` function L1 — `function se(e)`
-  `sn` function L1 — `function sn(e,n,t)`
-  `t` function L1 — `function t(o,i)`
-  `te` function L1 — `function te(e)`
-  `tn` function L1 — `function tn(e,n)`
-  `ue` function L1 — `function ue(e)`
-  `un` function L1 — `function un(e,n,t)`
-  `w` function L1 — `function w(e,n)`
-  `w` class L1 — `-`
-  `wn` function L1 — `function wn(e)`
-  `y` function L1 — `function y(e,n,t)`

#### docs/themes/hugo-geekdoc/static/js/573-5fb26808.chunk.min.js

-  `N` function L1 — `function N()`
-  `Y` function L1 — `function Y(t="",e="")`
-  `Z` function L1 — `function Z(t="")`
-  `d` function L1 — `function d(t,e,r,a)`
-  `g` function L1 — `function g()`
-  `t` function L1 — `function t(t,e,r,a)`
-  `u` function L1 — `function u(t,e)`

#### docs/themes/hugo-geekdoc/static/js/664-723fc55c.chunk.min.js

-  `B` function L1 — `function B(t="",e=0,s="",i=I)`
-  `Et` function L1 — `function Et(t,e,s)`
-  `G` function L1 — `function G(t)`
-  `J` function L1 — `function J()`
-  `S` function L1 — `function S()`
-  `Tt` function L1 — `function Tt(t="")`
-  `Y` function L1 — `function Y(t,e,s)`
-  `_t` function L1 — `function _t(t="")`
-  `bt` function L1 — `function bt(t="",e=d)`
-  `j` function L1 — `function j(t)`
-  `kt` function L1 — `function kt(t="",e=d)`
-  `w` function L1 — `function w()`

#### docs/themes/hugo-geekdoc/static/js/689-3cbd5ea9.chunk.min.js

-  `ee` function L1 — `function ee()`
-  `gt` function L1 — `function gt(t)`
-  `i` function L1 — `const i = (t,e)`
-  `k` function L1 — `function k()`
-  `u` function L1 — `function u(t)`

#### docs/themes/hugo-geekdoc/static/js/711-c5eeef68.chunk.min.js

-  `$` function L1 — `function $(t)`
-  `D` function L1 — `function D(t,e,n,s)`
-  `K` function L1 — `function K()`
-  `Kt` function L1 — `function Kt(t,e,n)`
-  `T` function L1 — `function T(t,e,n)`
-  `_` function L1 — `function _(t,e)`
-  `b` function L1 — `function b(t,e)`
-  `f` function L1 — `function f(n)`
-  `g` function L1 — `function g()`
-  `v` function L1 — `function v(t,n,a,o,c,l,u)`
-  `w` function L1 — `function w(t,e,n,s)`
-  `x` function L1 — `function x(t,e,n,o,c,l,d,u)`

#### docs/themes/hugo-geekdoc/static/js/731-70ea2831.chunk.min.js

- pub `DEFINE_RULE` method L1 — `DEFINE_RULE(e,t)`
- pub `IS_RECORDING` method L1 — `IS_RECORDING()`
- pub `accept` method L1 — `accept(e)`
- pub `action` method L1 — `action(e,t)`
- pub `add` method L1 — `add(e,t=null,n)`
- pub `addAll` method L1 — `addAll(e,t)`
- pub `addAstNodeRegionWithAssignmentsTo` method L1 — `addAstNodeRegionWithAssignmentsTo(e)`
- pub `addDocument` method L1 — `addDocument(e)`
- pub `addEntry` method L1 — `addEntry(e,t)`
- pub `addHiddenToken` method L1 — `addHiddenToken(e,t)`
- pub `addHiddenTokens` method L1 — `addHiddenTokens(e)`
- pub `addParents` method L1 — `addParents(e)`
- pub `addTokenUsingMemberAccess` method L1 — `addTokenUsingMemberAccess(e,t,n)`
- pub `addTokenUsingPush` method L1 — `addTokenUsingPush(e,t,n)`
- pub `after` method L1 — `after(e)`
- pub `all` method L1 — `all()`
- pub `allElements` method L1 — `allElements(e,t)`
- pub `alternative` method L1 — `alternative()`
- pub `alternatives` method L1 — `alternatives(e,t)`
- pub `alts` method L1 — `alts()`
- pub `assertion` method L1 — `assertion()`
- pub `assign` method L1 — `assign(e,t,n,r,i)`
- pub `assignWithoutOverride` method L1 — `assignWithoutOverride(e,t)`
- pub `astNode` method L1 — `astNode()`
- pub `atLeastOne` method L1 — `atLeastOne(e,t)`
- pub `atom` method L1 — `atom()`
- pub `atomEscape` method L1 — `atomEscape()`
- pub `before` method L1 — `before(e)`
- pub `build` method L1 — `build(e,t={},n=yc.XO.None)`
- pub `buildCompositeNode` method L1 — `buildCompositeNode(e)`
- pub `buildDocuments` method L1 — `buildDocuments(e,t,n)`
- pub `buildEarlyExitMessage` method L1 — `buildEarlyExitMessage(e)`
- pub `buildKeywordPattern` method L1 — `buildKeywordPattern(e,t)`
- pub `buildKeywordToken` method L1 — `buildKeywordToken(e,t,n)`
- pub `buildKeywordTokens` method L1 — `buildKeywordTokens(e,t,n)`
- pub `buildLeafNode` method L1 — `buildLeafNode(e,t)`
- pub `buildLookaheadForAlternation` method L1 — `buildLookaheadForAlternation(e)`
- pub `buildLookaheadForOptional` method L1 — `buildLookaheadForOptional(e)`
- pub `buildMismatchTokenMessage` method L1 — `buildMismatchTokenMessage(e)`
- pub `buildNoViableAltMessage` method L1 — `buildNoViableAltMessage(e)`
- pub `buildNotAllInputParsedMessage` method L1 — `buildNotAllInputParsedMessage(e)`
- pub `buildReference` method L1 — `buildReference(e,t,n,i)`
- pub `buildRootNode` method L1 — `buildRootNode(e)`
- pub `buildTerminalToken` method L1 — `buildTerminalToken(e)`
- pub `buildTerminalTokens` method L1 — `buildTerminalTokens(e)`
- pub `buildTokens` method L1 — `buildTokens(e,t)`
- pub `cacheForContext` method L1 — `cacheForContext(e)`
- pub `cancel` method L1 — `cancel()`
- pub `cancelWrite` method L1 — `cancelWrite()`
- pub `characterClass` method L1 — `characterClass()`
- pub `characterClassEscape` method L1 — `characterClassEscape()`
- pub `checkIsTarget` method L1 — `checkIsTarget(e,t,n,r)`
- pub `children` method L1 — `children()`
- pub `chopInput` method L1 — `chopInput(e,t)`
- pub `classAtom` method L1 — `classAtom()`
- pub `classEscape` method L1 — `classEscape()`
- pub `classPatternCharacterAtom` method L1 — `classPatternCharacterAtom()`
- pub `clear` method L1 — `clear()`
- pub `computeExports` method L1 — `computeExports(e,t=yc.XO.None)`
- pub `computeExportsForNode` method L1 — `computeExportsForNode(e,t,n=ke,r=yc.XO.None)`
- pub `computeIsSubtype` method L1 — `computeIsSubtype(e,t)`
- pub `computeLocalScopes` method L1 — `computeLocalScopes(e,t=yc.XO.None)`
- pub `computeNewColumn` method L1 — `computeNewColumn(e,t)`
- pub `concat` method L1 — `concat(e)`
- pub `construct` method L1 — `construct(e)`
- pub `constructor` method L1 — `constructor()`
- pub `consume` method L1 — `consume(e,t,n)`
- pub `consumeChar` method L1 — `consumeChar(e)`
- pub `controlEscapeAtom` method L1 — `controlEscapeAtom()`
- pub `controlLetterEscapeAtom` method L1 — `controlLetterEscapeAtom()`
- pub `convert` method L1 — `convert(e,t)`
- pub `count` method L1 — `count()`
- pub `create` method L1 — `create(e,t)`
- pub `createAsync` method L1 — `createAsync(e,t,n)`
- pub `createDehyrationContext` method L1 — `createDehyrationContext(e)`
- pub `createDescription` method L1 — `createDescription(e,t,n=Ee(e))`
- pub `createDescriptions` method L1 — `createDescriptions(e,t=yc.XO.None)`
- pub `createDocument` method L1 — `createDocument(e,t,n)`
- pub `createFullToken` method L1 — `createFullToken(e,t,n,r,i,s,a)`
- pub `createGrammarElementIdMap` method L1 — `createGrammarElementIdMap()`
- pub `createHydrationContext` method L1 — `createHydrationContext(e)`
- pub `createLangiumDocument` method L1 — `createLangiumDocument(e,t,n,r)`
- pub `createLinkingError` method L1 — `createLinkingError(e,t)`
- pub `createOffsetOnlyToken` method L1 — `createOffsetOnlyToken(e,t,n,r)`
- pub `createScope` method L1 — `createScope(e,t,n)`
- pub `createScopeForNodes` method L1 — `createScopeForNodes(e,t,n)`
- pub `createStartOnlyToken` method L1 — `createStartOnlyToken(e,t,n,r,i,s)`
- pub `createTextDocumentGetter` method L1 — `createTextDocumentGetter(e,t)`
- pub `currIdx` method L1 — `currIdx()`
- pub `current` method L1 — `current()`
- pub `decimalEscapeAtom` method L1 — `decimalEscapeAtom()`
- pub `definition` method L1 — `definition()`
- pub `definitionErrors` method L1 — `definitionErrors()`
- pub `dehydrate` method L1 — `dehydrate(e)`
- pub `dehydrateAstNode` method L1 — `dehydrateAstNode(e,t)`
- pub `dehydrateCstNode` method L1 — `dehydrateCstNode(e,t)`
- pub `dehydrateReference` method L1 — `dehydrateReference(e,t)`
- pub `delete` method L1 — `delete(e,t)`
- pub `deleteDocument` method L1 — `deleteDocument(e)`
- pub `deserialize` method L1 — `deserialize(e,t={})`
- pub `disjunction` method L1 — `disjunction()`
- pub `dispose` method L1 — `dispose()`
- pub `distinct` method L1 — `distinct(e)`
- pub `doLink` method L1 — `doLink(e,t)`
- pub `documentationLinkRenderer` method L1 — `documentationLinkRenderer(e,t,n)`
- pub `documentationTagRenderer` method L1 — `documentationTagRenderer(e,t)`
- pub `dotAll` method L1 — `dotAll()`
- pub `element` method L1 — `element()`
- pub `elements` method L1 — `elements()`
- pub `emitUpdate` method L1 — `emitUpdate(e,t)`
- pub `end` method L1 — `end()`
- pub `enqueue` method L1 — `enqueue(e,t,n)`
- pub `ensureBeforeEOL` method L1 — `ensureBeforeEOL(e,t)`
- pub `entries` method L1 — `entries()`
- pub `entriesGroupedByKey` method L1 — `entriesGroupedByKey()`
- pub `event` method L1 — `event()`
- pub `every` method L1 — `every(e)`
- pub `exclude` method L1 — `exclude(e,t)`
- pub `exportNode` method L1 — `exportNode(e,t,n)`
- pub `feature` method L1 — `feature()`
- pub `file` method L1 — `file(t)`
- pub `filter` method L1 — `filter(e)`
- pub `finalize` method L1 — `finalize()`
- pub `find` method L1 — `find(e)`
- pub `findAllReferences` method L1 — `findAllReferences(e,t)`
- pub `findDeclaration` method L1 — `findDeclaration(e)`
- pub `findDeclarationNode` method L1 — `findDeclarationNode(e)`
- pub `findIndex` method L1 — `findIndex(e)`
- pub `findLongerAlt` method L1 — `findLongerAlt(e,t)`
- pub `findNameInGlobalScope` method L1 — `findNameInGlobalScope(e,t)`
- pub `findNameInPrecomputedScopes` method L1 — `findNameInPrecomputedScopes(e,t)`
- pub `findReferences` method L1 — `findReferences(e,t)`
- pub `fire` method L1 — `fire(e)`
- pub `firstNonHiddenNode` method L1 — `firstNonHiddenNode()`
- pub `flat` method L1 — `flat(e)`
- pub `flatMap` method L1 — `flatMap(e)`
- pub `forEach` method L1 — `forEach(e)`
- pub `from` method L1 — `from(e)`
- pub `fromModel` method L1 — `fromModel(e,t)`
- pub `fromString` method L1 — `fromString(e,t,n)`
- pub `fromTextDocument` method L1 — `fromTextDocument(e,t,n)`
- pub `fromUri` method L1 — `fromUri(e,t=yc.XO.None)`
- pub `fsPath` method L1 — `fsPath()`
- pub `fullText` method L1 — `fullText()`
- pub `get` method L1 — `get(e)`
- pub `getAllElements` method L1 — `getAllElements()`
- pub `getAllSubTypes` method L1 — `getAllSubTypes(e)`
- pub `getAllTags` method L1 — `getAllTags()`
- pub `getAllTypes` method L1 — `getAllTypes()`
- pub `getAssignment` method L1 — `getAssignment(e)`
- pub `getAstNode` method L1 — `getAstNode(e,t)`
- pub `getAstNodePath` method L1 — `getAstNodePath(e)`
- pub `getBuildOptions` method L1 — `getBuildOptions(e)`
- pub `getCandidate` method L1 — `getCandidate(e)`
- pub `getChecks` method L1 — `getChecks(e,t)`
- pub `getComment` method L1 — `getComment(e)`
- pub `getConfiguration` method L1 — `getConfiguration(e,t)`
- pub `getDocument` method L1 — `getDocument(e)`
- pub `getDocumentation` method L1 — `getDocumentation(e)`
- pub `getElement` method L1 — `getElement(e)`
- pub `getFileDescriptions` method L1 — `getFileDescriptions(e,t)`
- pub `getGlobalScope` method L1 — `getGlobalScope(e,t)`
- pub `getGrammarElement` method L1 — `getGrammarElement(e)`
- pub `getGrammarElementId` method L1 — `getGrammarElementId(e)`
- pub `getKey` method L1 — `getKey(e)`
- pub `getLineOffsets` method L1 — `getLineOffsets()`
- pub `getLinkedNode` method L1 — `getLinkedNode(e)`
- pub `getName` method L1 — `getName(e)`
- pub `getNameNode` method L1 — `getNameNode(e)`
- pub `getOrCreateDocument` method L1 — `getOrCreateDocument(e,t)`
- pub `getPathSegment` method L1 — `getPathSegment({$containerProperty:e,$containerIndex:t})`
- pub `getRefNode` method L1 — `getRefNode(e,t,n)`
- pub `getReferenceToSelf` method L1 — `getReferenceToSelf(e)`
- pub `getReferenceType` method L1 — `getReferenceType(e)`
- pub `getRootFolder` method L1 — `getRootFolder(e)`
- pub `getRuleStack` method L1 — `getRuleStack()`
- pub `getScope` method L1 — `getScope(e)`
- pub `getServices` method L1 — `getServices(e)`
- pub `getSource` method L1 — `getSource()`
- pub `getTag` method L1 — `getTag(e)`
- pub `getTags` method L1 — `getTags(e)`
- pub `getText` method L1 — `getText(e)`
- pub `getTokenType` method L1 — `getTokenType(e)`
- pub `getTypeMetaData` method L1 — `getTypeMetaData(e)`
- pub `group` method L1 — `group()`
- pub `handleModes` method L1 — `handleModes(e,t,n,r)`
- pub `handlePayloadNoCustom` method L1 — `handlePayloadNoCustom(e,t)`
- pub `handlePayloadWithCustom` method L1 — `handlePayloadWithCustom(e,t)`
- pub `has` method L1 — `has(e,t)`
- pub `hasDocument` method L1 — `hasDocument(e)`
- pub `head` method L1 — `head()`
- pub `hexEscapeSequenceAtom` method L1 — `hexEscapeSequenceAtom()`
- pub `hidden` method L1 — `hidden()`
- pub `hydrate` method L1 — `hydrate(e)`
- pub `hydrateAstNode` method L1 — `hydrateAstNode(e,t)`
- pub `hydrateCstLeafNode` method L1 — `hydrateCstLeafNode(e)`
- pub `hydrateCstNode` method L1 — `hydrateCstNode(e,t,n=0)`
- pub `hydrateReference` method L1 — `hydrateReference(e,t,n,r)`
- pub `identityEscapeAtom` method L1 — `identityEscapeAtom()`
- pub `includeEntry` method L1 — `includeEntry(e,t,n)`
- pub `includes` method L1 — `includes(e)`
- pub `indexOf` method L1 — `indexOf(e,t=0)`
- pub `initialize` method L1 — `initialize(e)`
- pub `initializeWorkspace` method L1 — `initializeWorkspace(e,t=yc.XO.None)`
- pub `initialized` method L1 — `initialized(e)`
- pub `integerIncludingZero` method L1 — `integerIncludingZero()`
- pub `invalidateDocument` method L1 — `invalidateDocument(e)`
- pub `invoke` method L1 — `invoke(...e)`
- pub `is` method L1 — `is(e)`
- pub `isAffected` method L1 — `isAffected(e,t)`
- pub `isAssertion` method L1 — `isAssertion()`
- pub `isAtom` method L1 — `isAtom()`
- pub `isCancellationRequested` method L1 — `isCancellationRequested()`
- pub `isClassAtom` method L1 — `isClassAtom(e=0)`
- pub `isDigit` method L1 — `isDigit()`
- pub `isEmpty` method L1 — `isEmpty()`
- pub `isEpsilon` method L1 — `isEpsilon()`
- pub `isFull` method L1 — `isFull(e)`
- pub `isIncremental` method L1 — `isIncremental(e)`
- pub `isInstance` method L1 — `isInstance(e,t)`
- pub `isPatternCharacter` method L1 — `isPatternCharacter()`
- pub `isQuantifier` method L1 — `isQuantifier()`
- pub `isRangeDash` method L1 — `isRangeDash()`
- pub `isRecording` method L1 — `isRecording()`
- pub `isRegExpFlag` method L1 — `isRegExpFlag()`
- pub `isSubtype` method L1 — `isSubtype(e,t)`
- pub `isTerm` method L1 — `isTerm()`
- pub `isUri` method L1 — `isUri(e)`
- pub `isValidToken` method L1 — `isValidToken(e)`
- pub `iterator` method L1 — `iterator()`
- pub `join` method L1 — `join(e=",")`
- pub `keepStackSize` method L1 — `keepStackSize()`
- pub `key` method L1 — `key()`
- pub `keys` method L1 — `keys()`
- pub `languageId` method L1 — `languageId()`
- pub `lastNonHiddenNode` method L1 — `lastNonHiddenNode()`
- pub `length` method L1 — `length()`
- pub `limit` method L1 — `limit(e)`
- pub `lineCount` method L1 — `lineCount()`
- pub `link` method L1 — `link(e,t=yc.XO.None)`
- pub `linkNode` method L1 — `linkNode(e,t,n,i,s,a)`
- pub `loadAdditionalDocuments` method L1 — `loadAdditionalDocuments(e,t)`
- pub `loadAstNode` method L1 — `loadAstNode(e)`
- pub `loc` method L1 — `loc(e)`
- pub `many` method L1 — `many(e,t)`
- pub `map` method L1 — `map(e)`
- pub `matchWithExec` method L1 — `matchWithExec(e,t)`
- pub `matchWithTest` method L1 — `matchWithTest(e,t,n)`
- pub `nonNullable` method L1 — `nonNullable()`
- pub `notifyBuildPhase` method L1 — `notifyBuildPhase(e,t,n)`
- pub `nulCharacterAtom` method L1 — `nulCharacterAtom()`
- pub `offset` method L1 — `offset()`
- pub `offsetAt` method L1 — `offsetAt(e)`
- pub `onBuildPhase` method L1 — `onBuildPhase(e,t)`
- pub `onCancellationRequested` method L1 — `onCancellationRequested()`
- pub `onDispose` method L1 — `onDispose(e)`
- pub `onUpdate` method L1 — `onUpdate(e)`
- pub `optional` method L1 — `optional(e,t)`
- pub `parent` method L1 — `parent()`
- pub `parse` method L1 — `parse(e)`
- pub `parseAsync` method L1 — `parseAsync(e,t,n)`
- pub `parseHexDigits` method L1 — `parseHexDigits(e)`
- pub `pattern` method L1 — `pattern(e)`
- pub `patternCharacter` method L1 — `patternCharacter()`
- pub `peekChar` method L1 — `peekChar(e=0)`
- pub `performNextOperation` method L1 — `performNextOperation()`
- pub `performSelfAnalysis` method L1 — `performSelfAnalysis(e)`
- pub `performStartup` method L1 — `performStartup(e)`
- pub `performSubruleAssignment` method L1 — `performSubruleAssignment(e,t,n)`
- pub `popChar` method L1 — `popChar()`
- pub `positionAt` method L1 — `positionAt(e)`
- pub `positiveInteger` method L1 — `positiveInteger()`
- pub `prepareBuild` method L1 — `prepareBuild(e,t)`
- pub `processLexingErrors` method L1 — `processLexingErrors(e,t,n)`
- pub `processLinkingErrors` method L1 — `processLinkingErrors(e,t,n)`
- pub `processNode` method L1 — `processNode(e,t,n)`
- pub `processParsingErrors` method L1 — `processParsingErrors(e,t,n)`
- pub `push` method L1 — `push(...e)`
- pub `quantifier` method L1 — `quantifier(e=!1)`
- pub `range` method L1 — `range()`
- pub `read` method L1 — `read(e)`
- pub `readDirectory` method L1 — `readDirectory()`
- pub `readFile` method L1 — `readFile()`
- pub `ready` method L1 — `ready()`
- pub `recursiveReduce` method L1 — `recursiveReduce(e,t,n)`
- pub `reduce` method L1 — `reduce(e,t)`
- pub `reduceRight` method L1 — `reduceRight(e,t)`
- pub `regExpUnicodeEscapeSequenceAtom` method L1 — `regExpUnicodeEscapeSequenceAtom()`
- pub `regexPatternFunction` method L1 — `regexPatternFunction(e)`
- pub `register` method L1 — `register(e)`
- pub `remove` method L1 — `remove(e,t=null)`
- pub `removeNode` method L1 — `removeNode(e)`
- pub `removeUnexpectedElements` method L1 — `removeUnexpectedElements()`
- pub `replacer` method L1 — `replacer(e,t,{refText:n,sourceText:s,textRegions:a,comments:o,uriConverter:c})`
- pub `requiresCustomPattern` method L1 — `requiresCustomPattern(e)`
- pub `resetStackSize` method L1 — `resetStackSize(e)`
- pub `resetState` method L1 — `resetState()`
- pub `resolveRefs` method L1 — `resolveRefs()`
- pub `restoreState` method L1 — `restoreState(e)`
- pub `revive` method L1 — `revive(e)`
- pub `reviveReference` method L1 — `reviveReference(e,t,n,i,s)`
- pub `rule` method L1 — `rule(e,t)`
- pub `runCancelable` method L1 — `runCancelable(e,t,n,r)`
- pub `runConverter` method L1 — `runConverter(e,t,n)`
- pub `saveState` method L1 — `saveState()`
- pub `serialize` method L1 — `serialize(e,t={})`
- pub `set` method L1 — `set(e,t)`
- pub `setParent` method L1 — `setParent(e,t)`
- pub `shouldRelink` method L1 — `shouldRelink(e,t)`
- pub `shouldValidate` method L1 — `shouldValidate(e)`
- pub `size` method L1 — `size()`
- pub `some` method L1 — `some(e)`
- pub `splice` method L1 — `splice(e,t,...n)`
- pub `startImplementation` method L1 — `startImplementation(e,t)`
- pub `startWalking` method L1 — `startWalking()`
- pub `subrule` method L1 — `subrule(e,t,n,r)`
- pub `tail` method L1 — `tail(e=1)`
- pub `term` method L1 — `term()`
- pub `text` method L1 — `text()`
- pub `throwIfDisposed` method L1 — `throwIfDisposed()`
- pub `toArray` method L1 — `toArray()`
- pub `toDiagnostic` method L1 — `toDiagnostic(e,t,n)`
- pub `toJSON` method L1 — `toJSON()`
- pub `toMap` method L1 — `toMap(e,t)`
- pub `toMarkdown` method L1 — `toMarkdown(e)`
- pub `toMarkdownDefault` method L1 — `toMarkdownDefault(e)`
- pub `toSectionName` method L1 — `toSectionName(e)`
- pub `toSet` method L1 — `toSet()`
- pub `toString` method L1 — `toString()`
- pub `toTokenTypeDictionary` method L1 — `toTokenTypeDictionary(e)`
- pub `tokenType` method L1 — `tokenType()`
- pub `tokenize` method L1 — `tokenize(e,t=this.defaultMode)`
- pub `tokenizeInternal` method L1 — `tokenizeInternal(e,t)`
- pub `traverseFolder` method L1 — `traverseFolder(e,t,n,r)`
- pub `unlink` method L1 — `unlink(e)`
- pub `unorderedGroups` method L1 — `unorderedGroups()`
- pub `unshift` method L1 — `unshift(...e)`
- pub `update` method L1 — `update(e,t)`
- pub `updateConfiguration` method L1 — `updateConfiguration(e)`
- pub `updateContent` method L1 — `updateContent(e,t=yc.XO.None)`
- pub `updateExpectedNext` method L1 — `updateExpectedNext()`
- pub `updateLastIndex` method L1 — `updateLastIndex(e,t)`
- pub `updateReferences` method L1 — `updateReferences(e,t=yc.XO.None)`
- pub `updateSectionConfiguration` method L1 — `updateSectionConfiguration(e,t)`
- pub `updateTokenEndLineColumnLocation` method L1 — `updateTokenEndLineColumnLocation(e,t,n,r,i,s,a)`
- pub `uri` method L1 — `uri()`
- pub `validate` method L1 — `validate(e)`
- pub `validateAmbiguousAlternationAlternatives` method L1 — `validateAmbiguousAlternationAlternatives(e,t)`
- pub `validateAst` method L1 — `validateAst(e,t,n=yc.XO.None)`
- pub `validateDocument` method L1 — `validateDocument(e,t={},n=yc.XO.None)`
- pub `validateEmptyOrAlternatives` method L1 — `validateEmptyOrAlternatives(e)`
- pub `validateNoLeftRecursion` method L1 — `validateNoLeftRecursion(e)`
- pub `validateSomeNonEmptyLookaheadPath` method L1 — `validateSomeNonEmptyLookaheadPath(e,t)`
- pub `values` method L1 — `values()`
- pub `version` method L1 — `version()`
- pub `visit` method L1 — `visit(e)`
- pub `visitAlternation` method L1 — `visitAlternation(e)`
- pub `visitAlternative` method L1 — `visitAlternative(e)`
- pub `visitCharacter` method L1 — `visitCharacter(e)`
- pub `visitChildren` method L1 — `visitChildren(e)`
- pub `visitDisjunction` method L1 — `visitDisjunction(e)`
- pub `visitEndAnchor` method L1 — `visitEndAnchor(e)`
- pub `visitFlags` method L1 — `visitFlags(e)`
- pub `visitGroup` method L1 — `visitGroup(e)`
- pub `visitGroupBackReference` method L1 — `visitGroupBackReference(e)`
- pub `visitLookahead` method L1 — `visitLookahead(e)`
- pub `visitNegativeLookahead` method L1 — `visitNegativeLookahead(e)`
- pub `visitNonTerminal` method L1 — `visitNonTerminal(e)`
- pub `visitNonWordBoundary` method L1 — `visitNonWordBoundary(e)`
- pub `visitOption` method L1 — `visitOption(e)`
- pub `visitPattern` method L1 — `visitPattern(e)`
- pub `visitQuantifier` method L1 — `visitQuantifier(e)`
- pub `visitRepetition` method L1 — `visitRepetition(e)`
- pub `visitRepetitionMandatory` method L1 — `visitRepetitionMandatory(e)`
- pub `visitRepetitionMandatoryWithSeparator` method L1 — `visitRepetitionMandatoryWithSeparator(e)`
- pub `visitRepetitionWithSeparator` method L1 — `visitRepetitionWithSeparator(e)`
- pub `visitRule` method L1 — `visitRule(e)`
- pub `visitSet` method L1 — `visitSet(e)`
- pub `visitStartAnchor` method L1 — `visitStartAnchor(e)`
- pub `visitTerminal` method L1 — `visitTerminal(e)`
- pub `visitWordBoundary` method L1 — `visitWordBoundary(e)`
- pub `waitUntil` method L1 — `waitUntil(e,t,n)`
- pub `walk` method L1 — `walk(e,t=[])`
- pub `walkAtLeastOne` method L1 — `walkAtLeastOne(e,t,n)`
- pub `walkAtLeastOneSep` method L1 — `walkAtLeastOneSep(e,t,n)`
- pub `walkFlat` method L1 — `walkFlat(e,t,n)`
- pub `walkMany` method L1 — `walkMany(e,t,n)`
- pub `walkManySep` method L1 — `walkManySep(e,t,n)`
- pub `walkOption` method L1 — `walkOption(e,t,n)`
- pub `walkOr` method L1 — `walkOr(e,t,n)`
- pub `walkProdRef` method L1 — `walkProdRef(e,t,n)`
- pub `walkTerminal` method L1 — `walkTerminal(e,t,n)`
- pub `with` method L1 — `with(e)`
- pub `wrapAtLeastOne` method L1 — `wrapAtLeastOne(e,t)`
- pub `wrapConsume` method L1 — `wrapConsume(e,t)`
- pub `wrapMany` method L1 — `wrapMany(e,t)`
- pub `wrapOption` method L1 — `wrapOption(e,t)`
- pub `wrapOr` method L1 — `wrapOr(e,t)`
- pub `wrapSelfAnalysis` method L1 — `wrapSelfAnalysis()`
- pub `wrapSubrule` method L1 — `wrapSubrule(e,t,n)`
- pub `wrapValidationException` method L1 — `wrapValidationException(e,t)`
- pub `write` method L1 — `write(e)`
-  `$c` function L1 — `function $c(e)`
-  `$e` function L1 — `function $e(e)`
-  `$i` function L1 — `function $i(e,t)`
-  `$l` function L1 — `function $l(e)`
-  `$r` function L1 — `function $r(e,t)`
-  `A` function L1 — `function A(e)`
-  `Ai` function L1 — `function Ai(e,t,n,r)`
-  `Bc` class L1 — `-`
-  `Be` class L1 — `-`
-  `Bl` class L1 — `-`
-  `Bs` function L1 — `function Bs(e,t)`
-  `Bt` class L1 — `-`
-  `Cc` function L1 — `function Cc(e)`
-  `Ce` function L1 — `function Ce(e)`
-  `Ci` function L1 — `function Ci(e,t,n)`
-  `Cl` function L1 — `function Cl(e,t)`
-  `Cr` function L1 — `function Cr(e)`
-  `D` function L1 — `-`
-  `Di` function L1 — `function Di(e,t,n,r=[])`
-  `Dl` class L1 — `-`
-  `Dr` function L1 — `function Dr(e)`
-  `Ds` class L1 — `-`
-  `E` function L1 — `function E(e,t)`
-  `Ec` class L1 — `-`
-  `Ee` function L1 — `function Ee(e)`
-  `Ei` function L1 — `function Ei(e,t,n,r)`
-  `Es` function L1 — `function Es(e)`
-  `Fc` class L1 — `-`
-  `Fi` class L1 — `-`
-  `Fs` function L1 — `function Fs(e,t,n)`
-  `G` function L1 — `function G(e)`
-  `Gc` class L1 — `-`
-  `Gi` class L1 — `-`
-  `Gl` class L1 — `-`
-  `Gs` function L1 — `function Gs(e,t,n,r,i)`
-  `Gt` class L1 — `-`
-  `Hc` class L1 — `-`
-  `Hl` function L1 — `function Hl(e,t,n,r,i,s,a,o,c)`
-  `Ho` class L1 — `-`
-  `Hs` function L1 — `function Hs(e,t,n)`
-  `Ht` class L1 — `-`
-  `Ie` function L1 — `function Ie(e,t)`
-  `Ii` class L1 — `-`
-  `Il` function L1 — `function Il(e,t)`
-  `Ir` function L1 — `function Ir(e,t)`
-  `J` function L1 — `function J(e)`
-  `Jc` class L1 — `-`
-  `Je` function L1 — `function Je(e,t,n,r)`
-  `Jn` function L1 — `function Jn(e,t,n)`
-  `Jo` function L1 — `-`
-  `Js` function L1 — `function Js(e,t=!0)`
-  `Jt` class L1 — `-`
-  `Kc` class L1 — `-`
-  `Ke` class L1 — `-`
-  `Kl` class L1 — `-`
-  `Ks` function L1 — `function Ks(e,t,n,r,i)`
-  `Kt` class L1 — `-`
-  `Le` function L1 — `function Le(e,t)`
-  `Li` function L1 — `function Li(e,t,n,r)`
-  `Ll` function L1 — `function Ll(e,t)`
-  `Lr` function L1 — `function Lr(e)`
-  `Ls` class L1 — `-`
-  `M` function L1 — `function M(e)`
-  `Mi` class L1 — `-`
-  `Ml` function L1 — `function Ml(e)`
-  `Mr` function L1 — `function Mr(e)`
-  `Ms` class L1 — `-`
-  `Nc` function L1 — `function Nc(e,t,n=0)`
-  `Ne` function L1 — `function Ne(e)`
-  `Ni` function L1 — `function Ni(e)`
-  `Nl` function L1 — `function Nl(e)`
-  `Oe` function L1 — `function Oe(e)`
-  `Oi` function L1 — `function Oi(e,t)`
-  `Ol` class L1 — `-`
-  `Or` function L1 — `function Or(e)`
-  `Os` class L1 — `-`
-  `P` function L1 — `function P()`
-  `Pi` function L1 — `function Pi(e)`
-  `Pl` class L1 — `-`
-  `Pr` class L1 — `-`
-  `Ps` class L1 — `-`
-  `Qc` class L1 — `-`
-  `Qe` function L1 — `function Qe(e,t,n)`
-  `Qi` class L1 — `-`
-  `Ql` class L1 — `-`
-  `Qn` function L1 — `function Qn(e,t,n)`
-  `Qo` function L1 — `function Qo(e)`
-  `Qs` class L1 — `-`
-  `Qt` function L1 — `function Qt(e)`
-  `Rc` function L1 — `function Rc(e)`
-  `Re` function L1 — `function Re(e,t)`
-  `Ri` function L1 — `function Ri(e)`
-  `Rn` class L1 — `-`
-  `Rs` function L1 — `function Rs(e,t)`
-  `Sc` function L1 — `function Sc(e,t)`
-  `Se` function L1 — `function Se(e,t)`
-  `Si` function L1 — `function Si(e)`
-  `Sl` function L1 — `function Sl(e)`
-  `T` function L1 — `function T(e)`
-  `Te` class L1 — `-`
-  `Tn` function L1 — `function Tn(e)`
-  `Tr` function L1 — `function Tr(e)`
-  `U` function L1 — `function U(e)`
-  `Uc` class L1 — `-`
-  `Ui` function L1 — `function Ui(e)`
-  `Ul` class L1 — `-`
-  `Us` function L1 — `function Us(e,t,n)`
-  `V` function L1 — `function V(e)`
-  `Vc` class L1 — `-`
-  `Vl` function L1 — `function Vl(e)`
-  `Vn` function L1 — `function Vn(e)`
-  `Vo` class L1 — `-`
-  `Vs` function L1 — `function Vs(e,t,n,r)`
-  `Vt` class L1 — `-`
-  `W` function L1 — `function W(e)`
-  `Wc` class L1 — `-`
-  `We` function L1 — `function We(e)`
-  `Wi` function L1 — `function Wi(e)`
-  `Wo` class L1 — `-`
-  `Wr` function L1 — `function Wr(e)`
-  `Ws` function L1 — `function Ws(e,t)`
-  `Wt` class L1 — `-`
-  `Xc` class L1 — `-`
-  `Xe` function L1 — `function Xe(e,t)`
-  `Xi` class L1 — `-`
-  `Xl` function L1 — `function Xl(e,t,n,r)`
-  `Xo` class L1 — `-`
-  `Xr` function L1 — `function Xr(e,t)`
-  `Xs` function L1 — `function Xs(e,t)`
-  `Xt` class L1 — `-`
-  `Y` function L1 — `function Y(e)`
-  `Yc` class L1 — `-`
-  `Ye` function L1 — `function Ye(e)`
-  `Yi` class L1 — `-`
-  `Yo` class L1 — `-`
-  `Yr` function L1 — `function Yr(e,t,n,r,i,s,a,o)`
-  `Ys` function L1 — `function Ys(e,t)`
-  `Yt` class L1 — `-`
-  `Zc` function L1 — `function Zc(e)`
-  `Ze` function L1 — `function Ze(e)`
-  `Zn` function L1 — `function Zn(e,t)`
-  `Zo` class L1 — `-`
-  `Zr` class L1 — `-`
-  `_` function L1 — `function _(e)`
-  `_e` function L1 — `function _e(e)`
-  `_i` function L1 — `function _i(e)`
-  `_l` class L1 — `-`
-  `_s` class L1 — `-`
-  `a` class L1 — `-`
-  `a` function L1 — `function a(e,t)`
-  `a` class L1 — `-`
-  `a` function L1 — `function a(e)`
-  `aa` function L1 — `function aa(e,t,n,r)`
-  `ac` function L1 — `function ac(e,t,n)`
-  `ae` function L1 — `function ae(e)`
-  `al` function L1 — `function al(e)`
-  `as` class L1 — `-`
-  `be` function L1 — `function be()`
-  `bi` function L1 — `function bi(e)`
-  `bl` class L1 — `-`
-  `bs` function L1 — `function bs(e,t,n)`
-  `c` class L1 — `-`
-  `c` function L1 — `function c(e=i.DD)`
-  `ca` function L1 — `function ca(e,t,n,r)`
-  `cc` function L1 — `function cc(e)`
-  `cl` class L1 — `-`
-  `cs` function L1 — `function cs(e,t)`
-  `ct` function L1 — `function ct(e)`
-  `d` function L1 — `function d(e)`
-  `da` function L1 — `function da(e)`
-  `dc` function L1 — `function dc(e,t,n,r)`
-  `di` class L1 — `-`
-  `dl` class L1 — `-`
-  `dr` function L1 — `function dr(e)`
-  `ds` function L1 — `function ds(e,t)`
-  `ea` function L1 — `function ea(e,t)`
-  `ec` class L1 — `-`
-  `el` class L1 — `-`
-  `er` function L1 — `function er(e)`
-  `es` class L1 — `-`
-  `et` function L1 — `function et(e,t,n)`
-  `fa` function L1 — `function fa(e,t)`
-  `fc` function L1 — `function fc(e,t)`
-  `fi` class L1 — `-`
-  `fl` class L1 — `-`
-  `fn` function L1 — `function fn(e)`
-  `g` function L1 — `function g(t,n)`
-  `ge` function L1 — `function ge(e)`
-  `gi` function L1 — `function gi(e,t,n=[])`
-  `gl` function L1 — `function gl(e)`
-  `gr` function L1 — `function gr(e,t,n)`
-  `gt` function L1 — `function gt(e)`
-  `h` class L1 — `-`
-  `ha` function L1 — `function ha(e,t,n,r)`
-  `hc` function L1 — `function hc(e,t)`
-  `hi` class L1 — `-`
-  `hl` class L1 — `-`
-  `hn` function L1 — `function hn(e,t=[])`
-  `hr` function L1 — `function hr(e)`
-  `i` function L1 — `function i(e)`
-  `ia` function L1 — `function ia(e,t=!0)`
-  `ie` function L1 — `function ie(e)`
-  `il` class L1 — `-`
-  `it` function L1 — `function it(e)`
-  `jc` class L1 — `-`
-  `jl` function L1 — `function jl(e)`
-  `jn` function L1 — `function jn(e)`
-  `jo` class L1 — `-`
-  `js` function L1 — `function js(e,t,n,r,...i)`
-  `jt` class L1 — `-`
-  `k` class L1 — `-`
-  `kc` class L1 — `-`
-  `ke` function L1 — `function ke(e,t)`
-  `ki` function L1 — `function ki(e,t,n)`
-  `kl` function L1 — `function kl(e,t)`
-  `kr` function L1 — `function kr(e)`
-  `ks` function L1 — `function ks(e)`
-  `l` function L1 — `function l(e)`
-  `la` function L1 — `function la(e,t,n)`
-  `lc` function L1 — `function lc(e)`
-  `li` class L1 — `-`
-  `ll` class L1 — `-`
-  `lr` function L1 — `function lr(e)`
-  `ls` function L1 — `function ls(e,t)`
-  `lt` function L1 — `function lt(e,t)`
-  `m` class L1 — `-`
-  `m` function L1 — `function m(e)`
-  `ma` function L1 — `function ma(e,t)`
-  `mc` class L1 — `-`
-  `mi` class L1 — `-`
-  `ml` function L1 — `function ml(e)`
-  `mn` function L1 — `function mn(e,t,n)`
-  `mr` function L1 — `function mr(e)`
-  `n` function L1 — `function n(e)`
-  `nc` class L1 — `-`
-  `nl` function L1 — `function nl(e)`
-  `nr` function L1 — `function nr(e,t)`
-  `nt` function L1 — `function nt(e,t)`
-  `o` function L1 — `function o(e=i.DD)`
-  `oa` function L1 — `function oa(e,t,n,r,i,s)`
-  `oc` function L1 — `function oc(e,t,n=!1)`
-  `ol` class L1 — `-`
-  `ot` function L1 — `function ot(e,t)`
-  `p` function L1 — `function p(...e)`
-  `pa` function L1 — `function pa(e,t)`
-  `pc` class L1 — `-`
-  `pe` function L1 — `function pe(e)`
-  `pi` class L1 — `-`
-  `pl` class L1 — `-`
-  `pn` class L1 — `-`
-  `pr` function L1 — `function pr(e,t)`
-  `q` function L1 — `function q(e)`
-  `qc` class L1 — `-`
-  `qe` function L1 — `function qe(e,t,n)`
-  `qi` class L1 — `-`
-  `ql` function L1 — `function ql(e,t)`
-  `qn` function L1 — `function qn(e,t=!1)`
-  `qt` class L1 — `-`
-  `r` function L1 — `function r(e)`
-  `ra` class L1 — `-`
-  `rc` class L1 — `-`
-  `rl` class L1 — `-`
-  `rt` function L1 — `function rt(e)`
-  `s` class L1 — `-`
-  `s` function L1 — `function s(e)`
-  `sa` function L1 — `function sa(e,t,n,r)`
-  `sc` class L1 — `-`
-  `sl` function L1 — `function sl(e)`
-  `ss` function L1 — `function ss(e,t,n)`
-  `st` function L1 — `function st(e)`
-  `t` function L1 — `function t()`
-  `t` class L1 — `-`
-  `t` function L1 — `const t = ()`
-  `ta` class L1 — `-`
-  `tc` class L1 — `-`
-  `te` function L1 — `function te(e)`
-  `tl` class L1 — `-`
-  `tr` class L1 — `-`
-  `ts` function L1 — `function ts(e,t,n,r,i,s,a)`
-  `tt` function L1 — `function tt(e)`
-  `tu` function L1 — `function tu(e)`
-  `u` class L1 — `-`
-  `u` function L1 — `const u = ()`
-  `u` class L1 — `-`
-  `ua` function L1 — `function ua(e,t)`
-  `uc` function L1 — `function uc(e,t,n=t.terminal)`
-  `ue` function L1 — `function ue(e)`
-  `ui` class L1 — `-`
-  `ul` class L1 — `-`
-  `ur` function L1 — `function ur(e)`
-  `us` function L1 — `function us(e,t)`
-  `v` function L1 — `function v(e)`
-  `vc` function L1 — `function vc(e)`
-  `vi` function L1 — `function vi(e)`
-  `vl` function L1 — `function vl(e,t,n,r)`
-  `vs` function L1 — `function vs(e,t,n,r=!1)`
-  `wc` function L1 — `function wc(e)`
-  `we` function L1 — `function we(e,t)`
-  `wi` function L1 — `function wi(e,t,n,r)`
-  `wl` function L1 — `function wl(e)`
-  `wr` function L1 — `function wr(e)`
-  `ws` function L1 — `function ws(e=void 0)`
-  `x` function L1 — `function x(e)`
-  `xe` function L1 — `function xe(e,t)`
-  `xi` class L1 — `-`
-  `xl` function L1 — `function xl(e)`
-  `xr` function L1 — `function xr(e,t)`
-  `y` function L1 — `function y(e,t)`
-  `yi` function L1 — `function yi(e,t,n,r)`
-  `yl` function L1 — `function yl(e)`
-  `zc` class L1 — `-`
-  `ze` function L1 — `function ze(e)`
-  `zi` class L1 — `-`
-  `zl` function L1 — `function zl(e,t)`
-  `zn` function L1 — `function zn(e)`
-  `zo` class L1 — `-`
-  `zs` function L1 — `function zs(e,t,n,r)`
-  `zt` class L1 — `-`

#### docs/themes/hugo-geekdoc/static/js/763-66119f34.chunk.min.js

-  `i` function L1 — `function i(e,t)`

#### docs/themes/hugo-geekdoc/static/js/790-2b300153.chunk.min.js

-  `C` function L1 — `function C()`
-  `nt` function L1 — `function nt(t)`
-  `rt` function L1 — `function rt()`

#### docs/themes/hugo-geekdoc/static/js/802-4ae1987e.chunk.min.js

-  `$` function L1 — `function $()`
-  `B` function L1 — `function B(t)`
-  `E` function L1 — `function E()`
-  `F` function L1 — `function F(t,i)`
-  `G` function L1 — `function G(t,i)`
-  `H` function L1 — `function H(t)`
-  `I` function L1 — `function I()`
-  `M` function L1 — `function M()`
-  `N` function L1 — `function N(t)`
-  `O` function L1 — `function O(t)`
-  `Q` function L1 — `function Q()`
-  `U` function L1 — `function U(t)`
-  `V` function L1 — `function V(t)`
-  `W` function L1 — `function W(t)`
-  `X` function L1 — `function X(t,i)`
-  `Y` function L1 — `function Y(t)`
-  `Z` function L1 — `function Z()`
-  `b` function L1 — `function b(t,i,e,s)`
-  `c` function L1 — `function c(t)`
-  `g` function L1 — `function g(t)`
-  `j` function L1 — `function j(t,i)`
-  `l` function L1 — `function l(t)`
-  `m` function L1 — `function m()`
-  `q` function L1 — `function q()`
-  `u` function L1 — `function u(t)`
-  `w` function L1 — `function w(t,i,e)`
-  `y` function L1 — `function y(t,i,e,s)`
-  `z` function L1 — `function z(t)`

#### docs/themes/hugo-geekdoc/static/js/840-6b7093bb.chunk.min.js

- pub `_removeFromParentsChildList` method L1 — `_removeFromParentsChildList(t)`
- pub `children` method L1 — `children(t)`
- pub `constructor` method L1 — `constructor(t={})`
- pub `edge` method L1 — `edge(t,e,r)`
- pub `edgeCount` method L1 — `edgeCount()`
- pub `edges` method L1 — `edges()`
- pub `filterNodes` method L1 — `filterNodes(t)`
- pub `graph` method L1 — `graph()`
- pub `hasEdge` method L1 — `hasEdge(t,e,r)`
- pub `hasNode` method L1 — `hasNode(t)`
- pub `inEdges` method L1 — `inEdges(t,e)`
- pub `isCompound` method L1 — `isCompound()`
- pub `isDirected` method L1 — `isDirected()`
- pub `isLeaf` method L1 — `isLeaf(t)`
- pub `isMultigraph` method L1 — `isMultigraph()`
- pub `neighbors` method L1 — `neighbors(t)`
- pub `node` method L1 — `node(t)`
- pub `nodeCount` method L1 — `nodeCount()`
- pub `nodeEdges` method L1 — `nodeEdges(t,e)`
- pub `nodes` method L1 — `nodes()`
- pub `outEdges` method L1 — `outEdges(t,e)`
- pub `parent` method L1 — `parent(t)`
- pub `predecessors` method L1 — `predecessors(t)`
- pub `removeEdge` method L1 — `removeEdge(t,e,r)`
- pub `removeNode` method L1 — `removeNode(t)`
- pub `setDefaultEdgeLabel` method L1 — `setDefaultEdgeLabel(t)`
- pub `setDefaultNodeLabel` method L1 — `setDefaultNodeLabel(t)`
- pub `setEdge` method L1 — `setEdge()`
- pub `setGraph` method L1 — `setGraph(t)`
- pub `setNode` method L1 — `setNode(t,e)`
- pub `setNodes` method L1 — `setNodes(t,e)`
- pub `setParent` method L1 — `setParent(t,e)`
- pub `setPath` method L1 — `setPath(t,e)`
- pub `sinks` method L1 — `sinks()`
- pub `sources` method L1 — `sources()`
- pub `successors` method L1 — `successors(t)`
-  `At` function L1 — `function At(t,e,r,s)`
-  `Dt` function L1 — `function Dt(t,e)`
-  `Et` function L1 — `function Et(t,e,r,s)`
-  `J` function L1 — `function J(t,e)`
-  `K` function L1 — `function K(t)`
-  `Kt` function L1 — `function Kt(t,e,r)`
-  `L` function L1 — `function L(t,e)`
-  `Lt` function L1 — `function Lt(t,e,r,s)`
-  `N` function L1 — `function N(t)`
-  `Q` function L1 — `function Q(t,e,r=0,s=0)`
-  `St` function L1 — `function St(t,e,r)`
-  `T` function L1 — `function T(t)`
-  `Vt` function L1 — `function Vt(t,e,r,s)`
-  `_` function L1 — `function _(t,e,r,s)`
-  `a` function L1 — `function a(t)`
-  `at` function L1 — `function at(t)`
-  `be` function L1 — `function be(t,e,r,s,a)`
-  `de` function L1 — `function de(t,e,r=!1)`
-  `et` function L1 — `function et(t,{minX:e,minY:r,maxX:s,maxY:a}={minX:0,minY:0,maxX:0,maxY:0})`
-  `f` class L1 — `-`
-  `f` function L1 — `function f()`
-  `ge` function L1 — `function ge(t,e,r)`
-  `gt` function L1 — `function gt(t,e)`
-  `he` function L1 — `function he(t,e,r)`
-  `i` function L1 — `const i = (t,e)`
-  `m` function L1 — `function m(t,e)`
-  `pe` function L1 — `function pe(t,e,r)`
-  `rt` function L1 — `function rt(t)`
-  `s` function L1 — `function s()`
-  `st` function L1 — `function st(t,e)`
-  `tt` function L1 — `function tt(t,e)`
-  `ue` function L1 — `function ue(t,e,r,s)`
-  `w` function L1 — `function w(t,e)`
-  `wt` function L1 — `function wt(t,e)`
-  `ye` function L1 — `function ye(t,e,r)`

#### docs/themes/hugo-geekdoc/static/js/colortheme-662de488.bundle.min.js

-  `r` function L1 — `function r(n)`
-  `s` function L1 — `function s(r=!0)`

#### docs/themes/hugo-geekdoc/static/js/katex-81adfa46.bundle.min.js

- pub `_getExpansion` method L1 — `_getExpansion(e)`
- pub `baseSizingClasses` method L1 — `baseSizingClasses()`
- pub `beginGroup` method L1 — `beginGroup()`
- pub `callFunction` method L1 — `callFunction(e,t,r,a,n)`
- pub `constructor` method L1 — `constructor(e,t,r)`
- pub `consume` method L1 — `consume()`
- pub `consumeArg` method L1 — `consumeArg(e)`
- pub `consumeArgs` method L1 — `consumeArgs(e,t)`
- pub `consumeSpaces` method L1 — `consumeSpaces()`
- pub `countExpansion` method L1 — `countExpansion(e)`
- pub `cramp` method L1 — `cramp()`
- pub `endGroup` method L1 — `endGroup()`
- pub `endGroups` method L1 — `endGroups()`
- pub `expandAfterFuture` method L1 — `expandAfterFuture()`
- pub `expandMacro` method L1 — `expandMacro(e)`
- pub `expandMacroAsText` method L1 — `expandMacroAsText(e)`
- pub `expandNextToken` method L1 — `expandNextToken()`
- pub `expandOnce` method L1 — `expandOnce(e)`
- pub `expandTokens` method L1 — `expandTokens(e)`
- pub `expect` method L1 — `expect(e,t)`
- pub `extend` method L1 — `extend(e)`
- pub `feed` method L1 — `feed(e)`
- pub `fetch` method L1 — `fetch()`
- pub `fontMetrics` method L1 — `fontMetrics()`
- pub `formLigatures` method L1 — `formLigatures(e)`
- pub `formatUnsupportedCmd` method L1 — `formatUnsupportedCmd(e)`
- pub `fracDen` method L1 — `fracDen()`
- pub `fracNum` method L1 — `fracNum()`
- pub `future` method L1 — `future()`
- pub `get` method L1 — `get(e)`
- pub `getAttribute` method L1 — `getAttribute(e)`
- pub `getColor` method L1 — `getColor()`
- pub `handleInfixNodes` method L1 — `handleInfixNodes(e)`
- pub `handleSupSubscript` method L1 — `handleSupSubscript(e)`
- pub `has` method L1 — `has(e)`
- pub `hasClass` method L1 — `hasClass(e)`
- pub `havingBaseSizing` method L1 — `havingBaseSizing()`
- pub `havingBaseStyle` method L1 — `havingBaseStyle(e)`
- pub `havingCrampedStyle` method L1 — `havingCrampedStyle()`
- pub `havingSize` method L1 — `havingSize(e)`
- pub `havingStyle` method L1 — `havingStyle(e)`
- pub `isDefined` method L1 — `isDefined(e)`
- pub `isExpandable` method L1 — `isExpandable(e)`
- pub `isTight` method L1 — `isTight()`
- pub `isTrusted` method L1 — `isTrusted(e)`
- pub `lex` method L1 — `lex()`
- pub `parse` method L1 — `parse()`
- pub `parseArgumentGroup` method L1 — `parseArgumentGroup(e,t)`
- pub `parseArguments` method L1 — `parseArguments(e,t)`
- pub `parseAtom` method L1 — `parseAtom(e)`
- pub `parseColorGroup` method L1 — `parseColorGroup(e)`
- pub `parseExpression` method L1 — `parseExpression(e,t)`
- pub `parseFunction` method L1 — `parseFunction(e,t)`
- pub `parseGroup` method L1 — `parseGroup(e,t)`
- pub `parseGroupOfType` method L1 — `parseGroupOfType(e,t,r)`
- pub `parseRegexGroup` method L1 — `parseRegexGroup(e,t)`
- pub `parseSizeGroup` method L1 — `parseSizeGroup(e)`
- pub `parseStringGroup` method L1 — `parseStringGroup(e,t)`
- pub `parseSymbol` method L1 — `parseSymbol()`
- pub `parseUrlGroup` method L1 — `parseUrlGroup(e)`
- pub `popToken` method L1 — `popToken()`
- pub `pushToken` method L1 — `pushToken(e)`
- pub `pushTokens` method L1 — `pushTokens(e)`
- pub `range` method L1 — `range(e,t)`
- pub `reportNonstrict` method L1 — `reportNonstrict(e,t,r)`
- pub `scanArgument` method L1 — `scanArgument(e)`
- pub `set` method L1 — `set(e,t,r)`
- pub `setAttribute` method L1 — `setAttribute(e,t)`
- pub `setCatcode` method L1 — `setCatcode(e,t)`
- pub `sizingClasses` method L1 — `sizingClasses(e)`
- pub `sub` method L1 — `sub()`
- pub `subparse` method L1 — `subparse(e)`
- pub `sup` method L1 — `sup()`
- pub `switchMode` method L1 — `switchMode(e)`
- pub `text` method L1 — `text()`
- pub `toMarkup` method L1 — `toMarkup()`
- pub `toNode` method L1 — `toNode()`
- pub `toText` method L1 — `toText()`
- pub `useStrictBehavior` method L1 — `useStrictBehavior(e,t,r)`
- pub `withColor` method L1 — `withColor(e)`
- pub `withFont` method L1 — `withFont(e)`
- pub `withPhantom` method L1 — `withPhantom()`
- pub `withTextFontFamily` method L1 — `withTextFontFamily(e)`
- pub `withTextFontShape` method L1 — `withTextFontShape(e)`
- pub `withTextFontWeight` method L1 — `withTextFontWeight(e)`
-  `At` class L1 — `-`
-  `Bt` class L1 — `-`
-  `Dr` function L1 — `function Dr(e)`
-  `Et` function L1 — `function Et(e,t,r,a,n)`
-  `Fr` function L1 — `function Fr(e)`
-  `Ga` class L1 — `-`
-  `Gr` function L1 — `function Gr(e)`
-  `Gt` function L1 — `function Gt(e,t)`
-  `Ha` class L1 — `-`
-  `Hr` function L1 — `function Hr(e)`
-  `I` class L1 — `-`
-  `Ia` class L1 — `-`
-  `Ir` function L1 — `function Ir(e,t)`
-  `J` class L1 — `-`
-  `Jt` function L1 — `function Jt(e,t)`
-  `L` function L1 — `function L(e,t,r)`
-  `Mt` function L1 — `function Mt(e,t)`
-  `Q` class L1 — `-`
-  `Tt` function L1 — `function Tt(e)`
-  `Ur` function L1 — `function Ur(e)`
-  `Ut` function L1 — `function Ut(e)`
-  `Vr` function L1 — `function Vr(e,t)`
-  `Wa` class L1 — `-`
-  `X` class L1 — `-`
-  `Zt` function L1 — `function Zt(e,t)`
-  `_r` function L1 — `function _r(e,t,r)`
-  `_t` function L1 — `function _t(e)`
-  `a` class L1 — `-`
-  `ae` class L1 — `-`
-  `b` function L1 — `function b(e)`
-  `ee` class L1 — `-`
-  `ga` function L1 — `function ga(e,t,r)`
-  `ht` function L1 — `function ht(e)`
-  `i` class L1 — `-`
-  `ie` class L1 — `-`
-  `k` function L1 — `function k()`
-  `lt` function L1 — `function lt(e)`
-  `me` function L1 — `function me(e,t,r,a,n,i)`
-  `n` class L1 — `-`
-  `ne` class L1 — `-`
-  `oe` function L1 — `function oe(e)`
-  `q` function L1 — `function q(e)`
-  `r` function L1 — `function r(a)`
-  `re` class L1 — `-`
-  `rr` function L1 — `function rr(e,t,r)`
-  `w` function L1 — `function w()`
-  `x` class L1 — `-`
-  `x` function L1 — `function x(e)`
-  `y` class L1 — `-`
-  `zt` function L1 — `function zt(e,t)`

#### docs/themes/hugo-geekdoc/static/js/main-2e274343.bundle.min.js

-  `a` function L2 — `function a(t,e)`
-  `c` function L2 — `function c(t,e,n,r)`
-  `e` function L2 — `function e()`
-  `g` function L2 — `function g(t)`
-  `h` function L2 — `function h(t,e)`
-  `m` function L2 — `function m(t,e)`
-  `n` function L2 — `function n(o)`
-  `p` function L2 — `function p(t)`
-  `r` function L2 — `function r(t,e,n,o,r)`
-  `s` function L2 — `function s(t)`
-  `v` function L2 — `function v(t,e)`
-  `y` function L2 — `function y(t)`

#### docs/themes/hugo-geekdoc/static/js/mermaid-16393d09.bundle.min.js

- pub `_d` method L2 — `_d(t,e,r)`
- pub `_drawToContext` method L2 — `_drawToContext(t,e,r,n="nonzero")`
- pub `_fillPolygons` method L2 — `_fillPolygons(t,e)`
- pub `_mergedShape` method L2 — `_mergedShape(t)`
- pub `_o` method L2 — `_o(t)`
- pub `arc` method L2 — `arc(t,e,r,n,i,a,o=!1,s)`
- pub `arcTo` method L2 — `arcTo(t,e,r,n,i)`
- pub `areaEnd` method L2 — `areaEnd()`
- pub `areaStart` method L2 — `areaStart()`
- pub `autolink` method L2 — `autolink(t)`
- pub `bezierCurveTo` method L2 — `bezierCurveTo(t,e,r,n,i,a)`
- pub `blockTokens` method L2 — `blockTokens(t,e=[],r=!1)`
- pub `blockquote` method L2 — `blockquote(t)`
- pub `br` method L2 — `br(t)`
- pub `checkbox` method L2 — `checkbox({checked:t})`
- pub `circle` method L2 — `circle(t,e,r,n)`
- pub `closePath` method L2 — `closePath()`
- pub `code` method L2 — `code(t)`
- pub `codespan` method L2 — `codespan(t)`
- pub `constructor` method L2 — `constructor(t)`
- pub `curve` method L2 — `curve(t,e)`
- pub `dashedLine` method L2 — `dashedLine(t,e)`
- pub `def` method L2 — `def(t)`
- pub `del` method L2 — `del(t)`
- pub `delete` method L2 — `delete(t)`
- pub `dotsOnLines` method L2 — `dotsOnLines(t,e)`
- pub `draw` method L2 — `draw(t)`
- pub `ellipse` method L2 — `ellipse(t,e,r,n,i)`
- pub `em` method L2 — `em({tokens:t})`
- pub `emStrong` method L2 — `emStrong(t,e,r="")`
- pub `escape` method L2 — `escape(t)`
- pub `fences` method L2 — `fences(t)`
- pub `fillPolygons` method L2 — `fillPolygons(t,e)`
- pub `fillSketch` method L2 — `fillSketch(t,e)`
- pub `generator` method L2 — `generator()`
- pub `get` method L2 — `get(t)`
- pub `getDefaultOptions` method L2 — `getDefaultOptions()`
- pub `has` method L2 — `has(t)`
- pub `heading` method L2 — `heading(t)`
- pub `hr` method L2 — `hr(t)`
- pub `html` method L2 — `html(t)`
- pub `image` method L2 — `image({href:t,title:e,text:r})`
- pub `inline` method L2 — `inline(t,e=[])`
- pub `inlineText` method L2 — `inlineText(t)`
- pub `inlineTokens` method L2 — `inlineTokens(t,e=[])`
- pub `lex` method L2 — `lex(t,e)`
- pub `lexInline` method L2 — `lexInline(t,e)`
- pub `lheading` method L2 — `lheading(t)`
- pub `line` method L2 — `line(t,e,r,n,i)`
- pub `lineEnd` method L2 — `lineEnd()`
- pub `lineStart` method L2 — `lineStart()`
- pub `lineTo` method L2 — `lineTo(t,e)`
- pub `linearPath` method L2 — `linearPath(t,e)`
- pub `link` method L2 — `link(t)`
- pub `list` method L2 — `list(t)`
- pub `listitem` method L2 — `listitem(t)`
- pub `moveTo` method L2 — `moveTo(t,e)`
- pub `newSeed` method L2 — `newSeed()`
- pub `next` method L2 — `next()`
- pub `opsToPath` method L2 — `opsToPath(t,e)`
- pub `paragraph` method L2 — `paragraph(t)`
- pub `parse` method L2 — `parse(t,e)`
- pub `parseInline` method L2 — `parseInline(t,e)`
- pub `path` method L2 — `path(t,e)`
- pub `point` method L2 — `point(t,e)`
- pub `polygon` method L2 — `polygon(t,e)`
- pub `postprocess` method L2 — `postprocess(t)`
- pub `preprocess` method L2 — `preprocess(t)`
- pub `processAllTokens` method L2 — `processAllTokens(t)`
- pub `quadraticCurveTo` method L2 — `quadraticCurveTo(t,e,r,n)`
- pub `rect` method L2 — `rect(t,e,r,n)`
- pub `rectangle` method L2 — `rectangle(t,e,r,n,i)`
- pub `reflink` method L2 — `reflink(t,e)`
- pub `renderLines` method L2 — `renderLines(t,e)`
- pub `rules` method L2 — `rules()`
- pub `set` method L2 — `set(t,e)`
- pub `space` method L2 — `space(t)`
- pub `strong` method L2 — `strong({tokens:t})`
- pub `table` method L2 — `table(t)`
- pub `tablecell` method L2 — `tablecell(t)`
- pub `tablerow` method L2 — `tablerow({text:t})`
- pub `tag` method L2 — `tag(t)`
- pub `text` method L2 — `text(t)`
- pub `toPaths` method L2 — `toPaths(t)`
- pub `toString` method L2 — `toString()`
- pub `url` method L2 — `url(t)`
- pub `zigzagLines` method L2 — `zigzagLines(t,e,r)`
-  `$` function L2 — `function $(t,e)`
-  `$a` function L2 — `function $a(t,e)`
-  `$e` function L2 — `function $e(t)`
-  `$n` function L2 — `function $n()`
-  `$o` function L2 — `function $o(t)`
-  `$r` function L2 — `function $r(t)`
-  `$s` function L2 — `function $s(t)`
-  `$t` function L2 — `function $t()`
-  `A` function L2 — `function A(t,e)`
-  `Aa` function L2 — `function Aa(t,e)`
-  `Ae` function L2 — `function Ae(t,e)`
-  `An` function L2 — `function An(t)`
-  `As` function L2 — `function As(t,e,r)`
-  `At` function L2 — `function At(t,e,r)`
-  `B` function L2 — `function B(t,e,r,n,i,a,o,s,l)`
-  `Ba` function L2 — `function Ba(t,e)`
-  `Be` function L2 — `function Be(t,e)`
-  `Bn` function L2 — `function Bn(t,e,r)`
-  `Bs` function L2 — `function Bs(t)`
-  `Bt` function L2 — `function Bt()`
-  `C` function L2 — `function C(t,e,r)`
-  `Ca` function L2 — `function Ca(t,e)`
-  `Ce` function L2 — `function Ce(t,e,r)`
-  `Ci` function L2 — `function Ci(t)`
-  `Cn` function L2 — `function Cn(t,e)`
-  `Ct` function L2 — `function Ct()`
-  `D` function L2 — `function D(t)`
-  `Da` function L2 — `function Da(t,e)`
-  `De` function L2 — `function De()`
-  `Do` function L2 — `function Do(t)`
-  `Ds` function L2 — `function Ds(t)`
-  `Dt` function L2 — `function Dt(t,e)`
-  `E` function L2 — `function E(t)`
-  `Ea` function L2 — `function Ea(t,e)`
-  `Ee` function L2 — `function Ee(t,e,r)`
-  `Ei` function L2 — `function Ei(t,e,r,n,i,a)`
-  `Eo` function L2 — `function Eo(t)`
-  `Es` function L2 — `function Es(t)`
-  `Et` function L2 — `function Et(t)`
-  `F` function L2 — `function F(t,e)`
-  `Fa` function L2 — `function Fa(t,e)`
-  `Fe` function L2 — `function Fe(t)`
-  `Fn` function L2 — `function Fn(t,e)`
-  `Fo` class L2 — `-`
-  `Fs` function L2 — `function Fs(t)`
-  `Ft` function L2 — `function Ft(t,e)`
-  `G` function L2 — `function G(t,e,r)`
-  `Ga` function L2 — `function Ga(t)`
-  `Ge` function L2 — `function Ge(t)`
-  `Gi` function L2 — `function Gi(t)`
-  `Gn` function L2 — `function Gn()`
-  `Go` function L2 — `function Go()`
-  `Gr` function L2 — `function Gr(t,e,r,n)`
-  `Gt` function L2 — `function Gt()`
-  `H` function L2 — `function H(t,e=0)`
-  `Ha` function L2 — `function Ha(t,e)`
-  `He` function L2 — `function He(t)`
-  `Ho` function L2 — `function Ho(t,e)`
-  `Hr` function L2 — `function Hr(t)`
-  `Ht` function L2 — `function Ht(t,e)`
-  `I` function L2 — `function I(t,e,r=1)`
-  `Ia` function L2 — `function Ia(t)`
-  `Ie` function L2 — `function Ie(t)`
-  `In` function L2 — `function In(t)`
-  `Io` function L2 — `function Io(t)`
-  `Is` function L2 — `function Is(t,e)`
-  `It` function L2 — `function It(t,e)`
-  `J` function L2 — `function J(t)`
-  `Ja` function L2 — `function Ja(t,e)`
-  `Je` function L2 — `function Je(t,e,r,n,i)`
-  `Ji` function L2 — `function Ji(t,e,r)`
-  `Jo` function L2 — `function Jo(t)`
-  `Jr` class L2 — `-`
-  `Jt` function L2 — `function Jt(t)`
-  `K` function L2 — `function K(t,e,r)`
-  `Ka` function L2 — `function Ka(t,e)`
-  `Ke` function L2 — `function Ke(t,e,r,n)`
-  `Ki` function L2 — `function Ki(t,e,r)`
-  `Ko` function L2 — `function Ko(t)`
-  `Ks` function L2 — `function Ks(t,e,r)`
-  `Kt` function L2 — `function Kt(t)`
-  `L` function L2 — `function L(t,e)`
-  `La` function L2 — `function La(t)`
-  `Le` function L2 — `function Le(t)`
-  `Ln` function L2 — `function Ln(t,e,r)`
-  `Lo` function L2 — `function Lo(t)`
-  `Ls` function L2 — `function Ls(t)`
-  `Lt` function L2 — `function Lt(t,e)`
-  `M` function L2 — `function M(t,e,r,n)`
-  `Ma` function L2 — `function Ma(t)`
-  `Me` function L2 — `function Me(t,e)`
-  `Mn` function L2 — `function Mn(t,e)`
-  `Mr` function L2 — `function Mr(t,e)`
-  `Ms` function L2 — `function Ms(t)`
-  `Mt` function L2 — `function Mt(t,e)`
-  `N` function L2 — `function N(t,e,r,n,i,a=!1)`
-  `Na` function L2 — `function Na(t,e)`
-  `Ne` function L2 — `function Ne(t)`
-  `Nn` function L2 — `function Nn(t,e)`
-  `No` function L2 — `function No(t)`
-  `Ns` function L2 — `function Ns(t)`
-  `Nt` function L2 — `function Nt(t,e)`
-  `O` function L2 — `function O(t,e,r,n=1)`
-  `Oa` function L2 — `function Oa(t,e)`
-  `Oe` function L2 — `function Oe()`
-  `On` function L2 — `function On(t)`
-  `Oo` function L2 — `function Oo(t)`
-  `Os` function L2 — `function Os(t)`
-  `Ot` function L2 — `function Ot(t,e,r)`
-  `P` function L2 — `function P(t,e,r)`
-  `Pa` function L2 — `function Pa(t,e)`
-  `Pe` function L2 — `function Pe(t)`
-  `Pi` function L2 — `function Pi(t)`
-  `Pn` function L2 — `function Pn(t,e)`
-  `Po` function L2 — `function Po()`
-  `Ps` function L2 — `function Ps(t)`
-  `Pt` function L2 — `function Pt(t,e,{config:{themeVariables:r}})`
-  `Q` function L2 — `function Q(t,e=.15,r)`
-  `Qa` function L2 — `function Qa(t,e)`
-  `Qe` function L2 — `function Qe(t,e,r)`
-  `Qi` function L2 — `function Qi(t,e,r)`
-  `Qo` function L2 — `function Qo(t)`
-  `Qr` function L2 — `function Qr(t,e)`
-  `Qt` function L2 — `function Qt()`
-  `R` function L2 — `function R(t,e,r,n,i,a,o)`
-  `Ra` function L2 — `function Ra(t,e)`
-  `Re` function L2 — `function Re(t,e,r,n)`
-  `Ri` function L2 — `function Ri(t)`
-  `Rn` function L2 — `function Rn(t)`
-  `Ro` function L2 — `function Ro(t,e,r,n,i,a,o)`
-  `Rs` function L2 — `function Rs(t)`
-  `Rt` function L2 — `function Rt(t,e)`
-  `S` function L2 — `function S(t)`
-  `Sa` function L2 — `function Sa(t,e)`
-  `Se` function L2 — `function Se(t)`
-  `Sn` function L2 — `function Sn(t)`
-  `So` function L2 — `function So(t)`
-  `Sr` function L2 — `function Sr()`
-  `Ss` function L2 — `function Ss(t,e,r)`
-  `St` function L2 — `function St(t,e)`
-  `T` function L2 — `function T(e)`
-  `Ta` function L2 — `function Ta(t)`
-  `Te` function L2 — `function Te(t)`
-  `Ts` function L2 — `function Ts(t,e)`
-  `Tt` function L2 — `function Tt(t,e)`
-  `U` function L2 — `function U(t,e)`
-  `Ua` function L2 — `function Ua(t)`
-  `Ue` function L2 — `function Ue(t)`
-  `Uo` function L2 — `function Uo(t,e)`
-  `Ur` function L2 — `function Ur(t)`
-  `Ut` function L2 — `function Ut(t,e)`
-  `V` function L2 — `function V(t,e,r,n)`
-  `Va` function L2 — `function Va(t,e)`
-  `Ve` function L2 — `function Ve(t,e,r,n)`
-  `Vi` function L2 — `function Vi(t)`
-  `Vo` function L2 — `function Vo()`
-  `Vr` function L2 — `function Vr(t)`
-  `Vt` function L2 — `function Vt()`
-  `W` function L2 — `function W(t)`
-  `Wa` function L2 — `function Wa(t,e)`
-  `We` function L2 — `function We(t)`
-  `Wo` function L2 — `function Wo(t)`
-  `Wr` function L2 — `function Wr(t)`
-  `Wt` function L2 — `function Wt(t,e)`
-  `X` function L2 — `function X(t,e,r,n,i)`
-  `Xa` function L2 — `function Xa(t,e)`
-  `Xe` function L2 — `function Xe(t)`
-  `Xi` function L2 — `function Xi(t,e,r)`
-  `Xo` function L2 — `function Xo(t)`
-  `Xt` function L2 — `function Xt()`
-  `Y` function L2 — `function Y(t,e,r)`
-  `Ya` function L2 — `function Ya(t,e)`
-  `Ye` function L2 — `function Ye(t,e,r,n)`
-  `Yi` function L2 — `function Yi(t,e,r)`
-  `Yn` function L2 — `function Yn(t)`
-  `Yo` function L2 — `function Yo(t)`
-  `Yr` function L2 — `function Yr(t,e,r,n)`
-  `Yt` function L2 — `function Yt()`
-  `Z` function L2 — `function Z(t,e)`
-  `Za` function L2 — `function Za(t)`
-  `Ze` function L2 — `function Ze(t)`
-  `Zi` function L2 — `function Zi(t)`
-  `Zo` function L2 — `function Zo(t,e,r)`
-  `Zr` function L2 — `function Zr(t)`
-  `Zt` function L2 — `function Zt(t,e,r)`
-  `_` function L2 — `function _(t)`
-  `_` class L2 — `-`
-  `_` function L2 — `function _(t)`
-  `_a` function L2 — `function _a(t,e)`
-  `_n` function L2 — `function _n(t,e)`
-  `_s` function L2 — `function _s(t)`
-  `_t` function L2 — `function _t(t,e,r)`
-  `a` function L2 — `function a(t,e,r,a=1)`
-  `aa` function L2 — `function aa(t,e,r)`
-  `ae` function L2 — `function ae(t,e)`
-  `an` function L2 — `function an()`
-  `ao` function L2 — `function ao(t)`
-  `as` function L2 — `function as(t)`
-  `at` function L2 — `function at(t)`
-  `b` function L2 — `function b(t)`
-  `ba` function L2 — `function ba(t,e)`
-  `be` function L2 — `function be(t,e)`
-  `bn` function L2 — `function bn(t,e)`
-  `br` function L2 — `function br(t,e)`
-  `bt` function L2 — `function bt()`
-  `c` class L2 — `-`
-  `c` function L2 — `function c()`
-  `ca` function L2 — `function ca(t,e,r)`
-  `ce` function L2 — `function ce(t,e)`
-  `cn` function L2 — `function cn(t,e,r)`
-  `cr` function L2 — `function cr(t)`
-  `ct` function L2 — `function ct()`
-  `ct` class L2 — `-`
-  `d` class L2 — `-`
-  `d` function L2 — `function d(t)`
-  `da` function L2 — `function da(t,e,r)`
-  `de` function L2 — `function de(t,e,r,n)`
-  `dn` function L2 — `function dn(t,e)`
-  `dr` function L2 — `function dr(t,e,r)`
-  `dt` function L2 — `function dt(t)`
-  `e` function L2 — `function e(e,r)`
-  `ea` function L2 — `function ea(t,e,r)`
-  `ee` function L2 — `function ee(t,e)`
-  `en` function L2 — `function en(t)`
-  `eo` function L2 — `function eo(t,e)`
-  `er` function L2 — `function er(t,e)`
-  `es` function L2 — `function es(t)`
-  `et` class L2 — `-`
-  `et` function L2 — `function et(t)`
-  `f` function L2 — `function f(t,e)`
-  `fa` function L2 — `function fa(t,e,r)`
-  `fe` function L2 — `function fe(t,e)`
-  `fn` function L2 — `function fn(t)`
-  `fr` function L2 — `function fr(t,e,r)`
-  `ft` function L2 — `function ft()`
-  `g` function L2 — `function g(t)`
-  `g` class L2 — `-`
-  `g` function L2 — `function g()`
-  `ga` function L2 — `function ga(t,e,r)`
-  `ge` function L2 — `function ge(t,e)`
-  `gn` function L2 — `function gn()`
-  `gr` function L2 — `function gr(t,e)`
-  `gs` function L2 — `function gs(t,e,r)`
-  `gt` function L2 — `function gt(t)`
-  `h` class L2 — `-`
-  `h` function L2 — `const h = ()`
-  `ha` function L2 — `function ha(t,e,r)`
-  `he` function L2 — `function he(t,e)`
-  `hn` function L2 — `function hn(t,e,r)`
-  `ho` function L2 — `function ho()`
-  `hr` function L2 — `function hr(t)`
-  `hs` function L2 — `function hs(t,e)`
-  `ht` function L2 — `function ht(t)`
-  `ht` class L2 — `-`
-  `i` function L2 — `function i(t)`
-  `ia` function L2 — `function ia(t,e,r)`
-  `ie` function L2 — `function ie(t,e)`
-  `io` function L2 — `function io(t)`
-  `ir` function L2 — `function ir(t)`
-  `is` function L2 — `function is(t)`
-  `it` function L2 — `function it(t)`
-  `j` function L2 — `function j(t,e,r,n,i,a,o,s)`
-  `ja` function L2 — `function ja(t,e)`
-  `je` function L2 — `function je()`
-  `jo` function L2 — `function jo(t)`
-  `jr` function L2 — `function jr(t)`
-  `jt` function L2 — `function jt(t,e)`
-  `k` function L2 — `function k(t)`
-  `ka` function L2 — `function ka(t,e)`
-  `ke` function L2 — `function ke()`
-  `kn` function L2 — `function kn(t,e)`
-  `kr` function L2 — `function kr(t,e)`
-  `ks` function L2 — `function ks(t,e)`
-  `kt` function L2 — `function kt()`
-  `l` function L2 — `function l(t)`
-  `la` function L2 — `function la(t,e,r)`
-  `le` function L2 — `function le(t,e)`
-  `lo` function L2 — `function lo(t,e,r,n,i,a,o,s,l,h)`
-  `lr` function L2 — `function lr(t,e)`
-  `ls` function L2 — `function ls(t,e,r)`
-  `lt` function L2 — `function lt(t)`
-  `lt` class L2 — `-`
-  `m` function L2 — `function m(t,e)`
-  `ma` function L2 — `function ma(t,e)`
-  `me` function L2 — `function me(t,e,r)`
-  `mr` function L2 — `function mr(t,e)`
-  `mt` function L2 — `function mt(t)`
-  `n` function L2 — `-`
-  `na` function L2 — `function na(t,e,r)`
-  `ne` function L2 — `function ne(t,e,r,n,i,a)`
-  `ni` function L2 — `function ni(t,e,r,n)`
-  `nn` function L2 — `function nn()`
-  `no` function L2 — `function no()`
-  `ns` class L2 — `-`
-  `nt` class L2 — `-`
-  `nt` function L2 — `function nt(t,e)`
-  `o` function L2 — `function o(t,e)`
-  `oa` function L2 — `function oa(t,e,r)`
-  `oe` function L2 — `function oe(t,e)`
-  `oo` function L2 — `function oo(t)`
-  `os` function L2 — `function os(t,e)`
-  `ot` function L2 — `function ot(t,e)`
-  `ot` class L2 — `-`
-  `p` class L2 — `-`
-  `p` function L2 — `function p(t)`
-  `pa` function L2 — `function pa(t,e,r)`
-  `pe` function L2 — `function pe(t,e,r,n=[])`
-  `pi` function L2 — `function pi(t)`
-  `pn` function L2 — `function pn(t,e)`
-  `po` function L2 — `function po(t)`
-  `pr` function L2 — `function pr(t,e,r)`
-  `ps` function L2 — `function ps(t,e)`
-  `pt` function L2 — `function pt()`
-  `q` function L2 — `function q(t,e,r,n,i,a,o,s,l)`
-  `qa` function L2 — `function qa(t,e)`
-  `qe` function L2 — `function qe()`
-  `qo` function L2 — `function qo(t)`
-  `qr` function L2 — `function qr(t,e,r,n)`
-  `qt` function L2 — `function qt(t,e)`
-  `r` function L2 — `function r(t)`
-  `ra` function L2 — `function ra(t,e,r)`
-  `re` function L2 — `function re(t,e,r)`
-  `ro` function L2 — `function ro()`
-  `rr` function L2 — `function rr(t,e)`
-  `rs` function L2 — `function rs(t)`
-  `rt` function L2 — `function rt(t,e)`
-  `s` class L2 — `-`
-  `s` function L2 — `function s()`
-  `sa` function L2 — `function sa(t,e,r)`
-  `se` function L2 — `function se(t,e)`
-  `so` function L2 — `function so(t)`
-  `sr` function L2 — `function sr(t,e)`
-  `st` function L2 — `function st()`
-  `st` class L2 — `-`
-  `t` function L2 — `function t()`
-  `ta` function L2 — `function ta(t,e,r)`
-  `te` function L2 — `function te(t,e,r)`
-  `tn` function L2 — `function tn({_intern:t,_key:e},r)`
-  `to` function L2 — `function to(t,e)`
-  `tr` function L2 — `-`
-  `ts` function L2 — `function ts(t)`
-  `tt` class L2 — `-`
-  `tt` function L2 — `function tt(t)`
-  `u` class L2 — `-`
-  `u` function L2 — `function u(t)`
-  `ua` function L2 — `function ua(t,e,r)`
-  `ue` function L2 — `function ue(t,e,r,n,i,a)`
-  `un` function L2 — `function un(t,e,r)`
-  `uo` function L2 — `function uo(t)`
-  `ur` function L2 — `function ur(t,e,r)`
-  `us` function L2 — `function us(t,e)`
-  `ut` function L2 — `function ut(t)`
-  `v` function L2 — `function v(t,n)`
-  `va` function L2 — `function va(t,e)`
-  `ve` function L2 — `function ve(t,e)`
-  `vn` function L2 — `function vn(t,e)`
-  `vr` function L2 — `function vr(t,e,r,n)`
-  `vs` function L2 — `function vs(t)`
-  `vt` function L2 — `function vt(t,e,r)`
-  `w` function L2 — `function w(t,e,r,n,i,a,o,s,l,h)`
-  `wa` function L2 — `function wa(t,e)`
-  `wn` function L2 — `function wn(t,e)`
-  `wr` function L2 — `function wr(t)`
-  `ws` function L2 — `function ws(t)`
-  `wt` function L2 — `function wt(t)`
-  `x` function L2 — `function x(t)`
-  `xa` function L2 — `function xa(t,e)`
-  `xe` function L2 — `function xe(t,e,r)`
-  `xr` function L2 — `function xr(t,e)`
-  `xs` function L2 — `function xs(t,e)`
-  `xt` function L2 — `function xt()`
-  `y` function L2 — `const y = ()`
-  `ya` function L2 — `function ya(t,e,r)`
-  `ye` function L2 — `function ye(t,e,r)`
-  `yr` function L2 — `function yr(t,e)`
-  `ys` function L2 — `function ys(t,e)`
-  `yt` function L2 — `function yt(t)`
-  `z` function L2 — `function z(t,e,r,n,i,a,o,s)`
-  `za` function L2 — `function za(t,e)`
-  `ze` function L2 — `function ze(t,e,r,n)`
-  `zn` function L2 — `function zn(t)`
-  `zo` function L2 — `function zo(t)`
-  `zr` function L2 — `function zr(t)`
-  `zt` function L2 — `function zt(t,e)`

#### docs/themes/hugo-geekdoc/static/js/search-d0afef64.bundle.min.js

- pub `addSchema` method L2 — `addSchema(e,t)`
- pub `constructor` method L2 — `constructor(e,t="2019-09",r=!0)`
- pub `validate` method L2 — `validate(e)`
-  `$` function L2 — `function $(e,t=Object.create(null),r=w,n="")`
-  `D` function L2 — `function D(e,t)`
-  `E` function L2 — `function E(e,t)`
-  `P` function L2 — `function P(e)`
-  `R` class L2 — `-`
-  `S` function L2 — `function S(e,t,r="2019-09",n=$(t),o=!0,i=null,s="#",a="#",c=Object.create(null))`
-  `T` function L2 — `function T(e,r,n)`
-  `U` function L2 — `function U(e)`
-  `W` function L2 — `function W(e,t)`
-  `WorkerIndex` function L2 — `function WorkerIndex(e)`
-  `_` function L2 — `function _(e,t,r,n,i,s,a,c)`
-  `__webpack_require__` function L2 — `function __webpack_require__(e)`
-  `a` function L2 — `function a(e)`
-  `b` function L2 — `function b(e,t,r,n)`
-  `c` function L2 — `function c(e,t)`
-  `create` function L2 — `function create(factory,is_node_js,worker_path)`
-  `d` function L2 — `function d(e)`
-  `f` function L2 — `function f(e)`
-  `g` function L2 — `function g(e,t,r,n,o)`
-  `h` function L2 — `function h(e,t)`
-  `i` function L2 — `function i(e,t)`
-  `k` function L2 — `function k(e,t)`
-  `l` function L2 — `function l(e)`
-  `m` function L2 — `function m(e,t)`
-  `n` function L2 — `function n(e,t)`
-  `o` function L2 — `function o(e)`
-  `p` function L2 — `function p(e,t,r,n,i)`
-  `register` function L2 — `function register(e)`
-  `s` function L2 — `function s(e,t,r)`
-  `u` function L2 — `function u(e)`
-  `v` function L2 — `function v(e,t,r)`
-  `w` function L2 — `function w(e,t,r,o,i)`
-  `x` function L2 — `function x(e,t,r)`
-  `y` function L2 — `function y(e)`
-  `z` function L2 — `function z(e)`

### examples/ui-slim/src

> *Semantic summary to be generated by AI agent.*

#### examples/ui-slim/src/App.js

- pub `App` function L3172-3178 — `function App()`
-  `AgentsPanel` function L21-282 — `const AgentsPanel = ({ stacks, onRefresh })`
-  `selectAgent` function L58-67 — `const selectAgent = (agent)`
-  `addLabel` function L69-78 — `const addLabel = (label)`
-  `removeLabel` function L80-88 — `const removeLabel = (label)`
-  `addAnnotation` function L90-99 — `const addAnnotation = (key, value)`
-  `removeAnnotation` function L101-109 — `const removeAnnotation = (key)`
-  `addTarget` function L111-120 — `const addTarget = (stackId)`
-  `removeTarget` function L122-130 — `const removeTarget = (stackId)`
-  `toggleStatus` function L132-143 — `const toggleStatus = ()`
-  `StacksPanel` function L285-647 — `const StacksPanel = ({ generators, agents, onRefresh })`
-  `selectStack` function L322-335 — `const selectStack = (stack)`
-  `create` function L337-348 — `const create = (e)`
-  `deploy` function L350-363 — `const deploy = (e)`
-  `addLabel` function L365-374 — `const addLabel = (label)`
-  `removeLabel` function L376-384 — `const removeLabel = (label)`
-  `addAnnotation` function L386-395 — `const addAnnotation = (key, value)`
-  `removeAnnotation` function L397-405 — `const removeAnnotation = (key)`
-  `copyDeployment` function L407-416 — `const copyDeployment = (depId)`
-  `requestDiagnostic` function L418-443 — `const requestDiagnostic = (depId, agentId)`
-  `pollResult` function L424-438 — `const pollResult = ()`
-  `TemplatesPanel` function L650-888 — `const TemplatesPanel = ({ stacks })`
-  `create` function L689-700 — `const create = (e)`
-  `instantiate` function L702-713 — `const instantiate = (e)`
-  `remove` function L715-726 — `const remove = (id)`
-  `addLabel` function L728-737 — `const addLabel = (label)`
-  `removeLabel` function L739-747 — `const removeLabel = (label)`
-  `JobsPanel` function L891-1261 — `const JobsPanel = ({ agents })`
-  `create` function L927-945 — `const create = (e)`
-  `cancel` function L947-957 — `const cancel = (id)`
-  `runBuildDemo` function L960-1033 — `const runBuildDemo = ()`
-  `prefillBuildDemo` function L1036-1043 — `const prefillBuildDemo = ()`
-  `AdminPanel` function L1264-1411 — `const AdminPanel = ({ onGeneratorsChange, onAgentsChange })`
-  `create` function L1290-1307 — `const create = (e)`
-  `rotate` function L1309-1319 — `const rotate = (type, id)`
-  `copy` function L1321-1324 — `const copy = ()`
-  `closeCreate` function L1326-1332 — `const closeCreate = ()`
-  `WebhooksPanel` function L1414-1744 — `const WebhooksPanel = ()`
-  `selectWebhook` function L1447-1456 — `const selectWebhook = (webhook)`
-  `create` function L1458-1475 — `const create = (e)`
-  `toggleEnabled` function L1477-1488 — `const toggleEnabled = (webhook)`
-  `remove` function L1490-1501 — `const remove = (id)`
-  `toggleEventType` function L1503-1509 — `const toggleEventType = (type)`
-  `MetricsPanel` function L1747-1911 — `const MetricsPanel = ()`
-  `getMetricValue` function L1774-1780 — `const getMetricValue = (name, labels = {})`
-  `getMetricValues` function L1783 — `const getMetricValues = (name)`
-  `sumMetric` function L1786-1789 — `const sumMetric = (name)`
-  `DemoPanel` function L1914-3121 — `const DemoPanel = ()`
-  `startEventPolling` function L1942-1961 — `const startEventPolling = ()`
-  `poll` function L1945-1958 — `const poll = ()`
-  `stopEventPolling` function L1964-1970 — `const stopEventPolling = ()`
-  `clearWebhookEvents` function L1973-1980 — `const clearWebhookEvents = ()`
-  `getEventTypeClass` function L1992-1999 — `const getEventTypeClass = (eventType)`
-  `getEventStatusClass` function L2002-2014 — `const getEventStatusClass = (event)`
-  `formatEventPayload` function L2017-2025 — `const formatEventPayload = (event)`
-  `EventLogPanel` function L2028-2079 — `const EventLogPanel = ()`
-  `updatePhase` function L2082-2090 — `const updatePhase = (phaseNum, updates)`
-  `addStep` function L2093-2104 — `const addStep = (phaseNum, step)`
-  `formatDuration` function L2107-2113 — `const formatDuration = (ms)`
-  `resetDemo` function L2116-2146 — `const resetDemo = ()`
-  `canStartPhase` function L2151-2178 — `const canStartPhase = (phaseNum)`
-  `runPhase` function L2181-2212 — `const runPhase = (phaseNum)`
-  `runPhase1` function L2215-2289 — `const runPhase1 = ()`
-  `runPhase2` function L2292-2345 — `const runPhase2 = ()`
-  `runPhase3` function L2348-2429 — `const runPhase3 = ()`
-  `runPhase4` function L2432-2524 — `const runPhase4 = ()`
-  `runPhase5` function L2527-2619 — `const runPhase5 = ()`
-  `runPhase6` function L2622-2751 — `const runPhase6 = ()`
-  `runPhase7` function L2754-2831 — `const runPhase7 = ()`
-  `runPhase8` function L2834-2878 — `const runPhase8 = ()`
-  `runCleanup` function L2881-2969 — `const runCleanup = ()`
-  `PhaseCard` function L2975-3049 — `const PhaseCard = ({ num, phase })`
-  `AppContent` function L3125-3169 — `const AppContent = ()`

#### examples/ui-slim/src/api.js

- pub `getAgents` function L20 — `const getAgents = ()`
- pub `getAgentLabels` function L21 — `const getAgentLabels = (id)`
- pub `getAgentAnnotations` function L22 — `const getAgentAnnotations = (id)`
- pub `getAgentTargets` function L23 — `const getAgentTargets = (id)`
- pub `getAgentEvents` function L24 — `const getAgentEvents = (id)`
- pub `getAgentStacks` function L25 — `const getAgentStacks = (id)`
- pub `addAgentLabel` function L26 — `const addAgentLabel = (id, label)`
- pub `removeAgentLabel` function L27 — `const removeAgentLabel = (id, label)`
- pub `addAgentAnnotation` function L28 — `const addAgentAnnotation = (id, key, value)`
- pub `removeAgentAnnotation` function L29 — `const removeAgentAnnotation = (id, key)`
- pub `addAgentTarget` function L30 — `const addAgentTarget = (id, stackId)`
- pub `removeAgentTarget` function L31 — `const removeAgentTarget = (id, stackId)`
- pub `createAgent` function L32 — `const createAgent = (name, cluster)`
- pub `updateAgent` function L33 — `const updateAgent = (id, updates)`
- pub `rotateAgentPak` function L34 — `const rotateAgentPak = (id)`
- pub `getStacks` function L37 — `const getStacks = ()`
- pub `getStackLabels` function L38 — `const getStackLabels = (id)`
- pub `getStackAnnotations` function L39 — `const getStackAnnotations = (id)`
- pub `getStackDeployments` function L40 — `const getStackDeployments = (id)`
- pub `createStack` function L41 — `const createStack = (name, description, generatorId)`
- pub `addStackLabel` function L42 — `const addStackLabel = (id, label)`
- pub `removeStackLabel` function L43 — `const removeStackLabel = (id, label)`
- pub `addStackAnnotation` function L44 — `const addStackAnnotation = (id, key, value)`
- pub `removeStackAnnotation` function L45 — `const removeStackAnnotation = (id, key)`
- pub `createDeployment` function L46-49 — `const createDeployment = (stackId, yaml, isDeletion = false)`
- pub `getDeployment` function L50 — `const getDeployment = (id)`
- pub `getTemplates` function L53 — `const getTemplates = ()`
- pub `getTemplateLabels` function L54 — `const getTemplateLabels = (id)`
- pub `getTemplateAnnotations` function L55 — `const getTemplateAnnotations = (id)`
- pub `createTemplate` function L56 — `const createTemplate = (name, description, content, schema)`
- pub `updateTemplate` function L57 — `const updateTemplate = (id, description, content, schema)`
- pub `deleteTemplate` function L58 — `const deleteTemplate = (id)`
- pub `addTemplateLabel` function L59 — `const addTemplateLabel = (id, label)`
- pub `removeTemplateLabel` function L60 — `const removeTemplateLabel = (id, label)`
- pub `instantiateTemplate` function L61 — `const instantiateTemplate = (stackId, templateId, params)`
- pub `getGenerators` function L64 — `const getGenerators = ()`
- pub `createGenerator` function L65 — `const createGenerator = (name, description)`
- pub `rotateGeneratorPak` function L66 — `const rotateGeneratorPak = (id)`
- pub `getWorkOrders` function L69-75 — `const getWorkOrders = (status, workType)`
- pub `getWorkOrder` function L76 — `const getWorkOrder = (id)`
- pub `createWorkOrder` function L77-87 — `const createWorkOrder = (workType, yamlContent, targeting, options = {})`
- pub `deleteWorkOrder` function L88 — `const deleteWorkOrder = (id)`
- pub `getWorkOrderLog` function L89-97 — `const getWorkOrderLog = (workType, success, agentId, limit)`
- pub `createDiagnostic` function L100-104 — `const createDiagnostic = (deploymentObjectId, agentId, requestedBy, retentionMin...`
- pub `getDiagnostic` function L105 — `const getDiagnostic = (id)`
- pub `getDeploymentHealth` function L108 — `const getDeploymentHealth = (id)`
- pub `getStackHealth` function L109 — `const getStackHealth = (id)`
- pub `getWebhooks` function L112 — `const getWebhooks = ()`
- pub `getWebhook` function L113 — `const getWebhook = (id)`
- pub `createWebhook` function L114-125 — `const createWebhook = (name, url, eventTypes, authHeader, options = {})`
- pub `updateWebhook` function L126-129 — `const updateWebhook = (id, updates)`
- pub `deleteWebhook` function L130 — `const deleteWebhook = (id)`
- pub `getWebhookEventTypes` function L131 — `const getWebhookEventTypes = ()`
- pub `getWebhookDeliveries` function L132-138 — `const getWebhookDeliveries = (id, status, limit)`
- pub `getMetrics` function L141-145 — `const getMetrics = ()`
- pub `getWebhookCatcherStats` function L151-155 — `const getWebhookCatcherStats = ()`
- pub `clearWebhookCatcher` function L157-161 — `const clearWebhookCatcher = ()`
- pub `getDemoBuildYaml` function L165-182 — `const getDemoBuildYaml = ()`
- pub `createBuildWorkOrder` function L186-198 — `const createBuildWorkOrder = (imageTag = 'latest', agentId = null)`
- pub `getWebhookCatcherDeploymentYaml` function L201-249 — `const getWebhookCatcherDeploymentYaml = (imageTag = 'latest')`
- pub `parseMetrics` function L252-280 — `const parseMetrics = (metricsText)`
- pub `checkEnvironment` function L287-324 — `const checkEnvironment = ()`
- pub `getWebhookCatcherEvents` function L327-335 — `const getWebhookCatcherEvents = ()`
- pub `pollForCondition` function L338-350 — `const pollForCondition = (checkFn, intervalMs = 2000, timeoutMs = 60000)`
- pub `pollAgentStatus` function L353-371 — `const pollAgentStatus = (agentId, timeoutMs = 120000)`
- pub `pollWorkOrderStatus` function L374-387 — `const pollWorkOrderStatus = (workOrderId, timeoutMs = 300000)`
- pub `deleteStack` function L390 — `const deleteStack = (id)`
- pub `deleteAgent` function L393 — `const deleteAgent = (id)`
- pub `deleteGenerator` function L396 — `const deleteGenerator = (id)`
- pub `cleanupDemo` function L399-492 — `const cleanupDemo = (resources, onProgress)`
-  `sha256` function L4-8 — `const sha256 = (str)`
-  `request` function L10-17 — `const request = (path, options = {})`
-  `log` function L400 — `const log = (step, status)`

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

### tests/e2e/src

> *Semantic summary to be generated by AI agent.*

#### tests/e2e/src/api.rs

- pub `Result` type L17 — `= std::result::Result<T, Box<dyn std::error::Error>>` — HTTP API client for the Brokkr broker.
- pub `Client` struct L20-24 — `{ http: reqwest::Client, base_url: String, admin_pak: String }` — API client for the Brokkr broker
- pub `new` function L27-33 — `(base_url: &str, admin_pak: &str) -> Self` — HTTP API client for the Brokkr broker.
- pub `wait_for_ready` function L36-49 — `(&self, timeout_secs: u64) -> Result<()>` — Wait for the broker to be ready
- pub `list_agents` function L112-114 — `(&self) -> Result<Vec<Value>>` — HTTP API client for the Brokkr broker.
- pub `create_agent` function L116-121 — `(&self, name: &str, cluster: &str) -> Result<Value>` — HTTP API client for the Brokkr broker.
- pub `get_agent` function L123-125 — `(&self, id: Uuid) -> Result<Value>` — HTTP API client for the Brokkr broker.
- pub `update_agent` function L127-129 — `(&self, id: Uuid, updates: Value) -> Result<Value>` — HTTP API client for the Brokkr broker.
- pub `add_agent_label` function L131-136 — `(&self, id: Uuid, label: &str) -> Result<Value>` — HTTP API client for the Brokkr broker.
- pub `get_agent_labels` function L138-140 — `(&self, id: Uuid) -> Result<Vec<Value>>` — HTTP API client for the Brokkr broker.
- pub `add_agent_annotation` function L142-148 — `(&self, id: Uuid, key: &str, value: &str) -> Result<Value>` — HTTP API client for the Brokkr broker.
- pub `get_agent_annotations` function L150-152 — `(&self, id: Uuid) -> Result<Vec<Value>>` — HTTP API client for the Brokkr broker.
- pub `add_agent_target` function L154-159 — `(&self, agent_id: Uuid, stack_id: Uuid) -> Result<Value>` — HTTP API client for the Brokkr broker.
- pub `get_agent_targets` function L161-163 — `(&self, id: Uuid) -> Result<Vec<Value>>` — HTTP API client for the Brokkr broker.
- pub `get_agent_stacks` function L165-167 — `(&self, id: Uuid) -> Result<Vec<Value>>` — HTTP API client for the Brokkr broker.
- pub `get_agent_target_state` function L169-175 — `(&self, id: Uuid, mode: Option<&str>) -> Result<Vec<Value>>` — HTTP API client for the Brokkr broker.
- pub `create_generator` function L181-186 — `(&self, name: &str, description: Option<&str>) -> Result<Value>` — HTTP API client for the Brokkr broker.
- pub `list_generators` function L188-190 — `(&self) -> Result<Vec<Value>>` — HTTP API client for the Brokkr broker.
- pub `create_stack` function L196-202 — `(&self, name: &str, description: Option<&str>, generator_id: Uuid) -> Result<Val...` — HTTP API client for the Brokkr broker.
- pub `list_stacks` function L204-206 — `(&self) -> Result<Vec<Value>>` — HTTP API client for the Brokkr broker.
- pub `get_stack` function L208-210 — `(&self, id: Uuid) -> Result<Value>` — HTTP API client for the Brokkr broker.
- pub `add_stack_label` function L212-215 — `(&self, id: Uuid, label: &str) -> Result<Value>` — HTTP API client for the Brokkr broker.
- pub `get_stack_labels` function L217-219 — `(&self, id: Uuid) -> Result<Vec<Value>>` — HTTP API client for the Brokkr broker.
- pub `add_stack_annotation` function L221-227 — `(&self, id: Uuid, key: &str, value: &str) -> Result<Value>` — HTTP API client for the Brokkr broker.
- pub `create_deployment` function L233-241 — `(&self, stack_id: Uuid, yaml: &str, is_deletion: bool) -> Result<Value>` — HTTP API client for the Brokkr broker.
- pub `list_deployments` function L243-245 — `(&self, stack_id: Uuid) -> Result<Vec<Value>>` — HTTP API client for the Brokkr broker.
- pub `get_deployment` function L247-249 — `(&self, id: Uuid) -> Result<Value>` — HTTP API client for the Brokkr broker.
- pub `get_deployment_health` function L251-253 — `(&self, id: Uuid) -> Result<Value>` — HTTP API client for the Brokkr broker.
- pub `get_stack_health` function L255-257 — `(&self, id: Uuid) -> Result<Value>` — HTTP API client for the Brokkr broker.
- pub `create_template` function L263-276 — `( &self, name: &str, description: Option<&str>, content: &str, schema: &str, ) -...` — HTTP API client for the Brokkr broker.
- pub `list_templates` function L278-280 — `(&self) -> Result<Vec<Value>>` — HTTP API client for the Brokkr broker.
- pub `instantiate_template` function L282-295 — `( &self, stack_id: Uuid, template_id: Uuid, parameters: Value, ) -> Result<Value...` — HTTP API client for the Brokkr broker.
- pub `delete_template` function L297-299 — `(&self, id: Uuid) -> Result<()>` — HTTP API client for the Brokkr broker.
- pub `create_work_order` function L305-328 — `( &self, work_type: &str, yaml: &str, target_agent_ids: Option<Vec<Uuid>>, targe...` — HTTP API client for the Brokkr broker.
- pub `list_work_orders` function L330-332 — `(&self) -> Result<Vec<Value>>` — HTTP API client for the Brokkr broker.
- pub `get_work_order` function L334-336 — `(&self, id: Uuid) -> Result<Value>` — HTTP API client for the Brokkr broker.
- pub `get_work_order_log` function L338-340 — `(&self, id: Uuid) -> Result<Value>` — HTTP API client for the Brokkr broker.
- pub `delete_work_order` function L342-344 — `(&self, id: Uuid) -> Result<()>` — HTTP API client for the Brokkr broker.
- pub `create_diagnostic` function L350-363 — `( &self, deployment_id: Uuid, agent_id: Uuid, ) -> Result<Value>` — HTTP API client for the Brokkr broker.
- pub `get_diagnostic` function L365-367 — `(&self, id: Uuid) -> Result<Value>` — HTTP API client for the Brokkr broker.
- pub `create_webhook` function L373-381 — `( &self, name: &str, url: &str, event_types: Vec<&str>, auth_header: Option<&str...` — HTTP API client for the Brokkr broker.
- pub `create_webhook_with_options` function L383-407 — `( &self, name: &str, url: &str, event_types: Vec<&str>, auth_header: Option<&str...` — HTTP API client for the Brokkr broker.
- pub `list_webhooks` function L409-411 — `(&self) -> Result<Vec<Value>>` — HTTP API client for the Brokkr broker.
- pub `get_webhook` function L413-415 — `(&self, id: Uuid) -> Result<Value>` — HTTP API client for the Brokkr broker.
- pub `update_webhook` function L417-419 — `(&self, id: Uuid, updates: Value) -> Result<Value>` — HTTP API client for the Brokkr broker.
- pub `delete_webhook` function L421-423 — `(&self, id: Uuid) -> Result<()>` — HTTP API client for the Brokkr broker.
- pub `list_webhook_deliveries` function L425-427 — `(&self, webhook_id: Uuid) -> Result<Vec<Value>>` — HTTP API client for the Brokkr broker.
- pub `test_webhook` function L429-431 — `(&self, id: Uuid) -> Result<Value>` — HTTP API client for the Brokkr broker.
- pub `list_audit_logs` function L437-443 — `(&self, limit: Option<i32>) -> Result<Value>` — HTTP API client for the Brokkr broker.
- pub `get_metrics` function L450-461 — `(&self) -> Result<String>` — Fetch Prometheus metrics from the broker
- pub `get_healthz` function L464-475 — `(&self) -> Result<String>` — Fetch health check endpoint
- pub `WebhookCatcher` struct L479-482 — `{ http: reqwest::Client, base_url: String }` — Client for webhook-catcher test service
- pub `new` function L485-490 — `(base_url: &str) -> Self` — HTTP API client for the Brokkr broker.
- pub `get_messages` function L493-504 — `(&self) -> Result<Value>` — Get all messages received by webhook-catcher
- pub `clear_messages` function L507-517 — `(&self) -> Result<()>` — Clear all messages from webhook-catcher
- pub `wait_for_messages` function L520-539 — `(&self, count: usize, timeout_secs: u64) -> Result<Value>` — Wait for at least N messages to arrive, with timeout
-  `Client` type L26-476 — `= Client` — HTTP API client for the Brokkr broker.
-  `request` function L51-80 — `( &self, method: reqwest::Method, path: &str, body: Option<Value>, ) -> Result<T...` — HTTP API client for the Brokkr broker.
-  `get` function L82-84 — `(&self, path: &str) -> Result<T>` — HTTP API client for the Brokkr broker.
-  `post` function L86-88 — `(&self, path: &str, body: Value) -> Result<T>` — HTTP API client for the Brokkr broker.
-  `put` function L90-92 — `(&self, path: &str, body: Value) -> Result<T>` — HTTP API client for the Brokkr broker.
-  `delete` function L94-106 — `(&self, path: &str) -> Result<()>` — HTTP API client for the Brokkr broker.
-  `WebhookCatcher` type L484-540 — `= WebhookCatcher` — HTTP API client for the Brokkr broker.
-  `sha256_hex` function L542-546 — `(data: &str) -> String` — HTTP API client for the Brokkr broker.

#### tests/e2e/src/main.rs

-  `api` module L18 — `-` — Brokkr End-to-End Test Suite
-  `scenarios` module L19 — `-` — Run with: angreal tests e2e
-  `main` function L25-95 — `() -> ExitCode` — Run with: angreal tests e2e
-  `run_scenario` macro L54-71 — `-` — Run with: angreal tests e2e

#### tests/e2e/src/scenarios.rs

- pub `test_agent_management` function L133-174 — `(client: &Client) -> Result<()>` — Each scenario tests a complete user workflow through the system.
- pub `test_stack_deployment` function L180-214 — `(client: &Client) -> Result<()>` — Each scenario tests a complete user workflow through the system.
- pub `test_targeting` function L220-264 — `(client: &Client) -> Result<()>` — Each scenario tests a complete user workflow through the system.
- pub `test_templates` function L270-319 — `(client: &Client) -> Result<()>` — Each scenario tests a complete user workflow through the system.
- pub `test_work_orders` function L325-374 — `(client: &Client) -> Result<()>` — Each scenario tests a complete user workflow through the system.
- pub `test_build_work_orders` function L387-536 — `(client: &Client) -> Result<()>` — Test build work orders using Shipwright.
- pub `test_health_diagnostics` function L542-572 — `(client: &Client) -> Result<()>` — Each scenario tests a complete user workflow through the system.
- pub `test_webhooks` function L578-739 — `(client: &Client, webhook_catcher_url: Option<&str>) -> Result<()>` — Each scenario tests a complete user workflow through the system.
- pub `test_agent_reconciliation_existing_deployments` function L750-850 — `(client: &Client) -> Result<()>` — Test that agents can reconcile pre-existing deployments when targeted to a stack.
- pub `test_audit_logs` function L856-891 — `(client: &Client) -> Result<()>` — Each scenario tests a complete user workflow through the system.
- pub `test_metrics` function L897-973 — `(client: &Client) -> Result<()>` — Each scenario tests a complete user workflow through the system.
-  `DEMO_DEPLOYMENT_YAML` variable L16-53 — `: &str` — Sample deployment YAML for testing
-  `MICROSERVICE_TEMPLATE` variable L56-76 — `: &str` — Microservice template for testing
-  `MICROSERVICE_SCHEMA` variable L78-88 — `: &str` — Each scenario tests a complete user workflow through the system.
-  `JOB_YAML` variable L91-105 — `: &str` — Job YAML for work order testing
-  `BUILD_YAML` variable L110-127 — `: &str` — Shipwright Build YAML for build work order testing

