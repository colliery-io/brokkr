use k8s_openapi::api::core::v1::Namespace;
use kube::api::{DynamicObject, GroupVersionKind, Patch, PatchParams};
use kube::{Api, Client as K8sClient, Discovery};
use std::sync::Once;
use brokkr_agent::k8s::api::{apply_k8s_objects, dynamic_api, delete_k8s_objects, get_all_objects_by_annotation};

static INIT: Once = Once::new();

fn create_namespace_json(name: &str) -> serde_json::Value {
    serde_json::json!({
        "apiVersion": "v1",
        "kind": "Namespace",
        "metadata": {
            "name": name
        }
    })
}

fn create_busybox_deployment_json(name: &str, namespace: &str) -> serde_json::Value {
    serde_json::json!({
        "apiVersion": "apps/v1",
        "kind": "Deployment",
        "metadata": {
            "name": name,
            "namespace": namespace
        },
        "spec": {
            "replicas": 1,
            "selector": {
                "matchLabels": {
                    "app": "busybox"
                }
            },
            "template": {
                "metadata": {
                    "labels": {
                        "app": "busybox"
                    }
                },
                "spec": {
                    "containers": [
                        {
                            "name": "busybox",
                            "image": "busybox:latest",
                            "command": ["sleep", "infinity"],
                            "ports": [
                                {
                                    "containerPort": 8080
                                }
                            ]
                        }
                    ]
                }
            }
        }
    })
}

