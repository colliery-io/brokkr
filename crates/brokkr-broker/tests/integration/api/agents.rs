use axum::{
    body::{to_bytes, Body},
    http::{Request, StatusCode},
};
use chrono::Utc;
use serde_json::json;
use std::usize;
use tower::ServiceExt;
use uuid::Uuid;

use brokkr_models::models::{
    agent_annotations::AgentAnnotation,
    agent_events::AgentEvent,
    agent_labels::AgentLabel,
    agent_targets::AgentTarget,
    agents::{Agent, NewAgent},
};

use crate::fixtures::TestFixture;

#[tokio::test]
async fn test_list_agents() {
    let fixture = TestFixture::new();
    let app = fixture.create_test_router();

    // Create multiple test agents
    let agent1 = fixture.create_test_agent("Test Agent 1".to_string(), "Cluster 1".to_string());
    let agent2 = fixture.create_test_agent("Test Agent 2".to_string(), "Cluster 2".to_string());
    let agent3 = fixture.create_test_agent("Test Agent 3".to_string(), "Cluster 1".to_string());

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/v1/agents")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let agents: Vec<Agent> = serde_json::from_slice(&body).unwrap();

    assert!(
        agents.len() >= 3,
        "Expected at least 3 agents, got {}",
        agents.len()
    );

    // Helper function to find an agent by ID
    let find_agent = |id: Uuid| agents.iter().find(|a| a.id == id);

    // Verify agent1
    let found_agent1 = find_agent(agent1.id).expect("Agent 1 not found in the response");
    assert_eq!(found_agent1.name, "Test Agent 1");
    assert_eq!(found_agent1.cluster_name, "Cluster 1");

    // Verify agent2
    let found_agent2 = find_agent(agent2.id).expect("Agent 2 not found in the response");
    assert_eq!(found_agent2.name, "Test Agent 2");
    assert_eq!(found_agent2.cluster_name, "Cluster 2");

    // Verify agent3
    let found_agent3 = find_agent(agent3.id).expect("Agent 3 not found in the response");
    assert_eq!(found_agent3.name, "Test Agent 3");
    assert_eq!(found_agent3.cluster_name, "Cluster 1");
}

