// src/models/agents.rs

use chrono::{DateTime, Utc};
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Queryable, Selectable, Identifiable, AsChangeset, Debug, Clone, Serialize, Deserialize)]
#[diesel(table_name = crate::schema::agents)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Agent {
    pub id: Uuid,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
    pub uuid: Uuid,
    pub name: String,
    pub cluster_name: String,
    pub labels: Option<serde_json::Value>,
    pub annotations: Option<serde_json::Value>,
    pub last_heartbeat: Option<DateTime<Utc>>,
    pub status: String,
}

#[derive(Insertable, Debug, Clone, Serialize, Deserialize)]
#[diesel(table_name = crate::schema::agents)]
pub struct NewAgent {
    pub name: String,
    pub cluster_name: String,
    pub labels: Option<serde_json::Value>,
    pub annotations: Option<serde_json::Value>,
}

impl NewAgent {
    pub fn new(
        name: String,
        cluster_name: String,
        labels: Option<Vec<String>>,
        annotations: Option<Vec<(String, String)>>,
    ) -> Result<Self, String> {
        // Check for empty strings
        if name.trim().is_empty() {
            return Err("Name cannot be empty".to_string());
        }
        if cluster_name.trim().is_empty() {
            return Err("Cluster name cannot be empty".to_string());
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

        Ok(NewAgent {
            name,
            cluster_name,
            labels: labels_to_json(&labels),
            annotations: annotations_to_json(&annotations),
        })
    }
}

// Helper function to convert Vec<String> to serde_json::Value
pub fn labels_to_json(labels: &Option<Vec<String>>) -> Option<serde_json::Value> {
    labels.as_ref().map(|l| serde_json::to_value(l).unwrap())
}

// Helper function to convert Vec<(String, String)> to serde_json::Value
pub fn annotations_to_json(annotations: &Option<Vec<(String, String)>>) -> Option<serde_json::Value> {
    annotations.as_ref().map(|a| serde_json::to_value(a).unwrap())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_agent_success() {
        let name = "Test Agent".to_string();
        let cluster_name = "Test Cluster".to_string();
        let labels = Some(vec!["orange".to_string(), "blue".to_string()]);
        let annotations = Some(vec![
            ("security".to_string(), "high".to_string()),
            ("color".to_string(), "blue".to_string()),
        ]);


        let new_agent = NewAgent::new(
            name.clone(),
            cluster_name.clone(),
            labels.clone(),
            annotations.clone(),
        ).unwrap();

        assert_eq!(new_agent.name, name);
        assert_eq!(new_agent.cluster_name, cluster_name);
        assert_eq!(new_agent.labels, labels_to_json(&labels));
        assert_eq!(new_agent.annotations, annotations_to_json(&annotations));

        if let Some(lj) = new_agent.labels {
            assert_eq!(lj, serde_json::json!(["orange", "blue"]));
        }

        if let Some(aj) = new_agent.annotations {
            assert_eq!(aj, serde_json::json!([["security", "high"], ["color", "blue"]]));
        }
    }

    #[test]
    fn test_new_agent_empty_name() {
        let result = NewAgent::new(
            "".to_string(),
            "Test Cluster".to_string(),
            None,
            None,
        );
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Name cannot be empty");
    }

    #[test]
    fn test_new_agent_empty_cluster_name() {
        let result = NewAgent::new(
            "Test Agent".to_string(),
            "".to_string(),
            None,
            None,
        );
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Cluster name cannot be empty");
    }

    #[test]
    fn test_new_agent_empty_label() {
        let result = NewAgent::new(
            "Test Agent".to_string(),
            "Test Cluster".to_string(),
            Some(vec!["valid".to_string(), "".to_string()]),
            None,
        );
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Labels cannot contain empty strings");
    }

    #[test]
    fn test_new_agent_empty_annotation_key() {
        let result = NewAgent::new(
            "Test Agent".to_string(),
            "Test Cluster".to_string(),
            None,
            Some(vec![("".to_string(), "value".to_string())]),
        );
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Annotations cannot contain empty keys or values");
    }

    #[test]
    fn test_new_agent_empty_annotation_value() {
        let result = NewAgent::new(
            "Test Agent".to_string(),
            "Test Cluster".to_string(),
            None,
            Some(vec![("key".to_string(), "".to_string())]),
        );
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Annotations cannot contain empty keys or values");
    }
}