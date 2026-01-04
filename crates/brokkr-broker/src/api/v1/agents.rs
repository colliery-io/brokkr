/*
 * Copyright (c) 2025 Dylan Storey
 * Licensed under the Elastic License 2.0.
 * See LICENSE file in the project root for full license text.
 */

//! Agent management API endpoints.
//!
//! This module provides routes and handlers for managing agents, including CRUD operations,
//! event logging, label management, annotation management, target management, and heartbeat recording.

use crate::api::v1::middleware::AuthPayload;
use crate::dal::DAL;
use crate::metrics;
use crate::utils::{audit, event_bus, pak};
use brokkr_models::models::webhooks::{
    BrokkrEvent, EVENT_DEPLOYMENT_APPLIED, EVENT_DEPLOYMENT_FAILED,
};
use axum::http::StatusCode;
use brokkr_models::models::audit_logs::{
    ACTION_AGENT_CREATED, ACTION_AGENT_DELETED, ACTION_AGENT_UPDATED, ACTION_PAK_ROTATED,
    ACTOR_TYPE_ADMIN, RESOURCE_TYPE_AGENT,
};
use axum::{
    extract::{Extension, Path, Query, State},
    routing::{delete, get, post},
    Json, Router,
};
use brokkr_models::models::agent_annotations::{AgentAnnotation, NewAgentAnnotation};
use brokkr_models::models::agent_events::{AgentEvent, NewAgentEvent};
use brokkr_models::models::agent_labels::{AgentLabel, NewAgentLabel};
use brokkr_models::models::agent_targets::{AgentTarget, NewAgentTarget};
use brokkr_models::models::agents::{Agent, NewAgent};
use brokkr_models::models::deployment_objects::DeploymentObject;
use brokkr_models::models::stacks::Stack;
use tracing::{debug, error, info, warn};
use serde::Deserialize;
use serde_json::Value;
use uuid::Uuid;

/// Creates and returns the router for agent-related endpoints.
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
        .route("/agents/:id/target-state", get(get_target_state))
        .route("/agents/:id/stacks", get(get_associated_stacks))
        .route("/agents/:id/rotate-pak", post(rotate_agent_pak))
}

/// Lists all agents.
///
/// # Authorization
/// Requires admin privileges.
#[utoipa::path(
    get,
    path = "/agents",
    tag = "agents",
    responses(
        (status = 200, description = "Successfully retrieved agents", body = Vec<Agent>),
        (status = 403, description = "Forbidden - PAK does not have required rights", body = serde_json::Value),
        (status = 500, description = "Internal server error", body = serde_json::Value),
    ),
    security(
        ("admin_pak" = []),
    )
)]
async fn list_agents(
    State(dal): State<DAL>,
    Extension(auth_payload): Extension<AuthPayload>,
) -> Result<Json<Vec<Agent>>, (StatusCode, Json<serde_json::Value>)> {
    info!("Handling request to list agents");
    if !auth_payload.admin {
        warn!("Unauthorized attempt to list agents");
        return Err((
            StatusCode::FORBIDDEN,
            Json(serde_json::json!({"error": "Unauthorized"})),
        ));
    }

    match dal.agents().list() {
        Ok(agents) => {
            info!("Successfully retrieved {} agents", agents.len());
            // Update active agents metric
            let active_count = agents.iter().filter(|a| a.status == "ACTIVE").count();
            metrics::set_active_agents(active_count as i64);

            // Update heartbeat age metrics for all agents
            let now = chrono::Utc::now();
            for agent in &agents {
                if let Some(last_hb) = agent.last_heartbeat {
                    let age_seconds = (now - last_hb).num_seconds().max(0) as f64;
                    metrics::set_agent_heartbeat_age(&agent.id.to_string(), &agent.name, age_seconds);
                }
            }

            Ok(Json(agents))
        }
        Err(e) => {
            error!("Failed to fetch agents: {:?}", e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": "Failed to fetch agents"})),
            ))
        }
    }
}

/// Creates a new agent.
///
/// # Authorization
/// Requires admin privileges.
#[utoipa::path(
    post,
    path = "/agents",
    tag = "agents",
    request_body = NewAgent,
    responses(
        (status = 200, description = "Successfully created agent", body = serde_json::Value),
        (status = 403, description = "Forbidden - PAK does not have required rights", body = serde_json::Value),
        (status = 500, description = "Internal server error", body = serde_json::Value),
    ),
    security(
        ("admin_pak" = []),
    )
)]
async fn create_agent(
    State(dal): State<DAL>,
    Extension(auth_payload): Extension<AuthPayload>,
    Json(new_agent): Json<NewAgent>,
) -> Result<Json<Value>, (StatusCode, Json<serde_json::Value>)> {
    info!("Handling request to create a new agent");
    if !auth_payload.admin {
        warn!("Unauthorized attempt to create an agent");
        return Err((
            StatusCode::FORBIDDEN,
            Json(serde_json::json!({"error": "Unauthorized"})),
        ));
    }

    match dal.agents().create(&new_agent) {
        Ok(agent) => {
            info!("Successfully created agent with ID: {}", agent.id);
            // Generate initial PAK and set PAK hash
            let (pak, pak_hash) = pak::create_pak().map_err(|e| {
                error!("Failed to create PAK: {:?}", e);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(serde_json::json!({"error": "Failed to create PAK"})),
                )
            })?;

            match dal.agents().update_pak_hash(agent.id, pak_hash) {
                Ok(updated_agent) => {
                    info!("Successfully updated PAK hash for agent ID: {}", agent.id);

                    // Log audit entry for agent creation
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

                    let response = serde_json::json!({
                        "agent": updated_agent,
                        "initial_pak": pak
                    });
                    Ok(Json(response))
                }
                Err(e) => {
                    error!("Failed to update agent PAK hash: {:?}", e);
                    Err((
                        StatusCode::INTERNAL_SERVER_ERROR,
                        Json(serde_json::json!({"error": "Failed to update agent PAK hash"})),
                    ))
                }
            }
        }
        Err(e) => {
            error!("Failed to create agent: {:?}", e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": "Failed to create agent"})),
            ))
        }
    }
}

