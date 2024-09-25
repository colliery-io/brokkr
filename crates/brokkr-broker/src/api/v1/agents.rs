//! Agent management API endpoints.
//!
//! This module provides routes and handlers for managing agents, including CRUD operations,
//! event logging, label management, annotation management, target management, and heartbeat recording.


use crate::api::v1::middleware::AuthPayload;
use crate::dal::DAL;
use crate::utils::pak;
use axum::http::StatusCode;
use axum::{
    extract::{Extension, Path, State, Query},
    routing::{delete, get, post},
    Json, Router,
};
use brokkr_models::models::agent_annotations::{AgentAnnotation, NewAgentAnnotation};
use brokkr_models::models::agent_events::{AgentEvent, NewAgentEvent};
use brokkr_models::models::agent_labels::{AgentLabel, NewAgentLabel};
use brokkr_models::models::agent_targets::{AgentTarget, NewAgentTarget};
use brokkr_models::models::agents::{Agent, NewAgent};
use brokkr_models::models::deployment_objects::DeploymentObject;
use serde_json::Value;
use serde::Deserialize;
use uuid::Uuid;

/// Creates and returns the router for agent-related endpoints.
pub fn routes() -> Router<DAL> {
    Router::new()
        .route("/agents", get(list_agents).post(create_agent))
        .route(
            "/agents/",
            get(search_agent),
        )        
        .route(
            "/agents/:id",
            get(get_agent_by_id).put(update_agent).delete(delete_agent),
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
        // .route("/agents", get(get_agent))
}

/// Lists all agents.
///
/// # Authorization
/// Requires admin privileges.
async fn list_agents(
    State(dal): State<DAL>,
    Extension(auth_payload): Extension<AuthPayload>,
) -> Result<Json<Vec<Agent>>, (StatusCode, Json<serde_json::Value>)> {
    if !auth_payload.admin {
        return Err((
            StatusCode::FORBIDDEN,
            Json(serde_json::json!({"error": "Unauthorized"})),
        ));
    }

    match dal.agents().list() {
        Ok(agents) => Ok(Json(agents)),
        Err(_) => {
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
async fn create_agent(
    State(dal): State<DAL>,
    Extension(auth_payload): Extension<AuthPayload>,
    Json(new_agent): Json<NewAgent>,
) -> Result<Json<Value>, (StatusCode, Json<serde_json::Value>)> {
    if !auth_payload.admin {
        return Err((
            StatusCode::FORBIDDEN,
            Json(serde_json::json!({"error": "Unauthorized"})),
        ));
    }

    match dal.agents().create(&new_agent) {
        Ok(agent) => {
            // Generate initial PAK and set PAK hash
            let (pak, pak_hash) = pak::create_pak().map_err(|_| {
                
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(serde_json::json!({"error": "Failed to create PAK"})),
                )
            })?;

            match dal.agents().update_pak_hash(agent.id, pak_hash) {
                Ok(updated_agent) => {
                    let response = serde_json::json!({
                        "agent": updated_agent,
                        "initial_pak": pak
                    });
                    Ok(Json(response))
                }
                Err(_) => {
                    
                    Err((
                        StatusCode::INTERNAL_SERVER_ERROR,
                        Json(serde_json::json!({"error": "Failed to update agent PAK hash"})),
                    ))
                }
            }
        }
        Err(_) => {
            
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
async fn get_agent_by_id(
    State(dal): State<DAL>,
    Extension(auth_payload): Extension<AuthPayload>,
    Path(id): Path<Uuid>,
) -> Result<Json<Agent>, (StatusCode, Json<serde_json::Value>)> {
    if !auth_payload.admin && auth_payload.agent != Some(id) {
        return Err((
            StatusCode::FORBIDDEN,
            Json(serde_json::json!({"error": "Unauthorized"})),
        ));
    }

    match dal.agents().get(id) {
        Ok(Some(agent)) => Ok(Json(agent)),
        Ok(None) => Err((
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"error": "Agent not found"})),
        )),
        Err(_) => {
            
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
async fn search_agent(
    State(dal): State<DAL>,
    Extension(auth_payload): Extension<AuthPayload>,
    Query(query): Query<AgentQuery>,
) -> Result<Json<Agent>, (StatusCode, Json<serde_json::Value>)> {
    if let (Some(name), Some(cluster_name)) = (query.name.clone(), query.cluster_name.clone()) {
        match dal.agents().get_by_name_and_cluster_name(name, cluster_name) {
            Ok(Some(agent)) => {
                if auth_payload.admin || auth_payload.agent == Some(agent.id) {
                    Ok(Json(agent))
                } else {
                    Err((
                        StatusCode::FORBIDDEN,
                        Json(serde_json::json!({"error": "Unauthorized"})),
                    ))
                }
            }
            Ok(None) => Err((
                StatusCode::NOT_FOUND,
                Json(serde_json::json!({"error": "Agent not found"})),
            )),
            Err(_) => {
                Err((
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(serde_json::json!({"error": "Failed to fetch agent"})),
                ))
            }
        }
    } else {
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
async fn update_agent(
    State(dal): State<DAL>,
    Extension(auth_payload): Extension<AuthPayload>,
    Path(id): Path<Uuid>,
    Json(update_payload): Json<serde_json::Value>,
) -> Result<Json<Agent>, (StatusCode, Json<serde_json::Value>)> {
    if !auth_payload.admin && auth_payload.agent != Some(id) {
        return Err((
            StatusCode::FORBIDDEN,
            Json(serde_json::json!({"error": "Unauthorized"})),
        ));
    }

    let mut agent = dal
        .agents()
        .get(id)
        .map_err(|_| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": "Failed to fetch agent"})),
            )
        })?
        .ok_or((
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"error": "Agent not found"})),
        ))?;

    if let Some(name) = update_payload.get("name").and_then(|v| v.as_str()) {
        agent.name = name.to_string();
    }
    if let Some(cluster_name) = update_payload.get("cluster_name").and_then(|v| v.as_str()) {
        agent.cluster_name = cluster_name.to_string();
    }
    if let Some(status) = update_payload.get("status").and_then(|v| v.as_str()) {
        agent.status = status.to_string();
    }

    let updated_agent = dal.agents().update(id, &agent).map_err(|_| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": "Failed to update agent"})),
        )
    })?;

    Ok(Json(updated_agent))
}

/// Soft deletes an agent.
///
/// # Authorization
/// Requires admin privileges.
async fn delete_agent(
    State(dal): State<DAL>,
    Extension(auth_payload): Extension<AuthPayload>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, (StatusCode, Json<serde_json::Value>)> {
    if !auth_payload.admin {
        return Err((
            StatusCode::FORBIDDEN,
            Json(serde_json::json!({"error": "Unauthorized"})),
        ));
    }

    match dal.agents().soft_delete(id) {
        Ok(_) => Ok(StatusCode::NO_CONTENT),
        Err(_) => {
            
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
async fn list_events(
    State(dal): State<DAL>,
    Extension(auth_payload): Extension<AuthPayload>,
    Path(id): Path<Uuid>,
) -> Result<Json<Vec<Value>>, (StatusCode, Json<serde_json::Value>)> {
    if !auth_payload.admin && auth_payload.agent != Some(id) {
        return Err((
            StatusCode::FORBIDDEN,
            Json(serde_json::json!({"error": "Unauthorized"})),
        ));
    }

    match dal.agent_events().get_events(None, Some(id)) {
        Ok(events) => Ok(Json(
            events
                .into_iter()
                .map(|e| serde_json::to_value(e).unwrap())
                .collect(),
        )),
        Err(_) => {
            
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
async fn create_event(
    State(dal): State<DAL>,
    Extension(auth_payload): Extension<AuthPayload>,
    Path(id): Path<Uuid>,
    Json(new_event): Json<NewAgentEvent>,
) -> Result<Json<AgentEvent>, (StatusCode, Json<serde_json::Value>)> {
    if !auth_payload.admin && auth_payload.agent != Some(id) {
        return Err((
            StatusCode::FORBIDDEN,
            Json(serde_json::json!({"error": "Unauthorized"})),
        ));
    }

    match dal.agent_events().create(&new_event) {
        Ok(event) => Ok(Json(event)),
        Err(_) => {
            
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
async fn list_labels(
    State(dal): State<DAL>,
    Extension(auth_payload): Extension<AuthPayload>,
    Path(id): Path<Uuid>,
) -> Result<Json<Vec<AgentLabel>>, (StatusCode, Json<serde_json::Value>)> {
    if !auth_payload.admin && auth_payload.agent != Some(id) {
        return Err((
            StatusCode::FORBIDDEN,
            Json(serde_json::json!({"error": "Unauthorized"})),
        ));
    }

    match dal.agent_labels().list_for_agent(id) {
        Ok(labels) => Ok(Json(labels)),
        Err(_) => {
            
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
async fn add_label(
    State(dal): State<DAL>,
    Extension(auth_payload): Extension<AuthPayload>,
    Path(id): Path<Uuid>,
    Json(new_label): Json<NewAgentLabel>,
) -> Result<Json<AgentLabel>, (StatusCode, Json<serde_json::Value>)> {
    if !auth_payload.admin && auth_payload.agent != Some(id) {
        return Err((
            StatusCode::FORBIDDEN,
            Json(serde_json::json!({"error": "Unauthorized"})),
        ));
    }

    match dal.agent_labels().create(&new_label) {
        Ok(label) => Ok(Json(label)),
        Err(_) => {
            
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
async fn remove_label(
    State(dal): State<DAL>,
    Extension(auth_payload): Extension<AuthPayload>,
    Path((id, label)): Path<(Uuid, String)>,
) -> Result<StatusCode, (StatusCode, Json<serde_json::Value>)> {
    if !auth_payload.admin && auth_payload.agent != Some(id) {
        return Err((
            StatusCode::FORBIDDEN,
            Json(serde_json::json!({"error": "Unauthorized"})),
        ));
    }

    match dal.agent_labels().list_for_agent(id) {
        Ok(labels) => {
            if let Some(agent_label) = labels.into_iter().find(|l| l.label == label) {
                match dal.agent_labels().delete(agent_label.id) {
                    Ok(_) => Ok(StatusCode::NO_CONTENT),
                    Err(_) => {
                        
                        Err((
                            StatusCode::INTERNAL_SERVER_ERROR,
                            Json(serde_json::json!({"error": "Failed to remove agent label"})),
                        ))
                    }
                }
            } else {
                Err((
                    StatusCode::NOT_FOUND,
                    Json(serde_json::json!({"error": "Label not found"})),
                ))
            }
        }
        Err(_) => {
            
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
async fn list_annotations(
    State(dal): State<DAL>,
    Extension(auth_payload): Extension<AuthPayload>,
    Path(id): Path<Uuid>,
) -> Result<Json<Vec<AgentAnnotation>>, (StatusCode, Json<serde_json::Value>)> {
    if !auth_payload.admin && auth_payload.agent != Some(id) {
        return Err((
            StatusCode::FORBIDDEN,
            Json(serde_json::json!({"error": "Unauthorized"})),
        ));
    }

    match dal.agent_annotations().list_for_agent(id) {
        Ok(annotations) => Ok(Json(annotations)),
        Err(_) => {
            
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
async fn add_annotation(
    State(dal): State<DAL>,
    Extension(auth_payload): Extension<AuthPayload>,
    Path(id): Path<Uuid>,
    Json(new_annotation): Json<NewAgentAnnotation>,
) -> Result<Json<AgentAnnotation>, (StatusCode, Json<serde_json::Value>)> {
    if !auth_payload.admin && auth_payload.agent != Some(id) {
        return Err((
            StatusCode::FORBIDDEN,
            Json(serde_json::json!({"error": "Unauthorized"})),
        ));
    }

    match dal.agent_annotations().create(&new_annotation) {
        Ok(annotation) => Ok(Json(annotation)),
        Err(_) => {
            
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
async fn remove_annotation(
    State(dal): State<DAL>,
    Extension(auth_payload): Extension<AuthPayload>,
    Path((id, key)): Path<(Uuid, String)>,
) -> Result<StatusCode, (StatusCode, Json<serde_json::Value>)> {
    if !auth_payload.admin && auth_payload.agent != Some(id) {
        return Err((
            StatusCode::FORBIDDEN,
            Json(serde_json::json!({"error": "Unauthorized"})),
        ));
    }

    match dal.agent_annotations().list_for_agent(id) {
        Ok(annotations) => {
            if let Some(agent_annotation) = annotations.into_iter().find(|a| a.key == key) {
                match dal.agent_annotations().delete(agent_annotation.id) {
                    Ok(_) => Ok(StatusCode::NO_CONTENT),
                    Err(_) => {
                        
                        Err((
                            StatusCode::INTERNAL_SERVER_ERROR,
                            Json(serde_json::json!({"error": "Failed to remove agent annotation"})),
                        ))
                    }
                }
            } else {
                Err((
                    StatusCode::NOT_FOUND,
                    Json(serde_json::json!({"error": "Annotation not found"})),
                ))
            }
        }
        Err(_) => {
            
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
async fn list_targets(
    State(dal): State<DAL>,
    Extension(auth_payload): Extension<AuthPayload>,
    Path(id): Path<Uuid>,
) -> Result<Json<Vec<AgentTarget>>, (StatusCode, Json<serde_json::Value>)> {
    if !auth_payload.admin && auth_payload.agent != Some(id) {
        return Err((
            StatusCode::FORBIDDEN,
            Json(serde_json::json!({"error": "Unauthorized"})),
        ));
    }

    match dal.agent_targets().list_for_agent(id) {
        Ok(targets) => Ok(Json(targets)),
        Err(_) => {
            
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
async fn add_target(
    State(dal): State<DAL>,
    Extension(auth_payload): Extension<AuthPayload>,
    Path(id): Path<Uuid>,
    Json(new_target): Json<NewAgentTarget>,
) -> Result<Json<AgentTarget>, (StatusCode, Json<serde_json::Value>)> {
    if !auth_payload.admin && auth_payload.agent != Some(id) {
        return Err((
            StatusCode::FORBIDDEN,
            Json(serde_json::json!({"error": "Unauthorized"})),
        ));
    }

    match dal.agent_targets().create(&new_target) {
        Ok(target) => Ok(Json(target)),
        Err(_) => {
            
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
async fn remove_target(
    State(dal): State<DAL>,
    Extension(auth_payload): Extension<AuthPayload>,
    Path((id, stack_id)): Path<(Uuid, Uuid)>,
) -> Result<StatusCode, (StatusCode, Json<serde_json::Value>)> {
    if !auth_payload.admin && auth_payload.agent != Some(id) {
        return Err((
            StatusCode::FORBIDDEN,
            Json(serde_json::json!({"error": "Unauthorized"})),
        ));
    }

    match dal.agent_targets().list_for_agent(id) {
        Ok(targets) => {
            if let Some(target) = targets.into_iter().find(|t| t.stack_id == stack_id) {
                match dal.agent_targets().delete(target.id) {
                    Ok(_) => Ok(StatusCode::NO_CONTENT),
                    Err(_) => {
                        
                        Err((
                            StatusCode::INTERNAL_SERVER_ERROR,
                            Json(serde_json::json!({"error": "Failed to remove agent target"})),
                        ))
                    }
                }
            } else {
                Err((
                    StatusCode::NOT_FOUND,
                    Json(serde_json::json!({"error": "Target not found"})),
                ))
            }
        }
        Err(_) => {
            
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
async fn record_heartbeat(
    State(dal): State<DAL>,
    Extension(auth_payload): Extension<AuthPayload>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, (StatusCode, Json<serde_json::Value>)> {
    if auth_payload.agent != Some(id) {
        return Err((
            StatusCode::FORBIDDEN,
            Json(serde_json::json!({"error": "Unauthorized"})),
        ));
    }

    match dal.agents().record_heartbeat(id) {
        Ok(_) => Ok(StatusCode::NO_CONTENT),
        Err(_) => {
            
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
async fn get_applicable_deployment_objects(
    State(dal): State<DAL>,
    Extension(auth_payload): Extension<AuthPayload>,
    Path(id): Path<Uuid>,
) -> Result<Json<Vec<DeploymentObject>>, (StatusCode, Json<serde_json::Value>)> {
    if !auth_payload.admin && auth_payload.agent != Some(id) {
        return Err((
            StatusCode::FORBIDDEN,
            Json(serde_json::json!({"error": "Unauthorized"})),
        ));
    }

    match dal
        .deployment_objects()
        .get_undeployed_objects_for_agent(id)
    {
        Ok(objects) => Ok(Json(objects)),
        Err(_) => {
            
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": "Failed to fetch applicable deployment objects"})),
            ))
        }
    }
}

