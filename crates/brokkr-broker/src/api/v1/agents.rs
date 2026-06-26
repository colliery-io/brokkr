/*
 * Copyright (c) 2025-2026 Dylan Storey
 * Licensed under the Elastic License 2.0.
 * See LICENSE file in the project root for full license text.
 */

//! Agent management API endpoints.

use crate::api::v1::error::{ApiError, ErrorResponse};
use crate::api::v1::middleware::AuthPayload;
use crate::dal::DAL;
use crate::metrics;
use crate::utils::{audit, event_bus, pak};
use crate::ws::{ConnectionRegistry, push_target_changed};
use axum::http::StatusCode;
use axum::{
    Json, Router,
    extract::{Extension, Path, Query, State},
    routing::{delete, get, post},
};
use brokkr_models::models::agent_annotations::{AgentAnnotation, NewAgentAnnotation};
use brokkr_models::models::agent_events::{AgentEvent, NewAgentEvent};
use brokkr_models::models::agent_labels::{AgentLabel, NewAgentLabel};
use brokkr_models::models::agent_generator_registrations::AgentGeneratorRegistration;
use brokkr_models::models::agent_targets::{AgentTarget, NewAgentTarget};
use brokkr_models::models::agents::{Agent, NewAgent};
use brokkr_models::models::audit_logs::{
    ACTION_AGENT_CREATED, ACTION_AGENT_DELETED, ACTION_AGENT_UPDATED, ACTION_PAK_CREATED,
    ACTION_PAK_ROTATED, ACTOR_TYPE_ADMIN, RESOURCE_TYPE_AGENT, RESOURCE_TYPE_PAK,
};
use brokkr_models::models::deployment_objects::DeploymentObject;
use brokkr_models::models::stacks::Stack;
use brokkr_models::models::webhooks::{
    BrokkrEvent, EVENT_DEPLOYMENT_APPLIED, EVENT_DEPLOYMENT_FAILED,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::sync::Arc;
use tracing::{error, info, warn};
use utoipa::ToSchema;
use uuid::Uuid;

pub fn routes() -> Router<DAL> {
    info!("Setting up agent routes");
    Router::new()
        .route("/agents", get(list_agents).post(create_agent))
        .route("/agents/", get(search_agent))
        .route(
            "/agents/:id",
            get(get_agent).put(update_agent).delete(delete_agent),
        )
        .route("/agents/:id/events", get(list_events).post(create_event))
        .route("/agents/:id/labels", get(list_labels).post(add_label))
        .route("/agents/:id/labels/:label", delete(remove_label))
        .route(
            "/agents/:id/annotations",
            get(list_annotations).post(add_annotation),
        )
        .route("/agents/:id/annotations/:key", delete(remove_annotation))
        .route("/agents/:id/targets", get(list_targets).post(add_target))
        .route("/agents/:id/targets/:stack_id", delete(remove_target))
        .route("/agents/:id/heartbeat", post(record_heartbeat))
        // Fleet legibility (BROKKR-I-0027): broker-computed fleet surface.
        .route("/fleet", get(crate::api::v1::fleet::list_fleet))
        .route(
            "/agents/:id/fleet-status",
            get(crate::api::v1::fleet::get_agent_fleet_status),
        )
        .route("/agents/:id/target-state", get(get_target_state))
        .route("/agents/:id/stacks", get(get_associated_stacks))
        .route("/agents/:id/rotate-pak", post(rotate_agent_pak))
        .route("/agents/:id/registrations", get(list_agent_registrations))
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

fn require_admin_or_agent(auth: &AuthPayload, id: Uuid) -> Result<(), ApiError> {
    if auth.admin || auth.agent == Some(id) {
        Ok(())
    } else {
        Err(ApiError::forbidden(
            "agent_pak_mismatch",
            "not authorized for this agent",
        ))
    }
}

#[utoipa::path(
    get, path = "/agents", tag = "agents",
    responses(
        (status = 200, description = "Successfully retrieved agents", body = Vec<Agent>),
        (status = 403, description = "Forbidden", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse),
    ),
    security(("admin_pak" = []))
)]
async fn list_agents(
    State(dal): State<DAL>,
    Extension(auth_payload): Extension<AuthPayload>,
) -> Result<Json<Vec<Agent>>, ApiError> {
    info!("Handling request to list agents");
    require_admin(&auth_payload)?;

    let agents = dal.agents().list().map_err(|e| {
        error!("Failed to fetch agents: {:?}", e);
        ApiError::internal("failed to fetch agents")
    })?;
    info!("Successfully retrieved {} agents", agents.len());
    let active_count = agents.iter().filter(|a| a.status == "ACTIVE").count();
    metrics::set_active_agents(active_count as i64);

    let now = chrono::Utc::now();
    for agent in &agents {
        if let Some(last_hb) = agent.last_heartbeat {
            let age_seconds = (now - last_hb).num_seconds().max(0) as f64;
            metrics::set_agent_heartbeat_age(&agent.id.to_string(), &agent.name, age_seconds);
        }
    }
    Ok(Json(agents))
}

/// Request body for [`create_agent`]. Extends `NewAgent` with an optional list
/// of generator UUIDs the new agent should be registered with at creation time.
/// The system generator is always added automatically; this field is additive.
#[derive(Debug, Deserialize, ToSchema)]
pub struct CreateAgentRequest {
    pub name: String,
    pub cluster_name: String,
    #[serde(default)]
    pub generator_ids: Vec<Uuid>,
}

