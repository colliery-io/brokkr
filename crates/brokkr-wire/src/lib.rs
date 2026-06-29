/*
 * Copyright (c) 2025-2026 Dylan Storey
 * Licensed under the Elastic License 2.0.
 * See LICENSE file in the project root for full license text.
 */

//! # brokkr-wire
//!
//! Wire protocol for the **internal** WebSocket channel between `brokkr-broker`
//! and `brokkr-agent`. See [[BROKKR-A-0008]] and [[BROKKR-I-0019]] in `.metis/`.
//!
//! This is not a public API. It is not generated into the SDKs and is not part
//! of the OpenAPI surface. External integrators use the REST API.
//!
//! Bodies that already exist as REST/SDK types are re-exported from
//! `brokkr-models` so the wire and the REST contract share one definition.
//! Types that have no REST equivalent (heartbeat, kube event passthrough,
//! pod log line, gap markers) are defined here.
//!
//! The crate version is pinned in lockstep with `brokkr-broker` and the SDKs
//! per the project release-versioning convention; bumping the protocol means
//! bumping the release.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

// Re-exports from brokkr-models: these are the canonical body types shared
// with REST. Aliased here so call sites read against the wire crate.
pub use brokkr_models::models::agent_events::AgentEvent;
pub use brokkr_models::models::agent_targets::AgentTarget;
pub use brokkr_models::models::deployment_health::DeploymentHealth;
pub use brokkr_models::models::stacks::Stack;
pub use brokkr_models::models::work_orders::WorkOrder;

/// Heartbeat from agent to broker. Sent on a tick while the WS connection is
/// up; absence drives broker-side liveness for diagnostics.
///
/// `k8s_reachable` / `k8s_api_latency_ms` carry the one fleet signal the
/// broker cannot compute itself (BROKKR-T-0227): whether the agent can reach
/// its own Kubernetes API. Both are optional and `#[serde(default)]` so older
/// agents that omit them still deserialize, and agents that cannot determine
/// reachability simply leave them `None` (graceful degradation).
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Heartbeat {
    pub agent_id: Uuid,
    pub sent_at: DateTime<Utc>,
    /// Whether the agent can reach its own Kubernetes API, if it probed.
    #[serde(default)]
    pub k8s_reachable: Option<bool>,
    /// Measured latency (milliseconds) of the reachability probe, if any.
    #[serde(default)]
    pub k8s_api_latency_ms: Option<i32>,
}

/// Kubernetes object reference for events and log lines. Mirrors the fields
/// from `corev1.ObjectReference` that we actually care about — keeping it
/// local avoids dragging `k8s-openapi` into every consumer of this crate.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ObjectRef {
    pub api_version: String,
    pub kind: String,
    pub namespace: Option<String>,
    pub name: String,
    pub uid: Option<String>,
}

/// A Kubernetes `Event` for an object the agent manages, forwarded upstream
/// for short-lived operational visibility. Persisted under the retention
/// stance documented in `project_log_retention_stance` (hard 6h ceiling).
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct K8sEvent {
    pub agent_id: Uuid,
    pub stack_id: Uuid,
    pub observed_at: DateTime<Utc>,
    /// Reason field from the kube Event (e.g. `FailedScheduling`, `Pulled`).
    pub reason: String,
    pub message: String,
    /// Event type: typically `Normal` or `Warning`.
    pub event_type: String,
    pub source: Option<String>,
    pub involved_object: ObjectRef,
}

/// A single line of pod log output forwarded upstream. Per-stack opt-in
/// gating and agent-side rate limiting are enforced before send (see WS-08).
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct PodLogLine {
    pub agent_id: Uuid,
    pub stack_id: Uuid,
    pub namespace: String,
    pub pod: String,
    pub container: String,
    pub ts: DateTime<Utc>,
    pub line: String,
}

