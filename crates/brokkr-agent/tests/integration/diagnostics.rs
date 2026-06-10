/*
 * Copyright (c) 2025-2026 Dylan Storey
 * Licensed under the Elastic License 2.0.
 * See LICENSE file in the project root for full license text.
 */

use brokkr_agent::diagnostics::DiagnosticsHandler;
use brokkr_agent::k8s::api::create_k8s_client;
use brokkr_agent::k8s::objects::DEPLOYMENT_OBJECT_ID_LABEL;
use k8s_openapi::api::core::v1::{Namespace, Pod};
use kube::api::{DynamicObject, Patch, PatchParams, PostParams};
use kube::{Api, Client as K8sClient};
use std::collections::HashSet;
use uuid::Uuid;

async fn setup() -> K8sClient {
    // Initialize k8s client using the kubeconfig from the k3s container
    let client = create_k8s_client(Some("/tmp/brokkr-keys/kubeconfig.yaml"))
        .await
        .unwrap();

    // Verify cluster accessibility by attempting to list namespaces
    let ns_api = Api::<Namespace>::all(client.clone());
    ns_api
        .list(&Default::default())
        .await
        .expect("Failed to connect to Kubernetes cluster - could not list namespaces");

    client
}

async fn setup_namespace(client: &K8sClient, namespace: &str) {
    let ns_api = Api::<Namespace>::all(client.clone());
    let namespace_obj = serde_json::json!({
        "apiVersion": "v1",
        "kind": "Namespace",
        "metadata": {
            "name": namespace
        }
    });
    let ns: DynamicObject = serde_json::from_value(namespace_obj).unwrap();
    let patch_params = PatchParams::apply("brokkr-controller");
    ns_api
        .patch(
            namespace,
            &patch_params,
            &Patch::Apply(serde_json::to_value(&ns).unwrap()),
        )
        .await
        .expect("Failed to create test namespace");
}

async fn cleanup(client: &K8sClient, namespace: &str) {
    // Delete test namespace if it exists (best effort)
    let ns_api = Api::<Namespace>::all(client.clone());
    let _ = ns_api.delete(namespace, &Default::default()).await;
}

/// Creates a bare Pod carrying the deployment-object-id label and waits until
/// the API server reports it exists (it does not need to be Running).
async fn create_labeled_pod(
    client: &K8sClient,
    namespace: &str,
    name: &str,
    deployment_object_id: &Uuid,
) {
    let pod: Pod = serde_json::from_value(serde_json::json!({
        "apiVersion": "v1",
        "kind": "Pod",
        "metadata": {
            "name": name,
            "namespace": namespace,
            "labels": {
                DEPLOYMENT_OBJECT_ID_LABEL: deployment_object_id.to_string()
            }
        },
        "spec": {
            "containers": [{
                "name": "main",
                "image": "busybox",
                "command": ["sleep", "3600"]
            }]
        }
    }))
    .unwrap();

    let pod_api = Api::<Pod>::namespaced(client.clone(), namespace);
    pod_api
        .create(&PostParams::default(), &pod)
        .await
        .unwrap_or_else(|e| panic!("Failed to create pod {}/{}: {:?}", namespace, name, e));

    // Poll until the pod exists (existence is all diagnostics needs)
    let mut exists = false;
    for _ in 0..30 {
        match pod_api.get_opt(name).await {
            Ok(Some(_)) => {
                exists = true;
                break;
            }
            _ => tokio::time::sleep(std::time::Duration::from_secs(1)).await,
        }
    }
    assert!(exists, "Pod {}/{} should exist", namespace, name);
}

#[tokio::test]
async fn test_diagnostics_collects_pods_across_namespaces() {
    // BROKKR-T-0190: diagnostics collection must search every namespace it is
    // given and merge the results, so workloads outside a single namespace
    // are covered. Two namespaces each get a bare Pod labeled with the same
    // deployment-object id; the collector must return pods from BOTH.
    let ns1 = format!("test-diag-multi-a-{}", Uuid::new_v4());
    let ns2 = format!("test-diag-multi-b-{}", Uuid::new_v4());
    let deployment_object_id = Uuid::new_v4();

    let client = setup().await;
    setup_namespace(&client, &ns1).await;
    setup_namespace(&client, &ns2).await;

    create_labeled_pod(&client, &ns1, "diag-pod-a", &deployment_object_id).await;
    create_labeled_pod(&client, &ns2, "diag-pod-b", &deployment_object_id).await;

    let handler = DiagnosticsHandler::new(client.clone());
    let label_selector = format!("{}={}", DEPLOYMENT_OBJECT_ID_LABEL, deployment_object_id);
    let result = handler
        .collect_diagnostics_in(&[ns1.clone(), ns2.clone()], &label_selector)
        .await
        .expect("Diagnostics collection across namespaces should succeed");

    // pod_statuses is JSON-encoded; it must parse to an array containing
    // pods from BOTH namespaces
    let pod_statuses: serde_json::Value =
        serde_json::from_str(&result.pod_statuses).expect("pod_statuses should be valid JSON");
    let pods = pod_statuses
        .as_array()
        .expect("pod_statuses should be a JSON array");

    let found_namespaces: HashSet<&str> = pods
        .iter()
        .filter_map(|p| p.get("namespace").and_then(|v| v.as_str()))
        .collect();
    assert!(
        found_namespaces.contains(ns1.as_str()),
        "Diagnostics should include the pod from {}; got namespaces {:?}",
        ns1,
        found_namespaces
    );
    assert!(
        found_namespaces.contains(ns2.as_str()),
        "Diagnostics should include the pod from {}; got namespaces {:?}",
        ns2,
        found_namespaces
    );

    cleanup(&client, &ns1).await;
    cleanup(&client, &ns2).await;
}
