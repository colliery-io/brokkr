use crate::fixtures::TestFixture;
use brokkr_broker::dal::FilterType;
use brokkr_models::models::agent_annotations::NewAgentAnnotation;
use brokkr_models::models::agent_labels::NewAgentLabel;
use brokkr_models::models::agent_targets::NewAgentTarget;
use brokkr_models::models::agents::NewAgent;
use brokkr_models::models::stacks::NewStack;
use uuid::Uuid;

#[test]
fn test_create_agent() {
    let fixture = TestFixture::new();

    let new_agent = NewAgent::new("Test Agent".to_string(), "Test Cluster".to_string())
        .expect("Failed to create NewAgent");

    let created_agent = fixture
        .dal
        .agents()
        .create(&new_agent)
        .expect("Failed to create agent");

    assert_eq!(created_agent.name, new_agent.name);
    assert_eq!(created_agent.cluster_name, new_agent.cluster_name);
    assert!(created_agent.last_heartbeat.is_none());
    assert_eq!(created_agent.status, "INACTIVE");
}

#[test]
fn test_get_agent() {
    let fixture = TestFixture::new();
    let created_agent =
        fixture.create_test_agent("Test Agent".to_string(), "Test Cluster".to_string());

    let retrieved_agent = fixture
        .dal
        .agents()
        .get(created_agent.id)
        .expect("Failed to get agent")
        .unwrap();
    assert_eq!(retrieved_agent.id, created_agent.id);
    assert_eq!(retrieved_agent.name, created_agent.name);
}

#[test]
fn test_get_deleted_agent() {
    let fixture = TestFixture::new();
    let created_agent =
        fixture.create_test_agent("Test Agent".to_string(), "Test Cluster".to_string());

    fixture
        .dal
        .agents()
        .soft_delete(created_agent.id)
        .expect("Failed to soft delete agent");

    let retrieved_agent = fixture
        .dal
        .agents()
        .get(created_agent.id)
        .expect("Failed to get agent");
    assert!(retrieved_agent.is_none());

    let retrieved_deleted_agent = fixture
        .dal
        .agents()
        .get_including_deleted(created_agent.id)
        .expect("Failed to get deleted agent")
        .unwrap();
    assert_eq!(retrieved_deleted_agent.id, created_agent.id);
    assert!(retrieved_deleted_agent.deleted_at.is_some());
}

#[test]
fn test_list_agents() {
    let fixture = TestFixture::new();
    fixture.create_test_agent("Agent 1".to_string(), "Cluster 1".to_string());
    let deleted_agent = fixture.create_test_agent("Agent 2".to_string(), "Cluster 2".to_string());
    fixture
        .dal
        .agents()
        .soft_delete(deleted_agent.id)
        .expect("Failed to soft delete agent");

    let active_agents = fixture.dal.agents().list().expect("Failed to list agents");
    assert_eq!(active_agents.len(), 1);
    assert_eq!(active_agents[0].name, "Agent 1");

    let all_agents = fixture
        .dal
        .agents()
        .list_all()
        .expect("Failed to list all agents");
    assert_eq!(all_agents.len(), 2);
}

#[test]
fn test_update_agent() {
    let fixture = TestFixture::new();
    let created_agent =
        fixture.create_test_agent("Original Agent".to_string(), "Original Cluster".to_string());

    let mut updated_agent = created_agent.clone();
    updated_agent.name = "Updated Agent".to_string();
    updated_agent.cluster_name = "Updated Cluster".to_string();

    let result = fixture
        .dal
        .agents()
        .update(created_agent.id, &updated_agent)
        .expect("Failed to update agent");

    assert_eq!(result.name, "Updated Agent");
    assert_eq!(result.cluster_name, "Updated Cluster");
}

#[test]
fn test_soft_delete_agent() {
    let fixture = TestFixture::new();
    let created_agent =
        fixture.create_test_agent("To Be Deleted".to_string(), "Test Cluster".to_string());

    let affected_rows = fixture
        .dal
        .agents()
        .soft_delete(created_agent.id)
        .expect("Failed to soft delete agent");
    assert_eq!(affected_rows, 1);

    let deleted_agent = fixture
        .dal
        .agents()
        .get_including_deleted(created_agent.id)
        .expect("Failed to get deleted agent")
        .unwrap();
    assert!(deleted_agent.deleted_at.is_some());
}

