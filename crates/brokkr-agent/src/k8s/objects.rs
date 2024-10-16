use kube::api::DynamicObject;
use serde_yaml;
use std::collections::BTreeMap;
use brokkr_models::models::deployment_objects::DeploymentObject;
use crate::utils;

pub static STACK_LABEL: &str = "k8s.brokkr.io/stack";
pub static CHECKSUM_ANNOTATION: &str = "k8s.brokkr.io/deployment-checksum";
pub static LAST_CONFIG_ANNOTATION: &str = "k8s.brokkr.io/last-config-applied";
pub static DEPLOYMENT_OBJECT_ID_LABEL: &str = "brokkr.io/deployment-object-id";

pub fn create_k8s_objects(deployment_object: DeploymentObject) -> Result<Vec<DynamicObject>, Box<dyn std::error::Error>> {
    let mut k8s_objects = Vec::new();
    
    let yaml_docs = utils::multidoc_deserialize(&deployment_object.yaml_content)?;

    for yaml_doc in yaml_docs {
        let mut obj: DynamicObject = serde_yaml::from_value(yaml_doc)?;
        let mut annotations = BTreeMap::new();
        annotations.insert(STACK_LABEL.to_string(), deployment_object.stack_id.to_string());
        annotations.insert(CHECKSUM_ANNOTATION.to_string(), deployment_object.yaml_checksum.to_string());
        annotations.insert(DEPLOYMENT_OBJECT_ID_LABEL.to_string(), deployment_object.id.to_string());
        annotations.insert(LAST_CONFIG_ANNOTATION.to_string(), format!("{:?}", obj));
        obj.metadata.annotations = Some(annotations);

        let kind = obj.types.as_ref().map(|t| t.kind.clone()).unwrap_or_default();

        // Move namespace and CRDs to the front of objects list for apply
        if kind == "Namespace" || kind == "CustomResourceDefinition" {
            k8s_objects.insert(0, obj);
        } else {
            k8s_objects.push(obj);
        }
    }

    Ok(k8s_objects)
}

