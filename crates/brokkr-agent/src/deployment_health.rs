/*
 * Copyright (c) 2025 Dylan Storey
 * Licensed under the Elastic License 2.0.
 * See LICENSE file in the project root for full license text.
 */

//! Deployment Health Checker Module
//!
//! Monitors the health of deployed Kubernetes resources and reports status
//! to the broker. Detects common issues like ImagePullBackOff, CrashLoopBackOff,
//! OOMKilled, and other problematic conditions.

use crate::k8s::objects::DEPLOYMENT_OBJECT_ID_LABEL;
use brokkr_utils::logging::prelude::*;
use chrono::{DateTime, Utc};
use k8s_openapi::api::core::v1::Pod;
use kube::{api::ListParams, Api, Client};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Known problematic waiting conditions that indicate degraded health
const DEGRADED_CONDITIONS: &[&str] = &[
    "ImagePullBackOff",
    "ErrImagePull",
    "CrashLoopBackOff",
    "CreateContainerConfigError",
    "InvalidImageName",
    "RunContainerError",
    "ContainerCannotRun",
];

/// Conditions that indicate pending state (not yet problematic but not ready)
/// Reserved for future use to track long-pending states
#[allow(dead_code)]
const PENDING_CONDITIONS: &[&str] = &[
    "Pending",
    "ContainerCreating",
    "PodInitializing",
];

/// Reasons from terminated containers that indicate issues
const TERMINATED_ISSUES: &[&str] = &[
    "OOMKilled",
    "Error",
    "ContainerCannotRun",
];

/// Health status for a deployment object
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeploymentHealthStatus {
    /// The deployment object ID
    pub id: Uuid,
    /// Overall health status: healthy, degraded, failing, unknown
    pub status: String,
    /// Structured health summary
    pub summary: HealthSummary,
    /// When the health was checked
    pub checked_at: DateTime<Utc>,
}

/// Summary of health information for a deployment
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct HealthSummary {
    /// Number of pods in ready state
    pub pods_ready: usize,
    /// Total number of pods
    pub pods_total: usize,
    /// List of detected problematic conditions
    pub conditions: Vec<String>,
    /// Per-resource health details
    pub resources: Vec<ResourceHealth>,
}

/// Health status of an individual resource
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceHealth {
    /// Kind of the resource (e.g., "Pod", "Deployment")
    pub kind: String,
    /// Name of the resource
    pub name: String,
    /// Namespace of the resource
    pub namespace: String,
    /// Whether the resource is ready
    pub ready: bool,
    /// Human-readable status message
    pub message: Option<String>,
}

/// Checks deployment health for Kubernetes resources
pub struct HealthChecker {
    k8s_client: Client,
}

impl HealthChecker {
    /// Creates a new HealthChecker instance
    pub fn new(k8s_client: Client) -> Self {
        Self { k8s_client }
    }