#[derive(Deserialize)]
struct AgentQuery {
    name: Option<String>,
    cluster_name: Option<String>,
}

/// Retrieves a specific agent by ID.
///
/// # Authorization
/// Requires admin privileges or matching agent ID.
#[utoipa::path(
    get,
    path = "/agents/{id}",
    tag = "agents",
    params(
        ("id" = Uuid, Path, description = "ID of the agent to retrieve"),
    ),
    responses(
        (status = 200, description = "Successfully retrieved agent", body = Agent),
        (status = 403, description = "Forbidden - PAK does not have required rights", body = serde_json::Value),
        (status = 404, description = "Agent not found", body = serde_json::Value),
        (status = 500, description = "Internal server error", body = serde_json::Value),
    ),
    security(
        ("admin_pak" = []),
        ("agent_pak" = []),
    )
)]
async fn get_agent(
    State(dal): State<DAL>,
    Extension(auth_payload): Extension<AuthPayload>,
    Path(id): Path<Uuid>,
) -> Result<Json<Agent>, (StatusCode, Json<serde_json::Value>)> {
    info!("Handling request to get agent by ID: {}", id);
    if !auth_payload.admin && auth_payload.agent != Some(id) {
        warn!("Unauthorized attempt to get agent with ID: {}", id);
        return Err((
            StatusCode::FORBIDDEN,
            Json(serde_json::json!({"error": "Unauthorized"})),
        ));
    }

    match dal.agents().get(id) {
        Ok(Some(agent)) => {
            info!("Successfully retrieved agent with ID: {}", id);
            Ok(Json(agent))
        }
        Ok(None) => {
            warn!("Agent not found with ID: {}", id);
            Err((
                StatusCode::NOT_FOUND,
                Json(serde_json::json!({"error": "Agent not found"})),
            ))
        }
        Err(e) => {
            error!("Failed to fetch agent with ID {}: {:?}", id, e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": "Failed to fetch agent"})),
            ))
        }
    }
}

/// Searches for an agent by name and cluster name.
///
/// # Authorization
/// Requires admin privileges.
#[utoipa::path(
    get,
    path = "/agents/",
    tag = "agents",
    params(
        ("name" = Option<String>, Query, description = "Name of the agent to search for"),
        ("cluster_name" = Option<String>, Query, description = "Name of the cluster to search in"),
    ),
    responses(
        (status = 200, description = "Successfully found agent", body = Agent),
        (status = 400, description = "Invalid request - missing name or cluster_name", body = serde_json::Value),
        (status = 403, description = "Forbidden - PAK does not have required rights", body = serde_json::Value),
        (status = 404, description = "Agent not found", body = serde_json::Value),
        (status = 500, description = "Internal server error", body = serde_json::Value),
    ),
    security(
        ("admin_pak" = []),
    )
)]
async fn search_agent(
    State(dal): State<DAL>,
    Extension(auth_payload): Extension<AuthPayload>,
    Query(query): Query<AgentQuery>,
) -> Result<Json<Agent>, (StatusCode, Json<serde_json::Value>)> {
    info!("Handling request to search for agent");
    if let (Some(name), Some(cluster_name)) = (query.name.clone(), query.cluster_name.clone()) {
        info!(
            "Searching for agent with name: {} and cluster_name: {}",
            name, cluster_name
        );
        match dal
            .agents()
            .get_by_name_and_cluster_name(name, cluster_name)
        {
            Ok(Some(agent)) => {
                if auth_payload.admin || auth_payload.agent == Some(agent.id) {
                    info!("Successfully found agent with ID: {}", agent.id);
                    Ok(Json(agent))
                } else {
                    warn!("Unauthorized attempt to access agent with ID: {}", agent.id);
                    Err((
                        StatusCode::FORBIDDEN,
                        Json(serde_json::json!({"error": "Unauthorized"})),
                    ))
                }
            }
            Ok(None) => {
                warn!("Agent not found with provided name and cluster_name");
                Err((
                    StatusCode::NOT_FOUND,
                    Json(serde_json::json!({"error": "Agent not found"})),
                ))
            }
            Err(e) => {
                error!("Failed to fetch agent: {:?}", e);
                Err((
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(serde_json::json!({"error": "Failed to fetch agent"})),
                ))
            }
        }
    } else {
        warn!("Invalid request: missing name or cluster_name");
        Err((
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({"error": "Invalid request"})),
        ))
    }
}

