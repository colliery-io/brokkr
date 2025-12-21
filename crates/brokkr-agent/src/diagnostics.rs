/*
 * Copyright (c) 2025 Dylan Storey
 * Licensed under the Elastic License 2.0.
 * See LICENSE file in the project root for full license text.
 */

//! Diagnostics handler for on-demand diagnostic collection.
//!
//! This module provides functionality to collect detailed diagnostic information
//! about Kubernetes resources, including pod statuses, events, and log tails.

use brokkr_utils::logging::prelude::*;
use chrono::{DateTime, Utc};
use k8s_openapi::api::core::v1::{Event, Pod};
use kube::{
    api::{Api, ListParams},
    Client,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

/// Maximum number of log lines to collect per container.
const MAX_LOG_LINES: i64 = 100;

/// Diagnostic request received from the broker.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiagnosticRequest {
    /// Unique identifier for the diagnostic request.
    pub id: Uuid,
    /// The agent that should handle this request.
    pub agent_id: Uuid,
    /// The deployment object to gather diagnostics for.
    pub deployment_object_id: Uuid,
    /// Status: pending, claimed, completed, failed, expired.
    pub status: String,
    /// Who requested the diagnostics.
    pub requested_by: Option<String>,
    /// When the request was created.
    pub created_at: DateTime<Utc>,
    /// When the agent claimed the request.
    pub claimed_at: Option<DateTime<Utc>>,
    /// When the request was completed.
    pub completed_at: Option<DateTime<Utc>>,
    /// When the request expires.
    pub expires_at: DateTime<Utc>,
}

/// Result to submit back to the broker.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubmitDiagnosticResult {
    /// JSON-encoded pod statuses.
    pub pod_statuses: String,
    /// JSON-encoded Kubernetes events.
    pub events: String,
    /// JSON-encoded log tails (optional).
    pub log_tails: Option<String>,
    /// When the diagnostics were collected.
    pub collected_at: DateTime<Utc>,
}

/// Pod status information for diagnostics.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PodStatus {
    /// Pod name.
    pub name: String,
    /// Pod namespace.
    pub namespace: String,
    /// Pod phase (Pending, Running, Succeeded, Failed, Unknown).
    pub phase: String,
    /// Pod conditions.
    pub conditions: Vec<PodCondition>,
    /// Container statuses.
    pub containers: Vec<ContainerStatus>,
}

/// Pod condition information.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PodCondition {
    /// Condition type.
    pub condition_type: String,
    /// Condition status (True, False, Unknown).
    pub status: String,
    /// Reason for the condition.
    pub reason: Option<String>,
    /// Human-readable message.
    pub message: Option<String>,
}

/// Container status information.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContainerStatus {
    /// Container name.
    pub name: String,
    /// Whether the container is ready.
    pub ready: bool,
    /// Number of restarts.
    pub restart_count: i32,
    /// Current state of the container.
    pub state: String,
    /// Reason for current state.
    pub state_reason: Option<String>,
    /// Message for current state.
    pub state_message: Option<String>,
}

/// Kubernetes event information.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventInfo {
    /// Event type (Normal, Warning).
    pub event_type: Option<String>,
    /// Event reason.
    pub reason: Option<String>,
    /// Event message.
    pub message: Option<String>,
    /// Object involved.
    pub involved_object: String,
    /// First timestamp.
    pub first_timestamp: Option<DateTime<Utc>>,
    /// Last timestamp.
    pub last_timestamp: Option<DateTime<Utc>>,
    /// Event count.
    pub count: Option<i32>,
}

/// Diagnostics handler for collecting Kubernetes diagnostics.
pub struct DiagnosticsHandler {
    /// Kubernetes client.
    client: Client,
}

impl DiagnosticsHandler {
    /// Creates a new DiagnosticsHandler.
    pub fn new(client: Client) -> Self {
        Self { client }
    }

