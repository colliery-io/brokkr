/*
 * Copyright (c) 2025-2026 Dylan Storey
 * Licensed under the Elastic License 2.0.
 * See LICENSE file in the project root for full license text.
 */

//! Which work order a Kubernetes object belongs to (BROKKR-T-0406).
//!
//! The agent marks what it applies for a work order with
//! `brokkr.io/work-order-id`: an annotation on each object of a `custom` work
//! order, and a label on the `BuildRun` of a `build` work order. A pod never
//! carries the mark itself, because the agent does not change pod templates.
//! A pod belongs to a work order through its owners: `Job` → `Pod`, or
//! `BuildRun` → `TaskRun` → `Pod`.
//!
//! The pod log tailer and the kube event tailer use this to send the
//! telemetry of a work order with the work order id in place of a stack id.

use k8s_openapi::apimachinery::pkg::apis::meta::v1::{ObjectMeta, OwnerReference};
use kube::Client;
use kube::api::{Api, DynamicObject};
use kube::core::GroupVersionKind;
use kube::discovery;
use tracing::debug;
use uuid::Uuid;

/// The key of the mark, as an annotation or as a label.
pub const WORK_ORDER_ID_KEY: &str = "brokkr.io/work-order-id";

/// The kinds of owner the walk follows. A pod of any other owner (a
/// ReplicaSet, a StatefulSet) costs no API call: it is not a work order's.
const WALKED_OWNER_KINDS: [&str; 3] = ["Job", "TaskRun", "BuildRun"];

/// The longest chain the walk follows: `BuildRun` → `TaskRun` → `Pod` is two
/// owners; one more is slack.
const MAX_OWNER_DEPTH: usize = 3;

/// What a telemetry frame is about: a stack, or a work order. A frame has
/// exactly one of the two ids.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TelemetryKey {
    Stack(Uuid),
    WorkOrder(Uuid),
}

impl TelemetryKey {
    /// `(stack_id, work_order_id)` for a wire frame.
    pub fn ids(self) -> (Option<Uuid>, Option<Uuid>) {
        match self {
            TelemetryKey::Stack(id) => (Some(id), None),
            TelemetryKey::WorkOrder(id) => (None, Some(id)),
        }
    }
}

/// Marks an object that the agent applies for a work order.
pub fn stamp(object: &mut DynamicObject, work_order_id: Uuid) {
    object
        .metadata
        .annotations
        .get_or_insert_with(Default::default)
        .insert(WORK_ORDER_ID_KEY.to_string(), work_order_id.to_string());
}

/// The work order an object is marked with: the label, or the annotation.
pub fn direct_mark(meta: &ObjectMeta) -> Option<Uuid> {
    let from = |map: Option<&std::collections::BTreeMap<String, String>>| {
        map.and_then(|m| m.get(WORK_ORDER_ID_KEY))
            .and_then(|v| Uuid::parse_str(v).ok())
    };
    from(meta.labels.as_ref()).or_else(|| from(meta.annotations.as_ref()))
}

/// The owner to walk to: the controller reference (else the first one), when
/// its kind is one a work order makes.
fn walked_owner(refs: Option<&[OwnerReference]>) -> Option<&OwnerReference> {
    let refs = refs?;
    let owner = refs
        .iter()
        .find(|r| r.controller == Some(true))
        .or_else(|| refs.first())?;
    WALKED_OWNER_KINDS
        .contains(&owner.kind.as_str())
        .then_some(owner)
}

/// Reads the metadata of an owner. A trait so the walk has unit tests with
/// no cluster; [`KubeMetaFetcher`] is the one used at run time.
pub(crate) trait MetaFetcher {
    /// The owner's metadata, or `None` when it cannot be read. `None` ends
    /// the walk.
    async fn fetch(&mut self, namespace: &str, owner: &OwnerReference) -> Option<ObjectMeta>;
}

/// The work order that an object belongs to: its own mark, or the mark of an
/// owner up its chain.
pub(crate) async fn resolve<F: MetaFetcher>(
    fetcher: &mut F,
    namespace: Option<&str>,
    meta: &ObjectMeta,
) -> Option<Uuid> {
    if let Some(id) = direct_mark(meta) {
        return Some(id);
    }
    let namespace = namespace?;
    let mut owner = walked_owner(meta.owner_references.as_deref())?.clone();
    for _ in 0..MAX_OWNER_DEPTH {
        let owner_meta = fetcher.fetch(namespace, &owner).await?;
        if let Some(id) = direct_mark(&owner_meta) {
            return Some(id);
        }
        owner = walked_owner(owner_meta.owner_references.as_deref())?.clone();
    }
    None
}

/// Reads owners from the API server.
pub(crate) struct KubeMetaFetcher {
    client: Client,
}

impl KubeMetaFetcher {
    pub(crate) fn new(client: Client) -> Self {
        Self { client }
    }
}