/// Response body for [`create_agent`]: the newly-created agent plus the
/// one-time initial PAK shown only at creation.
#[derive(Debug, Serialize, ToSchema)]
pub struct CreateAgentResponse {
    pub agent: Agent,
    pub initial_pak: String,
}

#[utoipa::path(
    post, path = "/agents", tag = "agents",
    request_body = CreateAgentRequest,
    responses(
        (status = 201, description = "Successfully created agent", body = CreateAgentResponse),
        (status = 400, description = "Invalid generator ID", body = ErrorResponse),
        (status = 403, description = "Forbidden", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse),
    ),
    security(("admin_pak" = []))
)]
async fn create_agent(
    State(dal): State<DAL>,
    Extension(auth_payload): Extension<AuthPayload>,
    Json(req): Json<CreateAgentRequest>,
) -> Result<(StatusCode, Json<CreateAgentResponse>), ApiError> {
    info!("Handling request to create a new agent");
    require_admin(&auth_payload)?;

    // Validate all requested generator_ids before touching the DB.
    for gid in &req.generator_ids {
        dal.generators()
            .get(*gid)
            .map_err(|e| ApiError::from_diesel(e, "failed to look up generator"))?
            .ok_or_else(|| {
                ApiError::bad_request(
                    "invalid_generator_id",
                    &format!("generator {} does not exist", gid),
                )
            })?;
    }

    let new_agent = NewAgent::new(req.name.clone(), req.cluster_name.clone())
        .map_err(|e| ApiError::bad_request("invalid_agent", &e))?;

    let agent = dal.agents().create(&new_agent).map_err(|e| {
        warn!("Failed to create agent: {:?}", e);
        ApiError::from_diesel(e, "failed to create agent")
    })?;
    info!("Successfully created agent with ID: {}", agent.id);

    let (pak_value, pak_hash) = pak::create_pak().map_err(|e| {
        error!("Failed to create PAK: {:?}", e);
        ApiError::internal("failed to create PAK")
    })?;

    let updated_agent = dal
        .agents()
        .update_pak_hash(agent.id, pak_hash)
        .map_err(|e| {
            error!("Failed to update agent PAK hash: {:?}", e);
            ApiError::internal("failed to update agent PAK hash")
        })?;

    // Register with system generator (always).
    if let Some(system_id) = dal
        .generators()
        .get_system_generator_id()
        .map_err(|e| ApiError::from_diesel(e, "failed to look up system generator"))?
    {
        dal.agent_generator_registrations()
            .create(agent.id, system_id)
            .map_err(|e| ApiError::from_diesel(e, "failed to register agent with system generator"))?;
    }

    // Register with any additional generators supplied in the request.
    for gid in &req.generator_ids {
        dal.agent_generator_registrations()
            .create(agent.id, *gid)
            .map_err(|e| ApiError::from_diesel(e, "failed to register agent with generator"))?;
    }

    audit::log_action(
        ACTOR_TYPE_ADMIN,
        None,
        ACTION_AGENT_CREATED,
        RESOURCE_TYPE_AGENT,
        Some(agent.id),
        Some(serde_json::json!({
            "name": updated_agent.name,
            "cluster_name": updated_agent.cluster_name,
        })),
        None,
        None,
    );
    audit::log_action(
        ACTOR_TYPE_ADMIN,
        None,
        ACTION_PAK_CREATED,
        RESOURCE_TYPE_PAK,
        Some(agent.id),
        Some(serde_json::json!({ "entity": "agent", "name": updated_agent.name })),
        None,
        None,
    );
    Ok((
        StatusCode::CREATED,
        Json(CreateAgentResponse {
            agent: updated_agent,
            initial_pak: pak_value,
        }),
    ))
}

#[derive(Deserialize)]
struct AgentQuery {
    name: Option<String>,
    cluster_name: Option<String>,
}

#[utoipa::path(
    get, path = "/agents/{id}", tag = "agents",
    params(("id" = Uuid, Path, description = "ID of the agent to retrieve")),
    responses(
        (status = 200, description = "Successfully retrieved agent", body = Agent),
        (status = 403, description = "Forbidden", body = ErrorResponse),
        (status = 404, description = "Agent not found", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse),
    ),
    security(("admin_pak" = []), ("agent_pak" = []))
)]
async fn get_agent(
    State(dal): State<DAL>,
    Extension(auth_payload): Extension<AuthPayload>,
    Path(id): Path<Uuid>,
) -> Result<Json<Agent>, ApiError> {
    info!("Handling request to get agent by ID: {}", id);
    require_admin_or_agent(&auth_payload, id)?;
    let agent = dal
        .agents()
        .get(id)
        .map_err(|e| {
            error!("Failed to fetch agent with ID {}: {:?}", id, e);
            ApiError::internal("failed to fetch agent")
        })?
        .ok_or_else(|| ApiError::not_found("agent_not_found", "agent not found"))?;
    info!("Successfully retrieved agent with ID: {}", id);
    Ok(Json(agent))
}

