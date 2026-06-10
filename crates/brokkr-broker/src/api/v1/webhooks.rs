/*
 * Copyright (c) 2025-2026 Dylan Storey
 * Licensed under the Elastic License 2.0.
 * See LICENSE file in the project root for full license text.
 */

//! Webhooks API module for Brokkr.

use crate::api::v1::error::{ApiError, ErrorResponse};
use crate::api::v1::middleware::AuthPayload;
use crate::dal::DAL;
use crate::utils::{audit, encryption};
use axum::http::StatusCode;
use axum::{
    Json, Router,
    extract::{Extension, Path, Query, State},
    routing::{delete, get, post, put},
};
use brokkr_models::models::audit_logs::{
    ACTION_WEBHOOK_CREATED, ACTION_WEBHOOK_DELETED, ACTION_WEBHOOK_DELIVERY_FAILED,
    ACTION_WEBHOOK_UPDATED, ACTOR_TYPE_ADMIN, ACTOR_TYPE_SYSTEM, RESOURCE_TYPE_WEBHOOK,
};
use brokkr_models::models::webhooks::{
    NewWebhookSubscription, UpdateWebhookSubscription, VALID_EVENT_TYPES, WebhookDelivery,
    WebhookFilters, WebhookSubscription,
};
use serde::{Deserialize, Serialize};
use tracing::{debug, error, info, warn};
use utoipa::ToSchema;
use uuid::Uuid;

// =============================================================================
// Request/Response Types
// =============================================================================

#[derive(Debug, Clone, Deserialize, ToSchema)]
pub struct CreateWebhookRequest {
    pub name: String,
    pub url: String,
    #[serde(default)]
    pub auth_header: Option<String>,
    pub event_types: Vec<String>,
    #[serde(default)]
    pub filters: Option<WebhookFilters>,
    #[serde(default)]
    pub max_retries: Option<i32>,
    #[serde(default)]
    pub timeout_seconds: Option<i32>,
    #[serde(default)]
    pub validate: bool,
    #[serde(default)]
    pub target_labels: Option<Vec<String>>,
}

#[derive(Debug, Clone, Deserialize, ToSchema)]
pub struct UpdateWebhookRequest {
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub url: Option<String>,
    #[serde(default)]
    pub auth_header: Option<Option<String>>,
    #[serde(default)]
    pub event_types: Option<Vec<String>>,
    #[serde(default)]
    pub filters: Option<Option<WebhookFilters>>,
    #[serde(default)]
    pub enabled: Option<bool>,
    #[serde(default)]
    pub max_retries: Option<i32>,
    #[serde(default)]
    pub timeout_seconds: Option<i32>,
    #[serde(default)]
    pub target_labels: Option<Option<Vec<String>>>,
}

#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct WebhookResponse {
    pub id: Uuid,
    pub name: String,
    pub has_url: bool,
    pub has_auth_header: bool,
    pub event_types: Vec<String>,
    pub filters: Option<WebhookFilters>,
    pub target_labels: Option<Vec<String>>,
    pub enabled: bool,
    pub max_retries: i32,
    pub timeout_seconds: i32,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
    pub created_by: Option<String>,
}

impl From<WebhookSubscription> for WebhookResponse {
    fn from(sub: WebhookSubscription) -> Self {
        let filters = sub
            .filters
            .as_ref()
            .and_then(|f| serde_json::from_str(f).ok());
        let target_labels = sub
            .target_labels
            .map(|labels| labels.into_iter().flatten().collect());
        Self {
            id: sub.id,
            name: sub.name,
            has_url: !sub.url_encrypted.is_empty(),
            has_auth_header: sub.auth_header_encrypted.is_some(),
            event_types: sub.event_types.into_iter().flatten().collect(),
            filters,
            target_labels,
            enabled: sub.enabled,
            max_retries: sub.max_retries,
            timeout_seconds: sub.timeout_seconds,
            created_at: sub.created_at,
            updated_at: sub.updated_at,
            created_by: sub.created_by,
        }
    }
}