    /// Checks the health of a specific deployment object by ID
    ///
    /// Finds all pods labeled with the deployment object ID and analyzes
    /// their status to determine overall health.
    pub async fn check_deployment_object(
        &self,
        deployment_object_id: Uuid,
    ) -> Result<DeploymentHealthStatus, Box<dyn std::error::Error + Send + Sync>> {
        let checked_at = Utc::now();

        // Find pods matching this deployment object
        let pods = self.find_pods_for_deployment(deployment_object_id).await?;

        let mut summary = HealthSummary::default();
        let mut overall_status = "healthy";
        let mut conditions_set: std::collections::HashSet<String> = std::collections::HashSet::new();

        summary.pods_total = pods.len();

        for pod in &pods {
            let pod_name = pod.metadata.name.clone().unwrap_or_default();
            let pod_namespace = pod.metadata.namespace.clone().unwrap_or_default();

            // Check if pod is ready
            let pod_ready = is_pod_ready(pod);
            if pod_ready {
                summary.pods_ready += 1;
            }

            // Analyze pod status for issues
            if let Some(pod_status) = &pod.status {
                // Check container statuses for waiting/terminated issues
                if let Some(container_statuses) = &pod_status.container_statuses {
                    for cs in container_statuses {
                        if let Some(state) = &cs.state {
                            // Check waiting state
                            if let Some(waiting) = &state.waiting {
                                if let Some(reason) = &waiting.reason {
                                    if DEGRADED_CONDITIONS.contains(&reason.as_str()) {
                                        conditions_set.insert(reason.clone());
                                        overall_status = "degraded";

                                        summary.resources.push(ResourceHealth {
                                            kind: "Pod".to_string(),
                                            name: pod_name.clone(),
                                            namespace: pod_namespace.clone(),
                                            ready: false,
                                            message: waiting.message.clone(),
                                        });
                                    }
                                }
                            }

                            // Check terminated state for issues
                            if let Some(terminated) = &state.terminated {
                                if let Some(reason) = &terminated.reason {
                                    if TERMINATED_ISSUES.contains(&reason.as_str()) {
                                        conditions_set.insert(reason.clone());
                                        overall_status = "degraded";
                                    }
                                }
                            }
                        }

                        // Check last terminated state for recent crashes
                        if let Some(last_state) = &cs.last_state {
                            if let Some(terminated) = &last_state.terminated {
                                if let Some(reason) = &terminated.reason {
                                    if reason == "OOMKilled" {
                                        conditions_set.insert("OOMKilled".to_string());
                                        overall_status = "degraded";
                                    }
                                }
                            }
                        }
                    }
                }

                // Check init container statuses
                if let Some(init_statuses) = &pod_status.init_container_statuses {
                    for cs in init_statuses {
                        if let Some(state) = &cs.state {
                            if let Some(waiting) = &state.waiting {
                                if let Some(reason) = &waiting.reason {
                                    if DEGRADED_CONDITIONS.contains(&reason.as_str()) {
                                        conditions_set.insert(format!("InitContainer:{}", reason));
                                        overall_status = "degraded";
                                    }
                                }
                            }
                        }
                    }
                }

                // Check pod phase
                if let Some(phase) = &pod_status.phase {
                    match phase.as_str() {
                        "Failed" => {
                            overall_status = "failing";
                            conditions_set.insert("PodFailed".to_string());
                        }
                        "Unknown" => {
                            if overall_status != "failing" && overall_status != "degraded" {
                                overall_status = "unknown";
                            }
                        }
                        "Pending" => {
                            // Check if pending for too long might indicate an issue
                            // For now, we just note it's pending
                            if overall_status == "healthy" {
                                // Could add logic to check if pending too long
                            }
                        }
                        _ => {}
                    }
                }
            }
        }

        summary.conditions = conditions_set.into_iter().collect();

        // If no pods found and we expected some, mark as unknown
        if summary.pods_total == 0 {
            overall_status = "unknown";
        }

        Ok(DeploymentHealthStatus {
            id: deployment_object_id,
            status: overall_status.to_string(),
            summary,
            checked_at,
        })
    }

    /// Finds all pods labeled with the given deployment object ID
    async fn find_pods_for_deployment(
        &self,
        deployment_object_id: Uuid,
    ) -> Result<Vec<Pod>, Box<dyn std::error::Error + Send + Sync>> {
        // Query pods across all namespaces with the deployment object label
        let pods_api: Api<Pod> = Api::all(self.k8s_client.clone());

        let label_selector = format!("{}={}", DEPLOYMENT_OBJECT_ID_LABEL, deployment_object_id);
        let lp = ListParams::default().labels(&label_selector);

        let pod_list = pods_api.list(&lp).await?;
        Ok(pod_list.items)
    }