/// Updates an existing agent.
///
/// # Authorization
/// Requires admin privileges or matching agent ID.
#[utoipa::path(
    put,
    path = "/agents/{id}",
    tag = "agents",
    params(
        ("id" = Uuid, Path, description = "ID of the agent to update"),
    ),
    request_body = serde_json::Value,
    responses(
        (status = 200, description = "Successfully updated agent", body = Agent),
        (status = 403, description = "Forbidden - PAK does not have required rights", body = serde_json::Value),
        (status = 404, description = "Agent not found", body = serde_json::Value),
        (status = 500, description = "Internal server error", body = serde_json::Value),
    ),
    security(
        ("admin_pak" = []),
        ("agent_pak" = []),
    )
)]
async fn update_agent(
    State(dal): State<DAL>,
    Extension(auth_payload): Extension<AuthPayload>,
    Path(id): Path<Uuid>,
    Json(update_payload): Json<serde_json::Value>,
) -> Result<Json<Agent>, (StatusCode, Json<serde_json::Value>)> {
    info!("Handling request to update agent with ID: {}", id);
    if !auth_payload.admin && auth_payload.agent != Some(id) {
        warn!("Unauthorized attempt to update agent with ID: {}", id);
        return Err((
            StatusCode::FORBIDDEN,
            Json(serde_json::json!({"error": "Unauthorized"})),
        ));
    }

    let mut agent = match dal.agents().get(id) {
        Ok(Some(a)) => a,
        Ok(None) => {
            warn!("Agent not found with ID: {}", id);
            return Err((
                StatusCode::NOT_FOUND,
                Json(serde_json::json!({"error": "Agent not found"})),
            ));
        }
        Err(e) => {
            error!("Failed to fetch agent with ID {}: {:?}", id, e);
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": "Failed to fetch agent"})),
            ));
        }
    };

    if let Some(name) = update_payload.get("name").and_then(|v| v.as_str()) {
        agent.name = name.to_string();
    }
    if let Some(cluster_name) = update_payload.get("cluster_name").and_then(|v| v.as_str()) {
        agent.cluster_name = cluster_name.to_string();
    }
    if let Some(status) = update_payload.get("status").and_then(|v| v.as_str()) {
        agent.status = status.to_string();
    }

    match dal.agents().update(id, &agent) {
        Ok(updated_agent) => {
            info!("Successfully updated agent with ID: {}", id);

            // Log audit entry for agent update
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
        Err(e) => {
            error!("Failed to update agent with ID {}: {:?}", id, e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": "Failed to update agent"})),
            ))
        }
    }
}

/// Soft deletes an agent.
///
/// # Authorization
/// Requires admin privileges.
#[utoipa::path(
    delete,
    path = "/agents/{id}",
    tag = "agents",
    params(
        ("id" = Uuid, Path, description = "ID of the agent to delete"),
    ),
    responses(
        (status = 204, description = "Successfully deleted agent"),
        (status = 403, description = "Forbidden - PAK does not have required rights", body = serde_json::Value),
        (status = 500, description = "Internal server error", body = serde_json::Value),
    ),
    security(
        ("admin_pak" = []),
    )
)]
async fn delete_agent(
    State(dal): State<DAL>,
    Extension(auth_payload): Extension<AuthPayload>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, (StatusCode, Json<serde_json::Value>)> {
    info!("Handling request to delete agent with ID: {}", id);
    if !auth_payload.admin {
        warn!("Unauthorized attempt to delete agent with ID: {}", id);
        return Err((
            StatusCode::FORBIDDEN,
            Json(serde_json::json!({"error": "Unauthorized"})),
        ));
    }

    match dal.agents().soft_delete(id) {
        Ok(_) => {
            info!("Successfully deleted agent with ID: {}", id);

            // Log audit entry for agent deletion
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
        Err(e) => {
            error!("Failed to delete agent with ID {}: {:?}", id, e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": "Failed to delete agent"})),
            ))
        }
    }
}

/// Lists events for a specific agent.
///
/// # Authorization
/// Requires admin privileges or matching agent ID.
#[utoipa::path(
    get,
    path = "/agents/{id}/events",
    tag = "agent-events",
    params(
        ("id" = Uuid, Path, description = "ID of the agent to list events for"),
    ),
    responses(
        (status = 200, description = "Successfully retrieved agent events", body = Vec<AgentEvent>),
        (status = 403, description = "Forbidden - PAK does not have required rights", body = serde_json::Value),
        (status = 500, description = "Internal server error", body = serde_json::Value),
    ),
    security(
        ("admin_pak" = []),
        ("agent_pak" = []),
    )
)]
async fn list_events(
    State(dal): State<DAL>,
    Extension(auth_payload): Extension<AuthPayload>,
    Path(id): Path<Uuid>,
) -> Result<Json<Vec<Value>>, (StatusCode, Json<serde_json::Value>)> {
    info!("Handling request to list events for agent with ID: {}", id);
    if !auth_payload.admin && auth_payload.agent != Some(id) {
        warn!(
            "Unauthorized attempt to list events for agent with ID: {}",
            id
        );
        return Err((
            StatusCode::FORBIDDEN,
            Json(serde_json::json!({"error": "Unauthorized"})),
        ));
    }

    match dal.agent_events().get_events(None, Some(id)) {
        Ok(events) => {
            info!(
                "Successfully retrieved {} events for agent with ID: {}",
                events.len(),
                id
            );
            Ok(Json(
                events
                    .into_iter()
                    .map(|e| serde_json::to_value(e).unwrap())
                    .collect(),
            ))
        }
        Err(e) => {
            error!("Failed to fetch events for agent with ID {}: {:?}", id, e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": "Failed to fetch agent events"})),
            ))
        }
    }
}