#[derive(Debug, Clone, Deserialize, ToSchema)]
pub struct ListDeliveriesQuery {
    #[serde(default)]
    pub status: Option<String>,
    #[serde(default)]
    pub limit: Option<i64>,
    #[serde(default)]
    pub offset: Option<i64>,
}

#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct PendingWebhookDelivery {
    pub id: Uuid,
    pub subscription_id: Uuid,
    pub event_type: String,
    pub payload: String,
    pub url: String,
    pub auth_header: Option<String>,
    pub timeout_seconds: i32,
    pub max_retries: i32,
    pub attempts: i32,
}

#[derive(Debug, Clone, Deserialize, ToSchema)]
pub struct DeliveryResultRequest {
    pub success: bool,
    #[serde(default)]
    pub status_code: Option<i32>,
    #[serde(default)]
    pub error: Option<String>,
    #[serde(default)]
    pub duration_ms: Option<i64>,
}

// =============================================================================
// Encryption helpers
// =============================================================================

fn encrypt_value(value: &str) -> Result<Vec<u8>, ApiError> {
    encryption::encrypt_string(value).map_err(|e| {
        error!("Encryption failed: {}", e);
        ApiError::internal("failed to encrypt data")
    })
}

fn decrypt_value(encrypted: &[u8]) -> Result<String, String> {
    encryption::decrypt_string(encrypted)
}

// =============================================================================
// Routes
// =============================================================================

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
        .route(
            "/agents/:agent_id/webhooks/pending",
            get(get_pending_agent_webhooks),
        )
        .route(
            "/webhook-deliveries/:id/result",
            post(report_delivery_result),
        )
}

// =============================================================================
// Handlers
// =============================================================================

#[utoipa::path(
    get, path = "/webhooks", tag = "webhooks",
    responses(
        (status = 200, description = "List all webhook subscriptions", body = Vec<WebhookResponse>),
        (status = 403, description = "Forbidden - Admin access required", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse)
    ),
    security(("admin_pak" = []))
)]
async fn list_webhooks(
    State(dal): State<DAL>,
    Extension(auth_payload): Extension<AuthPayload>,
) -> Result<Json<Vec<WebhookResponse>>, ApiError> {
    info!("Handling request to list webhook subscriptions");
    if !auth_payload.admin {
        return Err(ApiError::forbidden(
            "admin_required",
            "admin access required",
        ));
    }
    let subscriptions = dal.webhook_subscriptions().list(false).map_err(|e| {
        error!("Failed to fetch webhook subscriptions: {:?}", e);
        ApiError::internal("failed to fetch webhook subscriptions")
    })?;
    info!(
        "Successfully retrieved {} webhook subscriptions",
        subscriptions.len()
    );
    Ok(Json(subscriptions.into_iter().map(Into::into).collect()))
}

#[utoipa::path(
    get, path = "/webhooks/event-types", tag = "webhooks",
    responses(
        (status = 200, description = "List available event types", body = Vec<String>),
        (status = 403, description = "Forbidden - Admin access required", body = ErrorResponse)
    ),
    security(("admin_pak" = []))
)]
async fn list_event_types(
    Extension(auth_payload): Extension<AuthPayload>,
) -> Result<Json<Vec<&'static str>>, ApiError> {
    if !auth_payload.admin {
        return Err(ApiError::forbidden(
            "admin_required",
            "admin access required",
        ));
    }
    Ok(Json(VALID_EVENT_TYPES.to_vec()))
}

