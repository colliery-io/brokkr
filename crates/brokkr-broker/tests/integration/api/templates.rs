/*
 * Copyright (c) 2025 Dylan Storey
 * Licensed under the Elastic License 2.0.
 * See LICENSE file in the project root for full license text.
 */

use axum::{
    body::{to_bytes, Body},
    http::{Request, StatusCode},
};
use serde_json::json;
use tower::ServiceExt;

use crate::fixtures::TestFixture;

const TEST_TEMPLATE_CONTENT: &str = r#"apiVersion: v1
kind: ConfigMap
metadata:
  name: {{ name }}
data:
  key: {{ value }}"#;

const TEST_PARAMETERS_SCHEMA: &str = r#"{
    "type": "object",
    "required": ["name", "value"],
    "properties": {
        "name": {"type": "string"},
        "value": {"type": "string"}
    }
}"#;

#[tokio::test]
async fn test_create_template() {
    let fixture = TestFixture::new();
    let app = fixture.create_test_router().with_state(fixture.dal.clone());
    let admin_pak = fixture.admin_pak.clone();

    let (_generator, _) = fixture.create_test_generator_with_pak("Test Generator".to_string(), None);

    let payload = json!({
        "name": "test-template",
        "description": "A test template",
        "template_content": TEST_TEMPLATE_CONTENT,
        "parameters_schema": TEST_PARAMETERS_SCHEMA
    });

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/templates")
                .header("Content-Type", "application/json")
                .header("Authorization", &admin_pak)
                .body(Body::from(serde_json::to_string(&payload).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::CREATED);

    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(json["name"], "test-template");
    assert_eq!(json["version"], 1);
    assert!(json["generator_id"].is_null()); // Admin creates system template
}

#[tokio::test]
async fn test_create_template_with_generator_pak() {
    let fixture = TestFixture::new();
    let app = fixture.create_test_router().with_state(fixture.dal.clone());

    let (generator, generator_pak) =
        fixture.create_test_generator_with_pak("Test Generator".to_string(), None);

    let payload = json!({
        "name": "generator-template",
        "description": "A generator template",
        "template_content": TEST_TEMPLATE_CONTENT,
        "parameters_schema": TEST_PARAMETERS_SCHEMA
    });

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/templates")
                .header("Content-Type", "application/json")
                .header("Authorization", format!("Bearer {}", generator_pak))
                .body(Body::from(serde_json::to_string(&payload).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::CREATED);

    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(json["name"], "generator-template");
    assert_eq!(json["generator_id"], generator.id.to_string());
}

#[tokio::test]
async fn test_create_template_invalid_tera_syntax() {
    let fixture = TestFixture::new();
    let app = fixture.create_test_router().with_state(fixture.dal.clone());
    let admin_pak = fixture.admin_pak.clone();

    let payload = json!({
        "name": "bad-template",
        "template_content": "{{ unclosed",
        "parameters_schema": "{}"
    });

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/templates")
                .header("Content-Type", "application/json")
                .header("Authorization", &admin_pak)
                .body(Body::from(serde_json::to_string(&payload).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_get_template() {
    let fixture = TestFixture::new();
    let app = fixture.create_test_router().with_state(fixture.dal.clone());
    let admin_pak = fixture.admin_pak.clone();

    let template = fixture.create_test_template(
        None,
        "test-template".to_string(),
        Some("Test description".to_string()),
        TEST_TEMPLATE_CONTENT.to_string(),
        TEST_PARAMETERS_SCHEMA.to_string(),
    );

    let response = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri(format!("/api/v1/templates/{}", template.id))
                .header("Authorization", &admin_pak)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(json["id"], template.id.to_string());
    assert_eq!(json["name"], "test-template");
}

#[tokio::test]
async fn test_list_templates() {
    let fixture = TestFixture::new();
    let app = fixture.create_test_router().with_state(fixture.dal.clone());
    let admin_pak = fixture.admin_pak.clone();

    fixture.create_test_template(
        None,
        "template-1".to_string(),
        None,
        TEST_TEMPLATE_CONTENT.to_string(),
        "{}".to_string(),
    );
    fixture.create_test_template(
        None,
        "template-2".to_string(),
        None,
        TEST_TEMPLATE_CONTENT.to_string(),
        "{}".to_string(),
    );

    let response = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/api/v1/templates")
                .header("Authorization", &admin_pak)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();

    assert!(json.as_array().unwrap().len() >= 2);
}

#[tokio::test]
async fn test_update_template_creates_new_version() {
    let fixture = TestFixture::new();
    let app = fixture.create_test_router().with_state(fixture.dal.clone());
    let admin_pak = fixture.admin_pak.clone();

    let template = fixture.create_test_template(
        None,
        "versioned-template".to_string(),
        Some("Version 1".to_string()),
        TEST_TEMPLATE_CONTENT.to_string(),
        TEST_PARAMETERS_SCHEMA.to_string(),
    );

    let update_payload = json!({
        "description": "Version 2",
        "template_content": "apiVersion: v2\nkind: ConfigMap\nmetadata:\n  name: {{ name }}",
        "parameters_schema": TEST_PARAMETERS_SCHEMA
    });

    let response = app
        .oneshot(
            Request::builder()
                .method("PUT")
                .uri(format!("/api/v1/templates/{}", template.id))
                .header("Content-Type", "application/json")
                .header("Authorization", &admin_pak)
                .body(Body::from(serde_json::to_string(&update_payload).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();

    // Should be a new version
    assert_eq!(json["version"], 2);
    assert_eq!(json["description"], "Version 2");
    // Should have a different ID (new record)
    assert_ne!(json["id"], template.id.to_string());
}

#[tokio::test]
async fn test_delete_template() {
    let fixture = TestFixture::new();
    let app = fixture.create_test_router().with_state(fixture.dal.clone());
    let admin_pak = fixture.admin_pak.clone();

    let template = fixture.create_test_template(
        None,
        "delete-me".to_string(),
        None,
        TEST_TEMPLATE_CONTENT.to_string(),
        "{}".to_string(),
    );

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("DELETE")
                .uri(format!("/api/v1/templates/{}", template.id))
                .header("Authorization", &admin_pak)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::NO_CONTENT);

    // Verify it's gone
    let response = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri(format!("/api/v1/templates/{}", template.id))
                .header("Authorization", &admin_pak)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_add_template_label() {
    let fixture = TestFixture::new();
    let app = fixture.create_test_router().with_state(fixture.dal.clone());
    let admin_pak = fixture.admin_pak.clone();

    let template = fixture.create_test_template(
        None,
        "labeled-template".to_string(),
        None,
        TEST_TEMPLATE_CONTENT.to_string(),
        "{}".to_string(),
    );

    let label = "env=prod".to_string();

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!("/api/v1/templates/{}/labels", template.id))
                .header("Content-Type", "application/json")
                .header("Authorization", &admin_pak)
                .body(Body::from(serde_json::to_string(&label).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(json["label"], "env=prod");
}

#[tokio::test]
async fn test_list_template_labels() {
    let fixture = TestFixture::new();
    let app = fixture.create_test_router().with_state(fixture.dal.clone());
    let admin_pak = fixture.admin_pak.clone();

    let template = fixture.create_test_template(
        None,
        "multi-label-template".to_string(),
        None,
        TEST_TEMPLATE_CONTENT.to_string(),
        "{}".to_string(),
    );

    fixture.create_test_template_label(template.id, "label1".to_string());
    fixture.create_test_template_label(template.id, "label2".to_string());

    let response = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri(format!("/api/v1/templates/{}/labels", template.id))
                .header("Authorization", &admin_pak)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(json.as_array().unwrap().len(), 2);
}

#[tokio::test]
async fn test_remove_template_label() {
    let fixture = TestFixture::new();
    let app = fixture.create_test_router().with_state(fixture.dal.clone());
    let admin_pak = fixture.admin_pak.clone();

    let template = fixture.create_test_template(
        None,
        "removable-label-template".to_string(),
        None,
        TEST_TEMPLATE_CONTENT.to_string(),
        "{}".to_string(),
    );

    fixture.create_test_template_label(template.id, "to-remove".to_string());

    let response = app
        .oneshot(
            Request::builder()
                .method("DELETE")
                .uri(format!("/api/v1/templates/{}/labels/to-remove", template.id))
                .header("Authorization", &admin_pak)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::NO_CONTENT);
}

#[tokio::test]
async fn test_add_template_annotation() {
    let fixture = TestFixture::new();
    let app = fixture.create_test_router().with_state(fixture.dal.clone());
    let admin_pak = fixture.admin_pak.clone();

    let template = fixture.create_test_template(
        None,
        "annotated-template".to_string(),
        None,
        TEST_TEMPLATE_CONTENT.to_string(),
        "{}".to_string(),
    );

    let annotation = json!({
        "key": "region",
        "value": "us-east"
    });

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!("/api/v1/templates/{}/annotations", template.id))
                .header("Content-Type", "application/json")
                .header("Authorization", &admin_pak)
                .body(Body::from(serde_json::to_string(&annotation).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(json["key"], "region");
    assert_eq!(json["value"], "us-east");
}

#[tokio::test]
async fn test_list_template_annotations() {
    let fixture = TestFixture::new();
    let app = fixture.create_test_router().with_state(fixture.dal.clone());
    let admin_pak = fixture.admin_pak.clone();

    let template = fixture.create_test_template(
        None,
        "multi-annotation-template".to_string(),
        None,
        TEST_TEMPLATE_CONTENT.to_string(),
        "{}".to_string(),
    );

    fixture.create_test_template_annotation(template.id, "key1", "value1");
    fixture.create_test_template_annotation(template.id, "key2", "value2");

    let response = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri(format!("/api/v1/templates/{}/annotations", template.id))
                .header("Authorization", &admin_pak)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(json.as_array().unwrap().len(), 2);
}

#[tokio::test]
async fn test_remove_template_annotation() {
    let fixture = TestFixture::new();
    let app = fixture.create_test_router().with_state(fixture.dal.clone());
    let admin_pak = fixture.admin_pak.clone();

    let template = fixture.create_test_template(
        None,
        "removable-annotation-template".to_string(),
        None,
        TEST_TEMPLATE_CONTENT.to_string(),
        "{}".to_string(),
    );

    fixture.create_test_template_annotation(template.id, "to-remove", "value");

    let response = app
        .oneshot(
            Request::builder()
                .method("DELETE")
                .uri(format!(
                    "/api/v1/templates/{}/annotations/to-remove",
                    template.id
                ))
                .header("Authorization", &admin_pak)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::NO_CONTENT);
}

#[tokio::test]
async fn test_instantiate_template() {
    let fixture = TestFixture::new();
    let app = fixture.create_test_router().with_state(fixture.dal.clone());
    let admin_pak = fixture.admin_pak.clone();

    let (generator, _) = fixture.create_test_generator_with_pak("Test Generator".to_string(), None);
    let stack = fixture.create_test_stack("test-stack".to_string(), None, generator.id);

    let template = fixture.create_test_template(
        None,
        "instantiable-template".to_string(),
        None,
        TEST_TEMPLATE_CONTENT.to_string(),
        TEST_PARAMETERS_SCHEMA.to_string(),
    );

    let instantiation_request = json!({
        "template_id": template.id,
        "parameters": {
            "name": "my-config",
            "value": "my-value"
        }
    });

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!(
                    "/api/v1/stacks/{}/deployment-objects/from-template",
                    stack.id
                ))
                .header("Content-Type", "application/json")
                .header("Authorization", &admin_pak)
                .body(Body::from(
                    serde_json::to_string(&instantiation_request).unwrap(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::CREATED);

    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(json["stack_id"], stack.id.to_string());
    // Check that the template was rendered
    let yaml_content = json["yaml_content"].as_str().unwrap();
    assert!(yaml_content.contains("name: my-config"));
    assert!(yaml_content.contains("key: my-value"));
}

#[tokio::test]
async fn test_instantiate_template_invalid_parameters() {
    let fixture = TestFixture::new();
    let app = fixture.create_test_router().with_state(fixture.dal.clone());
    let admin_pak = fixture.admin_pak.clone();

    let (generator, _) = fixture.create_test_generator_with_pak("Test Generator".to_string(), None);
    let stack = fixture.create_test_stack("test-stack".to_string(), None, generator.id);

    let template = fixture.create_test_template(
        None,
        "strict-template".to_string(),
        None,
        TEST_TEMPLATE_CONTENT.to_string(),
        TEST_PARAMETERS_SCHEMA.to_string(),
    );

    // Missing required "value" parameter
    let instantiation_request = json!({
        "template_id": template.id,
        "parameters": {
            "name": "my-config"
        }
    });

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!(
                    "/api/v1/stacks/{}/deployment-objects/from-template",
                    stack.id
                ))
                .header("Content-Type", "application/json")
                .header("Authorization", &admin_pak)
                .body(Body::from(
                    serde_json::to_string(&instantiation_request).unwrap(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);

    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();

    assert!(json["error"].as_str().unwrap().contains("Invalid parameters"));
}

#[tokio::test]
async fn test_instantiate_template_label_mismatch() {
    let fixture = TestFixture::new();
    let app = fixture.create_test_router().with_state(fixture.dal.clone());
    let admin_pak = fixture.admin_pak.clone();

    let (generator, _) = fixture.create_test_generator_with_pak("Test Generator".to_string(), None);
    let stack = fixture.create_test_stack("test-stack".to_string(), None, generator.id);
    // Stack has no labels

    let template = fixture.create_test_template(
        None,
        "labeled-template".to_string(),
        None,
        TEST_TEMPLATE_CONTENT.to_string(),
        "{}".to_string(),
    );
    // Add a label to the template that the stack doesn't have
    fixture.create_test_template_label(template.id, "env=prod".to_string());

    let instantiation_request = json!({
        "template_id": template.id,
        "parameters": {}
    });

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!(
                    "/api/v1/stacks/{}/deployment-objects/from-template",
                    stack.id
                ))
                .header("Content-Type", "application/json")
                .header("Authorization", &admin_pak)
                .body(Body::from(
                    serde_json::to_string(&instantiation_request).unwrap(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);
}

#[tokio::test]
async fn test_instantiate_template_with_matching_labels() {
    let fixture = TestFixture::new();
    let app = fixture.create_test_router().with_state(fixture.dal.clone());
    let admin_pak = fixture.admin_pak.clone();

    let (generator, _) = fixture.create_test_generator_with_pak("Test Generator".to_string(), None);
    let stack = fixture.create_test_stack("test-stack".to_string(), None, generator.id);
    fixture.create_test_stack_label(stack.id, "env=prod".to_string());

    let template = fixture.create_test_template(
        None,
        "labeled-template".to_string(),
        None,
        "name: {{ name | default(value=\"default\") }}".to_string(),
        "{}".to_string(),
    );
    fixture.create_test_template_label(template.id, "env=prod".to_string());

    let instantiation_request = json!({
        "template_id": template.id,
        "parameters": {}
    });

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!(
                    "/api/v1/stacks/{}/deployment-objects/from-template",
                    stack.id
                ))
                .header("Content-Type", "application/json")
                .header("Authorization", &admin_pak)
                .body(Body::from(
                    serde_json::to_string(&instantiation_request).unwrap(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::CREATED);
}

#[tokio::test]
async fn test_generator_cannot_access_other_generator_template() {
    let fixture = TestFixture::new();
    let app = fixture.create_test_router().with_state(fixture.dal.clone());

    let (generator1, _) =
        fixture.create_test_generator_with_pak("Generator 1".to_string(), None);
    let (_generator2, generator2_pak) =
        fixture.create_test_generator_with_pak("Generator 2".to_string(), None);

    // Create template owned by generator1
    let template = fixture.create_test_template(
        Some(generator1.id),
        "generator1-template".to_string(),
        None,
        TEST_TEMPLATE_CONTENT.to_string(),
        "{}".to_string(),
    );

    // Try to access with generator2's PAK
    let response = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri(format!("/api/v1/templates/{}", template.id))
                .header("Authorization", format!("Bearer {}", generator2_pak))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::FORBIDDEN);
}
