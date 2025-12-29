/*
 * Copyright (c) 2025 Dylan Storey
 * Licensed under the Elastic License 2.0.
 * See LICENSE file in the project root for full license text.
 */

//! Build handler for Shipwright Build integration.
//!
//! This module handles the execution of build work orders using Shipwright:
//! - Parsing Build and WorkOrder resources from YAML
//! - Applying Build resources to the cluster
//! - Creating BuildRun resources
//! - Watching BuildRun status until completion
//! - Extracting results (image digest, errors)
//!
//! ## Shipwright Resources
//!
//! - **Build**: Reusable template defining git source, build strategy, and output
//! - **BuildRun**: Execution instance created from a Build
//! - **ClusterBuildStrategy**: Pre-installed strategy (e.g., buildah)

use crate::k8s;
use tracing::{debug, error, info, trace, warn};
use kube::{
    api::{Api, DynamicObject, PatchParams, PostParams},
    Client as K8sClient, Discovery,
};
use serde::Deserialize;
use serde_json::json;
use std::time::Duration;
use tokio::time::sleep;

/// Shipwright API group
const SHIPWRIGHT_API_GROUP: &str = "shipwright.io";
const SHIPWRIGHT_API_VERSION: &str = "v1beta1";

/// BuildRun status conditions
const CONDITION_SUCCEEDED: &str = "Succeeded";

/// Maximum time to wait for a build to complete (15 minutes)
const BUILD_TIMEOUT_SECS: u64 = 900;

/// Polling interval for build status checks
const STATUS_POLL_INTERVAL_SECS: u64 = 5;

