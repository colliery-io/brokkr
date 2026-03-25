# Code Index

> Generated: 2026-03-25T21:09:38Z | 157 files | JavaScript, Python, Rust

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
-  `delete_agent` function L469-516 — `( State(dal): State<DAL>, Extension(auth_payload): Extension<AuthPayload>, Path(...` — event logging, label management, annotation management, target management, and heartbeat recording.
-  `list_events` function L539-578 — `( State(dal): State<DAL>, Extension(auth_payload): Extension<AuthPayload>, Path(...` — event logging, label management, annotation management, target management, and heartbeat recording.
-  `create_event` function L602-652 — `( State(dal): State<DAL>, Extension(auth_payload): Extension<AuthPayload>, Path(...` — event logging, label management, annotation management, target management, and heartbeat recording.
-  `list_labels` function L675-709 — `( State(dal): State<DAL>, Extension(auth_payload): Extension<AuthPayload>, Path(...` — event logging, label management, annotation management, target management, and heartbeat recording.
-  `add_label` function L732-763 — `( State(dal): State<DAL>, Extension(auth_payload): Extension<AuthPayload>, Path(...` — event logging, label management, annotation management, target management, and heartbeat recording.
-  `remove_label` function L787-834 — `( State(dal): State<DAL>, Extension(auth_payload): Extension<AuthPayload>, Path(...` — event logging, label management, annotation management, target management, and heartbeat recording.
-  `list_annotations` function L857-897 — `( State(dal): State<DAL>, Extension(auth_payload): Extension<AuthPayload>, Path(...` — event logging, label management, annotation management, target management, and heartbeat recording.
-  `add_annotation` function L920-954 — `( State(dal): State<DAL>, Extension(auth_payload): Extension<AuthPayload>, Path(...` — event logging, label management, annotation management, target management, and heartbeat recording.
-  `remove_annotation` function L978-1025 — `( State(dal): State<DAL>, Extension(auth_payload): Extension<AuthPayload>, Path(...` — event logging, label management, annotation management, target management, and heartbeat recording.
-  `list_targets` function L1048-1082 — `( State(dal): State<DAL>, Extension(auth_payload): Extension<AuthPayload>, Path(...` — event logging, label management, annotation management, target management, and heartbeat recording.
-  `add_target` function L1106-1137 — `( State(dal): State<DAL>, Extension(auth_payload): Extension<AuthPayload>, Path(...` — event logging, label management, annotation management, target management, and heartbeat recording.
-  `remove_target` function L1162-1212 — `( State(dal): State<DAL>, Extension(auth_payload): Extension<AuthPayload>, Path(...` — event logging, label management, annotation management, target management, and heartbeat recording.
-  `record_heartbeat` function L1234-1277 — `( State(dal): State<DAL>, Extension(auth_payload): Extension<AuthPayload>, Path(...` — event logging, label management, annotation management, target management, and heartbeat recording.
-  `TargetStateParams` struct L1281-1284 — `{ mode: Option<String> }` — Defines query parameters for the target state endpoint
-  `get_target_state` function L1313-1365 — `( State(dal): State<DAL>, Extension(auth_payload): Extension<AuthPayload>, Path(...` — event logging, label management, annotation management, target management, and heartbeat recording.
-  `get_associated_stacks` function L1388-1428 — `( State(dal): State<DAL>, Extension(auth_payload): Extension<AuthPayload>, Path(...` — event logging, label management, annotation management, target management, and heartbeat recording.
-  `rotate_agent_pak` function L1452-1535 — `( State(dal): State<DAL>, Extension(auth_payload): Extension<AuthPayload>, Path(...` — event logging, label management, annotation management, target management, and heartbeat recording.

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
-  `delete_generator` function L336-370 — `( State(dal): State<DAL>, Extension(auth_payload): Extension<AuthPayload>, Path(...` — Deletes a generator.
-  `rotate_generator_pak` function L401-474 — `( State(dal): State<DAL>, Extension(auth_payload): Extension<AuthPayload>, Path(...` — Rotates the PAK for a specific generator.

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
-  `verify_pak` function L108-197 — `(dal: &DAL, pak: &str) -> Result<AuthPayload, StatusCode>` — Verifies the provided PAK and returns the corresponding `AuthPayload`.

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
- pub `serve` function L38-183 — `(config: &Settings) -> Result<(), Box<dyn std::error::Error>>` — Function to start the Brokkr Broker server
- pub `rotate_admin` function L188-200 — `(config: &Settings) -> Result<(), Box<dyn std::error::Error>>` — Function to rotate the admin key
- pub `rotate_agent_key` function L202-219 — `(config: &Settings, uuid: Uuid) -> Result<(), Box<dyn std::error::Error>>`
- pub `rotate_generator_key` function L221-246 — `( config: &Settings, uuid: Uuid, ) -> Result<(), Box<dyn std::error::Error>>`
- pub `create_agent` function L248-281 — `( config: &Settings, name: String, cluster_name: String, ) -> Result<(), Box<dyn...`
- pub `create_generator` function L283-313 — `( config: &Settings, name: String, description: Option<String>, ) -> Result<(), ...`
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

