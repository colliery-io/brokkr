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
