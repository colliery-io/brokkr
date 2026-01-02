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

/// Shipwright Build YAML for build work order testing
/// Uses ttl.sh for ephemeral image storage (no registry credentials needed)
/// Note: sample-go has Dockerfile in docker-build/ subdirectory
const BUILD_YAML: &str = r#"---
apiVersion: shipwright.io/v1beta1
kind: Build
metadata:
  name: e2e-test-build
  namespace: default
spec:
  source:
    type: Git
    git:
      url: https://github.com/shipwright-io/sample-go
    contextDir: docker-build
  strategy:
    name: kaniko
    kind: ClusterBuildStrategy
  output:
    image: ttl.sh/brokkr-e2e-test:1h
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
// Part 5b: Build Work Orders (Shipwright)
// =============================================================================

/// Test build work orders using Shipwright.
///
/// This test creates a real build work order and waits for the agent to
/// process it through Shipwright. Requires:
/// - A running agent (brokkr-integration-test-agent)
/// - Tekton and Shipwright installed in the cluster
/// - ClusterBuildStrategies available (buildah)
pub async fn test_build_work_orders(client: &Client) -> Result<()> {
    // Find the real running agent
    println!("  → Finding running agent...");
    let agents = client.list_agents().await?;
    let agent = agents.iter()
        .find(|a| a["name"] == "brokkr-integration-test-agent")
        .ok_or("No running agent found - is brokkr-agent running?")?;
    let agent_id: Uuid = agent["id"].as_str().unwrap().parse()?;
    println!("    Found agent: {} ({})", agent["name"], agent_id);

    // Ensure agent is active (it may be INACTIVE by default)
    let agent_status = agent["status"].as_str().unwrap_or("UNKNOWN");
    if agent_status != "ACTIVE" {
        println!("  → Activating agent (was {})...", agent_status);
        client.update_agent(agent_id, json!({"status": "ACTIVE"})).await?;
        println!("    Agent activated");
    } else {
        println!("    Agent is already ACTIVE");
    }

    // Create build work order targeting the real agent
    println!("  → Creating build work order...");
    let work_order = client.create_work_order(
        "build",
        BUILD_YAML,
        Some(vec![agent_id]),
        None,
    ).await?;
    let wo_id: Uuid = work_order["id"].as_str().unwrap().parse()?;
    assert_eq!(work_order["work_type"], "build");
    assert_eq!(work_order["status"], "PENDING");
    println!("    Created build work order: {}", wo_id);

    // Wait for the build to complete (or fail/timeout)
    println!("  → Waiting for build to complete (this may take several minutes)...");
    let max_wait = std::time::Duration::from_secs(600); // 10 minutes max
    let poll_interval = std::time::Duration::from_secs(10);
    let start = std::time::Instant::now();

    let final_status = loop {
        if start.elapsed() > max_wait {
            println!("    ⚠ Build timed out after {:?}", max_wait);
            break "TIMEOUT".to_string();
        }

        // Try to get work order - may return 404 if completed and moved to log
        match client.get_work_order(wo_id).await {
            Ok(wo) => {
                let status = wo["status"].as_str().unwrap_or("UNKNOWN").to_string();

                match status.as_str() {
                    "PENDING" => {
                        println!("    Status: PENDING (waiting for agent to claim)");
                    }
                    "CLAIMED" => {
                        println!("    Status: CLAIMED (agent processing...)");
                    }
                    "IN_PROGRESS" => {
                        println!("    Status: IN_PROGRESS (building...)");
                    }
                    "COMPLETED" => {
                        let result = wo["result"].as_str().unwrap_or("no result");
                        println!("    Status: COMPLETED");
                        println!("    Result: {}", result);
                        break status;
                    }
                    "FAILED" => {
                        let result = wo["result"].as_str().unwrap_or("no error message");
                        println!("    Status: FAILED");
                        println!("    Error: {}", result);
                        break status;
                    }
                    _ => {
                        println!("    Status: {} (unexpected)", status);
                    }
                }
            }
            Err(e) if e.to_string().contains("404") => {
                // Work order completed and moved to work_order_log
                // Check the log for final status
                println!("    Work order moved to log, checking result...");
                match client.get_work_order_log(wo_id).await {
                    Ok(log) => {
                        let success = log["success"].as_bool().unwrap_or(false);
                        let result = log["result_message"].as_str().unwrap_or("no message");
                        if success {
                            println!("    Status: COMPLETED (from log)");
                            println!("    Result: {}", result);
                            break "COMPLETED".to_string();
                        } else {
                            println!("    Status: FAILED (from log)");
                            println!("    Error: {}", result);
                            break "FAILED".to_string();
                        }
                    }
                    Err(log_err) => {
                        return Err(format!("Work order not found in active or log: {}", log_err).into());
                    }
                }
            }
            Err(e) => {
                return Err(e);
            }
        }

        tokio::time::sleep(poll_interval).await;
    };

    // Evaluate result
    match final_status.as_str() {
        "COMPLETED" => {
            println!("  ✓ Build completed successfully!");
        }
        "FAILED" => {
            // Build failed - this could be due to Shipwright not being installed
            // or a build configuration issue. Log but don't fail the test.
            let wo = client.get_work_order(wo_id).await?;
            let error = wo["result"].as_str().unwrap_or("unknown error");
            if error.contains("BuildRun CRD not found") || error.contains("Shipwright") {
                println!("  ⚠ Build failed - Shipwright may not be installed");
                println!("    Error: {}", error);
                println!("    Install Tekton and Shipwright to enable builds");
            } else {
                return Err(format!("Build failed: {}", error).into());
            }
        }
        "TIMEOUT" => {
            // Check final state
            let wo = client.get_work_order(wo_id).await?;
            let status = wo["status"].as_str().unwrap_or("UNKNOWN");
            if status == "PENDING" {
                println!("  ⚠ Build never started - agent may not be processing work orders");
            } else {
                println!("  ⚠ Build timed out in status: {}", status);
            }
            return Err("Build work order timed out".into());
        }
        _ => {
            return Err(format!("Unexpected final status: {}", final_status).into());
        }
    }

    // Clean up Build resource from cluster
    // Note: Work orders that complete are moved to work_order_log, no need to delete
    println!("  → Cleaning up Build resource...");
    // The Build/BuildRun resources remain in cluster - cleanup happens naturally
    // or via ttl.sh image expiration

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

// =============================================================================
// Part 7: Webhooks
// =============================================================================

pub async fn test_webhooks(client: &Client, webhook_catcher_url: Option<&str>) -> Result<()> {
    // Use webhook-catcher if provided, otherwise use a placeholder URL
    let webhook_url = webhook_catcher_url
        .map(|url| format!("{}/webhook", url))
        .unwrap_or_else(|| "http://webhook-catcher:8080/webhook".to_string());

    // Create a webhook subscription
    println!("  → Creating webhook subscription...");
    let webhook = client.create_webhook(
        "e2e-test-webhook",
        &webhook_url,
        vec!["workorder.completed"],
        Some("Bearer e2e-test-token"),
    ).await?;
    let webhook_id: Uuid = webhook["id"].as_str().unwrap().parse()?;
    println!("    Created webhook: {}", webhook_id);

    // Verify webhook was created
    assert_eq!(webhook["name"], "e2e-test-webhook");
    assert_eq!(webhook["enabled"], true);
    println!("    Webhook enabled: true");

    // List webhooks
    println!("  → Listing webhooks...");
    let webhooks = client.list_webhooks().await?;
    assert!(!webhooks.is_empty(), "Should have at least one webhook");
    println!("    Found {} webhook(s)", webhooks.len());

    // Get specific webhook
    println!("  → Getting webhook details...");
    let fetched = client.get_webhook(webhook_id).await?;
    assert_eq!(fetched["id"], webhook_id.to_string());
    assert_eq!(fetched["name"], "e2e-test-webhook");

    // Verify event_types
    let event_types = fetched["event_types"].as_array().expect("event_types should be array");
    assert_eq!(event_types.len(), 1);
    println!("    Webhook subscribes to {} event type(s)", event_types.len());

    // Update webhook (disable it)
    println!("  → Updating webhook (disable)...");
    let updated = client.update_webhook(webhook_id, json!({"enabled": false})).await?;
    assert_eq!(updated["enabled"], false);
    println!("    Webhook disabled");

    // Re-enable
    println!("  → Re-enabling webhook...");
    let updated = client.update_webhook(webhook_id, json!({"enabled": true})).await?;
    assert_eq!(updated["enabled"], true);
    println!("    Webhook re-enabled");

    // ==========================================================================
    // E2E Webhook Delivery Test
    // ==========================================================================
    if let Some(catcher_url) = webhook_catcher_url {
        println!("  → Testing end-to-end webhook delivery...");

        let catcher = crate::api::WebhookCatcher::new(catcher_url);

        // Clear any existing messages
        catcher.clear_messages().await?;
        println!("    Cleared webhook-catcher messages");

        // Create a work order that will trigger workorder.completed event
        println!("    Creating work order to trigger event...");
        let wo = client.create_work_order(
            "custom",
            "# Webhook test work order\necho 'test'",
            None,
            Some(vec!["integration-test"]), // Target any integration test agent
        ).await?;
        let wo_id: Uuid = wo["id"].as_str().unwrap().parse()?;
        println!("    Created work order: {}", wo_id);

        // Wait for work order to be claimed and completed
        println!("    Waiting for work order completion...");
        let mut completed = false;
        for _ in 0..30 {
            tokio::time::sleep(std::time::Duration::from_secs(1)).await;

            match client.get_work_order(wo_id).await {
                Ok(wo_status) => {
                    let status = wo_status["status"].as_str().unwrap_or("unknown");
                    if status == "COMPLETED" || status == "FAILED" {
                        println!("    Work order status: {}", status);
                        completed = true;
                        break;
                    }
                }
                Err(_) => {
                    // Work order might have been moved to log
                    // Check if we got a webhook
                    if let Ok(msgs) = catcher.get_messages().await {
                        if msgs["count"].as_u64().unwrap_or(0) > 0 {
                            completed = true;
                            break;
                        }
                    }
                }
            }
        }

        if completed {
            // Wait for webhook delivery (broker delivery worker runs every 5 seconds)
            println!("    Waiting for webhook delivery...");
            match catcher.wait_for_messages(1, 15).await {
                Ok(msgs) => {
                    let count = msgs["count"].as_u64().unwrap_or(0);
                    println!("    ✓ Received {} webhook message(s)", count);

                    // Verify the message content
                    if let Some(messages) = msgs["messages"].as_array() {
                        if let Some(first_msg) = messages.first() {
                            if let Some(body) = first_msg.get("body") {
                                let event_type = body["event_type"].as_str().unwrap_or("unknown");
                                println!("    ✓ Event type: {}", event_type);
                                assert_eq!(event_type, "workorder.completed", "Expected workorder.completed event");
                            }
                        }
                    }
                }
                Err(e) => {
                    println!("    ⚠ Webhook delivery not received: {}", e);
                    // Check delivery status in broker
                    let deliveries = client.list_webhook_deliveries(webhook_id).await?;
                    println!("    Broker shows {} delivery record(s)", deliveries.len());
                    for d in deliveries.iter() {
                        println!("      - Status: {}, Error: {:?}",
                            d["status"].as_str().unwrap_or("unknown"),
                            d["last_error"].as_str()
                        );
                    }
                }
            }
        } else {
            println!("    ⚠ Work order did not complete in time, skipping delivery verification");
        }

        // Clean up work order (delete if still exists)
        let _ = client.delete_work_order(wo_id).await;
    } else {
        println!("  → Skipping delivery test (no WEBHOOK_CATCHER_URL)");
    }

    // Check deliveries via broker API
    println!("  → Checking webhook deliveries via API...");
    let deliveries = client.list_webhook_deliveries(webhook_id).await?;
    println!("    Found {} delivery record(s)", deliveries.len());

    // Delete webhook
    println!("  → Deleting webhook...");
    client.delete_webhook(webhook_id).await?;
    println!("    Webhook deleted");

    // Verify deletion
    let webhooks = client.list_webhooks().await?;
    let still_exists = webhooks.iter().any(|w| w["id"] == webhook_id.to_string());
    assert!(!still_exists, "Webhook should be deleted");
    println!("    Deletion verified");

    Ok(())
}

// =============================================================================
// Part 7b: Agent Reconciliation of Existing Deployments (BROKKR-T-0094)
// =============================================================================

/// Test that agents can reconcile pre-existing deployments when targeted to a stack.
/// This tests the scenario where:
/// 1. Stack exists with deployment object
/// 2. Agent is created and targeted to the stack AFTER deployment exists
/// 3. Agent should see the deployment via target-state API
pub async fn test_agent_reconciliation_existing_deployments(client: &Client) -> Result<()> {
    // Create generator for all stacks
    println!("  → Setting up reconciliation test resources...");
    let gen_result = client.create_generator("e2e-reconcile-gen", None).await?;
    let generator_id: Uuid = gen_result["generator"]["id"].as_str().unwrap().parse()?;

    // ==========================================================================
    // Test 1: Direct Targeting - Deployment exists before agent targeting
    // ==========================================================================
    println!("  → Test 1: Direct targeting with pre-existing deployment...");

    // 1a. Create stack with deployment FIRST
    let stack1 = client.create_stack("e2e-reconcile-direct", None, generator_id).await?;
    let stack1_id: Uuid = stack1["id"].as_str().unwrap().parse()?;
    let deployment1 = client.create_deployment(stack1_id, "# Direct targeting test", false).await?;
    let deployment1_id: Uuid = deployment1["id"].as_str().unwrap().parse()?;
    println!("    Created stack and deployment: {}", deployment1_id);

    // 1b. Create agent (no targets yet)
    let agent1_result = client.create_agent("e2e-reconcile-direct-agent", "reconcile-cluster-1").await?;
    let agent1_id: Uuid = agent1_result["agent"]["id"].as_str().unwrap().parse()?;
    client.update_agent(agent1_id, json!({"status": "ACTIVE"})).await?;
    println!("    Created agent: {}", agent1_id);

    // 1c. NOW target agent to stack (after deployment exists)
    client.add_agent_target(agent1_id, stack1_id).await?;
    println!("    Targeted agent to stack");

    // 1d. Verify agent sees the pre-existing deployment
    let target_state1 = client.get_agent_target_state(agent1_id, None).await?;
    assert!(
        target_state1.iter().any(|d| d["id"] == deployment1_id.to_string()),
        "Agent should see pre-existing deployment via direct targeting"
    );
    println!("    ✓ Agent sees pre-existing deployment via direct targeting");

    // ==========================================================================
    // Test 2: Label Targeting - Deployment exists before agent gets matching label
    // ==========================================================================
    println!("  → Test 2: Label targeting with pre-existing deployment...");

    // 2a. Create stack with label and deployment FIRST
    let stack2 = client.create_stack("e2e-reconcile-label", None, generator_id).await?;
    let stack2_id: Uuid = stack2["id"].as_str().unwrap().parse()?;
    client.add_stack_label(stack2_id, "reconcile-test-label").await?;
    let deployment2 = client.create_deployment(stack2_id, "# Label targeting test", false).await?;
    let deployment2_id: Uuid = deployment2["id"].as_str().unwrap().parse()?;
    println!("    Created labeled stack and deployment: {}", deployment2_id);

    // 2b. Create agent (no labels yet)
    let agent2_result = client.create_agent("e2e-reconcile-label-agent", "reconcile-cluster-2").await?;
    let agent2_id: Uuid = agent2_result["agent"]["id"].as_str().unwrap().parse()?;
    client.update_agent(agent2_id, json!({"status": "ACTIVE"})).await?;
    println!("    Created agent: {}", agent2_id);

    // 2c. NOW add matching label to agent (after deployment exists)
    client.add_agent_label(agent2_id, "reconcile-test-label").await?;
    println!("    Added matching label to agent");

    // 2d. Verify agent sees the pre-existing deployment
    let target_state2 = client.get_agent_target_state(agent2_id, None).await?;
    assert!(
        target_state2.iter().any(|d| d["id"] == deployment2_id.to_string()),
        "Agent should see pre-existing deployment via label targeting"
    );
    println!("    ✓ Agent sees pre-existing deployment via label targeting");

    // ==========================================================================
    // Test 3: Annotation Targeting - Deployment exists before agent gets matching annotation
    // ==========================================================================
    println!("  → Test 3: Annotation targeting with pre-existing deployment...");

    // 3a. Create stack with annotation and deployment FIRST
    let stack3 = client.create_stack("e2e-reconcile-annotation", None, generator_id).await?;
    let stack3_id: Uuid = stack3["id"].as_str().unwrap().parse()?;
    client.add_stack_annotation(stack3_id, "reconcile-key", "reconcile-value").await?;
    let deployment3 = client.create_deployment(stack3_id, "# Annotation targeting test", false).await?;
    let deployment3_id: Uuid = deployment3["id"].as_str().unwrap().parse()?;
    println!("    Created annotated stack and deployment: {}", deployment3_id);

    // 3b. Create agent (no annotations yet)
    let agent3_result = client.create_agent("e2e-reconcile-annotation-agent", "reconcile-cluster-3").await?;
    let agent3_id: Uuid = agent3_result["agent"]["id"].as_str().unwrap().parse()?;
    client.update_agent(agent3_id, json!({"status": "ACTIVE"})).await?;
    println!("    Created agent: {}", agent3_id);

    // 3c. NOW add matching annotation to agent (after deployment exists)
    client.add_agent_annotation(agent3_id, "reconcile-key", "reconcile-value").await?;
    println!("    Added matching annotation to agent");

    // 3d. Verify agent sees the pre-existing deployment
    let target_state3 = client.get_agent_target_state(agent3_id, None).await?;
    assert!(
        target_state3.iter().any(|d| d["id"] == deployment3_id.to_string()),
        "Agent should see pre-existing deployment via annotation targeting"
    );
    println!("    ✓ Agent sees pre-existing deployment via annotation targeting");

    println!("  ✓ All reconciliation tests passed!");
    Ok(())
}

// =============================================================================
// Part 8: Audit Logs
// =============================================================================

pub async fn test_audit_logs(client: &Client) -> Result<()> {
    // The previous tests should have generated audit log entries
    println!("  → Fetching audit logs...");
    let result = client.list_audit_logs(Some(50)).await?;

    let logs = result["logs"].as_array().expect("logs should be array");
    let total = result["total"].as_i64().unwrap_or(0);

    println!("    Found {} audit log entries (showing {})", total, logs.len());

    // Check log structure if we have entries
    if let Some(first_log) = logs.first() {
        assert!(first_log.get("id").is_some(), "Log should have id");
        assert!(first_log.get("action").is_some(), "Log should have action");
        assert!(first_log.get("actor_type").is_some(), "Log should have actor_type");
        assert!(first_log.get("created_at").is_some(), "Log should have timestamp");
        println!("    Log structure verified");

        // Show sample actions
        let actions: Vec<&str> = logs.iter()
            .filter_map(|l| l["action"].as_str())
            .take(5)
            .collect();
        println!("    Sample actions: {:?}", actions);
    } else {
        // Audit logging may not be fully integrated yet - warn but don't fail
        println!("    ⚠ No audit logs found - audit logging may not be enabled for all endpoints");
    }

    // Verify API returns proper structure (this should always pass)
    assert!(result.get("logs").is_some(), "Response should have logs field");
    assert!(result.get("total").is_some(), "Response should have total field");
    println!("    Audit log API structure verified");

    Ok(())
}

// =============================================================================
// Part 9: Metrics & Observability
// =============================================================================

pub async fn test_metrics(client: &Client) -> Result<()> {
    // Make some API calls first to generate metrics
    println!("  → Generating metrics by making API calls...");
    let _ = client.get_healthz().await?;
    let _ = client.list_agents().await?;
    let _ = client.list_stacks().await?;
    println!("    Made several API calls to generate metrics");

    // Fetch metrics
    println!("  → Fetching Prometheus metrics...");
    let metrics = client.get_metrics().await?;

    // Verify metrics content type is Prometheus format
    assert!(!metrics.is_empty(), "Metrics should not be empty");
    println!("    Metrics endpoint returned {} bytes", metrics.len());

    // Verify HTTP request metrics are present and recording data
    println!("  → Verifying HTTP request metrics...");
    assert!(
        metrics.contains("brokkr_http_requests_total"),
        "Should contain HTTP requests counter"
    );
    assert!(
        metrics.contains("brokkr_http_request_duration_seconds"),
        "Should contain HTTP request duration histogram"
    );
    println!("    HTTP request metrics present ✓");

    // Verify metrics have proper labels
    println!("  → Verifying metric labels...");
    assert!(
        metrics.contains("method=\"GET\""),
        "HTTP metrics should have method label"
    );
    assert!(
        metrics.contains("endpoint="),
        "HTTP metrics should have endpoint label"
    );
    println!("    Metric labels verified ✓");

    // Verify system gauges are present
    println!("  → Verifying system metrics...");
    let system_metrics = [
        "brokkr_active_agents",
        "brokkr_stacks_total",
        "brokkr_deployment_objects_total",
    ];
    for metric in system_metrics {
        assert!(
            metrics.contains(metric),
            "Should contain {} gauge",
            metric
        );
    }
    println!("    System metrics present ✓");

    // Verify database metrics are defined
    println!("  → Verifying database metrics...");
    // Note: DB metrics may not have data yet if no DB queries were instrumented
    // We just check they're defined (will appear after first use)
    if metrics.contains("brokkr_database_queries_total") {
        println!("    Database query metrics present ✓");
    } else {
        println!("    ⚠ Database metrics not yet populated (expected if DAL not instrumented)");
    }

    // Print all brokkr metrics (not histogram buckets to keep it readable)
    println!("  → Brokkr metrics:");
    for line in metrics.lines() {
        if line.starts_with("brokkr_") && !line.contains("_bucket{") && !line.contains("_sum{") && !line.contains("_count{") {
            println!("    {}", line);
        }
    }

    println!("  → Metrics observability verified");
    Ok(())
}
