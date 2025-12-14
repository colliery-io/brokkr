/*
 * Copyright (c) 2025 Dylan Storey
 * Licensed under the Elastic License 2.0.
 * See LICENSE file in the project root for full license text.
 */

//! Tera template rendering and JSON Schema validation utilities.
//!
//! This module provides functions for:
//! - Validating Tera template syntax at creation time
//! - Rendering Tera templates with parameters at instantiation time
//! - Validating JSON Schema definitions at creation time
//! - Validating parameters against JSON Schema at instantiation time

use jsonschema::JSONSchema;
use serde_json::Value;
use tera::{Context, Tera};

/// Error type for templating operations.
#[derive(Debug, Clone)]
pub struct TemplateError {
    pub message: String,
    pub details: Option<String>,
}

impl std::fmt::Display for TemplateError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.details {
            Some(details) => write!(f, "{}: {}", self.message, details),
            None => write!(f, "{}", self.message),
        }
    }
}

impl std::error::Error for TemplateError {}

/// Validate Tera template syntax without rendering.
///
/// Called at template creation time to fail fast on invalid templates.
/// Does not require actual parameter values - only checks syntax.
///
/// # Arguments
///
/// * `template_content` - The Tera template string to validate
///
/// # Returns
///
/// * `Ok(())` if the template syntax is valid
/// * `Err(TemplateError)` with details about the syntax error
///
/// # Example
///
/// ```
/// use brokkr_broker::utils::templating::validate_tera_syntax;
///
/// // Valid template
/// assert!(validate_tera_syntax("Hello, {{ name }}!").is_ok());
///
/// // Invalid template - unclosed brace
/// assert!(validate_tera_syntax("Hello, {{ name !").is_err());
/// ```
pub fn validate_tera_syntax(template_content: &str) -> Result<(), TemplateError> {
    let mut tera = Tera::default();

    // Try to add the template - this validates syntax
    tera.add_raw_template("__validation__", template_content)
        .map_err(|e| TemplateError {
            message: "Invalid Tera syntax".to_string(),
            details: Some(e.to_string()),
        })?;

    Ok(())
}

/// Render a Tera template with the provided parameters.
///
/// Called at template instantiation time to produce the final output.
///
/// # Arguments
///
/// * `template_content` - The Tera template string to render
/// * `parameters` - JSON object containing parameter values
///
/// # Returns
///
/// * `Ok(String)` with the rendered output
/// * `Err(TemplateError)` if rendering fails (e.g., missing required variable)
///
/// # Example
///
/// ```
/// use brokkr_broker::utils::templating::render_template;
/// use serde_json::json;
///
/// let template = "name: {{ name }}\nreplicas: {{ replicas }}";
/// let params = json!({"name": "my-app", "replicas": 3});
///
/// let result = render_template(template, &params).unwrap();
/// assert!(result.contains("name: my-app"));
/// ```
pub fn render_template(template_content: &str, parameters: &Value) -> Result<String, TemplateError> {
    let mut tera = Tera::default();

    tera.add_raw_template("template", template_content)
        .map_err(|e| TemplateError {
            message: "Template parse error".to_string(),
            details: Some(e.to_string()),
        })?;

    let mut context = Context::new();

    // Flatten JSON parameters into Tera context
    if let Value::Object(map) = parameters {
        for (key, value) in map {
            context.insert(key, value);
        }
    }

    tera.render("template", &context).map_err(|e| TemplateError {
        message: "Template rendering failed".to_string(),
        details: Some(e.to_string()),
    })
}

/// Validate that a string is a valid JSON Schema.
///
/// Called at template creation time to ensure the schema is valid.
///
/// # Arguments
///
/// * `schema_str` - The JSON Schema as a string
///
/// # Returns
///
/// * `Ok(())` if the schema is valid
/// * `Err(TemplateError)` with details about the validation error
///
/// # Example
///
/// ```
/// use brokkr_broker::utils::templating::validate_json_schema;
///
/// let schema = r#"{"type": "object", "properties": {"name": {"type": "string"}}}"#;
/// assert!(validate_json_schema(schema).is_ok());
///
/// // Invalid JSON
/// assert!(validate_json_schema("not json").is_err());
/// ```
pub fn validate_json_schema(schema_str: &str) -> Result<(), TemplateError> {
    let schema: Value = serde_json::from_str(schema_str).map_err(|e| TemplateError {
        message: "Invalid JSON".to_string(),
        details: Some(e.to_string()),
    })?;

    JSONSchema::compile(&schema).map_err(|e| TemplateError {
        message: "Invalid JSON Schema".to_string(),
        details: Some(e.to_string()),
    })?;

    Ok(())
}