#[utoipa::path(
    get, path = "/agents/", tag = "agents",
    params(
        ("name" = Option<String>, Query, description = "Name of the agent to search for"),
        ("cluster_name" = Option<String>, Query, description = "Name of the cluster to search in"),
    ),
    responses(
        (status = 200, description = "Successfully found agent", body = Agent),
        (status = 400, description = "Invalid request", body = ErrorResponse),
        (status = 403, description = "Forbidden", body = ErrorResponse),
        (status = 404, description = "Agent not found", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse),
    ),
    security(("admin_pak" = []), ("agent_pak" = []))
)]
async fn search_agent(
    State(dal): State<DAL>,
    Extension(auth_payload): Extension<AuthPayload>,
    Query(query): Query<AgentQuery>,
) -> Result<Json<Agent>, ApiError> {
    info!("Handling request to search for agent");
    let (name, cluster_name) = match (query.name.clone(), query.cluster_name.clone()) {
        (Some(n), Some(c)) => (n, c),
        _ => {
            return Err(ApiError::bad_request(
                "name_and_cluster_required",
                "name and cluster_name are required",
            ));
        }
    };

    let agent = dal
        .agents()
        .get_by_name_and_cluster_name(name, cluster_name)
        .map_err(|e| {
            error!("Failed to fetch agent: {:?}", e);
            ApiError::internal("failed to fetch agent")
        })?
        .ok_or_else(|| ApiError::not_found("agent_not_found", "agent not found"))?;

    if !auth_payload.admin && auth_payload.agent != Some(agent.id) {
        return Err(ApiError::forbidden(
            "agent_pak_mismatch",
            "not authorized for this agent",
        ));
    }
    info!("Successfully found agent with ID: {}", agent.id);
    Ok(Json(agent))
}

#[utoipa::path(
    put, path = "/agents/{id}", tag = "agents",
    params(("id" = Uuid, Path, description = "ID of the agent to update")),
    request_body = serde_json::Value,
    responses(
        (status = 200, description = "Successfully updated agent", body = Agent),
        (status = 403, description = "Forbidden", body = ErrorResponse),
        (status = 404, description = "Agent not found", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse),
    ),
    security(("admin_pak" = []), ("agent_pak" = []))
)]
async fn update_agent(
    State(dal): State<DAL>,
    Extension(auth_payload): Extension<AuthPayload>,
    Path(id): Path<Uuid>,
    Json(update_payload): Json<serde_json::Value>,
) -> Result<Json<Agent>, ApiError> {
    info!("Handling request to update agent with ID: {}", id);
    require_admin_or_agent(&auth_payload, id)?;

    let mut agent = dal
        .agents()
        .get(id)
        .map_err(|e| {
            error!("Failed to fetch agent with ID {}: {:?}", id, e);
            ApiError::internal("failed to fetch agent")
        })?
        .ok_or_else(|| ApiError::not_found("agent_not_found", "agent not found"))?;

    if let Some(name) = update_payload.get("name").and_then(|v| v.as_str()) {
        agent.name = name.to_string();
    }
    if let Some(cluster_name) = update_payload.get("cluster_name").and_then(|v| v.as_str()) {
        agent.cluster_name = cluster_name.to_string();
    }
    if let Some(status) = update_payload.get("status").and_then(|v| v.as_str()) {
        agent.status = status.to_string();
    }

    let updated_agent = dal
        .agents()
        .update(id, &agent)
        .map_err(|e| ApiError::from_diesel(e, format!("failed to update agent {id}")))?;
    info!("Successfully updated agent with ID: {}", id);

    audit::log_action(
        ACTOR_TYPE_ADMIN,
        None,
        ACTION_AGENT_UPDATED,
        RESOURCE_TYPE_AGENT,
        Some(id),
        Some(serde_json::json!({
            "name": updated_agent.name,
            "cluster_name": updated_agent.cluster_name,
            "status": updated_agent.status,
        })),
        None,
        None,
    );

    Ok(Json(updated_agent))
}

#[utoipa::path(
    delete, path = "/agents/{id}", tag = "agents",
    params(("id" = Uuid, Path, description = "ID of the agent to delete")),
    responses(
        (status = 204, description = "Successfully deleted agent"),
        (status = 403, description = "Forbidden", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse),
    ),
    security(("admin_pak" = []))
)]
async fn delete_agent(
    State(dal): State<DAL>,
    Extension(auth_payload): Extension<AuthPayload>,
    Extension(ws_registry): Extension<Arc<ConnectionRegistry>>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, ApiError> {
    info!("Handling request to delete agent with ID: {}", id);
    require_admin(&auth_payload)?;
    let old_pak_hash = dal.agents().get(id).ok().flatten().map(|a| a.pak_hash);

    dal.agents().soft_delete(id).map_err(|e| {
        error!("Failed to delete agent with ID {}: {:?}", id, e);
        ApiError::internal("failed to delete agent")
    })?;
    info!("Successfully deleted agent with ID: {}", id);
    if let Some(ref hash) = old_pak_hash {
        dal.invalidate_auth_cache(hash);
    }
    // Agent is gone and its PAK invalidated; close any open WS (BROKKR-T-0176).
    let closed = ws_registry.close_for_agent(id);
    if closed > 0 {
        info!(%id, closed, "closed open WS connection(s) after agent deletion");
    }
    audit::log_action(
        ACTOR_TYPE_ADMIN,
        None,
        ACTION_AGENT_DELETED,
        RESOURCE_TYPE_AGENT,
        Some(id),
        None,
        None,
        None,
    );
    Ok(StatusCode::NO_CONTENT)
}

#[utoipa::path(
    get, path = "/agents/{id}/events", tag = "agent-events",
    params(("id" = Uuid, Path, description = "ID of the agent to list events for")),
    responses(
        (status = 200, description = "Successfully retrieved agent events", body = Vec<AgentEvent>),
        (status = 403, description = "Forbidden", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse),
    ),
    security(("admin_pak" = []), ("agent_pak" = []))
)]
async fn list_events(
    State(dal): State<DAL>,
    Extension(auth_payload): Extension<AuthPayload>,
    Path(id): Path<Uuid>,
) -> Result<Json<Vec<Value>>, ApiError> {
    info!("Handling request to list events for agent with ID: {}", id);
    require_admin_or_agent(&auth_payload, id)?;
    let events = dal.agent_events().get_events(None, Some(id)).map_err(|e| {
        error!("Failed to fetch events for agent with ID {}: {:?}", id, e);
        ApiError::internal("failed to fetch agent events")
    })?;
    info!(
        "Successfully retrieved {} events for agent with ID: {}",
        events.len(),
        id
    );
    let values = events
        .into_iter()
        .map(serde_json::to_value)
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| ApiError::internal(format!("failed to serialize agent events: {e}")))?;
    Ok(Json(values))
}

