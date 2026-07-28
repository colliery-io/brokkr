/*
 * Copyright (c) 2025-2026 Dylan Storey
 * Licensed under the Elastic License 2.0.
 * See LICENSE file in the project root for full license text.
 */

//! Webhooks API module for Brokkr.

use crate::api::v1::error::{ApiError, ErrorResponse};
use crate::api::v1::middleware::AuthPayload;
use crate::dal::DAL;
use crate::dal::webhook_deliveries::is_retryable_status;
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

/// Body of `POST /webhooks`.
///
/// Unknown keys are ignored rather than rejected, with two deliberate
/// exceptions that this endpoint used to accept and silently ignore
/// (BROKKR-T-0288). Both are now **rejected with 422** by
/// [`reject_removed_write_fields`] before this struct is deserialized:
///
/// * `validate` ("send test request on creation") was documented and parsed but
///   never read by `create_webhook` — it did nothing, ever. The real mechanism
///   is `POST /webhooks/{id}/test`.
/// * `filters.labels` was stored and echoed but never evaluated; label-based
///   routing is `target_labels`, which is real.
///
/// Rejecting is the lesser evil. A caller who sends `filters.labels` believes
/// their deliveries are scoped; accepting the request and dropping the key
/// leaves them with a subscription that fires on everything and a response they
/// have no reason to re-read. A 422 naming the field and its replacement is
/// noisy exactly once, at the moment the operator can still fix it.
///
/// This is a **write-path** rule only. Subscription rows already stored with a
/// `labels` key keep loading and keep delivering — see [`WebhookFilters`],
/// whose deserialization stays tolerant of unknown keys.
#[derive(Debug, Clone, Deserialize, ToSchema)]
pub struct CreateWebhookRequest {
    pub name: String,
    pub url: String,
    #[serde(default)]
    pub auth_header: Option<String>,
    pub event_types: Vec<String>,
    /// Payload filters (`agent_id`, `stack_id`). An event that does not carry a
    /// filtered field does not match; see [`WebhookFilters`].
    #[serde(default)]
    pub filters: Option<WebhookFilters>,
    #[serde(default)]
    pub max_retries: Option<i32>,
    #[serde(default)]
    pub timeout_seconds: Option<i32>,
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
    /// Payload filters. `null` clears them; omitted leaves them unchanged. A
    /// legacy `labels` key inside the object is rejected with 422 rather than
    /// silently dropped — see [`CreateWebhookRequest`].
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
    /// Payload filters as stored, showing exactly what is evaluated at emission
    /// time. New writes can no longer introduce keys the broker does not
    /// understand, but rows predating BROKKR-T-0288 may still carry a `labels`
    /// key; it is not echoed here because it is not evaluated.
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
// Removed request fields (BROKKR-T-0288)
// =============================================================================

/// Stable error code for a request that carries a field the webhook API removed.
///
/// One code with a `details.field` discriminator rather than one code per field:
/// callers pattern-match on `unsupported_field` and read `details.field`, so
/// removing another key later does not add another code to the SDK contract.
const CODE_UNSUPPORTED_FIELD: &str = "unsupported_field";

/// Which removed fields apply to the body being validated.
///
/// `validate` only ever existed on the create body, so a `PUT` carrying it is
/// just an unknown key and stays ignored like any other.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum WriteBody {
    Create,
    Update,
}