/// Validation error details for parameter validation.
#[derive(Debug, Clone)]
pub struct ParameterValidationError {
    pub path: String,
    pub message: String,
}

impl std::fmt::Display for ParameterValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.path.is_empty() {
            write!(f, "{}", self.message)
        } else {
            write!(f, "{}: {}", self.path, self.message)
        }
    }
}

/// Validate parameters against a JSON Schema.
///
/// Called at template instantiation time to ensure parameters match the schema.
///
/// # Arguments
///
/// * `schema_str` - The JSON Schema as a string
/// * `parameters` - The parameters to validate
///
/// # Returns
///
/// * `Ok(())` if parameters match the schema
/// * `Err(Vec<ParameterValidationError>)` with all validation errors
///
/// # Example
///
/// ```
/// use brokkr_broker::utils::templating::validate_parameters;
/// use serde_json::json;
///
/// let schema = r#"{"type": "object", "required": ["name"], "properties": {"name": {"type": "string"}}}"#;
///
/// // Valid parameters
/// let params = json!({"name": "test"});
/// assert!(validate_parameters(schema, &params).is_ok());
///
/// // Missing required field
/// let params = json!({});
/// assert!(validate_parameters(schema, &params).is_err());
/// ```
pub fn validate_parameters(
    schema_str: &str,
    parameters: &Value,
) -> Result<(), Vec<ParameterValidationError>> {
    let schema: Value = serde_json::from_str(schema_str).map_err(|e| {
        vec![ParameterValidationError {
            path: String::new(),
            message: format!("Invalid schema JSON: {}", e),
        }]
    })?;

    let compiled = JSONSchema::compile(&schema).map_err(|e| {
        vec![ParameterValidationError {
            path: String::new(),
            message: format!("Invalid schema: {}", e),
        }]
    })?;

    if !compiled.is_valid(parameters) {
        let errors: Vec<ParameterValidationError> = compiled
            .validate(parameters)
            .err()
            .map(|iter| {
                iter.map(|e| ParameterValidationError {
                    path: e.instance_path.to_string(),
                    message: e.to_string(),
                })
                .collect()
            })
            .unwrap_or_default();

        return Err(errors);
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    // ==================== Tera Syntax Validation Tests ====================

    #[test]
    fn test_valid_tera_syntax() {
        let template = "Hello, {{ name }}!";
        assert!(validate_tera_syntax(template).is_ok());
    }

    #[test]
    fn test_valid_tera_syntax_with_filters() {
        let template = "Hello, {{ name | upper }}!";
        assert!(validate_tera_syntax(template).is_ok());
    }

    #[test]
    fn test_valid_tera_syntax_with_conditionals() {
        let template = r#"
            {% if enabled %}
            feature: on
            {% else %}
            feature: off
            {% endif %}
        "#;
        assert!(validate_tera_syntax(template).is_ok());
    }

    #[test]
    fn test_valid_tera_syntax_with_loops() {
        let template = r#"
            {% for item in items %}
            - {{ item }}
            {% endfor %}
        "#;
        assert!(validate_tera_syntax(template).is_ok());
    }

    #[test]
    fn test_invalid_tera_syntax_unclosed_brace() {
        let template = "Hello, {{ name !";
        let result = validate_tera_syntax(template);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.message.contains("Invalid Tera syntax"));
    }

    #[test]
    fn test_invalid_tera_syntax_unclosed_block() {
        let template = "{% if true %} hello";
        let result = validate_tera_syntax(template);
        assert!(result.is_err());
    }

    #[test]
    fn test_valid_tera_syntax_plain_text() {
        let template = "Just plain text without any variables";
        assert!(validate_tera_syntax(template).is_ok());
    }

    #[test]
    fn test_valid_tera_syntax_default_filter() {
        let template = "replicas: {{ replicas | default(value=1) }}";
        assert!(validate_tera_syntax(template).is_ok());
    }

    // ==================== Template Rendering Tests ====================

    #[test]
    fn test_render_template_simple() {
        let template = "Hello, {{ name }}!";
        let params = json!({"name": "World"});
        let result = render_template(template, &params).unwrap();
        assert_eq!(result, "Hello, World!");
    }

    #[test]
    fn test_render_template_multiple_vars() {
        let template = "name: {{ name }}\nreplicas: {{ replicas }}";
        let params = json!({"name": "my-app", "replicas": 3});
        let result = render_template(template, &params).unwrap();
        assert!(result.contains("name: my-app"));
        assert!(result.contains("replicas: 3"));
    }

    #[test]
    fn test_render_template_with_default() {
        let template = "replicas: {{ replicas | default(value=1) }}";
        let params = json!({});
        let result = render_template(template, &params).unwrap();
        assert_eq!(result, "replicas: 1");
    }

    #[test]
    fn test_render_template_missing_required_var() {
        let template = "Hello, {{ name }}!";
        let params = json!({});
        let result = render_template(template, &params);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.message.contains("rendering failed"));
    }

    #[test]
    fn test_render_template_with_filter() {
        let template = "Name: {{ name | upper }}";
        let params = json!({"name": "test"});
        let result = render_template(template, &params).unwrap();
        assert_eq!(result, "Name: TEST");
    }

    #[test]
    fn test_render_template_nested_object() {
        let template = "host: {{ config.host }}";
        let params = json!({"config": {"host": "localhost"}});
        let result = render_template(template, &params).unwrap();
        assert_eq!(result, "host: localhost");
    }

    // ==================== JSON Schema Validation Tests ====================

    #[test]
    fn test_valid_json_schema_simple() {
        let schema = r#"{"type": "object"}"#;
        assert!(validate_json_schema(schema).is_ok());
    }

    #[test]
    fn test_valid_json_schema_with_properties() {
        let schema = r#"{
            "type": "object",
            "properties": {
                "name": {"type": "string"},
                "count": {"type": "integer"}
            }
        }"#;
        assert!(validate_json_schema(schema).is_ok());
    }

    #[test]
    fn test_valid_json_schema_with_required() {
        let schema = r#"{
            "type": "object",
            "required": ["name"],
            "properties": {
                "name": {"type": "string"}
            }
        }"#;
        assert!(validate_json_schema(schema).is_ok());
    }

    #[test]
    fn test_invalid_json_not_json() {
        let schema = "not json";
        let result = validate_json_schema(schema);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.message.contains("Invalid JSON"));
    }

    #[test]
    fn test_empty_json_schema_valid() {
        // Empty object is a valid JSON Schema (matches anything)
        let schema = "{}";
        assert!(validate_json_schema(schema).is_ok());
    }

    // ==================== Parameter Validation Tests ====================

    #[test]
    fn test_validate_parameters_valid() {
        let schema = r#"{"type": "object", "properties": {"name": {"type": "string"}}}"#;
        let params = json!({"name": "test"});
        assert!(validate_parameters(schema, &params).is_ok());
    }

    #[test]
    fn test_validate_parameters_missing_required() {
        let schema = r#"{"type": "object", "required": ["name"], "properties": {"name": {"type": "string"}}}"#;
        let params = json!({});
        let result = validate_parameters(schema, &params);
        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert!(!errors.is_empty());
    }

    #[test]
    fn test_validate_parameters_wrong_type() {
        let schema = r#"{"type": "object", "properties": {"count": {"type": "integer"}}}"#;
        let params = json!({"count": "not a number"});
        let result = validate_parameters(schema, &params);
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_parameters_pattern() {
        let schema = r#"{"type": "object", "properties": {"size": {"type": "string", "pattern": "^[0-9]+[GMK]i$"}}}"#;

        // Valid pattern
        let params = json!({"size": "10Gi"});
        assert!(validate_parameters(schema, &params).is_ok());

        // Invalid pattern
        let params = json!({"size": "10GB"});
        assert!(validate_parameters(schema, &params).is_err());
    }

    #[test]
    fn test_validate_parameters_minimum() {
        let schema = r#"{"type": "object", "properties": {"replicas": {"type": "integer", "minimum": 1}}}"#;

        // Valid minimum
        let params = json!({"replicas": 1});
        assert!(validate_parameters(schema, &params).is_ok());

        // Below minimum
        let params = json!({"replicas": 0});
        assert!(validate_parameters(schema, &params).is_err());
    }

    #[test]
    fn test_validate_parameters_empty_schema() {
        // Empty schema matches anything
        let schema = "{}";
        let params = json!({"anything": "goes"});
        assert!(validate_parameters(schema, &params).is_ok());
    }

    #[test]
    fn test_validate_parameters_complex_schema() {
        let schema = r#"{
            "type": "object",
            "required": ["database_name", "storage_size"],
            "properties": {
                "database_name": {"type": "string", "minLength": 1},
                "storage_size": {"type": "string", "pattern": "^[0-9]+[GMK]i$"},
                "replicas": {"type": "integer", "minimum": 1, "default": 1},
                "postgres_version": {"type": "string", "default": "15"}
            }
        }"#;

        // Valid parameters
        let params = json!({
            "database_name": "mydb",
            "storage_size": "10Gi",
            "replicas": 3
        });
        assert!(validate_parameters(schema, &params).is_ok());

        // Missing required
        let params = json!({
            "database_name": "mydb"
        });
        assert!(validate_parameters(schema, &params).is_err());
    }
}
