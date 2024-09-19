use crate::dal::DAL;
use axum::{
    extract::{Path, Query, State},
    routing::{delete, get, post},
    Json, Router,
};
use brokkr_models::models::{
    agent_annotations::{AgentAnnotation, NewAgentAnnotation},
    agent_events::{AgentEvent, NewAgentEvent},
    agent_labels::{AgentLabel, NewAgentLabel},
    agent_targets::{AgentTarget, NewAgentTarget},
    agents::{Agent, NewAgent},
    deployment_objects::DeploymentObject,
};
use chrono::Utc;
use serde::Deserialize;
use uuid::Uuid;

#[derive(Deserialize)]
struct AgentFilters {
    labels: Option<String>,
    annotations: Option<String>,
    cluster: Option<String>,
    status: Option<String>,
}

#[derive(Deserialize)]
struct UpdateAgent {
    name: Option<String>,
    cluster_name: Option<String>,
    status: Option<String>,
    pak_hash: Option<String>,
}

#[derive(Deserialize)]
struct NewEventData {
    event_type: String,
    status: String,
    message: Option<String>,
    deployment_object_id: Uuid,
}

async fn list_agents(
    State(dal): State<DAL>,
    Query(filters): Query<AgentFilters>,
) -> Result<Json<Vec<Agent>>, axum::http::StatusCode> {
    let mut agents = dal
        .agents()
        .list()
        .map_err(|_| axum::http::StatusCode::INTERNAL_SERVER_ERROR)?;

    // Apply filters
    if let Some(labels) = filters.labels {
        // Implement label filtering logic
        // filter agent_labels
        // join to agents
        // return
        todo!()
    }
    if let Some(annotations) = filters.annotations {
        // Implement annotation filtering logic
        // filter agent_annotations
        // join agents
        // return
        todo!()
    }
    if let Some(cluster) = filters.cluster {
        agents.retain(|agent| agent.cluster_name == cluster);
    }
    if let Some(status) = filters.status {
        agents.retain(|agent| agent.status == status);
    }

    Ok(Json(agents))
}

async fn create_agent(
    State(dal): State<DAL>,
    Json(new_agent): Json<NewAgent>,
) -> Result<Json<Agent>, axum::http::StatusCode> {
    let agent = dal
        .agents()
        .create(&new_agent)
        .map_err(|_| axum::http::StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(agent))
}