#[utoipa::path(
    post, path = "/webhooks", tag = "webhooks",
    request_body = CreateWebhookRequest,
    responses(
        (status = 201, description = "Webhook subscription created", body = WebhookResponse),
        (status = 400, description = "Invalid request data", body = ErrorResponse),
        (status = 403, description = "Forbidden - Admin access required", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse)
    ),
    security(("admin_pak" = []))
)]
async fn create_webhook(
    State(dal): State<DAL>,
    Extension(auth_payload): Extension<AuthPayload>,
    Json(request): Json<CreateWebhookRequest>,
) -> Result<(StatusCode, Json<WebhookResponse>), ApiError> {
    info!("Handling request to create webhook subscription");
    if !auth_payload.admin {
        return Err(ApiError::forbidden(
            "admin_required",
            "admin access required",
        ));
    }

    if request.url.trim().is_empty() {
        return Err(ApiError::bad_request("url_required", "URL is required"));
    }
    if !request.url.starts_with("http://") && !request.url.starts_with("https://") {
        return Err(ApiError::bad_request(
            "invalid_url_scheme",
            "URL must start with http:// or https://",
        ));
    }

    let url_encrypted = encrypt_value(&request.url)?;
    let auth_header_encrypted = match &request.auth_header {
        Some(h) => Some(encrypt_value(h)?),
        None => None,
    };

    let created_by = if auth_payload.admin {
        Some("admin".to_string())
    } else {
        auth_payload.generator.map(|id| id.to_string())
    };

    let mut new_sub = NewWebhookSubscription::new(
        request.name,
        url_encrypted,
        auth_header_encrypted,
        request.event_types,
        request.filters,
        request.target_labels,
        created_by,
    )
    .map_err(|e| ApiError::bad_request("invalid_webhook", e))?;
    if let Some(max_retries) = request.max_retries {
        new_sub.max_retries = max_retries;
    }
    if let Some(timeout) = request.timeout_seconds {
        new_sub.timeout_seconds = timeout;
    }

    let subscription = dal.webhook_subscriptions().create(&new_sub).map_err(|e| {
        warn!("Failed to create webhook subscription: {:?}", e);
        ApiError::from_diesel(e, "failed to create webhook subscription")
    })?;
    info!(
        "Successfully created webhook subscription with ID: {}",
        subscription.id
    );

    audit::log_action(
        ACTOR_TYPE_ADMIN,
        None,
        ACTION_WEBHOOK_CREATED,
        RESOURCE_TYPE_WEBHOOK,
        Some(subscription.id),
        Some(serde_json::json!({
            "name": subscription.name,
            "event_types": subscription.event_types,
        })),
        None,
        None,
    );

    Ok((StatusCode::CREATED, Json(subscription.into())))
}

#[utoipa::path(
    get, path = "/webhooks/{id}", tag = "webhooks",
    params(("id" = Uuid, Path, description = "Webhook subscription ID")),
    responses(
        (status = 200, description = "Get webhook subscription by ID", body = WebhookResponse),
        (status = 403, description = "Forbidden - Admin access required", body = ErrorResponse),
        (status = 404, description = "Webhook subscription not found", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse)
    ),
    security(("admin_pak" = []))
)]
async fn get_webhook(
    State(dal): State<DAL>,
    Extension(auth_payload): Extension<AuthPayload>,
    Path(id): Path<Uuid>,
) -> Result<Json<WebhookResponse>, ApiError> {
    info!(
        "Handling request to get webhook subscription with ID: {}",
        id
    );
    if !auth_payload.admin {
        return Err(ApiError::forbidden(
            "admin_required",
            "admin access required",
        ));
    }
    let subscription = dal
        .webhook_subscriptions()
        .get(id)
        .map_err(|e| {
            error!(
                "Failed to fetch webhook subscription with ID {}: {:?}",
                id, e
            );
            ApiError::internal("failed to fetch webhook subscription")
        })?
        .ok_or_else(|| {
            ApiError::not_found("webhook_not_found", "webhook subscription not found")
        })?;
    Ok(Json(subscription.into()))
}

