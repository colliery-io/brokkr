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
pub(crate) struct BuildRunStatus {
    #[serde(default)]
    pub conditions: Vec<Condition>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub output: Option<BuildRunOutput>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub failure_details: Option<FailureDetails>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct Condition {
    #[serde(rename = "type")]
    pub condition_type: String,
    pub status: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
}

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct BuildRunOutput {
    pub digest: Option<String>,
    pub size: Option<i64>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct FailureDetails {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
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

/// Result of parsing build YAML content
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct ParsedBuildInfo {
    pub build_name: String,
    pub build_namespace: String,
    pub build_docs: Vec<serde_yaml::Value>,
}

/// Parses YAML content to extract Build resource information.
///
/// This function finds Shipwright Build resources in the YAML and extracts:
/// - The build name (from Build metadata or WorkOrder buildRef)
/// - The namespace (defaulting to "default")
/// - The Build documents to be applied
///
/// # Arguments
/// * `yaml_content` - Multi-document YAML string
///
/// # Returns
/// ParsedBuildInfo with extracted build details
pub(crate) fn parse_build_yaml(yaml_content: &str) -> Result<ParsedBuildInfo, Box<dyn std::error::Error>> {
    let docs: Vec<serde_yaml::Value> = serde_yaml::Deserializer::from_str(yaml_content)
        .map(|doc| serde_yaml::Value::deserialize(doc))
        .collect::<Result<Vec<_>, _>>()?;

    if docs.is_empty() {
        return Err("No YAML documents found in work order content".into());
    }

    let mut build_name: Option<String> = None;
    let mut build_namespace = String::from("default");
    let mut build_docs = Vec::new();

    for doc in &docs {
        let api_version = doc.get("apiVersion").and_then(|v| v.as_str());
        let kind = doc.get("kind").and_then(|v| v.as_str());

        match (api_version, kind) {
            (Some(av), Some("Build")) if av.starts_with(SHIPWRIGHT_API_GROUP) => {
                let metadata = doc.get("metadata");
                if let Some(name) = metadata.and_then(|m| m.get("name")).and_then(|n| n.as_str()) {
                    build_name = Some(name.to_string());
                }
                if let Some(ns) = metadata.and_then(|m| m.get("namespace")).and_then(|n| n.as_str()) {
                    build_namespace = ns.to_string();
                }
                build_docs.push(doc.clone());
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
            }
            _ => {
                // Skip non-build resources
            }
        }
    }

    let build_name = build_name.ok_or("No Build resource or buildRef found in YAML content")?;

    Ok(ParsedBuildInfo {
        build_name,
        build_namespace,
        build_docs,
    })
}

/// Interprets a BuildRun status to determine completion state.
///
/// # Returns
/// - `Ok(Some(digest))` if the build succeeded
/// - `Err(message)` if the build failed
/// - `Ok(None)` if the build is still in progress
pub(crate) fn interpret_buildrun_status(status: &BuildRunStatus) -> Result<Option<String>, String> {
    for condition in &status.conditions {
        if condition.condition_type == CONDITION_SUCCEEDED {
            match condition.status.as_str() {
                "True" => {
                    // Build succeeded - extract digest
                    let digest = status.output.as_ref().and_then(|o| o.digest.clone());
                    return Ok(digest);
                }
                "False" => {
                    // Build failed - extract error message
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
                    return Err(error_msg);
                }
                _ => {
                    // Still running (Unknown status)
                    return Ok(None);
                }
            }
        }
    }

    // No Succeeded condition found yet - still initializing
    Ok(None)
}

#[cfg(test)]
mod tests {
    use super::*;

    // ==================== YAML Parsing Tests ====================

    #[test]
    fn test_parse_build_yaml_with_build_resource() {
        let yaml = r#"
---
apiVersion: shipwright.io/v1beta1
kind: Build
metadata:
  name: my-build
  namespace: my-namespace
spec:
  source:
    git:
      url: https://github.com/example/repo
  strategy:
    name: buildah
    kind: ClusterBuildStrategy
  output:
    image: registry.example.com/my-image:latest
"#;
        let result = parse_build_yaml(yaml).unwrap();
        assert_eq!(result.build_name, "my-build");
        assert_eq!(result.build_namespace, "my-namespace");
        assert_eq!(result.build_docs.len(), 1);
    }

    #[test]
    fn test_parse_build_yaml_default_namespace() {
        let yaml = r#"
---
apiVersion: shipwright.io/v1beta1
kind: Build
metadata:
  name: my-build
spec:
  source:
    git:
      url: https://github.com/example/repo
"#;
        let result = parse_build_yaml(yaml).unwrap();
        assert_eq!(result.build_name, "my-build");
        assert_eq!(result.build_namespace, "default");
    }

    #[test]
    fn test_parse_build_yaml_with_work_order_buildref() {
        let yaml = r#"
---
apiVersion: brokkr.io/v1
kind: WorkOrder
metadata:
  name: my-work-order
spec:
  buildRef:
    name: referenced-build
"#;
        let result = parse_build_yaml(yaml).unwrap();
        assert_eq!(result.build_name, "referenced-build");
        assert_eq!(result.build_namespace, "default");
        assert_eq!(result.build_docs.len(), 0); // No Build resource to apply
    }

    #[test]
    fn test_parse_build_yaml_build_takes_precedence() {
        // When both Build and WorkOrder are present, Build name takes precedence
        let yaml = r#"
---
apiVersion: shipwright.io/v1beta1
kind: Build
metadata:
  name: actual-build
  namespace: build-ns
spec:
  source:
    git:
      url: https://github.com/example/repo
---
apiVersion: brokkr.io/v1
kind: WorkOrder
metadata:
  name: my-work-order
spec:
  buildRef:
    name: different-build
"#;
        let result = parse_build_yaml(yaml).unwrap();
        assert_eq!(result.build_name, "actual-build");
        assert_eq!(result.build_namespace, "build-ns");
        assert_eq!(result.build_docs.len(), 1);
    }

    #[test]
    fn test_parse_build_yaml_empty_content() {
        let yaml = "";
        let result = parse_build_yaml(yaml);
        assert!(result.is_err());
        // Empty string parses as no documents, which means no Build resource found
        assert!(result.unwrap_err().to_string().contains("No Build resource"));
    }

    #[test]
    fn test_parse_build_yaml_no_build_resource() {
        let yaml = r#"
---
apiVersion: v1
kind: ConfigMap
metadata:
  name: some-config
data:
  key: value
"#;
        let result = parse_build_yaml(yaml);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("No Build resource"));
    }

    #[test]
    fn test_parse_build_yaml_invalid_yaml() {
        let yaml = "this is: [not valid: yaml";
        let result = parse_build_yaml(yaml);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_build_yaml_multiple_builds() {
        // Last Build wins for the name (each overwrites the previous)
        let yaml = r#"
---
apiVersion: shipwright.io/v1beta1
kind: Build
metadata:
  name: first-build
spec: {}
---
apiVersion: shipwright.io/v1beta1
kind: Build
metadata:
  name: second-build
spec: {}
"#;
        let result = parse_build_yaml(yaml).unwrap();
        assert_eq!(result.build_name, "second-build");
        assert_eq!(result.build_docs.len(), 2);
    }

    // ==================== BuildRunStatus Deserialization Tests ====================

    #[test]
    fn test_buildrun_status_deserialization_success() {
        let json = serde_json::json!({
            "conditions": [
                {
                    "type": "Succeeded",
                    "status": "True",
                    "reason": "BuildSucceeded",
                    "message": "Build completed successfully"
                }
            ],
            "output": {
                "digest": "sha256:abc123def456",
                "size": 12345678
            }
        });
        let status: BuildRunStatus = serde_json::from_value(json).unwrap();
        assert_eq!(status.conditions.len(), 1);
        assert_eq!(status.conditions[0].condition_type, "Succeeded");
        assert_eq!(status.conditions[0].status, "True");
        assert_eq!(status.output.unwrap().digest, Some("sha256:abc123def456".to_string()));
    }

    #[test]
    fn test_buildrun_status_deserialization_failure() {
        let json = serde_json::json!({
            "conditions": [
                {
                    "type": "Succeeded",
                    "status": "False",
                    "reason": "BuildFailed",
                    "message": "Container build failed"
                }
            ],
            "failureDetails": {
                "reason": "BuildContainerFailed",
                "message": "Buildah exited with code 1"
            }
        });
        let status: BuildRunStatus = serde_json::from_value(json).unwrap();
        assert_eq!(status.conditions[0].status, "False");
        let failure = status.failure_details.unwrap();
        assert_eq!(failure.reason, Some("BuildContainerFailed".to_string()));
    }

    #[test]
    fn test_buildrun_status_deserialization_in_progress() {
        let json = serde_json::json!({
            "conditions": [
                {
                    "type": "Succeeded",
                    "status": "Unknown",
                    "reason": "Running",
                    "message": "Build is in progress"
                }
            ]
        });
        let status: BuildRunStatus = serde_json::from_value(json).unwrap();
        assert_eq!(status.conditions[0].status, "Unknown");
        assert!(status.output.is_none());
    }

    #[test]
    fn test_buildrun_status_deserialization_empty_conditions() {
        let json = serde_json::json!({});
        let status: BuildRunStatus = serde_json::from_value(json).unwrap();
        assert!(status.conditions.is_empty());
        assert!(status.output.is_none());
        assert!(status.failure_details.is_none());
    }

    // ==================== Status Interpretation Tests ====================

    #[test]
    fn test_interpret_status_succeeded_with_digest() {
        let status = BuildRunStatus {
            conditions: vec![Condition {
                condition_type: "Succeeded".to_string(),
                status: "True".to_string(),
                reason: Some("BuildSucceeded".to_string()),
                message: None,
            }],
            output: Some(BuildRunOutput {
                digest: Some("sha256:abc123".to_string()),
                size: Some(1000),
            }),
            failure_details: None,
        };

        let result = interpret_buildrun_status(&status);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Some("sha256:abc123".to_string()));
    }

    #[test]
    fn test_interpret_status_succeeded_no_digest() {
        let status = BuildRunStatus {
            conditions: vec![Condition {
                condition_type: "Succeeded".to_string(),
                status: "True".to_string(),
                reason: None,
                message: None,
            }],
            output: None,
            failure_details: None,
        };

        let result = interpret_buildrun_status(&status);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), None);
    }