impl MetaFetcher for KubeMetaFetcher {
    async fn fetch(&mut self, namespace: &str, owner: &OwnerReference) -> Option<ObjectMeta> {
        let gvk = match owner.api_version.split_once('/') {
            Some((group, version)) => GroupVersionKind::gvk(group, version, &owner.kind),
            None => GroupVersionKind::gvk("", &owner.api_version, &owner.kind),
        };
        let ar = match discovery::pinned_kind(&self.client, &gvk).await {
            Ok((ar, _caps)) => ar,
            Err(e) => {
                debug!(?gvk, error = %e, "could not discover the kind of an owner");
                return None;
            }
        };
        let api: Api<DynamicObject> = Api::namespaced_with(self.client.clone(), namespace, &ar);
        api.get_opt(&owner.name)
            .await
            .ok()
            .flatten()
            .map(|o| o.metadata)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::{BTreeMap, HashMap};

    /// Owners by `(kind, name)`, and the kinds that were asked for.
    #[derive(Default)]
    struct FakeFetcher {
        owners: HashMap<(String, String), ObjectMeta>,
        asked: Vec<String>,
    }

    impl FakeFetcher {
        fn with(mut self, kind: &str, name: &str, meta: ObjectMeta) -> Self {
            self.owners.insert((kind.into(), name.into()), meta);
            self
        }
    }

    impl MetaFetcher for FakeFetcher {
        async fn fetch(&mut self, _namespace: &str, owner: &OwnerReference) -> Option<ObjectMeta> {
            self.asked.push(owner.kind.clone());
            self.owners
                .get(&(owner.kind.clone(), owner.name.clone()))
                .cloned()
        }
    }

    fn owner_ref(kind: &str, name: &str) -> OwnerReference {
        OwnerReference {
            api_version: "v1".into(),
            kind: kind.into(),
            name: name.into(),
            uid: "u".into(),
            controller: Some(true),
            block_owner_deletion: None,
        }
    }

    fn meta(
        annotation: Option<Uuid>,
        label: Option<Uuid>,
        owner: Option<OwnerReference>,
    ) -> ObjectMeta {
        let map = |id: Option<Uuid>| {
            id.map(|id| BTreeMap::from([(WORK_ORDER_ID_KEY.to_string(), id.to_string())]))
        };
        ObjectMeta {
            annotations: map(annotation),
            labels: map(label),
            owner_references: owner.map(|o| vec![o]),
            ..Default::default()
        }
    }

    #[test]
    fn stamp_marks_the_object_and_keeps_its_annotations() {
        let id = Uuid::new_v4();
        let mut object: DynamicObject = serde_json::from_value(serde_json::json!({
            "apiVersion": "batch/v1", "kind": "Job",
            "metadata": { "name": "j", "annotations": { "team": "robin" } }
        }))
        .unwrap();
        stamp(&mut object, id);
        let annotations = object.metadata.annotations.unwrap();
        assert_eq!(annotations.get(WORK_ORDER_ID_KEY), Some(&id.to_string()));
        assert_eq!(annotations.get("team").map(String::as_str), Some("robin"));
        assert_eq!(direct_mark(&meta(Some(id), None, None)), Some(id));
    }

    #[tokio::test]
    async fn an_object_with_the_mark_is_the_work_orders_own() {
        let id = Uuid::new_v4();
        let mut fetcher = FakeFetcher::default();
        // The annotation: an object of a custom work order.
        let got = resolve(&mut fetcher, Some("ns"), &meta(Some(id), None, None)).await;
        assert_eq!(got, Some(id));
        // The label: the BuildRun of a build work order.
        let got = resolve(&mut fetcher, Some("ns"), &meta(None, Some(id), None)).await;
        assert_eq!(got, Some(id));
        assert!(fetcher.asked.is_empty(), "a direct mark needs no API call");
    }

    #[tokio::test]
    async fn a_pod_of_a_marked_job_belongs_to_the_work_order() {
        let id = Uuid::new_v4();
        let mut fetcher = FakeFetcher::default().with("Job", "migrate", meta(Some(id), None, None));
        let pod = meta(None, None, Some(owner_ref("Job", "migrate")));
        assert_eq!(resolve(&mut fetcher, Some("ns"), &pod).await, Some(id));
    }

    #[tokio::test]
    async fn a_pod_of_a_buildrun_belongs_through_its_taskrun() {
        let id = Uuid::new_v4();
        let mut fetcher = FakeFetcher::default()
            .with(
                "TaskRun",
                "tr",
                meta(None, None, Some(owner_ref("BuildRun", "br"))),
            )
            .with("BuildRun", "br", meta(None, Some(id), None));
        let pod = meta(None, None, Some(owner_ref("TaskRun", "tr")));
        assert_eq!(resolve(&mut fetcher, Some("ns"), &pod).await, Some(id));
        assert_eq!(fetcher.asked, ["TaskRun", "BuildRun"]);
    }

    #[tokio::test]
    async fn a_pod_with_no_mark_belongs_to_no_work_order() {
        let mut fetcher = FakeFetcher::default().with("Job", "plain", meta(None, None, None));
        // A Job that no work order made.
        let pod = meta(None, None, Some(owner_ref("Job", "plain")));
        assert_eq!(resolve(&mut fetcher, Some("ns"), &pod).await, None);
        // A bare pod.
        assert_eq!(
            resolve(&mut fetcher, Some("ns"), &meta(None, None, None)).await,
            None
        );
        // An owner that is gone.
        let pod = meta(None, None, Some(owner_ref("Job", "gone")));
        assert_eq!(resolve(&mut fetcher, Some("ns"), &pod).await, None);
    }

    #[tokio::test]
    async fn a_pod_of_a_deployment_costs_no_api_call() {
        let mut fetcher = FakeFetcher::default();
        let pod = meta(None, None, Some(owner_ref("ReplicaSet", "web-abc")));
        assert_eq!(resolve(&mut fetcher, Some("ns"), &pod).await, None);
        assert!(fetcher.asked.is_empty());
    }

    #[test]
    fn a_frame_has_one_id() {
        let id = Uuid::new_v4();
        assert_eq!(TelemetryKey::Stack(id).ids(), (Some(id), None));
        assert_eq!(TelemetryKey::WorkOrder(id).ids(), (None, Some(id)));
    }
}
