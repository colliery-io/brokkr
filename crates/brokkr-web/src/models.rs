//! Serde mirrors of the broker's JSON responses the console reads. Kept local
//! (not the broker's diesel-bound types) so the wasm crate stays light.

use serde::Deserialize;

/// One agent in `GET /api/v1/fleet` (mirrors the broker `FleetAgentRecord`).
#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct FleetAgentRecord {
    pub agent_id: String,
    pub name: String,
    /// Kubernetes cluster the agent runs in — used to group the fleet.
    #[serde(default)]
    pub cluster_name: String,
    pub status: String,
    pub ws_connected: bool,
    #[serde(default)]
    pub last_heartbeat: Option<String>,
    #[serde(default)]
    pub heartbeat_age_seconds: Option<i64>,
    #[serde(default)]
    pub pending_object_count: i64,
    #[serde(default)]
    pub pending_work_orders: i64,
    #[serde(default)]
    pub claimed_work_orders: i64,
    #[serde(default)]
    pub health_failing: i64,
    #[serde(default)]
    pub health_degraded: i64,
    #[serde(default)]
    pub k8s_reachable: Option<bool>,
    #[serde(default)]
    pub k8s_api_latency_ms: Option<i64>,
}

impl FleetAgentRecord {
    /// Derived health bucket from the failing/degraded counts.
    pub fn health(&self) -> (&'static str, &'static str) {
        use aurora_leptos::tokens::token;
        if self.health_failing > 0 {
            ("failing", token::BAD)
        } else if self.health_degraded > 0 {
            ("degraded", token::GOLD)
        } else {
            ("healthy", token::OK)
        }
    }
}

/// One deployment object in `GET /api/v1/agents/:id/target-state?mode=full`
/// (mirrors the broker `DeploymentObject`; only the fields the console needs —
/// unknown fields are ignored). Diagnostics are deployment-object-scoped
/// (`deployment_object_id` is NOT NULL), so this is what the Fleet modal's
/// diagnostic picker offers.
#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct TargetStateObject {
    pub id: String,
    pub stack_id: String,
    /// Global, monotonically increasing ordering key — unique per object, so it
    /// makes the picker labels unambiguous.
    #[serde(default)]
    pub sequence_id: i64,
    #[serde(default)]
    pub is_deletion_marker: bool,
}

impl TargetStateObject {
    /// Operator-readable, collision-free picker label (`sequence_id` is unique).
    /// Doubles as the picker's value — `aurora_leptos`'s `Select` uses one string
    /// for both — so the click handler maps back by matching this exact label.
    pub fn label(&self) -> String {
        let id8: String = self.id.chars().take(8).collect();
        let stack8: String = self.stack_id.chars().take(8).collect();
        let marker = if self.is_deletion_marker {
            " · deletion"
        } else {
            ""
        };
        format!("#{} · {id8} · stack {stack8}{marker}", self.sequence_id)
    }
}

// ---- diagnostics (BROKKR-T-0301) -----------------------------------------

/// `GET /api/v1/diagnostics/:id` — mirrors the broker's `DiagnosticResponse`:
/// the request, plus the result once an agent has submitted one.
#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct DiagnosticResponse {
    pub request: DiagnosticRequestDto,
    #[serde(default)]
    pub result: Option<DiagnosticResultDto>,
}

/// A diagnostic request (the 201 body of `POST /deployment-objects/:id/diagnostics`
/// and the `request` half of `GET /diagnostics/:id`). Only the fields the console
/// shows; unknown fields are ignored.
#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct DiagnosticRequestDto {
    pub id: String,
    #[serde(default)]
    pub status: String,
    #[serde(default)]
    pub claimed_at: Option<String>,
    #[serde(default)]
    pub completed_at: Option<String>,
}

/// A diagnostic result. **The three payload fields are JSON-encoded *strings***
/// (that is how the agent submits them and how the broker stores them), not
/// nested JSON — they have to be parsed a second time, which is what
/// [`DiagnosticData::parse`] does.
#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct DiagnosticResultDto {
    /// JSON array of [`PodStatus`].
    #[serde(default)]
    pub pod_statuses: String,
    /// JSON array of [`DiagEvent`].
    #[serde(default)]
    pub events: String,
    /// JSON object mapping `pod/container` to a log tail. `null` when the agent
    /// collected none.
    #[serde(default)]
    pub log_tails: Option<String>,
    #[serde(default)]
    pub collected_at: Option<String>,
}

/// One entry of the parsed `pod_statuses` array.
#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct PodStatus {
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub namespace: String,
    #[serde(default)]
    pub phase: String,
    #[serde(default)]
    pub containers: Vec<ContainerStatus>,
}

