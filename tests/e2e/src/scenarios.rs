/*
 * Copyright (c) 2025 Dylan Storey
 * Licensed under the Elastic License 2.0.
 * See LICENSE file in the project root for full license text.
 */

//! E2E Test Scenarios
//!
//! Each scenario tests a complete user workflow through the system.

use crate::api::{Client, Result};
use serde_json::json;
use uuid::Uuid;

/// Sample deployment YAML for testing
const DEMO_DEPLOYMENT_YAML: &str = r#"apiVersion: v1
kind: Namespace
metadata:
  name: e2e-test-ns
  labels:
    app: e2e-test
    managed-by: brokkr
---
apiVersion: v1
kind: ConfigMap
metadata:
  name: e2e-test-config
  namespace: e2e-test-ns
data:
  APP_ENV: "test"
  LOG_LEVEL: "debug"
---
apiVersion: apps/v1
kind: Deployment
metadata:
  name: e2e-test-web
  namespace: e2e-test-ns
spec:
  replicas: 1
  selector:
    matchLabels:
      app: e2e-test-web
  template:
    metadata:
      labels:
        app: e2e-test-web
    spec:
      containers:
      - name: web
        image: nginx:alpine
        ports:
        - containerPort: 80
"#;

/// Microservice template for testing
const MICROSERVICE_TEMPLATE: &str = r#"apiVersion: apps/v1
kind: Deployment
metadata:
  name: {{ name }}
  namespace: {{ namespace | default(value="default") }}
spec:
  replicas: {{ replicas | default(value=1) }}
  selector:
    matchLabels:
      app: {{ name }}
  template:
    metadata:
      labels:
        app: {{ name }}
    spec:
      containers:
      - name: {{ name }}
        image: {{ image }}
        ports:
        - containerPort: {{ port | default(value=8080) }}
"#;

const MICROSERVICE_SCHEMA: &str = r#"{
  "type": "object",
  "required": ["name", "image"],
  "properties": {
    "name": { "type": "string" },
    "namespace": { "type": "string", "default": "default" },
    "image": { "type": "string" },
    "replicas": { "type": "integer", "default": 1 },
    "port": { "type": "integer", "default": 8080 }
  }
}"#;

/// Job YAML for work order testing
const JOB_YAML: &str = r#"apiVersion: batch/v1
kind: Job
metadata:
  name: e2e-test-job
  namespace: default
spec:
  ttlSecondsAfterFinished: 60
  template:
    spec:
      restartPolicy: Never
      containers:
      - name: test
        image: alpine:latest
        command: ["echo", "E2E test job completed"]
"#;

// =============================================================================
// Part 1: Agent Management
// =============================================================================

pub async fn test_agent_management(client: &Client) -> Result<()> {
    println!("  → Creating a new agent...");
    let result = client.create_agent("e2e-test-agent", "e2e-test-cluster").await?;
    let agent_id: Uuid = result["agent"]["id"].as_str().unwrap().parse()?;
    let _initial_pak = result["initial_pak"].as_str().unwrap();
    println!("    Created agent: {}", agent_id);

    // Verify agent starts INACTIVE
    let agent = client.get_agent(agent_id).await?;
    assert_eq!(agent["status"], "INACTIVE", "New agent should be INACTIVE");
    println!("  → Verified agent starts INACTIVE");

    // Activate the agent
    println!("  → Activating agent...");
    let updated = client.update_agent(agent_id, json!({"status": "ACTIVE"})).await?;
    assert_eq!(updated["status"], "ACTIVE");
    println!("    Agent is now ACTIVE");

    // Add labels
    println!("  → Adding labels...");
    client.add_agent_label(agent_id, "e2e-test").await?;
    client.add_agent_label(agent_id, "development").await?;
    let labels = client.get_agent_labels(agent_id).await?;
    assert_eq!(labels.len(), 2, "Should have 2 labels");
    println!("    Added {} labels", labels.len());

    // Add annotations
    println!("  → Adding annotations...");
    client.add_agent_annotation(agent_id, "owner", "e2e-test-suite").await?;
    client.add_agent_annotation(agent_id, "environment", "test").await?;
    let annotations = client.get_agent_annotations(agent_id).await?;
    assert_eq!(annotations.len(), 2, "Should have 2 annotations");
    println!("    Added {} annotations", annotations.len());

    // Deactivate agent
    println!("  → Deactivating agent...");
    let updated = client.update_agent(agent_id, json!({"status": "INACTIVE"})).await?;
    assert_eq!(updated["status"], "INACTIVE");
    println!("    Agent is now INACTIVE");

    Ok(())
}