    /// Collects diagnostics for resources matching the given labels in the namespace.
    ///
    /// # Arguments
    /// * `namespace` - The Kubernetes namespace
    /// * `label_selector` - Label selector to find the resources
    ///
    /// # Returns
    /// A SubmitDiagnosticResult containing collected diagnostics
    pub async fn collect_diagnostics(
        &self,
        namespace: &str,
        label_selector: &str,
    ) -> Result<SubmitDiagnosticResult, Box<dyn std::error::Error + Send + Sync>> {
        info!(
            "Collecting diagnostics for namespace={}, labels={}",
            namespace, label_selector
        );

        // Collect pod statuses
        let pod_statuses = self.collect_pod_statuses(namespace, label_selector).await?;

        // Collect events
        let events = self.collect_events(namespace, label_selector).await?;

        // Collect log tails
        let log_tails = self.collect_log_tails(namespace, label_selector).await.ok();

        Ok(SubmitDiagnosticResult {
            pod_statuses: serde_json::to_string(&pod_statuses)?,
            events: serde_json::to_string(&events)?,
            log_tails: log_tails.map(|l| serde_json::to_string(&l)).transpose()?,
            collected_at: Utc::now(),
        })
    }

    /// Collects pod statuses for matching pods.
    async fn collect_pod_statuses(
        &self,
        namespace: &str,
        label_selector: &str,
    ) -> Result<Vec<PodStatus>, Box<dyn std::error::Error + Send + Sync>> {
        let pods: Api<Pod> = Api::namespaced(self.client.clone(), namespace);
        let lp = ListParams::default().labels(label_selector);

        let pod_list = pods.list(&lp).await?;
        let mut statuses = Vec::new();

        for pod in pod_list.items {
            let name = pod.metadata.name.clone().unwrap_or_default();
            let pod_namespace = pod.metadata.namespace.clone().unwrap_or_default();

            let status = if let Some(status) = &pod.status {
                let phase = status.phase.clone().unwrap_or_else(|| "Unknown".to_string());

                let conditions: Vec<PodCondition> = status
                    .conditions
                    .as_ref()
                    .map(|conds| {
                        conds
                            .iter()
                            .map(|c| PodCondition {
                                condition_type: c.type_.clone(),
                                status: c.status.clone(),
                                reason: c.reason.clone(),
                                message: c.message.clone(),
                            })
                            .collect()
                    })
                    .unwrap_or_default();

                let containers: Vec<ContainerStatus> = status
                    .container_statuses
                    .as_ref()
                    .map(|cs| {
                        cs.iter()
                            .map(|c| {
                                let (state, state_reason, state_message) =
                                    if let Some(state) = &c.state {
                                        if let Some(running) = &state.running {
                                            (
                                                "Running".to_string(),
                                                None,
                                                running
                                                    .started_at
                                                    .as_ref()
                                                    .map(|t| format!("Started at {}", t.0)),
                                            )
                                        } else if let Some(waiting) = &state.waiting {
                                            (
                                                "Waiting".to_string(),
                                                waiting.reason.clone(),
                                                waiting.message.clone(),
                                            )
                                        } else if let Some(terminated) = &state.terminated {
                                            (
                                                "Terminated".to_string(),
                                                terminated.reason.clone(),
                                                terminated.message.clone(),
                                            )
                                        } else {
                                            ("Unknown".to_string(), None, None)
                                        }
                                    } else {
                                        ("Unknown".to_string(), None, None)
                                    };

                                ContainerStatus {
                                    name: c.name.clone(),
                                    ready: c.ready,
                                    restart_count: c.restart_count,
                                    state,
                                    state_reason,
                                    state_message,
                                }
                            })
                            .collect()
                    })
                    .unwrap_or_default();

                PodStatus {
                    name,
                    namespace: pod_namespace,
                    phase,
                    conditions,
                    containers,
                }
            } else {
                PodStatus {
                    name,
                    namespace: pod_namespace,
                    phase: "Unknown".to_string(),
                    conditions: vec![],
                    containers: vec![],
                }
            };

            statuses.push(status);
        }

        debug!("Collected status for {} pods", statuses.len());
        Ok(statuses)
    }