- pub `DalError` enum L43-50 — `ConnectionPool | Query | NotFound` — Error types for DAL operations.
- pub `agents` module L96 — `-` — ```
- pub `agent_annotations` module L99 — `-` — ```
- pub `audit_logs` module L102 — `-` — ```
- pub `agent_events` module L105 — `-` — ```
- pub `agent_labels` module L108 — `-` — ```
- pub `agent_targets` module L111 — `-` — ```
- pub `stacks` module L114 — `-` — ```
- pub `stack_annotations` module L117 — `-` — ```
- pub `stack_labels` module L120 — `-` — ```
- pub `deployment_health` module L123 — `-` — ```
- pub `deployment_objects` module L126 — `-` — ```
- pub `diagnostic_requests` module L129 — `-` — ```
- pub `diagnostic_results` module L132 — `-` — ```
- pub `generators` module L135 — `-` — ```
- pub `templates` module L138 — `-` — ```
- pub `template_labels` module L141 — `-` — ```
- pub `template_annotations` module L144 — `-` — ```
- pub `template_targets` module L147 — `-` — ```
- pub `rendered_deployment_objects` module L150 — `-` — ```
- pub `webhook_deliveries` module L153 — `-` — ```
- pub `webhook_subscriptions` module L156 — `-` — ```
- pub `work_orders` module L159 — `-` — ```
- pub `DAL` struct L168-173 — `{ pool: ConnectionPool, auth_cache: Option<Cache<String, AuthPayload>> }` — The main Data Access Layer struct.
- pub `new` function L185-187 — `(pool: ConnectionPool) -> Self` — Creates a new DAL instance with the given connection pool.
- pub `new_with_auth_cache` function L195-207 — `(pool: ConnectionPool, auth_cache_ttl_seconds: u64) -> Self` — Creates a new DAL instance with an auth cache.
- pub `invalidate_auth_cache` function L210-214 — `(&self, pak_hash: &str)` — Invalidates a specific entry in the auth cache by PAK hash.
- pub `invalidate_all_auth_cache` function L217-221 — `(&self)` — Invalidates all entries in the auth cache.
- pub `agents` function L228-230 — `(&self) -> AgentsDAL` — Provides access to the Agents Data Access Layer.
- pub `agent_annotations` function L237-239 — `(&self) -> AgentAnnotationsDAL` — Provides access to the Agent Annotations Data Access Layer.
- pub `agent_events` function L246-248 — `(&self) -> AgentEventsDAL` — Provides access to the Agent Events Data Access Layer.
- pub `agent_labels` function L255-257 — `(&self) -> AgentLabelsDAL` — Provides access to the Agent Labels Data Access Layer.
- pub `agent_targets` function L264-266 — `(&self) -> AgentTargetsDAL` — Provides access to the Agent Targets Data Access Layer.
- pub `stack_labels` function L273-275 — `(&self) -> StackLabelsDAL` — Provides access to the Stack Labels Data Access Layer.
- pub `stack_annotations` function L282-284 — `(&self) -> StackAnnotationsDAL` — Provides access to the Stack Annotations Data Access Layer.
- pub `stacks` function L291-293 — `(&self) -> StacksDAL` — Provides access to the Stacks Data Access Layer.
- pub `deployment_health` function L300-302 — `(&self) -> DeploymentHealthDAL` — Provides access to the Deployment Health Data Access Layer.
- pub `deployment_objects` function L309-311 — `(&self) -> DeploymentObjectsDAL` — Provides access to the Deployment Objects Data Access Layer.
- pub `generators` function L318-320 — `(&self) -> GeneratorsDAL` — Provides access to the Generators Data Access Layer.
- pub `templates` function L327-329 — `(&self) -> TemplatesDAL` — Provides access to the Templates Data Access Layer.
- pub `template_labels` function L336-338 — `(&self) -> TemplateLabelsDAL` — Provides access to the Template Labels Data Access Layer.
- pub `template_annotations` function L345-347 — `(&self) -> TemplateAnnotationsDAL` — Provides access to the Template Annotations Data Access Layer.
- pub `template_targets` function L354-356 — `(&self) -> TemplateTargetsDAL` — Provides access to the Template Targets Data Access Layer.
- pub `rendered_deployment_objects` function L363-365 — `(&self) -> RenderedDeploymentObjectsDAL` — Provides access to the Rendered Deployment Objects Data Access Layer.
- pub `work_orders` function L372-374 — `(&self) -> WorkOrdersDAL` — Provides access to the Work Orders Data Access Layer.
- pub `diagnostic_requests` function L381-383 — `(&self) -> DiagnosticRequestsDAL` — Provides access to the Diagnostic Requests Data Access Layer.
- pub `diagnostic_results` function L390-392 — `(&self) -> DiagnosticResultsDAL` — Provides access to the Diagnostic Results Data Access Layer.
- pub `webhook_subscriptions` function L399-401 — `(&self) -> WebhookSubscriptionsDAL` — Provides access to the Webhook Subscriptions Data Access Layer.
- pub `webhook_deliveries` function L408-410 — `(&self) -> WebhookDeliveriesDAL` — Provides access to the Webhook Deliveries Data Access Layer.
- pub `audit_logs` function L417-419 — `(&self) -> AuditLogsDAL` — Provides access to the Audit Logs Data Access Layer.
- pub `FilterType` enum L423-426 — `And | Or` — ```
-  `DalError` type L52-60 — `= DalError` — ```
-  `fmt` function L53-59 — `(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result` — ```
-  `DalError` type L62 — `= DalError` — ```
-  `DalError` type L64-68 — `= DalError` — ```
-  `from` function L65-67 — `(e: r2d2::Error) -> Self` — ```
-  `DalError` type L70-77 — `= DalError` — ```
-  `from` function L71-76 — `(e: diesel::result::Error) -> Self` — ```
-  `DalError` type L79-94 — `impl IntoResponse for DalError` — ```
-  `into_response` function L80-93 — `(self) -> Response` — ```
-  `DAL` type L175-420 — `= DAL` — ```

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

- pub `Settings` struct L121-136 — `{ database: Database, log: Log, pak: PAK, agent: Agent, broker: Broker, cors: Co...` — Represents the main settings structure for the application
- pub `Cors` struct L140-156 — `{ allowed_origins: Vec<String>, allowed_methods: Vec<String>, allowed_headers: V...` — Represents the CORS configuration
- pub `Broker` struct L159-179 — `{ pak_hash: Option<String>, diagnostic_cleanup_interval_seconds: Option<u64>, di...` — Default: 60 (set to 0 to disable caching)
- pub `Agent` struct L184-209 — `{ broker_url: String, polling_interval: u64, kubeconfig_path: Option<String>, ma...` — Represents the agent configuration
- pub `Database` struct L214-219 — `{ url: String, schema: Option<String> }` — Represents the database configuration
- pub `Log` struct L223-229 — `{ level: String, format: String }` — Represents the logging configuration
- pub `Telemetry` struct L237-256 — `{ enabled: bool, otlp_endpoint: String, service_name: String, sampling_rate: f64...` — Represents the telemetry (OpenTelemetry) configuration with hierarchical overrides
- pub `TelemetryOverride` struct L260-269 — `{ enabled: Option<bool>, otlp_endpoint: Option<String>, service_name: Option<Str...` — Component-specific telemetry overrides (all fields optional)
- pub `ResolvedTelemetry` struct L273-278 — `{ enabled: bool, otlp_endpoint: String, service_name: String, sampling_rate: f64...` — Resolved telemetry configuration after merging base with overrides
- pub `for_broker` function L282-297 — `(&self) -> ResolvedTelemetry` — Get resolved telemetry config for broker (base merged with broker overrides)
- pub `for_agent` function L300-315 — `(&self) -> ResolvedTelemetry` — Get resolved telemetry config for agent (base merged with agent overrides)
- pub `PAK` struct L332-349 — `{ prefix: Option<String>, digest: Option<String>, rng: Option<String>, short_tok...` — Represents the PAK configuration
- pub `short_length_as_str` function L353-355 — `(&mut self)` — Convert short token length to string
- pub `long_length_as_str` function L358-360 — `(&mut self)` — Convert long token length to string
- pub `new` function L373-392 — `(file: Option<String>) -> Result<Self, ConfigError>` — Creates a new `Settings` instance
- pub `DynamicConfig` struct L400-417 — `{ log_level: String, diagnostic_cleanup_interval_seconds: u64, diagnostic_max_ag...` — Dynamic configuration values that can be hot-reloaded at runtime.
- pub `from_settings` function L421-441 — `(settings: &Settings) -> Self` — Create DynamicConfig from Settings
- pub `ConfigChange` struct L446-453 — `{ key: String, old_value: String, new_value: String }` — Represents a configuration change detected during reload
- pub `ReloadableConfig` struct L479-486 — `{ static_config: Settings, dynamic: Arc<RwLock<DynamicConfig>>, config_file: Opt...` — Configuration wrapper that separates static (restart-required) settings
- pub `new` function L498-507 — `(file: Option<String>) -> Result<Self, ConfigError>` — Creates a new ReloadableConfig instance
- pub `from_settings` function L519-527 — `(settings: Settings, config_file: Option<String>) -> Self` — Creates a ReloadableConfig from an existing Settings instance
- pub `static_config` function L532-534 — `(&self) -> &Settings` — Get a reference to the static (immutable) settings
- pub `reload` function L540-619 — `(&self) -> Result<Vec<ConfigChange>, ConfigError>` — Reload dynamic configuration from sources (file + environment)
- pub `log_level` function L626-631 — `(&self) -> String` — Get current log level
- pub `diagnostic_cleanup_interval_seconds` function L634-639 — `(&self) -> u64` — Get diagnostic cleanup interval in seconds
- pub `diagnostic_max_age_hours` function L642-647 — `(&self) -> i64` — Get diagnostic max age in hours
- pub `webhook_delivery_interval_seconds` function L650-655 — `(&self) -> u64` — Get webhook delivery interval in seconds
- pub `webhook_delivery_batch_size` function L658-663 — `(&self) -> i64` — Get webhook delivery batch size
- pub `webhook_cleanup_retention_days` function L666-671 — `(&self) -> i64` — Get webhook cleanup retention in days
- pub `cors_allowed_origins` function L674-679 — `(&self) -> Vec<String>` — Get CORS allowed origins
- pub `cors_max_age_seconds` function L682-684 — `(&self) -> u64` — Get CORS max age in seconds
- pub `dynamic_snapshot` function L687-689 — `(&self) -> Option<DynamicConfig>` — Get a snapshot of all dynamic config values
-  `deserialize_string_or_vec` function L76-113 — `(deserializer: D) -> Result<Vec<String>, D::Error>` — Deserializes a comma-separated string or array into Vec<String>
-  `StringOrVec` struct L83 — `-` — Default: 60 (set to 0 to disable caching)
-  `StringOrVec` type L85-110 — `= StringOrVec` — Default: 60 (set to 0 to disable caching)
-  `Value` type L86 — `= Vec<String>` — Default: 60 (set to 0 to disable caching)
-  `expecting` function L88-90 — `(&self, formatter: &mut fmt::Formatter) -> fmt::Result` — Default: 60 (set to 0 to disable caching)
-  `visit_str` function L92-98 — `(self, value: &str) -> Result<Self::Value, E>` — Default: 60 (set to 0 to disable caching)
-  `visit_seq` function L100-109 — `(self, mut seq: A) -> Result<Self::Value, A::Error>` — Default: 60 (set to 0 to disable caching)
-  `DEFAULT_SETTINGS` variable L116 — `: &str` — Default: 60 (set to 0 to disable caching)
-  `default_log_format` function L231-233 — `() -> String` — Default: 60 (set to 0 to disable caching)
-  `Telemetry` type L280-316 — `= Telemetry` — Default: 60 (set to 0 to disable caching)
-  `default_otlp_endpoint` function L318-320 — `() -> String` — Default: 60 (set to 0 to disable caching)
-  `default_service_name` function L322-324 — `() -> String` — Default: 60 (set to 0 to disable caching)
-  `default_sampling_rate` function L326-328 — `() -> f64` — Default: 60 (set to 0 to disable caching)
-  `PAK` type L351-361 — `= PAK` — Default: 60 (set to 0 to disable caching)
-  `Settings` type L363-393 — `= Settings` — Default: 60 (set to 0 to disable caching)
-  `DynamicConfig` type L419-442 — `= DynamicConfig` — Default: 60 (set to 0 to disable caching)
-  `ReloadableConfig` type L488-690 — `= ReloadableConfig` — Default: 60 (set to 0 to disable caching)
-  `tests` module L693-1035 — `-` — Default: 60 (set to 0 to disable caching)
-  `test_settings_default_values` function L703-712 — `()` — Test the creation of Settings with default values
-  `test_telemetry_default_values` function L715-723 — `()` — Default: 60 (set to 0 to disable caching)
-  `test_telemetry_for_broker_no_overrides` function L726-743 — `()` — Default: 60 (set to 0 to disable caching)
-  `test_telemetry_for_broker_full_overrides` function L746-768 — `()` — Default: 60 (set to 0 to disable caching)
-  `test_telemetry_for_broker_partial_overrides` function L771-793 — `()` — Default: 60 (set to 0 to disable caching)
-  `test_telemetry_for_agent_no_overrides` function L796-813 — `()` — Default: 60 (set to 0 to disable caching)
-  `test_telemetry_for_agent_full_overrides` function L816-838 — `()` — Default: 60 (set to 0 to disable caching)
-  `test_telemetry_broker_and_agent_independent` function L841-875 — `()` — Default: 60 (set to 0 to disable caching)
-  `test_telemetry_override_enabled_false_overrides_base_true` function L878-899 — `()` — Default: 60 (set to 0 to disable caching)
-  `test_telemetry_sampling_rate_extremes` function L902-924 — `()` — Default: 60 (set to 0 to disable caching)
-  `test_reloadable_config_creation` function L931-944 — `()` — Default: 60 (set to 0 to disable caching)
-  `test_dynamic_config_from_settings` function L947-958 — `()` — Default: 60 (set to 0 to disable caching)
-  `test_reloadable_config_accessors_with_defaults` function L961-971 — `()` — Default: 60 (set to 0 to disable caching)
-  `test_reloadable_config_dynamic_snapshot` function L974-986 — `()` — Default: 60 (set to 0 to disable caching)
-  `test_reloadable_config_reload_no_changes` function L989-999 — `()` — Default: 60 (set to 0 to disable caching)
-  `test_reloadable_config_is_clone` function L1002-1008 — `()` — Default: 60 (set to 0 to disable caching)
-  `test_reloadable_config_thread_safety` function L1011-1034 — `()` — Default: 60 (set to 0 to disable caching)

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

