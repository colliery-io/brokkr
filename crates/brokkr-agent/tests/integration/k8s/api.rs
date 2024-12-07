use k8s_openapi::api::core::v1::Namespace;
use kube::{
    Api,
    Client as K8sClient, 
    Discovery,
};
use kube::api::{DynamicObject, PatchParams, GroupVersionKind};
use std::sync::Once;

use brokkr_agent::k8s::api::apply_k8s_objects;

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
    std::env::set_var(
        "KUBECONFIG",
        "/tmp/brokkr-keys/kubeconfig.yaml"
    );

    let client = K8sClient::try_default()
        .await
        .expect("Failed to create k8s client");
    
    // Verify cluster accessibility by attempting to list namespaces
    let ns_api = Api::<Namespace>::all(client.clone());
    ns_api.list(&Default::default())
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
    
    // Create a test namespace
    let ns_api = Api::<Namespace>::all(client.clone());
    let namespace =  create_namespace_json(test_namespace);
    
    let ns = serde_json::from_value(namespace).unwrap();
    ns_api.create(&Default::default(), &ns).await.expect("Failed to create test namespace");
    
    // Verify namespace exists
    let namespaces = ns_api.list(&Default::default()).await.expect("Failed to list namespaces");
    assert!(namespaces.items.iter().any(|ns| ns.metadata.name.as_ref().unwrap() == test_namespace));
    
    // Test cleanup
    cleanup(&client, test_namespace).await;
    
    // Verify namespace is either deleted or terminating
    let namespaces = ns_api.list(&Default::default()).await.expect("Failed to list namespaces");
    let namespace = namespaces.items.iter().find(|ns| ns.metadata.name.as_ref().unwrap() == test_namespace);
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
    let (client, discovery) = setup().await;
    
    // Create a test namespace first
    let ns_api = Api::<Namespace>::all(client.clone());
    let namespace = create_namespace_json(test_namespace);
    
    let ns = serde_json::from_value(namespace).unwrap();
    ns_api.create(&Default::default(), &ns).await.expect("Failed to create test namespace");
   
    // Create and apply the service object
    let k8s_object: DynamicObject = serde_json::from_value(create_busybox_deployment_json("test-service", test_namespace)).unwrap();
    let objects = vec![k8s_object];
    
    // Create patch params
    let patch_params = PatchParams::apply("test-controller");
    
    // Apply the object
    let result = apply_k8s_objects(&objects, &discovery, client.clone(), &patch_params).await;
    assert!(result.is_ok(), "Failed to apply k8s object: {:?}", result.err());
    
 
}