    /// Collects events for matching resources.
    async fn collect_events(
        &self,
        namespace: &str,
        _label_selector: &str,
    ) -> Result<Vec<EventInfo>, Box<dyn std::error::Error + Send + Sync>> {
        let events: Api<Event> = Api::namespaced(self.client.clone(), namespace);
        let lp = ListParams::default();

        let event_list = events.list(&lp).await?;
        let mut event_infos = Vec::new();

        for event in event_list.items {
            let involved_object = event
                .involved_object
                .name
                .clone()
                .unwrap_or_else(|| "unknown".to_string());

            event_infos.push(EventInfo {
                event_type: event.type_.clone(),
                reason: event.reason.clone(),
                message: event.message.clone(),
                involved_object,
                first_timestamp: event.first_timestamp.map(|t| t.0),
                last_timestamp: event.last_timestamp.map(|t| t.0),
                count: event.count,
            });
        }

        // Sort by last_timestamp descending and take recent events
        event_infos.sort_by(|a, b| {
            b.last_timestamp
                .unwrap_or(DateTime::<Utc>::MIN_UTC)
                .cmp(&a.last_timestamp.unwrap_or(DateTime::<Utc>::MIN_UTC))
        });
        event_infos.truncate(50);

        debug!("Collected {} events", event_infos.len());
        Ok(event_infos)
    }

    /// Collects log tails for matching pods.
    async fn collect_log_tails(
        &self,
        namespace: &str,
        label_selector: &str,
    ) -> Result<HashMap<String, String>, Box<dyn std::error::Error + Send + Sync>> {
        let pods: Api<Pod> = Api::namespaced(self.client.clone(), namespace);
        let lp = ListParams::default().labels(label_selector);

        let pod_list = pods.list(&lp).await?;
        let mut log_tails = HashMap::new();

        for pod in pod_list.items {
            let pod_name = pod.metadata.name.clone().unwrap_or_default();

            // Get containers from the spec
            if let Some(spec) = &pod.spec {
                for container in &spec.containers {
                    let container_name = &container.name;
                    let key = format!("{}/{}", pod_name, container_name);

                    match self
                        .get_container_logs(namespace, &pod_name, container_name)
                        .await
                    {
                        Ok(logs) => {
                            log_tails.insert(key, logs);
                        }
                        Err(e) => {
                            debug!(
                                "Failed to get logs for {}/{}: {}",
                                pod_name, container_name, e
                            );
                            log_tails.insert(key, format!("Error: {}", e));
                        }
                    }
                }
            }
        }

        debug!("Collected logs for {} containers", log_tails.len());
        Ok(log_tails)
    }

    /// Gets logs for a specific container.
    async fn get_container_logs(
        &self,
        namespace: &str,
        pod_name: &str,
        container_name: &str,
    ) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        let pods: Api<Pod> = Api::namespaced(self.client.clone(), namespace);

        let logs = pods
            .logs(
                pod_name,
                &kube::api::LogParams {
                    container: Some(container_name.to_string()),
                    tail_lines: Some(MAX_LOG_LINES),
                    ..Default::default()
                },
            )
            .await?;

        Ok(logs)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pod_status_serialization() {
        let status = PodStatus {
            name: "test-pod".to_string(),
            namespace: "default".to_string(),
            phase: "Running".to_string(),
            conditions: vec![PodCondition {
                condition_type: "Ready".to_string(),
                status: "True".to_string(),
                reason: None,
                message: None,
            }],
            containers: vec![ContainerStatus {
                name: "main".to_string(),
                ready: true,
                restart_count: 0,
                state: "Running".to_string(),
                state_reason: None,
                state_message: None,
            }],
        };

        let json = serde_json::to_string(&status).unwrap();
        assert!(json.contains("test-pod"));
        assert!(json.contains("Running"));
    }

    #[test]
    fn test_event_info_serialization() {
        let event = EventInfo {
            event_type: Some("Normal".to_string()),
            reason: Some("Started".to_string()),
            message: Some("Container started".to_string()),
            involved_object: "test-pod".to_string(),
            first_timestamp: Some(Utc::now()),
            last_timestamp: Some(Utc::now()),
            count: Some(1),
        };

        let json = serde_json::to_string(&event).unwrap();
        assert!(json.contains("Normal"));
        assert!(json.contains("Started"));
    }

    #[test]
    fn test_submit_diagnostic_result_serialization() {
        let result = SubmitDiagnosticResult {
            pod_statuses: "[]".to_string(),
            events: "[]".to_string(),
            log_tails: Some("{}".to_string()),
            collected_at: Utc::now(),
        };

        let json = serde_json::to_string(&result).unwrap();
        assert!(json.contains("pod_statuses"));
        assert!(json.contains("events"));
    }
}
