/*
 * Copyright (c) 2025-2026 Dylan Storey
 * Licensed under the Elastic License 2.0.
 * See LICENSE file in the project root for full license text.
 */

use crate::api::v1::admin::{
    AuditLogListResponse, ConfigChangeInfo, ConfigReloadResponse, WsConnectionInfo,
    WsConnectionsResponse,
};
use crate::api::v1::agents::{CreateAgentRequest, CreateAgentResponse, HeartbeatReport};
use crate::api::v1::diagnostics::{
    CreateDiagnosticRequest, DiagnosticResponse, SubmitDiagnosticResult,
};
use crate::api::v1::error::ErrorResponse;
use crate::api::v1::fleet::{AgentFleetStatusResponse, FleetAgentRecord};
use crate::api::v1::generators::CreateGeneratorResponse;
use crate::api::v1::health::{
    DeploymentHealthResponse, DeploymentObjectHealthSummary, DeploymentObjectHealthUpdate,
    HealthStatusUpdate, StackHealthResponse,
};
use crate::api::v1::middleware::AuthResponse;
use crate::api::v1::stacks::{
    CreateDeploymentObjectRequest, K8sEventHistoryResponse, PodLogHistoryResponse, RetentionInfo,
    TemplateInstantiationRequest,
};
use crate::api::v1::templates::{
    AddAnnotationRequest, CreateTemplateRequest, UpdateTemplateRequest,
};
use crate::api::v1::webhooks::{
    CreateWebhookRequest, DeliveryResultRequest, ListDeliveriesQuery, PendingWebhookDelivery,
    UpdateWebhookRequest, WebhookResponse,
};
use crate::api::v1::work_orders::{
    ClaimWorkOrderRequest, CompleteWorkOrderRequest, CreateWorkOrderRequest, WorkOrderTargeting,
};
use crate::api::v1::{
    admin, agent_events, agents, auth, deployment_objects, diagnostics, fleet, generators, health,
    stacks, templates, webhooks, work_orders,
};
use crate::dal::DAL;
use axum::{Router, response::Json, routing::get};
use brokkr_models::models::agent_generator_registrations::AgentGeneratorRegistration;
use brokkr_models::models::{
    agent_annotations::{AgentAnnotation, NewAgentAnnotation},
    agent_events::{AgentEvent, NewAgentEvent},
    agent_k8s_events::AgentK8sEvent,
    agent_labels::{AgentLabel, NewAgentLabel},
    agent_pod_logs::AgentPodLog,
    agent_targets::{AgentTarget, NewAgentTarget},
    agents::{Agent, NewAgent},
    audit_logs::AuditLog,
    deployment_health::{DeploymentHealth, HealthSummary, ResourceHealth},
    deployment_objects::{DeploymentObject, NewDeploymentObject},
    diagnostic_requests::DiagnosticRequest,
    diagnostic_results::DiagnosticResult,
    generator::{Generator, NewGenerator},
    stack_annotations::{NewStackAnnotation, StackAnnotation},
    stack_labels::{NewStackLabel, StackLabel},
    stack_templates::{NewStackTemplate, StackTemplate},
    stacks::{NewStack, Stack},
    template_annotations::{NewTemplateAnnotation, TemplateAnnotation},
    template_labels::{NewTemplateLabel, TemplateLabel},
    webhooks::{WebhookDelivery, WebhookFilters, WebhookSubscription},
    work_orders::{WorkOrder, WorkOrderLog},
};
use utoipa::{
    OpenApi,
    openapi::{
        LicenseBuilder, Server,
        security::{ApiKey, ApiKeyValue, SecurityScheme},
    },
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
        generators::rotate_generator_pak,
        generators::register_agent,
        generators::deregister_agent,
        generators::list_generator_registered_agents,
        agents::list_agent_registrations,
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
        agents::get_target_state,
        agents::list_events,
        agents::create_event,
        agents::record_heartbeat,
        agents::get_associated_stacks,
        agents::rotate_agent_pak,
        fleet::list_fleet,
        fleet::get_agent_fleet_status,
        deployment_objects::get_deployment_object,
        stacks::list_stacks,
        stacks::create_stack,
        stacks::get_stack,
        stacks::update_stack,
        stacks::delete_stack,
        stacks::instantiate_template,
        stacks::list_deployment_objects,
        stacks::create_deployment_object,
        stacks::list_labels,
        stacks::add_label,
        stacks::remove_label,
        stacks::list_annotations,
        stacks::add_annotation,
        stacks::remove_annotation,
        stacks::list_telemetry_events,
        stacks::list_telemetry_logs,
        templates::list_templates,
        templates::create_template,
        templates::get_template,
        templates::update_template,
        templates::delete_template,
        templates::list_labels,
        templates::add_label,
        templates::remove_label,
        templates::list_annotations,
        templates::add_annotation,
        templates::remove_annotation,
        work_orders::list_work_orders,
        work_orders::create_work_order,
        work_orders::get_work_order,
        work_orders::delete_work_order,
        work_orders::claim_work_order,
        work_orders::complete_work_order,
        work_orders::list_pending_for_agent,
        work_orders::list_work_order_log,
        work_orders::get_work_order_log,
        auth::verify_pak,
        health::update_health_status,
        health::get_deployment_health,
        health::get_stack_health,
        diagnostics::create_diagnostic_request,
        diagnostics::get_diagnostic,
        diagnostics::get_pending_diagnostics,
        diagnostics::claim_diagnostic,
        diagnostics::submit_diagnostic_result,
        webhooks::list_webhooks,
        webhooks::list_event_types,
        webhooks::create_webhook,
        webhooks::get_webhook,
        webhooks::update_webhook,
        webhooks::delete_webhook,
        webhooks::list_deliveries,
        webhooks::test_webhook,
        webhooks::get_pending_agent_webhooks,
        webhooks::report_delivery_result,
        admin::reload_config,
        admin::list_audit_logs,
        admin::list_ws_connections,
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
            CreateAgentResponse,
            CreateAgentRequest,
            AgentGeneratorRegistration,
            HeartbeatReport,
            FleetAgentRecord,
            AgentFleetStatusResponse,
            DeploymentObject,
            NewDeploymentObject,
            CreateDeploymentObjectRequest,
            Stack,
            NewStack,
            AuthResponse,
            StackTemplate,
            NewStackTemplate,
            TemplateLabel,
            NewTemplateLabel,
            TemplateAnnotation,
            NewTemplateAnnotation,
            TemplateInstantiationRequest,
            CreateTemplateRequest,
            UpdateTemplateRequest,
            AddAnnotationRequest,
            WorkOrder,
            WorkOrderLog,
            CreateWorkOrderRequest,
            WorkOrderTargeting,
            ClaimWorkOrderRequest,
            CompleteWorkOrderRequest,
            DeploymentHealth,
            HealthSummary,
            ResourceHealth,
            HealthStatusUpdate,
            DeploymentObjectHealthUpdate,
            DeploymentHealthResponse,
            StackHealthResponse,
            DeploymentObjectHealthSummary,
            DiagnosticRequest,
            DiagnosticResult,
            CreateDiagnosticRequest,
            DiagnosticResponse,
            SubmitDiagnosticResult,
            ConfigReloadResponse,
            ConfigChangeInfo,
            AuditLog,
            AuditLogListResponse,
            ErrorResponse,
            AgentK8sEvent,
            AgentPodLog,
            K8sEventHistoryResponse,
            PodLogHistoryResponse,
            RetentionInfo,
            WsConnectionInfo,
            WsConnectionsResponse,
            NewAgentEvent,
            StackLabel,
            NewStackLabel,
            StackAnnotation,
            NewStackAnnotation,
            WebhookSubscription,
            WebhookDelivery,
            WebhookFilters,
            CreateWebhookRequest,
            UpdateWebhookRequest,
            WebhookResponse,
            ListDeliveriesQuery,
            PendingWebhookDelivery,
            DeliveryResultRequest,
        )
    ),
    tags(
        (name = "agent-events", description = "Agent Events management API"),
        (name = "generators", description = "Generator management API"),
        (name = "agent-labels", description = "Agent Labels management API"),
        (name = "agent-annotations", description = "Agent Annotations management API"),
        (name = "agent-targets", description = "Agent Targets management API"),
        (name = "agents", description = "Core Agent management API"),
        (name = "fleet", description = "Agent fleet legibility API (measured signals)"),
        (name = "deployment-objects", description = "Deployment Objects management API"),
        (name = "stacks", description = "Stack management API"),
        (name = "templates", description = "Stack Templates management API"),
        (name = "work-orders", description = "Work Orders management API"),
        (name = "work-order-log", description = "Work Order Log API"),
        (name = "auth", description = "Authentication API"),
        (name = "health", description = "Deployment Health monitoring API"),
        (name = "diagnostics", description = "On-demand diagnostics API"),
        (name = "webhooks", description = "Webhook subscription and delivery API"),
        (name = "Admin", description = "Administrative operations API")
    ),
    modifiers(&SecurityAddon, &ServersAddon, &LicenseAddon)
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

/// Declares the API base URL. Generated SDK clients prepend this prefix to
/// every path. Annotations therefore document resource paths only
/// (`/agents`, `/stacks/{id}`, etc.) and stay decoupled from the version prefix.
struct ServersAddon;

impl utoipa::Modify for ServersAddon {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        openapi.servers = Some(vec![Server::new("/api/v1")]);
    }
}

/// Normalizes `info.license` to a name+URL form. utoipa auto-derives the
/// license from the crate's SPDX `license` field, which emits the OpenAPI
/// **3.1-only** `info.license.identifier` member. `openapi-typescript` (used to
/// generate the TS SDK) validates against 3.0 and rejects `identifier`, so we
/// pin a 3.0-valid license here instead.
struct LicenseAddon;

impl utoipa::Modify for LicenseAddon {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        openapi.info.license = Some(
            LicenseBuilder::new()
                .name("Elastic-2.0")
                .url(Some("https://www.elastic.co/licensing/elastic-license"))
                .build(),
        );
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
