// mod agent_events;
// mod agents;
// mod deployment_objects;
mod stacks;

use axum::{
    body::{to_bytes, Body},
    http::{Request, StatusCode},
};

// use tower::ServiceExt;
use tower::util::ServiceExt;
use crate::fixtures::TestFixture;

// #[tokio::test]
// async fn test_api_routes_configuration() {
//     let fixture = TestFixture::new(); // Use default DATABASE_URL
//     let _app = fixture.create_test_router();
// }

#[tokio::test]
async fn test_healthz_endpoint() {
    let fixture = TestFixture::new();
    let app = fixture.create_test_router();

    // Send the request and get the response
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/healthz")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    // Assert that the response status is 200 OK
    assert_eq!(response.status(), StatusCode::OK);

    // Get the response body and confirm it says "OK"
    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    assert_eq!(&body[..], b"OK");
}