#[utoipa::path(
    put, path = "/webhooks/{id}", tag = "webhooks",
    params(("id" = Uuid, Path, description = "Webhook subscription ID")),
    request_body = UpdateWebhookRequest,
    responses(
        (status = 200, description = "Webhook subscription updated", body = WebhookResponse),
        (status = 400, description = "Invalid request data", body = ErrorResponse),
        (status = 403, description = "Forbidden - Admin access required", body = ErrorResponse),
        (status = 404, description = "Webhook subscription not found", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse)
    ),
    security(("admin_pak" = []))
)]
async fn update_webhook(
    State(dal): State<DAL>,
    Extension(auth_payload): Extension<AuthPayload>,
    Path(id): Path<Uuid>,
    Json(request): Json<UpdateWebhookRequest>,
) -> Result<Json<WebhookResponse>, ApiError> {
    info!(
        "Handling request to update webhook subscription with ID: {}",
        id
    );
    if !auth_payload.admin {
        return Err(ApiError::forbidden(
            "admin_required",
            "admin access required",
        ));
    }

    dal.webhook_subscriptions()
        .get(id)
        .map_err(|e| {
            error!("Failed to fetch webhook subscription: {:?}", e);
            ApiError::internal("failed to fetch webhook subscription")
        })?
        .ok_or_else(|| {
            ApiError::not_found("webhook_not_found", "webhook subscription not found")
        })?;

    let url_encrypted = match request.url {
        Some(u) => Some(encrypt_value(&u)?),
        None => None,
    };
    let auth_header_encrypted = match request.auth_header {
        Some(Some(h)) => Some(Some(encrypt_value(&h)?)),
        Some(None) => Some(None),
        None => None,
    };
    let target_labels = request
        .target_labels
        .map(|opt| opt.map(|labels| labels.into_iter().map(Some).collect()));

    let changeset = UpdateWebhookSubscription {
        name: request.name,
        url_encrypted,
        auth_header_encrypted,
        event_types: request
            .event_types
            .map(|types| types.into_iter().map(Some).collect()),
        filters: request
            .filters
            .map(|opt| opt.map(|f| serde_json::to_string(&f).unwrap_or_default())),
        target_labels,
        enabled: request.enabled,
        max_retries: request.max_retries,
        timeout_seconds: request.timeout_seconds,
    };

    let subscription = dal
        .webhook_subscriptions()
        .update(id, &changeset)
        .map_err(|e| {
            error!(
                "Failed to update webhook subscription with ID {}: {:?}",
                id, e
            );
            ApiError::internal("failed to update webhook subscription")
        })?;
    info!("Successfully updated webhook subscription with ID: {}", id);

    audit::log_action(
        ACTOR_TYPE_ADMIN,
        None,
        ACTION_WEBHOOK_UPDATED,
        RESOURCE_TYPE_WEBHOOK,
        Some(id),
        Some(serde_json::json!({
            "name": subscription.name,
            "enabled": subscription.enabled,
        })),
        None,
        None,
    );

    Ok(Json(subscription.into()))
}

