/*
 * Copyright (c) 2025 Dylan Storey
 * Licensed under the Elastic License 2.0.
 * See LICENSE file in the project root for full license text.
 */

//! Webhooks API module for Brokkr.
//!
//! This module provides routes and handlers for managing webhook subscriptions,
//! including CRUD operations and delivery status inspection.

use crate::api::v1::middleware::AuthPayload;
use crate::dal::DAL;
use axum::http::StatusCode;
use axum::{
    extract::{Extension, Path, Query, State},
    routing::{delete, get, post, put},
    Json, Router,
};
use brokkr_models::models::webhooks::{
    NewWebhookSubscription, UpdateWebhookSubscription, WebhookDelivery, WebhookFilters,
    WebhookSubscription, VALID_EVENT_TYPES,
};
use brokkr_utils::logging::prelude::*;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

// =============================================================================
// Request/Response Types
// =============================================================================

/// Request body for creating a webhook subscription.
#[derive(Debug, Clone, Deserialize, ToSchema)]
pub struct CreateWebhookRequest {
    /// Human-readable name for the subscription.
    pub name: String,
    /// Webhook endpoint URL (will be encrypted at rest).
    pub url: String,
    /// Optional Authorization header value (will be encrypted at rest).
    #[serde(default)]
    pub auth_header: Option<String>,
    /// Event types to subscribe to (supports wildcards like "health.*").
    pub event_types: Vec<String>,
    /// Optional filters to narrow which events are delivered.
    #[serde(default)]
    pub filters: Option<WebhookFilters>,
    /// Maximum number of delivery retries (default: 5).
    #[serde(default)]
    pub max_retries: Option<i32>,
    /// HTTP timeout in seconds (default: 30).
    #[serde(default)]
    pub timeout_seconds: Option<i32>,
    /// Whether to validate the URL by sending a test request.
    #[serde(default)]
    pub validate: bool,
}

/// Request body for updating a webhook subscription.
#[derive(Debug, Clone, Deserialize, ToSchema)]
pub struct UpdateWebhookRequest {
    /// New name.
    #[serde(default)]
    pub name: Option<String>,
    /// New URL (will be encrypted at rest).
    #[serde(default)]
    pub url: Option<String>,
    /// New Authorization header (will be encrypted at rest).
    /// Use null to remove, omit to keep unchanged.
    #[serde(default)]
    pub auth_header: Option<Option<String>>,
    /// New event types.
    #[serde(default)]
    pub event_types: Option<Vec<String>>,
    /// New filters.
    #[serde(default)]
    pub filters: Option<Option<WebhookFilters>>,
    /// Enable/disable the subscription.
    #[serde(default)]
    pub enabled: Option<bool>,
    /// New max retries.
    #[serde(default)]
    pub max_retries: Option<i32>,
    /// New timeout.
    #[serde(default)]
    pub timeout_seconds: Option<i32>,
}

/// Response for a webhook subscription (safe view without encrypted fields).
#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct WebhookResponse {
    /// Unique identifier.
    pub id: Uuid,
    /// Human-readable name.
    pub name: String,
    /// Whether a URL is configured (actual value is encrypted).
    pub has_url: bool,
    /// Whether an auth header is configured (actual value is encrypted).
    pub has_auth_header: bool,
    /// Subscribed event types.
    pub event_types: Vec<String>,
    /// Configured filters.
    pub filters: Option<WebhookFilters>,
    /// Whether the subscription is active.
    pub enabled: bool,
    /// Maximum delivery retries.
    pub max_retries: i32,
    /// HTTP timeout in seconds.
    pub timeout_seconds: i32,
    /// When created.
    pub created_at: chrono::DateTime<chrono::Utc>,
    /// When last updated.
    pub updated_at: chrono::DateTime<chrono::Utc>,
    /// Who created this subscription.
    pub created_by: Option<String>,
}