/// Creates a new event for a specific agent.
///
/// # Authorization
/// Requires admin privileges or matching agent ID.
#[utoipa::path(
    post,
    path = "/agents/{id}/events",
    tag = "agent-events",
    params(
        ("id" = Uuid, Path, description = "ID of the agent to create an event for"),
    ),
    request_body = NewAgentEvent,
    responses(
        (status = 200, description = "Successfully created agent event", body = AgentEvent),
        (status = 403, description = "Forbidden - PAK does not have required rights", body = serde_json::Value),
        (status = 500, description = "Internal server error", body = serde_json::Value),
    ),
    security(
        ("admin_pak" = []),
        ("agent_pak" = []),
    )
)]
async fn create_event(
    State(dal): State<DAL>,
    Extension(auth_payload): Extension<AuthPayload>,
    Path(id): Path<Uuid>,
    Json(new_event): Json<NewAgentEvent>,
) -> Result<Json<AgentEvent>, (StatusCode, Json<serde_json::Value>)> {
    info!("Handling request to create event for agent with ID: {}", id);
    if !auth_payload.admin && auth_payload.agent != Some(id) {
        warn!(
            "Unauthorized attempt to create event for agent with ID: {}",
            id
        );
        return Err((
            StatusCode::FORBIDDEN,
            Json(serde_json::json!({"error": "Unauthorized"})),
        ));
    }

    match dal.agent_events().create(&new_event) {
        Ok(event) => {
            info!("Successfully created event for agent with ID: {}", id);

            // Emit deployment webhook event based on status
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

            Ok(Json(event))
        }
        Err(e) => {
            error!("Failed to create event for agent with ID {}: {:?}", id, e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": "Failed to create agent event"})),
            ))
        }
    }
}

/// Lists labels for a specific agent.
///
/// # Authorization
/// Requires admin privileges or matching agent ID.
#[utoipa::path(
    get,
    path = "/agents/{id}/labels",
    tag = "agent-labels",
    params(
        ("id" = Uuid, Path, description = "ID of the agent to list labels for"),
    ),
    responses(
        (status = 200, description = "Successfully retrieved agent labels", body = Vec<AgentLabel>),
        (status = 403, description = "Forbidden - PAK does not have required rights", body = serde_json::Value),
        (status = 500, description = "Internal server error", body = serde_json::Value),
    ),
    security(
        ("admin_pak" = []),
        ("agent_pak" = []),
    )
)]
async fn list_labels(
    State(dal): State<DAL>,
    Extension(auth_payload): Extension<AuthPayload>,
    Path(id): Path<Uuid>,
) -> Result<Json<Vec<AgentLabel>>, (StatusCode, Json<serde_json::Value>)> {
    info!("Handling request to list labels for agent with ID: {}", id);
    if !auth_payload.admin && auth_payload.agent != Some(id) {
        warn!(
            "Unauthorized attempt to list labels for agent with ID: {}",
            id
        );
        return Err((
            StatusCode::FORBIDDEN,
            Json(serde_json::json!({"error": "Unauthorized"})),
        ));
    }

    match dal.agent_labels().list_for_agent(id) {
        Ok(labels) => {
            info!(
                "Successfully retrieved {} labels for agent with ID: {}",
                labels.len(),
                id
            );
            Ok(Json(labels))
        }
        Err(e) => {
            error!("Failed to fetch labels for agent with ID {}: {:?}", id, e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": "Failed to fetch agent labels"})),
            ))
        }
    }
}

/// Adds a new label to a specific agent.
///
/// # Authorization
/// Requires admin privileges.
#[utoipa::path(
    post,
    path = "/agents/{id}/labels",
    tag = "agent-labels",
    params(
        ("id" = Uuid, Path, description = "ID of the agent to add the label to"),
    ),
    request_body = NewAgentLabel,
    responses(
        (status = 200, description = "Successfully added agent label", body = AgentLabel),
        (status = 403, description = "Forbidden - PAK does not have required rights", body = serde_json::Value),
        (status = 500, description = "Internal server error", body = serde_json::Value),
    ),
    security(
        ("admin_pak" = []),
    )
)]
async fn add_label(
    State(dal): State<DAL>,
    Extension(auth_payload): Extension<AuthPayload>,
    Path(id): Path<Uuid>,
    Json(new_label): Json<NewAgentLabel>,
) -> Result<Json<AgentLabel>, (StatusCode, Json<serde_json::Value>)> {
    info!("Handling request to add label for agent with ID: {}", id);
    if !auth_payload.admin {
        warn!(
            "Unauthorized attempt to add label for agent with ID: {}",
            id
        );
        return Err((
            StatusCode::FORBIDDEN,
            Json(serde_json::json!({"error": "Unauthorized"})),
        ));
    }

    match dal.agent_labels().create(&new_label) {
        Ok(label) => {
            info!("Successfully added label for agent with ID: {}", id);
            Ok(Json(label))
        }
        Err(e) => {
            error!("Failed to add label for agent with ID {}: {:?}", id, e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": "Failed to add agent label"})),
            ))
        }
    }
}