/// BuildRun status for watching completion
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct BuildRunStatus {
    #[serde(default)]
    conditions: Vec<Condition>,
    #[serde(skip_serializing_if = "Option::is_none")]
    output: Option<BuildRunOutput>,
    #[serde(skip_serializing_if = "Option::is_none")]
    failure_details: Option<FailureDetails>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Condition {
    #[serde(rename = "type")]
    condition_type: String,
    status: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    reason: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    message: Option<String>,
}

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct BuildRunOutput {
    digest: Option<String>,
    size: Option<i64>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct FailureDetails {
    #[serde(skip_serializing_if = "Option::is_none")]
    reason: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    message: Option<String>,
}

/// Executes a build using Shipwright.
///
/// This function:
/// 1. Parses the YAML content to find Build resources
/// 2. Applies Build resources to the cluster
/// 3. Creates a BuildRun
/// 4. Watches the BuildRun until completion
/// 5. Returns the image digest on success or error details on failure
///
/// # Arguments
/// * `k8s_client` - Kubernetes client
/// * `yaml_content` - Multi-document YAML containing Build and WorkOrder
/// * `work_order_id` - Work order ID for labeling resources
///
/// # Returns
/// Optional result message (image digest on success)
pub async fn execute_build(
    k8s_client: &K8sClient,
    yaml_content: &str,
    work_order_id: &str,
) -> Result<Option<String>, Box<dyn std::error::Error>> {
    info!("Starting build execution for work order {}", work_order_id);

    // Parse YAML documents
    let docs: Vec<serde_yaml::Value> = serde_yaml::Deserializer::from_str(yaml_content)
        .map(|doc| serde_yaml::Value::deserialize(doc))
        .collect::<Result<Vec<_>, _>>()?;

    if docs.is_empty() {
        return Err("No YAML documents found in work order content".into());
    }

    // Find Build resources and apply them
    let mut build_name: Option<String> = None;
    let mut build_namespace = String::from("default");

    for doc in &docs {
        let api_version = doc.get("apiVersion").and_then(|v| v.as_str());
        let kind = doc.get("kind").and_then(|v| v.as_str());

        match (api_version, kind) {
            (Some(av), Some("Build")) if av.starts_with(SHIPWRIGHT_API_GROUP) => {
                // Apply the Build resource
                let metadata = doc.get("metadata");
                if let Some(name) = metadata.and_then(|m| m.get("name")).and_then(|n| n.as_str()) {
                    build_name = Some(name.to_string());
                }
                if let Some(ns) = metadata.and_then(|m| m.get("namespace")).and_then(|n| n.as_str()) {
                    build_namespace = ns.to_string();
                }

                info!("Applying Shipwright Build resource");
                apply_shipwright_resource(k8s_client, doc).await?;
            }
            (Some(av), Some("WorkOrder")) if av.starts_with("brokkr.io") => {
                // Extract buildRef from WorkOrder if present
                if let Some(spec) = doc.get("spec") {
                    if let Some(build_ref) = spec.get("buildRef").and_then(|b| b.get("name")).and_then(|n| n.as_str()) {
                        if build_name.is_none() {
                            build_name = Some(build_ref.to_string());
                        }
                    }
                }
                debug!("Found brokkr WorkOrder resource, skipping apply (handled separately)");
            }
            _ => {
                debug!("Skipping non-build resource: {:?}/{:?}", api_version, kind);
            }
        }
    }

    let build_name = build_name.ok_or("No Build resource or buildRef found in YAML content")?;
    info!("Using Build '{}' in namespace '{}'", build_name, build_namespace);

    // Create BuildRun
    let buildrun_name = format!("{}-{}", build_name, &work_order_id[..8.min(work_order_id.len())]);
    info!("Creating BuildRun '{}'", buildrun_name);

    let buildrun = create_buildrun(
        k8s_client,
        &buildrun_name,
        &build_name,
        &build_namespace,
        work_order_id,
    )
    .await?;

    info!("BuildRun '{}' created, waiting for completion", buildrun_name);

    // Watch BuildRun until completion
    let result = watch_buildrun_completion(k8s_client, &buildrun_name, &build_namespace).await?;

    Ok(result)
}

/// Applies a Shipwright resource (Build) to the cluster using the core k8s apply logic.
async fn apply_shipwright_resource(
    k8s_client: &K8sClient,
    resource: &serde_yaml::Value,
) -> Result<(), Box<dyn std::error::Error>> {
    // Convert YAML to DynamicObject
    let k8s_object: DynamicObject = serde_yaml::from_value(resource.clone())?;

    // Use the existing apply_k8s_objects function which has proper retry logic
    let patch_params = PatchParams::apply("brokkr-agent").force();
    k8s::api::apply_k8s_objects(&[k8s_object], k8s_client.clone(), patch_params).await
}

/// Creates a BuildRun resource.
async fn create_buildrun(
    k8s_client: &K8sClient,
    name: &str,
    build_name: &str,
    namespace: &str,
    work_order_id: &str,
) -> Result<DynamicObject, Box<dyn std::error::Error>> {
    // Discover the BuildRun API
    let discovery = Discovery::new(k8s_client.clone()).run().await?;

    let ar = discovery
        .groups()
        .flat_map(|g| g.recommended_resources())
        .find(|(ar, _)| ar.group == SHIPWRIGHT_API_GROUP && ar.kind == "BuildRun")
        .map(|(ar, _)| ar)
        .ok_or("Shipwright BuildRun CRD not found in cluster")?;

    let api: Api<DynamicObject> = Api::namespaced_with(k8s_client.clone(), namespace, &ar);

    let buildrun_data = json!({
        "apiVersion": format!("{}/{}", SHIPWRIGHT_API_GROUP, SHIPWRIGHT_API_VERSION),
        "kind": "BuildRun",
        "metadata": {
            "name": name,
            "namespace": namespace,
            "labels": {
                "brokkr.io/work-order-id": work_order_id,
                "shipwright.io/build": build_name
            }
        },
        "spec": {
            "build": {
                "name": build_name
            }
        }
    });

    let buildrun: DynamicObject = serde_json::from_value(buildrun_data)?;

    let result = api.create(&PostParams::default(), &buildrun).await?;

    Ok(result)
}

/// Watches a BuildRun until it completes (success or failure).
async fn watch_buildrun_completion(
    k8s_client: &K8sClient,
    name: &str,
    namespace: &str,
) -> Result<Option<String>, Box<dyn std::error::Error>> {
    // Discover the BuildRun API
    let discovery = Discovery::new(k8s_client.clone()).run().await?;

    let ar = discovery
        .groups()
        .flat_map(|g| g.recommended_resources())
        .find(|(ar, _)| ar.group == SHIPWRIGHT_API_GROUP && ar.kind == "BuildRun")
        .map(|(ar, _)| ar)
        .ok_or("Shipwright BuildRun CRD not found in cluster")?;

    let api: Api<DynamicObject> = Api::namespaced_with(k8s_client.clone(), namespace, &ar);

    let start_time = std::time::Instant::now();
    let timeout = Duration::from_secs(BUILD_TIMEOUT_SECS);

    loop {
        if start_time.elapsed() > timeout {
            return Err(format!(
                "BuildRun '{}' timed out after {} seconds",
                name, BUILD_TIMEOUT_SECS
            )
            .into());
        }

        let buildrun = api.get(name).await?;

        // Check status
        if let Some(status_value) = buildrun.data.get("status") {
            let status: BuildRunStatus = serde_json::from_value(status_value.clone())?;

            // Find the Succeeded condition
            for condition in &status.conditions {
                if condition.condition_type == CONDITION_SUCCEEDED {
                    match condition.status.as_str() {
                        "True" => {
                            // Build succeeded
                            let digest = status
                                .output
                                .as_ref()
                                .and_then(|o| o.digest.clone());

                            info!(
                                "BuildRun '{}' completed successfully. Digest: {:?}",
                                name, digest
                            );

                            return Ok(digest);
                        }
                        "False" => {
                            // Build failed
                            let error_msg = status
                                .failure_details
                                .as_ref()
                                .map(|f| {
                                    format!(
                                        "{}: {}",
                                        f.reason.as_deref().unwrap_or("Unknown"),
                                        f.message.as_deref().unwrap_or("No message")
                                    )
                                })
                                .or_else(|| condition.message.clone())
                                .unwrap_or_else(|| "Build failed".to_string());

                            error!("BuildRun '{}' failed: {}", name, error_msg);
                            return Err(error_msg.into());
                        }
                        _ => {
                            // Still running
                            debug!(
                                "BuildRun '{}' still in progress: {:?}",
                                name,
                                condition.reason
                            );
                        }
                    }
                }
            }
        }

        // Wait before next check
        sleep(Duration::from_secs(STATUS_POLL_INTERVAL_SECS)).await;
    }
}
