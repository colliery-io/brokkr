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

fn create_busybox_deployment_json(
    name: &str,
    namespace: &str,
    agent_id: &Uuid,
) -> serde_json::Value {
    let deployment = serde_json::json!({
        "apiVersion": "apps/v1",
        "kind": "Deployment",
        "metadata": {
            "name": name,
            "namespace": namespace,
            "annotations": {
                "brokkr.io/owner-id": agent_id.to_string()
            }
        },
        "spec": {
            "replicas": 1,
            "selector": {
                "matchLabels": {
                    "app": name
                }
            },
            "template": {
                "metadata": {
                    "labels": {
                        "app": name
                    }
                },
                "spec": {
                    "containers": [{
                        "name": name,
                        "image": "busybox",
                        "command": ["sleep", "3600"]
                    }]
                }
            }
        }
    });

    deployment
}

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

#[tokio::test]
async fn test_k8s_setup_and_cleanup() {
    let test_namespace = "test-setup-cleanup";
    let agent_id = Uuid::new_v4();
    // Test setup
    let (client, _discovery) = setup().await;

    // Create a test namespace using patch
    let ns_api = Api::<Namespace>::all(client.clone());
    let namespace = create_namespace_json(test_namespace, &agent_id);
    let ns: DynamicObject = serde_json::from_value(namespace).unwrap();

    let patch_params = PatchParams::apply("test-controller");
    ns_api
        .patch(
            test_namespace,
            &patch_params,
            &Patch::Apply(serde_json::to_value(&ns).unwrap()),
        )
        .await
        .expect("Failed to create test namespace");

    // Verify namespace exists
    let namespaces = ns_api
        .list(&Default::default())
        .await
        .expect("Failed to list namespaces");
    assert!(namespaces
        .items
        .iter()
        .any(|ns| ns.metadata.name.as_ref().unwrap() == test_namespace));

    // Test cleanup
    cleanup(&client, test_namespace).await;

    // Verify namespace is either deleted or terminating
    let namespaces = ns_api
        .list(&Default::default())
        .await
        .expect("Failed to list namespaces");
    let namespace = namespaces
        .items
        .iter()
        .find(|ns| ns.metadata.name.as_ref().unwrap() == test_namespace);
    match namespace {
        None => (), // Namespace doesn't exist - this is good
        Some(ns) => {
            // If namespace exists, it must be terminating
            assert_eq!(
                ns.status.as_ref().and_then(|s| s.phase.as_deref()),
                Some("Terminating"),
                "Namespace still exists and is not in Terminating state"
            );
        }
    }
}

#[tokio::test]
async fn test_create_k8s_client_with_kubeconfig() {
    let result = create_k8s_client(Some("/tmp/brokkr-keys/kubeconfig.yaml")).await;
    assert!(result.is_ok(), "Should create client with valid kubeconfig");

    // Verify cluster connectivity by attempting to list namespaces
    let client = result.unwrap();
    let ns_api = Api::<Namespace>::all(client);
    let namespaces = ns_api.list(&Default::default()).await;
    assert!(
        namespaces.is_ok(),
        "Should be able to list namespaces: {:?}",
        namespaces.err()
    );
}

#[tokio::test]
async fn test_create_k8s_client_with_invalid_path() {
    let result = create_k8s_client(Some("/nonexistent/path/config")).await;
    assert!(
        result.is_err(),
        "Should fail with non-existent kubeconfig path"
    );
}

#[tokio::test]
async fn test_create_k8s_client_default() {
    let result = create_k8s_client(None).await;
    match result {
        Ok(_) => println!("Successfully created client with default config"),
        Err(e) => println!("Failed to create client with default config: {}", e),
    }
}