/// Removes a label from a specific agent.
///
/// # Authorization
/// Requires admin privileges.
#[utoipa::path(
    delete,
    path = "/agents/{id}/labels/{label}",
    tag = "agent-labels",
    params(
        ("id" = Uuid, Path, description = "ID of the agent to remove the label from"),
        ("label" = String, Path, description = "The label to remove"),
    ),
    responses(
        (status = 204, description = "Successfully removed agent label"),
        (status = 403, description = "Forbidden - PAK does not have required rights", body = serde_json::Value),
        (status = 404, description = "Label not found", body = serde_json::Value),
        (status = 500, description = "Internal server error", body = serde_json::Value),
    ),
    security(
        ("admin_pak" = []),
    )
)]
async fn remove_label(
    State(dal): State<DAL>,
    Extension(auth_payload): Extension<AuthPayload>,
    Path((id, label)): Path<(Uuid, String)>,
) -> Result<StatusCode, (StatusCode, Json<serde_json::Value>)> {
    info!(
        "Handling request to remove label '{}' from agent with ID: {}",
        label, id
    );
    if !auth_payload.admin {
        warn!(
            "Unauthorized attempt to remove label from agent with ID: {}",
            id
        );
        return Err((
            StatusCode::FORBIDDEN,
            Json(serde_json::json!({"error": "Unauthorized"})),
        ));
    }

    match dal.agent_labels().delete_by_agent_and_label(id, &label) {
        Ok(deleted_count) => {
            if deleted_count > 0 {
                info!(
                    "Successfully removed label '{}' from agent with ID: {}",
                    label, id
                );
                Ok(StatusCode::NO_CONTENT)
            } else {
                warn!("Label '{}' not found for agent with ID: {}", label, id);
                Err((
                    StatusCode::NOT_FOUND,
                    Json(serde_json::json!({"error": "Label not found"})),
                ))
            }
        }
        Err(e) => {
            error!(
                "Failed to remove label '{}' from agent with ID {}: {:?}",
                label, id, e
            );
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": "Failed to remove agent label"})),
            ))
        }
    }
}

/// Lists annotations for a specific agent.
///
/// # Authorization
/// Requires admin privileges or matching agent ID.
#[utoipa::path(
    get,
    path = "/agents/{id}/annotations",
    tag = "agent-annotations",
    params(
        ("id" = Uuid, Path, description = "ID of the agent to list annotations for"),
    ),
    responses(
        (status = 200, description = "Successfully retrieved agent annotations", body = Vec<AgentAnnotation>),
        (status = 403, description = "Forbidden - PAK does not have required rights", body = serde_json::Value),
        (status = 500, description = "Internal server error", body = serde_json::Value),
    ),
    security(
        ("admin_pak" = []),
        ("agent_pak" = []),
    )
)]
async fn list_annotations(
    State(dal): State<DAL>,
    Extension(auth_payload): Extension<AuthPayload>,
    Path(id): Path<Uuid>,
) -> Result<Json<Vec<AgentAnnotation>>, (StatusCode, Json<serde_json::Value>)> {
    info!(
        "Handling request to list annotations for agent with ID: {}",
        id
    );
    if !auth_payload.admin && auth_payload.agent != Some(id) {
        warn!(
            "Unauthorized attempt to list annotations for agent with ID: {}",
            id
        );
        return Err((
            StatusCode::FORBIDDEN,
            Json(serde_json::json!({"error": "Unauthorized"})),
        ));
    }

    match dal.agent_annotations().list_for_agent(id) {
        Ok(annotations) => {
            info!(
                "Successfully retrieved {} annotations for agent with ID: {}",
                annotations.len(),
                id
            );
            Ok(Json(annotations))
        }
        Err(e) => {
            error!(
                "Failed to fetch annotations for agent with ID {}: {:?}",
                id, e
            );
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": "Failed to fetch agent annotations"})),
            ))
        }
    }
}

/// Adds a new annotation to a specific agent.
///
/// # Authorization
/// Requires admin privileges.
#[utoipa::path(
    post,
    path = "/agents/{id}/annotations",
    tag = "agent-annotations",
    params(
        ("id" = Uuid, Path, description = "ID of the agent to add the annotation to"),
    ),
    request_body = NewAgentAnnotation,
    responses(
        (status = 200, description = "Successfully added agent annotation", body = AgentAnnotation),
        (status = 403, description = "Forbidden - PAK does not have required rights", body = serde_json::Value),
        (status = 500, description = "Internal server error", body = serde_json::Value),
    ),
    security(
        ("admin_pak" = []),
    )
)]
async fn add_annotation(
    State(dal): State<DAL>,
    Extension(auth_payload): Extension<AuthPayload>,
    Path(id): Path<Uuid>,
    Json(new_annotation): Json<NewAgentAnnotation>,
) -> Result<Json<AgentAnnotation>, (StatusCode, Json<serde_json::Value>)> {
    info!(
        "Handling request to add annotation for agent with ID: {}",
        id
    );
    if !auth_payload.admin {
        warn!(
            "Unauthorized attempt to add annotation for agent with ID: {}",
            id
        );
        return Err((
            StatusCode::FORBIDDEN,
            Json(serde_json::json!({"error": "Unauthorized"})),
        ));
    }

    match dal.agent_annotations().create(&new_annotation) {
        Ok(annotation) => {
            info!("Successfully added annotation for agent with ID: {}", id);
            Ok(Json(annotation))
        }
        Err(e) => {
            error!("Failed to add annotation for agent with ID {}: {:?}", id, e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": "Failed to add agent annotation"})),
            ))
        }
    }
}