async fn setup() -> (K8sClient, Discovery) {
    // Initialize k8s client using the kubeconfig from the k3s container
    std::env::set_var("KUBECONFIG", "/tmp/brokkr-keys/kubeconfig.yaml");

    let client = K8sClient::try_default()
        .await
        .expect("Failed to create k8s client");

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

#[tokio::test]
async fn test_k8s_setup_and_cleanup() {
    let test_namespace = "test-setup-cleanup";
    
    // Test setup
    let (client, _discovery) = setup().await;
    
    // Create a test namespace using patch
    let ns_api = Api::<Namespace>::all(client.clone());
    let namespace = create_namespace_json(test_namespace);
    let ns: DynamicObject = serde_json::from_value(namespace).unwrap();
    
    let patch_params = PatchParams::apply("test-controller");
    ns_api
        .patch(test_namespace, &patch_params, &Patch::Apply(serde_json::to_value(&ns).unwrap()))
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
    
    // Test setup
    let (client, _discovery) = setup().await;
    
    // Create a test namespace using patch
    let ns_api = Api::<Namespace>::all(client.clone());
    let namespace = create_namespace_json(test_namespace);
    let ns: DynamicObject = serde_json::from_value(namespace).unwrap();
    
    let patch_params = PatchParams::apply("test-controller");
    ns_api
        .patch(test_namespace, &patch_params, &Patch::Apply(serde_json::to_value(&ns).unwrap()))
        .await
        .expect("Failed to create test namespace");

    // Create and apply the deployment object
    let k8s_object: DynamicObject = serde_json::from_value(create_busybox_deployment_json(
        "test-deployment",
        test_namespace,
    ))
    .unwrap();

    let objects = vec![k8s_object.clone()];

    // Create patch params
    let patch_params = PatchParams::apply("test-controller");

    // Apply the object
    let result = apply_k8s_objects(&objects, client.clone(), &patch_params).await;
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

    // Cleanup
    cleanup(&client, test_namespace).await;
}

#[tokio::test]
async fn test_reapply_k8s_objects() {
    let test_namespace = "test-reapply-k8sobjects";
    
    // Test setup
    let (client, _discovery) = setup().await;
    
    // Create a test namespace using patch
    let ns_api = Api::<Namespace>::all(client.clone());
    let namespace = create_namespace_json(test_namespace);
    let ns: DynamicObject = serde_json::from_value(namespace).unwrap();
    
    let patch_params = PatchParams::apply("test-controller");
    ns_api
        .patch(test_namespace, &patch_params, &Patch::Apply(serde_json::to_value(&ns).unwrap()))
        .await
        .expect("Failed to create test namespace");

    // Create and apply the deployment object
    let k8s_object: DynamicObject = serde_json::from_value(create_busybox_deployment_json(
        "test-deployment",
        test_namespace,
    ))
    .unwrap();

    let objects = vec![k8s_object.clone()];

    // Create patch params
    let patch_params = PatchParams::apply("test-controller");

    // Apply the object
    let result = apply_k8s_objects(&objects, client.clone(), &patch_params).await;
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

    let result = apply_k8s_objects(&objects, client.clone(), &patch_params).await;
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
    
    // Test setup
    let (client, _discovery) = setup().await;
    
    // Create a test namespace using patch
    let ns_api = Api::<Namespace>::all(client.clone());
    let namespace = create_namespace_json(test_namespace);
    let ns: DynamicObject = serde_json::from_value(namespace).unwrap();
    
    let patch_params = PatchParams::apply("test-controller");
    ns_api
        .patch(test_namespace, &patch_params, &Patch::Apply(serde_json::to_value(&ns).unwrap()))
        .await
        .expect("Failed to create test namespace");

    // Create and apply the deployment object
    let k8s_object: DynamicObject = serde_json::from_value(create_busybox_deployment_json(
        "test-deployment",
        test_namespace,
    ))
    .unwrap();

    let objects = vec![k8s_object.clone()];
    
    // Apply the object
    let result = apply_k8s_objects(&objects, client.clone(), &patch_params).await;
    assert!(result.is_ok(), "Failed to apply k8s object: {:?}", result.err());

    // Wait for the deployment to be created
    tokio::time::sleep(std::time::Duration::from_secs(5)).await;

    // Delete the deployment
    let result = delete_k8s_objects(&objects, client.clone()).await;
    assert!(result.is_ok(), "Failed to delete k8s object: {:?}", result.err());

    // Verify the deployment is deleted
    let discovery = Discovery::new(client.clone())
        .run()
        .await
        .expect("Failed to create discovery client");

    if let Some((ar, caps)) = discovery.resolve_gvk(&GroupVersionKind::gvk("apps", "v1", "Deployment"))
    {
        let api = dynamic_api(ar, caps, client.clone(), Some(test_namespace), false);
        let deployment = api.get("test-deployment").await;
        assert!(deployment.is_err(), "Deployment still exists after deletion");
    }

    // Cleanup
    cleanup(&client, test_namespace).await;
}

#[tokio::test]
async fn test_delete_k8s_object_not_found() {
    let test_namespace = "test-delete-nonexistent";
    
    // Test setup
    let (client, _discovery) = setup().await;
    
    // Create a test namespace using patch
    let ns_api = Api::<Namespace>::all(client.clone());
    let namespace = create_namespace_json(test_namespace);
    let ns: DynamicObject = serde_json::from_value(namespace).unwrap();
    
    let patch_params = PatchParams::apply("test-controller");
    ns_api
        .patch(test_namespace, &patch_params, &Patch::Apply(serde_json::to_value(&ns).unwrap()))
        .await
        .expect("Failed to create test namespace");

    // Create a DynamicObject for a non-existent deployment
    let k8s_object: DynamicObject = serde_json::from_value(create_busybox_deployment_json(
        "nonexistent-deployment",
        test_namespace,
    ))
    .unwrap();

    let objects = vec![k8s_object];

    // Try to delete the non-existent deployment
    let result = delete_k8s_objects(&objects, client.clone()).await;
    assert!(result.is_err(), "Expected error when deleting non-existent object");

    // Cleanup
    cleanup(&client, test_namespace).await;
}

#[tokio::test]
async fn test_get_objects_by_annotation_found() {
    let test_namespace = "test-get-objects-annotation";
    
    // Test setup
    let (client, _discovery) = setup().await;
    
    // Create test namespace
    let ns_api = Api::<Namespace>::all(client.clone());
    let namespace = create_namespace_json(test_namespace);
    let ns: DynamicObject = serde_json::from_value(namespace).unwrap();
    
    let patch_params = PatchParams::apply("test-controller");
    ns_api
        .patch(test_namespace, &patch_params, &Patch::Apply(serde_json::to_value(&ns).unwrap()))
        .await
        .expect("Failed to create test namespace");

    // Create and apply deployment with annotation
    let mut k8s_object: DynamicObject = serde_json::from_value(create_busybox_deployment_json(
        "test-deployment-1",
        test_namespace,
    )).unwrap();
    
    let mut annotations = std::collections::BTreeMap::new();
    annotations.insert("brokkr.io/test-key".to_string(), "test-value".to_string());
    k8s_object.metadata.annotations = Some(annotations);

    let result = apply_k8s_objects(&vec![k8s_object], client.clone(), &patch_params).await;
    assert!(result.is_ok(), "Failed to apply k8s objects");

    // Get objects by annotation
    let result = get_all_objects_by_annotation(&client, "brokkr.io/test-key", "test-value").await;
    assert!(result.is_ok(), "Failed to get objects by annotation");
    
    let found_objects = result.unwrap();
    assert_eq!(found_objects.len(), 2, "Should find both namespace and deployment");

    // Cleanup
    cleanup(&client, test_namespace).await;
}

#[tokio::test]
async fn test_get_objects_by_annotation_not_found() {
    let test_namespace = "test-get-objects-no-annotation";
    
    // Test setup
    let (client, _discovery) = setup().await;
    
    // Create test namespace
    let ns_api = Api::<Namespace>::all(client.clone());
    let namespace = create_namespace_json(test_namespace);
    let ns: DynamicObject = serde_json::from_value(namespace).unwrap();
    
    let patch_params = PatchParams::apply("test-controller");
    ns_api
        .patch(test_namespace, &patch_params, &Patch::Apply(serde_json::to_value(&ns).unwrap()))
        .await
        .expect("Failed to create test namespace");

    // Create deployment object
    let  k8s_object: DynamicObject = serde_json::from_value(create_busybox_deployment_json(
        "test-deployment",
        test_namespace,
    )).unwrap();

    // Apply deployment
    let objects = vec![k8s_object];
    let result = apply_k8s_objects(&objects, client.clone(), &patch_params).await;
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