#[tokio::test]
async fn test_apply_k8s_objects() {
    let test_namespace = "test-apply-k8sobjects";
    let (client, _discovery) = setup().await;
    let agent_id = Uuid::new_v4();

    // Create test namespace
    let ns_api = Api::<Namespace>::all(client.clone());
    let namespace = create_namespace_json(test_namespace, &agent_id);
    let ns: DynamicObject = serde_json::from_value(namespace).unwrap();

    let patch_params = PatchParams::apply("test-controller");
    ns_api
        .patch(
            test_namespace,
            &patch_params,
            &Patch::Apply(serde_json::to_value(&ns).unwrap()),
        )
        .await
        .expect("Failed to create test namespace");

    // Create and apply the deployment object
    let k8s_object: DynamicObject = serde_json::from_value(create_busybox_deployment_json(
        "test-deployment",
        test_namespace,
        &agent_id,
    ))
    .unwrap();

    let objects = vec![k8s_object.clone()];

    // Apply the object
    let patch_params = PatchParams::apply("test-controller");
    let result = apply_k8s_objects(&objects, client.clone(), patch_params).await;
    assert!(
        result.is_ok(),
        "Failed to apply k8s object: {:?}",
        result.err()
    );

    // Wait a bit for the deployment to be created
    tokio::time::sleep(std::time::Duration::from_secs(5)).await;

    // Verify the deployment exists
    let discovery = Discovery::new(client.clone())
        .run()
        .await
        .expect("Failed to create discovery client");
    if let Some((ar, caps)) =
        discovery.resolve_gvk(&GroupVersionKind::gvk("apps", "v1", "Deployment"))
    {
        let api = dynamic_api(ar, caps, client.clone(), Some(test_namespace), false);
        let deployment = api.get("test-deployment").await;
        assert!(
            deployment.is_ok(),
            "Failed to get deployment: {:?}",
            deployment.err()
        );

        // Verify deployment details
        let deployment = deployment.unwrap();
        assert_eq!(deployment.metadata.name.as_deref(), Some("test-deployment"));
        assert_eq!(
            deployment.metadata.namespace.as_deref(),
            Some(test_namespace)
        );
    } else {
        panic!("Failed to resolve GVK for deployment");
    }

    // Cleanup
    cleanup(&client, test_namespace).await;
}

#[tokio::test]
async fn test_validate_k8s_objects_valid() {
    let test_namespace = "test-validate-k8sobjects";
    let agent_id = Uuid::new_v4();
    // Test setup
    let (client, _discovery) = setup().await;

    // Create test namespace
    let ns_api = Api::<Namespace>::all(client.clone());
    let namespace = create_namespace_json(test_namespace, &agent_id);
    let ns: DynamicObject = serde_json::from_value(namespace).unwrap();

    let patch_params = PatchParams::apply("test-controller");
    ns_api
        .patch(
            test_namespace,
            &patch_params,
            &Patch::Apply(serde_json::to_value(&ns).unwrap()),
        )
        .await
        .expect("Failed to create test namespace");

    // Test validation of valid object
    let valid_object: DynamicObject = serde_json::from_value(create_busybox_deployment_json(
        "test-deployment",
        test_namespace,
        &agent_id,
    ))
    .unwrap();

    let result = validate_k8s_objects(&[valid_object], client.clone()).await;
    assert!(result.is_ok(), "Validation should succeed for valid object");

    // Cleanup
    cleanup(&client, test_namespace).await;
}

