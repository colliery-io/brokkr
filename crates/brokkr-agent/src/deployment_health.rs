/*
 * Copyright (c) 2025-2026 Dylan Storey
 * Licensed under the Elastic License 2.0.
 * See LICENSE file in the project root for full license text.
 */

//! Deployment Health Checker Module
//!
//! Monitors the health of deployed Kubernetes resources and reports status
//! to the broker. Detects common issues like ImagePullBackOff, CrashLoopBackOff,
//! OOMKilled, and other problematic conditions.

use crate::k8s::api::dynamic_api;
use crate::k8s::objects::DEPLOYMENT_OBJECT_ID_LABEL;
use chrono::{DateTime, Utc};
use k8s_openapi::api::core::v1::Pod;
use k8s_openapi::apimachinery::pkg::apis::meta::v1::{ObjectMeta, OwnerReference};
use kube::api::{DynamicObject, GroupVersionKind, ListParams};
use kube::discovery::Discovery;
use kube::{Api, Client};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashMap, HashSet};
use tracing::warn;
use uuid::Uuid;

/// Maximum ownerReference hops walked when attributing a pod to a
/// Brokkr-applied top-level object (Pod→ReplicaSet→Deployment is two hops;
/// CronJob-owned Jobs add one more).
const MAX_OWNER_DEPTH: usize = 4;

/// Cache key for owner-chain resolution within one discovery pass:
/// (namespace, apiVersion, kind, name).
type OwnerKey = (String, String, String, String);

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
const PENDING_CONDITIONS: &[&str] = &["Pending", "ContainerCreating", "PodInitializing"];

/// Reasons from terminated containers that indicate issues
const TERMINATED_ISSUES: &[&str] = &["OOMKilled", "Error", "ContainerCannotRun"];

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
    /// When set, pod discovery is restricted to this namespace
    /// (namespace-scoped RBAC deployments, BROKKR-T-0192).
    watch_namespace: Option<String>,
}

impl HealthChecker {
    /// Creates a new HealthChecker instance watching the whole cluster
    pub fn new(k8s_client: Client) -> Self {
        Self {
            k8s_client,
            watch_namespace: None,
        }
    }

    /// Restricts pod discovery to a single namespace when `namespace` is
    /// `Some` (for namespace-scoped RBAC deployments).
    pub fn with_watch_namespace(mut self, namespace: Option<String>) -> Self {
        self.watch_namespace = namespace;
        self
    }

    /// Checks the health of a specific deployment object by ID.
    pub async fn check_deployment_object(
        &self,
        deployment_object_id: Uuid,
    ) -> Result<DeploymentHealthStatus, Box<dyn std::error::Error + Send + Sync>> {
        let mut grouped = self.discover_pods(&[deployment_object_id]).await?;
        let pods = grouped.remove(&deployment_object_id).unwrap_or_default();
        Ok(self.analyze_pods(deployment_object_id, &pods))
    }

    /// Analyzes a set of pods attributed to one deployment object and
    /// produces its health status.
    fn analyze_pods(&self, deployment_object_id: Uuid, pods: &[Pod]) -> DeploymentHealthStatus {
        let checked_at = Utc::now();

        let mut summary = HealthSummary::default();
        let mut overall_status = "healthy";
        let mut conditions_set: std::collections::HashSet<String> =
            std::collections::HashSet::new();

        summary.pods_total = pods.len();

        for pod in pods {
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
                            if let Some(waiting) = &state.waiting
                                && let Some(reason) = &waiting.reason
                                && DEGRADED_CONDITIONS.contains(&reason.as_str())
                            {
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

                            // Check terminated state for issues
                            if let Some(terminated) = &state.terminated
                                && let Some(reason) = &terminated.reason
                                && TERMINATED_ISSUES.contains(&reason.as_str())
                            {
                                conditions_set.insert(reason.clone());
                                overall_status = "degraded";
                            }
                        }

                        // Check last terminated state for recent crashes
                        if let Some(last_state) = &cs.last_state
                            && let Some(terminated) = &last_state.terminated
                            && let Some(reason) = &terminated.reason
                            && reason == "OOMKilled"
                        {
                            conditions_set.insert("OOMKilled".to_string());
                            overall_status = "degraded";
                        }
                    }
                }

