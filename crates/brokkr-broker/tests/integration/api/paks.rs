/*
 * Copyright (c) 2025-2026 Dylan Storey
 * Licensed under the Elastic License 2.0.
 * See LICENSE file in the project root for full license text.
 */

//! Integration tests for `GET /api/v1/paks` (BROKKR-T-0269).

use axum::{
    body::{Body, to_bytes},
    http::{Request, StatusCode},
};
use tower::ServiceExt;

use brokkr_broker::utils::ui_pak;

use crate::fixtures::TestFixture;

async fn get_paks(app: axum::Router, token: &str) -> (StatusCode, Option<Vec<serde_json::Value>>) {
    let response = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/api/v1/paks")
                .header("Authorization", format!("Bearer {token}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    let status = response.status();
    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let parsed = serde_json::from_slice::<Vec<serde_json::Value>>(&body).ok();
    (status, parsed)
}

#[tokio::test]
async fn test_list_paks_returns_id_name_pairs() {
    let fixture = TestFixture::new();
    let app = fixture.create_test_router().with_state(fixture.dal.clone());

    let (team_payments, _) =
        fixture.create_test_generator_with_pak("team-payments".to_string(), None);

    let (status, body) = get_paks(app, &fixture.admin_pak).await;
    assert_eq!(status, StatusCode::OK);
    let paks = body.expect("body must be a JSON array");

    let entry = paks
        .iter()
        .find(|p| p["id"] == team_payments.id.to_string())
        .expect("created generator must be listed");
    assert_eq!(entry["name"], "team-payments");
    // Slim DTO: identity only, no pak_hash or other generator fields.
    assert_eq!(
        entry.as_object().unwrap().keys().collect::<Vec<_>>(),
        vec!["id", "name"]
    );
}

#[tokio::test]
async fn test_list_paks_excludes_system_generator() {
    let fixture = TestFixture::new();
    let app = fixture.create_test_router().with_state(fixture.dal.clone());

    let system_id = fixture
        .dal
        .generators()
        .get_system_generator_id()
        .expect("system generator lookup")
        .expect("system generator exists");

    let (status, body) = get_paks(app, &fixture.admin_pak).await;
    assert_eq!(status, StatusCode::OK);
    let paks = body.unwrap();
    assert!(
        !paks.iter().any(|p| p["id"] == system_id.to_string()),
        "system generator must not be listed as a tenant"
    );
}

#[tokio::test]
async fn test_list_paks_allows_ui_pak_rejects_non_admin() {
    let fixture = TestFixture::new();
    let app = fixture.create_test_router().with_state(fixture.dal.clone());

    // The read-only UI PAK is a readonly admin: allowed.
    let ui_token = ui_pak::token().expect("UI PAK minted").to_string();
    let (status, _) = get_paks(app.clone(), &ui_token).await;
    assert_eq!(status, StatusCode::OK);

    // A generator PAK is not an admin: 403.
    let (_generator, generator_pak) =
        fixture.create_test_generator_with_pak("not-admin".to_string(), None);
    let (status, _) = get_paks(app.clone(), &generator_pak).await;
    assert_eq!(status, StatusCode::FORBIDDEN);

    // An agent PAK is not an admin: 403.
    let (_agent, agent_pak) =
        fixture.create_test_agent_with_pak("paks-agent".to_string(), "paks-cluster".to_string());
    let (status, _) = get_paks(app, &agent_pak).await;
    assert_eq!(status, StatusCode::FORBIDDEN);
}
