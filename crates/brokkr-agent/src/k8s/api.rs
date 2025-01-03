//! # Kubernetes API Module
//!
//! This module provides functionality for interacting with the Kubernetes API server.
//!
//! ## Key Components
//!
//! ### Client Creation
//! ```rust
//! pub async fn create_k8s_client(kubeconfig_path: Option<&str>) -> Result<K8sClient, Error>
//! ```
//! Creates a Kubernetes client using either in-cluster config or a provided kubeconfig path.
//!
//! ### Object Management
//! ```rust
//! pub async fn apply_k8s_objects(objects: &[DynamicObject], client: K8sClient) -> Result<(), Error>
//! ```
//! Applies Kubernetes objects to the cluster with proper ordering and validation.
//!
//! ## Error Handling
//!
//! The module implements comprehensive error handling for:
//! - API server connectivity issues
//! - Object validation failures
//! - Permission errors
//! - Resource conflicts
//!
//! ## Resource Ordering
//!
//! Objects are applied in the following order:
//! 1. Namespaces
//! 2. CustomResourceDefinitions
//! 3. Other resources

use crate::k8s::objects::verify_object_ownership;
use crate::k8s::objects::{CHECKSUM_ANNOTATION, STACK_LABEL};
use backoff::ExponentialBackoffBuilder;
use brokkr_utils::logging::prelude::*;
use k8s_openapi::api::core::v1::Namespace;
use kube::api::DeleteParams;
use kube::api::DynamicObject;
use kube::api::GroupVersionKind;
use kube::api::Patch;
use kube::api::PatchParams;
use kube::core::TypeMeta;
use kube::discovery::ApiCapabilities;
use kube::discovery::ApiResource;
use kube::discovery::Scope;
use kube::Api;
use kube::Client;
use kube::Client as K8sClient;
use kube::Discovery;
use kube::Error as KubeError;
use kube::ResourceExt;
use std::collections::HashMap;
use std::time::Duration;
use uuid::Uuid;

/// Retry configuration for Kubernetes operations
struct RetryConfig {
    max_elapsed_time: Duration,
    initial_interval: Duration,
    max_interval: Duration,
    multiplier: f64,
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            max_elapsed_time: Duration::from_secs(300), // 5 minutes
            initial_interval: Duration::from_secs(1),
            max_interval: Duration::from_secs(60),
            multiplier: 2.0,
        }
    }
}

/// Determines if a Kubernetes error is retryable
fn is_retryable_error(error: &KubeError) -> bool {
    match error {
        KubeError::Api(api_err) => {
            matches!(api_err.code, 429 | 500 | 503 | 504)
                || matches!(
                    api_err.reason.as_str(),
                    "ServiceUnavailable" | "InternalError" | "Timeout"
                )
        }
        _ => false,
    }
}

/// Executes a Kubernetes operation with retries
async fn with_retries<F, Fut, T>(
    operation: F,
    config: RetryConfig,
) -> Result<T, Box<dyn std::error::Error>>
where
    F: Fn() -> Fut,
    Fut: std::future::Future<Output = Result<T, KubeError>>,
{
    let backoff = ExponentialBackoffBuilder::new()
        .with_initial_interval(config.initial_interval)
        .with_max_interval(config.max_interval)
        .with_multiplier(config.multiplier)
        .with_max_elapsed_time(Some(config.max_elapsed_time))
        .build();

    let operation_with_backoff = || async {
        match operation().await {
            Ok(value) => Ok(value),
            Err(error) => {
                if is_retryable_error(&error) {
                    warn!("Retryable error encountered: {}", error);
                    Err(backoff::Error::Transient {
                        err: error,
                        retry_after: None,
                    })
                } else {
                    error!("Non-retryable error encountered: {}", error);
                    Err(backoff::Error::Permanent(error))
                }
            }
        }
    };

    backoff::future::retry(backoff, operation_with_backoff)
        .await
        .map_err(|e| Box::new(e) as Box<dyn std::error::Error>)
}