#[test]
fn test_hard_delete_agent() {
    let fixture = TestFixture::new();
    let created_agent =
        fixture.create_test_agent("To Be Hard Deleted".to_string(), "Test Cluster".to_string());

    let affected_rows = fixture
        .dal
        .agents()
        .hard_delete(created_agent.id)
        .expect("Failed to hard delete agent");
    assert_eq!(affected_rows, 1);

    let hard_deleted_agent = fixture
        .dal
        .agents()
        .get_including_deleted(created_agent.id)
        .expect("Failed to attempt retrieval of hard-deleted agent");
    assert!(hard_deleted_agent.is_none());
}

#[test]
fn test_filter_by_labels_single_label() {
    let fixture = TestFixture::new();

    let agent1 = fixture.create_test_agent("Agent 1".to_string(), "Cluster 1".to_string());
    let agent2 = fixture.create_test_agent("Agent 2".to_string(), "Cluster 2".to_string());

    fixture
        .dal
        .agent_labels()
        .create(&NewAgentLabel::new(agent1.id, "label1".to_string()).unwrap())
        .unwrap();
    fixture
        .dal
        .agent_labels()
        .create(&NewAgentLabel::new(agent2.id, "label2".to_string()).unwrap())
        .unwrap();

    let filtered_agents = fixture
        .dal
        .agents()
        .filter_by_labels(vec!["label1".to_string()], FilterType::Or)
        .unwrap();
    assert_eq!(filtered_agents.len(), 1);
    assert_eq!(filtered_agents[0].id, agent1.id);
}

#[test]
fn test_filter_by_labels_multiple_labels_or() {
    let fixture = TestFixture::new();

    let agent1 = fixture.create_test_agent("Agent 1".to_string(), "Cluster 1".to_string());
    let agent2 = fixture.create_test_agent("Agent 2".to_string(), "Cluster 2".to_string());
    let agent3 = fixture.create_test_agent("Agent 3".to_string(), "Cluster 3".to_string());

    fixture
        .dal
        .agent_labels()
        .create(&NewAgentLabel::new(agent1.id, "label1".to_string()).unwrap())
        .unwrap();
    fixture
        .dal
        .agent_labels()
        .create(&NewAgentLabel::new(agent2.id, "label2".to_string()).unwrap())
        .unwrap();
    fixture
        .dal
        .agent_labels()
        .create(&NewAgentLabel::new(agent3.id, "label3".to_string()).unwrap())
        .unwrap();

    let filtered_agents = fixture
        .dal
        .agents()
        .filter_by_labels(
            vec!["label1".to_string(), "label2".to_string()],
            FilterType::Or,
        )
        .unwrap();
    assert_eq!(filtered_agents.len(), 2);
    assert!(filtered_agents.iter().any(|a| a.id == agent1.id));
    assert!(filtered_agents.iter().any(|a| a.id == agent2.id));
}

#[test]
fn test_filter_by_labels_multiple_labels_and() {
    let fixture = TestFixture::new();

    let agent1 = fixture.create_test_agent("Agent 1".to_string(), "Cluster 1".to_string());
    let agent2 = fixture.create_test_agent("Agent 2".to_string(), "Cluster 2".to_string());

    fixture
        .dal
        .agent_labels()
        .create(&NewAgentLabel::new(agent1.id, "label1".to_string()).unwrap())
        .unwrap();
    fixture
        .dal
        .agent_labels()
        .create(&NewAgentLabel::new(agent1.id, "label2".to_string()).unwrap())
        .unwrap();
    fixture
        .dal
        .agent_labels()
        .create(&NewAgentLabel::new(agent2.id, "label2".to_string()).unwrap())
        .unwrap();

    let filtered_agents = fixture
        .dal
        .agents()
        .filter_by_labels(
            vec!["label1".to_string(), "label2".to_string()],
            FilterType::And,
        )
        .unwrap();
    assert_eq!(filtered_agents.len(), 1);
    assert_eq!(filtered_agents[0].id, agent1.id);
}

