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
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY, "body: {json_body}");
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
        max_retries: 5,
        timeout_seconds: 30,
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