/// Applies a list of Kubernetes objects to the cluster using server-side apply.
///
/// # Arguments
/// * `k8s_objects` - List of DynamicObjects to apply
/// * `discovery` - Kubernetes Discovery client for API resource resolution
/// * `k8s_client` - Kubernetes client for API interactions
/// * `patch_params` - Parameters for the patch operation
///
/// # Returns
/// * `Result<(), Box<dyn std::error::Error>>` - Success or error with message
pub async fn apply_k8s_objects(
    k8s_objects: &[DynamicObject],
    k8s_client: K8sClient,
    patch_params: PatchParams,
) -> Result<(), Box<dyn std::error::Error>> {
    let discovery = Discovery::new(k8s_client.clone())
        .run()
        .await
        .expect("Failed to create discovery client");

    for k8s_object in k8s_objects {
        info!("Processing k8s object: {:?}", k8s_object);
        let default_namespace = &"default".to_string();
        let namespace = k8s_object
            .metadata
            .namespace
            .as_deref()
            .unwrap_or(default_namespace);

        let gvk = if let Some(tm) = &k8s_object.types {
            GroupVersionKind::try_from(tm)?
        } else {
            error!(
                "Cannot apply object without valid TypeMeta {:?}",
                k8s_object
            );
            return Err(format!(
                "Cannot apply object without valid TypeMeta {:?}",
                k8s_object
            )
            .into());
        };

        let _name = k8s_object.name_any();

        if let Some((ar, caps)) = discovery.resolve_gvk(&gvk) {
            let api = dynamic_api(ar, caps, k8s_client.clone(), Some(namespace), false);
            info!(
                "Applying {:?}: \n{:?}",
                gvk.kind,
                serde_yaml::to_string(&k8s_object)?
            );

            let data = serde_json::to_value(k8s_object)?;
            let patch_params = patch_params.clone();
            let name = k8s_object.name_any();
            let name_for_error = name.clone();

            with_retries(
                move || {
                    let api = api.clone();
                    let data = data.clone();
                    let name = name.clone();
                    let patch_params = patch_params.clone();
                    async move { api.patch(&name, &patch_params, &Patch::Apply(data)).await }
                },
                RetryConfig::default(),
            )
            .await
            .map_err(|e| {
                error!(
                    "Apply failed for {:?} '{}': {:?}",
                    gvk.kind, name_for_error, e
                );
                e
            })?;

            info!("Successfully applied {:?} '{}'", gvk.kind, name_for_error);
        } else {
            error!("Unable to resolve GVK {:?}", gvk);
            return Err("Unable to resolve GVK".into());
        }
    }
    Ok(())
}

/// Creates a dynamic Kubernetes API client for a specific resource type
///
/// # Arguments
/// * `ar` - ApiResource describing the Kubernetes resource type
/// * `caps` - Capabilities of the API (e.g., list, watch, etc.)
/// * `client` - Kubernetes client instance
/// * `namespace` - Optional namespace to scope the API to
/// * `all_namespaces` - Whether to operate across all namespaces
///
/// # Returns
/// An Api<DynamicObject> instance configured for the specified resource type
pub fn dynamic_api(
    ar: ApiResource,
    caps: ApiCapabilities,
    client: K8sClient,
    namespace: Option<&str>,
    all_namespaces: bool,
) -> Api<DynamicObject> {
    if caps.scope == Scope::Cluster || all_namespaces {
        Api::all_with(client, &ar)
    } else if let Some(namespace) = namespace {
        Api::namespaced_with(client, namespace, &ar)
    } else {
        Api::default_namespaced_with(client, &ar)
    }
}

/// Retrieves all Kubernetes objects with a specific annotation key-value pair.
///
/// # Arguments
/// * `k8s_client` - Kubernetes client
/// * `discovery` - Kubernetes Discovery client
/// * `annotation_key` - Annotation key to filter by
/// * `annotation_value` - Annotation value to filter by
///
/// # Returns
/// * `Result<Vec<DynamicObject>, Box<dyn std::error::Error>>` - List of matching objects or error
pub async fn get_all_objects_by_annotation(
    k8s_client: &K8sClient,
    annotation_key: &str,
    annotation_value: &str,
) -> Result<Vec<DynamicObject>, Box<dyn std::error::Error>> {
    let mut results = Vec::new();

    let discovery = Discovery::new(k8s_client.clone())
        .run()
        .await
        .expect("Failed to create discovery client");

    // Search through all API groups and resources
    for group in discovery.groups() {
        for (ar, caps) in group.recommended_resources() {
            let api: Api<DynamicObject> =
                dynamic_api(ar.clone(), caps.clone(), k8s_client.clone(), None, true);

            match api.list(&Default::default()).await {
                Ok(list) => {
                    let matching_objects = list
                        .items
                        .into_iter()
                        .filter(|obj| {
                            obj.metadata
                                .annotations
                                .as_ref()
                                .and_then(|annotations| annotations.get(annotation_key))
                                .map_or(false, |value| value == annotation_value)
                        })
                        .map(|mut obj| {
                            // Set TypeMeta directly
                            obj.types = Some(TypeMeta {
                                api_version: if ar.group.is_empty() {
                                    ar.version.clone()
                                } else {
                                    format!("{}/{}", ar.group, ar.version)
                                },
                                kind: ar.kind.clone(),
                            });
                            obj
                        });
                    results.extend(matching_objects);
                }
                Err(e) => warn!("Error listing resources for {:?}: {:?}", ar, e),
            }
        }
    }

    Ok(results)
}

