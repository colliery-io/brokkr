//! Serde mirrors of the broker's JSON responses the console reads. Kept local
//! (not the broker's diesel-bound types) so the wasm crate stays light.

use serde::Deserialize;

/// One agent in `GET /api/v1/fleet` (mirrors the broker `FleetAgentRecord`).
/// NOTE: the fleet record carries no `cluster_name`/`labels` today, so the
/// console renders a flat agent list rather than the handoff's per-cluster
/// grouping — that needs a broker enhancement (see the Fleet task).
#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct FleetAgentRecord {
    pub agent_id: String,
    pub name: String,
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