/// Reason a sequence of log lines was dropped before reaching the broker.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GapReason {
    RateLimit,
    BufferFull,
    Disconnected,
}

/// Marker emitted when log lines were dropped so consumers can render a
/// visible gap rather than silently presenting an incomplete tail.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct LogGap {
    pub agent_id: Uuid,
    pub stack_id: Uuid,
    pub since_ts: DateTime<Utc>,
    pub dropped_count: u64,
    pub reason: GapReason,
}

/// A per-agent fleet record: measured signals only, no health verdicts
/// (BROKKR-I-0028, the live-push twin of the broker's REST `FleetAgentRecord`).
///
/// This is the **wire** representation, intentionally a plain serde struct so
/// `brokkr-wire` stays free of `utoipa`/`diesel`. The broker owns the
/// authoritative `utoipa::ToSchema` version (`api/v1/fleet.rs:FleetAgentRecord`,
/// which is what the OpenAPI surface exposes) and converts it into this twin
/// before broadcasting. The two must stay field-for-field identical.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FleetAgentRecord {
    /// The agent's unique identifier.
    pub agent_id: Uuid,
    /// The agent's name.
    pub name: String,
    /// The Kubernetes cluster the agent runs in (used to group the fleet).
    pub cluster_name: String,
    /// The agent's lifecycle status (e.g. "ACTIVE").
    pub status: String,
    /// Whether the agent currently holds a broker↔agent WebSocket connection.
    pub ws_connected: bool,
    /// When the current WebSocket connection was established, if connected.
    pub connected_since: Option<DateTime<Utc>>,
    /// The agent's last recorded heartbeat timestamp.
    pub last_heartbeat: Option<DateTime<Utc>>,
    /// Seconds since the last heartbeat (`now - last_heartbeat`, clamped >= 0).
    pub heartbeat_age_seconds: Option<i64>,
    /// Number of pending (not-yet-acknowledged) deployment objects targeted at
    /// this agent.
    pub pending_object_count: i64,
    /// Number of PENDING work orders this agent is eligible to claim.
    pub pending_work_orders: i64,
    /// Number of work orders currently CLAIMED by this agent.
    pub claimed_work_orders: i64,
    /// Timestamp of this agent's most recent (non-deleted) event, if any.
    pub last_event_at: Option<DateTime<Utc>>,
    /// Seconds since the last event (`now - last_event_at`, clamped >= 0).
    pub seconds_since_last_event: Option<i64>,
    /// Count of this agent's deployment-health records with status `failing`.
    pub health_failing: i64,
    /// Count of this agent's deployment-health records with status `degraded`.
    pub health_degraded: i64,
    /// Latest agent-reported reachability of its own Kubernetes API.
    pub k8s_reachable: Option<bool>,
    /// Latest agent-reported latency (milliseconds) of the reachability probe.
    pub k8s_api_latency_ms: Option<i64>,
}

/// The canonical message envelope on the broker↔agent WebSocket. JSON-encoded
/// with an external tag so additions are forward-compatible.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "body", rename_all = "snake_case")]
pub enum WsMessage {
    // Control plane — broker → agent
    WorkOrder(WorkOrder),
    TargetChanged(AgentTarget),
    StackChanged(Stack),

    // Agent → broker uplink
    Heartbeat(Heartbeat),
    AgentEvent(AgentEvent),
    AgentHealth(DeploymentHealth),

    // Streaming telemetry — agent → broker
    K8sEvent(K8sEvent),
    PodLogLine(PodLogLine),
    LogGap(LogGap),

    // Consumer-facing fleet live-push — broker → consumer (BROKKR-I-0028).
    // Carries one agent's full fleet record; the consumer replaces its row.
    FleetUpdate(FleetAgentRecord),
}

/// Wire-protocol version. Matches the crate version, which matches the
/// broker/SDK release version (lockstep per `project_release_versioning`).
pub const WIRE_VERSION: &str = env!("CARGO_PKG_VERSION");