#[utoipa::path(
    post, path = "/agents/{id}/events", tag = "agent-events",
    params(("id" = Uuid, Path, description = "ID of the agent to create an event for")),
    request_body = NewAgentEvent,
    responses(
        (status = 201, description = "Successfully created agent event", body = AgentEvent),
        (status = 403, description = "Forbidden", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse),
    ),
    security(("admin_pak" = []), ("agent_pak" = []))
)]
async fn create_event(
    State(dal): State<DAL>,
    Extension(auth_payload): Extension<AuthPayload>,
    Path(id): Path<Uuid>,
    Json(new_event): Json<NewAgentEvent>,
) -> Result<(StatusCode, Json<AgentEvent>), ApiError> {
    info!("Handling request to create event for agent with ID: {}", id);
    require_admin_or_agent(&auth_payload, id)?;
    // The path id is what was authorized; the body must not attribute the event
    // to a different agent (these events feed the webhook bus).
    if new_event.agent_id != id {
        return Err(ApiError::bad_request(
            "agent_id_mismatch",
            "body agent_id must match the path agent id",
        ));
    }

    let event = dal
        .agent_events()
        .create(&new_event)
        .map_err(|e| ApiError::from_diesel(e, format!("failed to create event for agent {id}")))?;
    info!("Successfully created event for agent with ID: {}", id);

    let webhook_event_type = if new_event.status.to_uppercase() == "SUCCESS" {
        EVENT_DEPLOYMENT_APPLIED
    } else {
        EVENT_DEPLOYMENT_FAILED
    };
    let event_data = serde_json::json!({
        "agent_event_id": event.id,
        "agent_id": event.agent_id,
        "deployment_object_id": event.deployment_object_id,
        "event_type": event.event_type,
        "status": event.status,
        "message": event.message,
        "created_at": event.created_at,
    });
    event_bus::emit_event(&dal, &BrokkrEvent::new(webhook_event_type, event_data));

    Ok((StatusCode::CREATED, Json(event)))
}

#[utoipa::path(
    get, path = "/agents/{id}/labels", tag = "agent-labels",
    params(("id" = Uuid, Path, description = "ID of the agent")),
    responses(
        (status = 200, description = "Successfully retrieved agent labels", body = Vec<AgentLabel>),
        (status = 403, description = "Forbidden", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse),
    ),
    security(("admin_pak" = []), ("agent_pak" = []))
,
    operation_id = "agents_list_labels"
)]
async fn list_labels(
    State(dal): State<DAL>,
    Extension(auth_payload): Extension<AuthPayload>,
    Path(id): Path<Uuid>,
) -> Result<Json<Vec<AgentLabel>>, ApiError> {
    info!("Handling request to list labels for agent with ID: {}", id);
    require_admin_or_agent(&auth_payload, id)?;
    let labels = dal.agent_labels().list_for_agent(id).map_err(|e| {
        error!("Failed to fetch labels for agent with ID {}: {:?}", id, e);
        ApiError::internal("failed to fetch agent labels")
    })?;
    info!(
        "Successfully retrieved {} labels for agent with ID: {}",
        labels.len(),
        id
    );
    Ok(Json(labels))
}

#[utoipa::path(
    post, path = "/agents/{id}/labels", tag = "agent-labels",
    params(("id" = Uuid, Path, description = "ID of the agent")),
    request_body = NewAgentLabel,
    responses(
        (status = 201, description = "Successfully added agent label", body = AgentLabel),
        (status = 403, description = "Forbidden", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse),
    ),
    security(("admin_pak" = []))
