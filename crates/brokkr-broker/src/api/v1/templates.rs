/*
 * Copyright (c) 2025-2026 Dylan Storey
 * Licensed under the Elastic License 2.0.
 * See LICENSE file in the project root for full license text.
 */

//! API endpoints for stack template management.

use crate::api::v1::error::{ApiError, ErrorResponse};
use crate::api::v1::middleware::AuthPayload;
use crate::dal::DAL;
use crate::utils::audit;
use crate::utils::templating;
use axum::{
    extract::{Extension, Path, State},
    http::StatusCode,
    routing::{delete, get},
    Json, Router,
};
use brokkr_models::models::audit_logs::{
    ACTION_TEMPLATE_CREATED, ACTION_TEMPLATE_DELETED, ACTION_TEMPLATE_UPDATED, ACTOR_TYPE_ADMIN,
    ACTOR_TYPE_GENERATOR, RESOURCE_TYPE_TEMPLATE,
};
use brokkr_models::models::stack_templates::StackTemplate;
use brokkr_models::models::template_annotations::{NewTemplateAnnotation, TemplateAnnotation};
use brokkr_models::models::template_labels::{NewTemplateLabel, TemplateLabel};
use serde::{Deserialize, Serialize};
use tracing::{error, info, warn};
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub struct CreateTemplateRequest {
    pub name: String,
    pub description: Option<String>,
    pub template_content: String,
    pub parameters_schema: String,
}

#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub struct UpdateTemplateRequest {
    pub description: Option<String>,
    pub template_content: String,
    pub parameters_schema: String,
}

#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub struct AddAnnotationRequest {
    pub key: String,
    pub value: String,
}

pub fn routes() -> Router<DAL> {
    info!("Setting up template routes");
    Router::new()
        .route("/templates", get(list_templates).post(create_template))
        .route(
            "/templates/:id",
            get(get_template)
                .put(update_template)
                .delete(delete_template),
        )
        .route("/templates/:id/labels", get(list_labels).post(add_label))
        .route("/templates/:id/labels/:label", delete(remove_label))
        .route(
            "/templates/:id/annotations",
            get(list_annotations).post(add_annotation),
        )
        .route("/templates/:id/annotations/:key", delete(remove_annotation))
}

fn can_modify_template(auth: &AuthPayload, template: &StackTemplate) -> bool {
    if auth.admin {
        return true;
    }
    match (auth.generator, template.generator_id) {
        (Some(auth_gen), Some(tmpl_gen)) => auth_gen == tmpl_gen,
        _ => false,
    }
}

fn check_read_access(auth: &AuthPayload, template: &StackTemplate) -> Result<(), ApiError> {
    if auth.admin {
        return Ok(());
    }
    match (auth.generator, template.generator_id) {
        (Some(auth_gen), Some(tmpl_gen)) if auth_gen != tmpl_gen => Err(ApiError::forbidden(
            "template_not_accessible",
            "not authorized to access this template",
        )),
        (None, _) => Err(ApiError::forbidden(
            "template_not_accessible",
            "not authorized to access this template",
        )),
        _ => Ok(()),
    }
}

fn fetch_template_or_404(dal: &DAL, template_id: Uuid) -> Result<StackTemplate, ApiError> {
    dal.templates()
        .get(template_id)
        .map_err(|e| {
            error!("Failed to fetch template: {:?}", e);
            ApiError::internal("failed to fetch template")
        })?
        .ok_or_else(|| ApiError::not_found("template_not_found", "template not found"))
}

