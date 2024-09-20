use axum::{
    body::{to_bytes, Body},
    http::{Request, StatusCode},
};

use tower::ServiceExt;

use crate::fixtures::TestFixture;

#[tokio::test]
async fn test_get_agents() {
    let fixture = TestFixture::new();
    let app = fixture.create_test_router().with_state(fixture.dal.clone());

    // Create a test agent
    let test_agent = fixture.create_test_agent("Test Agent".to_string(), "Test Cluster".to_string());

    let admin_pak = fixture.admin_pak.clone();


    // Send a request to list agents
    let response = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/api/v1/agents")
                .header("Content-Type", "application/json")
                .header("Authorization", admin_pak)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let agents_response: serde_json::Value = serde_json::from_slice(&body).unwrap();

    // Assert that the response contains a list of agents
    assert!(agents_response.is_array());
    
    // Assert that the test agent is in the list
    let test_agent_in_list = agents_response
        .as_array()
        .unwrap()
        .iter()
        .any(|agent| agent["id"] == test_agent.id.to_string());
    assert!(test_agent_in_list, "Test agent not found in the list");
}