#[utoipa::path(
    delete, path = "/webhooks/{id}", tag = "webhooks",
    params(("id" = Uuid, Path, description = "Webhook subscription ID")),
    responses(
        (status = 204, description = "Webhook subscription deleted"),
        (status = 403, description = "Forbidden - Admin access required", body = ErrorResponse),
        (status = 404, description = "Webhook subscription not found", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse)
    ),
    security(("admin_pak" = []))
)]
async fn delete_webhook(
    State(dal): State<DAL>,
    Extension(auth_payload): Extension<AuthPayload>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, ApiError> {
    info!(
        "Handling request to delete webhook subscription with ID: {}",
        id
    );
    if !auth_payload.admin {
        return Err(ApiError::forbidden(
            "admin_required",
            "admin access required",
        ));
    }

    let count = dal.webhook_subscriptions().delete(id).map_err(|e| {
        error!(
            "Failed to delete webhook subscription with ID {}: {:?}",
            id, e
        );
        ApiError::internal("failed to delete webhook subscription")
    })?;

    if count == 0 {
        return Err(ApiError::not_found(
            "webhook_not_found",
            "webhook subscription not found",
        ));
    }
    info!("Successfully deleted webhook subscription with ID: {}", id);
    audit::log_action(
        ACTOR_TYPE_ADMIN,
        None,
        ACTION_WEBHOOK_DELETED,
        RESOURCE_TYPE_WEBHOOK,
        Some(id),
        None,
        None,
        None,
    );
    Ok(StatusCode::NO_CONTENT)
}

#[utoipa::path(
    get, path = "/webhooks/{id}/deliveries", tag = "webhooks",
    params(
        ("id" = Uuid, Path, description = "Webhook subscription ID"),
        ("status" = Option<String>, Query, description = "Filter by delivery status"),
        ("limit" = Option<i64>, Query, description = "Maximum number of results"),
        ("offset" = Option<i64>, Query, description = "Offset for pagination")
    ),
    responses(
        (status = 200, description = "List deliveries for subscription", body = Vec<WebhookDelivery>),
        (status = 403, description = "Forbidden - Admin access required", body = ErrorResponse),
        (status = 404, description = "Webhook subscription not found", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse)
    ),
    security(("admin_pak" = []))
)]
async fn list_deliveries(
    State(dal): State<DAL>,
    Extension(auth_payload): Extension<AuthPayload>,
    Path(id): Path<Uuid>,
    Query(query): Query<ListDeliveriesQuery>,
) -> Result<Json<Vec<WebhookDelivery>>, ApiError> {
    info!(
        "Handling request to list deliveries for webhook subscription: {}",
        id
    );
    if !auth_payload.admin {
        return Err(ApiError::forbidden(
            "admin_required",
            "admin access required",
        ));
    }

    dal.webhook_subscriptions()
        .get(id)
        .map_err(|e| {
            error!("Failed to fetch webhook subscription: {:?}", e);
            ApiError::internal("failed to fetch webhook subscription")
        })?
        .ok_or_else(|| {
            ApiError::not_found("webhook_not_found", "webhook subscription not found")
        })?;

    let limit = query.limit.unwrap_or(50);
    let offset = query.offset.unwrap_or(0);

    let deliveries = dal
        .webhook_deliveries()
        .list_for_subscription(id, query.status.as_deref(), limit, offset)
        .map_err(|e| {
            error!(
                "Failed to fetch deliveries for subscription {}: {:?}",
                id, e
            );
            ApiError::internal("failed to fetch deliveries")
        })?;
    info!(
        "Successfully retrieved {} deliveries for subscription {}",
        deliveries.len(),
        id
    );
    Ok(Json(deliveries))
}

#[utoipa::path(
    post, path = "/webhooks/{id}/test", tag = "webhooks",
    params(("id" = Uuid, Path, description = "Webhook subscription ID")),
    responses(
        (status = 200, description = "Test delivery successful"),
        (status = 400, description = "Test delivery failed", body = ErrorResponse),
        (status = 403, description = "Forbidden - Admin access required", body = ErrorResponse),
        (status = 404, description = "Webhook subscription not found", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse)
    ),
    security(("admin_pak" = []))
)]
async fn test_webhook(
    State(dal): State<DAL>,
    Extension(auth_payload): Extension<AuthPayload>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, ApiError> {
    info!(
        "Handling request to test webhook subscription with ID: {}",
        id
    );
    if !auth_payload.admin {
        return Err(ApiError::forbidden(
            "admin_required",
            "admin access required",
        ));
    }

    let subscription = dal
        .webhook_subscriptions()
        .get(id)
        .map_err(|e| {
            error!("Failed to fetch webhook subscription: {:?}", e);
            ApiError::internal("failed to fetch webhook subscription")
        })?
        .ok_or_else(|| {
            ApiError::not_found("webhook_not_found", "webhook subscription not found")
        })?;

    let url = decrypt_value(&subscription.url_encrypted).map_err(|e| {
        error!("Failed to decrypt URL: {}", e);
        ApiError::internal("failed to decrypt webhook URL")
    })?;
    let auth_header = subscription
        .auth_header_encrypted
        .as_ref()
        .map(|h| decrypt_value(h))
        .transpose()
        .map_err(|e| {
            error!("Failed to decrypt auth header: {}", e);
            ApiError::internal("failed to decrypt auth header")
        })?;

    let test_event = serde_json::json!({
        "id": Uuid::new_v4(),
        "event_type": "webhook.test",
        "timestamp": chrono::Utc::now(),
        "data": {
            "message": "This is a test webhook delivery from Brokkr",
            "subscription_id": id
        }
    });

    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(
            subscription.timeout_seconds as u64,
        ))
        .build()
        .map_err(|e| {
            error!("Failed to create HTTP client: {:?}", e);
            ApiError::internal("failed to create HTTP client")
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
                warn!(
                    "Test webhook delivery failed with status {}: {}",
                    status, body
                );
                let mut details = std::collections::BTreeMap::new();
                details.insert("status_code".into(), serde_json::json!(status.as_u16()));
                details.insert(
                    "body".into(),
                    serde_json::json!(body.chars().take(500).collect::<String>()),
                );
                Err(ApiError::bad_request(
                    "webhook_test_failed",
                    format!("endpoint returned HTTP {}", status),
                )
                .with_details(details))
            }
        }
        Err(e) => {
            error!("Test webhook delivery failed: {:?}", e);
            Err(ApiError::bad_request(
                "webhook_test_failed",
                format!("request failed: {}", e),
            ))
        }
    }
}

