use k8s_openapi::api::core::v1::Namespace;
use kube::{
    Api,
    Client as K8sClient, 
    Discovery,
};
use std::sync::Once;

static INIT: Once = Once::new();

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
    let namespace = serde_json::json!({
        "apiVersion": "v1",
        "kind": "Namespace",
        "metadata": {
            "name": test_namespace
        }
    });
    
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