impl From<WebhookSubscription> for WebhookResponse {
    fn from(sub: WebhookSubscription) -> Self {
        let filters = sub.filters.as_ref().and_then(|f| {
            serde_json::from_str(f).ok()
        });

        Self {
            id: sub.id,
            name: sub.name,
            has_url: !sub.url_encrypted.is_empty(),
            has_auth_header: sub.auth_header_encrypted.is_some(),
            event_types: sub.event_types.into_iter().flatten().collect(),
            filters,
            enabled: sub.enabled,
            max_retries: sub.max_retries,
            timeout_seconds: sub.timeout_seconds,
            created_at: sub.created_at,
            updated_at: sub.updated_at,
            created_by: sub.created_by,
        }
    }
}

/// Query parameters for listing deliveries.
#[derive(Debug, Clone, Deserialize, ToSchema)]
pub struct ListDeliveriesQuery {
    /// Filter by status (pending, success, failed, dead).
    #[serde(default)]
    pub status: Option<String>,
    /// Maximum number of results (default: 50).
    #[serde(default)]
    pub limit: Option<i64>,
    /// Offset for pagination.
    #[serde(default)]
    pub offset: Option<i64>,
}

// =============================================================================
// Encryption Helpers
// =============================================================================

use crate::utils::encryption;

/// Encrypts a value for storage.
fn encrypt_value(value: &str) -> Result<Vec<u8>, (StatusCode, Json<serde_json::Value>)> {
    encryption::encrypt_string(value).map_err(|e| {
        error!("Encryption failed: {}", e);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": "Failed to encrypt data"})),
        )
    })
}

/// Decrypts a stored value back to a string.
fn decrypt_value(encrypted: &[u8]) -> Result<String, String> {
    encryption::decrypt_string(encrypted)
}

// =============================================================================
// Routes
// =============================================================================

/// Creates and returns the router for webhook endpoints.
pub fn routes() -> Router<DAL> {
    info!("Setting up webhook routes");
    Router::new()
        .route("/webhooks", get(list_webhooks))
        .route("/webhooks", post(create_webhook))
        .route("/webhooks/event-types", get(list_event_types))
        .route("/webhooks/:id", get(get_webhook))
        .route("/webhooks/:id", put(update_webhook))
        .route("/webhooks/:id", delete(delete_webhook))
        .route("/webhooks/:id/deliveries", get(list_deliveries))
        .route("/webhooks/:id/test", post(test_webhook))
}

// =============================================================================
// Handlers
// =============================================================================

#[utoipa::path(
    get,
    path = "/api/v1/webhooks",
    responses(
        (status = 200, description = "List all webhook subscriptions", body = Vec<WebhookResponse>),
        (status = 403, description = "Forbidden - Admin access required"),
        (status = 500, description = "Internal server error")
    ),
    security(
        ("admin_pak" = [])
    ),
    tag = "webhooks"
)]
/// Lists all webhook subscriptions. Requires admin access.
async fn list_webhooks(
    State(dal): State<DAL>,
    Extension(auth_payload): Extension<AuthPayload>,
) -> Result<Json<Vec<WebhookResponse>>, (StatusCode, Json<serde_json::Value>)> {
    info!("Handling request to list webhook subscriptions");

    if !auth_payload.admin {
        warn!("Unauthorized attempt to list webhooks");
        return Err((
            StatusCode::FORBIDDEN,
            Json(serde_json::json!({"error": "Admin access required"})),
        ));
    }

    match dal.webhook_subscriptions().list(false) {
        Ok(subscriptions) => {
            info!("Successfully retrieved {} webhook subscriptions", subscriptions.len());
            let responses: Vec<WebhookResponse> = subscriptions.into_iter().map(Into::into).collect();
            Ok(Json(responses))
        }
        Err(e) => {
            error!("Failed to fetch webhook subscriptions: {:?}", e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": "Failed to fetch webhook subscriptions"})),
            ))
        }
    }
}

