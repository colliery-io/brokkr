/*
 * Copyright (c) 2025-2026 Dylan Storey
 * Licensed under the Elastic License 2.0.
 * See LICENSE file in the project root for full license text.
 */

//! Fleet legibility API endpoints (BROKKR-I-0027).
//!
//! These endpoints expose a per-agent "fleet record" of **measured values
//! only** (no health verdicts), computed entirely from data the broker already
//! holds. The rollup (`GET /fleet`) is computed with bounded, grouped queries
//! so it never fans out one query per agent.

use crate::api::v1::error::{ApiError, ErrorResponse};
use crate::api::v1::middleware::AuthPayload;
use crate::dal::DAL;
use crate::ws::ConnectionRegistry;
use axum::{
    Json,
    extract::{Extension, Path, State},
};
use brokkr_models::models::agent_events::AgentEvent;
use brokkr_models::models::deployment_health::{HEALTH_STATUS_DEGRADED, HEALTH_STATUS_FAILING};
use chrono::{DateTime, Utc};
use serde::Serialize;
use std::collections::HashMap;
use std::sync::Arc;
use tracing::{error, info};
use utoipa::ToSchema;
use uuid::Uuid;

/// Number of recent events returned by the per-agent fleet-status detail view.
const RECENT_EVENTS_LIMIT: usize = 20;

/// A per-agent fleet record: measured signals only, no health verdicts.
///
/// All time-relative fields (`heartbeat_age_seconds`, `seconds_since_last_event`)
/// are computed on read as `now - timestamp`, clamped to be non-negative.
#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct FleetAgentRecord {
    /// The agent's unique identifier.
    pub agent_id: Uuid,
    /// The agent's name.
    pub name: String,
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
    /// Latest agent-reported reachability of its own Kubernetes API
    /// (BROKKR-T-0227). `null` when the agent has never reported. The broker
    /// trusts this value as-is — it is the one fleet signal it cannot compute.
    pub k8s_reachable: Option<bool>,
    /// Latest agent-reported latency (milliseconds) of the Kubernetes API
    /// reachability probe. `null` when unreported or not measured.
    pub k8s_api_latency_ms: Option<i64>,
}

impl FleetAgentRecord {
    /// Convert into the `brokkr-wire` twin used for live-push frames
    /// (BROKKR-I-0028). The wire struct is intentionally `utoipa`/`diesel`-free;
    /// this is the single conversion point, so the two must stay field-aligned.
    pub fn to_wire(&self) -> brokkr_wire::FleetAgentRecord {
        brokkr_wire::FleetAgentRecord {
            agent_id: self.agent_id,
            name: self.name.clone(),
            status: self.status.clone(),
            ws_connected: self.ws_connected,
            connected_since: self.connected_since,
            last_heartbeat: self.last_heartbeat,
            heartbeat_age_seconds: self.heartbeat_age_seconds,
            pending_object_count: self.pending_object_count,
            pending_work_orders: self.pending_work_orders,
            claimed_work_orders: self.claimed_work_orders,
            last_event_at: self.last_event_at,
            seconds_since_last_event: self.seconds_since_last_event,
            health_failing: self.health_failing,
            health_degraded: self.health_degraded,
            k8s_reachable: self.k8s_reachable,
            k8s_api_latency_ms: self.k8s_api_latency_ms,
        }
    }
}

/// Response body for the per-agent fleet-status detail view: the agent's fleet
/// record plus its most recent events (newest first).
#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct AgentFleetStatusResponse {
    /// The per-agent fleet record (same shape as the rollup entries).
    pub record: FleetAgentRecord,
    /// The agent's most recent events, newest first (up to 20).
    pub recent_events: Vec<AgentEvent>,
}

/// Pre-aggregated, agent-keyed lookups shared by the rollup and detail views.
///
/// Each map/set is built from a single grouped query so assembling N records
/// costs O(N) in memory with no per-agent DB round-trips.
struct FleetAggregates {
    ws_connected_since: HashMap<Uuid, DateTime<Utc>>,
    last_event_at: HashMap<Uuid, DateTime<Utc>>,
    pending_objects: HashMap<Uuid, i64>,
    pending_work_orders: HashMap<Uuid, i64>,
    claimed_work_orders: HashMap<Uuid, i64>,
    health_failing: HashMap<Uuid, i64>,
    health_degraded: HashMap<Uuid, i64>,
}

