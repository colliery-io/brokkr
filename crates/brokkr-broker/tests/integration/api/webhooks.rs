/*
 * Copyright (c) 2025 Dylan Storey
 * Licensed under the Elastic License 2.0.
 * See LICENSE file in the project root for full license text.
 */

use crate::fixtures::TestFixture;
use axum::{
    body::{to_bytes, Body},
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
    let subscription = fixture.dal.webhook_subscriptions().create(&new_sub).unwrap();

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
    let subscription = fixture.dal.webhook_subscriptions().create(&new_sub).unwrap();

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
    let subscription = fixture.dal.webhook_subscriptions().create(&new_sub).unwrap();

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
    let deleted = fixture.dal.webhook_subscriptions().get(subscription.id).unwrap();
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
    let subscription = fixture.dal.webhook_subscriptions().create(&new_sub).unwrap();

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

    use brokkr_models::models::webhooks::{BrokkrEvent, NewWebhookDelivery, NewWebhookSubscription};

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
    let subscription = fixture.dal.webhook_subscriptions().create(&new_sub).unwrap();

    // Create a delivery
    let event = BrokkrEvent::new("deployment.applied", json!({"test": true}));
    let new_delivery = NewWebhookDelivery::new(subscription.id, &event, None).unwrap();
    fixture.dal.webhook_deliveries().create(&new_delivery).unwrap();

    let app = fixture.create_test_router().with_state(fixture.dal.clone());

    let response = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri(format!("/api/v1/webhooks/{}/deliveries?status=pending", subscription.id))
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