#[utoipa::path(
    get,
    path = "/api/v1/webhooks/event-types",
    responses(
        (status = 200, description = "List available event types", body = Vec<String>),
        (status = 403, description = "Forbidden - Admin access required")
    ),
    security(
        ("admin_pak" = [])
    ),
    tag = "webhooks"
)]
/// Lists all available event types for webhook subscriptions.
async fn list_event_types(
    Extension(auth_payload): Extension<AuthPayload>,
) -> Result<Json<Vec<&'static str>>, (StatusCode, Json<serde_json::Value>)> {
    if !auth_payload.admin {
        return Err((
            StatusCode::FORBIDDEN,
            Json(serde_json::json!({"error": "Admin access required"})),
        ));
    }

    Ok(Json(VALID_EVENT_TYPES.to_vec()))
}

#[utoipa::path(
    post,
    path = "/api/v1/webhooks",
    request_body = CreateWebhookRequest,
    responses(
        (status = 201, description = "Webhook subscription created", body = WebhookResponse),
        (status = 400, description = "Invalid request data"),
        (status = 403, description = "Forbidden - Admin access required"),
        (status = 500, description = "Internal server error")
    ),
    security(
        ("admin_pak" = [])
    ),
    tag = "webhooks"
)]
/// Creates a new webhook subscription. Requires admin access.
async fn create_webhook(
    State(dal): State<DAL>,
    Extension(auth_payload): Extension<AuthPayload>,
    Json(request): Json<CreateWebhookRequest>,
) -> Result<(StatusCode, Json<WebhookResponse>), (StatusCode, Json<serde_json::Value>)> {
    info!("Handling request to create webhook subscription");

    if !auth_payload.admin {
        warn!("Unauthorized attempt to create webhook");
        return Err((
            StatusCode::FORBIDDEN,
            Json(serde_json::json!({"error": "Admin access required"})),
        ));
    }

    // Validate URL
    if request.url.trim().is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({"error": "URL is required"})),
        ));
    }

    // Basic URL validation
    if !request.url.starts_with("http://") && !request.url.starts_with("https://") {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({"error": "URL must start with http:// or https://"})),
        ));
    }

    // Encrypt URL and auth header
    let url_encrypted = encrypt_value(&request.url)?;
    let auth_header_encrypted = match &request.auth_header {
        Some(h) => Some(encrypt_value(h)?),
        None => None,
    };

    // Determine who created this
    let created_by = if auth_payload.admin {
        Some("admin".to_string())
    } else {
        auth_payload.generator.map(|id| id.to_string())
    };

    // Build the new subscription
    let new_sub = match NewWebhookSubscription::new(
        request.name,
        url_encrypted,
        auth_header_encrypted,
        request.event_types,
        request.filters,
        created_by,
    ) {
        Ok(mut sub) => {
            // Apply optional settings
            if let Some(max_retries) = request.max_retries {
                sub.max_retries = max_retries;
            }
            if let Some(timeout) = request.timeout_seconds {
                sub.timeout_seconds = timeout;
            }
            sub
        }
        Err(e) => {
            return Err((
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({"error": e})),
            ));
        }
    };

    // Create in database
    match dal.webhook_subscriptions().create(&new_sub) {
        Ok(subscription) => {
            info!("Successfully created webhook subscription with ID: {}", subscription.id);
            Ok((StatusCode::CREATED, Json(subscription.into())))
        }
        Err(e) => {
            error!("Failed to create webhook subscription: {:?}", e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": "Failed to create webhook subscription"})),
            ))
        }
    }
}

