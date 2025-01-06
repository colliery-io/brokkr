use crate::api::v1::generators::CreateGeneratorResponse;
use crate::api::v1::middleware::AuthResponse;
use crate::api::v1::{agent_events, agents, auth, deployment_objects, generators, stacks};
use crate::dal::DAL;
use axum::{response::Json, routing::get, Router};
use brokkr_models::models::{
    agent_annotations::{AgentAnnotation, NewAgentAnnotation},
    agent_events::AgentEvent,
    agent_labels::{AgentLabel, NewAgentLabel},
    agent_targets::{AgentTarget, NewAgentTarget},
    agents::{Agent, NewAgent},
    deployment_objects::{DeploymentObject, NewDeploymentObject},
    generator::{Generator, NewGenerator},
    stacks::{NewStack, Stack},
};
use utoipa::{
    openapi::security::{ApiKey, ApiKeyValue, SecurityScheme},
    OpenApi,
};
use utoipa_swagger_ui::SwaggerUi;

#[derive(OpenApi)]
#[openapi(
    paths(
        agent_events::list_agent_events,
        agent_events::get_agent_event,
        generators::list_generators,
        generators::create_generator,
        generators::get_generator,
        generators::update_generator,
        generators::delete_generator,
        agents::list_labels,
        agents::add_label,
        agents::remove_label,
        agents::list_annotations,
        agents::add_annotation,
        agents::remove_annotation,
        agents::list_targets,
        agents::add_target,
        agents::remove_target,
        agents::list_agents,
        agents::create_agent,
        agents::get_agent,
        agents::update_agent,
        agents::delete_agent,
        agents::search_agent,
        deployment_objects::get_deployment_object,
        stacks::list_stacks,
        stacks::create_stack,
        stacks::get_stack,
        stacks::update_stack,
        stacks::delete_stack,
        auth::verify_pak,
    ),
    components(
        schemas(
            AgentEvent,
            Generator,
            NewGenerator,
            CreateGeneratorResponse,
            AgentLabel,
            NewAgentLabel,
            AgentAnnotation,
            NewAgentAnnotation,
            AgentTarget,
            NewAgentTarget,
            Agent,
            NewAgent,
            DeploymentObject,
            NewDeploymentObject,
            Stack,
            NewStack,
            AuthResponse,
        )
    ),
    tags(
        (name = "agent-events", description = "Agent Events management API"),
        (name = "generators", description = "Generator management API"),
        (name = "agent-labels", description = "Agent Labels management API"),
        (name = "agent-annotations", description = "Agent Annotations management API"),
        (name = "agent-targets", description = "Agent Targets management API"),
        (name = "agents", description = "Core Agent management API"),
        (name = "deployment-objects", description = "Deployment Objects management API"),
        (name = "stacks", description = "Stack management API"),
        (name = "auth", description = "Authentication API")
    ),
    modifiers(&SecurityAddon)
)]
pub struct ApiDoc;

struct SecurityAddon;

impl utoipa::Modify for SecurityAddon {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        if let Some(components) = openapi.components.as_mut() {
            components.add_security_scheme(
                "admin_pak",
                SecurityScheme::ApiKey(ApiKey::Header(ApiKeyValue::new("Authorization"))),
            );
            components.add_security_scheme(
                "generator_pak",
                SecurityScheme::ApiKey(ApiKey::Header(ApiKeyValue::new("Authorization"))),
            );
            components.add_security_scheme(
                "agent_pak",
                SecurityScheme::ApiKey(ApiKey::Header(ApiKeyValue::new("Authorization"))),
            );
        }
    }
}

pub fn configure_openapi() -> Router<DAL> {
    Router::new()
        .route("/docs/openapi.json", get(serve_openapi))
        .merge(SwaggerUi::new("/swagger-ui"))
}

async fn serve_openapi() -> Json<utoipa::openapi::OpenApi> {
    Json(ApiDoc::openapi())
}