/// Removes an annotation from a specific agent.
///
/// # Authorization
/// Requires admin privileges.
#[utoipa::path(
    delete,
    path = "/agents/{id}/annotations/{key}",
    tag = "agent-annotations",
    params(
        ("id" = Uuid, Path, description = "ID of the agent to remove the annotation from"),
        ("key" = String, Path, description = "The key of the annotation to remove"),
    ),
    responses(
        (status = 204, description = "Successfully removed agent annotation"),
        (status = 403, description = "Forbidden - PAK does not have required rights", body = serde_json::Value),
        (status = 404, description = "Annotation not found", body = serde_json::Value),
        (status = 500, description = "Internal server error", body = serde_json::Value),
    ),
    security(
        ("admin_pak" = []),
    )
)]
async fn remove_annotation(
    State(dal): State<DAL>,
    Extension(auth_payload): Extension<AuthPayload>,
    Path((id, key)): Path<(Uuid, String)>,
) -> Result<StatusCode, (StatusCode, Json<serde_json::Value>)> {
    info!(
        "Handling request to remove annotation '{}' from agent with ID: {}",
        key, id
    );
    if !auth_payload.admin {
        warn!(
            "Unauthorized attempt to remove annotation from agent with ID: {}",
            id
        );
        return Err((
            StatusCode::FORBIDDEN,
            Json(serde_json::json!({"error": "Unauthorized"})),
        ));
    }

    match dal.agent_annotations().delete_by_agent_and_key(id, &key) {
        Ok(deleted_count) => {
            if deleted_count > 0 {
                info!(
                    "Successfully removed annotation '{}' from agent with ID: {}",
                    key, id
                );
                Ok(StatusCode::NO_CONTENT)
            } else {
                warn!("Annotation '{}' not found for agent with ID: {}", key, id);
                Err((
                    StatusCode::NOT_FOUND,
                    Json(serde_json::json!({"error": "Annotation not found"})),
                ))
            }
        }
        Err(e) => {
            error!(
                "Failed to remove annotation '{}' from agent with ID {}: {:?}",
                key, id, e
            );
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": "Failed to remove agent annotation"})),
            ))
        }
    }
}

/// Lists targets for a specific agent.
///
/// # Authorization
/// Requires admin privileges or matching agent ID.
#[utoipa::path(
    get,
    path = "/agents/{id}/targets",
    tag = "agent-targets",
    params(
        ("id" = Uuid, Path, description = "ID of the agent to list targets for"),
    ),
    responses(
        (status = 200, description = "Successfully retrieved agent targets", body = Vec<AgentTarget>),
        (status = 403, description = "Forbidden - PAK does not have required rights", body = serde_json::Value),
        (status = 500, description = "Internal server error", body = serde_json::Value),
    ),
    security(
        ("admin_pak" = []),
        ("agent_pak" = []),
    )
)]
async fn list_targets(
    State(dal): State<DAL>,
    Extension(auth_payload): Extension<AuthPayload>,
    Path(id): Path<Uuid>,
) -> Result<Json<Vec<AgentTarget>>, (StatusCode, Json<serde_json::Value>)> {
    info!("Handling request to list targets for agent with ID: {}", id);
    if !auth_payload.admin && auth_payload.agent != Some(id) {
        warn!(
            "Unauthorized attempt to list targets for agent with ID: {}",
            id
        );
        return Err((
            StatusCode::FORBIDDEN,
            Json(serde_json::json!({"error": "Unauthorized"})),
        ));
    }

    match dal.agent_targets().list_for_agent(id) {
        Ok(targets) => {
            info!(
                "Successfully retrieved {} targets for agent with ID: {}",
                targets.len(),
                id
            );
            Ok(Json(targets))
        }
        Err(e) => {
            error!("Failed to fetch targets for agent with ID {}: {:?}", id, e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": "Failed to fetch agent targets"})),
            ))
        }
    }
}

/// Adds a new target to a specific agent.
///
/// # Authorization
/// Requires admin privileges or matching agent ID.
#[utoipa::path(
    post,
    path = "/agents/{id}/targets",
    tag = "agent-targets",
    params(
        ("id" = Uuid, Path, description = "ID of the agent to add the target to"),
    ),
    request_body = NewAgentTarget,
    responses(
        (status = 200, description = "Successfully added agent target", body = AgentTarget),
        (status = 403, description = "Forbidden - PAK does not have required rights", body = serde_json::Value),
        (status = 500, description = "Internal server error", body = serde_json::Value),
    ),
    security(
        ("admin_pak" = []),
        ("agent_pak" = []),
    )
)]
async fn add_target(
    State(dal): State<DAL>,
    Extension(auth_payload): Extension<AuthPayload>,
    Path(id): Path<Uuid>,
    Json(new_target): Json<NewAgentTarget>,
) -> Result<Json<AgentTarget>, (StatusCode, Json<serde_json::Value>)> {
    info!("Handling request to add target for agent with ID: {}", id);
    if !auth_payload.admin && auth_payload.agent != Some(id) {
        warn!(
            "Unauthorized attempt to add target for agent with ID: {}",
            id
        );
        return Err((
            StatusCode::FORBIDDEN,
            Json(serde_json::json!({"error": "Unauthorized"})),
        ));
    }

    match dal.agent_targets().create(&new_target) {
        Ok(target) => {
            info!("Successfully added target for agent with ID: {}", id);
            Ok(Json(target))
        }
        Err(e) => {
            error!("Failed to add target for agent with ID {}: {:?}", id, e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": "Failed to add agent target"})),
            ))
        }
    }
}