async fn get_agent(
    State(dal): State<DAL>,
    Path(id): Path<Uuid>,
) -> Result<Json<Agent>, axum::http::StatusCode> {
    let agent = dal
        .agents()
        .get(id)
        .map_err(|_| axum::http::StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(axum::http::StatusCode::NOT_FOUND)?;
    Ok(Json(agent))
}

async fn update_agent(
    State(dal): State<DAL>,
    Path(id): Path<Uuid>,
    Json(update_agent): Json<UpdateAgent>,
) -> Result<Json<Agent>, axum::http::StatusCode> {
    let mut agent = dal
        .agents()
        .get(id)
        .map_err(|_| axum::http::StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(axum::http::StatusCode::NOT_FOUND)?;

    if let Some(name) = update_agent.name {
        agent.name = name;
    }
    if let Some(cluster_name) = update_agent.cluster_name {
        agent.cluster_name = cluster_name;
    }
    if let Some(status) = update_agent.status {
        agent.status = status;
    }
    if let Some(pak_hash) = update_agent.pak_hash {
        agent.pak_hash = pak_hash;
    }

    let updated_agent = dal
        .agents()
        .update(id, &agent)
        .map_err(|_| axum::http::StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(updated_agent))
}

async fn delete_agent(
    State(dal): State<DAL>,
    Path(id): Path<Uuid>,
) -> Result<Json<()>, axum::http::StatusCode> {
    dal.agents()
        .soft_delete(id)
        .map_err(|_| axum::http::StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(()))
}

async fn list_agent_events(
    State(dal): State<DAL>,
    Path(id): Path<Uuid>,
) -> Result<Json<Vec<AgentEvent>>, axum::http::StatusCode> {
    let events = dal
        .agent_events()
        .get_events(None, Some(id))
        .map_err(|_| axum::http::StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(events))
}

async fn create_agent_event(
    State(dal): State<DAL>,
    Path(id): Path<Uuid>,
    Json(new_event_data): Json<NewEventData>,
) -> Result<Json<AgentEvent>, axum::http::StatusCode> {
    let new_event = NewAgentEvent {
        agent_id: id,
        deployment_object_id: new_event_data.deployment_object_id,
        event_type: new_event_data.event_type,
        status: new_event_data.status,
        message: new_event_data.message,
    };

    let event = dal
        .agent_events()
        .create(&new_event)
        .map_err(|_| axum::http::StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(event))
}

async fn list_agent_labels(
    State(dal): State<DAL>,
    Path(id): Path<Uuid>,
) -> Result<Json<Vec<AgentLabel>>, axum::http::StatusCode> {
    let labels = dal
        .agent_labels()
        .list_for_agent(id)
        .map_err(|_| axum::http::StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(labels))
}

async fn add_agent_label(
    State(dal): State<DAL>,
    Path(id): Path<Uuid>,
    Json(label): Json<String>,
) -> Result<Json<AgentLabel>, axum::http::StatusCode> {
    let new_label =
        NewAgentLabel::new(id, label).map_err(|_| axum::http::StatusCode::BAD_REQUEST)?;

    let created_label = dal
        .agent_labels()
        .create(&new_label)
        .map_err(|_| axum::http::StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(created_label))
}

async fn remove_agent_label(
    State(dal): State<DAL>,
    Path((id, label)): Path<(Uuid, String)>,
) -> Result<Json<()>, axum::http::StatusCode> {
    let labels = dal
        .agent_labels()
        .list_for_agent(id)
        .map_err(|_| axum::http::StatusCode::INTERNAL_SERVER_ERROR)?;

    if let Some(label_to_remove) = labels.iter().find(|l| l.label == label) {
        dal.agent_labels()
            .delete(label_to_remove.id)
            .map_err(|_| axum::http::StatusCode::INTERNAL_SERVER_ERROR)?;
        Ok(Json(()))
    } else {
        Err(axum::http::StatusCode::NOT_FOUND)
    }
}

async fn list_agent_annotations(
    State(dal): State<DAL>,
    Path(id): Path<Uuid>,
) -> Result<Json<Vec<AgentAnnotation>>, axum::http::StatusCode> {
    let annotations = dal
        .agent_annotations()
        .list_for_agent(id)
        .map_err(|_| axum::http::StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(annotations))
}

async fn add_agent_annotation(
    State(dal): State<DAL>,
    Path(id): Path<Uuid>,
    Json(annotation): Json<(String, String)>,
) -> Result<Json<AgentAnnotation>, axum::http::StatusCode> {
    let (key, value) = annotation;
    let new_annotation =
        NewAgentAnnotation::new(id, key, value).map_err(|_| axum::http::StatusCode::BAD_REQUEST)?;

    let created_annotation = dal
        .agent_annotations()
        .create(&new_annotation)
        .map_err(|_| axum::http::StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(created_annotation))
}

async fn remove_agent_annotation(
    State(dal): State<DAL>,
    Path((id, key)): Path<(Uuid, String)>,
) -> Result<Json<()>, axum::http::StatusCode> {
    let annotations = dal
        .agent_annotations()
        .list_for_agent(id)
        .map_err(|_| axum::http::StatusCode::INTERNAL_SERVER_ERROR)?;

    if let Some(annotation_to_remove) = annotations.iter().find(|a| a.key == key) {
        dal.agent_annotations()
            .delete(annotation_to_remove.id)
            .map_err(|_| axum::http::StatusCode::INTERNAL_SERVER_ERROR)?;
        Ok(Json(()))
    } else {
        Err(axum::http::StatusCode::NOT_FOUND)
    }
}

async fn list_agent_targets(
    State(dal): State<DAL>,
    Path(id): Path<Uuid>,
) -> Result<Json<Vec<AgentTarget>>, axum::http::StatusCode> {
    let targets = dal
        .agent_targets()
        .list_for_agent(id)
        .map_err(|_| axum::http::StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(targets))
}

async fn add_agent_target(
    State(dal): State<DAL>,
    Path(id): Path<Uuid>,
    Json(stack_id): Json<Uuid>,
) -> Result<Json<AgentTarget>, axum::http::StatusCode> {
    let new_target =
        NewAgentTarget::new(id, stack_id).map_err(|_| axum::http::StatusCode::BAD_REQUEST)?;

    let created_target = dal
        .agent_targets()
        .create(&new_target)
        .map_err(|_| axum::http::StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(created_target))
}

async fn remove_agent_target(
    State(dal): State<DAL>,
    Path((id, stack_id)): Path<(Uuid, Uuid)>,
) -> Result<Json<()>, axum::http::StatusCode> {
    let targets = dal
        .agent_targets()
        .list_for_agent(id)
        .map_err(|_| axum::http::StatusCode::INTERNAL_SERVER_ERROR)?;

    if let Some(target_to_remove) = targets.iter().find(|t| t.stack_id == stack_id) {
        dal.agent_targets()
            .delete(target_to_remove.id)
            .map_err(|_| axum::http::StatusCode::INTERNAL_SERVER_ERROR)?;
        Ok(Json(()))
    } else {
        Err(axum::http::StatusCode::NOT_FOUND)
    }
}

async fn record_agent_heartbeat(
    State(dal): State<DAL>,
    Path(id): Path<Uuid>,
) -> Result<Json<()>, axum::http::StatusCode> {
    let mut agent = dal
        .agents()
        .get(id)
        .map_err(|_| axum::http::StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(axum::http::StatusCode::NOT_FOUND)?;

    agent.last_heartbeat = Some(Utc::now());
    dal.agents()
        .update(id, &agent)
        .map_err(|_| axum::http::StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(()))
}

async fn get_agent_event(
    State(dal): State<DAL>,
    Path((agent_id, event_id)): Path<(Uuid, Uuid)>,
) -> Result<Json<AgentEvent>, axum::http::StatusCode> {
    let event = dal
        .agent_events()
        .get(event_id)
        .map_err(|_| axum::http::StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(axum::http::StatusCode::NOT_FOUND)?;

    if event.agent_id != agent_id {
        return Err(axum::http::StatusCode::NOT_FOUND);
    }

    Ok(Json(event))
}

async fn get_applicable_deployment_objects(
    State(dal): State<DAL>,
    Path(agent_id): Path<Uuid>,
) -> Result<Json<Vec<DeploymentObject>>, axum::http::StatusCode> {
    // Get the agent targets for the specified agent
    let agent_targets = dal
        .agent_targets()
        .list_for_agent(agent_id)
        .map_err(|_| axum::http::StatusCode::INTERNAL_SERVER_ERROR)?;

    // Collect all deployment objects for the agent's target stacks
    let mut applicable_objects = Vec::new();
    for target in agent_targets {
        let stack_objects = dal
            .deployment_objects()
            .list_for_stack(target.stack_id)
            .map_err(|_| axum::http::StatusCode::INTERNAL_SERVER_ERROR)?;
        applicable_objects.extend(stack_objects);
    }

    // Sort the objects by their sequence_id in descending order
    applicable_objects.sort_by(|a, b| b.sequence_id.cmp(&a.sequence_id));

    // Remove duplicates, keeping only the latest version of each deployment object
    applicable_objects.dedup_by(|a, b| a.stack_id == b.stack_id);

    Ok(Json(applicable_objects))
}

/// Create the router for agent-related endpoints
pub fn configure_routes() -> Router<DAL> {
    Router::new()
        .route("/agents", get(list_agents).post(create_agent))
        .route(
            "/agents/:id",
            get(get_agent).put(update_agent).delete(delete_agent),
        )
        .route(
            "/agents/:id/events",
            get(list_agent_events).post(create_agent_event),
        )
        .route(
            "/agents/:id/labels",
            get(list_agent_labels).post(add_agent_label),
        )
        .route("/agents/:id/labels/:label", delete(remove_agent_label))
        .route(
            "/agents/:id/annotations",
            get(list_agent_annotations).post(add_agent_annotation),
        )
        .route(
            "/agents/:id/annotations/:key",
            delete(remove_agent_annotation),
        )
        .route(
            "/agents/:id/targets",
            get(list_agent_targets).post(add_agent_target),
        )
        .route("/agents/:id/targets/:stack_id", delete(remove_agent_target))
        .route("/agents/:id/heartbeat", post(record_agent_heartbeat))
        .route("/agents/:agent_id/events/:event_id", get(get_agent_event))
        .route(
            "/agents/:id/applicable-deployment-objects",
            get(get_applicable_deployment_objects),
        )
}
