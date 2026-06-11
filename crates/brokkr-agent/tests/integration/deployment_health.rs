/*
 * Copyright (c) 2025-2026 Dylan Storey
 * Licensed under the Elastic License 2.0.
 * See LICENSE file in the project root for full license text.
 */

use brokkr_agent::deployment_health::HealthChecker;
use brokkr_agent::k8s::api::{create_k8s_client, reconcile_target_state};
use brokkr_agent::k8s::objects::{CHECKSUM_ANNOTATION, DEPLOYMENT_OBJECT_ID_LABEL, STACK_LABEL};
use k8s_openapi::api::core::v1::Namespace;
use kube::api::{DynamicObject, Patch, PatchParams};
use kube::{Api, Client as K8sClient};
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

#[tokio::test]
async fn test_health_pod_attribution_via_owner_references() {
    // BROKKR-T-0191: pods created by controllers (Deployment -> ReplicaSet ->
    // Pod) must be attributed to a deployment object through the
    // ownerReference chain when only the top-level Deployment carries the
    // `brokkr.io/deployment-object-id` annotation. The pod template carries
    // NO brokkr label/annotation at all.
    let test_namespace = format!("test-health-ownerref-{}", Uuid::new_v4());
    let stack_id = format!("test-stack-{}", Uuid::new_v4());
    let checksum = format!("test-checksum-{}", Uuid::new_v4());
    let deployment_object_id = Uuid::new_v4();

    let client = setup().await;
    setup_namespace(&client, &test_namespace).await;

    // Deployment annotated with the deployment-object id; the pod template is
    // deliberately bare so attribution can only work via ownerReferences.
    // Pods only need to EXIST for attribution, not be Ready.
    let deployment = serde_json::json!({
        "apiVersion": "apps/v1",
        "kind": "Deployment",
        "metadata": {
            "name": "health-attribution-test",
            "namespace": test_namespace,
            "annotations": {
                STACK_LABEL: stack_id,
                CHECKSUM_ANNOTATION: checksum,
                DEPLOYMENT_OBJECT_ID_LABEL: deployment_object_id.to_string()
            }
        },
        "spec": {
            "replicas": 1,
            "selector": {
                "matchLabels": {
                    "app": "health-attribution-test"
                }
            },
            "template": {
                "metadata": {
                    "labels": {
                        "app": "health-attribution-test"
                    }
                },
                "spec": {
                    "containers": [{
                        "name": "main",
                        "image": "busybox",
                        "command": ["sleep", "3600"]
                    }]
                }
            }
        }
    });

    let objects: Vec<DynamicObject> = vec![serde_json::from_value(deployment).unwrap()];

    let agent_id = Uuid::new_v4();
    let result =
        reconcile_target_state(&objects, client.clone(), &stack_id, &checksum, &agent_id, None)
            .await;
    assert!(
        result.is_ok(),
        "Reconciliation of the Deployment should succeed: {:?}",
        result.err()
    );

    // Poll until the health checker attributes at least one pod to the
    // deployment object (controller needs time to create the ReplicaSet/Pod)
    let checker = HealthChecker::new(client.clone());
    let mut attributed = false;
    for _ in 0..90 {
        let statuses = checker
            .check_deployment_objects(&[deployment_object_id])
            .await;
        if statuses
            .iter()
            .any(|s| s.id == deployment_object_id && s.summary.pods_total >= 1)
        {
            attributed = true;
            break;
        }
        tokio::time::sleep(std::time::Duration::from_secs(1)).await;
    }
    assert!(
        attributed,
        "HealthChecker should attribute at least one pod to the deployment object via ownerReferences"
    );

    // Negative control: an unrelated deployment object id in the same call
    // must come back with no pods and status "unknown"
    let unknown_id = Uuid::new_v4();
    let statuses = checker
        .check_deployment_objects(&[deployment_object_id, unknown_id])
        .await;

    let known = statuses
        .iter()
        .find(|s| s.id == deployment_object_id)
        .expect("Status for the real deployment object should be present");
    assert!(
        known.summary.pods_total >= 1,
        "Real deployment object should have at least one attributed pod"
    );

    let unknown = statuses
        .iter()
        .find(|s| s.id == unknown_id)
        .expect("Status for the unknown deployment object should be present");
    assert_eq!(
        unknown.summary.pods_total, 0,
        "Unknown deployment object should have no attributed pods"
    );
    assert_eq!(
        unknown.status, "unknown",
        "Unknown deployment object should report status 'unknown'"
    );

    cleanup(&client, &test_namespace).await;
}
