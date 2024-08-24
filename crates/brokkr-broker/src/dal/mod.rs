//! # Data Access Layer (DAL) Module
//!
//! This module provides a centralized Data Access Layer for managing database operations.
//!
//! ## Design
//!
//! The DAL is structured as follows:
//!
//! 1. A main `DAL` struct that holds a connection pool and provides access to entity-specific DALs.
//! 2. Separate modules for each entity (agents, agent_events, stacks, deployment_objects).
//! 3. Entity-specific DAL structs (e.g., `AgentsDAL`, `StacksDAL`) that handle database operations
//!    for their respective entities.
//!
//! This design allows for:
//! - Centralized management of database connections.
//! - Separation of concerns for different entities.
//! - Easy extension for new entities.
//!
//! ## Usage
//!
//!
//! 1. Create a DAL instance with a database connection pool:
//!
//! ```rust
//! use crate::db::create_shared_connection_pool;
//! use crade::dal::DAL;
//!
//! let database_url = "postgres://username:password@localhost/database_name";
//! let connection_pool = create_shared_connection_pool(&database_url, "brokkr", 5);
//! let dal = DAL::new(connection_pool.pool.clone());
//! ```
//!
//! 2. Use the DAL to perform database operations:
//!
//! ```rust
//! // Perform operations on agents
//! let agents = dal.agents().list_agents().await?;
//!
//! // Perform operations on stacks
//! let stack = dal.stacks().create_stack(new_stack).await?;
//!
//! // Perform operations on deployment objects
//! let objects = dal.deployment_objects().list_objects(stack_id).await?;
//! ```
//!
//! Each entity-specific DAL provides methods for common database operations like
//! create, read, update, and delete (CRUD).

use diesel::PgConnection;
use diesel::r2d2::{ConnectionManager, Pool};

mod agents;
mod agent_events;
mod stacks;
mod deployment_objects;

pub use stacks::StacksDAL;
pub use deployment_objects::DeploymentObjectsDAL;
pub use agents::AgentsDAL;
pub use agent_events::AgentEventsDAL;

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

    /// Provides access to the Agent Events Data Access Layer.
    ///
    /// # Returns
    ///
    /// An instance of AgentEventsDAL.
    pub fn agent_events(&self) -> AgentEventsDAL {
        AgentEventsDAL { dal: self }
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
}