#[test]
fn test_filter_by_labels_no_match() {
    let fixture = TestFixture::new();

    let agent1 = fixture.create_test_agent("Agent 1".to_string(), "Cluster 1".to_string());

    fixture
        .dal
        .agent_labels()
        .create(&NewAgentLabel::new(agent1.id, "label1".to_string()).unwrap())
        .unwrap();

    let filtered_agents = fixture
        .dal
        .agents()
        .filter_by_labels(vec!["non_existent_label".to_string()], FilterType::Or)
        .unwrap();
    assert_eq!(filtered_agents.len(), 0);
}

#[test]
fn test_filter_by_annotations() {
    let fixture = TestFixture::new();

    let agent1 = fixture.create_test_agent("Agent 1".to_string(), "Cluster 1".to_string());
    let agent2 = fixture.create_test_agent("Agent 2".to_string(), "Cluster 2".to_string());
    let agent3 = fixture.create_test_agent("Agent 3".to_string(), "Cluster 3".to_string());

    fixture
        .dal
        .agent_annotations()
        .create(
            &NewAgentAnnotation::new(agent1.id, "key1".to_string(), "value1".to_string()).unwrap(),
        )
        .unwrap();
    fixture
        .dal
        .agent_annotations()
        .create(
            &NewAgentAnnotation::new(agent1.id, "key2".to_string(), "value2".to_string()).unwrap(),
        )
        .unwrap();
    fixture
        .dal
        .agent_annotations()
        .create(
            &NewAgentAnnotation::new(agent2.id, "key2".to_string(), "value2".to_string()).unwrap(),
        )
        .unwrap();
    fixture
        .dal
        .agent_annotations()
        .create(
            &NewAgentAnnotation::new(agent3.id, "key3".to_string(), "value3".to_string()).unwrap(),
        )
        .unwrap();

    // Test OR logic
    // Test 1: Filter by a single annotation (OR)
    let filtered_agents = fixture
        .dal
        .agents()
        .filter_by_annotations(
            vec![("key1".to_string(), "value1".to_string())],
            FilterType::Or,
        )
        .unwrap();
    assert_eq!(filtered_agents.len(), 1);
    assert!(filtered_agents.iter().any(|a| a.id == agent1.id));

    // Test 2: Filter by multiple annotations (OR condition)
    let filtered_agents = fixture
        .dal
        .agents()
        .filter_by_annotations(
            vec![
                ("key1".to_string(), "value1".to_string()),
                ("key3".to_string(), "value3".to_string()),
            ],
            FilterType::Or,
        )
        .unwrap();
    assert_eq!(filtered_agents.len(), 2);
    assert!(filtered_agents.iter().any(|a| a.id == agent1.id));
    assert!(filtered_agents.iter().any(|a| a.id == agent3.id));

    // Test 3: Filter by an annotation present on multiple agents (OR)
    let filtered_agents = fixture
        .dal
        .agents()
        .filter_by_annotations(
            vec![("key2".to_string(), "value2".to_string())],
            FilterType::Or,
        )
        .unwrap();
    assert_eq!(filtered_agents.len(), 2);
    assert!(filtered_agents.iter().any(|a| a.id == agent1.id));
    assert!(filtered_agents.iter().any(|a| a.id == agent2.id));

    // Test AND logic
    // Test 4: Filter by multiple annotations (AND condition)
    let filtered_agents = fixture
        .dal
        .agents()
        .filter_by_annotations(
            vec![
                ("key1".to_string(), "value1".to_string()),
                ("key2".to_string(), "value2".to_string()),
            ],
            FilterType::And,
        )
        .unwrap();
    assert_eq!(filtered_agents.len(), 1);
    assert!(filtered_agents.iter().any(|a| a.id == agent1.id));

    // Test 5: Filter by non-matching annotations (AND condition)
    let filtered_agents = fixture
        .dal
        .agents()
        .filter_by_annotations(
            vec![
                ("key1".to_string(), "value1".to_string()),
                ("key3".to_string(), "value3".to_string()),
            ],
            FilterType::And,
        )
        .unwrap();
    assert_eq!(filtered_agents.len(), 0);

    // Test 6: Filter by non-existent annotation (OR)
    let filtered_agents = fixture
        .dal
        .agents()
        .filter_by_annotations(
            vec![(
                "non_existent_key".to_string(),
                "non_existent_value".to_string(),
            )],
            FilterType::Or,
        )
        .unwrap();
    assert_eq!(filtered_agents.len(), 0);

    // Test 7: Filter by non-existent annotation (AND)
    let filtered_agents = fixture
        .dal
        .agents()
        .filter_by_annotations(
            vec![(
                "non_existent_key".to_string(),
                "non_existent_value".to_string(),
            )],
            FilterType::And,
        )
        .unwrap();
    assert_eq!(filtered_agents.len(), 0);

    // Test 8: Filter by empty annotations list (OR)
    let filtered_agents = fixture
        .dal
        .agents()
        .filter_by_annotations(vec![], FilterType::Or)
        .unwrap();
    assert_eq!(filtered_agents.len(), 0);

    // Test 9: Filter by empty annotations list (AND)
    let filtered_agents = fixture
        .dal
        .agents()
        .filter_by_annotations(vec![], FilterType::And)
        .unwrap();
    assert_eq!(filtered_agents.len(), 0);
}

