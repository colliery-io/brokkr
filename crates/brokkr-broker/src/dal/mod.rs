//! # Data Access Layer (DAL) Module
//!
//! This module provides an abstraction layer for database operations in the Brokkr Broker.
//! It includes structures and methods for interacting with various data entities such as
//! agents, stacks, deployment objects, and more.
//!
//! ## Main Structures
//!
//! - `DAL`: The main Data Access Layer struct that provides access to all sub-DALs.
//! - `FilterType`: An enum used for specifying filter types in queries.
//!
//! ## Usage
//!
//! To use the DAL in your code:
//!
//! ```rust
//! use brokkr_broker::dal::DAL;
//! use brokkr_broker::db::create_shared_connection_pool;
//!
//! let pool = create_shared_connection_pool("database_url", "app_name", 5);
//! let dal = DAL::new(pool);
//!
//! // Now you can use dal to access various data operations
//! let agents = dal.agents().list().expect("Failed to list agents");
//! ```

use diesel::r2d2::{ConnectionManager, Pool};
use diesel::PgConnection;

pub mod agents;
use agents::AgentsDAL;

pub mod agent_annotations;
use agent_annotations::AgentAnnotationsDAL;

pub mod agent_events;
use agent_events::AgentEventsDAL;

pub mod agent_labels;
use agent_labels::AgentLabelsDAL;

pub mod agent_targets;
use agent_targets::AgentTargetsDAL;

pub mod stacks;
use stacks::StacksDAL;

pub mod stack_annotations;
use stack_annotations::StackAnnotationsDAL;

pub mod stack_labels;
use stack_labels::StackLabelsDAL;

pub mod deployment_objects;
use deployment_objects::DeploymentObjectsDAL;

pub mod generators;
use generators::GeneratorsDAL;

/// The main Data Access Layer struct.
///
/// This struct serves as the central point for database operations,
/// managing a connection pool and providing access to specific DAL
/// implementations for different entities.
#[derive(Clone)]
pub struct DAL {
    /// A connection pool for PostgreSQL database connections.
    pub pool: Pool<ConnectionManager<PgConnection>>,
}

impl DAL {
    /// Creates a new DAL instance with the given connection pool.
    ///
    /// # Arguments
    ///
    /// * `pool` - A connection pool for PostgreSQL database connections.
    ///
    /// # Returns
    ///
    /// A new DAL instance.
    pub fn new(pool: Pool<ConnectionManager<PgConnection>>) -> Self {
        DAL { pool }
    }

    /// Provides access to the Agents Data Access Layer.
    ///
    /// # Returns
    ///
    /// An instance of AgentsDAL.
    pub fn agents(&self) -> AgentsDAL {
        AgentsDAL { dal: self }
    }

    /// Provides access to the Agent Anotations Data Access Layer.
    ///
    /// # Returns
    ///
    /// An instance of AgentAnontationsDAL.
    pub fn agent_annotations(&self) -> AgentAnnotationsDAL {
        AgentAnnotationsDAL { dal: self }
    }

    /// Provides access to the Agent Events Data Access Layer.
    ///
    /// # Returns
    ///
    /// An instance of AgentEventsDAL.
    pub fn agent_events(&self) -> AgentEventsDAL {
        AgentEventsDAL { dal: self }
    }

    /// Provides access to the Agent Labels Data Access Layer.
    ///
    /// # Returns
    ///
    /// An instance of AgentLabelsDAL.
    pub fn agent_labels(&self) -> AgentLabelsDAL {
        AgentLabelsDAL { dal: self }
    }

    /// Provides access to the Agent Targets Data Access Layer.
    ///
    /// # Returns
    ///
    /// An instance of AgentTargetssDAL.
    pub fn agent_targets(&self) -> AgentTargetsDAL {
        AgentTargetsDAL { dal: self }
    }

    /// Provides access to the Stack Labels Data Access Layer.
    ///
    /// # Returns
    ///
    /// An instance of StackLabelsDAL.
    pub fn stack_labels(&self) -> StackLabelsDAL {
        StackLabelsDAL { dal: self }
    }

    /// Provides access to the Stack Anotations Data Access Layer.
    ///
    /// # Returns
    ///
    /// An instance of StackAnontationsDAL.
    pub fn stack_annotations(&self) -> StackAnnotationsDAL {
        StackAnnotationsDAL { dal: self }
    }

    /// Provides access to the Stacks Data Access Layer.
    ///
    /// # Returns
    ///
    /// An instance of StacksDAL.
    pub fn stacks(&self) -> StacksDAL {
        StacksDAL { dal: self }
    }

    /// Provides access to the Deployment Objects Data Access Layer.
    ///
    /// # Returns
    ///
    /// An instance of DeploymentObjectsDAL.
    pub fn deployment_objects(&self) -> DeploymentObjectsDAL {
        DeploymentObjectsDAL { dal: self }
    }

    /// Provides access to the Generators Data Access Layer.
    ///
    /// # Returns
    ///
    /// An instance of GeneratorsDal.
    pub fn generators(&self) -> GeneratorsDAL {
        GeneratorsDAL { dal: self }
    }
}

#[derive(PartialEq)]
pub enum FilterType {
    And,
    Or,
}