,
    operation_id = "agents_add_label"
)]
async fn add_label(
    State(dal): State<DAL>,
    Extension(auth_payload): Extension<AuthPayload>,
    Path(id): Path<Uuid>,
    Json(new_label): Json<NewAgentLabel>,
) -> Result<(StatusCode, Json<AgentLabel>), ApiError> {
    info!("Handling request to add label for agent with ID: {}", id);
    require_admin(&auth_payload)?;
    if new_label.agent_id != id {
        return Err(ApiError::bad_request(
            "agent_id_mismatch",
            "body agent_id must match the path agent id",
        ));
    }
    let label = dal
        .agent_labels()
        .create(&new_label)
        .map_err(|e| ApiError::from_diesel(e, format!("failed to add label for agent {id}")))?;
    info!("Successfully added label for agent with ID: {}", id);
    Ok((StatusCode::CREATED, Json(label)))
}

#[utoipa::path(
    delete, path = "/agents/{id}/labels/{label}", tag = "agent-labels",
    params(
        ("id" = Uuid, Path, description = "ID of the agent"),
        ("label" = String, Path, description = "The label to remove"),
    ),
    responses(
        (status = 204, description = "Successfully removed agent label"),
        (status = 403, description = "Forbidden", body = ErrorResponse),
        (status = 404, description = "Label not found", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse),
    ),
    security(("admin_pak" = []))
,
    operation_id = "agents_remove_label"
)]
async fn remove_label(
    State(dal): State<DAL>,
    Extension(auth_payload): Extension<AuthPayload>,
    Path((id, label)): Path<(Uuid, String)>,
) -> Result<StatusCode, ApiError> {
    info!(
        "Handling request to remove label '{}' from agent with ID: {}",
        label, id
    );
    require_admin(&auth_payload)?;
    let deleted = dal
        .agent_labels()
        .delete_by_agent_and_label(id, &label)
        .map_err(|e| {
            error!(
                "Failed to remove label '{}' from agent with ID {}: {:?}",
                label, id, e
            );
            ApiError::internal("failed to remove agent label")
        })?;
    if deleted > 0 {
        Ok(StatusCode::NO_CONTENT)
    } else {
        Err(ApiError::not_found(
            "agent_label_not_found",
            "label not found",
        ))
    }
}

#[utoipa::path(
    get, path = "/agents/{id}/annotations", tag = "agent-annotations",
    params(("id" = Uuid, Path, description = "ID of the agent")),
    responses(
        (status = 200, description = "Successfully retrieved agent annotations", body = Vec<AgentAnnotation>),
        (status = 403, description = "Forbidden", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse),
    ),
    security(("admin_pak" = []), ("agent_pak" = []))
,
    operation_id = "agents_list_annotations"
)]
async fn list_annotations(
    State(dal): State<DAL>,
    Extension(auth_payload): Extension<AuthPayload>,
    Path(id): Path<Uuid>,
) -> Result<Json<Vec<AgentAnnotation>>, ApiError> {
    info!(
        "Handling request to list annotations for agent with ID: {}",
        id
    );
    require_admin_or_agent(&auth_payload, id)?;
    let annotations = dal.agent_annotations().list_for_agent(id).map_err(|e| {
        error!(
            "Failed to fetch annotations for agent with ID {}: {:?}",
            id, e
        );
        ApiError::internal("failed to fetch agent annotations")
    })?;
    Ok(Json(annotations))
}

#[utoipa::path(
    post, path = "/agents/{id}/annotations", tag = "agent-annotations",
    params(("id" = Uuid, Path, description = "ID of the agent")),
    request_body = NewAgentAnnotation,
    responses(
        (status = 201, description = "Successfully added agent annotation", body = AgentAnnotation),
        (status = 403, description = "Forbidden", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse),
    ),
    security(("admin_pak" = []))
,
    operation_id = "agents_add_annotation"
)]
async fn add_annotation(
    State(dal): State<DAL>,
    Extension(auth_payload): Extension<AuthPayload>,
    Path(id): Path<Uuid>,
    Json(new_annotation): Json<NewAgentAnnotation>,
) -> Result<(StatusCode, Json<AgentAnnotation>), ApiError> {
    info!(
        "Handling request to add annotation for agent with ID: {}",
        id
    );
    require_admin(&auth_payload)?;
    if new_annotation.agent_id != id {
        return Err(ApiError::bad_request(
            "agent_id_mismatch",
            "body agent_id must match the path agent id",
        ));
    }
    let annotation = dal
        .agent_annotations()
        .create(&new_annotation)
        .map_err(|e| {
            ApiError::from_diesel(e, format!("failed to add annotation for agent {id}"))
        })?;
    Ok((StatusCode::CREATED, Json(annotation)))
}

#[utoipa::path(
    delete, path = "/agents/{id}/annotations/{key}", tag = "agent-annotations",
    params(
        ("id" = Uuid, Path, description = "ID of the agent"),
        ("key" = String, Path, description = "Annotation key to remove"),
    ),
    responses(
        (status = 204, description = "Successfully removed agent annotation"),
        (status = 403, description = "Forbidden", body = ErrorResponse),
        (status = 404, description = "Annotation not found", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse),
    ),
    security(("admin_pak" = []))