#[utoipa::path(
    get,
    path = "/templates",
    tag = "templates",
    responses(
        (status = 200, description = "List of templates", body = Vec<StackTemplate>),
        (status = 403, description = "Forbidden", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse),
    ),
    security(("admin_pak" = []), ("generator_pak" = []))
)]
async fn list_templates(
    State(dal): State<DAL>,
    Extension(auth_payload): Extension<AuthPayload>,
) -> Result<Json<Vec<StackTemplate>>, ApiError> {
    info!("Handling request to list templates");

    let templates = if auth_payload.admin {
        dal.templates().list().map_err(|e| {
            error!("Failed to fetch templates: {:?}", e);
            ApiError::internal("failed to fetch templates")
        })?
    } else if let Some(generator_id) = auth_payload.generator {
        let system = dal.templates().list_system_templates().map_err(|e| {
            error!("Failed to fetch system templates: {:?}", e);
            ApiError::internal("failed to fetch templates")
        })?;
        let own = dal
            .templates()
            .list_for_generator(generator_id)
            .map_err(|e| {
                error!("Failed to fetch generator templates: {:?}", e);
                ApiError::internal("failed to fetch templates")
            })?;
        let mut all = system;
        all.extend(own);
        all
    } else {
        warn!("Unauthorized attempt to list templates");
        return Err(ApiError::forbidden(
            "templates_not_accessible",
            "admin or generator access required",
        ));
    };

    info!("Successfully retrieved {} templates", templates.len());
    Ok(Json(templates))
}

/// Resolves the audit actor for template endpoints: the admin, or the
/// generator acting on its own templates.
fn audit_actor(auth_payload: &AuthPayload) -> (&'static str, Option<uuid::Uuid>) {
    if auth_payload.admin {
        (ACTOR_TYPE_ADMIN, None)
    } else {
        (ACTOR_TYPE_GENERATOR, auth_payload.generator)
    }
}

#[utoipa::path(
    post,
    path = "/templates",
    tag = "templates",
    request_body = CreateTemplateRequest,
    responses(
        (status = 201, description = "Template created", body = StackTemplate),
        (status = 400, description = "Invalid request - bad Tera syntax or validation error", body = ErrorResponse),
        (status = 403, description = "Forbidden", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse),
    ),
    security(("admin_pak" = []), ("generator_pak" = []))
)]
async fn create_template(
    State(dal): State<DAL>,
    Extension(auth_payload): Extension<AuthPayload>,
    Json(request): Json<CreateTemplateRequest>,
) -> Result<(StatusCode, Json<StackTemplate>), ApiError> {
    info!("Handling request to create a new template");

    let generator_id = if auth_payload.admin {
        None
    } else if let Some(gen_id) = auth_payload.generator {
        Some(gen_id)
    } else {
        warn!("Unauthorized attempt to create template");
        return Err(ApiError::forbidden(
            "templates_not_accessible",
            "admin or generator access required",
        ));
    };

    templating::validate_tera_syntax(&request.template_content)
        .map_err(|e| ApiError::bad_request("invalid_template_syntax", e.to_string()))?;
    templating::validate_json_schema(&request.parameters_schema)
        .map_err(|e| ApiError::bad_request("invalid_parameters_schema", e.to_string()))?;

    let template = dal
        .templates()
        .create_new_version(
            generator_id,
            request.name,
            request.description,
            request.template_content,
            request.parameters_schema,
        )
        .map_err(|e| {
            error!("Failed to create template: {:?}", e);
            ApiError::internal("failed to create template")
        })?;

    info!(
        "Successfully created template with ID: {} version: {}",
        template.id, template.version
    );
    let (actor_type, actor_id) = audit_actor(&auth_payload);
    audit::log_action(
        actor_type,
        actor_id,
        ACTION_TEMPLATE_CREATED,
        RESOURCE_TYPE_TEMPLATE,
        Some(template.id),
        Some(serde_json::json!({ "name": template.name, "version": template.version })),
        None,
        None,
    );
    Ok((StatusCode::CREATED, Json(template)))
}

#[utoipa::path(
    get,
    path = "/templates/{id}",
    tag = "templates",
    params(("id" = Uuid, Path, description = "Template ID")),
    responses(
        (status = 200, description = "Template found", body = StackTemplate),
        (status = 403, description = "Forbidden", body = ErrorResponse),
        (status = 404, description = "Template not found", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse),
    ),
    security(("admin_pak" = []), ("generator_pak" = []))
)]
async fn get_template(
    State(dal): State<DAL>,
    Extension(auth_payload): Extension<AuthPayload>,
    Path(id): Path<Uuid>,
) -> Result<Json<StackTemplate>, ApiError> {
    info!("Handling request to get template with ID: {}", id);
    let template = fetch_template_or_404(&dal, id)?;
    check_read_access(&auth_payload, &template)?;
    info!("Successfully retrieved template with ID: {}", id);
    Ok(Json(template))
}

