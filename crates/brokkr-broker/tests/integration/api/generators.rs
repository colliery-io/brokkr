use crate::fixtures::TestFixture;
use axum::{
    body::{to_bytes, Body},
    http::{Request, StatusCode},
};
use brokkr_models::models::generator::{Generator, NewGenerator};
use tower::ServiceExt;

#[tokio::test]
async fn test_list_generators_admin_success() {
    let fixture = TestFixture::new();
    let app = fixture.create_test_router().with_state(fixture.dal.clone());
    let admin_pak = fixture.admin_pak.clone();

    fixture.create_test_generator(
        "Test Generator".to_string(),
        None,
        "test_api_key_hash".to_string(),
    );

    let response = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/api/v1/generators")
                .header("Authorization", format!("Bearer {}", admin_pak))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let generators: Vec<Generator> = serde_json::from_slice(&body).unwrap();
    assert!(!generators.is_empty());
}

#[tokio::test]
async fn test_list_generators_non_admin_forbidden() {
    let fixture = TestFixture::new();
    let app = fixture.create_test_router().with_state(fixture.dal.clone());
    let (_, generator_pak) =
        fixture.create_test_generator_with_pak("Test Generator".to_string(), None);

    let response = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/api/v1/generators")
                .header("Authorization", format!("Bearer {}", generator_pak))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn test_create_generator_admin_success() {
    let fixture = TestFixture::new();
    let app = fixture.create_test_router().with_state(fixture.dal.clone());
    let admin_pak = fixture.admin_pak.clone();

    let new_generator = NewGenerator {
        name: "New Generator".to_string(),
        description: Some("Test Description".to_string()),
    };

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/generators")
                .header("Authorization", format!("Bearer {}", admin_pak))
                .header("Content-Type", "application/json")
                .body(Body::from(serde_json::to_string(&new_generator).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let result: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert!(result.get("generator").is_some());
    assert!(result.get("pak").is_some());
}

#[tokio::test]
async fn test_get_generator_admin_success() {
    let fixture = TestFixture::new();
    let app = fixture.create_test_router().with_state(fixture.dal.clone());
    let admin_pak = fixture.admin_pak.clone();

    let generator = fixture.create_test_generator(
        "Test Generator".to_string(),
        None,
        "test_api_key_hash".to_string(),
    );

    let response = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri(format!("/api/v1/generators/{}", generator.id))
                .header("Authorization", format!("Bearer {}", admin_pak))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let fetched_generator: Generator = serde_json::from_slice(&body).unwrap();
    assert_eq!(fetched_generator.id, generator.id);
}

#[tokio::test]
async fn test_get_generator_self_success() {
    let fixture = TestFixture::new();
    let app = fixture.create_test_router().with_state(fixture.dal.clone());
    let (generator, generator_pak) =
        fixture.create_test_generator_with_pak("Test Generator".to_string(), None);

    let response = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri(format!("/api/v1/generators/{}", generator.id))
                .header("Authorization", format!("Bearer {}", generator_pak))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let fetched_generator: Generator = serde_json::from_slice(&body).unwrap();
    assert_eq!(fetched_generator.id, generator.id);
}

#[tokio::test]
async fn test_update_generator_admin_success() {
    let fixture = TestFixture::new();
    let app = fixture.create_test_router().with_state(fixture.dal.clone());
    let admin_pak = fixture.admin_pak.clone();

    let generator = fixture.create_test_generator(
        "Test Generator".to_string(),
        None,
        "test_api_key_hash".to_string(),
    );

    let mut updated_generator = generator.clone();
    updated_generator.name = "Updated Generator".to_string();

    let response = app
        .oneshot(
            Request::builder()
                .method("PUT")
                .uri(format!("/api/v1/generators/{}", generator.id))
                .header("Authorization", format!("Bearer {}", admin_pak))
                .header("Content-Type", "application/json")
                .body(Body::from(
                    serde_json::to_string(&updated_generator).unwrap(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let result: Generator = serde_json::from_slice(&body).unwrap();
    assert_eq!(result.name, "Updated Generator");
}

#[tokio::test]
async fn test_delete_generator_admin_success() {
    let fixture = TestFixture::new();
    let app = fixture.create_test_router().with_state(fixture.dal.clone());
    let admin_pak = fixture.admin_pak.clone();

    let generator = fixture.create_test_generator(
        "Test Generator".to_string(),
        None,
        "test_api_key_hash".to_string(),
    );

    let response = app
        .oneshot(
            Request::builder()
                .method("DELETE")
                .uri(format!("/api/v1/generators/{}", generator.id))
                .header("Authorization", format!("Bearer {}", admin_pak))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::NO_CONTENT);
}

#[tokio::test]
async fn test_delete_generator_self_success() {
    let fixture = TestFixture::new();
    let app = fixture.create_test_router().with_state(fixture.dal.clone());
    let (generator, generator_pak) =
        fixture.create_test_generator_with_pak("Test Generator".to_string(), None);

    let response = app
        .oneshot(
            Request::builder()
                .method("DELETE")
                .uri(format!("/api/v1/generators/{}", generator.id))
                .header("Authorization", format!("Bearer {}", generator_pak))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::NO_CONTENT);
}

#[tokio::test]
async fn test_list_generators_unauthorized() {
    let fixture = TestFixture::new();
    let app = fixture.create_test_router().with_state(fixture.dal.clone());

    let response = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/api/v1/generators")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_create_generator_unauthorized() {
    let fixture = TestFixture::new();
    let app = fixture.create_test_router().with_state(fixture.dal.clone());

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/generators")
                .header("Content-Type", "application/json")
                .body(Body::from("{}"))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_get_generator_unauthorized() {
    let fixture = TestFixture::new();
    let app = fixture.create_test_router().with_state(fixture.dal.clone());
    let generator = fixture.create_test_generator(
        "Test Generator".to_string(),
        None,
        "test_api_key_hash".to_string(),
    );

    let response = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri(format!("/api/v1/generators/{}", generator.id))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_update_generator_unauthorized() {
    let fixture = TestFixture::new();
    let app = fixture.create_test_router().with_state(fixture.dal.clone());
    let generator = fixture.create_test_generator(
        "Test Generator".to_string(),
        None,
        "test_api_key_hash".to_string(),
    );

    let response = app
        .oneshot(
            Request::builder()
                .method("PUT")
                .uri(format!("/api/v1/generators/{}", generator.id))
                .header("Content-Type", "application/json")
                .body(Body::from("{}"))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_delete_generator_unauthorized() {
    let fixture = TestFixture::new();
    let app = fixture.create_test_router().with_state(fixture.dal.clone());
    let generator = fixture.create_test_generator(
        "Test Generator".to_string(),
        None,
        "test_api_key_hash".to_string(),
    );

    let response = app
        .oneshot(
            Request::builder()
                .method("DELETE")
                .uri(format!("/api/v1/generators/{}", generator.id))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}