                // Check init container statuses
                if let Some(init_statuses) = &pod_status.init_container_statuses {
                    for cs in init_statuses {
                        if let Some(state) = &cs.state
                            && let Some(waiting) = &state.waiting
                            && let Some(reason) = &waiting.reason
                            && DEGRADED_CONDITIONS.contains(&reason.as_str())
                        {
                            conditions_set.insert(format!("InitContainer:{}", reason));
                            overall_status = "degraded";
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

        DeploymentHealthStatus {
            id: deployment_object_id,
            status: overall_status.to_string(),
            summary,
            checked_at,
        }
    }

    /// Discovers the pods belonging to each requested deployment object in a
    /// single cluster-wide pass, using the shared [`PodAttributor`] strategy.
    async fn discover_pods(
        &self,
        deployment_object_ids: &[Uuid],
    ) -> Result<HashMap<Uuid, Vec<Pod>>, Box<dyn std::error::Error + Send + Sync>> {
        let wanted: HashSet<Uuid> = deployment_object_ids.iter().copied().collect();
        let pods_api: Api<Pod> = match self.watch_namespace.as_deref() {
            Some(ns) => Api::namespaced(self.k8s_client.clone(), ns),
            None => Api::all(self.k8s_client.clone()),
        };
        let pods = pods_api.list(&ListParams::default()).await?;

        let mut attributor = PodAttributor::new(self.k8s_client.clone());
        Ok(attributor.group_by_deployment_object(pods, &wanted).await)
    }

    /// Checks health for multiple deployment objects with one cluster-wide
    /// pod-discovery pass.
    pub async fn check_deployment_objects(
        &self,
        deployment_object_ids: &[Uuid],
    ) -> Vec<DeploymentHealthStatus> {
        let grouped = match self.discover_pods(deployment_object_ids).await {
            Ok(grouped) => grouped,
            Err(e) => {
                warn!("Failed to discover pods for health checking: {}", e);
                // Report everything as unknown on discovery error
                return deployment_object_ids
                    .iter()
                    .map(|&id| DeploymentHealthStatus {
                        id,
                        status: "unknown".to_string(),
                        summary: HealthSummary::default(),
                        checked_at: Utc::now(),
                    })
                    .collect();
            }
        };

        deployment_object_ids
            .iter()
            .map(|&id| {
                let pods = grouped.get(&id).map(|v| v.as_slice()).unwrap_or(&[]);
                self.analyze_pods(id, pods)
            })
            .collect()
    }
}

/// Attributes pods to the Brokkr deployment object that produced them.
///
/// A pod is attributed to a deployment object when, in order:
///
/// 1. it carries the `brokkr.io/deployment-object-id` **label** (manual
///    opt-in, the historical mechanism),
/// 2. it carries the key as an **annotation** directly (bare `Pod` manifests
///    applied by Brokkr are stamped with it),
/// 3. an object in its ownerReference chain carries the annotation — pods
///    created by controllers (Deployment→ReplicaSet→Pod, Job→Pod,
///    StatefulSet/DaemonSet→Pod) resolve to the Brokkr-applied top-level
///    object (BROKKR-T-0191).
///
/// Step 3 is the case that matters in practice: Brokkr stamps the key as an
/// annotation on the top-level applied object only and never injects it into
/// pod templates, so neither a label selector nor a direct annotation match
/// finds the pods of a Deployment/StatefulSet/Job. Both continuous health
/// checking and on-demand diagnostics resolve pods through this type so the
/// two always agree on what belongs to a deployment object (BROKKR-T-0299).
///
/// One instance corresponds to one discovery pass: API discovery is built
/// lazily on first owner walk and owner-chain results (including misses) are
/// memoized, so pods sharing a ReplicaSet cost a single lookup.
pub struct PodAttributor {
    fetcher: KubeOwnerFetcher,
    cache: HashMap<OwnerKey, Option<Uuid>>,
}

impl PodAttributor {
    /// Creates an attributor bound to a Kubernetes client.
    pub fn new(client: Client) -> Self {
        Self {
            fetcher: KubeOwnerFetcher {
                client,
                discovery: None,
            },
            cache: HashMap::new(),
        }
    }

    /// Resolves the deployment object a single pod belongs to, if any.
    pub async fn deployment_object_of(&mut self, pod: &Pod) -> Option<Uuid> {
        match pod_direct_doid(pod) {
            Some(id) => Some(id),
            None => resolve_owner_doid(&mut self.fetcher, pod, &mut self.cache).await,
        }
    }

    /// Groups pods by the deployment object they belong to, keeping only the
    /// ids in `wanted`. Pods that resolve to nothing are dropped.
    pub async fn group_by_deployment_object(
        &mut self,
        pods: impl IntoIterator<Item = Pod>,
        wanted: &HashSet<Uuid>,
    ) -> HashMap<Uuid, Vec<Pod>> {
        let mut grouped: HashMap<Uuid, Vec<Pod>> = HashMap::new();
        for pod in pods {
            if let Some(id) = self.deployment_object_of(&pod).await
                && wanted.contains(&id)
            {
                grouped.entry(id).or_default().push(pod);
            }
        }
        grouped
    }

    /// Returns the subset of `pods` belonging to one deployment object.
    pub async fn pods_for(
        &mut self,
        pods: impl IntoIterator<Item = Pod>,
        deployment_object_id: Uuid,
    ) -> Vec<Pod> {
        let wanted: HashSet<Uuid> = HashSet::from([deployment_object_id]);
        self.group_by_deployment_object(pods, &wanted)
            .await
            .remove(&deployment_object_id)
            .unwrap_or_default()
    }
}

/// Looks up the metadata of an ownerReference target.
///
/// Abstracted from the chain walk purely so the walk can be unit-tested
/// without a cluster; the only production implementation is
/// [`KubeOwnerFetcher`].
trait OwnerFetcher {
    /// Returns the owner's metadata, or `None` when it cannot be resolved
    /// (discovery failure, unknown GVK, object deleted, API error). `None`
    /// stops the walk.
    async fn fetch(&mut self, namespace: &str, owner: &OwnerReference) -> Option<ObjectMeta>;
}

/// Fetches owner objects from the API server via dynamic discovery.
struct KubeOwnerFetcher {
    client: Client,
    /// Built lazily: clusters where every pod is directly attributable never
    /// pay the cost of a discovery pass.
    discovery: Option<Discovery>,
}

impl OwnerFetcher for KubeOwnerFetcher {
    async fn fetch(&mut self, namespace: &str, owner: &OwnerReference) -> Option<ObjectMeta> {
        if self.discovery.is_none() {
            match Discovery::new(self.client.clone()).run().await {
                Ok(d) => self.discovery = Some(d),
                Err(e) => {
                    warn!("Discovery failed during pod attribution: {}", e);
                    return None;
                }
            }
        }

        let gvk = gvk_of(&owner.api_version, &owner.kind);
        let (ar, caps) = self.discovery.as_ref()?.resolve_gvk(&gvk)?;
        let api: Api<DynamicObject> =
            dynamic_api(ar, caps, self.client.clone(), Some(namespace), false);

        match api.get_opt(&owner.name).await {
            Ok(Some(obj)) => Some(obj.metadata),
            Ok(None) => None,
            Err(e) => {
                warn!(
                    "Failed to fetch owner {}/{} '{}' during pod attribution: {}",
                    owner.api_version, owner.kind, owner.name, e
                );
                None
            }
        }
    }
}

/// Walks a pod's controller ownerReference chain upward until an object
/// carrying the deployment-object annotation is found, the chain ends, or
/// `MAX_OWNER_DEPTH` is reached. Results (including misses) are memoized per
/// owner in `cache`.
async fn resolve_owner_doid<F: OwnerFetcher>(
    fetcher: &mut F,
    pod: &Pod,
    cache: &mut HashMap<OwnerKey, Option<Uuid>>,
) -> Option<Uuid> {
    let namespace = pod.metadata.namespace.clone()?;
    let mut owner = controller_owner(pod.metadata.owner_references.as_deref())?.clone();
    let mut visited: Vec<OwnerKey> = Vec::new();
    let mut result: Option<Uuid> = None;

    for _ in 0..MAX_OWNER_DEPTH {
        let key: OwnerKey = (
            namespace.clone(),
            owner.api_version.clone(),
            owner.kind.clone(),
            owner.name.clone(),
        );
        if let Some(cached) = cache.get(&key) {
            result = *cached;
            break;
        }
        visited.push(key);

        let Some(meta) = fetcher.fetch(&namespace, &owner).await else {
            break;
        };
        if let Some(id) = annotations_doid(meta.annotations.as_ref()) {
            result = Some(id);
            break;
        }
        match controller_owner(meta.owner_references.as_deref()) {
            Some(next) => owner = next.clone(),
            None => break,
        }
    }

    for key in visited {
        cache.insert(key, result);
    }
    result
}

/// Extracts the deployment-object id directly carried by a pod: the
/// `brokkr.io/deployment-object-id` label, or the same key as an annotation
/// (bare Pod manifests applied by Brokkr are stamped with the annotation).
fn pod_direct_doid(pod: &Pod) -> Option<Uuid> {
    pod.metadata
        .labels
        .as_ref()
        .and_then(|labels| labels.get(DEPLOYMENT_OBJECT_ID_LABEL))
        .and_then(|v| Uuid::parse_str(v).ok())
        .or_else(|| annotations_doid(pod.metadata.annotations.as_ref()))
}

/// Extracts the deployment-object id from an annotation map.
fn annotations_doid(annotations: Option<&BTreeMap<String, String>>) -> Option<Uuid> {
    annotations
        .and_then(|a| a.get(DEPLOYMENT_OBJECT_ID_LABEL))
        .and_then(|v| Uuid::parse_str(v).ok())
}

/// Picks the owner to walk: the controller reference when present, otherwise
/// the first reference.
fn controller_owner(refs: Option<&[OwnerReference]>) -> Option<&OwnerReference> {
    let refs = refs?;
    refs.iter()
        .find(|r| r.controller == Some(true))
        .or_else(|| refs.first())
}

/// Builds a GroupVersionKind from an ownerReference's apiVersion + kind.
fn gvk_of(api_version: &str, kind: &str) -> GroupVersionKind {
    match api_version.split_once('/') {
        Some((group, version)) => GroupVersionKind::gvk(group, version, kind),
        None => GroupVersionKind::gvk("", api_version, kind),
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

    fn pod_with(
        labels: Option<BTreeMap<String, String>>,
        annotations: Option<BTreeMap<String, String>>,
    ) -> Pod {
        let mut pod = Pod::default();
        pod.metadata.labels = labels;
        pod.metadata.annotations = annotations;
        pod
    }

    #[test]
    fn test_pod_direct_doid_prefers_label_then_annotation() {
        let id = Uuid::new_v4();
        let other = Uuid::new_v4();
        let mut labels = BTreeMap::new();
        labels.insert(DEPLOYMENT_OBJECT_ID_LABEL.to_string(), id.to_string());
        let mut annotations = BTreeMap::new();
        annotations.insert(DEPLOYMENT_OBJECT_ID_LABEL.to_string(), other.to_string());

        // label wins when both are present
        let pod = pod_with(Some(labels.clone()), Some(annotations.clone()));
        assert_eq!(pod_direct_doid(&pod), Some(id));

        // annotation used when no label
        let pod = pod_with(None, Some(annotations));
        assert_eq!(pod_direct_doid(&pod), Some(other));

        // neither → None
        let pod = pod_with(None, None);
        assert_eq!(pod_direct_doid(&pod), None);

        // unparseable value → None
        let mut bad = BTreeMap::new();
        bad.insert(
            DEPLOYMENT_OBJECT_ID_LABEL.to_string(),
            "not-a-uuid".to_string(),
        );
        let pod = pod_with(Some(bad), None);
        assert_eq!(pod_direct_doid(&pod), None);
    }

    #[test]
    fn test_controller_owner_prefers_controller_ref() {
        let plain = OwnerReference {
            api_version: "apps/v1".into(),
            kind: "ReplicaSet".into(),
            name: "plain".into(),
            uid: "1".into(),
            controller: None,
            ..Default::default()
        };
        let controller = OwnerReference {
            api_version: "apps/v1".into(),
            kind: "ReplicaSet".into(),
            name: "controller".into(),
            uid: "2".into(),
            controller: Some(true),
            ..Default::default()
        };
        let refs = vec![plain.clone(), controller.clone()];
        assert_eq!(controller_owner(Some(&refs)).unwrap().name, "controller");
        let refs = vec![plain.clone()];
        assert_eq!(controller_owner(Some(&refs)).unwrap().name, "plain");
        assert!(controller_owner(None).is_none());
        assert!(controller_owner(Some(&[])).is_none());
    }

    /// In-memory stand-in for the API server, keyed by (kind, name). The
    /// real [`KubeOwnerFetcher`] needs a live cluster, so the chain walk is
    /// exercised against this instead; `KubeOwnerFetcher` itself is covered
    /// by the integration suite.
    #[derive(Default)]
    struct FakeOwnerFetcher {
        objects: HashMap<(String, String), ObjectMeta>,
        fetches: usize,
    }

    impl FakeOwnerFetcher {
        fn with(mut self, kind: &str, name: &str, meta: ObjectMeta) -> Self {
            self.objects
                .insert((kind.to_string(), name.to_string()), meta);
            self
        }
    }

    impl OwnerFetcher for FakeOwnerFetcher {
        async fn fetch(&mut self, _namespace: &str, owner: &OwnerReference) -> Option<ObjectMeta> {
            self.fetches += 1;
            self.objects
                .get(&(owner.kind.clone(), owner.name.clone()))
                .cloned()
        }
    }

    fn owner_ref(kind: &str, name: &str) -> OwnerReference {
        OwnerReference {
            api_version: "apps/v1".into(),
            kind: kind.into(),
            name: name.into(),
            uid: format!("uid-{}", name),
            controller: Some(true),
            ..Default::default()
        }
    }

    fn meta_with(annotation: Option<Uuid>, owner: Option<OwnerReference>) -> ObjectMeta {
        ObjectMeta {
            annotations: annotation.map(|id| {
                let mut a = BTreeMap::new();
                a.insert(DEPLOYMENT_OBJECT_ID_LABEL.to_string(), id.to_string());
                a
            }),
            owner_references: owner.map(|o| vec![o]),
            ..Default::default()
        }
    }

    /// Pod → ReplicaSet → Deployment, where the pod carries neither the
    /// label nor the annotation (the shape Brokkr actually produces: only
    /// the top-level applied Deployment is stamped).
    fn owned_pod(namespace: &str, name: &str, replicaset: &str) -> Pod {
        let mut pod = Pod::default();
        pod.metadata.name = Some(name.to_string());
        pod.metadata.namespace = Some(namespace.to_string());
        pod.metadata.owner_references = Some(vec![owner_ref("ReplicaSet", replicaset)]);
        pod
    }

    #[tokio::test]
    async fn test_owner_chain_resolves_deployment_replicaset_pod() {
        let id = Uuid::new_v4();
        let pod = owned_pod("prod", "web-7d9-abcde", "web-7d9");

        // The pod itself is unattributable — this is the case the label
        // selector used to miss entirely.
        assert_eq!(pod_direct_doid(&pod), None);

        let mut fetcher = FakeOwnerFetcher::default()
            .with(
                "ReplicaSet",
                "web-7d9",
                meta_with(None, Some(owner_ref("Deployment", "web"))),
            )
            .with("Deployment", "web", meta_with(Some(id), None));
        let mut cache = HashMap::new();

        assert_eq!(
            resolve_owner_doid(&mut fetcher, &pod, &mut cache).await,
            Some(id)
        );
        assert_eq!(fetcher.fetches, 2, "walked ReplicaSet then Deployment");
    }

    #[tokio::test]
    async fn test_owner_chain_memoizes_shared_replicaset() {
        let id = Uuid::new_v4();
        let mut fetcher = FakeOwnerFetcher::default()
            .with(
                "ReplicaSet",
                "web-7d9",
                meta_with(None, Some(owner_ref("Deployment", "web"))),
            )
            .with("Deployment", "web", meta_with(Some(id), None));
        let mut cache = HashMap::new();

        let first = owned_pod("prod", "web-7d9-aaaaa", "web-7d9");
        let second = owned_pod("prod", "web-7d9-bbbbb", "web-7d9");
        assert_eq!(
            resolve_owner_doid(&mut fetcher, &first, &mut cache).await,
            Some(id)
        );
        assert_eq!(
            resolve_owner_doid(&mut fetcher, &second, &mut cache).await,
            Some(id)
        );
        assert_eq!(
            fetcher.fetches, 2,
            "the second pod must be served from the owner cache"
        );
    }

    #[tokio::test]
    async fn test_owner_chain_misses_are_cached_and_bounded() {
        // An unstamped chain resolves to None, and the miss is memoized so a
        // sibling pod does not re-walk it.
        let mut fetcher = FakeOwnerFetcher::default().with(
            "ReplicaSet",
            "other-1",
            meta_with(None, Some(owner_ref("Deployment", "other"))),
        );
        // "Deployment/other" is deliberately absent: the fetch returns None
        // and the walk stops.
        let mut cache = HashMap::new();
        let pod = owned_pod("prod", "other-1-aaaaa", "other-1");
        assert_eq!(
            resolve_owner_doid(&mut fetcher, &pod, &mut cache).await,
            None
        );
        let before = fetcher.fetches;
        let sibling = owned_pod("prod", "other-1-bbbbb", "other-1");
        assert_eq!(
            resolve_owner_doid(&mut fetcher, &sibling, &mut cache).await,
            None
        );
        assert_eq!(fetcher.fetches, before, "misses are cached too");
    }

    #[tokio::test]
    async fn test_owner_chain_stops_at_max_depth() {
        // A self-referential chain must terminate rather than loop forever.
        let mut fetcher = FakeOwnerFetcher::default().with(
            "ReplicaSet",
            "loop",
            meta_with(None, Some(owner_ref("ReplicaSet", "loop"))),
        );
        let mut cache = HashMap::new();
        let pod = owned_pod("prod", "loop-aaaaa", "loop");
        assert_eq!(
            resolve_owner_doid(&mut fetcher, &pod, &mut cache).await,
            None
        );
        // The cache is only written once the walk finishes, so the sole
        // guard against a cycle is the hop budget.
        assert_eq!(fetcher.fetches, MAX_OWNER_DEPTH);
    }

    #[tokio::test]
    async fn test_owner_chain_skipped_without_namespace_or_owner() {
        let mut fetcher = FakeOwnerFetcher::default();
        let mut cache = HashMap::new();

        // No namespace → nothing to look up in.
        let mut orphan = Pod::default();
        orphan.metadata.owner_references = Some(vec![owner_ref("ReplicaSet", "rs")]);
        assert_eq!(
            resolve_owner_doid(&mut fetcher, &orphan, &mut cache).await,
            None
        );

        // No ownerReferences → bare pod, nothing to walk.
        let mut bare = Pod::default();
        bare.metadata.namespace = Some("prod".to_string());
        assert_eq!(
            resolve_owner_doid(&mut fetcher, &bare, &mut cache).await,
            None
        );

        assert_eq!(fetcher.fetches, 0, "neither case may hit the API");
    }

    #[test]
    fn test_gvk_of_grouped_and_core() {
        let gvk = gvk_of("apps/v1", "Deployment");
        assert_eq!(
            (gvk.group.as_str(), gvk.version.as_str(), gvk.kind.as_str()),
            ("apps", "v1", "Deployment")
        );
        let gvk = gvk_of("v1", "Pod");
        assert_eq!(
            (gvk.group.as_str(), gvk.version.as_str(), gvk.kind.as_str()),
            ("", "v1", "Pod")
        );
    }

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