    /// Checks health for multiple deployment objects
    pub async fn check_deployment_objects(
        &self,
        deployment_object_ids: &[Uuid],
    ) -> Vec<DeploymentHealthStatus> {
        let mut results = Vec::new();

        for &id in deployment_object_ids {
            match self.check_deployment_object(id).await {
                Ok(status) => results.push(status),
                Err(e) => {
                    warn!("Failed to check health for deployment object {}: {}", id, e);
                    // Report as unknown on error
                    results.push(DeploymentHealthStatus {
                        id,
                        status: "unknown".to_string(),
                        summary: HealthSummary::default(),
                        checked_at: Utc::now(),
                    });
                }
            }
        }

        results
    }
}

/// Checks if a pod is in ready state
fn is_pod_ready(pod: &Pod) -> bool {
    pod.status
        .as_ref()
        .and_then(|s| s.conditions.as_ref())
        .map(|conditions| {
            conditions
                .iter()
                .any(|c| c.type_ == "Ready" && c.status == "True")
        })
        .unwrap_or(false)
}

/// Request body for sending health status updates to the broker
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthStatusUpdate {
    /// List of deployment object health updates
    pub deployment_objects: Vec<DeploymentObjectHealthUpdate>,
}

/// Health update for a single deployment object (matches broker API)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeploymentObjectHealthUpdate {
    /// The deployment object ID
    pub id: Uuid,
    /// Health status: healthy, degraded, failing, or unknown
    pub status: String,
    /// Structured health summary
    pub summary: Option<HealthSummary>,
    /// When the health was checked
    pub checked_at: DateTime<Utc>,
}

impl From<DeploymentHealthStatus> for DeploymentObjectHealthUpdate {
    fn from(status: DeploymentHealthStatus) -> Self {
        Self {
            id: status.id,
            status: status.status,
            summary: Some(status.summary),
            checked_at: status.checked_at,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_degraded_conditions_are_detected() {
        // Verify all expected degraded conditions are in our list
        assert!(DEGRADED_CONDITIONS.contains(&"ImagePullBackOff"));
        assert!(DEGRADED_CONDITIONS.contains(&"CrashLoopBackOff"));
        assert!(DEGRADED_CONDITIONS.contains(&"CreateContainerConfigError"));
        assert!(DEGRADED_CONDITIONS.contains(&"ErrImagePull"));
    }

    #[test]
    fn test_terminated_issues_include_oomkilled() {
        assert!(TERMINATED_ISSUES.contains(&"OOMKilled"));
        assert!(TERMINATED_ISSUES.contains(&"Error"));
    }

    #[test]
    fn test_health_summary_default() {
        let summary = HealthSummary::default();
        assert_eq!(summary.pods_ready, 0);
        assert_eq!(summary.pods_total, 0);
        assert!(summary.conditions.is_empty());
        assert!(summary.resources.is_empty());
    }

    #[test]
    fn test_deployment_health_status_serialization() {
        let status = DeploymentHealthStatus {
            id: Uuid::new_v4(),
            status: "healthy".to_string(),
            summary: HealthSummary {
                pods_ready: 3,
                pods_total: 3,
                conditions: vec![],
                resources: vec![],
            },
            checked_at: Utc::now(),
        };

        let json = serde_json::to_string(&status).unwrap();
        let deserialized: DeploymentHealthStatus = serde_json::from_str(&json).unwrap();

        assert_eq!(status.id, deserialized.id);
        assert_eq!(status.status, deserialized.status);
        assert_eq!(status.summary.pods_ready, deserialized.summary.pods_ready);
    }

    #[test]
    fn test_health_update_conversion() {
        let status = DeploymentHealthStatus {
            id: Uuid::new_v4(),
            status: "degraded".to_string(),
            summary: HealthSummary {
                pods_ready: 1,
                pods_total: 3,
                conditions: vec!["ImagePullBackOff".to_string()],
                resources: vec![],
            },
            checked_at: Utc::now(),
        };

        let update: DeploymentObjectHealthUpdate = status.clone().into();

        assert_eq!(update.id, status.id);
        assert_eq!(update.status, status.status);
        assert!(update.summary.is_some());
    }
}