#[utoipa::path(
    put,
    path = "/templates/{id}",
    tag = "templates",
    params(("id" = Uuid, Path, description = "Template ID")),
    request_body = UpdateTemplateRequest,
    responses(
        (status = 200, description = "New template version created", body = StackTemplate),
        (status = 400, description = "Invalid request - bad Tera syntax or validation error", body = ErrorResponse),
        (status = 403, description = "Forbidden", body = ErrorResponse),
        (status = 404, description = "Template not found", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse),
    ),
    security(("admin_pak" = []), ("generator_pak" = []))
)]
async fn update_template(
    State(dal): State<DAL>,
    Extension(auth_payload): Extension<AuthPayload>,
    Path(id): Path<Uuid>,
    Json(request): Json<UpdateTemplateRequest>,
) -> Result<Json<StackTemplate>, ApiError> {
    info!("Handling request to update template with ID: {}", id);

    let existing = fetch_template_or_404(&dal, id)?;
    if !can_modify_template(&auth_payload, &existing) {
        warn!("Unauthorized attempt to update template with ID: {}", id);
        return Err(ApiError::forbidden(
            "template_not_owned",
            "not authorized to modify this template",
        ));
    }

    templating::validate_tera_syntax(&request.template_content)
        .map_err(|e| ApiError::bad_request("invalid_template_syntax", e.to_string()))?;
    templating::validate_json_schema(&request.parameters_schema)
        .map_err(|e| ApiError::bad_request("invalid_parameters_schema", e.to_string()))?;

    let template = dal
        .templates()
        .create_new_version(
            existing.generator_id,
            existing.name,
            request.description.or(existing.description),
            request.template_content,
            request.parameters_schema,
        )
        .map_err(|e| {
            error!("Failed to create new template version: {:?}", e);
            ApiError::internal("failed to update template")
        })?;

    info!(
        "Successfully created new version {} for template: {}",
        template.version, template.name
    );
    let (actor_type, actor_id) = audit_actor(&auth_payload);
    audit::log_action(
        actor_type,
        actor_id,
        ACTION_TEMPLATE_UPDATED,
        RESOURCE_TYPE_TEMPLATE,
        Some(template.id),
        Some(serde_json::json!({ "name": template.name, "version": template.version })),
        None,
        None,
    );
    Ok(Json(template))
}

#[utoipa::path(
    delete,
    path = "/templates/{id}",
    tag = "templates",
    params(("id" = Uuid, Path, description = "Template ID")),
    responses(
        (status = 204, description = "Template deleted"),
        (status = 403, description = "Forbidden", body = ErrorResponse),
        (status = 404, description = "Template not found", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse),
    ),
    security(("admin_pak" = []), ("generator_pak" = []))
)]
async fn delete_template(
    State(dal): State<DAL>,
    Extension(auth_payload): Extension<AuthPayload>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, ApiError> {
    info!("Handling request to delete template with ID: {}", id);

    let existing = fetch_template_or_404(&dal, id)?;
    if !can_modify_template(&auth_payload, &existing) {
        warn!("Unauthorized attempt to delete template with ID: {}", id);
        return Err(ApiError::forbidden(
            "template_not_owned",
            "not authorized to delete this template",
        ));
    }

    dal.templates().soft_delete(id).map_err(|e| {
        error!("Failed to delete template with ID {}: {:?}", id, e);
        ApiError::internal("failed to delete template")
    })?;
    info!("Successfully deleted template with ID: {}", id);
    let (actor_type, actor_id) = audit_actor(&auth_payload);
    audit::log_action(
        actor_type,
        actor_id,
        ACTION_TEMPLATE_DELETED,
        RESOURCE_TYPE_TEMPLATE,
        Some(id),
        None,
        None,
        None,
    );
    Ok(StatusCode::NO_CONTENT)
}

#[utoipa::path(
    get,
    path = "/templates/{id}/labels",
    tag = "templates",
    params(("id" = Uuid, Path, description = "Template ID")),
    responses(
        (status = 200, description = "List of labels", body = Vec<TemplateLabel>),
        (status = 403, description = "Forbidden", body = ErrorResponse),
        (status = 404, description = "Template not found", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse),
    ),
    security(("admin_pak" = []), ("generator_pak" = []))
,
    operation_id = "templates_list_labels"
)]
async fn list_labels(
    State(dal): State<DAL>,
    Extension(auth_payload): Extension<AuthPayload>,
    Path(template_id): Path<Uuid>,
) -> Result<Json<Vec<TemplateLabel>>, ApiError> {
    let template = fetch_template_or_404(&dal, template_id)?;
    check_read_access(&auth_payload, &template)?;
    let labels = dal
        .template_labels()
        .list_for_template(template_id)
        .map_err(|_| ApiError::internal("failed to fetch template labels"))?;
    Ok(Json(labels))
}

