use chrono::{DateTime, Utc};
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Queryable, Selectable, Identifiable, AsChangeset, Debug, Clone, Serialize, Deserialize)]
#[diesel(table_name = crate::schema::agents)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct AgentDB {
    pub id: Uuid,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
    pub name: String,
    pub cluster_name: String,
    pub last_heartbeat: Option<DateTime<Utc>>,
    pub status: String,
    pub pak_hash: String,
}

#[derive(Insertable, Debug, Clone, Serialize, Deserialize)]
#[diesel(table_name = crate::schema::agents)]
pub struct NewAgentDB {
    pub name: String,
    pub cluster_name: String,
    pub pak_hash: Option<String>,
    pub status: Option<String>,
}

impl NewAgentDB {
    pub fn new(name: String, cluster_name: String, pak_hash: Option<String>) -> Result<Self, String> {
        // Validation
        if name.trim().is_empty() {
            return Err("Name cannot be empty".to_string());
        }
        if name.len() > 255 {
            return Err("Name cannot exceed 255 characters".to_string());
        }
        if cluster_name.trim().is_empty() {
            return Err("Cluster name cannot be empty".to_string());
        }
        if cluster_name.len() > 255 {
            return Err("Cluster name cannot exceed 255 characters".to_string());
        }

        Ok(NewAgentDB {
            name,
            cluster_name,
            pak_hash,
            status: Some("INACTIVE".to_string()),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_agent_success() {
        let name = "Test Agent".to_string();
        let cluster_name = "Test Cluster".to_string();
        let pak_hash = Some("test_hash".to_string());

        let new_agent = NewAgentDB::new(
            name.clone(),
            cluster_name.clone(),
            pak_hash.clone(),
        )
        .unwrap();

        assert_eq!(new_agent.name, name);
        assert_eq!(new_agent.cluster_name, cluster_name);
        assert_eq!(new_agent.pak_hash, pak_hash);
        assert_eq!(new_agent.status, Some("INACTIVE".to_string()));
    }

    #[test]
    fn test_new_agent_without_pak_hash() {
        let name = "Test Agent".to_string();
        let cluster_name = "Test Cluster".to_string();

        let new_agent = NewAgentDB::new(
            name.clone(),
            cluster_name.clone(),
            None,
        )
        .unwrap();

        assert_eq!(new_agent.name, name);
        assert_eq!(new_agent.cluster_name, cluster_name);
        assert_eq!(new_agent.pak_hash, None);
        assert_eq!(new_agent.status, Some("INACTIVE".to_string()));
    }

    #[test]
    fn test_new_agent_empty_name() {
        let result = NewAgentDB::new(
            "".to_string(),
            "Test Cluster".to_string(),
            None,
        );
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Name cannot be empty");
    }

    #[test]
    fn test_new_agent_empty_cluster_name() {
        let result = NewAgentDB::new(
            "Test Agent".to_string(),
            "".to_string(),
            None,
        );
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Cluster name cannot be empty");
    }
}