// =============================================================================
// Part 2: Stack Creation & Deployment
// =============================================================================

pub async fn test_stack_deployment(client: &Client) -> Result<()> {
    // Create a generator
    println!("  → Creating generator...");
    let gen_result = client.create_generator("e2e-test-generator", Some("E2E test generator")).await?;
    let generator_id: Uuid = gen_result["generator"]["id"].as_str().unwrap().parse()?;
    println!("    Created generator: {}", generator_id);

    // Create a stack
    println!("  → Creating stack...");
    let stack = client.create_stack("e2e-test-stack", Some("E2E test stack"), generator_id).await?;
    let stack_id: Uuid = stack["id"].as_str().unwrap().parse()?;
    println!("    Created stack: {}", stack_id);

    // Deploy multi-resource YAML
    println!("  → Deploying multi-resource YAML...");
    let deployment = client.create_deployment(stack_id, DEMO_DEPLOYMENT_YAML, false).await?;
    let deployment_id: Uuid = deployment["id"].as_str().unwrap().parse()?;
    assert_eq!(deployment["is_deletion_marker"], false);
    println!("    Created deployment object: {}", deployment_id);

    // List deployments
    println!("  → Listing deployments...");
    let deployments = client.list_deployments(stack_id).await?;
    assert!(!deployments.is_empty(), "Stack should have deployments");
    println!("    Stack has {} deployment(s)", deployments.len());

    // Add stack labels
    println!("  → Adding stack labels...");
    client.add_stack_label(stack_id, "e2e-test").await?;
    let labels = client.get_stack_labels(stack_id).await?;
    assert!(!labels.is_empty());
    println!("    Stack has {} label(s)", labels.len());

    Ok(())
}

// =============================================================================
// Part 3: Agent Targeting
// =============================================================================

pub async fn test_targeting(client: &Client) -> Result<()> {
    // Create resources for targeting test
    println!("  → Setting up targeting test resources...");

    let gen_result = client.create_generator("e2e-targeting-gen", None).await?;
    let generator_id: Uuid = gen_result["generator"]["id"].as_str().unwrap().parse()?;

    // Create agent with labels
    let agent_result = client.create_agent("e2e-targeting-agent", "targeting-cluster").await?;
    let agent_id: Uuid = agent_result["agent"]["id"].as_str().unwrap().parse()?;
    client.update_agent(agent_id, json!({"status": "ACTIVE"})).await?;
    client.add_agent_label(agent_id, "targeting-test").await?;
    println!("    Created agent with label 'targeting-test'");

    // Create stack with matching label
    let stack = client.create_stack("e2e-targeting-stack", None, generator_id).await?;
    let stack_id: Uuid = stack["id"].as_str().unwrap().parse()?;
    client.add_stack_label(stack_id, "targeting-test").await?;
    println!("    Created stack with label 'targeting-test'");

    // Verify label matching
    println!("  → Verifying label-based targeting...");
    let agent_stacks = client.get_agent_stacks(agent_id).await?;
    let has_stack = agent_stacks.iter().any(|s| s["id"] == stack_id.to_string());
    assert!(has_stack, "Agent should see stack via label matching");
    println!("    Agent sees stack via label matching ✓");

    // Test explicit targeting
    println!("  → Testing explicit targeting...");
    let stack2 = client.create_stack("e2e-explicit-stack", None, generator_id).await?;
    let stack2_id: Uuid = stack2["id"].as_str().unwrap().parse()?;

    client.add_agent_target(agent_id, stack2_id).await?;
    let targets = client.get_agent_targets(agent_id).await?;
    let has_target = targets.iter().any(|t| t["stack_id"] == stack2_id.to_string());
    assert!(has_target, "Agent should have explicit target");
    println!("    Explicit target created ✓");

    // Verify agent sees both stacks
    let agent_stacks = client.get_agent_stacks(agent_id).await?;
    assert!(agent_stacks.len() >= 2, "Agent should see multiple stacks");
    println!("    Agent sees {} stack(s) total", agent_stacks.len());

    Ok(())
}

// =============================================================================
// Part 4: Templates
// =============================================================================

