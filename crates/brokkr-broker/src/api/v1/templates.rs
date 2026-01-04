/*
 * Copyright (c) 2025 Dylan Storey
 * Licensed under the Elastic License 2.0.
 * See LICENSE file in the project root for full license text.
 */

//! API endpoints for stack template management.
//!
//! This module provides REST API endpoints for creating, reading, updating, and deleting
//! stack templates, as well as managing template labels and annotations.

use crate::api::v1::middleware::AuthPayload;
use crate::dal::DAL;
use crate::utils::templating;
use axum::{
    extract::{Extension, Path, State},
    http::StatusCode,
    routing::{delete, get},
    Json, Router,
};
use brokkr_models::models::stack_templates::StackTemplate;
use brokkr_models::models::template_annotations::{NewTemplateAnnotation, TemplateAnnotation};
use brokkr_models::models::template_labels::{NewTemplateLabel, TemplateLabel};
use tracing::{debug, error, info, warn};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

/// Request body for creating a new template.
#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub struct CreateTemplateRequest {
    /// Name of the template.
    pub name: String,
    /// Optional description.
    pub description: Option<String>,
    /// Tera template content.
    pub template_content: String,
    /// JSON Schema for parameter validation.
    pub parameters_schema: String,
}

/// Request body for updating a template (creates new version).
#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub struct UpdateTemplateRequest {
    /// Optional new description.
    pub description: Option<String>,
    /// Tera template content.
    pub template_content: String,
    /// JSON Schema for parameter validation.
    pub parameters_schema: String,
}

/// Sets up the routes for template management.
pub fn routes() -> Router<DAL> {
    info!("Setting up template routes");
    Router::new()
        .route("/templates", get(list_templates).post(create_template))
        .route(
            "/templates/:id",
            get(get_template).put(update_template).delete(delete_template),
        )
        .route(
            "/templates/:id/labels",
            get(list_labels).post(add_label),
        )
        .route("/templates/:id/labels/:label", delete(remove_label))
        .route(
            "/templates/:id/annotations",
            get(list_annotations).post(add_annotation),
        )
        .route("/templates/:id/annotations/:key", delete(remove_annotation))
}

/// Checks if the authenticated user can modify the given template.
///
/// System templates (generator_id = NULL) require admin access.
/// Generator templates can be modified by admin or the owning generator.
fn can_modify_template(auth: &AuthPayload, template: &StackTemplate) -> bool {
    if auth.admin {
        return true;
    }
    match (auth.generator, template.generator_id) {
        (Some(auth_gen), Some(tmpl_gen)) => auth_gen == tmpl_gen,
        _ => false,
    }
}


/// Lists all templates.
///
/// # Authorization
/// - Admin: sees all templates
/// - Generator: sees system templates and own templates
#[utoipa::path(
    get,
    path = "/api/v1/templates",
    tag = "templates",
    responses(
        (status = 200, description = "List of templates", body = Vec<StackTemplate>),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
    ),
    security(
        ("pak" = [])
    )
)]
async fn list_templates(
    State(dal): State<DAL>,
    Extension(auth_payload): Extension<AuthPayload>,
) -> Result<Json<Vec<StackTemplate>>, (StatusCode, Json<serde_json::Value>)> {
    info!("Handling request to list templates");

    let templates = if auth_payload.admin {
        // Admin sees all templates
        dal.templates().list()
    } else if let Some(generator_id) = auth_payload.generator {
        // Generator sees system templates + own templates
        let system = dal.templates().list_system_templates().map_err(|e| {
            error!("Failed to fetch system templates: {:?}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": "Failed to fetch templates"})),
            )
        })?;

        let own = dal.templates().list_for_generator(generator_id).map_err(|e| {
            error!("Failed to fetch generator templates: {:?}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": "Failed to fetch templates"})),
            )
        })?;

        let mut all = system;
        all.extend(own);
        Ok(all)
    } else {
        warn!("Unauthorized attempt to list templates");
        return Err((
            StatusCode::FORBIDDEN,
            Json(serde_json::json!({"error": "Admin or generator access required"})),
        ));
    };

    match templates {
        Ok(templates) => {
            info!("Successfully retrieved {} templates", templates.len());
            Ok(Json(templates))
        }
        Err(e) => {
            error!("Failed to fetch templates: {:?}", e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": "Failed to fetch templates"})),
            ))
        }
    }
}