// =============================================================================
// Agent webhook delivery endpoints
// =============================================================================

#[utoipa::path(
    get, path = "/agents/{agent_id}/webhooks/pending", tag = "webhooks",
    params(("agent_id" = Uuid, Path, description = "Agent ID")),
    responses(
        (status = 200, description = "Pending webhook deliveries for this agent", body = Vec<PendingWebhookDelivery>),
        (status = 403, description = "Forbidden - Agent access required", body = ErrorResponse),
        (status = 404, description = "Agent not found", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse)
    ),
    security(("admin_pak" = []), ("agent_pak" = []))
)]
async fn get_pending_agent_webhooks(
    State(dal): State<DAL>,
    Extension(auth_payload): Extension<AuthPayload>,
    Path(agent_id): Path<Uuid>,
) -> Result<Json<Vec<PendingWebhookDelivery>>, ApiError> {
    debug!(
        "Handling request for pending webhooks for agent: {}",
        agent_id
    );
    if !auth_payload.admin && auth_payload.agent != Some(agent_id) {
        warn!(
            "Unauthorized access to agent webhooks: {:?} != {:?}",
            auth_payload.agent, agent_id
        );
        return Err(ApiError::forbidden(
            "agent_pak_mismatch",
            "must be the agent or admin",
        ));
    }

    dal.agents()
        .get(agent_id)
        .map_err(|e| {
            error!("Failed to fetch agent: {:?}", e);
            ApiError::internal("failed to fetch agent")
        })?
        .ok_or_else(|| ApiError::not_found("agent_not_found", "agent not found"))?;

    let agent_labels: Vec<String> = match dal.agent_labels().list_for_agent(agent_id) {
        Ok(labels) => labels.into_iter().map(|l| l.label).collect(),
        Err(e) => {
            error!("Failed to fetch agent labels: {:?}", e);
            vec![]
        }
    };

    let deliveries = dal
        .webhook_deliveries()
        .claim_for_agent(agent_id, &agent_labels, 10, None)
        .map_err(|e| {
            error!("Failed to claim pending deliveries: {:?}", e);
            ApiError::internal("failed to claim pending deliveries")
        })?;

    let mut pending = Vec::with_capacity(deliveries.len());
    for delivery in deliveries {
        let subscription = match dal.webhook_subscriptions().get(delivery.subscription_id) {
            Ok(Some(sub)) => sub,
            Ok(None) => {
                warn!(
                    "Subscription {} not found for delivery {}",
                    delivery.subscription_id, delivery.id
                );
                continue;
            }
            Err(e) => {
                error!("Failed to fetch subscription: {:?}", e);
                continue;
            }
        };
        let url = match decrypt_value(&subscription.url_encrypted) {
            Ok(u) => u,
            Err(e) => {
                error!(
                    "Failed to decrypt URL for subscription {}: {}",
                    subscription.id, e
                );
                continue;
            }
        };
        let auth_header = match subscription.auth_header_encrypted {
            Some(ref encrypted) => match decrypt_value(encrypted) {
                Ok(h) => Some(h),
                Err(e) => {
                    error!(
                        "Failed to decrypt auth header for subscription {}: {}",
                        subscription.id, e
                    );
                    None
                }
            },
            None => None,
        };
        pending.push(PendingWebhookDelivery {
            id: delivery.id,
            subscription_id: delivery.subscription_id,
            event_type: delivery.event_type,
            payload: delivery.payload,
            url,
            auth_header,
            timeout_seconds: subscription.timeout_seconds,
            max_retries: subscription.max_retries,
            attempts: delivery.attempts,
        });
    }

    debug!(
        "Returning {} pending webhook deliveries for agent {}",
        pending.len(),
        agent_id
    );
    Ok(Json(pending))
}

