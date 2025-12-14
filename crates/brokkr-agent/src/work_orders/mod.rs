/*
 * Copyright (c) 2025 Dylan Storey
 * Licensed under the Elastic License 2.0.
 * See LICENSE file in the project root for full license text.
 */

//! # Work Orders Module
//!
//! This module handles the work order lifecycle for the Brokkr agent:
//! - Fetching pending work orders from the broker
//! - Claiming work orders for execution
//! - Executing work based on work type (e.g., builds)
//! - Reporting completion (success/failure) to the broker
//!
//! ## Work Order Flow
//!
//! ```text
//! 1. Poll broker for pending work orders
//! 2. Claim a work order
//! 3. Apply YAML content (Build + WorkOrder resources)
//! 4. Execute work type handler (e.g., create BuildRun for builds)
//! 5. Watch for completion
//! 6. Report result to broker
//! ```

pub mod broker;
pub mod build;

use brokkr_models::models::agents::Agent;
use brokkr_models::models::work_orders::WorkOrder;
use brokkr_utils::config::Settings;
use brokkr_utils::logging::prelude::*;
use kube::Client as K8sClient;
use reqwest::Client;

/// Processes pending work orders for the agent.
///
/// This function:
/// 1. Fetches pending work orders from the broker
/// 2. Claims the first available work order
/// 3. Executes the work based on work type
/// 4. Reports completion to the broker
///
/// # Arguments
/// * `config` - Application settings
/// * `http_client` - HTTP client for broker communication
/// * `k8s_client` - Kubernetes client for resource operations
/// * `agent` - Agent details
///
/// # Returns
/// Number of work orders processed
pub async fn process_pending_work_orders(
    config: &Settings,
    http_client: &Client,
    k8s_client: &K8sClient,
    agent: &Agent,
) -> Result<usize, Box<dyn std::error::Error>> {
    // Fetch pending work orders
    let pending = broker::fetch_pending_work_orders(config, http_client, agent, None).await?;

    if pending.is_empty() {
        trace!("No pending work orders for agent {}", agent.name);
        return Ok(0);
    }

    info!(
        "Found {} pending work orders for agent {}",
        pending.len(),
        agent.name
    );

    let mut processed = 0;

    // Process one work order at a time
    // In the future, we could parallelize this based on work type
    for work_order in pending.iter().take(1) {
        match process_single_work_order(config, http_client, k8s_client, agent, work_order).await {
            Ok(_) => {
                processed += 1;
                info!(
                    "Successfully processed work order {} (type: {})",
                    work_order.id, work_order.work_type
                );
            }
            Err(e) => {
                error!(
                    "Failed to process work order {} (type: {}): {}",
                    work_order.id, work_order.work_type, e
                );
                // Continue with next work order instead of failing completely
            }
        }
    }

    Ok(processed)
}

/// Processes a single work order through its complete lifecycle.
async fn process_single_work_order(
    config: &Settings,
    http_client: &Client,
    k8s_client: &K8sClient,
    agent: &Agent,
    work_order: &WorkOrder,
) -> Result<(), Box<dyn std::error::Error>> {
    info!(
        "Processing work order {} (type: {}, status: {})",
        work_order.id, work_order.work_type, work_order.status
    );

    // Claim the work order
    let claimed = broker::claim_work_order(config, http_client, agent, work_order.id).await?;
    info!("Successfully claimed work order {}", claimed.id);

    // Execute based on work type
    let result = match work_order.work_type.as_str() {
        "build" => {
            execute_build_work_order(config, http_client, k8s_client, agent, &claimed).await
        }
        unknown => {
            Err(format!("Unknown work type: {}", unknown).into())
        }
    };

    // Report completion
    match result {
        Ok(message) => {
            broker::complete_work_order(config, http_client, work_order.id, true, message).await?;
            info!("Work order {} completed successfully", work_order.id);
        }
        Err(e) => {
            let error_msg = e.to_string();
            broker::complete_work_order(config, http_client, work_order.id, false, Some(error_msg))
                .await?;
            error!("Work order {} failed: {}", work_order.id, e);
            return Err(e);
        }
    }

    Ok(())
}

/// Executes a build work order using Shipwright.
async fn execute_build_work_order(
    _config: &Settings,
    _http_client: &Client,
    k8s_client: &K8sClient,
    agent: &Agent,
    work_order: &WorkOrder,
) -> Result<Option<String>, Box<dyn std::error::Error>> {
    info!(
        "Executing build work order {} for agent {}",
        work_order.id, agent.name
    );

    // Parse the YAML content to extract Build and WorkOrder resources
    let yaml_docs = crate::utils::multidoc_deserialize(&work_order.yaml_content)?;

    if yaml_docs.is_empty() {
        return Err("Work order YAML content is empty".into());
    }

    // Apply all K8s resources from the YAML
    // The YAML should contain Shipwright Build + brokkr WorkOrder CRD
    for doc in &yaml_docs {
        debug!("Applying K8s resource from work order YAML");
        // We'll implement the actual application in the build module
    }

    // Execute the build using the build handler
    let result = build::execute_build(k8s_client, &work_order.yaml_content, &work_order.id.to_string()).await?;

    Ok(result)
}