#[utoipa::path(
    get,
    path = "/api/v1/webhooks/{id}",
    responses(
        (status = 200, description = "Get webhook subscription by ID", body = WebhookResponse),
        (status = 403, description = "Forbidden - Admin access required"),
        (status = 404, description = "Webhook subscription not found"),
        (status = 500, description = "Internal server error")
    ),
    params(
        ("id" = Uuid, Path, description = "Webhook subscription ID")
    ),
    security(
        ("admin_pak" = [])
    ),
    tag = "webhooks"
)]
/// Retrieves a specific webhook subscription by ID. Requires admin access.
async fn get_webhook(
    State(dal): State<DAL>,
    Extension(auth_payload): Extension<AuthPayload>,
    Path(id): Path<Uuid>,
) -> Result<Json<WebhookResponse>, (StatusCode, Json<serde_json::Value>)> {
    info!("Handling request to get webhook subscription with ID: {}", id);

    if !auth_payload.admin {
        warn!("Unauthorized attempt to access webhook with ID: {}", id);
        return Err((
            StatusCode::FORBIDDEN,
            Json(serde_json::json!({"error": "Admin access required"})),
        ));
    }

    match dal.webhook_subscriptions().get(id) {
        Ok(Some(subscription)) => {
            info!("Successfully retrieved webhook subscription with ID: {}", id);
            Ok(Json(subscription.into()))
        }
        Ok(None) => {
            warn!("Webhook subscription not found with ID: {}", id);
            Err((
                StatusCode::NOT_FOUND,
                Json(serde_json::json!({"error": "Webhook subscription not found"})),
            ))
        }
        Err(e) => {
            error!("Failed to fetch webhook subscription with ID {}: {:?}", id, e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": "Failed to fetch webhook subscription"})),
            ))
        }
    }
}

#[utoipa::path(
    put,
    path = "/api/v1/webhooks/{id}",
    request_body = UpdateWebhookRequest,
    responses(
        (status = 200, description = "Webhook subscription updated", body = WebhookResponse),
        (status = 400, description = "Invalid request data"),
        (status = 403, description = "Forbidden - Admin access required"),
        (status = 404, description = "Webhook subscription not found"),
        (status = 500, description = "Internal server error")
    ),
    params(
        ("id" = Uuid, Path, description = "Webhook subscription ID")
    ),
    security(
        ("admin_pak" = [])
    ),
    tag = "webhooks"
)]
/// Updates an existing webhook subscription. Requires admin access.
async fn update_webhook(
    State(dal): State<DAL>,
    Extension(auth_payload): Extension<AuthPayload>,
    Path(id): Path<Uuid>,
    Json(request): Json<UpdateWebhookRequest>,
) -> Result<Json<WebhookResponse>, (StatusCode, Json<serde_json::Value>)> {
    info!("Handling request to update webhook subscription with ID: {}", id);

    if !auth_payload.admin {
        warn!("Unauthorized attempt to update webhook with ID: {}", id);
        return Err((
            StatusCode::FORBIDDEN,
            Json(serde_json::json!({"error": "Admin access required"})),
        ));
    }

    // Verify it exists
    match dal.webhook_subscriptions().get(id) {
        Ok(None) => {
            return Err((
                StatusCode::NOT_FOUND,
                Json(serde_json::json!({"error": "Webhook subscription not found"})),
            ));
        }
        Err(e) => {
            error!("Failed to fetch webhook subscription: {:?}", e);
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": "Failed to fetch webhook subscription"})),
            ));
        }
        Ok(Some(_)) => {}
    }

    // Build changeset - handle encryption with proper error handling
    let url_encrypted = match request.url {
        Some(u) => Some(encrypt_value(&u)?),
        None => None,
    };
    let auth_header_encrypted = match request.auth_header {
        Some(Some(h)) => Some(Some(encrypt_value(&h)?)),
        Some(None) => Some(None), // Explicitly clear the auth header
        None => None,             // No change to auth header
    };

    let changeset = UpdateWebhookSubscription {
        name: request.name,
        url_encrypted,
        auth_header_encrypted,
        event_types: request.event_types.map(|types| types.into_iter().map(Some).collect()),
        filters: request.filters.map(|opt| {
            opt.map(|f| serde_json::to_string(&f).unwrap_or_default())
        }),
        enabled: request.enabled,
        max_retries: request.max_retries,
        timeout_seconds: request.timeout_seconds,
    };

    match dal.webhook_subscriptions().update(id, &changeset) {
        Ok(subscription) => {
            info!("Successfully updated webhook subscription with ID: {}", id);
            Ok(Json(subscription.into()))
        }
        Err(e) => {
            error!("Failed to update webhook subscription with ID {}: {:?}", id, e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": "Failed to update webhook subscription"})),
            ))
        }
    }
}

