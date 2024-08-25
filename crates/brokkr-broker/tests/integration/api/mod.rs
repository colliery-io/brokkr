mod agents;
mod stacks;
mod deployment_objects;
mod agent_events;


use axum::{
    body::Body,
    http::{Request, StatusCode, Method},
};

use tower::ServiceExt;

use crate::fixtures::TestFixture;


#[tokio::test]
async fn test_api_routes_configuration() {
    let fixture = TestFixture::new(); // Use default DATABASE_URL
    let _app = fixture.create_test_router();

    // Your test cases here...
}


#[tokio::test]
async fn test_healthz_endpoint() {
    let fixture = TestFixture::new();
    let app = fixture.create_test_router();

 
    // Send the request and get the response
    let get_response = app
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
    assert_eq!(get_response.status(), StatusCode::OK);

    // Get the response body and confirm it says "OK"
    let body = hyper::body::to_bytes(get_response.into_body()).await.unwrap();
    assert_eq!(&body[..], b"OK");
}