#[tokio::test]
async fn test_validate_k8s_objects_invalid() {
    let test_namespace = "test-validate-invalid-k8sobjects";
    let agent_id = Uuid::new_v4();
    // Test setup
    let (client, _discovery) = setup().await;

    // Create test namespace
    let ns_api = Api::<Namespace>::all(client.clone());
    let namespace = create_namespace_json(test_namespace, &agent_id);
    let ns: DynamicObject = serde_json::from_value(namespace).unwrap();

    let patch_params = PatchParams::apply("test-controller");
    ns_api
        .patch(
            test_namespace,
            &patch_params,
            &Patch::Apply(serde_json::to_value(&ns).unwrap()),
        )
        .await
        .expect("Failed to create test namespace");

    // Test validation of multiple invalid objects
    let invalid_objects = vec![
        serde_json::from_value(serde_json::json!({
            "apiVersion": "apps/v1",
            "kind": "Deployment",
            "metadata": {
                "name": "invalid-deployment-1",
                "namespace": test_namespace
            }
            // Missing required 'spec' field
        }))
        .unwrap(),
        serde_json::from_value(serde_json::json!({
            "apiVersion": "v1",
            "kind": "NonexistentKind",
            "metadata": {
                "name": "invalid-object-2",
                "namespace": test_namespace
            }
        }))
        .unwrap(),
    ];

    let result = validate_k8s_objects(&invalid_objects, client.clone()).await;
    assert!(
        result.is_err(),
        "Validation should fail for invalid objects"
    );
    let err = result.unwrap_err().to_string();
    assert!(
        err.contains("invalid-deployment-1"),
        "Error should mention first invalid object"
    );
    assert!(
        err.contains("invalid-object-2"),
        "Error should mention second invalid object"
    );

    // Cleanup
    cleanup(&client, test_namespace).await;
}

#[tokio::test]
async fn test_get_objects_by_annotation_found() {
    let test_namespace = "test-get-objects-annotation";
    let agent_id = Uuid::new_v4();
    // Test setup
    let (client, _discovery) = setup().await;

    // Create test namespace
    let ns_api = Api::<Namespace>::all(client.clone());
    let namespace = create_namespace_json(test_namespace, &agent_id);
    let ns: DynamicObject = serde_json::from_value(namespace).unwrap();

    let patch_params = PatchParams::apply("test-controller");
    ns_api
        .patch(
            test_namespace,
            &patch_params,
            &Patch::Apply(serde_json::to_value(&ns).unwrap()),
        )
        .await
        .expect("Failed to create test namespace");

    // Create and apply deployment with annotation
    let mut k8s_object: DynamicObject = serde_json::from_value(create_busybox_deployment_json(
        "test-deployment-1",
        test_namespace,
        &agent_id,
    ))
    .unwrap();

    let mut annotations = std::collections::BTreeMap::new();
    annotations.insert("brokkr.io/test-key".to_string(), "test-value".to_string());
    k8s_object.metadata.annotations = Some(annotations);

    let result = apply_k8s_objects(&vec![k8s_object], client.clone(), patch_params).await;
    assert!(result.is_ok(), "Failed to apply k8s objects");

    // Get objects by annotation
    let result = get_all_objects_by_annotation(&client, "brokkr.io/test-key", "test-value").await;
    assert!(result.is_ok(), "Failed to get objects by annotation");

    let found_objects = result.unwrap();
    assert_eq!(
        found_objects.len(),
        1,
        "Should find the deployment"
    );

    // Cleanup
    cleanup(&client, test_namespace).await;
}

#[tokio::test]
async fn test_get_objects_by_annotation_not_found() {
    let test_namespace = "test-get-objects-no-annotation";
    let agent_id = Uuid::new_v4();
    // Test setup
    let (client, _discovery) = setup().await;

    // Create test namespace
    let ns_api = Api::<Namespace>::all(client.clone());
    let namespace = create_namespace_json(test_namespace, &agent_id);
    let ns: DynamicObject = serde_json::from_value(namespace).unwrap();

    let patch_params = PatchParams::apply("test-controller");
    ns_api
        .patch(
            test_namespace,
            &patch_params,
            &Patch::Apply(serde_json::to_value(&ns).unwrap()),
        )
        .await
        .expect("Failed to create test namespace");

    // Create deployment object
    let k8s_object: DynamicObject = serde_json::from_value(create_busybox_deployment_json(
        "test-deployment",
        test_namespace,
        &agent_id,
    ))
    .unwrap();

    // Apply deployment
    let objects = vec![k8s_object];
    let result = apply_k8s_objects(&objects, client.clone(), patch_params).await;
    assert!(result.is_ok(), "Failed to apply k8s object");

    // Wait for deployment to be created
    tokio::time::sleep(std::time::Duration::from_secs(5)).await;

    // Try to get objects by non-existent annotation
    let result = get_all_objects_by_annotation(&client, "non-existent", "value").await;
    assert!(result.is_ok(), "Failed to get objects by annotation");

    let found_objects = result.unwrap();
    assert_eq!(found_objects.len(), 0, "Should not find any objects");

    // Cleanup
    cleanup(&client, test_namespace).await;
}

