use kube::api::DynamicObject;
use kube::Discovery;
use kube::Client as K8sClient;
use kube::api::PatchParams;
use brokkr_utils::logging::prelude::*;
use kube::api::GroupVersionKind;
use kube::ResourceExt;
use kube::api::Patch;
use kube::discovery::ApiResource;
use kube::discovery::ApiCapabilities;
use kube::Api;
use kube::discovery::Scope;
use kube::api::ListParams;

pub async fn apply_k8s_objects(
    k8s_objects: &[DynamicObject],
    discovery: &Discovery,
    k8s_client: K8sClient,
    patch_params: &PatchParams,
) -> Result<(), Box<dyn std::error::Error>> {
    for k8s_object in k8s_objects {
        info!("Processing k8s object: {:?}", k8s_object);
        let default_namespace = &"default".to_string();
        let namespace = k8s_object.metadata.namespace.as_ref().or(Some(default_namespace)).unwrap();

        let gvk = if let Some(tm) = &k8s_object.types {
            GroupVersionKind::try_from(tm)?
        } else {
            error!("Cannot apply object without valid TypeMeta {:?}", k8s_object);
            return Err(format!("Cannot apply object without valid TypeMeta {:?}", k8s_object).into());
        };
        let name = k8s_object.name_any();
        if let Some((ar, caps)) = discovery.resolve_gvk(&gvk) {
            let api = dynamic_api(ar, caps, k8s_client.clone(), Some(namespace), false);
            info!("Apply {:?}: \n{:?}", gvk.kind, serde_yaml::to_string(&k8s_object));
            let data = serde_json::to_value(&k8s_object)?;
            match api.patch(&name, patch_params, &Patch::Apply(data)).await {
                Ok(_) => {
                    info!("Apply successful for {:?} '{}'", gvk.kind, name);
                },
                Err(e) => {
                    error!("Apply failed for {:?} '{}': {:?}", gvk.kind, name, e);
                    // TODO: register failed apply event
                    return Err(Box::new(e));
                }
            }
        }
    }
    Ok(())
}

fn dynamic_api(
    ar: ApiResource,
    caps: ApiCapabilities,
    client: K8sClient,
    ns: Option<&str>,
    all: bool,
) -> Api<DynamicObject> {
    if caps.scope == Scope::Cluster || all {
        Api::all_with(client, &ar)
    } else if let Some(namespace) = ns {
        Api::namespaced_with(client, namespace, &ar)
    } else {
        Api::default_namespaced_with(client, &ar)
    }
}

pub async fn get_all_objects_by_annotation(
    k8s_client: &K8sClient,
    discovery: &Discovery,
    annotation_key: &str,
    annotation_value: &str,
) -> Result<Vec<DynamicObject>, Box<dyn std::error::Error>> {
    let mut results = Vec::new();

    for group in discovery.groups() {
        for (ar, caps) in group.recommended_resources() {
            let api: Api<DynamicObject> = dynamic_api(ar.clone(), caps.clone(), k8s_client.clone(), None, true);
            let lp = ListParams::default().labels(&format!("{}={}", annotation_key, annotation_value));
            
            match api.list(&lp).await {
                Ok(list) => results.extend(list.items),
                Err(e) => warn!("Error listing resources for {:?}: {:?}", ar, e),
            }
        }
    }

    Ok(results)
}

pub async fn delete_k8s_objects(
    k8s_objects: &[DynamicObject],
    discovery: &Discovery,
    k8s_client: K8sClient,
) -> Result<(), Box<dyn std::error::Error>> {
    for k8s_object in k8s_objects {
        info!("Processing k8s object for deletion: {:?}", k8s_object);
        let default_namespace = &"default".to_string();
        let namespace = k8s_object.metadata.namespace.as_ref().or(Some(default_namespace)).unwrap();

        let gvk = if let Some(tm) = &k8s_object.types {
            GroupVersionKind::try_from(tm)?
        } else {
            error!("Cannot delete object without valid TypeMeta {:?}", k8s_object);
            return Err(format!("Cannot delete object without valid TypeMeta {:?}", k8s_object).into());
        };
        let name = k8s_object.name_any();
        if let Some((ar, caps)) = discovery.resolve_gvk(&gvk) {
            let api = dynamic_api(ar, caps, k8s_client.clone(), Some(namespace), false);
            info!("Deleting {:?}: {}", gvk.kind, name);
            match api.delete(&name, &Default::default()).await {
                Ok(_) => {
                    info!("Delete successful for {:?} '{}'", gvk.kind, name);
                },
                Err(e) => {
                    error!("Delete failed for {:?} '{}': {:?}", gvk.kind, name, e);
                    // TODO: register failed delete event
                    return Err(Box::new(e));
                }
            }
        }
    }
    Ok(())
}