/// Creates a new template.
///
/// # Authorization
/// - Admin: can create system templates (generator_id = NULL)
/// - Generator: creates templates under their own generator_id
#[utoipa::path(
    post,
    path = "/api/v1/templates",
    tag = "templates",
    request_body = CreateTemplateRequest,
    responses(
        (status = 201, description = "Template created", body = StackTemplate),
        (status = 400, description = "Invalid request - bad Tera syntax or validation error"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
    ),
    security(
        ("pak" = [])
    )
)]
async fn create_template(
    State(dal): State<DAL>,
    Extension(auth_payload): Extension<AuthPayload>,
    Json(request): Json<CreateTemplateRequest>,
) -> Result<(StatusCode, Json<StackTemplate>), (StatusCode, Json<serde_json::Value>)> {
    info!("Handling request to create a new template");

    // Determine generator_id based on auth
    let generator_id = if auth_payload.admin {
        // Admin can create system templates (None) or specify a generator
        None
    } else if let Some(gen_id) = auth_payload.generator {
        Some(gen_id)
    } else {
        warn!("Unauthorized attempt to create template");
        return Err((
            StatusCode::FORBIDDEN,
            Json(serde_json::json!({"error": "Admin or generator access required"})),
        ));
    };

    // Validate Tera syntax
    if let Err(e) = templating::validate_tera_syntax(&request.template_content) {
        warn!("Template validation failed: {}", e);
        return Err((
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({"error": e.to_string()})),
        ));
    }

    // Validate JSON Schema
    if let Err(e) = templating::validate_json_schema(&request.parameters_schema) {
        warn!("JSON Schema validation failed: {}", e);
        return Err((
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({"error": e.to_string()})),
        ));
    }

    // Create new version (auto-increments version number)
    match dal.templates().create_new_version(
        generator_id,
        request.name,
        request.description,
        request.template_content,
        request.parameters_schema,
    ) {
        Ok(template) => {
            info!(
                "Successfully created template with ID: {} version: {}",
                template.id, template.version
            );
            Ok((StatusCode::CREATED, Json(template)))
        }
        Err(e) => {
            error!("Failed to create template: {:?}", e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": "Failed to create template"})),
            ))
        }
    }
}

/// Gets a template by ID.
#[utoipa::path(
    get,
    path = "/api/v1/templates/{id}",
    tag = "templates",
    params(
        ("id" = Uuid, Path, description = "Template ID")
    ),
    responses(
        (status = 200, description = "Template found", body = StackTemplate),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Template not found"),
    ),
    security(
        ("pak" = [])
    )
)]
async fn get_template(
    State(dal): State<DAL>,
    Extension(auth_payload): Extension<AuthPayload>,
    Path(id): Path<Uuid>,
) -> Result<Json<StackTemplate>, (StatusCode, Json<serde_json::Value>)> {
    info!("Handling request to get template with ID: {}", id);

    let template = dal.templates().get(id).map_err(|e| {
        error!("Failed to fetch template with ID {}: {:?}", id, e);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": "Failed to fetch template"})),
        )
    })?;

    let template = match template {
        Some(t) => t,
        None => {
            warn!("Template not found with ID: {}", id);
            return Err((
                StatusCode::NOT_FOUND,
                Json(serde_json::json!({"error": "Template not found"})),
            ));
        }
    };

    // Check read access
    if !auth_payload.admin {
        match (auth_payload.generator, template.generator_id) {
            // Generator can read system templates or own templates
            (Some(auth_gen), Some(tmpl_gen)) if auth_gen != tmpl_gen => {
                warn!("Unauthorized attempt to access template with ID: {}", id);
                return Err((
                    StatusCode::FORBIDDEN,
                    Json(serde_json::json!({"error": "Access denied"})),
                ));
            }
            (None, _) => {
                warn!("Unauthorized attempt to access template with ID: {}", id);
                return Err((
                    StatusCode::FORBIDDEN,
                    Json(serde_json::json!({"error": "Access denied"})),
                ));
            }
            _ => {} // System template (None) or own template - allow
        }
    }

    info!("Successfully retrieved template with ID: {}", id);
    Ok(Json(template))
}