pub async fn test_templates(client: &Client) -> Result<()> {
    // Create a template
    println!("  → Creating template...");
    let template = client.create_template(
        "e2e-microservice",
        Some("E2E test microservice template"),
        MICROSERVICE_TEMPLATE,
        MICROSERVICE_SCHEMA,
    ).await?;
    let template_id: Uuid = template["id"].as_str().unwrap().parse()?;
    assert_eq!(template["name"], "e2e-microservice");
    println!("    Created template: {}", template_id);

    // Create a stack for instantiation
    let gen_result = client.create_generator("e2e-template-gen", None).await?;
    let generator_id: Uuid = gen_result["generator"]["id"].as_str().unwrap().parse()?;
    let stack = client.create_stack("e2e-template-stack", None, generator_id).await?;
    let stack_id: Uuid = stack["id"].as_str().unwrap().parse()?;

    // Instantiate the template
    println!("  → Instantiating template...");
    let deployment = client.instantiate_template(
        stack_id,
        template_id,
        json!({
            "name": "e2e-service",
            "image": "nginx:alpine",
            "replicas": 2,
            "port": 8080
        }),
    ).await?;

    let yaml_content = deployment["yaml_content"].as_str().unwrap();
    assert!(yaml_content.contains("name: e2e-service"), "Template should render name");
    assert!(yaml_content.contains("replicas: 2"), "Template should render replicas");
    println!("    Template instantiated successfully");

    // List templates
    println!("  → Listing templates...");
    let templates = client.list_templates().await?;
    assert!(!templates.is_empty(), "Should have templates");
    println!("    Found {} template(s)", templates.len());

    // Delete template
    println!("  → Deleting template...");
    client.delete_template(template_id).await?;
    println!("    Template deleted");

    Ok(())
}

// =============================================================================
// Part 5: Work Orders
// =============================================================================

pub async fn test_work_orders(client: &Client) -> Result<()> {
    // Create an agent for work order targeting
    println!("  → Setting up work order test...");
    let agent_result = client.create_agent("e2e-work-order-agent", "work-order-cluster").await?;
    let agent_id: Uuid = agent_result["agent"]["id"].as_str().unwrap().parse()?;
    client.update_agent(agent_id, json!({"status": "ACTIVE"})).await?;
    client.add_agent_label(agent_id, "work-order-test").await?;

    // Create work order with explicit agent targeting
    println!("  → Creating work order with agent targeting...");
    let work_order = client.create_work_order(
        "custom",
        JOB_YAML,
        Some(vec![agent_id]),
        None,
    ).await?;
    let wo_id: Uuid = work_order["id"].as_str().unwrap().parse()?;
    assert_eq!(work_order["status"], "PENDING");
    println!("    Created work order: {}", wo_id);

    // List work orders
    println!("  → Listing work orders...");
    let work_orders = client.list_work_orders().await?;
    assert!(!work_orders.is_empty(), "Should have work orders");
    println!("    Found {} work order(s)", work_orders.len());

    // Get specific work order
    println!("  → Getting work order details...");
    let fetched = client.get_work_order(wo_id).await?;
    assert_eq!(fetched["id"], wo_id.to_string());
    println!("    Work order status: {}", fetched["status"]);

    // Create work order with label targeting
    println!("  → Creating work order with label targeting...");
    let wo2 = client.create_work_order(
        "custom",
        JOB_YAML,
        None,
        Some(vec!["work-order-test"]),
    ).await?;
    assert_eq!(wo2["status"], "PENDING");
    println!("    Created label-targeted work order");

    // Delete work order
    println!("  → Deleting work order...");
    client.delete_work_order(wo_id).await?;
    println!("    Work order deleted");

    Ok(())
}

// =============================================================================
// Part 6: Health & Diagnostics
// =============================================================================

pub async fn test_health_diagnostics(client: &Client) -> Result<()> {
    // Create resources
    println!("  → Setting up health test resources...");
    let gen_result = client.create_generator("e2e-health-gen", None).await?;
    let generator_id: Uuid = gen_result["generator"]["id"].as_str().unwrap().parse()?;

    let stack = client.create_stack("e2e-health-stack", None, generator_id).await?;
    let stack_id: Uuid = stack["id"].as_str().unwrap().parse()?;

    let deployment = client.create_deployment(stack_id, DEMO_DEPLOYMENT_YAML, false).await?;
    let deployment_id: Uuid = deployment["id"].as_str().unwrap().parse()?;
    println!("    Created stack and deployment");

    // Get stack health
    println!("  → Checking stack health...");
    let health = client.get_stack_health(stack_id).await?;
    println!("    Stack health retrieved: {:?}", health.get("summary"));

    // Get deployment health
    println!("  → Checking deployment health...");
    let dep_health = client.get_deployment_health(deployment_id).await?;
    let records = dep_health.get("health_records").and_then(|r| r.as_array());
    println!("    Deployment health: {} agent report(s)", records.map(|r| r.len()).unwrap_or(0));

    // Note: Diagnostics require an agent that has processed the deployment,
    // which may not be available in all test environments. We verify the
    // API is callable but may not have data.
    println!("  → Health monitoring APIs verified");

    Ok(())
}
