/*
 * Copyright (c) 2025-2026 Dylan Storey
 * Licensed under the Elastic License 2.0.
 * See LICENSE file in the project root for full license text.
 */

use crate::fixtures::TestFixture;
use axum::{
    body::{Body, to_bytes},
    http::{Request, StatusCode},
};
use serde_json::json;
use tower::ServiceExt;

// =============================================================================
// List Webhooks Tests
// =============================================================================

#[tokio::test]
async fn test_list_webhooks_admin_success() {
    let fixture = TestFixture::new();
    let app = fixture.create_test_router().with_state(fixture.dal.clone());
    let admin_pak = fixture.admin_pak.clone();

    let response = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/api/v1/webhooks")
                .header("Authorization", format!("Bearer {}", admin_pak))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let webhooks: Vec<serde_json::Value> = serde_json::from_slice(&body).unwrap();
    assert!(webhooks.is_empty()); // No webhooks created yet
}

#[tokio::test]
async fn test_list_webhooks_non_admin_forbidden() {
    let fixture = TestFixture::new();
    let app = fixture.create_test_router().with_state(fixture.dal.clone());
    let (_, generator_pak) =
        fixture.create_test_generator_with_pak("Test Generator".to_string(), None);

    let response = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/api/v1/webhooks")
                .header("Authorization", format!("Bearer {}", generator_pak))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn test_list_webhooks_unauthorized() {
    let fixture = TestFixture::new();
    let app = fixture.create_test_router().with_state(fixture.dal.clone());

    let response = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/api/v1/webhooks")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

// =============================================================================
// Create Webhook Tests
// =============================================================================

#[tokio::test]
async fn test_create_webhook_admin_success() {
    let fixture = TestFixture::new();
    let app = fixture.create_test_router().with_state(fixture.dal.clone());
    let admin_pak = fixture.admin_pak.clone();

    let new_webhook = json!({
        "name": "Test Webhook",
        "url": "https://example.com/webhook",
        "event_types": ["deployment.applied", "deployment.failed"],
        "auth_header": "Bearer secret-token"
    });

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/webhooks")
                .header("Authorization", format!("Bearer {}", admin_pak))
                .header("Content-Type", "application/json")
                .body(Body::from(serde_json::to_string(&new_webhook).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::CREATED);
    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let result: serde_json::Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(result["name"], "Test Webhook");
    assert!(result["has_url"].as_bool().unwrap());
    assert!(result["has_auth_header"].as_bool().unwrap());
    assert!(result["enabled"].as_bool().unwrap());
    assert!(result["id"].is_string());
}

#[tokio::test]
async fn test_create_webhook_rejects_invalid_timeout() {
    // An out-of-range timeout_seconds must be rejected with 422, not stored
    // (a negative would later sign-extend to an absurd timeout) (BROKKR-T-0210).
    let fixture = TestFixture::new();
    let app = fixture.create_test_router().with_state(fixture.dal.clone());
    let admin_pak = fixture.admin_pak.clone();

    let new_webhook = json!({
        "name": "Bad Timeout",
        "url": "https://example.com/webhook",
        "event_types": ["deployment.applied"],
        "timeout_seconds": 0
    });

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/webhooks")
                .header("Authorization", format!("Bearer {}", admin_pak))
                .header("Content-Type", "application/json")
                .body(Body::from(serde_json::to_string(&new_webhook).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    let status = response.status();
    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let json_body: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(
        status,
        StatusCode::UNPROCESSABLE_ENTITY,
        "body: {json_body}"
    );
    assert_eq!(json_body["code"], "invalid_webhook");
}

#[tokio::test]
async fn test_create_webhook_with_wildcard_events() {
    let fixture = TestFixture::new();
    let app = fixture.create_test_router().with_state(fixture.dal.clone());
    let admin_pak = fixture.admin_pak.clone();

    let new_webhook = json!({
        "name": "All Deployment Events",
        "url": "https://example.com/webhook",
        "event_types": ["deployment.*"]
    });

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/webhooks")
                .header("Authorization", format!("Bearer {}", admin_pak))
                .header("Content-Type", "application/json")
                .body(Body::from(serde_json::to_string(&new_webhook).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::CREATED);
}

#[tokio::test]
async fn test_create_webhook_invalid_url() {
    let fixture = TestFixture::new();
    let app = fixture.create_test_router().with_state(fixture.dal.clone());
    let admin_pak = fixture.admin_pak.clone();

    let new_webhook = json!({
        "name": "Invalid URL Webhook",
        "url": "not-a-valid-url",
        "event_types": ["deployment.applied"]
    });

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/webhooks")
                .header("Authorization", format!("Bearer {}", admin_pak))
                .header("Content-Type", "application/json")
                .body(Body::from(serde_json::to_string(&new_webhook).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_create_webhook_non_admin_forbidden() {
    let fixture = TestFixture::new();
    let app = fixture.create_test_router().with_state(fixture.dal.clone());
    let (_, generator_pak) =
        fixture.create_test_generator_with_pak("Test Generator".to_string(), None);

    let new_webhook = json!({
        "name": "Test Webhook",
        "url": "https://example.com/webhook",
        "event_types": ["deployment.applied"]
    });

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/webhooks")
                .header("Authorization", format!("Bearer {}", generator_pak))
                .header("Content-Type", "application/json")
                .body(Body::from(serde_json::to_string(&new_webhook).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::FORBIDDEN);
}

// =============================================================================
// Get Webhook Tests
// =============================================================================

#[tokio::test]
async fn test_get_webhook_admin_success() {
    let fixture = TestFixture::new();
    let admin_pak = fixture.admin_pak.clone();

    // Create a webhook via DAL first
    use brokkr_models::models::webhooks::NewWebhookSubscription;
    let new_sub = NewWebhookSubscription {
        name: "Test Webhook".to_string(),
        url_encrypted: b"https://example.com/webhook".to_vec(),
        auth_header_encrypted: None,
        event_types: vec![Some("deployment.applied".to_string())],
        filters: None,
        target_labels: None,
        enabled: true,
        max_retries: 5,
        timeout_seconds: 30,
        created_by: Some("test".to_string()),
    };
    let subscription = fixture
        .dal
        .webhook_subscriptions()
        .create(&new_sub)
        .unwrap();

    let app = fixture.create_test_router().with_state(fixture.dal.clone());
    let response = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri(format!("/api/v1/webhooks/{}", subscription.id))
                .header("Authorization", format!("Bearer {}", admin_pak))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let result: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(result["id"], subscription.id.to_string());
    assert_eq!(result["name"], "Test Webhook");
}

#[tokio::test]
async fn test_get_webhook_not_found() {
    let fixture = TestFixture::new();
    let app = fixture.create_test_router().with_state(fixture.dal.clone());
    let admin_pak = fixture.admin_pak.clone();

    let fake_id = uuid::Uuid::new_v4();
    let response = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri(format!("/api/v1/webhooks/{}", fake_id))
                .header("Authorization", format!("Bearer {}", admin_pak))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}

// =============================================================================
// Update Webhook Tests
// =============================================================================

#[tokio::test]
async fn test_update_webhook_admin_success() {
    let fixture = TestFixture::new();
    let admin_pak = fixture.admin_pak.clone();

    // Create a webhook
    use brokkr_models::models::webhooks::NewWebhookSubscription;
    let new_sub = NewWebhookSubscription {
        name: "Original Name".to_string(),
        url_encrypted: b"https://example.com/webhook".to_vec(),
        auth_header_encrypted: None,
        event_types: vec![Some("deployment.applied".to_string())],
        filters: None,
        target_labels: None,
        enabled: true,
        max_retries: 5,
        timeout_seconds: 30,
        created_by: Some("test".to_string()),
    };
    let subscription = fixture
        .dal
        .webhook_subscriptions()
        .create(&new_sub)
        .unwrap();

    let app = fixture.create_test_router().with_state(fixture.dal.clone());

    let update_request = json!({
        "name": "Updated Name",
        "enabled": false
    });

    let response = app
        .oneshot(
            Request::builder()
                .method("PUT")
                .uri(format!("/api/v1/webhooks/{}", subscription.id))
                .header("Authorization", format!("Bearer {}", admin_pak))
                .header("Content-Type", "application/json")
                .body(Body::from(serde_json::to_string(&update_request).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let result: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(result["name"], "Updated Name");
    assert!(!result["enabled"].as_bool().unwrap());
}

// =============================================================================
// Delete Webhook Tests
// =============================================================================

#[tokio::test]
async fn test_delete_webhook_admin_success() {
    let fixture = TestFixture::new();
    let admin_pak = fixture.admin_pak.clone();

    // Create a webhook
    use brokkr_models::models::webhooks::NewWebhookSubscription;
    let new_sub = NewWebhookSubscription {
        name: "To Delete".to_string(),
        url_encrypted: b"https://example.com/webhook".to_vec(),
        auth_header_encrypted: None,
        event_types: vec![Some("deployment.applied".to_string())],
        filters: None,
        target_labels: None,
        enabled: true,
        max_retries: 5,
        timeout_seconds: 30,
        created_by: Some("test".to_string()),
    };
    let subscription = fixture
        .dal
        .webhook_subscriptions()
        .create(&new_sub)
        .unwrap();

    let app = fixture.create_test_router().with_state(fixture.dal.clone());

    let response = app
        .oneshot(
            Request::builder()
                .method("DELETE")
                .uri(format!("/api/v1/webhooks/{}", subscription.id))
                .header("Authorization", format!("Bearer {}", admin_pak))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::NO_CONTENT);

    // Verify deleted
    let deleted = fixture
        .dal
        .webhook_subscriptions()
        .get(subscription.id)
        .unwrap();
    assert!(deleted.is_none());
}

#[tokio::test]
async fn test_delete_webhook_not_found() {
    let fixture = TestFixture::new();
    let app = fixture.create_test_router().with_state(fixture.dal.clone());
    let admin_pak = fixture.admin_pak.clone();

    let fake_id = uuid::Uuid::new_v4();
    let response = app
        .oneshot(
            Request::builder()
                .method("DELETE")
                .uri(format!("/api/v1/webhooks/{}", fake_id))
                .header("Authorization", format!("Bearer {}", admin_pak))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}

// =============================================================================
// Event Types Tests
// =============================================================================

#[tokio::test]
async fn test_list_event_types_admin_success() {
    let fixture = TestFixture::new();
    let app = fixture.create_test_router().with_state(fixture.dal.clone());
    let admin_pak = fixture.admin_pak.clone();

    let response = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/api/v1/webhooks/event-types")
                .header("Authorization", format!("Bearer {}", admin_pak))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let event_types: Vec<String> = serde_json::from_slice(&body).unwrap();

    // Should contain known event types
    assert!(event_types.contains(&"deployment.applied".to_string()));
    assert!(event_types.contains(&"deployment.failed".to_string()));
    assert!(event_types.contains(&"workorder.completed".to_string()));
    assert!(event_types.contains(&"agent.registered".to_string()));
}

// =============================================================================
// Deliveries Tests
// =============================================================================

#[tokio::test]
async fn test_list_deliveries_admin_success() {
    let fixture = TestFixture::new();
    let admin_pak = fixture.admin_pak.clone();

    // Create a webhook
    use brokkr_models::models::webhooks::NewWebhookSubscription;
    let new_sub = NewWebhookSubscription {
        name: "Test Webhook".to_string(),
        url_encrypted: b"https://example.com/webhook".to_vec(),
        auth_header_encrypted: None,
        event_types: vec![Some("deployment.applied".to_string())],
        filters: None,
        target_labels: None,
        enabled: true,
        max_retries: 5,
        timeout_seconds: 30,
        created_by: Some("test".to_string()),
    };
    let subscription = fixture
        .dal
        .webhook_subscriptions()
        .create(&new_sub)
        .unwrap();

    let app = fixture.create_test_router().with_state(fixture.dal.clone());

    let response = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri(format!("/api/v1/webhooks/{}/deliveries", subscription.id))
                .header("Authorization", format!("Bearer {}", admin_pak))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let deliveries: Vec<serde_json::Value> = serde_json::from_slice(&body).unwrap();
    assert!(deliveries.is_empty()); // No deliveries yet
}

#[tokio::test]
async fn test_list_deliveries_with_status_filter() {
    let fixture = TestFixture::new();
    let admin_pak = fixture.admin_pak.clone();

    use brokkr_models::models::webhooks::{
        BrokkrEvent, NewWebhookDelivery, NewWebhookSubscription,
    };

    // Create a subscription
    let new_sub = NewWebhookSubscription {
        name: "Test Webhook".to_string(),
        url_encrypted: b"https://example.com/webhook".to_vec(),
        auth_header_encrypted: None,
        event_types: vec![Some("deployment.applied".to_string())],
        filters: None,
        target_labels: None,
        enabled: true,
        max_retries: 5,
        timeout_seconds: 30,
        created_by: Some("test".to_string()),
    };
    let subscription = fixture
        .dal
        .webhook_subscriptions()
        .create(&new_sub)
        .unwrap();

    // Create a delivery
    let event = BrokkrEvent::new("deployment.applied", json!({"test": true}));
    let new_delivery = NewWebhookDelivery::new(subscription.id, &event, None).unwrap();
    fixture
        .dal
        .webhook_deliveries()
        .create(&new_delivery)
        .unwrap();

    let app = fixture.create_test_router().with_state(fixture.dal.clone());

    let response = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri(format!(
                    "/api/v1/webhooks/{}/deliveries?status=pending",
                    subscription.id
                ))
                .header("Authorization", format!("Bearer {}", admin_pak))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let deliveries: Vec<serde_json::Value> = serde_json::from_slice(&body).unwrap();
    assert_eq!(deliveries.len(), 1);
    assert_eq!(deliveries[0]["status"], "pending");
}

#[tokio::test]
async fn test_list_deliveries_subscription_not_found() {
    let fixture = TestFixture::new();
    let app = fixture.create_test_router().with_state(fixture.dal.clone());
    let admin_pak = fixture.admin_pak.clone();

    let fake_id = uuid::Uuid::new_v4();
    let response = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri(format!("/api/v1/webhooks/{}/deliveries", fake_id))
                .header("Authorization", format!("Bearer {}", admin_pak))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}

// =============================================================================
// Agent Pending Delivery Tests (BROKKR-T-0302)
// =============================================================================

/// Produces ciphertext in the current storage format that the broker's active
/// key cannot decrypt, simulating a subscription written under a different
/// `BROKKR__BROKER__WEBHOOK_ENCRYPTION_KEY` (see BROKKR-T-0288). The fixture's
/// key is randomly generated per process, so this fixed foreign key never
/// matches it.
fn undecryptable_ciphertext(plaintext: &str) -> Vec<u8> {
    use brokkr_broker::utils::encryption::EncryptionKey;

    let foreign_key =
        EncryptionKey::from_hex("00112233445566778899aabbccddeeff00112233445566778899aabbccddeeff")
            .expect("valid foreign key");
    foreign_key
        .encrypt(plaintext.as_bytes())
        .expect("encryption succeeds")
}

/// Creates a label-targeted subscription plus one pending delivery for an agent
/// that carries the matching label, so the delivery is claimable on the agent
/// polling path. Returns `(agent_id, agent_pak, delivery_id)`.
fn setup_agent_targeted_delivery(
    fixture: &TestFixture,
    url_encrypted: Vec<u8>,
    auth_header_encrypted: Option<Vec<u8>>,
) -> (uuid::Uuid, String, uuid::Uuid) {
    setup_agent_targeted_delivery_with(fixture, url_encrypted, auth_header_encrypted, 5, 30)
}

/// As [`setup_agent_targeted_delivery`], with the subscription's retry budget
/// and timeout set explicitly.
fn setup_agent_targeted_delivery_with(
    fixture: &TestFixture,
    url_encrypted: Vec<u8>,
    auth_header_encrypted: Option<Vec<u8>>,
    max_retries: i32,
    timeout_seconds: i32,
) -> (uuid::Uuid, String, uuid::Uuid) {
    use brokkr_models::models::webhooks::{
        BrokkrEvent, NewWebhookDelivery, NewWebhookSubscription,
    };

    let (agent, agent_pak) = fixture
        .create_test_agent_with_pak("Webhook Agent".to_string(), "Webhook Cluster".to_string());
    fixture.create_test_agent_label(agent.id, "webhook-delivery".to_string());

    let target_labels = vec![Some("webhook-delivery".to_string())];
    let new_sub = NewWebhookSubscription {
        name: "Undecryptable Webhook".to_string(),
        url_encrypted,
        auth_header_encrypted,
        event_types: vec![Some("deployment.applied".to_string())],
        filters: None,
        target_labels: Some(target_labels.clone()),
        enabled: true,
        max_retries,
        timeout_seconds,
        created_by: Some("test".to_string()),
    };
    let subscription = fixture
        .dal
        .webhook_subscriptions()
        .create(&new_sub)
        .unwrap();

    let event = BrokkrEvent::new("deployment.applied", json!({"test": true}));
    let new_delivery =
        NewWebhookDelivery::new(subscription.id, &event, Some(target_labels)).unwrap();
    let delivery = fixture
        .dal
        .webhook_deliveries()
        .create(&new_delivery)
        .unwrap();

    (agent.id, agent_pak, delivery.id)
}

/// Polls the agent pending-webhooks endpoint and returns the decoded list.
async fn poll_pending(
    fixture: &TestFixture,
    agent_id: uuid::Uuid,
    agent_pak: &str,
) -> Vec<serde_json::Value> {
    let app = fixture.create_test_router().with_state(fixture.dal.clone());
    let response = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri(format!("/api/v1/agents/{}/webhooks/pending", agent_id))
                .header("Authorization", format!("Bearer {}", agent_pak))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    serde_json::from_slice(&body).unwrap()
}

#[tokio::test]
async fn test_pending_agent_webhooks_undecryptable_url_marks_delivery_dead() {
    // A URL that cannot be decrypted used to `continue` *after* the delivery was
    // claimed, leaving it acquired until the TTL lapsed, reclaimed on the next
    // poll, and failing forever with attempts pinned at 0 (BROKKR-T-0302).
    let fixture = TestFixture::new();
    let (agent_id, agent_pak, delivery_id) = setup_agent_targeted_delivery(
        &fixture,
        undecryptable_ciphertext("https://example.com/webhook"),
        None,
    );

    let pending = poll_pending(&fixture, agent_id, &agent_pak).await;
    assert!(
        pending.is_empty(),
        "undecryptable subscription must not be handed to the agent"
    );

    let delivery = fixture
        .dal
        .webhook_deliveries()
        .get(delivery_id)
        .unwrap()
        .expect("delivery still exists");
    assert_eq!(delivery.status, "dead");
    assert_eq!(delivery.attempts, 1);
    assert!(delivery.acquired_by.is_none(), "claim must be released");
    let last_error = delivery.last_error.expect("last_error recorded");
    assert!(
        last_error.contains("Failed to decrypt URL"),
        "decryption failure must be attributable, got: {}",
        last_error
    );

    // Second poll: a dead delivery is not claimable, so the reclaim loop ends.
    let pending = poll_pending(&fixture, agent_id, &agent_pak).await;
    assert!(pending.is_empty());
    let delivery = fixture
        .dal
        .webhook_deliveries()
        .get(delivery_id)
        .unwrap()
        .unwrap();
    assert_eq!(delivery.status, "dead");
    assert_eq!(delivery.attempts, 1, "delivery must not be retried");
}

#[tokio::test]
async fn test_pending_agent_webhooks_undecryptable_auth_header_never_delivers() {
    // An auth header that cannot be decrypted used to be swallowed to None,
    // dispatching the webhook without its configured Authorization header
    // (BROKKR-T-0302). A subscription configured with auth must never be
    // delivered without it.
    use brokkr_broker::utils::encryption::encrypt_string;

    let fixture = TestFixture::new();
    let (agent_id, agent_pak, delivery_id) = setup_agent_targeted_delivery(
        &fixture,
        encrypt_string("https://example.com/webhook").unwrap(),
        Some(undecryptable_ciphertext("Bearer secret-token")),
    );

    let pending = poll_pending(&fixture, agent_id, &agent_pak).await;
    assert!(
        pending.is_empty(),
        "a subscription with an undecryptable auth header must not be dispatched"
    );

    let delivery = fixture
        .dal
        .webhook_deliveries()
        .get(delivery_id)
        .unwrap()
        .expect("delivery still exists");
    assert_eq!(delivery.status, "dead");
    assert_eq!(delivery.attempts, 1);
    assert!(delivery.acquired_by.is_none(), "claim must be released");
    let last_error = delivery.last_error.expect("last_error recorded");
    assert!(
        last_error.contains("Failed to decrypt auth header"),
        "auth-header decryption failure must be distinguishable from a URL failure, got: {}",
        last_error
    );
}

/// Deletes a subscription row while leaving its deliveries behind.
///
/// `webhook_deliveries.subscription_id` is `ON DELETE CASCADE` (migration
/// `13_webhooks`), so a plain delete takes the deliveries with it and the
/// "subscription not found" arm is unreachable from the outside. In production
/// that arm is reached by a delete that lands *between* the claim and the
/// subscription fetch of a single poll, where the claimed rows are already in
/// memory; suppressing the FK trigger for one transaction reproduces the same
/// on-disk state deterministically. `SET LOCAL` is reverted when the
/// transaction ends, so the pooled connection is handed back untouched.
fn delete_subscription_without_cascade(fixture: &TestFixture, subscription_id: uuid::Uuid) {
    use brokkr_models::schema::webhook_subscriptions;
    use diesel::connection::SimpleConnection;
    use diesel::prelude::*;

    let mut conn = fixture.dal.conn().expect("db connection");
    conn.transaction::<_, diesel::result::Error, _>(|conn| {
        conn.batch_execute("SET LOCAL session_replication_role = replica")?;
        diesel::delete(
            webhook_subscriptions::table.filter(webhook_subscriptions::id.eq(subscription_id)),
        )
        .execute(conn)?;
        Ok(())
    })
    .expect("subscription deleted without cascading to its deliveries");
}

#[tokio::test]
async fn test_pending_agent_webhooks_missing_subscription_marks_delivery_dead() {
    // A delivery whose subscription is gone used to `continue` *after* the
    // delivery was claimed, leaving it acquired until the TTL lapsed, reclaimed
    // on the next poll, and failing forever with attempts pinned at 0
    // (BROKKR-T-0304). There is nothing left to deliver to, so it must die.
    use brokkr_broker::utils::encryption::encrypt_string;

    let fixture = TestFixture::new();
    let (agent_id, agent_pak, delivery_id) = setup_agent_targeted_delivery(
        &fixture,
        encrypt_string("https://example.com/webhook").unwrap(),
        None,
    );
    let subscription_id = fixture
        .dal
        .webhook_deliveries()
        .get(delivery_id)
        .unwrap()
        .expect("delivery created")
        .subscription_id;
    delete_subscription_without_cascade(&fixture, subscription_id);

    let pending = poll_pending(&fixture, agent_id, &agent_pak).await;
    assert!(
        !pending
            .iter()
            .any(|d| d["id"] == delivery_id.to_string().as_str()),
        "a delivery with no subscription must not be handed to the agent"
    );

    let delivery = fixture
        .dal
        .webhook_deliveries()
        .get(delivery_id)
        .unwrap()
        .expect("delivery still exists");
    assert_eq!(delivery.status, "dead");
    assert_eq!(delivery.attempts, 1);
    assert!(delivery.acquired_by.is_none(), "claim must be released");
    let last_error = delivery.last_error.expect("last_error recorded");
    assert!(
        last_error.contains("Subscription not found"),
        "the missing subscription must be attributable, got: {}",
        last_error
    );

    // Second poll: a dead delivery is not claimable, so the reclaim loop ends.
    let pending = poll_pending(&fixture, agent_id, &agent_pak).await;
    assert!(
        !pending
            .iter()
            .any(|d| d["id"] == delivery_id.to_string().as_str())
    );
    let delivery = fixture
        .dal
        .webhook_deliveries()
        .get(delivery_id)
        .unwrap()
        .unwrap();
    assert_eq!(delivery.status, "dead");
    assert_eq!(delivery.attempts, 1, "delivery must not be retried");
}

// =============================================================================
// Retry Classification Tests (BROKKR-T-0288)
// =============================================================================

/// Claims the delivery for the agent (so the result report is authorized) and
/// reports the given result back, returning the decoded response body.
async fn report_agent_delivery_result(
    fixture: &TestFixture,
    agent_pak: &str,
    delivery_id: uuid::Uuid,
    result: serde_json::Value,
) -> serde_json::Value {
    let app = fixture.create_test_router().with_state(fixture.dal.clone());
    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!("/api/v1/webhook-deliveries/{}/result", delivery_id))
                .header("Authorization", format!("Bearer {}", agent_pak))
                .header("Content-Type", "application/json")
                .body(Body::from(serde_json::to_string(&result).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    serde_json::from_slice(&body).unwrap()
}

#[tokio::test]
async fn test_report_delivery_result_non_retryable_4xx_is_dead_on_first_attempt() {
    // A 404 endpoint will answer identically on every attempt, so the delivery
    // must die immediately instead of burning all 5 attempts, and the status
    // must be recorded in last_error (BROKKR-T-0288).
    use brokkr_broker::utils::encryption::encrypt_string;

    let fixture = TestFixture::new();
    let (agent_id, agent_pak, delivery_id) = setup_agent_targeted_delivery(
        &fixture,
        encrypt_string("https://example.com/webhook").unwrap(),
        None,
    );

    let pending = poll_pending(&fixture, agent_id, &agent_pak).await;
    assert_eq!(pending.len(), 1, "delivery must be claimed by the agent");

    let reported = report_agent_delivery_result(
        &fixture,
        &agent_pak,
        delivery_id,
        json!({
            "success": false,
            "status_code": 404,
            "error": "HTTP 404: no such hook",
        }),
    )
    .await;
    assert_eq!(reported["status"], "dead");
    assert_eq!(reported["attempts"], 1);
    assert!(reported["next_retry_at"].is_null());

    let delivery = fixture
        .dal
        .webhook_deliveries()
        .get(delivery_id)
        .unwrap()
        .expect("delivery still exists");
    assert_eq!(delivery.status, "dead");
    assert_eq!(
        delivery.attempts, 1,
        "the subscription's 5-attempt budget must not be spent on a 404"
    );
    assert!(delivery.acquired_by.is_none(), "claim must be released");
    assert!(delivery.next_retry_at.is_none());
    let last_error = delivery.last_error.expect("last_error recorded");
    assert!(
        last_error.contains("404"),
        "the status code must be recorded in last_error, got: {}",
        last_error
    );
}

#[tokio::test]
async fn test_report_delivery_result_retryable_5xx_stays_failed() {
    // A 503 is the endpoint being broken, not the payload: the attempt counts
    // and the delivery stays retryable (BROKKR-T-0288).
    use brokkr_broker::utils::encryption::encrypt_string;

    let fixture = TestFixture::new();
    let (agent_id, agent_pak, delivery_id) = setup_agent_targeted_delivery(
        &fixture,
        encrypt_string("https://example.com/webhook").unwrap(),
        None,
    );

    let pending = poll_pending(&fixture, agent_id, &agent_pak).await;
    assert_eq!(pending.len(), 1);

    let reported = report_agent_delivery_result(
        &fixture,
        &agent_pak,
        delivery_id,
        json!({
            "success": false,
            "status_code": 503,
            "error": "HTTP 503: upstream unavailable",
        }),
    )
    .await;
    assert_eq!(reported["status"], "failed");
    assert_eq!(reported["attempts"], 1);
    assert!(
        !reported["next_retry_at"].is_null(),
        "a retryable failure must schedule the next attempt"
    );

    let delivery = fixture
        .dal
        .webhook_deliveries()
        .get(delivery_id)
        .unwrap()
        .expect("delivery still exists");
    assert_eq!(delivery.status, "failed");
    assert_eq!(delivery.attempts, 1);
    assert!(delivery.next_retry_at.is_some());
    let last_error = delivery.last_error.expect("last_error recorded");
    assert!(last_error.contains("503"), "got: {}", last_error);
}

#[tokio::test]
async fn test_report_delivery_result_transport_failure_stays_retryable() {
    // No status at all means the agent never reached the endpoint; that is a
    // transport failure and must keep retrying (BROKKR-T-0288).
    use brokkr_broker::utils::encryption::encrypt_string;

    let fixture = TestFixture::new();
    let (agent_id, agent_pak, delivery_id) = setup_agent_targeted_delivery(
        &fixture,
        encrypt_string("https://example.com/webhook").unwrap(),
        None,
    );

    let pending = poll_pending(&fixture, agent_id, &agent_pak).await;
    assert_eq!(pending.len(), 1);

    let reported = report_agent_delivery_result(
        &fixture,
        &agent_pak,
        delivery_id,
        json!({
            "success": false,
            "error": "Request failed: connection refused",
        }),
    )
    .await;
    assert_eq!(reported["status"], "failed");
    assert_eq!(reported["attempts"], 1);

    let delivery = fixture
        .dal
        .webhook_deliveries()
        .get(delivery_id)
        .unwrap()
        .unwrap();
    assert_eq!(delivery.status, "failed");
    assert_eq!(
        delivery.last_error.unwrap(),
        "Request failed: connection refused"
    );
}

#[tokio::test]
async fn test_report_delivery_result_records_status_without_error_body() {
    // Agents may report only a status code; last_error must still name it so a
    // dead delivery is diagnosable (BROKKR-T-0288).
    use brokkr_broker::utils::encryption::encrypt_string;

    let fixture = TestFixture::new();
    let (agent_id, agent_pak, delivery_id) = setup_agent_targeted_delivery(
        &fixture,
        encrypt_string("https://example.com/webhook").unwrap(),
        None,
    );

    let pending = poll_pending(&fixture, agent_id, &agent_pak).await;
    assert_eq!(pending.len(), 1);

    let reported = report_agent_delivery_result(
        &fixture,
        &agent_pak,
        delivery_id,
        json!({
            "success": false,
            "status_code": 401,
        }),
    )
    .await;
    assert_eq!(reported["status"], "dead");

    let delivery = fixture
        .dal
        .webhook_deliveries()
        .get(delivery_id)
        .unwrap()
        .unwrap();
    assert_eq!(delivery.last_error.unwrap(), "HTTP 401");
}

// =============================================================================
// Per-subscription Timeout Tests (BROKKR-T-0288)
// =============================================================================

#[tokio::test]
async fn test_pending_agent_webhooks_uses_subscription_timeout() {
    // The deliverer must receive the subscription's own timeout_seconds, not a
    // fixed default. (The broker worker's per-request timeout is covered by
    // `test_attempt_delivery_honors_per_request_timeout` in
    // `utils::background_tasks`; it cannot be observed through this HTTP
    // harness, which never runs the delivery worker.)
    use brokkr_broker::utils::encryption::encrypt_string;

    let fixture = TestFixture::new();
    let (agent_id, agent_pak, _delivery_id) = setup_agent_targeted_delivery_with(
        &fixture,
        encrypt_string("https://example.com/webhook").unwrap(),
        None,
        3,
        7,
    );

    let pending = poll_pending(&fixture, agent_id, &agent_pak).await;
    assert_eq!(pending.len(), 1);
    assert_eq!(pending[0]["timeout_seconds"], 7);
    assert_eq!(pending[0]["max_retries"], 3);
}

// =============================================================================
// Subscription Filter Tests (BROKKR-T-0288 item 1)
// =============================================================================
//
// `filters` used to be stored, echoed, and never evaluated: every subscription
// whose event *type* matched received every event. These tests drive the real
// create endpoint, then emit real events through the DAL, and assert on the
// deliveries the emission produced.
//
// The shared integration database is never reset between tests, but deliveries
// are listed per subscription id, so these counts are unaffected by rows other
// tests leave behind.

/// Creates a subscription through `POST /webhooks` and returns its id.
///
/// `filters` is passed through verbatim and the helper asserts a 201, so it is
/// only for bodies the API accepts. Rejected bodies (the removed `labels`) go
/// through [`post_webhook_body`], which returns the status instead.
async fn create_webhook_via_api(
    fixture: &TestFixture,
    name: &str,
    event_types: &[&str],
    filters: Option<serde_json::Value>,
) -> uuid::Uuid {
    let app = fixture.create_test_router().with_state(fixture.dal.clone());
    let mut body = json!({
        "name": name,
        "url": "https://example.com/webhook",
        "event_types": event_types,
    });
    if let Some(filters) = filters {
        body["filters"] = filters;
    }

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/webhooks")
                .header("Authorization", format!("Bearer {}", fixture.admin_pak))
                .header("Content-Type", "application/json")
                .body(Body::from(serde_json::to_string(&body).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    let status = response.status();
    let raw = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let created: serde_json::Value = serde_json::from_slice(&raw).unwrap();
    assert_eq!(status, StatusCode::CREATED, "body: {created}");

    uuid::Uuid::parse_str(created["id"].as_str().unwrap()).unwrap()
}

/// All deliveries queued for one subscription, oldest handling aside — the
/// per-subscription scope keeps this deterministic on the shared database.
fn deliveries_for(fixture: &TestFixture, subscription_id: uuid::Uuid) -> Vec<serde_json::Value> {
    fixture
        .dal
        .webhook_deliveries()
        .list_for_subscription(subscription_id, None, 100, 0)
        .unwrap()
        .into_iter()
        .map(|delivery| serde_json::from_str(&delivery.payload).unwrap())
        .collect()
}

#[tokio::test]
async fn test_webhook_filter_agent_id_delivers_only_matching_agent() {
    let fixture = TestFixture::new();
    let target = fixture.create_test_agent(
        format!("filter-target-{}", uuid::Uuid::new_v4()),
        format!("cluster-{}", uuid::Uuid::new_v4()),
    );
    let other = fixture.create_test_agent(
        format!("filter-other-{}", uuid::Uuid::new_v4()),
        format!("cluster-{}", uuid::Uuid::new_v4()),
    );

    let subscription_id = create_webhook_via_api(
        &fixture,
        "Agent Filtered",
        &["agent.deregistered"],
        Some(json!({"agent_id": target.id})),
    )
    .await;

    // Non-matching agent: right event type, wrong agent_id.
    fixture.dal.agents().soft_delete(other.id).unwrap();
    assert!(
        deliveries_for(&fixture, subscription_id).is_empty(),
        "a non-matching agent_id must not produce a delivery"
    );

    fixture.dal.agents().soft_delete(target.id).unwrap();
    let payloads = deliveries_for(&fixture, subscription_id);
    assert_eq!(payloads.len(), 1, "the matching agent must be delivered");
    assert_eq!(payloads[0]["data"]["agent_id"], json!(target.id));
}

#[tokio::test]
async fn test_webhook_filter_stack_id_delivers_only_matching_stack() {
    let fixture = TestFixture::new();
    let generator = fixture.create_test_generator(
        format!("filter-gen-{}", uuid::Uuid::new_v4()),
        None,
        "hash".to_string(),
    );
    let target = fixture.create_test_stack(
        format!("filter-target-{}", uuid::Uuid::new_v4()),
        None,
        generator.id,
    );
    let other = fixture.create_test_stack(
        format!("filter-other-{}", uuid::Uuid::new_v4()),
        None,
        generator.id,
    );

    let subscription_id = create_webhook_via_api(
        &fixture,
        "Stack Filtered",
        &["deployment.created"],
        Some(json!({"stack_id": target.id})),
    )
    .await;

    fixture.create_test_deployment_object(other.id, "key: other".to_string(), false);
    assert!(
        deliveries_for(&fixture, subscription_id).is_empty(),
        "a non-matching stack_id must not produce a delivery"
    );

    fixture.create_test_deployment_object(target.id, "key: target".to_string(), false);
    let payloads = deliveries_for(&fixture, subscription_id);
    assert_eq!(payloads.len(), 1, "the matching stack must be delivered");
    assert_eq!(payloads[0]["data"]["stack_id"], json!(target.id));
}

#[tokio::test]
async fn test_webhook_filter_field_absent_from_event_never_matches() {
    // The decided rule (BROKKR-T-0288): an event that does not carry the
    // filtered field does not match. `agent.deregistered` has no stack_id and
    // `deployment.created` has no agent_id, so each subscription below stays
    // empty even though its event type fires.
    let fixture = TestFixture::new();
    let generator = fixture.create_test_generator(
        format!("absent-gen-{}", uuid::Uuid::new_v4()),
        None,
        "hash".to_string(),
    );
    let stack = fixture.create_test_stack(
        format!("absent-stack-{}", uuid::Uuid::new_v4()),
        None,
        generator.id,
    );
    let agent = fixture.create_test_agent(
        format!("absent-agent-{}", uuid::Uuid::new_v4()),
        format!("cluster-{}", uuid::Uuid::new_v4()),
    );

    let stack_filtered_on_agent_event = create_webhook_via_api(
        &fixture,
        "Stack Filter On Agent Event",
        &["agent.deregistered"],
        Some(json!({"stack_id": stack.id})),
    )
    .await;
    let agent_filtered_on_deployment_event = create_webhook_via_api(
        &fixture,
        "Agent Filter On Deployment Event",
        &["deployment.created"],
        Some(json!({"agent_id": agent.id})),
    )
    .await;

    fixture.dal.agents().soft_delete(agent.id).unwrap();
    fixture.create_test_deployment_object(stack.id, "key: value".to_string(), false);

    assert!(
        deliveries_for(&fixture, stack_filtered_on_agent_event).is_empty(),
        "agent.deregistered carries no stack_id, so a stack_id filter excludes it"
    );
    assert!(
        deliveries_for(&fixture, agent_filtered_on_deployment_event).is_empty(),
        "deployment.created carries no agent_id, so an agent_id filter excludes it"
    );
}

/// Reports an agent event as the agent itself, which is what emits
/// `deployment.applied` (`status` "SUCCESS") or `deployment.failed` (anything
/// else). Asserts the 201 so callers only deal with the resulting deliveries.
async fn report_agent_event(
    fixture: &TestFixture,
    agent_id: uuid::Uuid,
    agent_pak: &str,
    deployment_object_id: uuid::Uuid,
    status: &str,
) {
    let app = fixture.create_test_router().with_state(fixture.dal.clone());
    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!("/api/v1/agents/{agent_id}/events"))
                .header("Authorization", format!("Bearer {agent_pak}"))
                .header("Content-Type", "application/json")
                .body(Body::from(
                    json!({
                        "agent_id": agent_id,
                        "deployment_object_id": deployment_object_id,
                        "event_type": "deploy",
                        "status": status,
                        "message": "reported by test",
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::CREATED);
}

#[tokio::test]
async fn test_webhook_filter_stack_id_delivers_deployment_applied() {
    // BROKKR-T-0305 replaced the exclusion this test used to pin: the emission
    // site now resolves the owning stack once, so `deployment.applied` carries
    // `stack_id` and a stack_id filter scopes apply results to a stack.
    let fixture = TestFixture::new();
    let generator = fixture.create_test_generator(
        format!("applied-gen-{}", uuid::Uuid::new_v4()),
        None,
        "hash".to_string(),
    );
    let stack = fixture.create_test_stack(
        format!("applied-stack-{}", uuid::Uuid::new_v4()),
        None,
        generator.id,
    );
    let other_stack = fixture.create_test_stack(
        format!("applied-other-{}", uuid::Uuid::new_v4()),
        None,
        generator.id,
    );
    let deployment_object =
        fixture.create_test_deployment_object(stack.id, "key: value".to_string(), false);
    let (agent, agent_pak) = fixture.create_test_agent_with_pak(
        format!("applied-agent-{}", uuid::Uuid::new_v4()),
        format!("cluster-{}", uuid::Uuid::new_v4()),
    );

    let stack_filtered = create_webhook_via_api(
        &fixture,
        "Applied Stack Filter",
        &["deployment.applied"],
        Some(json!({"stack_id": stack.id})),
    )
    .await;
    let other_stack_filtered = create_webhook_via_api(
        &fixture,
        "Applied Other Stack Filter",
        &["deployment.applied"],
        Some(json!({"stack_id": other_stack.id})),
    )
    .await;
    let agent_filtered = create_webhook_via_api(
        &fixture,
        "Applied Agent Filter",
        &["deployment.applied"],
        Some(json!({"agent_id": agent.id})),
    )
    .await;

    report_agent_event(
        &fixture,
        agent.id,
        &agent_pak,
        deployment_object.id,
        "SUCCESS",
    )
    .await;

    let payloads = deliveries_for(&fixture, stack_filtered);
    assert_eq!(
        payloads.len(),
        1,
        "deployment.applied now carries stack_id, so a stack_id filter matches it"
    );
    assert_eq!(payloads[0]["data"]["stack_id"], json!(stack.id));
    assert_eq!(
        payloads[0]["data"]["deployment_object_id"],
        json!(deployment_object.id)
    );

    assert!(
        deliveries_for(&fixture, other_stack_filtered).is_empty(),
        "a stack_id filter for a different stack must still exclude the event"
    );
    assert_eq!(
        deliveries_for(&fixture, agent_filtered).len(),
        1,
        "the same event still carries agent_id, so an agent_id filter matches"
    );
}

#[tokio::test]
async fn test_webhook_filter_stack_id_delivers_deployment_failed() {
    // Same emission site, the non-SUCCESS branch: a failed apply must be
    // scopable to its stack too.
    let fixture = TestFixture::new();
    let generator = fixture.create_test_generator(
        format!("failed-gen-{}", uuid::Uuid::new_v4()),
        None,
        "hash".to_string(),
    );
    let stack = fixture.create_test_stack(
        format!("failed-stack-{}", uuid::Uuid::new_v4()),
        None,
        generator.id,
    );
    let deployment_object =
        fixture.create_test_deployment_object(stack.id, "key: value".to_string(), false);
    let (agent, agent_pak) = fixture.create_test_agent_with_pak(
        format!("failed-agent-{}", uuid::Uuid::new_v4()),
        format!("cluster-{}", uuid::Uuid::new_v4()),
    );

    let stack_filtered = create_webhook_via_api(
        &fixture,
        "Failed Stack Filter",
        &["deployment.failed"],
        Some(json!({"stack_id": stack.id})),
    )
    .await;

    report_agent_event(
        &fixture,
        agent.id,
        &agent_pak,
        deployment_object.id,
        "FAILURE",
    )
    .await;

    let payloads = deliveries_for(&fixture, stack_filtered);
    assert_eq!(
        payloads.len(),
        1,
        "deployment.failed now carries stack_id, so a stack_id filter matches it"
    );
    assert_eq!(payloads[0]["event_type"], json!("deployment.failed"));
    assert_eq!(payloads[0]["data"]["stack_id"], json!(stack.id));
}

#[tokio::test]
async fn test_deployment_applied_stack_id_is_null_when_object_soft_deleted() {
    // The object can be soft-deleted between the apply and the agent reporting
    // it. `stack_id` is then emitted as JSON `null` rather than omitted, which
    // the filter predicate treats as absent — so an unfiltered subscription
    // still sees the event, and a stack_id filter does not match it.
    let fixture = TestFixture::new();
    let generator = fixture.create_test_generator(
        format!("gone-gen-{}", uuid::Uuid::new_v4()),
        None,
        "hash".to_string(),
    );
    let stack = fixture.create_test_stack(
        format!("gone-stack-{}", uuid::Uuid::new_v4()),
        None,
        generator.id,
    );
    let deployment_object =
        fixture.create_test_deployment_object(stack.id, "key: value".to_string(), false);
    let (agent, agent_pak) = fixture.create_test_agent_with_pak(
        format!("gone-agent-{}", uuid::Uuid::new_v4()),
        format!("cluster-{}", uuid::Uuid::new_v4()),
    );

    // Subscribed to `deployment.applied` only, so the soft delete below does not
    // add a `deployment.deleted` delivery to either subscription.
    let unfiltered =
        create_webhook_via_api(&fixture, "Gone Unfiltered", &["deployment.applied"], None).await;
    let stack_filtered = create_webhook_via_api(
        &fixture,
        "Gone Stack Filter",
        &["deployment.applied"],
        Some(json!({"stack_id": stack.id})),
    )
    .await;

    fixture
        .dal
        .deployment_objects()
        .soft_delete(deployment_object.id)
        .unwrap();

    report_agent_event(
        &fixture,
        agent.id,
        &agent_pak,
        deployment_object.id,
        "SUCCESS",
    )
    .await;

    let payloads = deliveries_for(&fixture, unfiltered);
    assert_eq!(payloads.len(), 1);
    assert!(
        payloads[0]["data"].get("stack_id").is_some(),
        "the key must be present rather than omitted"
    );
    assert_eq!(
        payloads[0]["data"]["stack_id"],
        json!(null),
        "an unresolvable deployment object emits stack_id: null"
    );
    assert!(
        deliveries_for(&fixture, stack_filtered).is_empty(),
        "null counts as absent, so a stack_id filter does not match"
    );
}

#[tokio::test]
async fn test_unfiltered_webhook_subscription_receives_every_matching_event() {
    // Regression guard: adding filter evaluation must not change delivery for
    // the subscriptions that set no filters.
    let fixture = TestFixture::new();

    let no_filters_key = create_webhook_via_api(&fixture, "Unfiltered", &["agent.*"], None).await;
    let empty_filters_object =
        create_webhook_via_api(&fixture, "Empty Filters", &["agent.*"], Some(json!({}))).await;

    let first = fixture.create_test_agent(
        format!("unfiltered-a-{}", uuid::Uuid::new_v4()),
        format!("cluster-{}", uuid::Uuid::new_v4()),
    );
    let second = fixture.create_test_agent(
        format!("unfiltered-b-{}", uuid::Uuid::new_v4()),
        format!("cluster-{}", uuid::Uuid::new_v4()),
    );
    fixture.dal.agents().soft_delete(first.id).unwrap();
    fixture.dal.agents().soft_delete(second.id).unwrap();

    // Two registrations plus two deregistrations, on both subscriptions.
    assert_eq!(deliveries_for(&fixture, no_filters_key).len(), 4);
    assert_eq!(deliveries_for(&fixture, empty_filters_object).len(), 4);
}

/// Sends a raw create body and returns `(status, parsed body)`.
async fn post_webhook_body(
    fixture: &TestFixture,
    body: serde_json::Value,
) -> (StatusCode, serde_json::Value) {
    let app = fixture.create_test_router().with_state(fixture.dal.clone());
    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/webhooks")
                .header("Authorization", format!("Bearer {}", fixture.admin_pak))
                .header("Content-Type", "application/json")
                .body(Body::from(body.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    let status = response.status();
    let raw = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    (status, serde_json::from_slice(&raw).unwrap())
}

#[tokio::test]
async fn test_create_webhook_rejects_legacy_filter_labels_and_validate() {
    // Was `test_create_webhook_drops_legacy_filter_labels_and_validate`:
    // `filters.labels` and `validate` were accepted and dropped. Dropping them
    // was worse than failing — a caller who sends `filters.labels` believes the
    // subscription is scoped and it is not — so both are now 422, naming the
    // offending field and its replacement.
    let fixture = TestFixture::new();
    let agent_id = uuid::Uuid::new_v4();
    // Unique so the "nothing was persisted" check below cannot collide with a
    // row left by an earlier run on the shared, never-reset integration database.
    let name = format!("Legacy Client {}", uuid::Uuid::new_v4());

    let (status, body) = post_webhook_body(
        &fixture,
        json!({
            "name": name,
            "url": "https://example.com/webhook",
            "event_types": ["agent.registered"],
            "validate": true,
            "filters": {
                "agent_id": agent_id,
                "labels": {"env": "prod"},
            },
        }),
    )
    .await;

    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY, "body: {body}");
    assert_eq!(body["code"], json!("unsupported_field"));
    assert_eq!(body["details"]["field"], json!("validate"));
    assert!(
        body["message"]
            .as_str()
            .unwrap()
            .contains("/webhooks/{id}/test"),
        "the error must point at the real test endpoint: {body}"
    );

    // Same body without `validate`: now `filters.labels` is the offender.
    let (status, body) = post_webhook_body(
        &fixture,
        json!({
            "name": name,
            "url": "https://example.com/webhook",
            "event_types": ["agent.registered"],
            "filters": {
                "agent_id": agent_id,
                "labels": {"env": "prod"},
            },
        }),
    )
    .await;

    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY, "body: {body}");
    assert_eq!(body["code"], json!("unsupported_field"));
    assert_eq!(body["details"]["field"], json!("filters.labels"));
    assert_eq!(body["details"]["use_instead"], json!("target_labels"));
    assert!(
        body["message"].as_str().unwrap().contains("target_labels"),
        "the error must point at target_labels: {body}"
    );

    // Nothing was persisted by either rejected request.
    let named: Vec<_> = fixture
        .dal
        .webhook_subscriptions()
        .list(false)
        .unwrap()
        .into_iter()
        .filter(|sub| sub.name == name)
        .collect();
    assert!(
        named.is_empty(),
        "a rejected create must not store a subscription"
    );
}

#[tokio::test]
async fn test_create_webhook_accepts_supported_filters_unchanged() {
    // The rejection must be narrow: the surviving filter fields still work, and
    // `target_labels` — the replacement the 422 advertises — is still accepted.
    let fixture = TestFixture::new();
    let agent_id = uuid::Uuid::new_v4();

    let (status, created) = post_webhook_body(
        &fixture,
        json!({
            "name": format!("Supported Filters {}", uuid::Uuid::new_v4()),
            "url": "https://example.com/webhook",
            "event_types": ["agent.registered"],
            "filters": {"agent_id": agent_id},
            "target_labels": ["env:prod"],
        }),
    )
    .await;

    assert_eq!(status, StatusCode::CREATED, "body: {created}");
    assert_eq!(created["filters"]["agent_id"], json!(agent_id));
    assert_eq!(created["target_labels"], json!(["env:prod"]));
}

#[tokio::test]
async fn test_update_webhook_rejects_legacy_filter_labels() {
    // `filters` is updatable, so the write-path rule has to hold on PUT too —
    // otherwise a caller could reintroduce the key the create path refuses.
    let fixture = TestFixture::new();
    let subscription_id = create_webhook_via_api(
        &fixture,
        "Update Legacy Labels",
        &["agent.registered"],
        None,
    )
    .await;

    let app = fixture.create_test_router().with_state(fixture.dal.clone());
    let response = app
        .oneshot(
            Request::builder()
                .method("PUT")
                .uri(format!("/api/v1/webhooks/{}", subscription_id))
                .header("Authorization", format!("Bearer {}", fixture.admin_pak))
                .header("Content-Type", "application/json")
                .body(Body::from(
                    json!({"filters": {"stack_id": uuid::Uuid::new_v4(), "labels": ["env:prod"]}})
                        .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    let status = response.status();
    let raw = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let body: serde_json::Value = serde_json::from_slice(&raw).unwrap();

    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY, "body: {body}");
    assert_eq!(body["code"], json!("unsupported_field"));
    assert_eq!(body["details"]["field"], json!("filters.labels"));

    // The rejected update changed nothing.
    let stored = fixture
        .dal
        .webhook_subscriptions()
        .get(subscription_id)
        .unwrap()
        .unwrap();
    assert!(
        stored.filters.is_none(),
        "a rejected update must not persist filters: {:?}",
        stored.filters
    );
}

#[tokio::test]
async fn test_legacy_stored_labels_filter_still_delivers() {
    // A row written before `labels` was dropped deserializes to an empty filter
    // and must keep delivering, not stall.
    use brokkr_models::models::webhooks::NewWebhookSubscription;

    let fixture = TestFixture::new();
    let subscription = fixture
        .dal
        .webhook_subscriptions()
        .create(&NewWebhookSubscription {
            name: "Legacy Labels Filter".to_string(),
            url_encrypted: b"https://example.com/webhook".to_vec(),
            auth_header_encrypted: None,
            event_types: vec![Some("agent.registered".to_string())],
            filters: Some(r#"{"labels":{"env":"prod"}}"#.to_string()),
            target_labels: None,
            enabled: true,
            max_retries: 5,
            timeout_seconds: 30,
            created_by: Some("test".to_string()),
        })
        .unwrap();

    fixture.create_test_agent(
        format!("legacy-agent-{}", uuid::Uuid::new_v4()),
        format!("cluster-{}", uuid::Uuid::new_v4()),
    );

    assert_eq!(
        deliveries_for(&fixture, subscription.id).len(),
        1,
        "a legacy labels-only filter must not stop deliveries"
    );
}