#[utoipa::path(
    post,
    path = "/templates/{id}/labels",
    tag = "templates",
    params(("id" = Uuid, Path, description = "Template ID")),
    request_body(content = String, content_type = "application/json", description = "JSON-encoded label string, e.g. \"mylabel\""),
    responses(
        (status = 201, description = "Label added", body = TemplateLabel),
        (status = 400, description = "Invalid label", body = ErrorResponse),
        (status = 403, description = "Forbidden", body = ErrorResponse),
        (status = 404, description = "Template not found", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse),
    ),
    security(("admin_pak" = []), ("generator_pak" = []))
,
    operation_id = "templates_add_label"
)]
async fn add_label(
    State(dal): State<DAL>,
    Extension(auth_payload): Extension<AuthPayload>,
    Path(template_id): Path<Uuid>,
    Json(label): Json<String>,
) -> Result<(StatusCode, Json<TemplateLabel>), ApiError> {
    let template = fetch_template_or_404(&dal, template_id)?;
    if !can_modify_template(&auth_payload, &template) {
        return Err(ApiError::forbidden(
            "template_not_owned",
            "not authorized to modify this template",
        ));
    }
    let new_label = NewTemplateLabel::new(template_id, label)
        .map_err(|e| ApiError::bad_request("invalid_label", e))?;
    let label = dal
        .template_labels()
        .create(&new_label)
        .map_err(|_| ApiError::internal("failed to add template label"))?;
    Ok((StatusCode::CREATED, Json(label)))
}

#[utoipa::path(
    delete,
    path = "/templates/{id}/labels/{label}",
    tag = "templates",
    params(
        ("id" = Uuid, Path, description = "Template ID"),
        ("label" = String, Path, description = "Label to remove")
    ),
    responses(
        (status = 204, description = "Label removed"),
        (status = 403, description = "Forbidden", body = ErrorResponse),
        (status = 404, description = "Template or label not found", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse),
    ),
    security(("admin_pak" = []), ("generator_pak" = []))
,
    operation_id = "templates_remove_label"
)]
async fn remove_label(
    State(dal): State<DAL>,
    Extension(auth_payload): Extension<AuthPayload>,
    Path((template_id, label)): Path<(Uuid, String)>,
) -> Result<StatusCode, ApiError> {
    let template = fetch_template_or_404(&dal, template_id)?;
    if !can_modify_template(&auth_payload, &template) {
        return Err(ApiError::forbidden(
            "template_not_owned",
            "not authorized to modify this template",
        ));
    }
    let labels = dal
        .template_labels()
        .list_for_template(template_id)
        .map_err(|_| ApiError::internal("failed to fetch template labels"))?;
    let template_label = labels
        .into_iter()
        .find(|l| l.label == label)
        .ok_or_else(|| ApiError::not_found("template_label_not_found", "label not found"))?;
    dal.template_labels()
        .delete(template_label.id)
        .map_err(|_| ApiError::internal("failed to remove template label"))?;
    Ok(StatusCode::NO_CONTENT)
}