/// Updates a template by creating a new version.
///
/// Templates are immutable - updating creates a new version with the same name.
#[utoipa::path(
    put,
    path = "/api/v1/templates/{id}",
    tag = "templates",
    params(
        ("id" = Uuid, Path, description = "Template ID")
    ),
    request_body = UpdateTemplateRequest,
    responses(
        (status = 200, description = "New template version created", body = StackTemplate),
        (status = 400, description = "Invalid request - bad Tera syntax or validation error"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Template not found"),
    ),
    security(
        ("pak" = [])
    )
)]
async fn update_template(
    State(dal): State<DAL>,
    Extension(auth_payload): Extension<AuthPayload>,
    Path(id): Path<Uuid>,
    Json(request): Json<UpdateTemplateRequest>,
) -> Result<Json<StackTemplate>, (StatusCode, Json<serde_json::Value>)> {
    info!("Handling request to update template with ID: {}", id);

    // Fetch existing template
    let existing = dal.templates().get(id).map_err(|e| {
        error!("Failed to fetch template with ID {}: {:?}", id, e);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": "Failed to fetch template"})),
        )
    })?;

    let existing = match existing {
        Some(t) => t,
        None => {
            warn!("Template not found with ID: {}", id);
            return Err((
                StatusCode::NOT_FOUND,
                Json(serde_json::json!({"error": "Template not found"})),
            ));
        }
    };

    // Check authorization
    if !can_modify_template(&auth_payload, &existing) {
        warn!("Unauthorized attempt to update template with ID: {}", id);
        return Err((
            StatusCode::FORBIDDEN,
            Json(serde_json::json!({"error": "Access denied"})),
        ));
    }

    // Validate Tera syntax
    if let Err(e) = templating::validate_tera_syntax(&request.template_content) {
        warn!("Template validation failed: {}", e);
        return Err((
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({"error": e.to_string()})),
        ));
    }

    // Validate JSON Schema
    if let Err(e) = templating::validate_json_schema(&request.parameters_schema) {
        warn!("JSON Schema validation failed: {}", e);
        return Err((
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({"error": e.to_string()})),
        ));
    }

    // Create new version with same name and generator_id
    match dal.templates().create_new_version(
        existing.generator_id,
        existing.name,
        request.description.or(existing.description),
        request.template_content,
        request.parameters_schema,
    ) {
        Ok(template) => {
            info!(
                "Successfully created new version {} for template: {}",
                template.version, template.name
            );
            Ok(Json(template))
        }
        Err(e) => {
            error!("Failed to create new template version: {:?}", e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": "Failed to update template"})),
            ))
        }
    }
}

/// Deletes a template (soft delete).
#[utoipa::path(
    delete,
    path = "/api/v1/templates/{id}",
    tag = "templates",
    params(
        ("id" = Uuid, Path, description = "Template ID")
    ),
    responses(
        (status = 204, description = "Template deleted"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Template not found"),
    ),
    security(
        ("pak" = [])
    )
)]
async fn delete_template(
    State(dal): State<DAL>,
    Extension(auth_payload): Extension<AuthPayload>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, (StatusCode, Json<serde_json::Value>)> {
    info!("Handling request to delete template with ID: {}", id);

    // Fetch existing template
    let existing = dal.templates().get(id).map_err(|e| {
        error!("Failed to fetch template with ID {}: {:?}", id, e);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": "Failed to fetch template"})),
        )
    })?;

    let existing = match existing {
        Some(t) => t,
        None => {
            warn!("Template not found with ID: {}", id);
            return Err((
                StatusCode::NOT_FOUND,
                Json(serde_json::json!({"error": "Template not found"})),
            ));
        }
    };

    // Check authorization
    if !can_modify_template(&auth_payload, &existing) {
        warn!("Unauthorized attempt to delete template with ID: {}", id);
        return Err((
            StatusCode::FORBIDDEN,
            Json(serde_json::json!({"error": "Access denied"})),
        ));
    }

    match dal.templates().soft_delete(id) {
        Ok(_) => {
            info!("Successfully deleted template with ID: {}", id);
            Ok(StatusCode::NO_CONTENT)
        }
        Err(e) => {
            error!("Failed to delete template with ID {}: {:?}", id, e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": "Failed to delete template"})),
            ))
        }
    }
}