#[tokio::test]
async fn test_create_agent() {
    let fixture = TestFixture::new();
    let app = fixture.create_test_router();

    let new_agent = NewAgent {
        name: "New Test Agent".to_string(),
        cluster_name: "Test Cluster".to_string(),
    };

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/agents")
                .header("Content-Type", "application/json")
                .body(Body::from(serde_json::to_string(&new_agent).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let created_agent: Agent = serde_json::from_slice(&body).unwrap();

    assert_eq!(created_agent.name, new_agent.name);
    assert_eq!(created_agent.cluster_name, new_agent.cluster_name);
}

#[tokio::test]
async fn test_get_agent() {
    let fixture = TestFixture::new();
    let app = fixture.create_test_router();

    let test_agent =
        fixture.create_test_agent("Get Test Agent".to_string(), "Get Test Cluster".to_string());

    let response = app
        .oneshot(
            Request::builder()
                .uri(format!("/api/v1/agents/{}", test_agent.id))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let agent: Agent = serde_json::from_slice(&body).unwrap();

    assert_eq!(agent.id, test_agent.id);
    assert_eq!(agent.name, test_agent.name);
    assert_eq!(agent.cluster_name, test_agent.cluster_name);
}

#[tokio::test]
async fn test_update_agent() {
    let fixture = TestFixture::new();
    let app = fixture.create_test_router();

    let test_agent = fixture.create_test_agent(
        "Update Test Agent".to_string(),
        "Update Test Cluster".to_string(),
    );

    let update_data = json!({
        "name": "Updated Agent Name",
        "cluster_name": "Updated Cluster Name",
        "status": "ACTIVE",
    });

    let response = app
        .oneshot(
            Request::builder()
                .method("PUT")
                .uri(format!("/api/v1/agents/{}", test_agent.id))
                .header("Content-Type", "application/json")
                .body(Body::from(serde_json::to_string(&update_data).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let updated_agent: Agent = serde_json::from_slice(&body).unwrap();

    assert_eq!(updated_agent.id, test_agent.id);
    assert_eq!(updated_agent.name, "Updated Agent Name");
    assert_eq!(updated_agent.cluster_name, "Updated Cluster Name");
    assert_eq!(updated_agent.status, "ACTIVE");
}

#[tokio::test]
async fn test_delete_agent() {
    let fixture = TestFixture::new();
    let app = fixture.create_test_router();

    let test_agent = fixture.create_test_agent(
        "Delete Test Agent".to_string(),
        "Delete Test Cluster".to_string(),
    );

    let response = app
        .oneshot(
            Request::builder()
                .method("DELETE")
                .uri(format!("/api/v1/agents/{}", test_agent.id))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    // Verify the agent is soft deleted
    let deleted_agent = fixture
        .dal
        .agents()
        .get_including_deleted(test_agent.id)
        .unwrap()
        .unwrap();
    assert!(deleted_agent.deleted_at.is_some());
}

#[tokio::test]
async fn test_list_agent_events() {
    let fixture = TestFixture::new();
    let app = fixture.create_test_router();

    let test_agent = fixture.create_test_agent(
        "Event Test Agent".to_string(),
        "Event Test Cluster".to_string(),
    );
    let test_stack = fixture.create_test_stack("Event Test Stack".to_string(), None);
    let test_deployment_object =
        fixture.create_test_deployment_object(test_stack.id, "test: yaml".to_string(), false);

    // Create test events
    let event1 = fixture.create_test_agent_event(
        &test_agent,
        &test_deployment_object,
        "TEST_EVENT",
        "SUCCESS",
        Some("Test message 1"),
    );
    let event2 = fixture.create_test_agent_event(
        &test_agent,
        &test_deployment_object,
        "TEST_EVENT",
        "FAILURE",
        Some("Test message 2"),
    );

    let response = app
        .oneshot(
            Request::builder()
                .uri(format!("/api/v1/agents/{}/events", test_agent.id))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let events: Vec<AgentEvent> = serde_json::from_slice(&body).unwrap();

    assert_eq!(events.len(), 2);
    assert!(events.iter().any(|e| e.id == event1.id));
    assert!(events.iter().any(|e| e.id == event2.id));
}

#[tokio::test]
async fn test_create_agent_event() {
    let fixture = TestFixture::new();
    let app = fixture.create_test_router();

    let test_agent = fixture.create_test_agent(
        "Event Create Test Agent".to_string(),
        "Event Create Test Cluster".to_string(),
    );
    let test_stack = fixture.create_test_stack("Event Create Test Stack".to_string(), None);
    let test_deployment_object =
        fixture.create_test_deployment_object(test_stack.id, "test: yaml".to_string(), false);

    let new_event_data = json!({
        "event_type": "TEST_CREATE_EVENT",
        "status": "SUCCESS",
        "message": "Test create event message",
        "deployment_object_id": test_deployment_object.id
    });

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!("/api/v1/agents/{}/events", test_agent.id))
                .header("Content-Type", "application/json")
                .body(Body::from(serde_json::to_string(&new_event_data).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let created_event: AgentEvent = serde_json::from_slice(&body).unwrap();

    assert_eq!(created_event.agent_id, test_agent.id);
    assert_eq!(
        created_event.deployment_object_id,
        test_deployment_object.id
    );
    assert_eq!(created_event.event_type, "TEST_CREATE_EVENT");
    assert_eq!(created_event.status, "SUCCESS");
    assert_eq!(
        created_event.message,
        Some("Test create event message".to_string())
    );
}

#[tokio::test]
async fn test_list_agent_labels() {
    let fixture = TestFixture::new();
    let app = fixture.create_test_router();

    let test_agent = fixture.create_test_agent(
        "Label Test Agent".to_string(),
        "Label Test Cluster".to_string(),
    );

    // Create test labels
    let label1 = fixture.create_test_agent_label(test_agent.id, "test-label-1".to_string());
    let label2 = fixture.create_test_agent_label(test_agent.id, "test-label-2".to_string());

    let response = app
        .oneshot(
            Request::builder()
                .uri(format!("/api/v1/agents/{}/labels", test_agent.id))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let labels: Vec<AgentLabel> = serde_json::from_slice(&body).unwrap();

    assert_eq!(labels.len(), 2);
    assert!(labels
        .iter()
        .any(|l| l.id == label1.id && l.label == "test-label-1"));
    assert!(labels
        .iter()
        .any(|l| l.id == label2.id && l.label == "test-label-2"));
}

#[tokio::test]
async fn test_add_agent_label() {
    let fixture = TestFixture::new();
    let app = fixture.create_test_router();

    let test_agent = fixture.create_test_agent(
        "Label Add Test Agent".to_string(),
        "Label Add Test Cluster".to_string(),
    );

    let new_label = "test-add-label";

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!("/api/v1/agents/{}/labels", test_agent.id))
                .header("Content-Type", "application/json")
                .body(Body::from(serde_json::to_string(&new_label).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let created_label: AgentLabel = serde_json::from_slice(&body).unwrap();

    assert_eq!(created_label.agent_id, test_agent.id);
    assert_eq!(created_label.label, new_label);
}

#[tokio::test]
async fn test_remove_agent_label() {
    let fixture = TestFixture::new();
    let app = fixture.create_test_router();

    let test_agent = fixture.create_test_agent(
        "Label Remove Test Agent".to_string(),
        "Label Remove Test Cluster".to_string(),
    );
    let label = fixture.create_test_agent_label(test_agent.id, "test-remove-label".to_string());

    let response = app
        .oneshot(
            Request::builder()
                .method("DELETE")
                .uri(format!(
                    "/api/v1/agents/{}/labels/{}",
                    test_agent.id, label.label
                ))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    // Verify the label is removed
    let labels = fixture
        .dal
        .agent_labels()
        .list_for_agent(test_agent.id)
        .unwrap();
    assert!(!labels.iter().any(|l| l.id == label.id));
}

#[tokio::test]
async fn test_add_agent_annotation() {
    let fixture = TestFixture::new();
    let app = fixture.create_test_router();

    let test_agent = fixture.create_test_agent(
        "Annotation Add Test Agent".to_string(),
        "Annotation Add Test Cluster".to_string(),
    );

    let new_annotation = ("test-key", "test-value");

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!("/api/v1/agents/{}/annotations", test_agent.id))
                .header("Content-Type", "application/json")
                .body(Body::from(serde_json::to_string(&new_annotation).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let created_annotation: AgentAnnotation = serde_json::from_slice(&body).unwrap();

    assert_eq!(created_annotation.agent_id, test_agent.id);
    assert_eq!(created_annotation.key, new_annotation.0);
    assert_eq!(created_annotation.value, new_annotation.1);
}

#[tokio::test]
async fn test_remove_agent_annotation() {
    let fixture = TestFixture::new();
    let app = fixture.create_test_router();

    let test_agent = fixture.create_test_agent(
        "Annotation Remove Test Agent".to_string(),
        "Annotation Remove Test Cluster".to_string(),
    );
    let annotation = fixture.create_test_agent_annotation(
        test_agent.id,
        "test-key".to_string(),
        "test-value".to_string(),
    );

    let response = app
        .oneshot(
            Request::builder()
                .method("DELETE")
                .uri(format!(
                    "/api/v1/agents/{}/annotations/{}",
                    test_agent.id, annotation.key
                ))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    // Verify the annotation is removed
    let annotations = fixture
        .dal
        .agent_annotations()
        .list_for_agent(test_agent.id)
        .unwrap();
    assert!(!annotations.iter().any(|a| a.id == annotation.id));
}

#[tokio::test]
async fn test_list_agent_targets() {
    let fixture = TestFixture::new();
    let app = fixture.create_test_router();

    let test_agent = fixture.create_test_agent(
        "Target Test Agent".to_string(),
        "Target Test Cluster".to_string(),
    );
    let test_stack1 = fixture.create_test_stack("Target Test Stack 1".to_string(), None);
    let test_stack2 = fixture.create_test_stack("Target Test Stack 2".to_string(), None);

    // Create test targets
    let target1 = fixture.create_test_agent_target(test_agent.id, test_stack1.id);
    let target2 = fixture.create_test_agent_target(test_agent.id, test_stack2.id);

    let response = app
        .oneshot(
            Request::builder()
                .uri(format!("/api/v1/agents/{}/targets", test_agent.id))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let targets: Vec<AgentTarget> = serde_json::from_slice(&body).unwrap();

    assert_eq!(targets.len(), 2);
    assert!(targets
        .iter()
        .any(|t| t.id == target1.id && t.stack_id == test_stack1.id));
    assert!(targets
        .iter()
        .any(|t| t.id == target2.id && t.stack_id == test_stack2.id));
}

#[tokio::test]
async fn test_add_agent_target() {
    let fixture = TestFixture::new();
    let app = fixture.create_test_router();

    let test_agent = fixture.create_test_agent(
        "Target Add Test Agent".to_string(),
        "Target Add Test Cluster".to_string(),
    );
    let test_stack = fixture.create_test_stack("Target Add Test Stack".to_string(), None);

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!("/api/v1/agents/{}/targets", test_agent.id))
                .header("Content-Type", "application/json")
                .body(Body::from(serde_json::to_string(&test_stack.id).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let created_target: AgentTarget = serde_json::from_slice(&body).unwrap();

    assert_eq!(created_target.agent_id, test_agent.id);
    assert_eq!(created_target.stack_id, test_stack.id);
}

#[tokio::test]
async fn test_remove_agent_target() {
    let fixture = TestFixture::new();
    let app = fixture.create_test_router();

    let test_agent = fixture.create_test_agent(
        "Target Remove Test Agent".to_string(),
        "Target Remove Test Cluster".to_string(),
    );
    let test_stack = fixture.create_test_stack("Target Remove Test Stack".to_string(), None);
    let target = fixture.create_test_agent_target(test_agent.id, test_stack.id);

    let response = app
        .oneshot(
            Request::builder()
                .method("DELETE")
                .uri(format!(
                    "/api/v1/agents/{}/targets/{}",
                    test_agent.id, test_stack.id
                ))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    // Verify the target is removed
    let targets = fixture
        .dal
        .agent_targets()
        .list_for_agent(test_agent.id)
        .unwrap();
    assert!(!targets.iter().any(|t| t.id == target.id));
}

#[tokio::test]
async fn test_record_agent_heartbeat() {
    let fixture = TestFixture::new();
    let app = fixture.create_test_router();

    let test_agent = fixture.create_test_agent(
        "Heartbeat Test Agent".to_string(),
        "Heartbeat Test Cluster".to_string(),
    );

    // Ensure some time passes before we record the heartbeat
    tokio::time::sleep(std::time::Duration::from_millis(10)).await;

    let before_heartbeat = Utc::now();

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!("/api/v1/agents/{}/heartbeat", test_agent.id))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let after_heartbeat = Utc::now();

    // Verify the heartbeat is recorded
    let updated_agent = fixture.dal.agents().get(test_agent.id).unwrap().unwrap();

    assert!(
        updated_agent.last_heartbeat.is_some(),
        "Heartbeat should be recorded"
    );

    let recorded_heartbeat = updated_agent.last_heartbeat.unwrap();

    assert!(
        recorded_heartbeat >= before_heartbeat,
        "Recorded heartbeat should be after or equal to the time before the request"
    );
    assert!(
        recorded_heartbeat <= after_heartbeat,
        "Recorded heartbeat should be before or equal to the time after the request"
    );

    if let Some(original_heartbeat) = test_agent.last_heartbeat {
        assert!(
            recorded_heartbeat > original_heartbeat,
            "Recorded heartbeat should be later than the original heartbeat"
        );
    }
}