impl FleetAggregates {
    /// Computes all per-agent aggregates with a bounded number of grouped
    /// queries (no N+1 fan-out over agents).
    fn load(dal: &DAL, registry: &ConnectionRegistry) -> Result<Self, ApiError> {
        let ws_connected_since: HashMap<Uuid, DateTime<Utc>> = registry
            .snapshot()
            .into_iter()
            .map(|c| (c.agent_id, c.connected_since))
            .collect();

        let last_event_at: HashMap<Uuid, DateTime<Utc>> = dal
            .agent_events()
            .last_event_at_by_agent()
            .map_err(|e| {
                error!("Failed to compute last_event_at_by_agent: {:?}", e);
                ApiError::internal("failed to compute fleet activity")
            })?
            .into_iter()
            .collect();

        let pending_objects: HashMap<Uuid, i64> = dal
            .deployment_objects()
            .pending_counts_by_agent()
            .map_err(|e| {
                error!("Failed to compute pending_counts_by_agent: {:?}", e);
                ApiError::internal("failed to compute fleet backpressure")
            })?
            .into_iter()
            .collect();

        let pending_work_orders: HashMap<Uuid, i64> = dal
            .work_orders()
            .pending_counts_by_agent()
            .map_err(|e| {
                error!("Failed to compute work order pending_counts_by_agent: {:?}", e);
                ApiError::internal("failed to compute fleet backpressure")
            })?
            .into_iter()
            .collect();

        let claimed_work_orders: HashMap<Uuid, i64> = dal
            .work_orders()
            .claimed_counts_by_agent()
            .map_err(|e| {
                error!("Failed to compute claimed_counts_by_agent: {:?}", e);
                ApiError::internal("failed to compute fleet backpressure")
            })?
            .into_iter()
            .collect();

        let mut health_failing: HashMap<Uuid, i64> = HashMap::new();
        let mut health_degraded: HashMap<Uuid, i64> = HashMap::new();
        for (agent_id, status, count) in dal
            .deployment_health()
            .status_counts_by_agent()
            .map_err(|e| {
                error!("Failed to compute status_counts_by_agent: {:?}", e);
                ApiError::internal("failed to compute fleet health counts")
            })?
        {
            if status == HEALTH_STATUS_FAILING {
                *health_failing.entry(agent_id).or_insert(0) += count;
            } else if status == HEALTH_STATUS_DEGRADED {
                *health_degraded.entry(agent_id).or_insert(0) += count;
            }
        }

        Ok(Self {
            ws_connected_since,
            last_event_at,
            pending_objects,
            pending_work_orders,
            claimed_work_orders,
            health_failing,
            health_degraded,
        })
    }

    /// Assembles a single agent's fleet record from the pre-aggregated lookups.
    fn build_record(
        &self,
        agent: &brokkr_models::models::agents::Agent,
        now: DateTime<Utc>,
    ) -> FleetAgentRecord {
        let connected_since = self.ws_connected_since.get(&agent.id).copied();
        let last_event_at = self.last_event_at.get(&agent.id).copied();

        FleetAgentRecord {
            agent_id: agent.id,
            name: agent.name.clone(),
            status: agent.status.clone(),
            ws_connected: connected_since.is_some(),
            connected_since,
            last_heartbeat: agent.last_heartbeat,
            heartbeat_age_seconds: agent
                .last_heartbeat
                .map(|hb| (now - hb).num_seconds().max(0)),
            pending_object_count: self.pending_objects.get(&agent.id).copied().unwrap_or(0),
            pending_work_orders: self.pending_work_orders.get(&agent.id).copied().unwrap_or(0),
            claimed_work_orders: self.claimed_work_orders.get(&agent.id).copied().unwrap_or(0),
            last_event_at,
            seconds_since_last_event: last_event_at.map(|e| (now - e).num_seconds().max(0)),
            health_failing: self.health_failing.get(&agent.id).copied().unwrap_or(0),
            health_degraded: self.health_degraded.get(&agent.id).copied().unwrap_or(0),
            k8s_reachable: agent.k8s_reachable,
            k8s_api_latency_ms: agent.k8s_api_latency_ms.map(i64::from),
        }
    }
}

/// Build a single agent's fleet record, or `None` if the agent no longer
/// exists. Used by the event-driven live-push producers (BROKKR-I-0028) on WS
/// connect/disconnect and heartbeat. Reuses the same `FleetAggregates`
/// assembly as `get_agent_fleet_status` so a pushed record is identical to the
/// pull surface. The aggregate queries are whole-fleet grouped queries (bounded,
/// no per-agent N+1), so this is cheap enough for the event cadence.
pub fn build_agent_fleet_record(
    dal: &DAL,
    registry: &ConnectionRegistry,
    agent_id: Uuid,
) -> Option<FleetAgentRecord> {
    let agent = match dal.agents().get(agent_id) {
        Ok(Some(agent)) => agent,
        Ok(None) => return None,
        Err(e) => {
            error!(
                "Failed to fetch agent {} for fleet live-push: {:?}",
                agent_id, e
            );
            return None;
        }
    };

    let aggregates = match FleetAggregates::load(dal, registry) {
        Ok(aggregates) => aggregates,
        Err(e) => {
            error!(
                "Failed to load fleet aggregates for live-push (agent {}): {:?}",
                agent_id, e
            );
            return None;
        }
    };

    Some(aggregates.build_record(&agent, Utc::now()))
}