/// Lists labels for a template.
#[utoipa::path(
    get,
    path = "/api/v1/templates/{id}/labels",
    tag = "templates",
    params(
        ("id" = Uuid, Path, description = "Template ID")
    ),
    responses(
        (status = 200, description = "List of labels", body = Vec<TemplateLabel>),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Template not found"),
    ),
    security(
        ("pak" = [])
    )
)]
async fn list_labels(
    State(dal): State<DAL>,
    Extension(auth_payload): Extension<AuthPayload>,
    Path(template_id): Path<Uuid>,
) -> Result<Json<Vec<TemplateLabel>>, (StatusCode, Json<serde_json::Value>)> {
    // Check if template exists and user has access
    let template = dal.templates().get(template_id).map_err(|e| {
        error!("Failed to fetch template: {:?}", e);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": "Failed to fetch template"})),
        )
    })?;

    let template = match template {
        Some(t) => t,
        None => {
            return Err((
                StatusCode::NOT_FOUND,
                Json(serde_json::json!({"error": "Template not found"})),
            ))
        }
    };

    // Check read access (same as get_template)
    if !auth_payload.admin {
        match (auth_payload.generator, template.generator_id) {
            (Some(auth_gen), Some(tmpl_gen)) if auth_gen != tmpl_gen => {
                return Err((
                    StatusCode::FORBIDDEN,
                    Json(serde_json::json!({"error": "Access denied"})),
                ))
            }
            (None, _) => {
                return Err((
                    StatusCode::FORBIDDEN,
                    Json(serde_json::json!({"error": "Access denied"})),
                ))
            }
            _ => {}
        }
    }

    match dal.template_labels().list_for_template(template_id) {
        Ok(labels) => Ok(Json(labels)),
        Err(_) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": "Failed to fetch template labels"})),
        )),
    }
}

/// Adds a label to a template.
#[utoipa::path(
    post,
    path = "/api/v1/templates/{id}/labels",
    tag = "templates",
    params(
        ("id" = Uuid, Path, description = "Template ID")
    ),
    request_body = String,
    responses(
        (status = 200, description = "Label added", body = TemplateLabel),
        (status = 400, description = "Invalid label"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Template not found"),
    ),
    security(
        ("pak" = [])
    )
)]
async fn add_label(
    State(dal): State<DAL>,
    Extension(auth_payload): Extension<AuthPayload>,
    Path(template_id): Path<Uuid>,
    Json(label): Json<String>,
) -> Result<Json<TemplateLabel>, (StatusCode, Json<serde_json::Value>)> {
    // Check if template exists and user can modify
    let template = dal.templates().get(template_id).map_err(|e| {
        error!("Failed to fetch template: {:?}", e);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": "Failed to fetch template"})),
        )
    })?;

    let template = match template {
        Some(t) => t,
        None => {
            return Err((
                StatusCode::NOT_FOUND,
                Json(serde_json::json!({"error": "Template not found"})),
            ))
        }
    };

    if !can_modify_template(&auth_payload, &template) {
        return Err((
            StatusCode::FORBIDDEN,
            Json(serde_json::json!({"error": "Access denied"})),
        ));
    }

    let new_label = NewTemplateLabel::new(template_id, label).map_err(|e| {
        (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({"error": e})),
        )
    })?;

    match dal.template_labels().create(&new_label) {
        Ok(label) => Ok(Json(label)),
        Err(_) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": "Failed to add template label"})),
        )),
    }
}