#[utoipa::path(
    post, path = "/webhook-deliveries/{id}/result", tag = "webhooks",
    params(("id" = Uuid, Path, description = "Delivery ID")),
    request_body = DeliveryResultRequest,
    responses(
        (status = 200, description = "Delivery result recorded"),
        (status = 403, description = "Forbidden - Agent access required", body = ErrorResponse),
        (status = 404, description = "Delivery not found", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse)
    ),
    security(("agent_pak" = []))
)]
async fn report_delivery_result(
    State(dal): State<DAL>,
    Extension(auth_payload): Extension<AuthPayload>,
    Path(delivery_id): Path<Uuid>,
    Json(request): Json<DeliveryResultRequest>,
) -> Result<Json<serde_json::Value>, ApiError> {
    debug!(
        "Handling delivery result report for delivery: {}",
        delivery_id
    );

    let agent_id = auth_payload.agent.ok_or_else(|| {
        ApiError::forbidden("agent_pak_required", "agent authentication required")
    })?;

    let delivery = dal
        .webhook_deliveries()
        .get(delivery_id)
        .map_err(|e| {
            error!("Failed to fetch delivery: {:?}", e);
            ApiError::internal("failed to fetch delivery")
        })?
        .ok_or_else(|| ApiError::not_found("delivery_not_found", "delivery not found"))?;

    if delivery.acquired_by != Some(agent_id) {
        warn!(
            "Agent {} tried to report result for delivery {} acquired by {:?}",
            agent_id, delivery_id, delivery.acquired_by
        );
        return Err(ApiError::forbidden(
            "delivery_not_acquired_by_agent",
            "delivery not acquired by this agent",
        ));
    }

    let subscription = dal
        .webhook_subscriptions()
        .get(delivery.subscription_id)
        .map_err(|e| {
            error!("Failed to fetch subscription: {:?}", e);
            ApiError::internal("failed to fetch subscription")
        })?
        .ok_or_else(|| {
            error!(
                "Subscription {} not found for delivery {}",
                delivery.subscription_id, delivery_id
            );
            ApiError::internal("subscription not found")
        })?;

    if request.success {
        dal.webhook_deliveries()
            .mark_success(delivery_id)
            .map_err(|e| {
                error!("Failed to mark delivery as success: {:?}", e);
                ApiError::internal("failed to update delivery")
            })?;
        info!(
            "Webhook delivery {} succeeded via agent {}",
            delivery_id, agent_id
        );
        Ok(Json(serde_json::json!({
            "status": "success",
            "delivery_id": delivery_id
        })))
    } else {
        let error_msg = request.error.unwrap_or_else(|| "Unknown error".to_string());
        let updated = dal
            .webhook_deliveries()
            .mark_failed(delivery_id, &error_msg, subscription.max_retries)
            .map_err(|e| {
                error!("Failed to mark delivery as failed: {:?}", e);
                ApiError::internal("failed to update delivery")
            })?;
        info!(
            "Webhook delivery {} failed via agent {}: {}",
            delivery_id, agent_id, error_msg
        );
        if updated.status == "dead" {
            audit::log_action(
                ACTOR_TYPE_SYSTEM,
                None,
                ACTION_WEBHOOK_DELIVERY_FAILED,
                RESOURCE_TYPE_WEBHOOK,
                Some(updated.subscription_id),
                Some(serde_json::json!({
                    "delivery_id": delivery_id,
                    "attempts": updated.attempts,
                    "error": error_msg,
                    "delivered_by_agent": agent_id,
                })),
                None,
                None,
            );
        }
        Ok(Json(serde_json::json!({
            "status": updated.status,
            "delivery_id": delivery_id,
            "attempts": updated.attempts,
            "next_retry_at": updated.next_retry_at
        })))
    }
}
