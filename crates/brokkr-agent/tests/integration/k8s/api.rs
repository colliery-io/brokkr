use brokkr_agent::k8s::api::{
    apply_k8s_objects, create_k8s_client, delete_k8s_objects, dynamic_api,
    get_all_objects_by_annotation, reconcile_target_state, validate_k8s_objects,
};
use brokkr_agent::k8s::objects::{BROKKR_AGENT_OWNER_ANNOTATION, CHECKSUM_ANNOTATION, STACK_LABEL};
use k8s_openapi::api::core::v1::ConfigMap;
use k8s_openapi::api::core::v1::Namespace;
use kube::api::{DynamicObject, GroupVersionKind, Patch, PatchParams};
use kube::{Api, Client as K8sClient, Discovery};
use std::sync::Once;
use uuid::Uuid;

static INIT: Once = Once::new();

async fn wait_for_configmap_value(
    api: &Api<ConfigMap>,
    name: &str,
    expected_value: &str,
    max_attempts: u32,
) -> bool {
    for _ in 0..max_attempts {
        match api.get(name).await {
            Ok(cm) => {
                if let Some(data) = &cm.data {
                    if data.get("key1") == Some(&expected_value.to_string()) {
                        return true;
                    }
                }
                tokio::time::sleep(std::time::Duration::from_secs(1)).await;
            }
            Err(_) => {
                tokio::time::sleep(std::time::Duration::from_secs(1)).await;
            }
        }
    }
    false
}

fn create_namespace_json(name: &str, agent_id: &Uuid) -> serde_json::Value {
    serde_json::json!({
            "apiVersion": "v1",
        "kind": "Namespace",
            "metadata": {
            "name": name,
                "annotations": {
                "brokkr.io/owner-id": agent_id.to_string()
            }
        }
    })
}

async fn setup() -> (K8sClient, Discovery) {
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

    // Create discovery client
    let discovery = Discovery::new(client.clone());

    INIT.call_once(|| {
        // Any one-time initialization can go here
    });

    (client, discovery)
}

async fn cleanup(client: &K8sClient, namespace: &str) {
    // Delete test namespace if it exists
    let ns_api = Api::<Namespace>::all(client.clone());
    let _ = ns_api.delete(namespace, &Default::default()).await;
}