    #[test]
    fn test_interpret_status_failed_with_details() {
        let status = BuildRunStatus {
            conditions: vec![Condition {
                condition_type: "Succeeded".to_string(),
                status: "False".to_string(),
                reason: Some("BuildFailed".to_string()),
                message: Some("Container exited with error".to_string()),
            }],
            output: None,
            failure_details: Some(FailureDetails {
                reason: Some("ContainerFailed".to_string()),
                message: Some("Exit code 1".to_string()),
            }),
        };

        let result = interpret_buildrun_status(&status);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.contains("ContainerFailed"));
        assert!(err.contains("Exit code 1"));
    }

    #[test]
    fn test_interpret_status_failed_no_details() {
        let status = BuildRunStatus {
            conditions: vec![Condition {
                condition_type: "Succeeded".to_string(),
                status: "False".to_string(),
                reason: None,
                message: Some("Generic failure".to_string()),
            }],
            output: None,
            failure_details: None,
        };

        let result = interpret_buildrun_status(&status);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Generic failure");
    }

    #[test]
    fn test_interpret_status_failed_fallback_message() {
        let status = BuildRunStatus {
            conditions: vec![Condition {
                condition_type: "Succeeded".to_string(),
                status: "False".to_string(),
                reason: None,
                message: None,
            }],
            output: None,
            failure_details: None,
        };

        let result = interpret_buildrun_status(&status);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Build failed");
    }

    #[test]
    fn test_interpret_status_in_progress() {
        let status = BuildRunStatus {
            conditions: vec![Condition {
                condition_type: "Succeeded".to_string(),
                status: "Unknown".to_string(),
                reason: Some("Running".to_string()),
                message: None,
            }],
            output: None,
            failure_details: None,
        };

        let result = interpret_buildrun_status(&status);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), None);
    }

    #[test]
    fn test_interpret_status_no_succeeded_condition() {
        let status = BuildRunStatus {
            conditions: vec![Condition {
                condition_type: "Ready".to_string(),
                status: "True".to_string(),
                reason: None,
                message: None,
            }],
            output: None,
            failure_details: None,
        };

        let result = interpret_buildrun_status(&status);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), None); // Still initializing
    }

    #[test]
    fn test_interpret_status_empty_conditions() {
        let status = BuildRunStatus {
            conditions: vec![],
            output: None,
            failure_details: None,
        };

        let result = interpret_buildrun_status(&status);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), None); // Still initializing
    }

    // ==================== BuildRun Name Generation Tests ====================

    #[test]
    fn test_buildrun_name_generation_short_id() {
        let build_name = "my-build";
        let work_order_id = "abc123";
        let buildrun_name = format!("{}-{}", build_name, &work_order_id[..8.min(work_order_id.len())]);
        assert_eq!(buildrun_name, "my-build-abc123");
    }

    #[test]
    fn test_buildrun_name_generation_long_id() {
        let build_name = "my-build";
        let work_order_id = "12345678-abcd-efgh-ijkl-mnopqrstuvwx";
        let buildrun_name = format!("{}-{}", build_name, &work_order_id[..8.min(work_order_id.len())]);
        assert_eq!(buildrun_name, "my-build-12345678");
    }
}
