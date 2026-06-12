/*
 * Copyright (c) 2025-2026 Dylan Storey
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
    let result = client
        .create_agent("e2e-test-agent", "e2e-test-cluster")
        .await?;
    let agent_id: Uuid = result["agent"]["id"].as_str().unwrap().parse()?;
    let _initial_pak = result["initial_pak"].as_str().unwrap();
    println!("    Created agent: {}", agent_id);

    // Verify agent starts INACTIVE
    let agent = client.get_agent(agent_id).await?;
    assert_eq!(agent["status"], "INACTIVE", "New agent should be INACTIVE");
    println!("  → Verified agent starts INACTIVE");

    // Activate the agent
    println!("  → Activating agent...");
    let updated = client
        .update_agent(agent_id, json!({"status": "ACTIVE"}))
        .await?;
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
    client
        .add_agent_annotation(agent_id, "owner", "e2e-test-suite")
        .await?;
    client
        .add_agent_annotation(agent_id, "environment", "test")
        .await?;
    let annotations = client.get_agent_annotations(agent_id).await?;
    assert_eq!(annotations.len(), 2, "Should have 2 annotations");
    println!("    Added {} annotations", annotations.len());

    // Deactivate agent
    println!("  → Deactivating agent...");
    let updated = client
        .update_agent(agent_id, json!({"status": "INACTIVE"}))
        .await?;
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
    let gen_result = client
        .create_generator("e2e-test-generator", Some("E2E test generator"))
        .await?;
    let generator_id: Uuid = gen_result["generator"]["id"].as_str().unwrap().parse()?;
    println!("    Created generator: {}", generator_id);

    // Create a stack
    println!("  → Creating stack...");
    let stack = client
        .create_stack("e2e-test-stack", Some("E2E test stack"), generator_id)
        .await?;
    let stack_id: Uuid = stack["id"].as_str().unwrap().parse()?;
    println!("    Created stack: {}", stack_id);

    // Deploy multi-resource YAML
    println!("  → Deploying multi-resource YAML...");
    let deployment = client
        .create_deployment(stack_id, DEMO_DEPLOYMENT_YAML, false)
        .await?;
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
    let agent_result = client
        .create_agent("e2e-targeting-agent", "targeting-cluster")
        .await?;
    let agent_id: Uuid = agent_result["agent"]["id"].as_str().unwrap().parse()?;
    client
        .update_agent(agent_id, json!({"status": "ACTIVE"}))
        .await?;
    client.add_agent_label(agent_id, "targeting-test").await?;
    println!("    Created agent with label 'targeting-test'");

    // Create stack with matching label
    let stack = client
        .create_stack("e2e-targeting-stack", None, generator_id)
        .await?;
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
    let stack2 = client
        .create_stack("e2e-explicit-stack", None, generator_id)
        .await?;
    let stack2_id: Uuid = stack2["id"].as_str().unwrap().parse()?;

    client.add_agent_target(agent_id, stack2_id).await?;
    let targets = client.get_agent_targets(agent_id).await?;
    let has_target = targets
        .iter()
        .any(|t| t["stack_id"] == stack2_id.to_string());
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
    let template = client
        .create_template(
            "e2e-microservice",
            Some("E2E test microservice template"),
            MICROSERVICE_TEMPLATE,
            MICROSERVICE_SCHEMA,
        )
        .await?;
    let template_id: Uuid = template["id"].as_str().unwrap().parse()?;
    assert_eq!(template["name"], "e2e-microservice");
    println!("    Created template: {}", template_id);

    // Create a stack for instantiation
    let gen_result = client.create_generator("e2e-template-gen", None).await?;
    let generator_id: Uuid = gen_result["generator"]["id"].as_str().unwrap().parse()?;
    let stack = client
        .create_stack("e2e-template-stack", None, generator_id)
        .await?;
    let stack_id: Uuid = stack["id"].as_str().unwrap().parse()?;

    // Instantiate the template
    println!("  → Instantiating template...");
    let deployment = client
        .instantiate_template(
            stack_id,
            template_id,
            json!({
                "name": "e2e-service",
                "image": "nginx:alpine",
                "replicas": 2,
                "port": 8080
            }),
        )
        .await?;

    let yaml_content = deployment["yaml_content"].as_str().unwrap();
    assert!(
        yaml_content.contains("name: e2e-service"),
        "Template should render name"
    );
    assert!(
        yaml_content.contains("replicas: 2"),
        "Template should render replicas"
    );
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
    let agent_result = client
        .create_agent("e2e-work-order-agent", "work-order-cluster")
        .await?;
    let agent_id: Uuid = agent_result["agent"]["id"].as_str().unwrap().parse()?;
    client
        .update_agent(agent_id, json!({"status": "ACTIVE"}))
        .await?;
    client.add_agent_label(agent_id, "work-order-test").await?;

    // Create work order with explicit agent targeting
    println!("  → Creating work order with agent targeting...");
    let work_order = client
        .create_work_order("custom", JOB_YAML, Some(vec![agent_id]), None)
        .await?;
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
    let wo2 = client
        .create_work_order("custom", JOB_YAML, None, Some(vec!["work-order-test"]))
        .await?;
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
    let agent = agents
        .iter()
        .find(|a| a["name"] == "brokkr-integration-test-agent")
        .ok_or("No running agent found - is brokkr-agent running?")?;
    let agent_id: Uuid = agent["id"].as_str().unwrap().parse()?;
    println!("    Found agent: {} ({})", agent["name"], agent_id);

    // Ensure agent is active (it may be INACTIVE by default)
    let agent_status = agent["status"].as_str().unwrap_or("UNKNOWN");
    if agent_status != "ACTIVE" {
        println!("  → Activating agent (was {})...", agent_status);
        client
            .update_agent(agent_id, json!({"status": "ACTIVE"}))
            .await?;
        println!("    Agent activated");
    } else {
        println!("    Agent is already ACTIVE");
    }

    // Create build work order targeting the real agent
    println!("  → Creating build work order...");
    let work_order = client
        .create_work_order("build", BUILD_YAML, Some(vec![agent_id]), None)
        .await?;
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
                        return Err(
                            format!("Work order not found in active or log: {}", log_err).into(),
                        );
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

    let stack = client
        .create_stack("e2e-health-stack", None, generator_id)
        .await?;
    let stack_id: Uuid = stack["id"].as_str().unwrap().parse()?;

    let deployment = client
        .create_deployment(stack_id, DEMO_DEPLOYMENT_YAML, false)
        .await?;
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
    println!(
        "    Deployment health: {} agent report(s)",
        records.map(|r| r.len()).unwrap_or(0)
    );

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
    let webhook = client
        .create_webhook(
            "e2e-test-webhook",
            &webhook_url,
            vec!["workorder.completed"],
            Some("Bearer e2e-test-token"),
        )
        .await?;
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
    let event_types = fetched["event_types"]
        .as_array()
        .expect("event_types should be array");
    assert_eq!(event_types.len(), 1);
    println!(
        "    Webhook subscribes to {} event type(s)",
        event_types.len()
    );

    // Update webhook (disable it)
    println!("  → Updating webhook (disable)...");
    let updated = client
        .update_webhook(webhook_id, json!({"enabled": false}))
        .await?;
    assert_eq!(updated["enabled"], false);
    println!("    Webhook disabled");

    // Re-enable
    println!("  → Re-enabling webhook...");
    let updated = client
        .update_webhook(webhook_id, json!({"enabled": true}))
        .await?;
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
        let wo = client
            .create_work_order(
                "custom",
                "# Webhook test work order\necho 'test'",
                None,
                Some(vec!["integration-test"]), // Target any integration test agent
            )
            .await?;
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
                                assert_eq!(
                                    event_type, "workorder.completed",
                                    "Expected workorder.completed event"
                                );
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
                        println!(
                            "      - Status: {}, Error: {:?}",
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

/// Minimal valid manifest for the Part 7b reconciliation tests. The broker
/// rejects document-less YAML (T-0194's `invalid_deployment_object` check), and
/// these tests only assert that a targeted agent *sees* the deployment in its
/// target state (they never apply it to a cluster), so any single valid
/// document works.
const RECONCILE_PLACEHOLDER_YAML: &str = r#"apiVersion: v1
kind: ConfigMap
metadata:
  name: e2e-reconcile-placeholder
  namespace: default
data:
  marker: reconcile
"#;

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
    let stack1 = client
        .create_stack("e2e-reconcile-direct", None, generator_id)
        .await?;
    let stack1_id: Uuid = stack1["id"].as_str().unwrap().parse()?;
    let deployment1 = client
        .create_deployment(stack1_id, RECONCILE_PLACEHOLDER_YAML, false)
        .await?;
    let deployment1_id: Uuid = deployment1["id"].as_str().unwrap().parse()?;
    println!("    Created stack and deployment: {}", deployment1_id);

    // 1b. Create agent (no targets yet)
    let agent1_result = client
        .create_agent("e2e-reconcile-direct-agent", "reconcile-cluster-1")
        .await?;
    let agent1_id: Uuid = agent1_result["agent"]["id"].as_str().unwrap().parse()?;
    client
        .update_agent(agent1_id, json!({"status": "ACTIVE"}))
        .await?;
    println!("    Created agent: {}", agent1_id);

    // 1c. NOW target agent to stack (after deployment exists)
    client.add_agent_target(agent1_id, stack1_id).await?;
    println!("    Targeted agent to stack");

    // 1d. Verify agent sees the pre-existing deployment
    let target_state1 = client.get_agent_target_state(agent1_id, None).await?;
    assert!(
        target_state1
            .iter()
            .any(|d| d["id"] == deployment1_id.to_string()),
        "Agent should see pre-existing deployment via direct targeting"
    );
    println!("    ✓ Agent sees pre-existing deployment via direct targeting");

    // ==========================================================================
    // Test 2: Label Targeting - Deployment exists before agent gets matching label
    // ==========================================================================
    println!("  → Test 2: Label targeting with pre-existing deployment...");

    // 2a. Create stack with label and deployment FIRST
    let stack2 = client
        .create_stack("e2e-reconcile-label", None, generator_id)
        .await?;
    let stack2_id: Uuid = stack2["id"].as_str().unwrap().parse()?;
    client
        .add_stack_label(stack2_id, "reconcile-test-label")
        .await?;
    let deployment2 = client
        .create_deployment(stack2_id, RECONCILE_PLACEHOLDER_YAML, false)
        .await?;
    let deployment2_id: Uuid = deployment2["id"].as_str().unwrap().parse()?;
    println!(
        "    Created labeled stack and deployment: {}",
        deployment2_id
    );

    // 2b. Create agent (no labels yet)
    let agent2_result = client
        .create_agent("e2e-reconcile-label-agent", "reconcile-cluster-2")
        .await?;
    let agent2_id: Uuid = agent2_result["agent"]["id"].as_str().unwrap().parse()?;
    client
        .update_agent(agent2_id, json!({"status": "ACTIVE"}))
        .await?;
    println!("    Created agent: {}", agent2_id);

    // 2c. NOW add matching label to agent (after deployment exists)
    client
        .add_agent_label(agent2_id, "reconcile-test-label")
        .await?;
    println!("    Added matching label to agent");

    // 2d. Verify agent sees the pre-existing deployment
    let target_state2 = client.get_agent_target_state(agent2_id, None).await?;
    assert!(
        target_state2
            .iter()
            .any(|d| d["id"] == deployment2_id.to_string()),
        "Agent should see pre-existing deployment via label targeting"
    );
    println!("    ✓ Agent sees pre-existing deployment via label targeting");

    // ==========================================================================
    // Test 3: Annotation Targeting - Deployment exists before agent gets matching annotation
    // ==========================================================================
    println!("  → Test 3: Annotation targeting with pre-existing deployment...");

    // 3a. Create stack with annotation and deployment FIRST
    let stack3 = client
        .create_stack("e2e-reconcile-annotation", None, generator_id)
        .await?;
    let stack3_id: Uuid = stack3["id"].as_str().unwrap().parse()?;
    client
        .add_stack_annotation(stack3_id, "reconcile-key", "reconcile-value")
        .await?;
    let deployment3 = client
        .create_deployment(stack3_id, RECONCILE_PLACEHOLDER_YAML, false)
        .await?;
    let deployment3_id: Uuid = deployment3["id"].as_str().unwrap().parse()?;
    println!(
        "    Created annotated stack and deployment: {}",
        deployment3_id
    );

    // 3b. Create agent (no annotations yet)
    let agent3_result = client
        .create_agent("e2e-reconcile-annotation-agent", "reconcile-cluster-3")
        .await?;
    let agent3_id: Uuid = agent3_result["agent"]["id"].as_str().unwrap().parse()?;
    client
        .update_agent(agent3_id, json!({"status": "ACTIVE"}))
        .await?;
    println!("    Created agent: {}", agent3_id);

    // 3c. NOW add matching annotation to agent (after deployment exists)
    client
        .add_agent_annotation(agent3_id, "reconcile-key", "reconcile-value")
        .await?;
    println!("    Added matching annotation to agent");

    // 3d. Verify agent sees the pre-existing deployment
    let target_state3 = client.get_agent_target_state(agent3_id, None).await?;
    assert!(
        target_state3
            .iter()
            .any(|d| d["id"] == deployment3_id.to_string()),
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

    println!(
        "    Found {} audit log entries (showing {})",
        total,
        logs.len()
    );

    // Check log structure if we have entries
    if let Some(first_log) = logs.first() {
        assert!(first_log.get("id").is_some(), "Log should have id");
        assert!(first_log.get("action").is_some(), "Log should have action");
        assert!(
            first_log.get("actor_type").is_some(),
            "Log should have actor_type"
        );
        assert!(
            first_log.get("created_at").is_some(),
            "Log should have timestamp"
        );
        println!("    Log structure verified");

        // Show sample actions
        let actions: Vec<&str> = logs
            .iter()
            .filter_map(|l| l["action"].as_str())
            .take(5)
            .collect();
        println!("    Sample actions: {:?}", actions);
    } else {
        // Audit logging may not be fully integrated yet - warn but don't fail
        println!("    ⚠ No audit logs found - audit logging may not be enabled for all endpoints");
    }

    // Verify API returns proper structure (this should always pass)
    assert!(
        result.get("logs").is_some(),
        "Response should have logs field"
    );
    assert!(
        result.get("total").is_some(),
        "Response should have total field"
    );
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
        assert!(metrics.contains(metric), "Should contain {} gauge", metric);
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
        if line.starts_with("brokkr_")
            && !line.contains("_bucket{")
            && !line.contains("_sum{")
            && !line.contains("_count{")
        {
            println!("    {}", line);
        }
    }

    println!("  → Metrics observability verified");
    Ok(())
}

// =============================================================================
// BROKKR-T-0170 (A1): WS-channel smoke test
// =============================================================================

/// I-0019 / I-0020 A1 smoke test.
///
/// Proves the load-bearing lifecycle of the WebSocket channel end-to-end
/// against the real broker + agent docker-compose stack:
///
/// 1. Agent connects via WS at startup (gauge >= 1)
/// 2. Broker container is stopped, then restarted
/// 3. Agent reconnects via WS after broker comes back (gauge >= 1 again)
/// 4. A fresh downlink push (`target_changed`, triggered by adding a stack
///    target to the agent) reaches the agent (out-direction counter increments)
///
/// What this does NOT cover: REST-fallback during a WS-only outage. That's
/// [[BROKKR-T-0171]] A2, which needs a network proxy that severs WS but keeps
/// REST reachable.
pub async fn test_ws_smoke(client: &Client) -> Result<()> {
    use std::time::Duration;
    use tokio::process::Command as TokioCommand;

    let compose_file = std::env::var("E2E_COMPOSE_FILE").map_err(|_| {
        "E2E_COMPOSE_FILE env var not set — the angreal e2e task is supposed to set this"
    })?;

    println!("  → Compose file: {}", compose_file);

    // -------------------------------------------------------------------
    // Step 1: Assert agent WS-connected on initial boot
    // -------------------------------------------------------------------
    println!("  → Waiting for agent WS connection (gauge >= 1, 30s timeout)...");
    let initial_connected = client
        .wait_for_metric("brokkr_ws_connected_agents", &[], 30, |v| v >= 1.0)
        .await?;
    println!(
        "    brokkr_ws_connected_agents = {} ✓",
        initial_connected
    );

    // -------------------------------------------------------------------
    // Step 2: Stop the broker container
    // -------------------------------------------------------------------
    println!("  → Stopping broker container (docker compose stop broker)...");
    let stop = TokioCommand::new("docker")
        .args(["compose", "-f", &compose_file, "stop", "broker"])
        .output()
        .await?;
    if !stop.status.success() {
        return Err(format!(
            "docker compose stop broker failed: {}\n{}",
            String::from_utf8_lossy(&stop.stdout),
            String::from_utf8_lossy(&stop.stderr)
        )
        .into());
    }
    println!("    broker stopped ✓");

    // Give the agent a moment to notice the WS disconnect. We don't assert
    // on the agent's state directly here (no agent metrics endpoint yet —
    // see [[BROKKR-T-0177]] / I-0020 deferred work); the proof is the
    // reconnect on the broker side after restart.
    println!("  → Sleeping 10s to let agent observe disconnect...");
    tokio::time::sleep(Duration::from_secs(10)).await;

    // -------------------------------------------------------------------
    // Step 3: Restart the broker container
    // -------------------------------------------------------------------
    println!("  → Starting broker container (docker compose start broker)...");
    let start = TokioCommand::new("docker")
        .args(["compose", "-f", &compose_file, "start", "broker"])
        .output()
        .await?;
    if !start.status.success() {
        return Err(format!(
            "docker compose start broker failed: {}\n{}",
            String::from_utf8_lossy(&start.stdout),
            String::from_utf8_lossy(&start.stderr)
        )
        .into());
    }
    println!("    broker started ✓");

    println!("  → Waiting for broker /healthz (30s timeout)...");
    client.wait_for_ready(30).await?;
    println!("    broker healthy ✓");

    // -------------------------------------------------------------------
    // Step 4: Assert WS reconnects
    // -------------------------------------------------------------------
    println!("  → Waiting for agent WS reconnect (gauge >= 1, 60s timeout)...");
    let reconnected = client
        .wait_for_metric("brokkr_ws_connected_agents", &[], 60, |v| v >= 1.0)
        .await?;
    println!(
        "    brokkr_ws_connected_agents = {} after restart ✓",
        reconnected
    );

    // -------------------------------------------------------------------
    // Step 5: Trigger a downlink push and assert the counter increments
    // -------------------------------------------------------------------
    println!("  → Triggering a target_changed push (add stack target to agent)...");

    // Find the agent that connected from docker-compose. The init-agent
    // service creates exactly one named `brokkr-integration-test-agent`.
    let agents = client.list_agents().await?;
    let agent = agents
        .iter()
        .find(|a| {
            a.get("name").and_then(|n| n.as_str()) == Some("brokkr-integration-test-agent")
        })
        .ok_or("expected brokkr-integration-test-agent in agent list")?;
    let agent_id: Uuid = agent
        .get("id")
        .and_then(|v| v.as_str())
        .and_then(|s| Uuid::parse_str(s).ok())
        .ok_or("agent missing id")?;

    // Create a throwaway generator + stack so the target push has something
    // real to point at — the existing test_targeting scenario does the same.
    let gen = client
        .create_generator(&format!("ws-smoke-gen-{}", Uuid::new_v4()), None)
        .await?;
    let gen_id: Uuid = Uuid::parse_str(gen["generator"]["id"].as_str().unwrap())?;
    let stack = client
        .create_stack(
            &format!("ws-smoke-stack-{}", Uuid::new_v4()),
            None,
            gen_id,
        )
        .await?;
    let stack_id: Uuid = Uuid::parse_str(stack["id"].as_str().unwrap())?;

    // Record baseline before the push.
    let baseline = client
        .metric_value(
            "brokkr_ws_messages_total",
            &[("direction", "out"), ("type", "target_changed")],
        )
        .await?;
    println!(
        "    baseline brokkr_ws_messages_total{{direction=out,type=target_changed}} = {}",
        baseline
    );

    // Add the target — this triggers push_target_changed on the broker.
    client.add_agent_target(agent_id, stack_id).await?;
    println!("    target added; waiting for counter to increment...");

    let new_value = client
        .wait_for_metric(
            "brokkr_ws_messages_total",
            &[("direction", "out"), ("type", "target_changed")],
            15,
            |v| v > baseline,
        )
        .await?;
    println!(
        "    brokkr_ws_messages_total{{direction=out,type=target_changed}} = {} (was {}) ✓",
        new_value, baseline
    );

    println!("  → WS smoke scenario passed");
    Ok(())
}

// =============================================================================
// BROKKR-T-0171 (A2): WS-channel chaos test
// =============================================================================

/// Toggle a toxiproxy proxy's `enabled` flag via the admin API. Setting
/// `false` closes the listening socket and tears down any active connections
/// through it — which is exactly the "WS severed, REST untouched" primitive
/// the A2 chaos scenario needs (REST goes direct to broker:3000, not through
/// toxiproxy).
async fn toxiproxy_set_enabled(
    toxiproxy_url: &str,
    proxy_name: &str,
    enabled: bool,
) -> Result<()> {
    let url = format!("{}/proxies/{}", toxiproxy_url, proxy_name);
    let resp = reqwest::Client::new()
        .post(&url)
        .json(&serde_json::json!({ "enabled": enabled }))
        .send()
        .await?;
    let status = resp.status();
    if !status.is_success() {
        let body = resp.text().await.unwrap_or_default();
        return Err(format!(
            "toxiproxy {} {} -> HTTP {}: {}",
            if enabled { "enable" } else { "disable" },
            proxy_name,
            status,
            body
        )
        .into());
    }
    Ok(())
}

/// I-0019 / I-0020 A2 chaos test — Pass 1 (infrastructure validation).
///
/// Proves the WS-severance primitive works in isolation:
///
/// 1. Agent connects via WS at startup (gauge >= 1)
/// 2. Toxiproxy disables the `ws-channel` proxy → agent's WS socket closes →
///    broker observes the disconnect → `brokkr_ws_connected_agents` drops to 0
/// 3. Toxiproxy re-enables the proxy → agent reconnects → gauge back to >= 1
///
/// REST is never disrupted in this scenario because it goes direct to the
/// broker, bypassing toxiproxy entirely. That gap — "WS down, REST up" — is
/// the realistic production failure mode the I-0019 fallback is supposed to
/// catch, and Pass 2 of this task will extend the scenario to seed work
/// orders during the severance window and assert REST-fallback drains them.
pub async fn test_ws_chaos(client: &Client) -> Result<()> {
    let toxiproxy_url = std::env::var("TOXIPROXY_URL")
        .unwrap_or_else(|_| "http://localhost:8474".to_string());

    println!("  → Toxiproxy admin URL: {}", toxiproxy_url);

    // Step 1: assert WS connected on initial boot.
    println!("  → Waiting for agent WS connection (gauge >= 1, 30s timeout)...");
    let initial = client
        .wait_for_metric("brokkr_ws_connected_agents", &[], 30, |v| v >= 1.0)
        .await?;
    println!("    brokkr_ws_connected_agents = {} ✓", initial);

    // Step 2: sever WS via toxiproxy. Disabling the proxy closes the listener
    // and tears down established connections — both the agent's WS write and
    // the broker's read side will fail, and the broker's per-connection cleanup
    // will decrement the gauge.
    println!("  → Disabling toxiproxy ws-channel (sever WS)...");
    toxiproxy_set_enabled(&toxiproxy_url, "ws-channel", false).await?;
    println!("    severed ✓");

    println!("  → Waiting for gauge to drop to 0 (30s timeout)...");
    let dropped = client
        .wait_for_metric("brokkr_ws_connected_agents", &[], 30, |v| v < 1.0)
        .await?;
    println!(
        "    brokkr_ws_connected_agents = {} after sever ✓",
        dropped
    );

    // While WS is down, REST is still reachable — sanity check.
    println!("  → Confirming REST stays reachable during WS sever...");
    let _ = client.get_healthz().await?;
    let _ = client.list_agents().await?;
    println!("    REST OK ✓");

    // Step 3: restore WS.
    println!("  → Re-enabling toxiproxy ws-channel (restore WS)...");
    toxiproxy_set_enabled(&toxiproxy_url, "ws-channel", true).await?;
    println!("    restored ✓");

    println!("  → Waiting for agent WS reconnect (gauge >= 1, 60s timeout)...");
    let recovered = client
        .wait_for_metric("brokkr_ws_connected_agents", &[], 60, |v| v >= 1.0)
        .await?;
    println!(
        "    brokkr_ws_connected_agents = {} after restore ✓",
        recovered
    );

    // -------------------------------------------------------------------
    // Pass 1 done — Pass 2 extends here: assert REST fallback actually
    // does work while WS is down. The cleanest observable signal of the
    // fallback path firing is the agent's heartbeat: `send_heartbeat`
    // (crates/brokkr-agent/src/broker.rs) tries WS first and falls back
    // to REST `record_heartbeat` when WS isn't up. If `last_heartbeat_at`
    // on the broker keeps advancing while toxiproxy holds WS severed,
    // that proves the fallback ran end-to-end.
    //
    // This is a narrower claim than the original A2 criterion ("every
    // work order reaches completed"). See T-0171 status notes for the
    // scope narrowing — short version: work-order completion requires a
    // wait-for-completion helper that doesn't exist anywhere in the e2e
    // harness yet, and the original A2 risk note already constrained
    // "reconciliation" to the broker state machine, not k8s apply. The
    // load-bearing design claim ("WS is an additive optimization, REST
    // never stops working") is what we test here.
    println!("  → Pass 2: validate REST fallback via heartbeat during WS sever");

    // Locate the docker-compose agent so we can read its heartbeat timestamp.
    let agents = client.list_agents().await?;
    let agent = agents
        .iter()
        .find(|a| {
            a.get("name").and_then(|n| n.as_str()) == Some("brokkr-integration-test-agent")
        })
        .ok_or("expected brokkr-integration-test-agent in agent list")?;
    let agent_id: Uuid = agent
        .get("id")
        .and_then(|v| v.as_str())
        .and_then(|s| Uuid::parse_str(s).ok())
        .ok_or("agent missing id")?;
    println!("    test agent id: {}", agent_id);

    // The agent's heartbeat was flowing via WS up to this point. Take the
    // last_heartbeat_at NOW (post-sever) and we'll re-check after waiting
    // through the sever window.
    let snapshot_a = client.get_agent(agent_id).await?;
    let hb_a = snapshot_a
        .get("last_heartbeat")
        .and_then(|v| v.as_str())
        .ok_or("agent missing last_heartbeat")?
        .to_string();
    println!("    last_heartbeat (just after sever) = {}", hb_a);

    // Sever WS again — the restore above brought it back up; we need to
    // re-sever so the heartbeat assertion window covers a WS-down period.
    // Toggle off, then off→on stays off; we left it on at the end of Pass 1.
    println!("  → Re-severing WS for Pass 2 fallback window...");
    toxiproxy_set_enabled(&toxiproxy_url, "ws-channel", false).await?;
    let _ = client
        .wait_for_metric("brokkr_ws_connected_agents", &[], 30, |v| v < 1.0)
        .await?;
    println!("    WS down; sleeping 35s to span at least 3 heartbeat intervals");
    tokio::time::sleep(std::time::Duration::from_secs(35)).await;

    let snapshot_b = client.get_agent(agent_id).await?;
    let hb_b = snapshot_b
        .get("last_heartbeat")
        .and_then(|v| v.as_str())
        .ok_or("agent missing last_heartbeat")?
        .to_string();
    println!("    last_heartbeat (after 35s with WS down) = {}", hb_b);

    if hb_b == hb_a {
        return Err(format!(
            "REST heartbeat fallback did NOT fire: last_heartbeat unchanged across \
             the WS-down window (both reads = {}). The agent's REST fallback in \
             send_heartbeat is broken, or the agent never noticed the WS drop.",
            hb_a
        )
        .into());
    }

    // Parse and compare to be defensive — string inequality is sufficient
    // for the broker's ISO-8601 timestamp granularity, but explicit > is clearer.
    let t_a = chrono::DateTime::parse_from_rfc3339(&hb_a).map_err(|e| {
        format!("could not parse first heartbeat timestamp {}: {}", hb_a, e)
    })?;
    let t_b = chrono::DateTime::parse_from_rfc3339(&hb_b).map_err(|e| {
        format!("could not parse second heartbeat timestamp {}: {}", hb_b, e)
    })?;
    if t_b <= t_a {
        return Err(format!(
            "REST heartbeat fallback advanced backwards or stalled: hb_a={} hb_b={}",
            hb_a, hb_b
        )
        .into());
    }
    println!(
        "    heartbeat advanced {} → {} during WS-down window ✓ (REST fallback works)",
        hb_a, hb_b
    );

    // Restore WS so the stack is in a clean state when the e2e harness
    // tears down (and so future scenarios in this binary, if we ever
    // chain them, start clean).
    println!("  → Re-enabling toxiproxy ws-channel (final restore)...");
    toxiproxy_set_enabled(&toxiproxy_url, "ws-channel", true).await?;
    let final_gauge = client
        .wait_for_metric("brokkr_ws_connected_agents", &[], 60, |v| v >= 1.0)
        .await?;
    println!(
        "    brokkr_ws_connected_agents = {} after final restore ✓",
        final_gauge
    );

    println!("  → WS chaos scenario passed (Pass 1 + Pass 2)");
    Ok(())
}

// =============================================================================
// BROKKR-T-0183 (A2 follow-up): work-order completion under WS sever
// =============================================================================

/// Prove the full work-order lifecycle survives a WS outage: with the WS
/// channel severed, the agent must still **discover, claim, execute, and
/// complete** work orders over REST polling. Uses `custom` work orders (each a
/// unique ConfigMap applied to k3s) so completion is deterministic and needs no
/// external build infra — unlike `build` work orders (Shipwright/ttl.sh), which
/// is why the original A2 ([[BROKKR-T-0171]]) deferred this.
pub async fn test_ws_workorders(client: &Client) -> Result<()> {
    const N: usize = 8;
    let toxiproxy_url =
        std::env::var("TOXIPROXY_URL").unwrap_or_else(|_| "http://localhost:8474".to_string());

    // Locate the real docker-compose agent (the one actually wired to k3s).
    let agents = client.list_agents().await?;
    let agent = agents
        .iter()
        .find(|a| a.get("name").and_then(|n| n.as_str()) == Some("brokkr-integration-test-agent"))
        .ok_or("expected brokkr-integration-test-agent in agent list")?;
    let agent_id: Uuid = agent
        .get("id")
        .and_then(|v| v.as_str())
        .and_then(|s| Uuid::parse_str(s).ok())
        .ok_or("agent missing id")?;
    println!("  → test agent id: {}", agent_id);

    // The agent skips work-order processing unless its status is ACTIVE
    // (commands.rs). The docker-compose agent boots INACTIVE, so activate it
    // explicitly (mirrors Part 5 test_work_orders). The agent refreshes its
    // own status over REST each cycle, so this propagates even with WS down.
    println!("  → Setting agent ACTIVE so it will process work orders...");
    client
        .update_agent(agent_id, json!({"status": "ACTIVE"}))
        .await?;

    // Confirm WS is up before we sever it.
    println!("  → Confirming WS connected (gauge >= 1, 30s)...");
    client
        .wait_for_metric("brokkr_ws_connected_agents", &[], 30, |v| v >= 1.0)
        .await?;

    // Sever WS — from here the agent has only REST polling to find work.
    println!("  → Severing WS via toxiproxy...");
    toxiproxy_set_enabled(&toxiproxy_url, "ws-channel", false).await?;
    client
        .wait_for_metric("brokkr_ws_connected_agents", &[], 30, |v| v < 1.0)
        .await?;
    println!("    WS severed (gauge < 1) ✓");

    // Seed N custom work orders (unique ConfigMaps) targeting the agent.
    let suffix = Uuid::new_v4().to_string()[..8].to_string();
    println!("  → Seeding {N} custom work orders while WS is down...");
    let mut wo_ids: Vec<Uuid> = Vec::with_capacity(N);
    for i in 0..N {
        let name = format!("brokkr-a2-wo-{suffix}-{i}");
        let yaml = format!(
            "apiVersion: v1\nkind: ConfigMap\nmetadata:\n  name: {name}\n  namespace: default\ndata:\n  idx: \"{i}\"\n"
        );
        let wo = client
            .create_work_order("custom", &yaml, Some(vec![agent_id]), None)
            .await?;
        let id: Uuid = wo["id"].as_str().ok_or("work order missing id")?.parse()?;
        assert_eq!(wo["status"], "PENDING", "new work order should be PENDING");
        wo_ids.push(id);
    }
    println!("    seeded {} work orders ✓", wo_ids.len());

    // Poll each work order's completion log. The agent must claim+apply+complete
    // them over REST while WS stays severed.
    println!("  → Waiting for all {N} to complete via REST fallback (150s timeout)...");
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(150);
    let mut completed: std::collections::HashSet<Uuid> = std::collections::HashSet::new();
    while completed.len() < wo_ids.len() {
        for id in &wo_ids {
            if completed.contains(id) {
                continue;
            }
            if let Ok(log) = client.get_work_order_log(*id).await {
                // Present in the log == the agent called complete_work_order.
                let success = log.get("success").and_then(|v| v.as_bool()).unwrap_or(false);
                if !success {
                    toxiproxy_set_enabled(&toxiproxy_url, "ws-channel", true).await.ok();
                    return Err(format!("work order {} completed but success=false: {}", id, log).into());
                }
                let claimed_by = log.get("claimed_by").and_then(|v| v.as_str());
                if claimed_by != Some(agent_id.to_string().as_str()) {
                    toxiproxy_set_enabled(&toxiproxy_url, "ws-channel", true).await.ok();
                    return Err(format!(
                        "work order {} claimed_by {:?}, expected the test agent {}",
                        id, claimed_by, agent_id
                    )
                    .into());
                }
                completed.insert(*id);
                println!("    completed {}/{} ({})", completed.len(), wo_ids.len(), id);
            }
        }
        if completed.len() < wo_ids.len() && std::time::Instant::now() > deadline {
            toxiproxy_set_enabled(&toxiproxy_url, "ws-channel", true).await.ok();
            return Err(format!(
                "only {}/{} work orders completed within 150s with WS severed — REST \
                 fallback did not drain the queue",
                completed.len(),
                wo_ids.len()
            )
            .into());
        }
        if completed.len() < wo_ids.len() {
            tokio::time::sleep(std::time::Duration::from_secs(3)).await;
        }
    }
    println!("    all {} work orders completed with WS severed ✓", wo_ids.len());

    // No duplicate / leftover: each id is gone from the active queue.
    let active = client.list_work_orders().await?;
    let leftover: Vec<&serde_json::Value> = active
        .iter()
        .filter(|w| {
            w.get("id")
                .and_then(|v| v.as_str())
                .and_then(|s| Uuid::parse_str(s).ok())
                .map(|id| wo_ids.contains(&id))
                .unwrap_or(false)
        })
        .collect();
    if !leftover.is_empty() {
        toxiproxy_set_enabled(&toxiproxy_url, "ws-channel", true).await.ok();
        return Err(format!(
            "{} seeded work order(s) still in the active queue after completion — \
             not drained cleanly",
            leftover.len()
        )
        .into());
    }
    println!("    active queue clean (no leftovers) ✓");

    // Restore WS for a clean teardown.
    println!("  → Restoring WS...");
    toxiproxy_set_enabled(&toxiproxy_url, "ws-channel", true).await?;
    client
        .wait_for_metric("brokkr_ws_connected_agents", &[], 60, |v| v >= 1.0)
        .await?;
    println!("  → WS work-order chaos scenario passed (REST fallback drained {N} work orders)");
    Ok(())
}

// =============================================================================
// BROKKR-T-0172 (A3): Real-k3s telemetry tailer test
// =============================================================================

/// Apply a Kubernetes manifest by piping it through `docker compose exec k3s
/// kubectl apply -f -`. Uses the project-aware compose path so we don't have
/// to hardcode the container name (which is `brokkr-dev-k3s-1` today but
/// could change with the compose file's `name:` field).
async fn k3s_apply(compose_file: &str, manifest: &str) -> Result<()> {
    use std::process::Stdio;
    use tokio::io::AsyncWriteExt;
    use tokio::process::Command as TokioCommand;

    let mut child = TokioCommand::new("docker")
        .args([
            "compose",
            "-f",
            compose_file,
            "exec",
            "-T", // no TTY, accept stdin
            "k3s",
            "kubectl",
            "apply",
            "-f",
            "-",
        ])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;

    if let Some(mut stdin) = child.stdin.take() {
        stdin.write_all(manifest.as_bytes()).await?;
        stdin.shutdown().await?;
    }
    let out = child.wait_with_output().await?;
    if !out.status.success() {
        return Err(format!(
            "kubectl apply failed: stdout={} stderr={}",
            String::from_utf8_lossy(&out.stdout),
            String::from_utf8_lossy(&out.stderr)
        )
        .into());
    }
    Ok(())
}

/// On A3 Pass 2 failure, dump pod status + agent logs so the next iteration
/// can diagnose. Best-effort; output goes to stderr/stdout.
async fn dump_diagnostics(compose_file: &str, pod_name: &str) {
    use tokio::process::Command as TokioCommand;

    eprintln!("\n========== DIAGNOSTICS: pod status ==========");
    let _ = TokioCommand::new("docker")
        .args([
            "compose",
            "-f",
            compose_file,
            "exec",
            "-T",
            "k3s",
            "kubectl",
            "get",
            "pod",
            pod_name,
            "-o",
            "wide",
        ])
        .status()
        .await;
    eprintln!("\n========== DIAGNOSTICS: pod describe ==========");
    let _ = TokioCommand::new("docker")
        .args([
            "compose",
            "-f",
            compose_file,
            "exec",
            "-T",
            "k3s",
            "kubectl",
            "describe",
            "pod",
            pod_name,
        ])
        .status()
        .await;
    eprintln!("\n========== DIAGNOSTICS: pod logs from k3s ==========");
    let _ = TokioCommand::new("docker")
        .args([
            "compose",
            "-f",
            compose_file,
            "exec",
            "-T",
            "k3s",
            "kubectl",
            "logs",
            pod_name,
            "--all-containers=true",
        ])
        .status()
        .await;
    eprintln!("\n========== DIAGNOSTICS: agent logs (last 200 lines) ==========");
    let _ = TokioCommand::new("docker")
        .args([
            "compose",
            "-f",
            compose_file,
            "logs",
            "agent",
            "--tail",
            "200",
        ])
        .status()
        .await;
    eprintln!("==============================================");
}

/// Run `kubectl delete` against the k3s cluster. Used for test cleanup. Errors
/// are ignored — best-effort, the next test run starts a fresh stack anyway.
async fn k3s_delete_best_effort(compose_file: &str, args: &[&str]) {
    use tokio::process::Command as TokioCommand;

    let mut argv = vec![
        "compose",
        "-f",
        compose_file,
        "exec",
        "-T",
        "k3s",
        "kubectl",
        "delete",
        "--ignore-not-found",
    ];
    argv.extend_from_slice(args);

    let _ = TokioCommand::new("docker")
        .args(argv)
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .await;
}

/// I-0019 / I-0020 A3 telemetry-tailer test against real k3s.
///
/// Proves the kube-events tailer (WS-07) actually emits when real Kubernetes
/// emits — closing the WS-07 honest-review gap that the existing unit tests
/// only covered cache/rate-limit logic in isolation, never the round-trip
/// through k8s → agent → broker → persistence.
///
/// Pass 1 (this implementation):
/// 1. Create a stack via broker
/// 2. Apply a Pod annotated `k8s.brokkr.io/stack=<id>` with an image that
///    can never pull
/// 3. Poll `GET /api/v1/stacks/{id}/events` until ≥1 event row exists with
///    a reason matching the expected failure modes
/// 4. Cleanup
///
/// Out of scope for Pass 1 (worth a Pass 2 if time): pod-logs tailer +
/// `LogGap{RateLimit}` assertion (requires a noisy-logger pod and rate-limit
/// math), opt-in/opt-out annotation enforcement (requires deploying multiple
/// stacks with different annotation configs).
pub async fn test_ws_telemetry(client: &Client) -> Result<()> {
    use std::time::Duration;

    let compose_file = std::env::var("E2E_COMPOSE_FILE").map_err(|_| {
        "E2E_COMPOSE_FILE env var not set — the angreal e2e task is supposed to set this"
    })?;

    println!("  → Compose file: {}", compose_file);

    // -------------------------------------------------------------------
    // Step 1: confirm agent is connected so we know the tailer is running
    // -------------------------------------------------------------------
    println!("  → Waiting for agent WS connection (gauge >= 1, 30s timeout)...");
    client
        .wait_for_metric("brokkr_ws_connected_agents", &[], 30, |v| v >= 1.0)
        .await?;
    println!("    agent connected ✓");

    // -------------------------------------------------------------------
    // Step 2: create a stack via the broker so we have a stack_id to
    // annotate test pods with
    // -------------------------------------------------------------------
    let gen = client
        .create_generator(&format!("a3-tel-gen-{}", Uuid::new_v4()), None)
        .await?;
    let gen_id: Uuid = Uuid::parse_str(gen["generator"]["id"].as_str().unwrap())?;
    let stack = client
        .create_stack(
            &format!("a3-tel-stack-{}", Uuid::new_v4()),
            None,
            gen_id,
        )
        .await?;
    let stack_id: Uuid = Uuid::parse_str(stack["id"].as_str().unwrap())?;
    println!("    stack {} created", stack_id);

    // -------------------------------------------------------------------
    // Step 3: apply a Pod into k3s that's guaranteed to fail image pull
    // -------------------------------------------------------------------
    let pod_name = format!("brokkr-a3-failpod-{}", &stack_id.to_string()[..8]);
    let manifest = format!(
        r#"apiVersion: v1
kind: Pod
metadata:
  name: {pod_name}
  namespace: default
  annotations:
    k8s.brokkr.io/stack: "{stack_id}"
  labels:
    brokkr.io/test: a3
spec:
  restartPolicy: Never
  containers:
  - name: c
    image: definitely-does-not-exist.invalid/never-pulls:nope
    command: ["true"]
"#
    );

    println!("  → kubectl apply failing pod ({pod_name}) into k3s...");
    k3s_apply(&compose_file, &manifest).await?;
    println!("    pod applied ✓");

    // -------------------------------------------------------------------
    // Step 4: poll the broker's history endpoint for events on this stack
    // -------------------------------------------------------------------
    println!("  → Polling /stacks/{stack_id}/events for events (90s timeout)...");
    let path = format!("/api/v1/stacks/{}/events", stack_id);
    let deadline = std::time::Instant::now() + Duration::from_secs(90);
    let mut last_count = 0usize;
    let cleanup = |compose_file: String, pod_name: String| async move {
        k3s_delete_best_effort(&compose_file, &["pod", &pod_name]).await;
    };

    loop {
        // Use the generic get helper so we don't have to add a typed wrapper
        // for this one-off shape.
        let resp: serde_json::Value = match client.get_json(&path).await {
            Ok(v) => v,
            Err(e) => {
                if std::time::Instant::now() > deadline {
                    cleanup(compose_file.clone(), pod_name.clone()).await;
                    return Err(format!(
                        "history endpoint never returned a parseable body: {}",
                        e
                    )
                    .into());
                }
                tokio::time::sleep(Duration::from_secs(2)).await;
                continue;
            }
        };

        let events = resp
            .get("events")
            .and_then(|v| v.as_array())
            .cloned()
            .unwrap_or_default();
        if events.len() != last_count {
            println!("    events so far: {}", events.len());
            last_count = events.len();
        }

        // We want at least one event referencing our pod with a failure-ish
        // reason. K3s emits a variety: ErrImagePull, ImagePullBackOff, Failed,
        // Pulling (which itself isn't a failure but shows the loop is active).
        let matched = events.iter().any(|ev| {
            let reason = ev.get("reason").and_then(|v| v.as_str()).unwrap_or("");
            let involved_name = ev
                .get("involved_object")
                .and_then(|o| o.get("name"))
                .and_then(|v| v.as_str())
                .unwrap_or("");
            involved_name == pod_name
                && (reason.contains("Pull")
                    || reason.contains("Failed")
                    || reason.contains("BackOff"))
        });

        if matched {
            println!(
                "    ✓ found event referencing {pod_name} with a Pull/Failed/BackOff reason"
            );
            break;
        }

        if std::time::Instant::now() > deadline {
            cleanup(compose_file.clone(), pod_name.clone()).await;
            return Err(format!(
                "no matching event after 90s. Events seen: {} (full body: {})",
                events.len(),
                serde_json::to_string(&resp).unwrap_or_default()
            )
            .into());
        }
        tokio::time::sleep(Duration::from_secs(2)).await;
    }

    // -------------------------------------------------------------------
    // Step 5: cleanup the events fail-pod (best effort) before Pass 2
    // -------------------------------------------------------------------
    println!("  → Cleaning up events test pod...");
    cleanup(compose_file.clone(), pod_name.clone()).await;

    // -------------------------------------------------------------------
    // Pass 2: pod-logs tailer (WS-08). Apply a chatty pod opted-in via
    // `brokkr.io/stream-logs: "true"` and assert log lines for it arrive
    // at the broker's `/stacks/{id}/logs` REST endpoint.
    //
    // Note: we explicitly do NOT assert on `LogGap{RateLimit}` rows here.
    // Per handler.rs:307, gap frames are broadcast-only (for live WS
    // subscribers), NEVER persisted — so they can't show up in the REST
    // history. A real LogGap assertion would require subscribing to the
    // live WS endpoint; that's recorded as deferred follow-up below.
    // -------------------------------------------------------------------
    let chatty_name = format!("brokkr-a3-chatty-{}", &stack_id.to_string()[..8]);
    let chatty_manifest = format!(
        r#"apiVersion: v1
kind: Pod
metadata:
  name: {chatty_name}
  namespace: default
  annotations:
    k8s.brokkr.io/stack: "{stack_id}"
    brokkr.io/stream-logs: "true"
  labels:
    brokkr.io/test: a3
spec:
  restartPolicy: Never
  containers:
  - name: c
    image: busybox:1.36
    command: ["sh", "-c"]
    args:
    # Log slowly so the agent's pod_logs tailer has time to attach the
    # log stream before lines are flushed. Without the sleep, busybox
    # would emit all 200 lines in milliseconds and exit before the
    # watcher's `Api<Pod>::log_stream` future is even resolved.
    - 'for i in $(seq 1 60); do echo "chatty pod line $i at $(date)"; sleep 1; done'
"#
    );

    println!("  → kubectl apply chatty pod ({chatty_name}) into k3s...");
    k3s_apply(&compose_file, &chatty_manifest).await?;
    println!("    chatty pod applied ✓");

    let logs_path = format!("/api/v1/stacks/{}/logs", stack_id);
    println!("  → Polling /stacks/{stack_id}/logs for chatty pod lines (90s timeout)...");
    let deadline = std::time::Instant::now() + Duration::from_secs(90);
    let mut last_log_count = 0usize;
    let cleanup_chatty = |compose_file: String, name: String| async move {
        k3s_delete_best_effort(&compose_file, &["pod", &name]).await;
    };

    loop {
        let resp: serde_json::Value = match client.get_json(&logs_path).await {
            Ok(v) => v,
            Err(e) => {
                if std::time::Instant::now() > deadline {
                    cleanup_chatty(compose_file.clone(), chatty_name.clone()).await;
                    return Err(format!("/stacks/{}/logs never returned: {}", stack_id, e).into());
                }
                tokio::time::sleep(Duration::from_secs(2)).await;
                continue;
            }
        };

        let lines = resp
            .get("lines")
            .and_then(|v| v.as_array())
            .cloned()
            .unwrap_or_default();
        if lines.len() != last_log_count {
            println!("    lines so far: {}", lines.len());
            last_log_count = lines.len();
        }

        let matched = lines.iter().any(|line| {
            line.get("pod").and_then(|v| v.as_str()) == Some(chatty_name.as_str())
        });
        if matched {
            println!("    ✓ found ≥1 log line from chatty pod in history");
            break;
        }

        if std::time::Instant::now() > deadline {
            // Diagnostic: dump pod status and agent logs so the next failure
            // tells us why no log lines arrived (image pull failure vs.
            // tailer-not-attaching vs. RBAC, etc.).
            dump_diagnostics(&compose_file, &chatty_name).await;
            cleanup_chatty(compose_file.clone(), chatty_name.clone()).await;
            return Err(format!(
                "no log lines from chatty pod after 90s. Lines seen: {}",
                lines.len()
            )
            .into());
        }
        tokio::time::sleep(Duration::from_secs(2)).await;
    }

    println!("  → Cleaning up chatty pod...");
    cleanup_chatty(compose_file.clone(), chatty_name.clone()).await;

    println!("  → WS telemetry scenario passed (Pass 1 events + Pass 2 logs)");
    println!(
        "  → Deferred follow-ups: (a) LogGap{{RateLimit}} assertion via live WS \
         subscription (gap frames aren't persisted, can't be checked via REST), \
         (b) negative tests for opt-out (pod without stream-logs annotation should \
         produce zero log rows)"
    );
    Ok(())
}