#[utoipa::path(
    delete,
    path = "/api/v1/webhooks/{id}",
    responses(
        (status = 204, description = "Webhook subscription deleted"),
        (status = 403, description = "Forbidden - Admin access required"),
        (status = 404, description = "Webhook subscription not found"),
        (status = 500, description = "Internal server error")
    ),
    params(
        ("id" = Uuid, Path, description = "Webhook subscription ID")
    ),
    security(
        ("admin_pak" = [])
    ),
    tag = "webhooks"
)]
/// Deletes a webhook subscription. Requires admin access.
async fn delete_webhook(
    State(dal): State<DAL>,
    Extension(auth_payload): Extension<AuthPayload>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, (StatusCode, Json<serde_json::Value>)> {
    info!("Handling request to delete webhook subscription with ID: {}", id);

    if !auth_payload.admin {
        warn!("Unauthorized attempt to delete webhook with ID: {}", id);
        return Err((
            StatusCode::FORBIDDEN,
            Json(serde_json::json!({"error": "Admin access required"})),
        ));
    }

    match dal.webhook_subscriptions().delete(id) {
        Ok(count) if count > 0 => {
            info!("Successfully deleted webhook subscription with ID: {}", id);
            Ok(StatusCode::NO_CONTENT)
        }
        Ok(_) => {
            warn!("Webhook subscription not found with ID: {}", id);
            Err((
                StatusCode::NOT_FOUND,
                Json(serde_json::json!({"error": "Webhook subscription not found"})),
            ))
        }
        Err(e) => {
            error!("Failed to delete webhook subscription with ID {}: {:?}", id, e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": "Failed to delete webhook subscription"})),
            ))
        }
    }
}

#[utoipa::path(
    get,
    path = "/api/v1/webhooks/{id}/deliveries",
    responses(
        (status = 200, description = "List deliveries for subscription", body = Vec<WebhookDelivery>),
        (status = 403, description = "Forbidden - Admin access required"),
        (status = 404, description = "Webhook subscription not found"),
        (status = 500, description = "Internal server error")
    ),
    params(
        ("id" = Uuid, Path, description = "Webhook subscription ID"),
        ("status" = Option<String>, Query, description = "Filter by delivery status"),
        ("limit" = Option<i64>, Query, description = "Maximum number of results"),
        ("offset" = Option<i64>, Query, description = "Offset for pagination")
    ),
    security(
        ("admin_pak" = [])
    ),
    tag = "webhooks"
)]
/// Lists deliveries for a specific webhook subscription. Requires admin access.
async fn list_deliveries(
    State(dal): State<DAL>,
    Extension(auth_payload): Extension<AuthPayload>,
    Path(id): Path<Uuid>,
    Query(query): Query<ListDeliveriesQuery>,
) -> Result<Json<Vec<WebhookDelivery>>, (StatusCode, Json<serde_json::Value>)> {
    info!("Handling request to list deliveries for webhook subscription: {}", id);

    if !auth_payload.admin {
        warn!("Unauthorized attempt to list deliveries for webhook: {}", id);
        return Err((
            StatusCode::FORBIDDEN,
            Json(serde_json::json!({"error": "Admin access required"})),
        ));
    }

    // Verify subscription exists
    match dal.webhook_subscriptions().get(id) {
        Ok(None) => {
            return Err((
                StatusCode::NOT_FOUND,
                Json(serde_json::json!({"error": "Webhook subscription not found"})),
            ));
        }
        Err(e) => {
            error!("Failed to fetch webhook subscription: {:?}", e);
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": "Failed to fetch webhook subscription"})),
            ));
        }
        Ok(Some(_)) => {}
    }

    let limit = query.limit.unwrap_or(50);
    let offset = query.offset.unwrap_or(0);

    match dal.webhook_deliveries().list_for_subscription(id, query.status.as_deref(), limit, offset) {
        Ok(deliveries) => {
            info!("Successfully retrieved {} deliveries for subscription {}", deliveries.len(), id);
            Ok(Json(deliveries))
        }
        Err(e) => {
            error!("Failed to fetch deliveries for subscription {}: {:?}", id, e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": "Failed to fetch deliveries"})),
            ))
        }
    }
}