/// Best-effort: recompute one agent's fleet record and broadcast it as a
/// `FleetUpdate` to every `/api/v1/fleet/live` subscriber. **Never returns an
/// error and never panics** — a push failure (agent gone, DB hiccup, no
/// subscribers) must not affect the triggering operation (heartbeat,
/// connect/disconnect). This is the single producer entry point.
pub fn broadcast_agent_fleet_update(
    dal: &DAL,
    registry: &ConnectionRegistry,
    fleet: &crate::ws::FleetBroadcaster,
    agent_id: Uuid,
) {
    if let Some(record) = build_agent_fleet_record(dal, registry, agent_id) {
        fleet.broadcast(brokkr_wire::WsMessage::FleetUpdate(record.to_wire()));
    }
}

/// Builds the full per-agent fleet record set (the `GET /fleet` rollup payload).
/// Shared by the pull handler and the periodic live-push sweep (I-0028 Slice 2).
/// Bounded grouped queries — no per-agent N+1.
pub fn build_all_fleet_records(
    dal: &DAL,
    registry: &ConnectionRegistry,
) -> Result<Vec<FleetAgentRecord>, ApiError> {
    let agents = dal.agents().list().map_err(|e| {
        error!("Failed to fetch agents for fleet rollup: {:?}", e);
        ApiError::internal("failed to fetch agents")
    })?;
    let aggregates = FleetAggregates::load(dal, registry)?;
    let now = Utc::now();
    Ok(agents
        .iter()
        .map(|agent| aggregates.build_record(agent, now))
        .collect())
}

fn require_admin(auth: &AuthPayload) -> Result<(), ApiError> {
    if auth.admin {
        Ok(())
    } else {
        Err(ApiError::forbidden(
            "admin_required",
            "admin access required",
        ))
    }
}

#[utoipa::path(
    get, path = "/fleet", tag = "fleet",
    responses(
        (status = 200, description = "Per-agent fleet records (measured values only)", body = Vec<FleetAgentRecord>),
        (status = 403, description = "Forbidden", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse),
    ),
    security(("admin_pak" = []))
)]
pub async fn list_fleet(
    State(dal): State<DAL>,
    Extension(auth_payload): Extension<AuthPayload>,
    Extension(registry): Extension<Arc<ConnectionRegistry>>,
) -> Result<Json<Vec<FleetAgentRecord>>, ApiError> {
    info!("Handling request to list fleet records");
    require_admin(&auth_payload)?;

    let records = build_all_fleet_records(&dal, &registry)?;
    info!("Successfully assembled {} fleet records", records.len());
    Ok(Json(records))
}

#[utoipa::path(
    get, path = "/agents/{id}/fleet-status", tag = "fleet",
    params(("id" = Uuid, Path, description = "ID of the agent")),
    responses(
        (status = 200, description = "Fleet record plus recent events for the agent", body = AgentFleetStatusResponse),
        (status = 403, description = "Forbidden", body = ErrorResponse),
        (status = 404, description = "Agent not found", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse),
    ),
    security(("admin_pak" = []))
)]
pub async fn get_agent_fleet_status(
    State(dal): State<DAL>,
    Extension(auth_payload): Extension<AuthPayload>,
    Extension(registry): Extension<Arc<ConnectionRegistry>>,
    Path(id): Path<Uuid>,
) -> Result<Json<AgentFleetStatusResponse>, ApiError> {
    info!("Handling request to get fleet status for agent {}", id);
    require_admin(&auth_payload)?;

    let agent = dal
        .agents()
        .get(id)
        .map_err(|e| {
            error!("Failed to fetch agent {} for fleet status: {:?}", id, e);
            ApiError::internal("failed to fetch agent")
        })?
        .ok_or_else(|| ApiError::not_found("agent_not_found", "agent not found"))?;

    let aggregates = FleetAggregates::load(&dal, &registry)?;
    let now = Utc::now();
    let record = aggregates.build_record(&agent, now);

    // get_events returns newest-first; cap to the most recent N.
    let recent_events = dal
        .agent_events()
        .get_events(None, Some(id))
        .map_err(|e| {
            error!("Failed to fetch recent events for agent {}: {:?}", id, e);
            ApiError::internal("failed to fetch agent events")
        })?
        .into_iter()
        .take(RECENT_EVENTS_LIMIT)
        .collect::<Vec<_>>();

    Ok(Json(AgentFleetStatusResponse {
        record,
        recent_events,
    }))
}