,
    operation_id = "agents_remove_annotation"
)]
async fn remove_annotation(
    State(dal): State<DAL>,
    Extension(auth_payload): Extension<AuthPayload>,
    Path((id, key)): Path<(Uuid, String)>,
) -> Result<StatusCode, ApiError> {
    info!(
        "Handling request to remove annotation '{}' from agent with ID: {}",
        key, id
    );
    require_admin(&auth_payload)?;
    let deleted = dal
        .agent_annotations()
        .delete_by_agent_and_key(id, &key)
        .map_err(|e| {
            error!(
                "Failed to remove annotation '{}' from agent with ID {}: {:?}",
                key, id, e
            );
            ApiError::internal("failed to remove agent annotation")
        })?;
    if deleted > 0 {
        Ok(StatusCode::NO_CONTENT)
    } else {
        Err(ApiError::not_found(
            "agent_annotation_not_found",
            "annotation not found",
        ))
    }
}

#[utoipa::path(
    get, path = "/agents/{id}/targets", tag = "agent-targets",
    params(("id" = Uuid, Path, description = "ID of the agent")),
    responses(
        (status = 200, description = "Successfully retrieved agent targets", body = Vec<AgentTarget>),
        (status = 403, description = "Forbidden", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse),
    ),
    security(("admin_pak" = []), ("agent_pak" = []))
)]
async fn list_targets(
    State(dal): State<DAL>,
    Extension(auth_payload): Extension<AuthPayload>,
    Path(id): Path<Uuid>,
) -> Result<Json<Vec<AgentTarget>>, ApiError> {
    info!("Handling request to list targets for agent with ID: {}", id);
    require_admin_or_agent(&auth_payload, id)?;
    let targets = dal.agent_targets().list_for_agent(id).map_err(|e| {
        error!("Failed to fetch targets for agent with ID {}: {:?}", id, e);
        ApiError::internal("failed to fetch agent targets")
    })?;
    Ok(Json(targets))
}

#[utoipa::path(
    post, path = "/agents/{id}/targets", tag = "agent-targets",
    params(("id" = Uuid, Path, description = "ID of the agent")),
    request_body = NewAgentTarget,
    responses(
        (status = 201, description = "Successfully added agent target", body = AgentTarget),
        (status = 403, description = "Forbidden", body = ErrorResponse),
        (status = 404, description = "Stack not found", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse),
    ),
    security(("admin_pak" = []), ("generator_pak" = []))
)]
async fn add_target(
    State(dal): State<DAL>,
    Extension(auth_payload): Extension<AuthPayload>,
    Extension(ws_registry): Extension<Arc<ConnectionRegistry>>,
    Path(id): Path<Uuid>,
    Json(new_target): Json<NewAgentTarget>,
) -> Result<(StatusCode, Json<AgentTarget>), ApiError> {
    info!("Handling request to add target for agent with ID: {}", id);
    authorize_target_mutation(&dal, &auth_payload, id, new_target.stack_id)?;
    if new_target.agent_id != id {
        return Err(ApiError::bad_request(
            "agent_id_mismatch",
            "body agent_id must match the path agent id",
        ));
    }
    let target = dal
        .agent_targets()
        .create(&new_target)
        .map_err(|e| ApiError::from_diesel(e, format!("failed to add target for agent {id}")))?;
    // Post-commit: tell the affected agent its targets changed so it can
    // start reconciling the new stack immediately. Remove is intentionally
    // not pushed in v1 — REST polling surfaces deletions on the next tick.
    push_target_changed(&ws_registry, &target);
    Ok((StatusCode::CREATED, Json(target)))
}

/// Authorize a target create/delete operation.
///
/// Allowed when:
/// - caller is admin OR a generator PAK that owns the stack, AND
/// - the agent is registered with the stack's generator.
fn authorize_target_mutation(
    dal: &DAL,
    auth: &AuthPayload,
    agent_id: Uuid,
    stack_id: Uuid,
) -> Result<(), ApiError> {
    let mut stacks = dal.stacks().get(vec![stack_id]).map_err(|e| {
        error!("Failed to fetch stack {} for target auth: {:?}", stack_id, e);
        ApiError::internal("failed to fetch stack")
    })?;
    let stack = stacks
        .pop()
        .ok_or_else(|| ApiError::not_found("stack_not_found", "stack not found"))?;

    // Auth: admin passes; generator must own the stack.
    if !auth.admin {
        if let Some(generator_id) = auth.generator {
            if stack.generator_id != generator_id {
                return Err(ApiError::forbidden(
                    "target_generator_mismatch",
                    "generator can only target its own stacks",
                ));
            }
        } else {
            return Err(ApiError::forbidden(
                "target_create_denied",
                "admin or owning generator required",
            ));
        }
    }

    // Registration gate: agent must be registered with the stack's generator.
    let registered = dal
        .agent_generator_registrations()
        .is_registered(agent_id, stack.generator_id)
        .map_err(|e| ApiError::from_diesel(e, "failed to check generator registration"))?;
    if !registered {
        return Err(ApiError::forbidden(
            "agent_not_registered",
            "agent must be registered with this generator before stacks can be targeted at it",
        ));
    }

    Ok(())
}