#[utoipa::path(
    get,
    path = "/templates/{id}/annotations",
    tag = "templates",
    params(("id" = Uuid, Path, description = "Template ID")),
    responses(
        (status = 200, description = "List of annotations", body = Vec<TemplateAnnotation>),
        (status = 403, description = "Forbidden", body = ErrorResponse),
        (status = 404, description = "Template not found", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse),
    ),
    security(("admin_pak" = []), ("generator_pak" = []))
,
    operation_id = "templates_list_annotations"
)]
async fn list_annotations(
    State(dal): State<DAL>,
    Extension(auth_payload): Extension<AuthPayload>,
    Path(template_id): Path<Uuid>,
) -> Result<Json<Vec<TemplateAnnotation>>, ApiError> {
    let template = fetch_template_or_404(&dal, template_id)?;
    check_read_access(&auth_payload, &template)?;
    let annotations = dal
        .template_annotations()
        .list_for_template(template_id)
        .map_err(|_| ApiError::internal("failed to fetch template annotations"))?;
    Ok(Json(annotations))
}

#[utoipa::path(
    post,
    path = "/templates/{id}/annotations",
    tag = "templates",
    params(("id" = Uuid, Path, description = "Template ID")),
    request_body = AddAnnotationRequest,
    responses(
        (status = 201, description = "Annotation added", body = TemplateAnnotation),
        (status = 400, description = "Invalid annotation", body = ErrorResponse),
        (status = 403, description = "Forbidden", body = ErrorResponse),
        (status = 404, description = "Template not found", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse),
    ),
    security(("admin_pak" = []), ("generator_pak" = []))
,
    operation_id = "templates_add_annotation"
)]
async fn add_annotation(
    State(dal): State<DAL>,
    Extension(auth_payload): Extension<AuthPayload>,
    Path(template_id): Path<Uuid>,
    Json(request): Json<AddAnnotationRequest>,
) -> Result<(StatusCode, Json<TemplateAnnotation>), ApiError> {
    let template = fetch_template_or_404(&dal, template_id)?;
    if !can_modify_template(&auth_payload, &template) {
        return Err(ApiError::forbidden(
            "template_not_owned",
            "not authorized to modify this template",
        ));
    }
    let new_annotation = NewTemplateAnnotation::new(template_id, request.key, request.value)
        .map_err(|e| ApiError::bad_request("invalid_annotation", e))?;
    let annotation = dal
        .template_annotations()
        .create(&new_annotation)
        .map_err(|_| ApiError::internal("failed to add template annotation"))?;
    Ok((StatusCode::CREATED, Json(annotation)))
}

#[utoipa::path(
    delete,
    path = "/templates/{id}/annotations/{key}",
    tag = "templates",
    params(
        ("id" = Uuid, Path, description = "Template ID"),
        ("key" = String, Path, description = "Annotation key to remove")
    ),
    responses(
        (status = 204, description = "Annotation removed"),
        (status = 403, description = "Forbidden", body = ErrorResponse),
        (status = 404, description = "Template or annotation not found", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse),
    ),
    security(("admin_pak" = []), ("generator_pak" = []))
,
    operation_id = "templates_remove_annotation"
)]
async fn remove_annotation(
    State(dal): State<DAL>,
    Extension(auth_payload): Extension<AuthPayload>,
    Path((template_id, key)): Path<(Uuid, String)>,
) -> Result<StatusCode, ApiError> {
    let template = fetch_template_or_404(&dal, template_id)?;
    if !can_modify_template(&auth_payload, &template) {
        return Err(ApiError::forbidden(
            "template_not_owned",
            "not authorized to modify this template",
        ));
    }
    let annotations = dal
        .template_annotations()
        .list_for_template(template_id)
        .map_err(|_| ApiError::internal("failed to fetch template annotations"))?;
    let annotation = annotations
        .into_iter()
        .find(|a| a.key == key)
        .ok_or_else(|| {
            ApiError::not_found("template_annotation_not_found", "annotation not found")
        })?;
    dal.template_annotations()
        .delete(annotation.id)
        .map_err(|_| ApiError::internal("failed to remove template annotation"))?;
    Ok(StatusCode::NO_CONTENT)
}