/// Rejects a create/update body that carries a field the webhook API removed.
///
/// Operates on the **raw** body, before typed deserialization, for two reasons:
///
/// 1. It keeps the rejection surgical. Modelling the removed keys as fields on
///    the request DTOs would work, but they would then appear in the generated
///    OpenAPI schema as if they were part of the contract. `deny_unknown_fields`
///    on the DTOs would work too, but it rejects every stray key — typos
///    included — which is a much broader behavior change, and serde's
///    "unknown field `labels`" cannot name the replacement.
/// 2. Only the write path is affected. [`WebhookFilters`]'s `Deserialize` impl
///    is untouched and stays tolerant of unknown keys, so stored rows that
///    predate the removal keep loading and keep delivering.
///
/// A key counts as present even when its value is `null`: the caller wrote it
/// down, so they believe it means something.
///
/// # Arguments
/// * `body` - The raw JSON request body.
/// * `which` - Whether this is a create or an update body.
fn reject_removed_write_fields(body: &serde_json::Value, which: WriteBody) -> Result<(), ApiError> {
    if which == WriteBody::Create && body.get("validate").is_some() {
        return Err(removed_field_error(
            "validate",
            "`validate` is no longer accepted: it never sent a test request. \
             Create the subscription, then call POST /webhooks/{id}/test.",
            "POST /webhooks/{id}/test",
        ));
    }

    if let Some(filters) = body.get("filters")
        && filters.get("labels").is_some()
    {
        return Err(removed_field_error(
            "filters.labels",
            "`filters.labels` is no longer accepted: it was never evaluated, so a \
             subscription that set it received every event of its subscribed types. \
             Use `target_labels` for label-based delivery routing, or filter on \
             `filters.agent_id` / `filters.stack_id`.",
            "target_labels",
        ));
    }

    Ok(())
}

/// Builds the 422 for a removed field, naming the field and its replacement in
/// both the message and machine-readable `details`.
fn removed_field_error(field: &str, message: &str, use_instead: &str) -> ApiError {
    let mut details = std::collections::BTreeMap::new();
    details.insert("field".into(), serde_json::json!(field));
    details.insert("use_instead".into(), serde_json::json!(use_instead));
    ApiError::unprocessable(CODE_UNSUPPORTED_FIELD, message).with_details(details)
}