#[test]
fn test_get_agent_by_target_id() {
    let fixture = TestFixture::new();
    let generator = fixture.create_test_generator(
        "Test Generator".to_string(),
        None,
        "test_api_key_hash".to_string(),
    );

    let agent1 = fixture.create_test_agent("Agent 1".to_string(), "Cluster 1".to_string());
    let agent2 = fixture.create_test_agent("Agent 2".to_string(), "Cluster 2".to_string());

    let stack1 = fixture
        .dal
        .stacks()
        .create(&NewStack::new("Stack 1".to_string(), None, generator.id).unwrap())
        .unwrap();
    let stack2 = fixture
        .dal
        .stacks()
        .create(&NewStack::new("Stack 2".to_string(), None, generator.id).unwrap())
        .unwrap();

    fixture
        .dal
        .agent_targets()
        .create(&NewAgentTarget::new(agent1.id, stack1.id).unwrap())
        .unwrap();
    fixture
        .dal
        .agent_targets()
        .create(&NewAgentTarget::new(agent2.id, stack2.id).unwrap())
        .unwrap();

    // Test getting an agent by existing target
    let found_agent = fixture
        .dal
        .agents()
        .get_agent_by_target_id(stack1.id)
        .unwrap();
    assert!(found_agent.is_some());
    assert_eq!(found_agent.unwrap().id, agent1.id);

    // Test getting an agent by non-existing target
    let non_existent_uuid = Uuid::new_v4();
    let not_found_agent = fixture
        .dal
        .agents()
        .get_agent_by_target_id(non_existent_uuid)
        .unwrap();
    assert!(not_found_agent.is_none());
}

#[test]
fn test_get_agent_details() {
    let fixture = TestFixture::new();

    let generator = fixture.create_test_generator(
        "Test Generator".to_string(),
        None,
        "test_api_key_hash".to_string(),
    );
    let agent = fixture.create_test_agent("Test Agent".to_string(), "Test Cluster".to_string());

    // Create labels
    fixture
        .dal
        .agent_labels()
        .create(&NewAgentLabel::new(agent.id, "label1".to_string()).unwrap())
        .unwrap();
    fixture
        .dal
        .agent_labels()
        .create(&NewAgentLabel::new(agent.id, "label2".to_string()).unwrap())
        .unwrap();

    // Create annotations
    fixture
        .dal
        .agent_annotations()
        .create(
            &NewAgentAnnotation::new(agent.id, "key1".to_string(), "value1".to_string()).unwrap(),
        )
        .unwrap();
    fixture
        .dal
        .agent_annotations()
        .create(
            &NewAgentAnnotation::new(agent.id, "key2".to_string(), "value2".to_string()).unwrap(),
        )
        .unwrap();

    // Create targets
    let stack = fixture
        .dal
        .stacks()
        .create(&NewStack::new("Test Stack".to_string(), None, generator.id).unwrap())
        .unwrap();
    fixture
        .dal
        .agent_targets()
        .create(&NewAgentTarget::new(agent.id, stack.id).unwrap())
        .unwrap();

    // Get agent details
    let (labels, targets, annotations) = fixture.dal.agents().get_agent_details(agent.id).unwrap();

    // Assert labels
    assert_eq!(labels.len(), 2);
    assert!(labels.iter().any(|l| l.label == "label1"));
    assert!(labels.iter().any(|l| l.label == "label2"));

    // Assert targets
    assert_eq!(targets.len(), 1);
    assert_eq!(targets[0].stack_id, stack.id);

    // Assert annotations
    assert_eq!(annotations.len(), 2);
    assert!(annotations
        .iter()
        .any(|a| a.key == "key1" && a.value == "value1"));
    assert!(annotations
        .iter()
        .any(|a| a.key == "key2" && a.value == "value2"));

    // Test with non-existent agent
    let non_existent_uuid = Uuid::new_v4();
    let result = fixture.dal.agents().get_agent_details(non_existent_uuid);
    assert!(result.is_ok());
    let (labels, targets, annotations) = result.unwrap();
    assert!(labels.is_empty());
    assert!(targets.is_empty());
    assert!(annotations.is_empty());
}