/// One container inside a [`PodStatus`].
#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct ContainerStatus {
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub ready: bool,
    #[serde(default)]
    pub restart_count: i32,
    #[serde(default)]
    pub state: String,
    #[serde(default)]
    pub state_reason: Option<String>,
}

/// One entry of the parsed `events` array.
///
/// `error` is not a Kubernetes event field: when collection fails the agent
/// submits `[{"error": "..."}]` here and the broker still marks the request
/// `completed` (there is no `failed` status — see
/// `docs/src/reference/diagnostics.md`, "Collection Errors"). Modelling it as an
/// optional field on the same struct lets one parse cover both shapes.
#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct DiagEvent {
    #[serde(default)]
    pub event_type: Option<String>,
    #[serde(default)]
    pub reason: Option<String>,
    #[serde(default)]
    pub message: Option<String>,
    #[serde(default)]
    pub involved_object: Option<String>,
    #[serde(default)]
    pub involved_object_kind: Option<String>,
    #[serde(default)]
    pub count: Option<i32>,
    #[serde(default)]
    pub last_timestamp: Option<String>,
    /// Present only on a collection-failure marker (see above).
    #[serde(default)]
    pub error: Option<String>,
}

/// The result's three JSON-in-string payloads, parsed. Each is kept as a
/// `Result` so a payload the console cannot read is reported as unreadable
/// rather than silently rendered as "nothing collected".
#[derive(Debug, Clone, PartialEq)]
pub struct DiagnosticData {
    pub pods: Result<Vec<PodStatus>, String>,
    pub events: Result<Vec<DiagEvent>, String>,
    /// `pod/container` -> log tail, sorted by key. Empty when `log_tails` was null.
    pub log_tails: Result<Vec<(String, String)>, String>,
    pub collected_at: Option<String>,
}

impl DiagnosticData {
    /// Parse the JSON-encoded payload strings.
    pub fn parse(dto: &DiagnosticResultDto) -> Self {
        fn json<T: serde::de::DeserializeOwned>(raw: &str) -> Result<T, String> {
            serde_json::from_str::<T>(raw).map_err(|e| e.to_string())
        }
        let log_tails = match dto.log_tails.as_deref().filter(|s| !s.is_empty()) {
            None => Ok(Vec::new()),
            Some(raw) => json::<std::collections::BTreeMap<String, String>>(raw)
                .map(|m| m.into_iter().collect()),
        };
        Self {
            pods: json(if dto.pod_statuses.is_empty() {
                "[]"
            } else {
                &dto.pod_statuses
            }),
            events: json(if dto.events.is_empty() {
                "[]"
            } else {
                &dto.events
            }),
            log_tails,
            collected_at: dto.collected_at.clone(),
        }
    }

    /// The collection-failure messages carried inside `events`, if any.
    ///
    /// Any event bearing an `error` key is a failure marker — real Kubernetes
    /// events never carry one. (The documented shape is exactly one such entry;
    /// accepting more can only make a real failure more visible, never turn a
    /// good collection into a false alarm.)
    pub fn collection_errors(&self) -> Vec<String> {
        match &self.events {
            Ok(evs) => evs.iter().filter_map(|e| e.error.clone()).collect(),
            Err(_) => Vec::new(),
        }
    }
}

/// What a polled diagnostic actually amounts to, once the `completed`-with-an-
/// `error`-event case is separated from a genuine collection.
#[derive(Debug, Clone, PartialEq)]
pub enum DiagnosticOutcome {
    /// `pending` or `claimed` — the agent has not submitted anything yet.
    InFlight,
    /// `completed`, but the result carries collection errors: the agent could not
    /// read the cluster. This is a FAILURE, not an empty success.
    CollectionFailed(Vec<String>),
    /// `completed` with a real result (which may legitimately be empty).
    Collected(Box<DiagnosticData>),
    /// A terminal status with nothing to show — `expired`, the reserved-but-never-
    /// set `failed`, or `completed` without a stored result. Carries the status.
    NoResult(String),
}

impl DiagnosticResponse {
    /// Whether the request has reached a state the agent will never move it out
    /// of, so polling can stop.
    pub fn is_terminal(&self) -> bool {
        matches!(
            self.request.status.to_ascii_lowercase().as_str(),
            "completed" | "failed" | "expired"
        )
    }

    /// Classify the response for rendering.
    pub fn outcome(&self) -> DiagnosticOutcome {
        let status = self.request.status.to_ascii_lowercase();
        match (status.as_str(), self.result.as_ref()) {
            ("pending" | "claimed", _) => DiagnosticOutcome::InFlight,
            (_, Some(dto)) => {
                let data = DiagnosticData::parse(dto);
                let errs = data.collection_errors();
                if errs.is_empty() {
                    DiagnosticOutcome::Collected(Box::new(data))
                } else {
                    DiagnosticOutcome::CollectionFailed(errs)
                }
            }
            (_, None) => DiagnosticOutcome::NoResult(self.request.status.clone()),
        }
    }
}