#[tokio::test]
async fn test_delete_k8s_object_success() {
    let test_namespace = "test-delete-k8sobjects";
    let agent_id = Uuid::new_v4();
    // Test setup
    let (client, _discovery) = setup().await;

    // Create a test namespace using patch
    let ns_api = Api::<Namespace>::all(client.clone());
    let namespace = create_namespace_json(test_namespace, &agent_id);
    let ns: DynamicObject = serde_json::from_value(namespace).unwrap();

    let patch_params = PatchParams::apply("test-controller");
    ns_api
        .patch(
            test_namespace,
            &patch_params,
            &Patch::Apply(serde_json::to_value(&ns).unwrap()),
        )
        .await
        .expect("Failed to create test namespace");

    // Create and apply the deployment object
    let k8s_object: DynamicObject = serde_json::from_value(create_busybox_deployment_json(
        "test-deployment",
        test_namespace,
        &agent_id,
    ))
    .unwrap();

    let objects = vec![k8s_object.clone()];

    // Apply the object
    let result = apply_k8s_objects(&objects, client.clone(), patch_params).await;
    assert!(
        result.is_ok(),
        "Failed to apply k8s object: {:?}",
        result.err()
    );

    // Wait for the deployment to be created
    tokio::time::sleep(std::time::Duration::from_secs(5)).await;

    // Delete the deployment
    let result = delete_k8s_objects(&objects, client.clone(), &agent_id).await;
    assert!(
        result.is_ok(),
        "Failed to delete k8s object: {:?}",
        result.err()
    );

    // Verify the deployment is deleted
    let discovery = Discovery::new(client.clone())
        .run()
        .await
        .expect("Failed to create discovery client");

    if let Some((ar, caps)) =
        discovery.resolve_gvk(&GroupVersionKind::gvk("apps", "v1", "Deployment"))
    {
        let api = dynamic_api(ar, caps, client.clone(), Some(test_namespace), false);
        let deployment = api.get("test-deployment").await;
        assert!(
            deployment.is_err(),
            "Deployment still exists after deletion"
        );
    }

    // Cleanup
    cleanup(&client, test_namespace).await;
}

#[tokio::test]
async fn test_delete_k8s_object_not_found() {
    let test_namespace = "test-delete-nonexistent";
    let agent_id = Uuid::new_v4();
    // Test setup
    let (client, _discovery) = setup().await;

    // Create a test namespace using patch
    let ns_api = Api::<Namespace>::all(client.clone());
    let namespace = create_namespace_json(test_namespace, &agent_id);
    let ns: DynamicObject = serde_json::from_value(namespace).unwrap();

    let patch_params = PatchParams::apply("test-controller");
    ns_api
        .patch(
            test_namespace,
            &patch_params,
            &Patch::Apply(serde_json::to_value(&ns).unwrap()),
        )
        .await
        .expect("Failed to create test namespace");

    // Create a DynamicObject for a non-existent deployment
    let k8s_object: DynamicObject = serde_json::from_value(create_busybox_deployment_json(
        "nonexistent-deployment",
        test_namespace,
        &agent_id,
    ))
    .unwrap();

    let objects = vec![k8s_object];

    // Try to delete the non-existent deployment
    let result = delete_k8s_objects(&objects, client.clone(), &agent_id).await;
    assert!(
        result.is_err(),
        "Expected error when deleting non-existent object"
    );

    // Cleanup
    cleanup(&client, test_namespace).await;
}