/// Removes a label from a template.
#[utoipa::path(
    delete,
    path = "/api/v1/templates/{id}/labels/{label}",
    tag = "templates",
    params(
        ("id" = Uuid, Path, description = "Template ID"),
        ("label" = String, Path, description = "Label to remove")
    ),
    responses(
        (status = 204, description = "Label removed"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Template or label not found"),
    ),
    security(
        ("pak" = [])
    )
)]
async fn remove_label(
    State(dal): State<DAL>,
    Extension(auth_payload): Extension<AuthPayload>,
    Path((template_id, label)): Path<(Uuid, String)>,
) -> Result<StatusCode, (StatusCode, Json<serde_json::Value>)> {
    // Check if template exists and user can modify
    let template = dal.templates().get(template_id).map_err(|e| {
        error!("Failed to fetch template: {:?}", e);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": "Failed to fetch template"})),
        )
    })?;

    let template = match template {
        Some(t) => t,
        None => {
            return Err((
                StatusCode::NOT_FOUND,
                Json(serde_json::json!({"error": "Template not found"})),
            ))
        }
    };

    if !can_modify_template(&auth_payload, &template) {
        return Err((
            StatusCode::FORBIDDEN,
            Json(serde_json::json!({"error": "Access denied"})),
        ));
    }

    // Find and delete the label
    match dal.template_labels().list_for_template(template_id) {
        Ok(labels) => {
            if let Some(template_label) = labels.into_iter().find(|l| l.label == label) {
                match dal.template_labels().delete(template_label.id) {
                    Ok(_) => Ok(StatusCode::NO_CONTENT),
                    Err(_) => Err((
                        StatusCode::INTERNAL_SERVER_ERROR,
                        Json(serde_json::json!({"error": "Failed to remove template label"})),
                    )),
                }
            } else {
                Err((
                    StatusCode::NOT_FOUND,
                    Json(serde_json::json!({"error": "Label not found"})),
                ))
            }
        }
        Err(_) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": "Failed to fetch template labels"})),
        )),
    }
}

/// Lists annotations for a template.
#[utoipa::path(
    get,
    path = "/api/v1/templates/{id}/annotations",
    tag = "templates",
    params(
        ("id" = Uuid, Path, description = "Template ID")
    ),
    responses(
        (status = 200, description = "List of annotations", body = Vec<TemplateAnnotation>),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Template not found"),
    ),
    security(
        ("pak" = [])
    )
)]
async fn list_annotations(
    State(dal): State<DAL>,
    Extension(auth_payload): Extension<AuthPayload>,
    Path(template_id): Path<Uuid>,
) -> Result<Json<Vec<TemplateAnnotation>>, (StatusCode, Json<serde_json::Value>)> {
    // Check if template exists and user has access
    let template = dal.templates().get(template_id).map_err(|e| {
        error!("Failed to fetch template: {:?}", e);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": "Failed to fetch template"})),
        )
    })?;

    let template = match template {
        Some(t) => t,
        None => {
            return Err((
                StatusCode::NOT_FOUND,
                Json(serde_json::json!({"error": "Template not found"})),
            ))
        }
    };

    // Check read access
    if !auth_payload.admin {
        match (auth_payload.generator, template.generator_id) {
            (Some(auth_gen), Some(tmpl_gen)) if auth_gen != tmpl_gen => {
                return Err((
                    StatusCode::FORBIDDEN,
                    Json(serde_json::json!({"error": "Access denied"})),
                ))
            }
            (None, _) => {
                return Err((
                    StatusCode::FORBIDDEN,
                    Json(serde_json::json!({"error": "Access denied"})),
                ))
            }
            _ => {}
        }
    }

    match dal.template_annotations().list_for_template(template_id) {
        Ok(annotations) => Ok(Json(annotations)),
        Err(_) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": "Failed to fetch template annotations"})),
        )),
    }
}

