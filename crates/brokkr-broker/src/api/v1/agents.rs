//! Agent management API endpoints.
//!
//! This module provides routes and handlers for managing agents, including CRUD operations,
//! event logging, label management, annotation management, target management, and heartbeat recording.

use crate::api::v1::middleware::AuthPayload;
use crate::dal::DAL;
use crate::utils::pak;
use axum::http::StatusCode;
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
use brokkr_utils::logging::prelude::*;
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
        .route(
            "/agents/:id/applicable-deployment-objects",
            get(get_applicable_deployment_objects),
        )
        .route("/agents/:id/stacks", get(get_associated_stacks))
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
/// Requires admin privileges or matching agent ID.
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
        ("agent_pak" = []),
    )
)]
async fn add_label(
    State(dal): State<DAL>,
    Extension(auth_payload): Extension<AuthPayload>,
    Path(id): Path<Uuid>,
    Json(new_label): Json<NewAgentLabel>,
) -> Result<Json<AgentLabel>, (StatusCode, Json<serde_json::Value>)> {
    info!("Handling request to add label for agent with ID: {}", id);
    if !auth_payload.admin && auth_payload.agent != Some(id) {
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
/// Requires admin privileges or matching agent ID.
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
        ("agent_pak" = []),
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
    if !auth_payload.admin && auth_payload.agent != Some(id) {
        warn!(
            "Unauthorized attempt to remove label from agent with ID: {}",
            id
        );
        return Err((
            StatusCode::FORBIDDEN,
            Json(serde_json::json!({"error": "Unauthorized"})),
        ));
    }

    match dal.agent_labels().list_for_agent(id) {
        Ok(labels) => {
            if let Some(agent_label) = labels.into_iter().find(|l| l.label == label) {
                match dal.agent_labels().delete(agent_label.id) {
                    Ok(_) => {
                        info!(
                            "Successfully removed label '{}' from agent with ID: {}",
                            label, id
                        );
                        Ok(StatusCode::NO_CONTENT)
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
            } else {
                warn!("Label '{}' not found for agent with ID: {}", label, id);
                Err((
                    StatusCode::NOT_FOUND,
                    Json(serde_json::json!({"error": "Label not found"})),
                ))
            }
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
/// Requires admin privileges or matching agent ID.
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
        ("agent_pak" = []),
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
    if !auth_payload.admin && auth_payload.agent != Some(id) {
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
/// Requires admin privileges or matching agent ID.
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
        ("agent_pak" = []),
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
    if !auth_payload.admin && auth_payload.agent != Some(id) {
        warn!(
            "Unauthorized attempt to remove annotation from agent with ID: {}",
            id
        );
        return Err((
            StatusCode::FORBIDDEN,
            Json(serde_json::json!({"error": "Unauthorized"})),
        ));
    }

    match dal.agent_annotations().list_for_agent(id) {
        Ok(annotations) => {
            if let Some(agent_annotation) = annotations.into_iter().find(|a| a.key == key) {
                match dal.agent_annotations().delete(agent_annotation.id) {
                    Ok(_) => {
                        info!(
                            "Successfully removed annotation '{}' from agent with ID: {}",
                            key, id
                        );
                        Ok(StatusCode::NO_CONTENT)
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

    match dal.agent_targets().list_for_agent(id) {
        Ok(targets) => {
            if let Some(target) = targets.into_iter().find(|t| t.stack_id == stack_id) {
                match dal.agent_targets().delete(target.id) {
                    Ok(_) => {
                        info!(
                            "Successfully removed target for stack {} from agent with ID: {}",
                            stack_id, id
                        );
                        Ok(StatusCode::NO_CONTENT)
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
            error!("Failed to fetch targets for agent with ID {}: {:?}", id, e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": "Failed to fetch agent targets"})),
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

/// Retrieves applicable deployment objects for a specific agent.
///
/// # Authorization
/// Requires admin privileges or matching agent ID.
#[utoipa::path(
    get,
    path = "/agents/{id}/applicable-deployment-objects",
    tag = "agents",
    params(
        ("id" = Uuid, Path, description = "ID of the agent to get deployment objects for"),
    ),
    responses(
        (status = 200, description = "Successfully retrieved applicable deployment objects", body = Vec<DeploymentObject>),
        (status = 403, description = "Forbidden - PAK does not have required rights", body = serde_json::Value),
        (status = 500, description = "Internal server error", body = serde_json::Value),
    ),
    security(
        ("admin_pak" = []),
        ("agent_pak" = []),
    )
)]
async fn get_applicable_deployment_objects(
    State(dal): State<DAL>,
    Extension(auth_payload): Extension<AuthPayload>,
    Path(id): Path<Uuid>,
) -> Result<Json<Vec<DeploymentObject>>, (StatusCode, Json<serde_json::Value>)> {
    info!(
        "Handling request to get applicable deployment objects for agent with ID: {}",
        id
    );
    if !auth_payload.admin && auth_payload.agent != Some(id) {
        warn!(
            "Unauthorized attempt to get applicable deployment objects for agent with ID: {}",
            id
        );
        return Err((
            StatusCode::FORBIDDEN,
            Json(serde_json::json!({"error": "Unauthorized"})),
        ));
    }

    match dal
        .deployment_objects()
        .get_undeployed_objects_for_agent(id)
    {
        Ok(objects) => {
            info!(
                "Successfully retrieved {} applicable deployment objects for agent with ID: {}",
                objects.len(),
                id
            );
            Ok(Json(objects))
        }
        Err(e) => {
            error!(
                "Failed to fetch applicable deployment objects for agent with ID {}: {:?}",
                id, e
            );
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": "Failed to fetch applicable deployment objects"})),
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