#[utoipa::path(
    post,
    path = "/api/v1/webhooks/{id}/test",
    responses(
        (status = 200, description = "Test delivery successful"),
        (status = 400, description = "Test delivery failed"),
        (status = 403, description = "Forbidden - Admin access required"),
        (status = 404, description = "Webhook subscription not found"),
        (status = 500, description = "Internal server error")
    ),
    params(
        ("id" = Uuid, Path, description = "Webhook subscription ID")
    ),
    security(
        ("admin_pak" = [])
    ),
    tag = "webhooks"
)]
/// Sends a test event to the webhook endpoint. Requires admin access.
async fn test_webhook(
    State(dal): State<DAL>,
    Extension(auth_payload): Extension<AuthPayload>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    info!("Handling request to test webhook subscription with ID: {}", id);

    if !auth_payload.admin {
        warn!("Unauthorized attempt to test webhook with ID: {}", id);
        return Err((
            StatusCode::FORBIDDEN,
            Json(serde_json::json!({"error": "Admin access required"})),
        ));
    }

    // Get the subscription
    let subscription = match dal.webhook_subscriptions().get(id) {
        Ok(Some(sub)) => sub,
        Ok(None) => {
            return Err((
                StatusCode::NOT_FOUND,
                Json(serde_json::json!({"error": "Webhook subscription not found"})),
            ));
        }
        Err(e) => {
            error!("Failed to fetch webhook subscription: {:?}", e);
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": "Failed to fetch webhook subscription"})),
            ));
        }
    };

    // Decrypt URL and auth header
    let url = match decrypt_value(&subscription.url_encrypted) {
        Ok(u) => u,
        Err(e) => {
            error!("Failed to decrypt URL: {}", e);
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": "Failed to decrypt webhook URL"})),
            ));
        }
    };

    let auth_header = subscription
        .auth_header_encrypted
        .as_ref()
        .map(|h| decrypt_value(h))
        .transpose()
        .map_err(|e| {
            error!("Failed to decrypt auth header: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": "Failed to decrypt auth header"})),
            )
        })?;

    // Create test payload
    let test_event = serde_json::json!({
        "id": Uuid::new_v4(),
        "event_type": "webhook.test",
        "timestamp": chrono::Utc::now(),
        "data": {
            "message": "This is a test webhook delivery from Brokkr",
            "subscription_id": id
        }
    });

    // Send test request
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(subscription.timeout_seconds as u64))
        .build()
        .map_err(|e| {
            error!("Failed to create HTTP client: {:?}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": "Failed to create HTTP client"})),
            )
        })?;

    let mut request = client
        .post(&url)
        .header("Content-Type", "application/json")
        .json(&test_event);

    if let Some(auth) = &auth_header {
        request = request.header("Authorization", auth);
    }

    match request.send().await {
        Ok(response) => {
            let status = response.status();
            if status.is_success() {
                info!("Test webhook delivery succeeded for subscription {}", id);
                Ok(Json(serde_json::json!({
                    "success": true,
                    "status_code": status.as_u16(),
                    "message": "Test delivery successful"
                })))
            } else {
                let body = response.text().await.unwrap_or_default();
                warn!("Test webhook delivery failed with status {}: {}", status, body);
                Err((
                    StatusCode::BAD_REQUEST,
                    Json(serde_json::json!({
                        "success": false,
                        "status_code": status.as_u16(),
                        "error": format!("Endpoint returned HTTP {}", status),
                        "body": body.chars().take(500).collect::<String>()
                    })),
                ))
            }
        }
        Err(e) => {
            error!("Test webhook delivery failed: {:?}", e);
            Err((
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({
                    "success": false,
                    "error": format!("Request failed: {}", e)
                })),
            ))
        }
    }
}