#[test]
fn test_record_heartbeat() {
    let fixture = TestFixture::new();

    // Create an agent
    let agent = fixture.create_test_agent(
        "Heartbeat Test Agent".to_string(),
        "Test Cluster".to_string(),
    );

    // Record a heartbeat
    fixture
        .dal
        .agents()
        .record_heartbeat(agent.id)
        .expect("Failed to record heartbeat");

    // Retrieve the agent and check the last_heartbeat
    let updated_agent = fixture
        .dal
        .agents()
        .get(agent.id)
        .expect("Failed to get agent")
        .expect("Agent not found");

    assert!(
        updated_agent.last_heartbeat.is_some(),
        "Last heartbeat should be set"
    );

    // Record another heartbeat after a short delay
    std::thread::sleep(std::time::Duration::from_millis(10));
    fixture
        .dal
        .agents()
        .record_heartbeat(agent.id)
        .expect("Failed to record second heartbeat");

    // Retrieve the agent again and check if the last_heartbeat was updated
    let agent_after_second_heartbeat = fixture
        .dal
        .agents()
        .get(agent.id)
        .expect("Failed to get agent")
        .expect("Agent not found");

    assert!(
        agent_after_second_heartbeat.last_heartbeat.unwrap()
            > updated_agent.last_heartbeat.unwrap(),
        "Last heartbeat should be updated after second heartbeat"
    );
}

#[test]
fn test_update_agent_pak_hash() {
    let fixture = TestFixture::new();
    let agent = fixture.create_test_agent("Test Agent".to_string(), "Test Cluster".to_string());

    let new_pak_hash = "new_pak_hash".to_string();
    let updated_agent = fixture
        .dal
        .agents()
        .update_pak_hash(agent.id, new_pak_hash.clone())
        .expect("Failed to update pak_hash");

    assert_eq!(updated_agent.pak_hash, new_pak_hash);

    // Verify the update by fetching the agent again
    let fetched_agent = fixture
        .dal
        .agents()
        .get(agent.id)
        .expect("Failed to get agent")
        .expect("Agent not found");

    assert_eq!(fetched_agent.pak_hash, new_pak_hash);
}

#[test]
fn test_get_agent_by_name_and_cluster_name() {
    let fixture = TestFixture::new();

    // Create a test agent
    let created_agent =
        fixture.create_test_agent("Test Agent".to_string(), "Test Cluster".to_string());

    // Retrieve the agent by name and cluster name
    let retrieved_agent = fixture
        .dal
        .agents()
        .get_by_name_and_cluster_name("Test Agent".to_string(), "Test Cluster".to_string())
        .expect("Failed to get agent by name and cluster name")
        .unwrap();

    // Verify the retrieved agent
    assert_eq!(retrieved_agent.id, created_agent.id);
    assert_eq!(retrieved_agent.name, created_agent.name);
    assert_eq!(retrieved_agent.cluster_name, created_agent.cluster_name);

    // Test with non-existent agent
    let non_existent_agent = fixture
        .dal
        .agents()
        .get_by_name_and_cluster_name("Non Existent Agent".to_string(), "Test Cluster".to_string())
        .expect("Failed to get non-existent agent by name and cluster name");
    assert!(non_existent_agent.is_none());
}
