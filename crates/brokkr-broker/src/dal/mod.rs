/*
 * Copyright (c) 2025 Dylan Storey
 * Licensed under the Elastic License 2.0.
 * See LICENSE file in the project root for full license text.
 */

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

use crate::db::ConnectionPool;

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

pub mod templates;
use templates::TemplatesDAL;

pub mod template_labels;
use template_labels::TemplateLabelsDAL;

pub mod template_annotations;
use template_annotations::TemplateAnnotationsDAL;

pub mod template_targets;
use template_targets::TemplateTargetsDAL;

pub mod rendered_deployment_objects;
use rendered_deployment_objects::RenderedDeploymentObjectsDAL;

pub mod work_orders;
use work_orders::WorkOrdersDAL;

/// The main Data Access Layer struct.
///
/// This struct serves as the central point for database operations,
/// managing a connection pool and providing access to specific DAL
/// implementations for different entities.
#[derive(Clone)]
pub struct DAL {
    /// A connection pool for PostgreSQL database connections with schema support.
    pub pool: ConnectionPool,
}

impl DAL {
    /// Creates a new DAL instance with the given connection pool.
    ///
    /// # Arguments
    ///
    /// * `pool` - A connection pool for PostgreSQL database connections with schema support.
    ///
    /// # Returns
    ///
    /// A new DAL instance.
    pub fn new(pool: ConnectionPool) -> Self {
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

    /// Provides access to the Agent Annotations Data Access Layer.
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

    /// Provides access to the Stack Annotations Data Access Layer.
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

    /// Provides access to the Templates Data Access Layer.
    ///
    /// # Returns
    ///
    /// An instance of TemplatesDAL.
    pub fn templates(&self) -> TemplatesDAL {
        TemplatesDAL { dal: self }
    }

    /// Provides access to the Template Labels Data Access Layer.
    ///
    /// # Returns
    ///
    /// An instance of TemplateLabelsDAL.
    pub fn template_labels(&self) -> TemplateLabelsDAL {
        TemplateLabelsDAL { dal: self }
    }

    /// Provides access to the Template Annotations Data Access Layer.
    ///
    /// # Returns
    ///
    /// An instance of TemplateAnnotationsDAL.
    pub fn template_annotations(&self) -> TemplateAnnotationsDAL {
        TemplateAnnotationsDAL { dal: self }
    }

    /// Provides access to the Template Targets Data Access Layer.
    ///
    /// # Returns
    ///
    /// An instance of TemplateTargetsDAL.
    pub fn template_targets(&self) -> TemplateTargetsDAL {
        TemplateTargetsDAL { dal: self }
    }

    /// Provides access to the Rendered Deployment Objects Data Access Layer.
    ///
    /// # Returns
    ///
    /// An instance of RenderedDeploymentObjectsDAL.
    pub fn rendered_deployment_objects(&self) -> RenderedDeploymentObjectsDAL {
        RenderedDeploymentObjectsDAL { dal: self }
    }

    /// Provides access to the Work Orders Data Access Layer.
    ///
    /// # Returns
    ///
    /// An instance of WorkOrdersDAL.
    pub fn work_orders(&self) -> WorkOrdersDAL {
        WorkOrdersDAL { dal: self }
    }
}

#[derive(PartialEq)]
pub enum FilterType {
    And,
    Or,
}
