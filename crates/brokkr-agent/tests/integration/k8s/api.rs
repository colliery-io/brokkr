use brokkr_agent::k8s::api::{
    apply_k8s_objects, create_k8s_client, delete_k8s_objects, dynamic_api,
    get_all_objects_by_annotation, reconcile_target_state, validate_k8s_objects,
};
use brokkr_agent::k8s::objects::{self, CHECKSUM_ANNOTATION, STACK_LABEL};
use brokkr_models::models::deployment_objects::DeploymentObject;
use chrono::Utc;
use k8s_openapi::api::core::v1::ConfigMap;
use k8s_openapi::api::core::v1::Namespace;
use kube::api::{DynamicObject, GroupVersionKind, Patch, PatchParams};
use kube::discovery::{ApiCapabilities, ApiResource};
use kube::{Api, Client as K8sClient, Discovery};
use std::sync::Once;
use uuid::Uuid;

static INIT: Once = Once::new();

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

async fn wait_for_deletion<T>(api: &Api<T>, name: &str, max_attempts: u32) -> bool
where
    T: kube::Resource + std::fmt::Debug + Send + Sync + Clone + serde::de::DeserializeOwned,
    T::DynamicType: Default,
{
    for _ in 0..max_attempts {
        match api.get(name).await {
            Ok(_) => {
                tokio::time::sleep(std::time::Duration::from_millis(500)).await;
                continue;
            }
            Err(kube::Error::Api(err)) if err.code == 404 => return true,
            Err(_) => continue,
        }
    }
    false
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

    let patch_params = PatchParams::apply("brokkr-controller");
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
async fn test_apply_k8s_objects() {
    let test_namespace = "test-apply-k8sobjects";
    let (client, _discovery) = setup().await;
    let agent_id = Uuid::new_v4();

    // Create test namespace
    let ns_api = Api::<Namespace>::all(client.clone());
    let namespace = create_namespace_json(test_namespace, &agent_id);
    let ns: DynamicObject = serde_json::from_value(namespace).unwrap();

    let patch_params = PatchParams::apply("brokkr-controller");
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
    let result = apply_k8s_objects(
        &objects,
        client.clone(),
        PatchParams::apply("brokkr-controller"),
    )
    .await;
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
async fn test_reapply_k8s_objects() {
    let test_namespace = "test-reapply-k8sobjects";
    let agent_id = Uuid::new_v4();
    // Test setup
    let (client, _discovery) = setup().await;

    // Create a test namespace using patch
    let ns_api = Api::<Namespace>::all(client.clone());
    let namespace = create_namespace_json(test_namespace, &agent_id);
    let ns: DynamicObject = serde_json::from_value(namespace).unwrap();

    let patch_params = PatchParams::apply("brokkr-controller");
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
    let result = apply_k8s_objects(
        &objects,
        client.clone(),
        PatchParams::apply("brokkr-controller"),
    )
    .await;
    assert!(
        result.is_ok(),
        "Failed to apply k8s object: {:?}",
        result.err()
    );

    // Wait a bit for the deployment to be created
    tokio::time::sleep(std::time::Duration::from_secs(5)).await;

    let discovery = Discovery::new(client.clone())
        .run()
        .await
        .expect("Failed to create discovery client");
    // Verify the deployment exists
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

    let result = apply_k8s_objects(
        &objects,
        client.clone(),
        PatchParams::apply("brokkr-controller"),
    )
    .await;
    assert!(
        result.is_ok(),
        "Failed to reapply k8s object: {:?}",
        result.err()
    );

    tokio::time::sleep(std::time::Duration::from_secs(5)).await;
    let discovery = Discovery::new(client.clone())
        .run()
        .await
        .expect("Failed to create discovery client");
    // Verify the deployment exists
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
async fn test_delete_k8s_object_success() {
    let test_namespace = "test-delete-k8sobjects";
    let agent_id = Uuid::new_v4();
    // Test setup
    let (client, _discovery) = setup().await;

    // Create a test namespace using patch
    let ns_api = Api::<Namespace>::all(client.clone());
    let namespace = create_namespace_json(test_namespace, &agent_id);
    let ns: DynamicObject = serde_json::from_value(namespace).unwrap();

    let patch_params = PatchParams::apply("brokkr-controller");
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
    let result = apply_k8s_objects(
        &objects,
        client.clone(),
        PatchParams::apply("brokkr-controller"),
    )
    .await;
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

    let patch_params = PatchParams::apply("brokkr-controller");
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

    let patch_params = PatchParams::apply("brokkr-controller");
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

    let result = apply_k8s_objects(
        &vec![k8s_object],
        client.clone(),
        PatchParams::apply("brokkr-controller"),
    )
    .await;
    assert!(result.is_ok(), "Failed to apply k8s objects");

    // Get objects by annotation
    let result = get_all_objects_by_annotation(&client, "brokkr.io/test-key", "test-value").await;
    assert!(result.is_ok(), "Failed to get objects by annotation");

    let found_objects = result.unwrap();
    assert_eq!(
        found_objects.len(),
        2,
        "Should find both namespace and deployment"
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

    let patch_params = PatchParams::apply("brokkr-controller");
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
    let result = apply_k8s_objects(
        &objects,
        client.clone(),
        PatchParams::apply("brokkr-controller"),
    )
    .await;
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
async fn test_validate_k8s_objects_valid() {
    let test_namespace = "test-validate-k8sobjects";
    let agent_id = Uuid::new_v4();
    // Test setup
    let (client, _discovery) = setup().await;

    // Create test namespace
    let ns_api = Api::<Namespace>::all(client.clone());
    let namespace = create_namespace_json(test_namespace, &agent_id);
    let ns: DynamicObject = serde_json::from_value(namespace).unwrap();

    let patch_params = PatchParams::apply("brokkr-controller");
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

    let patch_params = PatchParams::apply("brokkr-controller");
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
async fn test_reconcile_target_state() {
    let test_namespace = "test-reconcile-target-state";
    let agent_id = Uuid::new_v4();
    // Test setup
    let (client, _discovery) = setup().await;

    // Create a test namespace using patch
    let ns_api = Api::<Namespace>::all(client.clone());
    let namespace = create_namespace_json(test_namespace, &agent_id);
    let ns: DynamicObject = serde_json::from_value(namespace).unwrap();

    let patch_params = PatchParams::apply("brokkr-controller");
    ns_api
        .patch(
            test_namespace,
            &patch_params,
            &Patch::Apply(serde_json::to_value(&ns).unwrap()),
        )
        .await
        .expect("Failed to create test namespace");

    // Create initial valid deployment with a specific replica count
    let mut initial_deployment: DynamicObject = serde_json::from_value(
        create_busybox_deployment_json("test-deployment", test_namespace, &agent_id),
    )
    .unwrap();

    // Modify the initial deployment to have a specific replica count
    let spec = initial_deployment.data.get_mut("spec").unwrap();
    spec["replicas"] = serde_json::json!(2);

    // Apply initial deployment
    let result = apply_k8s_objects(
        &[initial_deployment.clone()],
        client.clone(),
        PatchParams::apply("brokkr-controller"),
    )
    .await;
    assert!(result.is_ok(), "Failed to apply initial deployment");

    // Create a mix of valid and invalid objects for update
    let mut updated_deployment = initial_deployment.clone();
    let spec = updated_deployment.data.get_mut("spec").unwrap();
    spec["replicas"] = serde_json::json!(3); // Change replica count

    let objects = vec![
        updated_deployment,
        // Invalid object that should cause failure
        serde_json::from_value(serde_json::json!({
            "apiVersion": "v1",
            "kind": "NonexistentKind",
            "metadata": {
                "name": "invalid-object",
                "namespace": test_namespace
            }
        }))
        .unwrap(),
    ];

    // Attempt to apply updates with rollback
    let result = reconcile_target_state(&objects, client.clone()).await;
    assert!(result.is_err(), "Should fail due to invalid object");

    // Verify the original deployment was restored
    let discovery = Discovery::new(client.clone())
        .run()
        .await
        .expect("Failed to create discovery client");

    if let Some((ar, caps)) =
        discovery.resolve_gvk(&GroupVersionKind::gvk("apps", "v1", "Deployment"))
    {
        let api = dynamic_api(ar, caps, client.clone(), Some(test_namespace), false);
        let deployment = api.get("test-deployment").await.unwrap();

        // Compare the replica count instead of annotations
        let deployment_replicas = deployment
            .data
            .get("spec")
            .and_then(|spec| spec.get("replicas"))
            .and_then(|replicas| replicas.as_i64())
            .unwrap_or(0);

        let original_replicas = initial_deployment
            .data
            .get("spec")
            .and_then(|spec| spec.get("replicas"))
            .and_then(|replicas| replicas.as_i64())
            .unwrap_or(0);

        assert_eq!(
            deployment_replicas, original_replicas,
            "Deployment should be rolled back to original replica count"
        );
    }

    // Cleanup
    cleanup(&client, test_namespace).await;
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
async fn test_reconcile_target_state_pruning() {
    let (client, _discovery) = setup().await;
    let test_namespace = "test-pruning";
    let stack_id = Uuid::new_v4().to_string();
    let initial_checksum = "checksum-1".to_string();
    let updated_checksum = "checksum-2".to_string();

    // Create test namespace
    let ns_api = Api::<Namespace>::all(client.clone());
    let namespace = create_namespace_json(test_namespace, &Uuid::new_v4());
    let ns: DynamicObject = serde_json::from_value(namespace).unwrap();
    let patch_params = PatchParams::apply("brokkr-controller");
    ns_api
        .patch(
            test_namespace,
            &patch_params,
            &Patch::Apply(serde_json::to_value(&ns).unwrap()),
        )
        .await
        .expect("Failed to create test namespace");

    // Create initial objects including a NetworkPolicy (non-common type)
    let initial_objects = vec![
        serde_json::json!({
            "apiVersion": "networking.k8s.io/v1",
            "kind": "NetworkPolicy",
            "metadata": {
                "name": "test-netpol",
                "namespace": test_namespace,
                "annotations": {
                    STACK_LABEL: stack_id,
                    CHECKSUM_ANNOTATION: initial_checksum
                }
            },
            "spec": {
                "podSelector": {},
                "policyTypes": ["Ingress"],
                "ingress": [{
                    "from": [{
                        "podSelector": {}
                    }]
                }]
            }
        }),
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
            "data": { "key1": "value1" }
        }),
    ];

    // Convert to DynamicObjects and reconcile initial state
    let initial_objects: Vec<DynamicObject> = initial_objects
        .into_iter()
        .map(|obj| serde_json::from_value(obj).unwrap())
        .collect();

    let result = reconcile_target_state(&initial_objects, client.clone()).await;
    assert!(result.is_ok(), "Failed to reconcile initial objects");

    // Wait for objects to be created
    tokio::time::sleep(std::time::Duration::from_secs(2)).await;

    // Create updated state with different objects
    let updated_objects = vec![serde_json::json!({
        "apiVersion": "v1",
        "kind": "ConfigMap",
        "metadata": {
            "name": "config-2",
            "namespace": test_namespace,
            "annotations": {
                STACK_LABEL: stack_id,
                CHECKSUM_ANNOTATION: updated_checksum
            }
        },
        "data": { "key2": "value2" }
    })];

    // Convert to DynamicObjects and reconcile with updated state
    let updated_objects: Vec<DynamicObject> = updated_objects
        .into_iter()
        .map(|obj| serde_json::from_value(obj).unwrap())
        .collect();

    let result = reconcile_target_state(&updated_objects, client.clone()).await;
    assert!(result.is_ok(), "Failed to reconcile updated objects");

    // Wait for pruning to complete
    tokio::time::sleep(std::time::Duration::from_secs(2)).await;

    // Verify old objects are pruned
    let cm_api = Api::<ConfigMap>::namespaced(client.clone(), test_namespace);
    let netpol_api = Api::<k8s_openapi::api::networking::v1::NetworkPolicy>::namespaced(
        client.clone(),
        test_namespace,
    );

    assert!(
        cm_api.get("config-1").await.is_err(),
        "config-1 should be pruned"
    );
    assert!(
        netpol_api.get("test-netpol").await.is_err(),
        "test-netpol should be pruned"
    );

    // Verify new objects exist
    assert!(
        cm_api.get("config-2").await.is_ok(),
        "config-2 should exist"
    );

    // Cleanup
    cleanup(&client, test_namespace).await;
}

#[tokio::test]
async fn test_reconcile_target_state_non_common_types() {
    let (client, _discovery) = setup().await;
    let test_namespace = "test-non-common";
    let stack_id = Uuid::new_v4().to_string();
    let initial_checksum = "checksum-1".to_string();
    let updated_checksum = "checksum-2".to_string();

    // Create test namespace
    let ns_api = Api::<Namespace>::all(client.clone());
    let namespace = create_namespace_json(test_namespace, &Uuid::new_v4());
    let ns: DynamicObject = serde_json::from_value(namespace).unwrap();
    let patch_params = PatchParams::apply("brokkr-controller");
    ns_api
        .patch(
            test_namespace,
            &patch_params,
            &Patch::Apply(serde_json::to_value(&ns).unwrap()),
        )
        .await
        .expect("Failed to create test namespace");

    // Create initial objects
    let initial_objects = vec![serde_json::json!({
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
        "data": { "key1": "value1" }
    })];

    // Convert to DynamicObjects and reconcile initial state
    let initial_objects: Vec<DynamicObject> = initial_objects
        .into_iter()
        .map(|obj| serde_json::from_value(obj).unwrap())
        .collect();

    let result = reconcile_target_state(&initial_objects, client.clone()).await;
    assert!(result.is_ok(), "Failed to reconcile initial objects");

    // Wait for objects to be created
    tokio::time::sleep(std::time::Duration::from_secs(2)).await;

    // Create updated state with same name but different data
    let updated_objects = vec![serde_json::json!({
        "apiVersion": "v1",
        "kind": "ConfigMap",
        "metadata": {
            "name": "config-1",
            "namespace": test_namespace,
            "annotations": {
                STACK_LABEL: stack_id,
                CHECKSUM_ANNOTATION: updated_checksum
            }
        },
        "data": { "key1": "value2" }
    })];

    // Convert to DynamicObjects and reconcile with updated state
    let updated_objects: Vec<DynamicObject> = updated_objects
        .into_iter()
        .map(|obj| serde_json::from_value(obj).unwrap())
        .collect();

    let result = reconcile_target_state(&updated_objects, client.clone()).await;
    assert!(result.is_ok(), "Failed to reconcile updated objects");

    // Wait for update with polling
    let cm_api = Api::<ConfigMap>::namespaced(client.clone(), test_namespace);
    let mut attempts = 0;
    const MAX_ATTEMPTS: u32 = 10;
    let mut updated = false;

    while attempts < MAX_ATTEMPTS && !updated {
        match cm_api.get("config-1").await {
            Ok(cm) => {
                if let Some(data) = cm.data {
                    if data.get("key1") == Some(&"value2".to_string()) {
                        updated = true;
                    }
                }
            }
            Err(e) => println!("Error checking ConfigMap: {}", e),
        }
        if !updated {
            tokio::time::sleep(std::time::Duration::from_secs(1)).await;
            attempts += 1;
        }
    }

    assert!(updated, "config-1 should be updated with new data");

    // Cleanup
    cleanup(&client, test_namespace).await;
}

#[tokio::test]
async fn test_reconcile_target_state_empty_objects() {
    let (client, _discovery) = setup().await;
    let test_namespace = "test-empty-reconcile";
    let stack_id = Uuid::new_v4().to_string();
    let initial_checksum = "checksum-1".to_string();

    // Create test namespace
    let ns_api = Api::<Namespace>::all(client.clone());
    let namespace = create_namespace_json(test_namespace, &Uuid::new_v4());
    let ns: DynamicObject = serde_json::from_value(namespace).unwrap();
    let patch_params = PatchParams::apply("brokkr-controller");
    ns_api
        .patch(
            test_namespace,
            &patch_params,
            &Patch::Apply(serde_json::to_value(&ns).unwrap()),
        )
        .await
        .expect("Failed to create test namespace");

    // First create some objects that we expect to be pruned
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
            "data": { "key1": "value1" }
        }),
        serde_json::json!({
            "apiVersion": "networking.k8s.io/v1",
            "kind": "NetworkPolicy",
            "metadata": {
                "name": "test-netpol",
                "namespace": test_namespace,
                "annotations": {
                    STACK_LABEL: stack_id,
                    CHECKSUM_ANNOTATION: initial_checksum
                }
            },
            "spec": {
                "podSelector": {},
                "policyTypes": ["Ingress"],
                "ingress": [{
                    "from": [{
                        "podSelector": {}
                    }]
                }]
            }
        }),
    ];

    // Convert to DynamicObjects and apply initial state
    let initial_objects: Vec<DynamicObject> = initial_objects
        .into_iter()
        .map(|obj| serde_json::from_value(obj).unwrap())
        .collect();

    let result = apply_k8s_objects(
        &initial_objects,
        client.clone(),
        PatchParams::apply("brokkr-controller"),
    )
    .await;
    assert!(result.is_ok(), "Failed to apply initial objects");

    // Wait for objects to be created with polling
    let netpol_api = Api::<k8s_openapi::api::networking::v1::NetworkPolicy>::namespaced(
        client.clone(),
        test_namespace,
    );
    let cm_api = Api::<ConfigMap>::namespaced(client.clone(), test_namespace);

    let mut attempts = 0;
    const MAX_ATTEMPTS: u32 = 20;
    let mut netpol_exists = false;
    let mut cm_exists = false;

    while attempts < MAX_ATTEMPTS && (!netpol_exists || !cm_exists) {
        if !netpol_exists {
            // List all NetworkPolicies in namespace for debugging
            match netpol_api.list(&Default::default()).await {
                Ok(list) => {
                    println!("NetworkPolicies in namespace:");
                    for netpol in &list.items {
                        println!(
                            "  - {} (annotations: {:?})",
                            netpol.metadata.name.as_deref().unwrap_or("unnamed"),
                            netpol.metadata.annotations
                        );
                    }
                }
                Err(e) => println!("Error listing NetworkPolicies: {}", e),
            }

            match netpol_api.get("test-netpol").await {
                Ok(netpol) => {
                    println!("Found test-netpol: {:?}", netpol.metadata.annotations);
                    netpol_exists = true;
                }
                Err(e) => println!("NetworkPolicy not ready (attempt {}): {}", attempts + 1, e),
            }
        }
        if !cm_exists {
            match cm_api.get("config-1").await {
                Ok(_) => cm_exists = true,
                Err(e) => println!("ConfigMap not ready (attempt {}): {}", attempts + 1, e),
            }
        }
        if !netpol_exists || !cm_exists {
            tokio::time::sleep(std::time::Duration::from_secs(2)).await;
            attempts += 1;
        }
    }

    assert!(netpol_exists, "test-netpol should exist after polling");
    assert!(cm_exists, "config-1 should exist after polling");

    // Now try to reconcile with an empty set of objects
    let empty_objects: Vec<DynamicObject> = vec![];
    let result = reconcile_target_state(&empty_objects, client.clone()).await;

    // Should return Ok(()) for empty vector
    assert!(
        result.is_ok(),
        "Should handle empty objects vector gracefully"
    );

    // Original objects should still exist since we can't determine the stack ID from an empty set
    assert!(
        cm_api.get("config-1").await.is_ok(),
        "config-1 should still exist after empty reconcile"
    );
    assert!(
        netpol_api.get("test-netpol").await.is_ok(),
        "test-netpol should still exist after empty reconcile"
    );

    // Cleanup
    cleanup(&client, test_namespace).await;
}

#[tokio::test]
async fn test_reconcile_target_state_checksum_pruning() {
    let (client, _discovery) = setup().await;
    let test_namespace = "test-checksum-pruning";
    let stack_id = Uuid::new_v4().to_string();
    let initial_checksum = "checksum-1".to_string();
    let updated_checksum = "checksum-2".to_string();

    // Create test namespace
    let ns_api = Api::<Namespace>::all(client.clone());
    let namespace = create_namespace_json(test_namespace, &Uuid::new_v4());
    let ns: DynamicObject = serde_json::from_value(namespace).unwrap();
    let patch_params = PatchParams::apply("brokkr-controller");
    ns_api
        .patch(
            test_namespace,
            &patch_params,
            &Patch::Apply(serde_json::to_value(&ns).unwrap()),
        )
        .await
        .expect("Failed to create test namespace");

    // Create initial object
    let initial_objects = vec![serde_json::json!({
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
        "data": { "key1": "value1" }
    })];

    // Convert to DynamicObjects and apply initial state
    let initial_objects: Vec<DynamicObject> = initial_objects
        .into_iter()
        .map(|obj| serde_json::from_value(obj).unwrap())
        .collect();

    let result = apply_k8s_objects(
        &initial_objects,
        client.clone(),
        PatchParams::apply("brokkr-controller"),
    )
    .await;
    assert!(result.is_ok(), "Failed to apply initial objects");

    // Wait for object to be created with polling
    let cm_api = Api::<ConfigMap>::namespaced(client.clone(), test_namespace);

    let mut attempts = 0;
    const MAX_ATTEMPTS: u32 = 20;
    let mut cm_exists = false;

    while attempts < MAX_ATTEMPTS && !cm_exists {
        match cm_api.get("config-1").await {
            Ok(_) => cm_exists = true,
            Err(e) => println!("ConfigMap not ready (attempt {}): {}", attempts + 1, e),
        }
        if !cm_exists {
            tokio::time::sleep(std::time::Duration::from_secs(2)).await;
            attempts += 1;
        }
    }

    assert!(cm_exists, "config-1 should exist after polling");

    // Create updated state with same object but different checksum
    let updated_objects = vec![serde_json::json!({
        "apiVersion": "v1",
        "kind": "ConfigMap",
        "metadata": {
            "name": "config-1", // Same name
            "namespace": test_namespace,
            "annotations": {
                STACK_LABEL: stack_id,
                CHECKSUM_ANNOTATION: updated_checksum // Different checksum
            }
        },
        "data": { "key1": "value2" } // Different data
    })];

    // Convert to DynamicObjects and apply updated state
    let updated_objects: Vec<DynamicObject> = updated_objects
        .into_iter()
        .map(|obj| {
            // Force ownership of all fields
            let mut obj: DynamicObject = serde_json::from_value(obj).unwrap();
            if let Some(annotations) = &mut obj.metadata.annotations {
                annotations.insert("force-ownership".to_string(), Utc::now().to_rfc3339());
            }
            obj
        })
        .collect();

    let result = apply_k8s_objects(
        &updated_objects,
        client.clone(),
        PatchParams::apply("brokkr-controller"),
    )
    .await;
    assert!(result.is_ok(), "Failed to apply updated objects");

    // Wait for object to be updated with polling
    attempts = 0;
    let mut updated = false;

    while attempts < MAX_ATTEMPTS && !updated {
        match cm_api.get("config-1").await {
            Ok(cm) => {
                // Check if the ConfigMap has been updated with the new data
                if let Some(data) = cm.data {
                    if data.get("key1") == Some(&"value2".to_string()) {
                        updated = true;
                    }
                }
            }
            Err(e) => println!("Error checking ConfigMap: {}", e),
        }
        if !updated {
            tokio::time::sleep(std::time::Duration::from_secs(2)).await;
            attempts += 1;
        }
    }

    assert!(updated, "config-1 should be updated with new data");

    // Cleanup
    cleanup(&client, test_namespace).await;
}