#[utoipa::path(
    delete, path = "/agents/{id}/targets/{stack_id}", tag = "agent-targets",
    params(
        ("id" = Uuid, Path, description = "ID of the agent"),
        ("stack_id" = Uuid, Path, description = "ID of the stack"),
    ),
    responses(
        (status = 204, description = "Successfully removed agent target"),
        (status = 403, description = "Forbidden", body = ErrorResponse),
        (status = 404, description = "Target not found", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse),
    ),
    security(("admin_pak" = []), ("generator_pak" = []))
)]
async fn remove_target(
    State(dal): State<DAL>,
    Extension(auth_payload): Extension<AuthPayload>,
    Path((id, stack_id)): Path<(Uuid, Uuid)>,
) -> Result<StatusCode, ApiError> {
    info!(
        "Handling request to remove target for stack {} from agent with ID: {}",
        stack_id, id
    );
    authorize_target_mutation(&dal, &auth_payload, id, stack_id)?;
    let deleted = dal
        .agent_targets()
        .delete_by_agent_and_stack(id, stack_id)
        .map_err(|e| {
            error!(
                "Failed to remove target for stack {} from agent with ID {}: {:?}",
                stack_id, id, e
            );
            ApiError::internal("failed to remove agent target")
        })?;
    if deleted > 0 {
        Ok(StatusCode::NO_CONTENT)
    } else {
        Err(ApiError::not_found(
            "agent_target_not_found",
            "target not found",
        ))
    }
}

/// Optional heartbeat report body (BROKKR-T-0227).
///
/// A plain heartbeat carries no body; agents that probe their own Kubernetes
/// API attach this to self-report reachability. Both fields are optional so a
/// body may carry only what the agent could measure, and the entire body may
/// be omitted (legacy/no-body heartbeats still work).
#[derive(Debug, Deserialize, Default, ToSchema)]
pub struct HeartbeatReport {
    /// Whether the agent can reach its own Kubernetes API.
    pub k8s_reachable: Option<bool>,
    /// Measured latency (milliseconds) of the reachability probe, if any.
    pub k8s_api_latency_ms: Option<i32>,
}

#[utoipa::path(
    post, path = "/agents/{id}/heartbeat", tag = "agents",
    params(("id" = Uuid, Path, description = "ID of the agent")),
    request_body(content = HeartbeatReport, description = "Optional agent-reported K8s connectivity"),
    responses(
        (status = 204, description = "Successfully recorded agent heartbeat"),
        (status = 403, description = "Forbidden", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse),
    ),
    security(("agent_pak" = []))
)]
async fn record_heartbeat(
    State(dal): State<DAL>,
    Extension(auth_payload): Extension<AuthPayload>,
    Extension(ws_registry): Extension<Arc<ConnectionRegistry>>,
    Extension(fleet_broadcaster): Extension<Arc<crate::ws::FleetBroadcaster>>,
    Path(id): Path<Uuid>,
    report: Option<Json<HeartbeatReport>>,
) -> Result<StatusCode, ApiError> {
    info!(
        "Handling request to record heartbeat for agent with ID: {}",
        id
    );
    if auth_payload.agent != Some(id) {
        return Err(ApiError::forbidden(
            "agent_pak_mismatch",
            "agent PAK does not match the agent ID",
        ));
    }
    dal.agents().record_heartbeat(id).map_err(|e| {
        error!(
            "Failed to record heartbeat for agent with ID {}: {:?}",
            id, e
        );
        ApiError::internal("failed to record agent heartbeat")
    })?;

    // BROKKR-T-0227: persist agent-reported K8s connectivity when present.
    // Absent body / field leaves the columns untouched (NULL for agents that
    // never report).
    if let Some(Json(report)) = report
        && let Some(reachable) = report.k8s_reachable
    {
        dal.agents()
            .record_k8s_connectivity(id, reachable, report.k8s_api_latency_ms)
            .map_err(|e| {
                error!(
                    "Failed to record K8s connectivity for agent with ID {}: {:?}",
                    id, e
                );
                ApiError::internal("failed to record agent K8s connectivity")
            })?;
    }

    if let Ok(Some(agent)) = dal.agents().get(id) {
        metrics::set_agent_heartbeat_age(&id.to_string(), &agent.name, 0.0);
    }

    // BROKKR-I-0028: event-driven fleet live-push on heartbeat. Best-effort,
    // after the heartbeat + T-0227 k8s fields are persisted, so the pushed
    // record reflects the just-recorded values. A push failure must never
    // affect the heartbeat response.
    crate::api::v1::fleet::broadcast_agent_fleet_update(
        &dal,
        &ws_registry,
        &fleet_broadcaster,
        id,
    );

    Ok(StatusCode::NO_CONTENT)
}

#[derive(Deserialize, Default)]
struct TargetStateParams {
    mode: Option<String>,
}

#[utoipa::path(
    get, path = "/agents/{id}/target-state", tag = "agents",
    params(
        ("id" = Uuid, Path, description = "ID of the agent"),
        ("mode" = Option<String>, Query, description = "Mode of operation: 'incremental' (default) or 'full'")
    ),
    responses(
        (status = 200, description = "Successfully retrieved target state", body = Vec<DeploymentObject>),
        (status = 403, description = "Forbidden", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse),
    ),
    security(("admin_pak" = []), ("agent_pak" = []))
)]
async fn get_target_state(
    State(dal): State<DAL>,
    Extension(auth_payload): Extension<AuthPayload>,
    Path(id): Path<Uuid>,
    Query(params): Query<TargetStateParams>,
) -> Result<Json<Vec<DeploymentObject>>, ApiError> {
    info!(
        "Handling request to get target state for agent with ID: {}",
        id
    );
    require_admin_or_agent(&auth_payload, id)?;
    let include_deployed = params.mode.as_deref() == Some("full");
    info!(
        "Target state request mode is '{}', include_deployed={}",
        params.mode.unwrap_or_else(|| "incremental".to_string()),
        include_deployed
    );
    let objects = dal
        .deployment_objects()
        .get_target_state_for_agent(id, include_deployed)
        .map_err(|e| {
            error!(
                "Failed to fetch target state for agent with ID {}: {:?}",
                id, e
            );
            ApiError::internal("failed to fetch target state")
        })?;
    info!(
        "Successfully retrieved {} objects in target state for agent with ID: {}",
        objects.len(),
        id
    );
    Ok(Json(objects))
}