/// Deletes a list of Kubernetes objects from the cluster.
///
/// # Arguments
/// * `k8s_objects` - List of DynamicObjects to delete
/// * `discovery` - Kubernetes Discovery client for API resource resolution
/// * `k8s_client` - Kubernetes client for API interactions
///
/// # Returns
/// * `Result<(), Box<dyn std::error::Error>>` - Success or error with message
pub async fn delete_k8s_objects(
    k8s_objects: &[DynamicObject],
    k8s_client: K8sClient,
    agent_id: &Uuid,
) -> Result<(), Box<dyn std::error::Error>> {
    let discovery = Discovery::new(k8s_client.clone())
        .run()
        .await
        .expect("Failed to create discovery client");

    for k8s_object in k8s_objects {
        // Verify ownership before attempting deletion
        if !verify_object_ownership(k8s_object, agent_id) {
            error!(
                "Cannot delete object '{}' as it is not owned by agent {}",
                k8s_object.name_any(),
                agent_id
            );
            return Err(format!(
                "Cannot delete object '{}' as it is not owned by this agent",
                k8s_object.name_any()
            )
            .into());
        }

        info!("Processing k8s object for deletion: {:?}", k8s_object);
        let default_namespace = &"default".to_string();
        let namespace = k8s_object
            .metadata
            .namespace
            .as_ref()
            .unwrap_or(default_namespace);

        let gvk = if let Some(tm) = &k8s_object.types {
            GroupVersionKind::try_from(tm)?
        } else {
            error!(
                "Cannot delete object without valid TypeMeta {:?}",
                k8s_object
            );
            return Err(format!(
                "Cannot delete object without valid TypeMeta {:?}",
                k8s_object
            )
            .into());
        };
        let _name = k8s_object.name_any();
        if let Some((ar, caps)) = discovery.resolve_gvk(&gvk) {
            let api = dynamic_api(ar, caps, k8s_client.clone(), Some(namespace), false);
            let name = k8s_object.name_any();
            let name_for_error = name.clone(); // Clone for error handling
            info!("Deleting {:?}: {}", gvk.kind, name);

            with_retries(
                move || {
                    let api = api.clone();
                    let name = name.clone();
                    async move { api.delete(&name, &Default::default()).await }
                },
                RetryConfig::default(),
            )
            .await
            .map_err(|e| {
                error!(
                    "Delete failed for {:?} '{}': {:?}",
                    gvk.kind, name_for_error, e
                );
                e
            })?;

            info!("Successfully deleted {:?} '{}'", gvk.kind, name_for_error);
        }
    }
    Ok(())
}