/// Deserializes an already-parsed body into a request DTO.
///
/// The handlers take `Json<serde_json::Value>` so [`reject_removed_write_fields`]
/// can inspect the raw keys; this restores the typed step. Axum's own
/// `JsonDataError` is a 422, so the status for a badly shaped body is unchanged
/// — only the body format is, from axum's plain text to the API's
/// [`ErrorResponse`]. Malformed JSON still fails in the extractor with a 400.
fn parse_body<T: serde::de::DeserializeOwned>(body: serde_json::Value) -> Result<T, ApiError> {
    serde_json::from_value(body).map_err(|e| {
        ApiError::unprocessable("invalid_request_body", format!("invalid request body: {e}"))
    })
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

/// Attempt budget used when a delivery must be failed without its subscription
/// in hand. Mirrors the `max_retries` column default in migration
/// `13_webhooks` (and `NewWebhookSubscription::new`); only reachable when the
/// subscription row itself could not be read (BROKKR-T-0304).
const FALLBACK_MAX_RETRIES: i32 = 5;

/// How a claimed delivery that cannot be dispatched should be disposed of.
///
/// The question each arm has to answer is whether re-claiming the same row
/// could ever produce a different outcome.
enum DeliveryDisposition {
    /// The failure is a property of the stored data rather than of this
    /// attempt — unreadable ciphertext, or a subscription that no longer
    /// exists. Every subsequent claim fails identically, so the delivery goes
    /// straight to `dead` whatever budget it had left.
    Terminal,
    /// The failure may not recur — a transient database error says nothing
    /// about this delivery. Ordinary retry semantics apply: count the attempt,
    /// release the claim, back off, and die only once `max_retries` is
    /// exhausted.
    Retryable {
        /// Total attempts allowed before the delivery dies.
        max_retries: i32,
    },
}

/// Fails a delivery that has already been claimed and cannot be handed to the
/// agent (BROKKR-T-0302, BROKKR-T-0304).
///
/// Every arm of the agent poll loop that gives up on a claimed delivery must go
/// through here. Bare `continue` leaves the row `acquired` until its 60s TTL
/// lapses, at which point `release_expired` returns it to `pending`, the next
/// poll re-claims it, and it fails again — forever, with `attempts` pinned at 0
/// because nothing ever recorded the failure. Both dispositions write a state
/// that releases the claim and bounds the cycle. The `Terminal` disposition
/// mirrors the broker delivery path in `utils::background_tasks`.
///
/// Errors are logged rather than returned: one poisoned delivery must not fail
/// the whole poll for the agent's other pending deliveries.
///
/// # Arguments
/// * `reason` - Recorded verbatim as the delivery's `last_error`; it must name
///   the specific failure so a dead row stays attributable.
/// * `disposition` - See [`DeliveryDisposition`].
fn fail_claimed_delivery(
    dal: &DAL,
    delivery: &WebhookDelivery,
    reason: &str,
    disposition: DeliveryDisposition,
) {
    let deliveries = dal.webhook_deliveries();
    let result = match disposition {
        DeliveryDisposition::Terminal => deliveries.mark_dead(delivery.id, reason),
        DeliveryDisposition::Retryable { max_retries } => {
            deliveries.mark_failed(delivery.id, reason, max_retries)
        }
    };
    match result {
        Ok(updated) => {
            warn!(
                "Webhook delivery {} marked {} for subscription {}: {}",
                delivery.id, updated.status, delivery.subscription_id, reason
            );
            if updated.status == "dead" {
                audit::log_action(
                    ACTOR_TYPE_SYSTEM,
                    None,
                    ACTION_WEBHOOK_DELIVERY_FAILED,
                    RESOURCE_TYPE_WEBHOOK,
                    Some(delivery.subscription_id),
                    Some(serde_json::json!({
                        "delivery_id": delivery.id,
                        "event_type": delivery.event_type,
                        "attempts": updated.attempts,
                        "error": reason,
                    })),
                    None,
                    None,
                );
            }
        }
        Err(e) => {
            error!(
                "Failed to record failure for claimed delivery {}: {:?}",
                delivery.id, e
            );
        }
    }
}

/// Builds the `last_error` recorded for an agent-reported failure, guaranteeing
/// the HTTP status is present whenever the agent observed one (BROKKR-T-0288).
///
/// Agents already format their message as `HTTP <code>: <body>`, so that form
/// is passed through untouched instead of being prefixed twice.
fn delivery_failure_message(error: Option<&str>, status_code: Option<u16>) -> String {
    let detail = error.map(str::trim).filter(|e| !e.is_empty());
    match (status_code, detail) {
        (Some(code), Some(d)) if d.starts_with(&format!("HTTP {}", code)) => d.to_string(),
        (Some(code), Some(d)) => format!("HTTP {}: {}", code, d),
        (Some(code), None) => format!("HTTP {}", code),
        (None, Some(d)) => d.to_string(),
        (None, None) => "Unknown error".to_string(),
    }
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
        (status = 422, description = "Unprocessable - out-of-range value, or a removed field (`validate`, `filters.labels`)", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse)
    ),
    security(("admin_pak" = []))
)]
async fn create_webhook(
    State(dal): State<DAL>,
    Extension(auth_payload): Extension<AuthPayload>,
    Json(body): Json<serde_json::Value>,
) -> Result<(StatusCode, Json<WebhookResponse>), ApiError> {
    info!("Handling request to create webhook subscription");
    if !auth_payload.admin {
        return Err(ApiError::forbidden(
            "admin_required",
            "admin access required",
        ));
    }

    reject_removed_write_fields(&body, WriteBody::Create)?;
    let request: CreateWebhookRequest = parse_body(body)?;

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
        if !(0..=10).contains(&max_retries) {
            return Err(ApiError::unprocessable(
                "invalid_webhook",
                "max_retries must be between 0 and 10",
            ));
        }
        new_sub.max_retries = max_retries;
    }
    if let Some(timeout) = request.timeout_seconds {
        if !(1..=300).contains(&timeout) {
            return Err(ApiError::unprocessable(
                "invalid_webhook",
                "timeout_seconds must be between 1 and 300",
            ));
        }
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
        (status = 422, description = "Unprocessable - out-of-range value, or the removed `filters.labels` field", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse)
    ),
    security(("admin_pak" = []))
)]
async fn update_webhook(
    State(dal): State<DAL>,
    Extension(auth_payload): Extension<AuthPayload>,
    Path(id): Path<Uuid>,
    Json(body): Json<serde_json::Value>,
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

    reject_removed_write_fields(&body, WriteBody::Update)?;
    let request: UpdateWebhookRequest = parse_body(body)?;

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

    if let Some(max_retries) = request.max_retries
        && !(0..=10).contains(&max_retries)
    {
        return Err(ApiError::unprocessable(
            "invalid_webhook",
            "max_retries must be between 0 and 10",
        ));
    }
    if let Some(timeout) = request.timeout_seconds
        && !(1..=300).contains(&timeout)
    {
        return Err(ApiError::unprocessable(
            "invalid_webhook",
            "timeout_seconds must be between 1 and 300",
        ));
    }

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
            // Clamp defensively: a row written before validation could be
            // negative, which would sign-extend to an absurd timeout.
            subscription.timeout_seconds.max(1) as u64,
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
                    "Subscription {} not found for delivery {}, marking as dead",
                    delivery.subscription_id, delivery.id
                );
                // Permanent: a missing subscription never comes back, and
                // without its URL there is nothing to deliver to. The delivery
                // is already claimed, so `continue` alone would cycle it
                // forever (BROKKR-T-0304). Matches the broker delivery path.
                fail_claimed_delivery(
                    &dal,
                    &delivery,
                    "Subscription not found",
                    DeliveryDisposition::Terminal,
                );
                continue;
            }
            Err(e) => {
                error!(
                    "Failed to fetch subscription {} for delivery {}: {:?}",
                    delivery.subscription_id, delivery.id, e
                );
                // Retryable, unlike the arm above: this is a database error,
                // not a statement about the delivery. The subscription is
                // probably still there and readable on the next poll, so
                // killing the delivery on the first blip would discard a valid
                // webhook. The claim must still be settled, or the row cycles
                // claimed -> TTL expiry -> reclaimed with `attempts` never
                // moving (BROKKR-T-0304); recording the attempt bounds the
                // cycle by the retry budget instead. The subscription's own
                // `max_retries` is exactly what could not be read, so the
                // schema default stands in. If the database is down hard this
                // write fails too, leaving the row to TTL recovery — which is
                // the right outcome for an outage that recorded nothing.
                fail_claimed_delivery(
                    &dal,
                    &delivery,
                    &format!("Failed to fetch subscription: {}", e),
                    DeliveryDisposition::Retryable {
                        max_retries: FALLBACK_MAX_RETRIES,
                    },
                );
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
                // The delivery is already claimed; without this it would sit
                // acquired until its TTL lapsed and then be reclaimed forever
                // with attempts pinned at 0 (BROKKR-T-0302). Terminal: the
                // ciphertext is unreadable with the broker's current key, so
                // every later claim fails identically.
                crate::metrics::record_webhook_decrypt_failure("url", "agent");
                fail_claimed_delivery(
                    &dal,
                    &delivery,
                    &format!("Failed to decrypt URL: {}", e),
                    DeliveryDisposition::Terminal,
                );
                continue;
            }
        };
        // A subscription configured with an auth header must never be delivered
        // without it: dispatching unauthenticated would silently downgrade the
        // subscriber's authentication (BROKKR-T-0302).
        let auth_header = match subscription.auth_header_encrypted {
            Some(ref encrypted) => match decrypt_value(encrypted) {
                Ok(h) => Some(h),
                Err(e) => {
                    error!(
                        "Failed to decrypt auth header for subscription {}: {}",
                        subscription.id, e
                    );
                    crate::metrics::record_webhook_decrypt_failure("auth_header", "agent");
                    fail_claimed_delivery(
                        &dal,
                        &delivery,
                        &format!("Failed to decrypt auth header: {}", e),
                        DeliveryDisposition::Terminal,
                    );
                    continue;
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
        // The agent already knows the endpoint's status; classify with it
        // rather than discarding it and letting a 404 burn the whole attempt
        // budget (BROKKR-T-0288). A result with no status is a transport
        // failure on the agent side and stays retryable.
        let status_code = request.status_code.and_then(|c| u16::try_from(c).ok());
        let error_msg = delivery_failure_message(request.error.as_deref(), status_code);
        let retryable = status_code.map(is_retryable_status).unwrap_or(true);

        let updated = if retryable {
            dal.webhook_deliveries()
                .mark_failed(delivery_id, &error_msg, subscription.max_retries)
        } else {
            dal.webhook_deliveries().mark_dead(delivery_id, &error_msg)
        }
        .map_err(|e| {
            error!("Failed to mark delivery as failed: {:?}", e);
            ApiError::internal("failed to update delivery")
        })?;
        info!(
            "Webhook delivery {} failed via agent {} (status {:?}, retryable {}): {}",
            delivery_id, agent_id, status_code, retryable, error_msg
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
                    "status_code": status_code,
                    "retryable": retryable,
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

#[cfg(test)]
mod removed_write_field_tests {
    use super::*;
    use serde_json::json;

    fn err(body: serde_json::Value, which: WriteBody) -> ApiError {
        reject_removed_write_fields(&body, which).expect_err("expected a rejection")
    }

    fn field_of(error: &ApiError) -> String {
        error.details.as_ref().unwrap()["field"]
            .as_str()
            .unwrap()
            .to_string()
    }

    #[test]
    fn create_rejects_validate_with_422_naming_the_replacement() {
        let error = err(
            json!({"name": "n", "url": "https://x", "event_types": ["agent.registered"], "validate": true}),
            WriteBody::Create,
        );

        assert_eq!(error.status, StatusCode::UNPROCESSABLE_ENTITY);
        assert_eq!(error.code, CODE_UNSUPPORTED_FIELD);
        assert_eq!(field_of(&error), "validate");
        assert!(error.message.contains("validate"));
        assert!(error.message.contains("POST /webhooks/{id}/test"));
    }

    #[test]
    fn create_rejects_filters_labels_with_422_naming_target_labels() {
        let error = err(
            json!({"name": "n", "filters": {"agent_id": Uuid::new_v4(), "labels": {"env": "prod"}}}),
            WriteBody::Create,
        );

        assert_eq!(error.status, StatusCode::UNPROCESSABLE_ENTITY);
        assert_eq!(error.code, CODE_UNSUPPORTED_FIELD);
        assert_eq!(field_of(&error), "filters.labels");
        assert!(error.message.contains("filters.labels"));
        assert!(error.message.contains("target_labels"));
    }

    #[test]
    fn update_rejects_filters_labels() {
        let error = err(json!({"filters": {"labels": []}}), WriteBody::Update);

        assert_eq!(error.status, StatusCode::UNPROCESSABLE_ENTITY);
        assert_eq!(field_of(&error), "filters.labels");
    }

    #[test]
    fn a_null_valued_removed_key_is_still_present() {
        // The caller wrote the key down, so they believe it does something.
        assert_eq!(
            field_of(&err(json!({"validate": null}), WriteBody::Create)),
            "validate"
        );
        assert_eq!(
            field_of(&err(
                json!({"filters": {"labels": null}}),
                WriteBody::Update
            )),
            "filters.labels"
        );
    }

    #[test]
    fn update_ignores_validate_because_it_never_existed_there() {
        // `validate` was only ever a create-body key; on PUT it is an ordinary
        // unknown field and keeps being ignored.
        assert!(reject_removed_write_fields(&json!({"validate": true}), WriteBody::Update).is_ok());
    }

    #[test]
    fn supported_bodies_pass() {
        for which in [WriteBody::Create, WriteBody::Update] {
            assert!(reject_removed_write_fields(&json!({}), which).is_ok());
            assert!(reject_removed_write_fields(&json!({"filters": null}), which).is_ok());
            assert!(reject_removed_write_fields(&json!({"filters": {}}), which).is_ok());
            assert!(
                reject_removed_write_fields(
                    &json!({
                        "filters": {"agent_id": Uuid::new_v4(), "stack_id": Uuid::new_v4()},
                        "target_labels": ["env:prod"],
                    }),
                    which,
                )
                .is_ok()
            );
        }
    }

    #[test]
    fn stored_filter_deserialization_stays_tolerant() {
        // The read path is deliberately *not* symmetric with the write path: a
        // row persisted before `labels` was removed must keep loading.
        let stored: WebhookFilters =
            serde_json::from_str(r#"{"labels":{"env":"prod"}}"#).expect("legacy row must load");
        assert!(stored.is_empty());
    }

    #[test]
    fn parse_body_reports_shape_errors_as_422() {
        let error = parse_body::<CreateWebhookRequest>(json!({"url": "https://x"}))
            .expect_err("missing required fields");

        assert_eq!(error.status, StatusCode::UNPROCESSABLE_ENTITY);
        assert_eq!(error.code, "invalid_request_body");
        assert!(error.message.contains("name"), "message: {}", error.message);
    }
}
