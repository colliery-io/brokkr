// src/models/stacks.rs

use chrono::{DateTime, Utc};
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Queryable, Selectable, Identifiable, AsChangeset, Debug, Clone, Serialize, Deserialize)]
#[diesel(table_name = crate::schema::stacks)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Stack {
    pub id: Uuid,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
    pub name: String,
    pub description: Option<String>,
    pub labels: Option<serde_json::Value>,
    pub annotations: Option<serde_json::Value>,
    pub agent_target: Option<serde_json::Value>,
}

#[derive(Insertable, Debug, Clone, Serialize, Deserialize)]
#[diesel(table_name = crate::schema::stacks)]
pub struct NewStack {
    pub name: String,
    pub description: Option<String>,
    pub labels: Option<serde_json::Value>,
    pub annotations: Option<serde_json::Value>,
    pub agent_target: Option<serde_json::Value>,
}

impl NewStack {
    pub fn new(
        name: String,
        description: Option<String>,
        labels: Option<Vec<String>>,
        annotations: Option<Vec<(String, String)>>,
        agent_target: Option<Vec<String>>,
    ) -> Result<Self, String> {
        // Check for empty name
        if name.trim().is_empty() {
            return Err("Name cannot be empty".to_string());
        }

        // Check labels
        if let Some(ref labels) = labels {
            if labels.iter().any(|label| label.trim().is_empty()) {
                return Err("Labels cannot contain empty strings".to_string());
            }
        }

        // Check annotations
        if let Some(ref annotations) = annotations {
            if annotations.iter().any(|(k, v)| k.trim().is_empty() || v.trim().is_empty()) {
                return Err("Annotations cannot contain empty keys or values".to_string());
            }
        }
        
         // Check labels
         if let Some(ref agent_target) = agent_target {
            if agent_target.iter().any(|target| target.trim().is_empty()) {
                return Err("Labels cannot contain empty strings".to_string());
            }
        }

        Ok(NewStack {
            name,
            description,
            labels: labels.map(|l| serde_json::to_value(l).unwrap()),
            annotations: annotations.map(|a| serde_json::to_value(a).unwrap()),
            agent_target: agent_target.map(|l| serde_json::to_value(l).unwrap())
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_new_stack_success() {
        let name = "Test Stack".to_string();
        let description = Some("A test stack".to_string());
        let labels = Some(vec!["test".to_string(), "example".to_string()]);
        let annotations = Some(vec![
            ("key1".to_string(), "value1".to_string()),
            ("key2".to_string(), "value2".to_string()),
        ]);
        let agent_target = Some(vec!["agent1".to_string(), "agent2".to_string()]);

        let new_stack = NewStack::new(
            name.clone(),
            description.clone(),
            labels.clone(),
            annotations.clone(),
            agent_target.clone(),
        ).unwrap();

        assert_eq!(new_stack.name, name);
        assert_eq!(new_stack.description, description);
        assert_eq!(new_stack.labels, labels.map(|l| json!(l)));
        assert_eq!(new_stack.annotations, annotations.map(|a| json!(a)));
        assert_eq!(new_stack.agent_target, agent_target.map(|a| json!(a)));
    }

    #[test]
    fn test_new_stack_empty_name() {
        let result = NewStack::new(
            "".to_string(),
            None,
            None,
            None,
            None,
        );
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Name cannot be empty");
    }

    #[test]
    fn test_new_stack_empty_label() {
        let result = NewStack::new(
            "Test Stack".to_string(),
            None,
            Some(vec!["valid".to_string(), "".to_string()]),
            None,
            None,
        );
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Labels cannot contain empty strings");
    }

    #[test]
    fn test_new_stack_empty_annotation_key() {
        let result = NewStack::new(
            "Test Stack".to_string(),
            None,
            None,
            Some(vec![("".to_string(), "value".to_string())]),
            None,
        );
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Annotations cannot contain empty keys or values");
    }

    #[test]
    fn test_new_stack_empty_annotation_value() {
        let result = NewStack::new(
            "Test Stack".to_string(),
            None,
            None,
            Some(vec![("key".to_string(), "".to_string())]),
            None,
        );
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Annotations cannot contain empty keys or values");
    }

    #[test]
    fn test_new_stack_empty_agent_target() {
        let result = NewStack::new(
            "Test Stack".to_string(),
            None,
            None,
            None,
            Some(vec!["valid".to_string(), "".to_string()]),
        );
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Labels cannot contain empty strings");
    }

    #[test]
    fn test_new_stack_valid_empty_description() {
        let result = NewStack::new(
            "Test Stack".to_string(),
            Some("".to_string()),
            None,
            None,
            None,
        );
        assert!(result.is_ok());
    }

    #[test]
    fn test_new_stack_valid_agent_target() {
        let result = NewStack::new(
            "Test Stack".to_string(),
            None,
            None,
            None,
            Some(vec!["agent1".to_string(), "agent2".to_string()]),
        );
        assert!(result.is_ok());
    }

    #[test]
    fn test_new_stack_all_none_optional_fields() {
        let result = NewStack::new(
            "Test Stack".to_string(),
            None,
            None,
            None,
            None,
        );
        assert!(result.is_ok());
        let new_stack = result.unwrap();
        assert_eq!(new_stack.name, "Test Stack");
        assert_eq!(new_stack.description, None);
        assert_eq!(new_stack.labels, None);
        assert_eq!(new_stack.annotations, None);
        assert_eq!(new_stack.agent_target, None);
    }
}