/// Removes a target from a specific agent.
///
/// # Authorization
/// Requires admin privileges or matching agent ID.
#[utoipa::path(
    delete,
    path = "/agents/{id}/targets/{stack_id}",
    tag = "agent-targets",
    params(
        ("id" = Uuid, Path, description = "ID of the agent to remove the target from"),
        ("stack_id" = Uuid, Path, description = "ID of the stack to remove from the agent's targets"),
    ),
    responses(
        (status = 204, description = "Successfully removed agent target"),
        (status = 403, description = "Forbidden - PAK does not have required rights", body = serde_json::Value),
        (status = 404, description = "Target not found", body = serde_json::Value),
        (status = 500, description = "Internal server error", body = serde_json::Value),
    ),
    security(
        ("admin_pak" = []),
        ("agent_pak" = []),
    )
)]
async fn remove_target(
    State(dal): State<DAL>,
    Extension(auth_payload): Extension<AuthPayload>,
    Path((id, stack_id)): Path<(Uuid, Uuid)>,
) -> Result<StatusCode, (StatusCode, Json<serde_json::Value>)> {
    info!(
        "Handling request to remove target for stack {} from agent with ID: {}",
        stack_id, id
    );
    if !auth_payload.admin && auth_payload.agent != Some(id) {
        warn!(
            "Unauthorized attempt to remove target from agent with ID: {}",
            id
        );
        return Err((
            StatusCode::FORBIDDEN,
            Json(serde_json::json!({"error": "Unauthorized"})),
        ));
    }

    match dal.agent_targets().delete_by_agent_and_stack(id, stack_id) {
        Ok(deleted_count) => {
            if deleted_count > 0 {
                info!(
                    "Successfully removed target for stack {} from agent with ID: {}",
                    stack_id, id
                );
                Ok(StatusCode::NO_CONTENT)
            } else {
                warn!(
                    "Target for stack {} not found for agent with ID: {}",
                    stack_id, id
                );
                Err((
                    StatusCode::NOT_FOUND,
                    Json(serde_json::json!({"error": "Target not found"})),
                ))
            }
        }
        Err(e) => {
            error!(
                "Failed to remove target for stack {} from agent with ID {}: {:?}",
                stack_id, id, e
            );
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": "Failed to remove agent target"})),
            ))
        }
    }
}

/// Records a heartbeat for a specific agent.
///
/// # Authorization
/// Requires matching agent ID.
#[utoipa::path(
    post,
    path = "/agents/{id}/heartbeat",
    tag = "agents",
    params(
        ("id" = Uuid, Path, description = "ID of the agent to record heartbeat for"),
    ),
    responses(
        (status = 204, description = "Successfully recorded agent heartbeat"),
        (status = 403, description = "Forbidden - PAK does not have required rights", body = serde_json::Value),
        (status = 500, description = "Internal server error", body = serde_json::Value),
    ),
    security(
        ("agent_pak" = []),
    )
)]
async fn record_heartbeat(
    State(dal): State<DAL>,
    Extension(auth_payload): Extension<AuthPayload>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, (StatusCode, Json<serde_json::Value>)> {
    info!(
        "Handling request to record heartbeat for agent with ID: {}",
        id
    );
    if auth_payload.agent != Some(id) {
        warn!(
            "Unauthorized attempt to record heartbeat for agent with ID: {}",
            id
        );
        return Err((
            StatusCode::FORBIDDEN,
            Json(serde_json::json!({"error": "Unauthorized"})),
        ));
    }

    match dal.agents().record_heartbeat(id) {
        Ok(_) => {
            info!("Successfully recorded heartbeat for agent with ID: {}", id);

            // Update heartbeat age metric (age is now 0 since we just recorded it)
            // Also get agent name for the metric label
            if let Ok(Some(agent)) = dal.agents().get(id) {
                metrics::set_agent_heartbeat_age(&id.to_string(), &agent.name, 0.0);
            }

            Ok(StatusCode::NO_CONTENT)
        }
        Err(e) => {
            error!(
                "Failed to record heartbeat for agent with ID {}: {:?}",
                id, e
            );
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": "Failed to record agent heartbeat"})),
            ))
        }
    }
}

/// Defines query parameters for the target state endpoint
#[derive(Deserialize, Default)]
struct TargetStateParams {
    /// Mode of operation: "incremental" (default) or "full"
    mode: Option<String>,
}

