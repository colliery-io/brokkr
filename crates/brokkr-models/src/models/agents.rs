//! # Agent Module
//! 
//! This module defines structures and methods for managing agents in the system.
//! 
//! ## Data Model
//! 
//! Agents represent the entities responsible for executing tasks and managing deployments.
//! They are stored in the `agents` table with the following structure:
//! 
//! - `id`: UUID, primary key
//! - `created_at`: TIMESTAMP, when the agent was created
//! - `updated_at`: TIMESTAMP, when the agent was last updated
//! - `deleted_at`: TIMESTAMP, for soft deletion support
//! - `name`: VARCHAR(255), name of the agent
//! - `cluster_name`: VARCHAR(255), name of the cluster the agent belongs to
//! - `last_heartbeat`: TIMESTAMP, last time the agent sent a heartbeat
//! - `status`: VARCHAR(50), current status of the agent
//! - `pak_hash`: VARCHAR(255), hash of the agent's PAK (Pre-shared Authentication Key)
//! 
//! ## Usage
//! 
//! Agents are core entities in the system, responsible for executing tasks, managing deployments,
//! and interacting with stacks. They report their status and heartbeats, allowing the system to
//! monitor their health and availability.
//! 
//! ## Constraints
//! 
//! - Both `name` and `cluster_name` must be non-empty strings.
//! - There should be a unique constraint on the combination of `name` and `cluster_name`.

use chrono::{DateTime, Utc};
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Represents an agent in the database.
#[derive(Queryable, Selectable, Identifiable, AsChangeset, Debug, Clone, Serialize, Deserialize, Eq, PartialEq, Hash)]
#[diesel(table_name = crate::schema::agents)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Agent {
    /// Unique identifier for the agent.
    pub id: Uuid,
    /// Timestamp when the agent was created.
    pub created_at: DateTime<Utc>,
    /// Timestamp when the agent was last updated.
    pub updated_at: DateTime<Utc>,
    /// Timestamp for soft deletion, if applicable.
    pub deleted_at: Option<DateTime<Utc>>,
    /// Name of the agent.
    pub name: String,
    /// Name of the cluster the agent belongs to.
    pub cluster_name: String,
    /// Timestamp of the last heartbeat received from the agent.
    pub last_heartbeat: Option<DateTime<Utc>>,
    /// Current status of the agent.
    pub status: String,
    /// Hash of the agent's Pre-shared Authentication Key (PAK).
    pub pak_hash: String,
}

/// Represents a new agent to be inserted into the database.
#[derive(Insertable, Debug, Clone, Serialize, Deserialize)]
#[diesel(table_name = crate::schema::agents)]
pub struct NewAgent {
    /// Name of the agent.
    pub name: String,
    /// Name of the cluster the agent belongs to.
    pub cluster_name: String,
}

impl NewAgent {
    /// Creates a new `NewAgent` instance.
    ///
    /// # Parameters
    ///
    /// * `name`: Name of the agent. Must be a non-empty string.
    /// * `cluster_name`: Name of the cluster the agent belongs to. Must be a non-empty string.
    ///
    /// # Returns
    ///
    /// Returns `Ok(NewAgent)` if both parameters are valid non-empty strings,
    /// otherwise returns an `Err` with a description of the validation failure.
    pub fn new(
        name: String,
        cluster_name: String,
    ) -> Result<Self, String> {
        // Validate name
        if name.trim().is_empty() {
            return Err("Agent name cannot be empty".to_string());
        }

        // Validate cluster_name
        if cluster_name.trim().is_empty() {
            return Err("Cluster name cannot be empty".to_string());
        }

        Ok(NewAgent {
            name,
            cluster_name,
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

        let result = NewAgent::new(
            name.clone(),
            cluster_name.clone(),
        );

        assert!(result.is_ok(), "NewAgent creation should succeed with valid inputs");
        let new_agent = result.unwrap();
        assert_eq!(new_agent.name, name, "name should match the input value");
        assert_eq!(new_agent.cluster_name, cluster_name, "cluster_name should match the input value");
    }

    #[test]
    fn test_new_agent_empty_name() {
        let result = NewAgent::new(
            "".to_string(),
            "Test Cluster".to_string(),
        );
        assert!(result.is_err(), "NewAgent creation should fail with empty name");
        assert_eq!(result.unwrap_err(), "Agent name cannot be empty", "Error message should indicate empty name");
    }

    #[test]
    fn test_new_agent_empty_cluster_name() {
        let result = NewAgent::new(
            "Test Agent".to_string(),
            "".to_string(),
        );
        assert!(result.is_err(), "NewAgent creation should fail with empty cluster name");
        assert_eq!(result.unwrap_err(), "Cluster name cannot be empty", "Error message should indicate empty cluster name");
    }
}