/// One named PAK (tenant) in `GET /api/v1/paks` — powers the scope selector
/// (BROKKR-I-0032). `id` is the generator id used as `?pak_id=`.
#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct PakSummary {
    pub id: String,
    pub name: String,
}

/// The broker's `ErrorResponse` body (`{ code, message, details? }`).
#[derive(Debug, Clone, Deserialize)]
pub struct ErrorBody {
    pub code: String,
    #[allow(dead_code)]
    #[serde(default)]
    pub message: String,
}

/// One internal-WS connection in `GET /api/v1/admin/ws/connections`.
#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct WsConnectionInfo {
    pub agent_id: String,
    #[serde(default)]
    pub connected_since: Option<String>,
    #[serde(default)]
    pub messages_in: u64,
    #[serde(default)]
    pub messages_out: u64,
}

/// `GET /api/v1/admin/ws/connections`.
#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct WsConnectionsResponse {
    #[serde(default)]
    pub connected_agents: usize,
    #[serde(default)]
    pub connections: Vec<WsConnectionInfo>,
    #[serde(default)]
    pub live_subscribers: usize,
}

/// `GET /api/v1/webhooks` (safe DTO — URL is redacted to `has_url`).
#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct WebhookSummary {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub enabled: bool,
    #[serde(default)]
    pub event_types: Vec<String>,
    #[serde(default)]
    pub has_url: bool,
}

/// `GET /api/v1/work-order-log` (completed work-order history).
#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct WorkOrderLogEntry {
    pub id: String,
    pub work_type: String,
    #[serde(default)]
    pub success: bool,
    #[serde(default)]
    pub retries_attempted: i32,
    #[serde(default)]
    pub result_message: Option<String>,
}

/// `GET /api/v1/stacks`.
#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct Stack {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub description: Option<String>,
    pub generator_id: String,
}

/// `GET /api/v1/agent-events` (agent lifecycle events: Apply/Heartbeat/Reconcile).
#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct AgentEventDto {
    #[serde(default)]
    pub agent_id: String,
    pub event_type: String,
    pub status: String,
    #[serde(default)]
    pub message: Option<String>,
}

/// `GET /api/v1/stacks/:id/health` — per-stack deployment-object health rollup.
#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct DeploymentObjectHealth {
    pub id: String,
    pub status: String,
    #[serde(default)]
    pub healthy_agents: usize,
    #[serde(default)]
    pub degraded_agents: usize,
    #[serde(default)]
    pub failing_agents: usize,
}

/// `GET /api/v1/stacks/:id/health`.
#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct StackHealth {
    #[serde(default)]
    pub overall_status: String,
    #[serde(default)]
    pub deployment_objects: Vec<DeploymentObjectHealth>,
}

/// `GET /api/v1/webhooks/:id/deliveries` — recent delivery attempts (summary).
#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct WebhookDeliveryDto {
    #[serde(default)]
    pub event_type: String,
    #[serde(default)]
    pub status: String,
    #[serde(default)]
    pub attempts: i32,
    #[serde(default)]
    pub last_error: Option<String>,
}

/// One work order in `GET /api/v1/work-orders` (admin-gated list). NOTE: requires
/// an admin PAK; with an operator-scoped PAK the Active panel renders an error.
#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct WorkOrder {
    pub id: String,
    pub work_type: String,
    pub status: String,
    #[serde(default)]
    pub retry_count: i32,
    #[serde(default)]
    pub claimed_by: Option<String>,
    #[serde(default)]
    pub last_error: Option<String>,
}

impl WorkOrder {
    /// Whether the order is still in flight (not in a terminal state).
    pub fn is_active(&self) -> bool {
        !matches!(
            self.status.to_ascii_lowercase().as_str(),
            "completed" | "failed" | "cancelled" | "canceled" | "succeeded"
        )
    }
}

/// One generator (tenant) from `GET /api/v1/generators`.
///
/// The broker never serializes `pak_hash` (`#[serde(skip_serializing)]`), so no
/// credential material reaches the console on this path — only the minting
/// response below carries a secret, and only once.
#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct Generator {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub is_active: bool,
    #[serde(default)]
    pub is_system: bool,
    #[serde(default)]
    pub created_at: Option<String>,
    #[serde(default)]
    pub last_active_at: Option<String>,
}

/// `POST /api/v1/generators` response — the created generator plus its
/// one-time PAK (BROKKR-T-0318).
///
/// `pak` is the only secret the console ever receives. It is unrecoverable
/// afterwards: the broker stores a hash, so losing it means rotating.
#[derive(Debug, Clone, Deserialize)]
pub struct CreateGeneratorResponse {
    pub generator: Generator,
    pub pak: String,
}