/// Request body for adding an annotation.
#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub struct AddAnnotationRequest {
    /// Annotation key.
    pub key: String,
    /// Annotation value.
    pub value: String,
}

/// Adds an annotation to a template.
#[utoipa::path(
    post,
    path = "/api/v1/templates/{id}/annotations",
    tag = "templates",
    params(
        ("id" = Uuid, Path, description = "Template ID")
    ),
    request_body = AddAnnotationRequest,
    responses(
        (status = 200, description = "Annotation added", body = TemplateAnnotation),
        (status = 400, description = "Invalid annotation"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Template not found"),
    ),
    security(
        ("pak" = [])
    )
)]
async fn add_annotation(
    State(dal): State<DAL>,
    Extension(auth_payload): Extension<AuthPayload>,
    Path(template_id): Path<Uuid>,
    Json(request): Json<AddAnnotationRequest>,
) -> Result<Json<TemplateAnnotation>, (StatusCode, Json<serde_json::Value>)> {
    // Check if template exists and user can modify
    let template = dal.templates().get(template_id).map_err(|e| {
        error!("Failed to fetch template: {:?}", e);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": "Failed to fetch template"})),
        )
    })?;

    let template = match template {
        Some(t) => t,
        None => {
            return Err((
                StatusCode::NOT_FOUND,
                Json(serde_json::json!({"error": "Template not found"})),
            ))
        }
    };

    if !can_modify_template(&auth_payload, &template) {
        return Err((
            StatusCode::FORBIDDEN,
            Json(serde_json::json!({"error": "Access denied"})),
        ));
    }

    let new_annotation =
        NewTemplateAnnotation::new(template_id, request.key, request.value).map_err(|e| {
            (
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({"error": e})),
            )
        })?;

    match dal.template_annotations().create(&new_annotation) {
        Ok(annotation) => Ok(Json(annotation)),
        Err(_) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": "Failed to add template annotation"})),
        )),
    }
}

/// Removes an annotation from a template.
#[utoipa::path(
    delete,
    path = "/api/v1/templates/{id}/annotations/{key}",
    tag = "templates",
    params(
        ("id" = Uuid, Path, description = "Template ID"),
        ("key" = String, Path, description = "Annotation key to remove")
    ),
    responses(
        (status = 204, description = "Annotation removed"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Template or annotation not found"),
    ),
    security(
        ("pak" = [])
    )
)]
async fn remove_annotation(
    State(dal): State<DAL>,
    Extension(auth_payload): Extension<AuthPayload>,
    Path((template_id, key)): Path<(Uuid, String)>,
) -> Result<StatusCode, (StatusCode, Json<serde_json::Value>)> {
    // Check if template exists and user can modify
    let template = dal.templates().get(template_id).map_err(|e| {
        error!("Failed to fetch template: {:?}", e);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": "Failed to fetch template"})),
        )
    })?;

    let template = match template {
        Some(t) => t,
        None => {
            return Err((
                StatusCode::NOT_FOUND,
                Json(serde_json::json!({"error": "Template not found"})),
            ))
        }
    };

    if !can_modify_template(&auth_payload, &template) {
        return Err((
            StatusCode::FORBIDDEN,
            Json(serde_json::json!({"error": "Access denied"})),
        ));
    }

    // Find and delete the annotation
    match dal.template_annotations().list_for_template(template_id) {
        Ok(annotations) => {
            if let Some(annotation) = annotations.into_iter().find(|a| a.key == key) {
                match dal.template_annotations().delete(annotation.id) {
                    Ok(_) => Ok(StatusCode::NO_CONTENT),
                    Err(_) => Err((
                        StatusCode::INTERNAL_SERVER_ERROR,
                        Json(serde_json::json!({"error": "Failed to remove template annotation"})),
                    )),
                }
            } else {
                Err((
                    StatusCode::NOT_FOUND,
                    Json(serde_json::json!({"error": "Annotation not found"})),
                ))
            }
        }
        Err(_) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": "Failed to fetch template annotations"})),
        )),
    }
}
