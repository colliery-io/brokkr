use crate::fixtures::TestFixture;
use axum::{
    body::{to_bytes, Body},
    http::{Request, StatusCode},
};
use tower::ServiceExt;
use uuid::Uuid;
use brokkr_broker::utils::pak::create_pak;

#[tokio::test]
async fn test_get_deployment_object_admin_success() {
    let fixture = TestFixture::new();
    let app = fixture.create_test_router().with_state(fixture.dal.clone());
    let admin_pak = fixture.admin_pak.clone();

    let stack = fixture.create_test_stack("Test Stack".to_string(), None, fixture.admin_generator.id);
    let deployment_object = fixture.create_test_deployment_object(stack.id, "test yaml".to_string(), false);

    let response = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri(format!("/api/v1/deployment-objects/{}", deployment_object.id))
                .header("Authorization", format!("Bearer {}", admin_pak))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let fetched_object: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(fetched_object["id"], deployment_object.id.to_string());
}

#[tokio::test]
async fn test_get_deployment_object_agent_success() {
    let fixture = TestFixture::new();
    let app = fixture.create_test_router().with_state(fixture.dal.clone());
    let (agent, agent_pak) = fixture.create_test_agent_with_pak("Test Agent".to_string(), "Test Cluster".to_string());

    let stack = fixture.create_test_stack("Test Stack".to_string(), None, fixture.admin_generator.id);
    let deployment_object = fixture.create_test_deployment_object(stack.id, "test yaml".to_string(), false);
    fixture.create_test_agent_target(agent.id, stack.id);

    let response = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri(format!("/api/v1/deployment-objects/{}", deployment_object.id))
                .header("Authorization", format!("Bearer {}", agent_pak))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let fetched_object: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(fetched_object["id"], deployment_object.id.to_string());
}

#[tokio::test]
async fn test_get_deployment_object_generator_success() {
    let fixture = TestFixture::new();
    let app = fixture.create_test_router().with_state(fixture.dal.clone());
    
    let (pak, hash) = create_pak().unwrap();

    let generator = fixture.create_test_generator(
        "Test Generator".to_string(),
        Some("Test Description".to_string()),
        hash,
    );

    let stack = fixture.create_test_stack("Test Stack".to_string(), None, generator.id);
    let deployment_object = fixture.create_test_deployment_object(stack.id, "test yaml".to_string(), false);

    let response = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri(format!("/api/v1/deployment-objects/{}", deployment_object.id))
                .header("Authorization", format!("Bearer {}", pak))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let fetched_object: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(fetched_object["id"], deployment_object.id.to_string());
}

#[tokio::test]
async fn test_get_deployment_object_agent_forbidden() {
    let fixture = TestFixture::new();
    let app = fixture.create_test_router().with_state(fixture.dal.clone());
    let (agent, agent_pak) = fixture.create_test_agent_with_pak("Test Agent".to_string(), "Test Cluster".to_string());

    let stack = fixture.create_test_stack("Test Stack".to_string(), None, fixture.admin_generator.id);
    let deployment_object = fixture.create_test_deployment_object(stack.id, "test yaml".to_string(), false);
    // Note: We're not creating an agent target, so the agent shouldn't have access

    let response = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri(format!("/api/v1/deployment-objects/{}", deployment_object.id))
                .header("Authorization", format!("Bearer {}", agent_pak))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn test_get_deployment_object_generator_forbidden() {
    let fixture = TestFixture::new();
    let app = fixture.create_test_router().with_state(fixture.dal.clone());
    
    let (generator1_pak, generator1_hash) = create_pak().unwrap();
    let (generator2_pak, generator2_hash) = create_pak().unwrap();

    let generator1 = fixture.create_test_generator(
        "Test Generator 1".to_string(),
        Some("Test Description 1".to_string()),
        generator1_hash
    );
    

    let generator2 = fixture.create_test_generator(
        "Test Generator 2".to_string(),
        Some("Test Description 2".to_string()),
        generator2_hash
    );

    let stack = fixture.create_test_stack("Test Stack".to_string(), None, generator2.id);
    let deployment_object = fixture.create_test_deployment_object(stack.id, "test yaml".to_string(), false);

    let response = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri(format!("/api/v1/deployment-objects/{}", deployment_object.id))
                .header("Authorization", format!("Bearer {}", generator1_pak))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn test_get_deployment_object_not_found() {
    let fixture = TestFixture::new();
    let app = fixture.create_test_router().with_state(fixture.dal.clone());
    let admin_pak = fixture.admin_pak.clone();

    let non_existent_id = Uuid::new_v4();
    let response = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri(format!("/api/v1/deployment-objects/{}", non_existent_id))
                .header("Authorization", format!("Bearer {}", admin_pak))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_get_deployment_object_unauthorized() {
    let fixture = TestFixture::new();
    let app = fixture.create_test_router().with_state(fixture.dal.clone());

    let stack = fixture.create_test_stack("Test Stack".to_string(), None, fixture.admin_generator.id);
    let deployment_object = fixture.create_test_deployment_object(stack.id, "test yaml".to_string(), false);

    let response = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri(format!("/api/v1/deployment-objects/{}", deployment_object.id))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}