/// Retrieves the target state (deployment objects that should be applied) for a specific agent.
///
/// # Query Parameters
/// * `mode` - Optional. Specifies the mode of operation:
///   * `incremental` (default) - Returns only objects that haven't been deployed yet
///   * `full` - Returns all objects regardless of deployment status
///
/// # Authorization
/// Requires admin privileges or matching agent ID.
#[utoipa::path(
    get,
    path = "/agents/{id}/target-state",
    tag = "agents",
    params(
        ("id" = Uuid, Path, description = "ID of the agent to get target state for"),
        ("mode" = Option<String>, Query, description = "Mode of operation: 'incremental' (default) or 'full'")
    ),
    responses(
        (status = 200, description = "Successfully retrieved target state", body = Vec<DeploymentObject>),
        (status = 403, description = "Forbidden - PAK does not have required rights", body = serde_json::Value),
        (status = 500, description = "Internal server error", body = serde_json::Value),
    ),
    security(
        ("admin_pak" = []),
        ("agent_pak" = []),
    )
)]
async fn get_target_state(
    State(dal): State<DAL>,
    Extension(auth_payload): Extension<AuthPayload>,
    Path(id): Path<Uuid>,
    Query(params): Query<TargetStateParams>,
) -> Result<Json<Vec<DeploymentObject>>, (StatusCode, Json<serde_json::Value>)> {
    info!(
        "Handling request to get target state for agent with ID: {}",
        id
    );
    if !auth_payload.admin && auth_payload.agent != Some(id) {
        warn!(
            "Unauthorized attempt to get target state for agent with ID: {}",
            id
        );
        return Err((
            StatusCode::FORBIDDEN,
            Json(serde_json::json!({"error": "Unauthorized"})),
        ));
    }

    // Determine if we should include deployed objects based on query parameter
    let include_deployed = params.mode.as_deref() == Some("full");
    info!(
        "Target state request mode is '{}', include_deployed={}",
        params.mode.unwrap_or_else(|| "incremental".to_string()),
        include_deployed
    );

    match dal
        .deployment_objects()
        .get_target_state_for_agent(id, include_deployed)
    {
        Ok(objects) => {
            info!(
                "Successfully retrieved {} objects in target state for agent with ID: {}",
                objects.len(),
                id
            );
            Ok(Json(objects))
        }
        Err(e) => {
            error!(
                "Failed to fetch target state for agent with ID {}: {:?}",
                id, e
            );
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": "Failed to fetch target state"})),
            ))
        }
    }
}

/// Retrieves all stacks associated with a specific agent based on targets, labels, and annotations.
///
/// # Authorization
/// Requires admin privileges or matching agent ID.
#[utoipa::path(
    get,
    path = "/agents/{id}/stacks",
    tag = "agents",
    params(
        ("id" = Uuid, Path, description = "ID of the agent to get associated stacks for"),
    ),
    responses(
        (status = 200, description = "Successfully retrieved associated stacks", body = Vec<Stack>),
        (status = 403, description = "Forbidden - PAK does not have required rights", body = serde_json::Value),
        (status = 500, description = "Internal server error", body = serde_json::Value),
    ),
    security(
        ("admin_pak" = []),
        ("agent_pak" = []),
    )
)]
async fn get_associated_stacks(
    State(dal): State<DAL>,
    Extension(auth_payload): Extension<AuthPayload>,
    Path(id): Path<Uuid>,
) -> Result<Json<Vec<Stack>>, (StatusCode, Json<serde_json::Value>)> {
    info!(
        "Handling request to get associated stacks for agent with ID: {}",
        id
    );
    if !auth_payload.admin && auth_payload.agent != Some(id) {
        warn!(
            "Unauthorized attempt to get associated stacks for agent with ID: {}",
            id
        );
        return Err((
            StatusCode::FORBIDDEN,
            Json(serde_json::json!({"error": "Unauthorized"})),
        ));
    }

    match dal.stacks().get_associated_stacks(id) {
        Ok(stacks) => {
            info!(
                "Successfully retrieved {} associated stacks for agent with ID: {}",
                stacks.len(),
                id
            );
            Ok(Json(stacks))
        }
        Err(e) => {
            error!(
                "Failed to fetch associated stacks for agent with ID {}: {:?}",
                id, e
            );
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": "Failed to fetch associated stacks"})),
            ))
        }
    }
}

/// Rotates the PAK for a specific agent.
///
/// # Authorization
/// Requires either admin privileges or the current agent's PAK.
#[utoipa::path(
    post,
    path = "/api/v1/agents/{id}/rotate-pak",
    responses(
        (status = 200, description = "Successfully rotated agent PAK", body = serde_json::Value),
        (status = 403, description = "Forbidden - Unauthorized access"),
        (status = 404, description = "Agent not found"),
        (status = 500, description = "Internal server error")
    ),
    params(
        ("id" = Uuid, Path, description = "Agent id")
    ),
    security(
        ("admin_pak" = []),
        ("agent_pak" = [])
    ),
    tag = "agents"
)]
async fn rotate_agent_pak(
    State(dal): State<DAL>,
    Extension(auth_payload): Extension<AuthPayload>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    info!("Handling request to rotate PAK for agent with ID: {}", id);

    // Check authorization - must be admin or the agent itself
    if !auth_payload.admin && auth_payload.agent != Some(id) {
        warn!(
            "Unauthorized attempt to rotate PAK for agent with ID: {}",
            id
        );
        return Err((
            StatusCode::FORBIDDEN,
            Json(serde_json::json!({"error": "Unauthorized access"})),
        ));
    }

    // Verify agent exists
    if let Err(e) = dal.agents().get(id) {
        error!("Failed to fetch agent with ID {}: {:?}", id, e);
        return Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": "Failed to fetch agent"})),
        ));
    }

    // Generate new PAK and hash
    let (pak, pak_hash) = match crate::utils::pak::create_pak() {
        Ok((pak, hash)) => (pak, hash),
        Err(e) => {
            error!("Failed to create new PAK: {:?}", e);
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": "Failed to create new PAK"})),
            ));
        }
    };

    // Update agent's PAK hash
    match dal.agents().update_pak_hash(id, pak_hash) {
        Ok(updated_agent) => {
            info!("Successfully rotated PAK for agent with ID: {}", id);

            // Log audit entry for PAK rotation
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
                "pak": pak
            })))
        }
        Err(e) => {
            error!("Failed to update agent PAK hash: {:?}", e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": "Failed to update agent PAK hash"})),
            ))
        }
    }
}