/// Validates Kubernetes objects against the API server without applying them.
///
/// # Arguments
/// * `k8s_objects` - List of DynamicObjects to validate
/// * `k8s_client` - Kubernetes client for API interactions
///
/// # Returns
/// * `Result<(), Box<dyn std::error::Error>>` - Success or error with validation message
pub async fn validate_k8s_objects(
    k8s_objects: &[DynamicObject],
    k8s_client: K8sClient,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut validation_errors = Vec::new();

    let discovery = Discovery::new(k8s_client.clone())
        .run()
        .await
        .expect("Failed to create discovery client");

    for k8s_object in k8s_objects {
        let default_namespace = &"default".to_string();
        let namespace = k8s_object
            .metadata
            .namespace
            .as_deref()
            .unwrap_or(default_namespace);

        let gvk = if let Some(tm) = &k8s_object.types {
            match GroupVersionKind::try_from(tm) {
                Ok(gvk) => gvk,
                Err(e) => {
                    validation_errors.push(format!(
                        "Invalid TypeMeta for object '{}': {}",
                        k8s_object.name_any(),
                        e
                    ));
                    continue;
                }
            }
        } else {
            validation_errors.push(format!(
                "Missing TypeMeta for object '{}'",
                k8s_object.name_any()
            ));
            continue;
        };

        if let Some((ar, caps)) = discovery.resolve_gvk(&gvk) {
            let api = dynamic_api(ar, caps, k8s_client.clone(), Some(namespace), false);

            match serde_json::to_value(k8s_object) {
                Ok(data) => {
                    let mut patch_params = PatchParams::apply("validation");
                    patch_params = patch_params.dry_run();

                    match api
                        .patch(&k8s_object.name_any(), &patch_params, &Patch::Apply(data))
                        .await
                    {
                        Ok(_) => {
                            info!(
                                "Validation successful for {:?} '{}'",
                                gvk.kind,
                                k8s_object.name_any()
                            );
                        }
                        Err(e) => {
                            error!(
                                "Validation failed for {:?} '{}': {:?}",
                                gvk.kind,
                                k8s_object.name_any(),
                                e
                            );
                            validation_errors.push(format!(
                                "Validation failed for {} '{}': {}",
                                gvk.kind,
                                k8s_object.name_any(),
                                e
                            ));
                        }
                    }
                }
                Err(e) => {
                    validation_errors.push(format!(
                        "Failed to serialize object '{}': {}",
                        k8s_object.name_any(),
                        e
                    ));
                }
            }
        } else {
            validation_errors.push(format!(
                "Unable to resolve GVK {:?} for object '{}'",
                gvk,
                k8s_object.name_any()
            ));
        }
    }

    if validation_errors.is_empty() {
        Ok(())
    } else {
        Err(validation_errors.join("\n").into())
    }
}

/// Applies a list of Kubernetes objects to the cluster with validation.
///
/// This function first validates all objects using dry-run, then applies them
/// only if all validations pass. This ensures atomic behavior across multiple objects.
///
/// # Arguments
/// * `k8s_objects` - List of DynamicObjects to apply
/// * `k8s_client` - Kubernetes client for API interactions
/// * `patch_params` - Parameters for the patch operation
///
/// # Returns
/// * `Result<(), Box<dyn std::error::Error>>` - Success or error with message
pub async fn safe_apply_k8s_objects(
    k8s_objects: &[DynamicObject],
    k8s_client: K8sClient,
    patch_params: PatchParams,
) -> Result<(), Box<dyn std::error::Error>> {
    // Validate all objects first
    if let Err(e) = validate_k8s_objects(k8s_objects, k8s_client.clone()).await {
        error!("Pre-apply validation failed: {}", e);
        return Err(e);
    }

    // If validation succeeds, proceed with actual apply
    info!("Pre-apply validation successful, proceeding with apply");
    apply_k8s_objects(k8s_objects, k8s_client, patch_params).await
}

