/*
 * Copyright (c) 2025-2026 Dylan Storey
 * Licensed under the Elastic License 2.0.
 * See LICENSE file in the project root for full license text.
 */

//! Diagnostics handler for on-demand diagnostic collection.
//!
//! This module provides functionality to collect detailed diagnostic information
//! about Kubernetes resources, including pod statuses, events, and log tails.

use crate::deployment_health::PodAttributor;
use chrono::{DateTime, Utc};
use k8s_openapi::api::core::v1::{Event, Pod};
use kube::{
    Client,
    api::{Api, ListParams},
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tracing::{debug, info};
use uuid::Uuid;

/// Maximum number of log lines to collect per container.
const MAX_LOG_LINES: i64 = 100;

/// Maximum number of (most recent) namespace events returned per namespace.
const MAX_EVENTS: usize = 50;

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
    /// Name of the object the event refers to, or `"unknown"` when the
    /// source event carries no name.
    pub involved_object: String,
    /// Kind of the object the event refers to (`Pod`, `ReplicaSet`, …).
    /// `None` when the source event omits it. Without this a bare name is
    /// ambiguous — several kinds in a namespace routinely share one.
    pub involved_object_kind: Option<String>,
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

    /// Collects diagnostics for one deployment object within a single namespace.
    ///
    /// # Arguments
    /// * `namespace` - The Kubernetes namespace
    /// * `deployment_object_id` - The deployment object to attribute pods to
    ///
    /// # Returns
    /// A SubmitDiagnosticResult containing collected diagnostics
    pub async fn collect_diagnostics(
        &self,
        namespace: &str,
        deployment_object_id: Uuid,
    ) -> Result<SubmitDiagnosticResult, Box<dyn std::error::Error + Send + Sync>> {
        self.collect_diagnostics_in(&[namespace.to_string()], deployment_object_id)
            .await
    }

    /// Collects diagnostics across multiple namespaces and merges the results.
    ///
    /// Namespaces are derived from the deployment object's manifests so
    /// diagnostics work for workloads outside `default` (BROKKR-T-0190).
    ///
    /// Pods are attributed to `deployment_object_id` with the shared
    /// [`PodAttributor`] strategy rather than a label selector: Brokkr stamps
    /// `brokkr.io/deployment-object-id` as an annotation on the top-level
    /// applied object and never injects it into pod templates, so selecting
    /// pods by that label matched nothing for controller-managed workloads
    /// (BROKKR-T-0299).
    ///
    /// # Arguments
    /// * `namespaces` - The Kubernetes namespaces to search
    /// * `deployment_object_id` - The deployment object to attribute pods to
    ///
    /// # Returns
    /// A SubmitDiagnosticResult containing the merged diagnostics
    pub async fn collect_diagnostics_in(
        &self,
        namespaces: &[String],
        deployment_object_id: Uuid,
    ) -> Result<SubmitDiagnosticResult, Box<dyn std::error::Error + Send + Sync>> {
        info!(
            "Collecting diagnostics for namespaces={:?}, deployment_object={}",
            namespaces, deployment_object_id
        );

        let mut pod_statuses = Vec::new();
        let mut events = Vec::new();
        let mut log_tails: HashMap<String, String> = HashMap::new();

        // One attributor for the whole request so API discovery and
        // owner-chain lookups are shared across namespaces.
        let mut attributor = PodAttributor::new(self.client.clone());

        for namespace in namespaces {
            let pods = self
                .resolve_pods(&mut attributor, namespace, deployment_object_id)
                .await?;
            pod_statuses.extend(pods.iter().map(pod_status_of));
            events.extend(self.collect_events(namespace).await?);
            log_tails.extend(self.collect_log_tails(namespace, &pods).await);
        }

        Ok(SubmitDiagnosticResult {
            pod_statuses: serde_json::to_string(&pod_statuses)?,
            events: serde_json::to_string(&events)?,
            // With no namespace to search, `{}` would claim "looked, found no
            // containers"; `null` correctly says logs were never collected.
            log_tails: if namespaces.is_empty() {
                None
            } else {
                Some(serde_json::to_string(&log_tails)?)
            },
            collected_at: Utc::now(),
        })
    }

    /// Lists the pods in `namespace` and keeps the ones belonging to
    /// `deployment_object_id`.
    ///
    /// Attribution is delegated to [`PodAttributor`], the same resolver used
    /// by continuous health checking: direct `brokkr.io/deployment-object-id`
    /// label, then the same key as a direct annotation, then a walk up the
    /// ownerReference chain to the Brokkr-applied top-level object
    /// (Deployment→ReplicaSet→Pod and friends).
    async fn resolve_pods(
        &self,
        attributor: &mut PodAttributor,
        namespace: &str,
        deployment_object_id: Uuid,
    ) -> Result<Vec<Pod>, Box<dyn std::error::Error + Send + Sync>> {
        let pods: Api<Pod> = Api::namespaced(self.client.clone(), namespace);
        let pod_list = pods.list(&ListParams::default()).await?;

        let matched = attributor.pods_for(pod_list, deployment_object_id).await;
        debug!(
            "Resolved {} pods in namespace {} for deployment object {}",
            matched.len(),
            namespace,
            deployment_object_id
        );
        Ok(matched)
    }

    /// Collects the most recent events in a namespace.
    ///
    /// Events are deliberately **namespace-scoped and unfiltered**, not
    /// narrowed to the deployment object's pods (BROKKR-T-0299):
    ///
    /// * The Kubernetes events API has no way to select by the involved
    ///   object's labels or annotations — `ListParams::labels` matches labels
    ///   on the `Event` resource itself, which controllers never set. The
    ///   `_label_selector` this function used to accept was therefore not
    ///   merely ignored, it was unimplementable.
    /// * Filtering to `involvedObject.name ∈ {resolved pods}` would drop the
    ///   events that most often explain a failure, because they are recorded
    ///   against something other than the pod: `FailedCreate` on the
    ///   ReplicaSet, `FailedScheduling` for a pod that never existed,
    ///   quota/PVC/node events in the namespace.
    ///
    /// The namespaces searched are already narrowed to those the deployment
    /// object's manifests declare, so this is "everything happening where
    /// this deployment object lives", capped at the `MAX_EVENTS` most recent.
    async fn collect_events(
        &self,
        namespace: &str,
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
                involved_object_kind: event.involved_object.kind.clone(),
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
        event_infos.truncate(MAX_EVENTS);

        debug!("Collected {} events", event_infos.len());
        Ok(event_infos)
    }

    /// Collects log tails for the already-resolved pods of a deployment object.
    ///
    /// Per-container fetch failures are recorded as the tail's value rather
    /// than failing the collection, so one unreadable container cannot blank
    /// the whole result.
    async fn collect_log_tails(&self, namespace: &str, pods: &[Pod]) -> HashMap<String, String> {
        let mut log_tails = HashMap::new();

        for pod in pods {
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
        log_tails
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

/// Projects a `Pod` onto the diagnostic [`PodStatus`] wire shape.
///
/// Pure and independent of the API client so the projection is unit-testable;
/// pod selection happens before this point (see
/// [`DiagnosticsHandler::resolve_pods`]).
fn pod_status_of(pod: &Pod) -> PodStatus {
    let name = pod.metadata.name.clone().unwrap_or_default();
    let namespace = pod.metadata.namespace.clone().unwrap_or_default();

    let Some(status) = &pod.status else {
        return PodStatus {
            name,
            namespace,
            phase: "Unknown".to_string(),
            conditions: vec![],
            containers: vec![],
        };
    };

    let phase = status
        .phase
        .clone()
        .unwrap_or_else(|| "Unknown".to_string());

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
                    let (state, state_reason, state_message) = match &c.state {
                        Some(state) => {
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
                        }
                        None => ("Unknown".to_string(), None, None),
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
        namespace,
        phase,
        conditions,
        containers,
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
            involved_object_kind: Some("Pod".to_string()),
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

    #[test]
    fn test_pod_status_of_projects_waiting_container() {
        // A pod stuck in ImagePullBackOff is the archetypal diagnostics
        // payload; the reason/message must survive the projection.
        let pod: Pod = serde_json::from_value(serde_json::json!({
            "apiVersion": "v1",
            "kind": "Pod",
            "metadata": { "name": "web-7d9-abcde", "namespace": "prod" },
            "status": {
                "phase": "Pending",
                "conditions": [{
                    "type": "Ready",
                    "status": "False",
                    "reason": "ContainersNotReady",
                    "message": "containers with unready status: [web]"
                }],
                "containerStatuses": [{
                    "name": "web",
                    "ready": false,
                    "restartCount": 3,
                    "image": "example/web:1",
                    "imageID": "",
                    "state": {
                        "waiting": {
                            "reason": "ImagePullBackOff",
                            "message": "Back-off pulling image"
                        }
                    }
                }]
            }
        }))
        .unwrap();

        let status = pod_status_of(&pod);
        assert_eq!(status.name, "web-7d9-abcde");
        assert_eq!(status.namespace, "prod");
        assert_eq!(status.phase, "Pending");
        assert_eq!(status.conditions.len(), 1);
        assert_eq!(status.conditions[0].condition_type, "Ready");
        assert_eq!(status.containers.len(), 1);
        assert_eq!(status.containers[0].state, "Waiting");
        assert_eq!(
            status.containers[0].state_reason.as_deref(),
            Some("ImagePullBackOff")
        );
        assert_eq!(status.containers[0].restart_count, 3);
        assert!(!status.containers[0].ready);
    }

    #[test]
    fn test_pod_status_of_without_status_is_unknown() {
        let mut pod = Pod::default();
        pod.metadata.name = Some("no-status".to_string());
        pod.metadata.namespace = Some("prod".to_string());

        let status = pod_status_of(&pod);
        assert_eq!(status.phase, "Unknown");
        assert!(status.conditions.is_empty());
        assert!(status.containers.is_empty());
    }
}
