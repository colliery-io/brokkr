mod agents;
mod stacks;
mod deployment_objects;
mod agent_events;
use axum::{
    body::Body,
    http::{Request, StatusCode, Method},
};
use brokkr_broker::{api, dal::DAL};
use crate::fixtures::TestFixture;
use tower::ServiceExt; // for `oneshot` and other extensions
use uuid::Uuid;


#[tokio::test]
async fn test_api_routes_configuration() {
    let fixture = TestFixture::new(); // Use default DATABASE_URL
    let app = fixture.create_test_router();

    // Your test cases here...
}