// Helper function to set up namespace
async fn setup_namespace(client: &K8sClient, namespace: &str, agent_id: &Uuid) {
    let ns_api = Api::<Namespace>::all(client.clone());
    let namespace_obj = create_namespace_json(namespace, agent_id);
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

async fn wait_for_deletion<T>(api: &Api<T>, name: &str, max_attempts: u32) -> bool
where
    T: kube::Resource + std::fmt::Debug + Send + Sync + Clone + serde::de::DeserializeOwned,
    T::DynamicType: Default,
{
    for _ in 0..max_attempts {
        match api.get(name).await {
            Ok(_) => {
                tokio::time::sleep(std::time::Duration::from_secs(1)).await;
                continue;
            }
            Err(kube::Error::Api(err)) if err.code == 404 => return true,
            Err(_) => continue,
        }
    }
    false
}

#[tokio::test]
async fn test_reconcile_single_object() {
    let test_namespace = format!("test-reconcile-single-{}", Uuid::new_v4());
    let agent_id = Uuid::new_v4();
    let stack_id = format!("test-stack-{}", Uuid::new_v4());
    let checksum = format!("test-checksum-{}", Uuid::new_v4());

    // Test setup
    let (client, _discovery) = setup().await;
    setup_namespace(&client, &test_namespace, &agent_id).await;

    // Create a single ConfigMap
    let objects = vec![serde_json::json!({
        "apiVersion": "v1",
        "kind": "ConfigMap",
        "metadata": {
            "name": "test-config",
            "namespace": test_namespace,
            "annotations": {
                STACK_LABEL: stack_id,
                CHECKSUM_ANNOTATION: checksum,
                BROKKR_AGENT_OWNER_ANNOTATION: agent_id.to_string()
            }
        },
        "data": {
            "key1": "value1"
        }
    })];

    let objects: Vec<DynamicObject> = objects
        .into_iter()
        .map(|obj| serde_json::from_value(obj).unwrap())
        .collect();

    // Apply the object
    let result = reconcile_target_state(&objects, client.clone(), &stack_id, &checksum).await;
    assert!(result.is_ok(), "Initial reconciliation should succeed");

    // Verify ConfigMap exists with correct data
    let cm_api = Api::<ConfigMap>::namespaced(client.clone(), &test_namespace);
    assert!(
        wait_for_configmap_value(&cm_api, "test-config", "value1", 10).await,
        "ConfigMap should have correct value"
    );

    // Delete the object and verify deletion
    let result = delete_k8s_objects(&objects, client.clone(), &agent_id).await;
    assert!(result.is_ok(), "Deletion should succeed");

    assert!(
        wait_for_deletion(&cm_api, "test-config", 10).await,
        "ConfigMap should be deleted"
    );

    cleanup(&client, &test_namespace).await;
}

#[tokio::test]
async fn test_reconcile_update_object() {
    let test_namespace = "test-reconcile-update-configmap";
    let agent_id = Uuid::new_v4();
    let stack_id = "test-stack-configmap";
    let initial_checksum = "initial-checksum-v1";
    let updated_checksum = "updated-checksum-v2";

    // Test setup
    let (client, _discovery) = setup().await;
    setup_namespace(&client, test_namespace, &agent_id).await;

    // Create initial ConfigMap
    let initial_objects = vec![serde_json::json!({
        "apiVersion": "v1",
        "kind": "ConfigMap",
        "metadata": {
            "name": "test-config",
            "namespace": test_namespace,
            "annotations": {
                STACK_LABEL: stack_id,
                CHECKSUM_ANNOTATION: initial_checksum,
                BROKKR_AGENT_OWNER_ANNOTATION: agent_id.to_string()
            }
        },
        "data": {
            "key1": "value1"
        }
    })];

    let initial_objects: Vec<DynamicObject> = initial_objects
        .into_iter()
        .map(|obj| serde_json::from_value(obj).unwrap())
        .collect();

    // Apply initial object
    let result =
        reconcile_target_state(&initial_objects, client.clone(), stack_id, initial_checksum).await;
    assert!(result.is_ok(), "Initial reconciliation should succeed");

    // Verify initial state
    let cm_api = Api::<ConfigMap>::namespaced(client.clone(), test_namespace);
    assert!(
        wait_for_configmap_value(&cm_api, "test-config", "value1", 10).await,
        "ConfigMap should have initial value"
    );

    // Update the ConfigMap
    let updated_objects = vec![serde_json::json!({
        "apiVersion": "v1",
        "kind": "ConfigMap",
        "metadata": {
            "name": "test-config",
            "namespace": test_namespace,
            "annotations": {
                STACK_LABEL: stack_id,
                CHECKSUM_ANNOTATION: updated_checksum,
                BROKKR_AGENT_OWNER_ANNOTATION: agent_id.to_string()
            }
        },
        "data": {
            "key1": "value2"
        }
    })];

    let updated_objects: Vec<DynamicObject> = updated_objects
        .into_iter()
        .map(|obj| serde_json::from_value(obj).unwrap())
        .collect();

    // Apply update
    let result =
        reconcile_target_state(&updated_objects, client.clone(), stack_id, updated_checksum).await;
    assert!(result.is_ok(), "Update reconciliation should succeed");

    // Verify update
    assert!(
        wait_for_configmap_value(&cm_api, "test-config", "value2", 10).await,
        "ConfigMap should have updated value"
    );

    cleanup(&client, test_namespace).await;
}

#[tokio::test]
async fn test_reconcile_invalid_object_rollback() {
    let test_namespace = format!("test-reconcile-rollback-{}", Uuid::new_v4());
    let agent_id = Uuid::new_v4();
    let stack_id = format!("test-stack-{}", Uuid::new_v4());
    let initial_checksum = format!("test-checksum-{}", Uuid::new_v4());
    let updated_checksum = format!("test-checksum-{}", Uuid::new_v4());

    // Test setup
    let (client, _discovery) = setup().await;
    setup_namespace(&client, &test_namespace, &agent_id).await;

    // Create initial ConfigMap
    let initial_objects = vec![serde_json::json!({
        "apiVersion": "v1",
        "kind": "ConfigMap",
        "metadata": {
            "name": "test-config",
            "namespace": test_namespace,
            "annotations": {
                STACK_LABEL: stack_id,
                CHECKSUM_ANNOTATION: initial_checksum
            }
        },
        "data": {
            "key1": "value1"
        }
    })];

    let initial_objects: Vec<DynamicObject> = initial_objects
        .into_iter()
        .map(|obj| serde_json::from_value(obj).unwrap())
        .collect();

    // Apply initial object
    let result = reconcile_target_state(
        &initial_objects,
        client.clone(),
        &stack_id,
        &initial_checksum,
    )
    .await;
    assert!(result.is_ok(), "Initial reconciliation should succeed");

    // Create invalid objects (one valid update and one invalid)
    let invalid_objects = vec![
        serde_json::json!({
            "apiVersion": "v1",
            "kind": "ConfigMap",
            "metadata": {
                "name": "test-config",
                "namespace": test_namespace,
                "annotations": {
                    STACK_LABEL: stack_id,
                    CHECKSUM_ANNOTATION: updated_checksum
                }
            },
            "data": {
                "key1": "value2"
            }
        }),
        serde_json::json!({
            "apiVersion": "v1",
            "kind": "Pod",
            "metadata": {
                "name": "invalid-pod",
                "namespace": test_namespace,
                "annotations": {
                    STACK_LABEL: stack_id,
                    CHECKSUM_ANNOTATION: updated_checksum
                }
            }
            // Missing required spec field
        }),
    ];

    let invalid_objects: Vec<DynamicObject> = invalid_objects
        .into_iter()
        .map(|obj| serde_json::from_value(obj).unwrap())
        .collect();

    // Attempt to apply invalid objects
    let result = reconcile_target_state(
        &invalid_objects,
        client.clone(),
        &stack_id,
        &updated_checksum,
    )
    .await;
    assert!(
        result.is_err(),
        "Reconciliation should fail with invalid object"
    );

    // Verify original state is preserved
    let cm_api = Api::<ConfigMap>::namespaced(client.clone(), &test_namespace);
    assert!(
        wait_for_configmap_value(&cm_api, "test-config", "value1", 10).await,
        "ConfigMap should retain original value after failed reconciliation"
    );

    cleanup(&client, &test_namespace).await;
}

#[tokio::test]
async fn test_reconcile_object_pruning() {
    let test_namespace = format!("test-reconcile-pruning-{}", Uuid::new_v4());
    let agent_id = Uuid::new_v4();
    let stack_id = format!("test-stack-{}", Uuid::new_v4());
    let initial_checksum = format!("test-checksum-{}", Uuid::new_v4());
    let updated_checksum = format!("test-checksum-{}", Uuid::new_v4());

    // Test setup
    let (client, _discovery) = setup().await;
    setup_namespace(&client, &test_namespace, &agent_id).await;

    // Create initial set of objects
    let initial_objects = vec![
        serde_json::json!({
            "apiVersion": "v1",
            "kind": "ConfigMap",
            "metadata": {
                "name": "config-1",
                "namespace": test_namespace,
                "annotations": {
                    STACK_LABEL: stack_id,
                    CHECKSUM_ANNOTATION: initial_checksum
                }
            },
            "data": {
                "key1": "value1"
            }
        }),
        serde_json::json!({
            "apiVersion": "v1",
            "kind": "ConfigMap",
            "metadata": {
                "name": "config-2",
                "namespace": test_namespace,
                "annotations": {
                    STACK_LABEL: stack_id,
                    CHECKSUM_ANNOTATION: initial_checksum
                }
            },
            "data": {
                "key2": "value2"
            }
        }),
    ];

    let initial_objects: Vec<DynamicObject> = initial_objects
        .into_iter()
        .map(|obj| serde_json::from_value(obj).unwrap())
        .collect();

    // Apply initial objects
    let result = reconcile_target_state(
        &initial_objects,
        client.clone(),
        &stack_id,
        &initial_checksum,
    )
    .await;
    assert!(result.is_ok(), "Initial reconciliation should succeed");

    // Create different set of objects
    let updated_objects = vec![serde_json::json!({
        "apiVersion": "v1",
        "kind": "ConfigMap",
        "metadata": {
            "name": "config-3",
            "namespace": test_namespace,
            "annotations": {
                STACK_LABEL: stack_id,
                CHECKSUM_ANNOTATION: updated_checksum
            }
        },
        "data": {
            "key3": "value3"
        }
    })];

    let updated_objects: Vec<DynamicObject> = updated_objects
        .into_iter()
        .map(|obj| serde_json::from_value(obj).unwrap())
        .collect();

    // Apply updated objects
    let result = reconcile_target_state(
        &updated_objects,
        client.clone(),
        &stack_id,
        &updated_checksum,
    )
    .await;
    assert!(result.is_ok(), "Update reconciliation should succeed");

    // Verify old objects are pruned
    let cm_api = Api::<ConfigMap>::namespaced(client.clone(), &test_namespace);

    // Wait for deletions
    assert!(
        wait_for_deletion(&cm_api, "config-1", 10).await,
        "config-1 should be pruned"
    );
    assert!(
        wait_for_deletion(&cm_api, "config-2", 10).await,
        "config-2 should be pruned"
    );

    // Verify new object exists
    let cm = cm_api
        .get("config-3")
        .await
        .expect("Failed to get config-3");
    assert_eq!(
        cm.data.unwrap().get("key3").unwrap(),
        "value3",
        "config-3 should exist with correct value"
    );

    cleanup(&client, &test_namespace).await;
}

#[tokio::test]
async fn test_reconcile_empty_object_list() {
    let test_namespace = "test-reconcile-empty";
    let agent_id = Uuid::new_v4();
    let stack_id = "test-stack-empty";
    let initial_checksum = "initial-checksum";
    let empty_checksum = "empty-state-checksum";

    // Test setup
    let (client, _discovery) = setup().await;
    setup_namespace(&client, test_namespace, &agent_id).await;

    // Create initial objects
    let initial_objects = vec![
        serde_json::json!({
            "apiVersion": "v1",
            "kind": "ConfigMap",
            "metadata": {
                "name": "config-1",
                "namespace": test_namespace,
                "annotations": {
                    STACK_LABEL: stack_id,
                    CHECKSUM_ANNOTATION: initial_checksum,
                    BROKKR_AGENT_OWNER_ANNOTATION: agent_id.to_string()
                }
            },
            "data": {
                "key1": "value1"
            }
        }),
        serde_json::json!({
            "apiVersion": "v1",
            "kind": "ConfigMap",
            "metadata": {
                "name": "config-2",
                "namespace": test_namespace,
                "annotations": {
                    STACK_LABEL: stack_id,
                    CHECKSUM_ANNOTATION: initial_checksum,
                    BROKKR_AGENT_OWNER_ANNOTATION: agent_id.to_string()
                }
            },
            "data": {
                "key2": "value2"
            }
        }),
    ];

    let initial_objects: Vec<DynamicObject> = initial_objects
        .into_iter()
        .map(|obj| serde_json::from_value(obj).unwrap())
        .collect();

    // Apply initial objects
    let result =
        reconcile_target_state(&initial_objects, client.clone(), stack_id, initial_checksum).await;
    assert!(result.is_ok(), "Initial reconciliation should succeed");

    // Verify initial objects exist
    let cm_api = Api::<ConfigMap>::namespaced(client.clone(), test_namespace);
    let cm1 = cm_api
        .get("config-1")
        .await
        .expect("Failed to get config-1");
    assert_eq!(
        cm1.data.unwrap().get("key1").unwrap(),
        "value1",
        "config-1 should exist with correct value"
    );
    let cm2 = cm_api
        .get("config-2")
        .await
        .expect("Failed to get config-2");
    assert_eq!(
        cm2.data.unwrap().get("key2").unwrap(),
        "value2",
        "config-2 should exist with correct value"
    );

    // Reconcile with empty object list (should delete everything)
    let empty_objects: Vec<DynamicObject> = vec![];
    let result =
        reconcile_target_state(&empty_objects, client.clone(), stack_id, empty_checksum).await;
    assert!(result.is_ok(), "Empty reconciliation should succeed");

    // Verify all objects are deleted
    assert!(
        wait_for_deletion(&cm_api, "config-1", 10).await,
        "config-1 should be deleted"
    );
    assert!(
        wait_for_deletion(&cm_api, "config-2", 10).await,
        "config-2 should be deleted"
    );

    cleanup(&client, test_namespace).await;
}