#[utoipa::path(
    get, path = "/agents/{id}/stacks", tag = "agents",
    params(("id" = Uuid, Path, description = "ID of the agent")),
    responses(
        (status = 200, description = "Successfully retrieved associated stacks", body = Vec<Stack>),
        (status = 403, description = "Forbidden", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse),
    ),
    security(("admin_pak" = []), ("agent_pak" = []))
)]
async fn get_associated_stacks(
    State(dal): State<DAL>,
    Extension(auth_payload): Extension<AuthPayload>,
    Path(id): Path<Uuid>,
) -> Result<Json<Vec<Stack>>, ApiError> {
    info!(
        "Handling request to get associated stacks for agent with ID: {}",
        id
    );
    require_admin_or_agent(&auth_payload, id)?;
    let stacks = dal.stacks().get_associated_stacks(id).map_err(|e| {
        error!(
            "Failed to fetch associated stacks for agent with ID {}: {:?}",
            id, e
        );
        ApiError::internal("failed to fetch associated stacks")
    })?;
    Ok(Json(stacks))
}

#[utoipa::path(
    post, path = "/agents/{id}/rotate-pak", tag = "agents",
    params(("id" = Uuid, Path, description = "Agent id")),
    responses(
        (status = 200, description = "Successfully rotated agent PAK", body = serde_json::Value),
        (status = 403, description = "Forbidden", body = ErrorResponse),
        (status = 404, description = "Agent not found", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse)
    ),
    security(("admin_pak" = []), ("agent_pak" = []))
)]
async fn rotate_agent_pak(
    State(dal): State<DAL>,
    Extension(auth_payload): Extension<AuthPayload>,
    Extension(ws_registry): Extension<Arc<ConnectionRegistry>>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, ApiError> {
    info!("Handling request to rotate PAK for agent with ID: {}", id);
    require_admin_or_agent(&auth_payload, id)?;

    let old_pak_hash = dal
        .agents()
        .get(id)
        .map_err(|e| {
            error!("Failed to fetch agent with ID {}: {:?}", id, e);
            ApiError::internal("failed to fetch agent")
        })?
        .ok_or_else(|| ApiError::not_found("agent_not_found", "agent not found"))?
        .pak_hash;

    let (pak_value, pak_hash) = pak::create_pak().map_err(|e| {
        error!("Failed to create new PAK: {:?}", e);
        ApiError::internal("failed to create new PAK")
    })?;

    let updated_agent = dal.agents().update_pak_hash(id, pak_hash).map_err(|e| {
        error!("Failed to update agent PAK hash: {:?}", e);
        ApiError::internal("failed to update agent PAK hash")
    })?;
    info!("Successfully rotated PAK for agent with ID: {}", id);
    dal.invalidate_auth_cache(&old_pak_hash);
    // The old PAK is now invalid; tear down any open WS so it can't keep
    // streaming on already-upgraded credentials (BROKKR-T-0176). Done after
    // the DB commit + cache invalidation so we never hold those while doing
    // socket teardown.
    let closed = ws_registry.close_for_agent(id);
    if closed > 0 {
        info!(%id, closed, "closed open WS connection(s) after PAK rotation");
    }

    audit::log_action(
        ACTOR_TYPE_ADMIN,
        None,
        ACTION_PAK_ROTATED,
        RESOURCE_TYPE_AGENT,
        Some(id),
        Some(serde_json::json!({
            "agent_name": updated_agent.name,
        })),
        None,
        None,
    );

    Ok(Json(serde_json::json!({
        "agent": updated_agent,
        "pak": pak_value
    })))
}

#[utoipa::path(
    get, path = "/agents/{id}/registrations", tag = "agents",
    params(("id" = Uuid, Path, description = "Agent ID")),
    responses(
        (status = 200, description = "Generator registrations for agent", body = Vec<AgentGeneratorRegistration>),
        (status = 403, description = "Forbidden", body = ErrorResponse),
        (status = 404, description = "Agent not found", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse),
    ),
    security(("admin_pak" = []), ("agent_pak" = []))
)]
async fn list_agent_registrations(
    State(dal): State<DAL>,
    Extension(auth_payload): Extension<AuthPayload>,
    Path(id): Path<Uuid>,
) -> Result<Json<Vec<AgentGeneratorRegistration>>, ApiError> {
    require_admin_or_agent(&auth_payload, id)?;
    dal.agents()
        .get(id)
        .map_err(|e| ApiError::from_diesel(e, "failed to look up agent"))?
        .ok_or_else(|| ApiError::not_found("agent_not_found", "agent not found"))?;
    let registrations = dal
        .agent_generator_registrations()
        .list_for_agent(id)
        .map_err(|e| ApiError::from_diesel(e, "failed to list registrations"))?;
    Ok(Json(registrations))
}
