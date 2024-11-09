use brokkr_agent::k8s::objects::{self, STACK_LABEL, CHECKSUM_ANNOTATION, DEPLOYMENT_OBJECT_ID_LABEL, LAST_CONFIG_ANNOTATION};
use brokkr_models::models::deployment_objects::DeploymentObject;
use uuid::Uuid;
use chrono::Utc;

#[test]
fn test_create_k8s_objects_single_document() {
    let yaml_content = r#"
apiVersion: v1
kind: Namespace
metadata:
  name: test-namespace
"#;

    let deployment_object = DeploymentObject {
        id: Uuid::new_v4(),
        stack_id: Uuid::new_v4(),
        yaml_content: yaml_content.to_string(),
        yaml_checksum: "test-checksum".to_string(),
        is_deletion_marker: false,
        created_at: Utc::now(),
        updated_at: Utc::now(),
        deleted_at: None,
        sequence_id: 1,
        submitted_at: Utc::now(),
    };

    let result = objects::create_k8s_objects(deployment_object);
    assert!(result.is_ok());

    let k8s_objects = result.unwrap();
    assert_eq!(k8s_objects.len(), 1);
    
    let obj = &k8s_objects[0];
    assert_eq!(obj.types.as_ref().unwrap().kind, "Namespace");
    
    // Verify annotations
    let annotations = obj.metadata.annotations.as_ref().unwrap();
    assert!(annotations.contains_key(STACK_LABEL));
    assert!(annotations.contains_key(CHECKSUM_ANNOTATION));
    assert!(annotations.contains_key(DEPLOYMENT_OBJECT_ID_LABEL));
    assert!(annotations.contains_key(LAST_CONFIG_ANNOTATION));
}

#[test]
fn test_create_k8s_objects_multiple_documents() {
    let yaml_content = r#"
apiVersion: v1
kind: Namespace
metadata:
  name: test-namespace
---
apiVersion: v1
kind: ConfigMap
metadata:
  name: test-config
  namespace: test-namespace
data:
  key: value
---
apiVersion: apps/v1
kind: Deployment
metadata:
  name: test-deployment
  namespace: test-namespace
spec:
  replicas: 1
"#;

    let deployment_object = DeploymentObject {
        id: Uuid::new_v4(),
        stack_id: Uuid::new_v4(),
        yaml_content: yaml_content.to_string(),
        yaml_checksum: "test-checksum".to_string(),
        is_deletion_marker: false,
        created_at: Utc::now(),
        updated_at: Utc::now(),
        deleted_at: None,
        sequence_id: 1,
        submitted_at: Utc::now(),
    };

    let result = objects::create_k8s_objects(deployment_object);
    assert!(result.is_ok());

    let k8s_objects = result.unwrap();
    assert_eq!(k8s_objects.len(), 3);
    
    // Verify Namespace is first
    assert_eq!(k8s_objects[0].types.as_ref().unwrap().kind, "Namespace");
    
    // Verify all objects have required annotations
    for obj in k8s_objects {
        let annotations = obj.metadata.annotations.as_ref().unwrap();
        assert!(annotations.contains_key(STACK_LABEL));
        assert!(annotations.contains_key(CHECKSUM_ANNOTATION));
        assert!(annotations.contains_key(DEPLOYMENT_OBJECT_ID_LABEL));
        assert!(annotations.contains_key(LAST_CONFIG_ANNOTATION));
    }
}

#[test]
fn test_create_k8s_objects_with_crds() {
    let yaml_content = r#"
apiVersion: apiextensions.k8s.io/v1
kind: CustomResourceDefinition
metadata:
  name: test.example.com
spec:
  group: example.com
  names:
    kind: Test
    plural: tests
  scope: Namespaced
  versions:
    - name: v1
      served: true
      storage: true
---
apiVersion: v1
kind: ConfigMap
metadata:
  name: test-config
"#;

    let deployment_object = DeploymentObject {
        id: Uuid::new_v4(),
        stack_id: Uuid::new_v4(),
        yaml_content: yaml_content.to_string(),
        yaml_checksum: "test-checksum".to_string(),
        is_deletion_marker: false,
        created_at: Utc::now(),
        updated_at: Utc::now(),
        deleted_at: None,
        sequence_id: 1,
        submitted_at: Utc::now(),
    };

    let result = objects::create_k8s_objects(deployment_object);
    assert!(result.is_ok());

    let k8s_objects = result.unwrap();
    assert_eq!(k8s_objects.len(), 2);
    
    // Verify CRD is first
    assert_eq!(k8s_objects[0].types.as_ref().unwrap().kind, "CustomResourceDefinition");
}

#[test]
fn test_create_k8s_objects_invalid_yaml() {
    let yaml_content = r#"
invalid: [
  this is not valid yaml
  missing: quotes
  broken indentation
    nested: {
      unclosed bracket
"#;

    let deployment_object = DeploymentObject {
        id: Uuid::new_v4(),
        stack_id: Uuid::new_v4(),
        yaml_content: yaml_content.to_string(),
        yaml_checksum: "test-checksum".to_string(),
        is_deletion_marker: false,
        created_at: Utc::now(),
        updated_at: Utc::now(),
        deleted_at: None,
        sequence_id: 1,
        submitted_at: Utc::now(),
    };

    let result = objects::create_k8s_objects(deployment_object);
    assert!(result.is_err());
}

#[test]
fn test_create_k8s_objects_empty_yaml() {
    let deployment_object = DeploymentObject {
        id: Uuid::new_v4(),
        stack_id: Uuid::new_v4(),
        yaml_content: "".to_string(),
        yaml_checksum: "test-checksum".to_string(),
        is_deletion_marker: false,
        created_at: Utc::now(),
        updated_at: Utc::now(),
        deleted_at: None,
        sequence_id: 1,
        submitted_at: Utc::now(),
    };

    let result = objects::create_k8s_objects(deployment_object);
    assert!(result.is_ok());
    assert_eq!(result.unwrap().len(), 0);
}

#[test]
fn test_create_k8s_objects_ordering() {
    let yaml_content = r#"
apiVersion: apps/v1
kind: Deployment
metadata:
  name: test-deployment
---
apiVersion: v1
kind: Namespace
metadata:
  name: test-namespace
---
apiVersion: apiextensions.k8s.io/v1
kind: CustomResourceDefinition
metadata:
  name: test.example.com
---
apiVersion: v1
kind: ConfigMap
metadata:
  name: test-config
"#;

    let deployment_object = DeploymentObject {
        id: Uuid::new_v4(),
        stack_id: Uuid::new_v4(),
        yaml_content: yaml_content.to_string(),
        yaml_checksum: "test-checksum".to_string(),
        is_deletion_marker: false,
        created_at: Utc::now(),
        updated_at: Utc::now(),
        deleted_at: None,
        sequence_id: 1,
        submitted_at: Utc::now(),
    };

    let result = objects::create_k8s_objects(deployment_object);
    assert!(result.is_ok());

    let k8s_objects = result.unwrap();
    assert_eq!(k8s_objects.len(), 4);
    
    // Verify ordering: Namespace and CRD should be first
    assert!(matches!(
        k8s_objects[0].types.as_ref().unwrap().kind.as_str(),
        "Namespace" | "CustomResourceDefinition"
    ));
    assert!(matches!(
        k8s_objects[1].types.as_ref().unwrap().kind.as_str(),
        "Namespace" | "CustomResourceDefinition"
    ));
}
