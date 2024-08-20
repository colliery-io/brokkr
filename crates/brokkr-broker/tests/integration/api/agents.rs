// File: tests/integration/api/agents.rs

use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use brokkr_broker::api;
use brokkr_models::models::agents::{Agent, NewAgent};
use hyper::body::to_bytes;
use serde_json;
use tower::ServiceExt;
use uuid::Uuid;
use brokkr_models::schema::agents::dsl::agents;

use crate::fixtures::TestFixture;


#[tokio::test]
async fn test_create_agent() {
    let fixture = TestFixture::new();
    let app = api::create_router(fixture.dal.clone());

    let new_agent = NewAgent::new(
        "Test Agent".to_string(),
        "Test Cluster".to_string(),
        Some(vec!["test".to_string()]),
        Some(vec![("key".to_string(), "value".to_string())]),
    )
    .unwrap();

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/agents")
                .header("Content-Type", "application/json")
                .body(Body::from(serde_json::to_string(&new_agent).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
    let created_agent: Agent = serde_json::from_slice(&body).unwrap();

    assert_eq!(created_agent.name, "Test Agent");
}

#[tokio::test]
async fn test_get_agent() {
    let fixture = TestFixture::new();
    let app = api::create_router(fixture.dal.clone());

    // Create a test agent
    let agent = fixture.create_test_agent();

    // Test 1: Successful retrieval of the agent
    let response = app.clone()
        .oneshot(
            Request::builder()
                .uri(format!("/agents/{}", agent.uuid))
                .body(Body::empty())
                .unwrap()
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    
    let body = to_bytes(response.into_body()).await.unwrap();
    let retrieved_agent: Option<Agent> = serde_json::from_slice(&body).unwrap();    
    assert!(retrieved_agent.is_some());
    let retrieved_agent = retrieved_agent.unwrap();
    assert_eq!(retrieved_agent.uuid, agent.uuid);
    }


#[tokio::test]
async fn test_update_agent() {
    let fixture = TestFixture::new();
    let app = api::create_router(fixture.dal.clone());

    let agent = fixture.create_test_agent();

    let mut updated_agent = agent.clone();
    updated_agent.name = "Updated Agent".to_string();

    let response = app
        .oneshot(
            Request::builder()
                .method("PUT")
                .uri(format!("/agents/{}", agent.uuid))
                .header("Content-Type", "application/json")
                .body(Body::from(serde_json::to_string(&updated_agent).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
    let updated_agent: Option<Agent> = serde_json::from_slice(&body).unwrap();

    assert!(updated_agent.is_some());
    let updated_agent = updated_agent.unwrap();
    assert_eq!(updated_agent.name, "Updated Agent");
}


#[tokio::test]
async fn test_soft_delete(){
    let fixture = TestFixture::new();
    let app = api::create_router(fixture.dal.clone());

    let agent = fixture.create_test_agent();
    let soft_delete_result = fixture.dal.agents().soft_delete(agent.uuid);


    let response = app.clone()
    .oneshot(
        Request::builder()
            .uri(format!("/agents/{}", agent.uuid))
            .body(Body::empty())
            .unwrap()
    )
    .await
    .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = to_bytes(response.into_body()).await.unwrap();
    let retrieved_agent: Option<Agent> = serde_json::from_slice(&body).unwrap();
    assert!(retrieved_agent.is_none(), "Agent should be None after soft delete");

}



#[tokio::test]
async fn test_delete_agent() {
    let fixture = TestFixture::new();
    let app = api::create_router(fixture.dal.clone());

    // Create a test agent
    let agent = fixture.create_test_agent();

    // Clone app for the first request
    let response = app.clone()
        .oneshot(
            Request::builder()
                .method("DELETE")
                .uri(format!("/agents/{}", agent.uuid))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    // Clone app for the second request
    let response = app.clone()
        .oneshot(
            Request::builder()
                .uri(format!("/agents/{}", agent.uuid))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
    let retrieved_agent: Option<Agent> = serde_json::from_slice(&body).unwrap();
    assert!(retrieved_agent.is_none());
}

#[tokio::test]
async fn test_update_agent_heartbeat() {
    let fixture = TestFixture::new();
    let app = api::create_router(fixture.dal.clone());

    let agent = fixture.create_test_agent();

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!("/agents/{}/heartbeat", agent.uuid))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
    let updated_agent: Agent = serde_json::from_slice(&body).unwrap();

    let test = agent.last_heartbeat < updated_agent.last_heartbeat;
    assert!(test)
}

#[tokio::test]
async fn test_update_agent_status() {
    let fixture = TestFixture::new();
    let app = api::create_router(fixture.dal.clone());

    let agent = fixture.create_test_agent();

    let new_status = "ACTIVE".to_string();

    let response = app
        .oneshot(
            Request::builder()
                .method("PUT")
                .uri(format!("/agents/{}/status", agent.uuid))
                .header("Content-Type", "application/json")
                .body(Body::from(serde_json::to_string(&new_status).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
    let updated_agent: Agent = serde_json::from_slice(&body).unwrap();

    assert_eq!(updated_agent.status, "ACTIVE");
}

#[tokio::test]
async fn test_list_agents() {
    let fixture = TestFixture::new();
    let app = api::create_router(fixture.dal.clone());

    // Create a test agent
    fixture.create_test_agent();

    let response = app
        .oneshot(
            Request::builder().uri("/agents").body(Body::empty()).unwrap()
            )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
    
    let agents: Vec<Agent> = serde_json::from_slice(&body).unwrap();
    

    // assert!(!agents.is_empty());
}    
//     // Additional assertions
//     let first_agent = &agents[0];
//     assert!(first_agent.uuid.is_some());
//     assert!(!first_agent.name.is_empty());
//     assert!(!first_agent.cluster_name.is_empty());
// }