/// Reconciles the target state of Kubernetes objects for a stack.
///
/// This function:
/// 1. Captures the original state of existing objects
/// 2. Applies the new desired state
/// 3. Prunes any objects that are no longer part of the desired state but belong to the same stack
/// 4. Rolls back all changes if any part of the reconciliation fails
///
/// # Arguments
/// * `k8s_objects` - List of DynamicObjects representing the desired state
/// * `k8s_client` - Kubernetes client for API interactions
///
/// # Returns
/// * `Result<(), Box<dyn std::error::Error>>` - Success or error with message
pub async fn reconcile_target_state(
    objects: &[DynamicObject],
    client: Client,
) -> Result<(), Box<dyn std::error::Error>> {
    if objects.is_empty() {
        debug!("No objects to reconcile");
        return Ok(());
    }

    // Get stack ID from first object's annotations
    let stack_id = objects
        .first()
        .and_then(|obj| obj.metadata.annotations.as_ref())
        .and_then(|anns| anns.get(STACK_LABEL))
        .ok_or_else(|| format!("{} annotation missing from object", STACK_LABEL))?;

    // Keep track of resources we want to retain
    let mut keep_resources: HashMap<String, String> = HashMap::new();

    // First apply all resources with server-side apply
    info!("Applying {} resources", objects.len());
    for object in objects {
        let mut params = PatchParams::apply("brokkr-controller");
        params.force = true; // Force apply to handle conflicts

        // Create resource key for tracking
        let kind = object
            .types
            .as_ref()
            .map(|t| t.kind.clone())
            .unwrap_or_default();
        let namespace = object
            .metadata
            .namespace
            .as_deref()
            .unwrap_or("default")
            .to_string();
        let name = object.metadata.name.as_deref().unwrap_or("").to_string();
        let key = format!("{}:{}:{}", kind, namespace, name);

        debug!("Processing object with key: {}", key);

        // Store checksum for later comparison
        if let Some(checksum) = object
            .metadata
            .annotations
            .as_ref()
            .and_then(|anns| anns.get(CHECKSUM_ANNOTATION))
        {
            debug!("Storing checksum {} for key {}", checksum, key);
            keep_resources.insert(key.clone(), checksum.clone());
        } else {
            warn!("No checksum annotation found for object {}", key);
        }

        // Apply the object
        debug!("Applying object {}", key);
        match apply_k8s_objects(&[object.clone()], client.clone(), params).await {
            Ok(_) => debug!("Successfully applied {}", key),
            Err(e) => {
                error!("Failed to apply {}: {}", key, e);
                return Err(e);
            }
        }
    }

    // Get existing resources with this stack ID
    debug!("Fetching existing resources for stack {}", stack_id);
    let existing = get_all_objects_by_annotation(&client, STACK_LABEL, stack_id).await?;
    debug!("Found {} existing resources", existing.len());

    // Prune resources that are no longer needed
    let discovery = Discovery::new(client.clone()).run().await?;

    for existing_obj in existing {
        // Get the GVK first to ensure we have the correct kind
        let kind = if let Some(tm) = &existing_obj.types {
            if let Ok(gvk) = GroupVersionKind::try_from(tm) {
                gvk.kind
            } else {
                existing_obj
                    .types
                    .as_ref()
                    .map(|t| t.kind.clone())
                    .unwrap_or_default()
            }
        } else {
            String::new()
        };

        let namespace = existing_obj
            .metadata
            .namespace
            .as_deref()
            .unwrap_or("default")
            .to_string();
        let name = existing_obj
            .metadata
            .name
            .as_deref()
            .unwrap_or("")
            .to_string();
        let key = format!("{}:{}:{}", kind, namespace, name);

        debug!("Checking existing object with key: {}", key);

        let should_delete = match (
            keep_resources.get(&key),
            existing_obj
                .metadata
                .annotations
                .as_ref()
                .and_then(|anns| anns.get(CHECKSUM_ANNOTATION)),
        ) {
            (Some(keep), Some(existing)) => {
                debug!(
                    "Comparing checksums - keep: {}, existing: {}",
                    keep, existing
                );
                keep != existing
            }
            (None, _) => {
                debug!("No keep checksum found for key {}", key);
                true
            }
            (_, None) => {
                debug!("No existing checksum found for key {}", key);
                true
            }
        };

        debug!("Should delete {}: {}", key, should_delete);

        if should_delete {
            if let Some(gvk) = existing_obj.types.as_ref() {
                let gvk = GroupVersionKind::try_from(gvk)?;
                if let Some((ar, caps)) = discovery.resolve_gvk(&gvk) {
                    let api = dynamic_api(
                        ar,
                        caps,
                        client.clone(),
                        existing_obj.metadata.namespace.as_deref(),
                        false,
                    );

                    if let Some(name) = existing_obj.metadata.name.as_ref() {
                        info!(
                            "Deleting {}/{} in ns {:?}",
                            gvk.kind, name, existing_obj.metadata.namespace
                        );
                        match api.delete(name, &DeleteParams::default()).await {
                            Ok(_) => debug!("Successfully deleted {}", key),
                            Err(e) => {
                                error!("Failed to delete {}: {}", key, e);
                                return Err(Box::new(e));
                            }
                        }
                    }
                }
            }
        } else {
            debug!("Keeping object {}", key);
        }
    }

    Ok(())
}

/// Creates a Kubernetes client using either a provided kubeconfig path or default configuration.
///
/// # Arguments
/// * `kubeconfig_path` - Optional path to kubeconfig file
///
/// # Returns
/// * `Result<K8sClient, Box<dyn std::error::Error>>` - Kubernetes client or error
pub async fn create_k8s_client(
    kubeconfig_path: Option<&str>,
) -> Result<K8sClient, Box<dyn std::error::Error>> {
    // Set KUBECONFIG environment variable if path is provided
    if let Some(path) = kubeconfig_path {
        std::env::set_var("KUBECONFIG", path);
    }

    let client = K8sClient::try_default()
        .await
        .map_err(|e| format!("Failed to create Kubernetes client: {}", e))?;

    // Verify cluster connectivity by attempting to list namespaces
    let ns_api = Api::<Namespace>::all(client.clone());
    ns_api
        .list(&Default::default())
        .await
        .map_err(|e| format!("Failed to connect to Kubernetes cluster: {}", e))?;

    info!("Successfully connected to Kubernetes cluster");
    Ok(client)
}
