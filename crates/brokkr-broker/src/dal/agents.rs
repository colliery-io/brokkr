use crate::dal::DAL;
use brokkr_models::models::agents::{Agent, NewAgent};
use brokkr_models::schema::{agents,deployment_objects, agent_labels, agent_annotations, agent_targets};
use brokkr_models::models::deployment_objects::DeploymentObject;
use chrono::Utc;
use diesel::prelude::*;
use uuid::Uuid;
use crate::dal::FilterType;
use std::collections::HashSet;
use brokkr_models::models::agent_labels::AgentLabel;
use brokkr_models::models::agent_targets::AgentTarget;
use brokkr_models::models::agent_annotations::AgentAnnotation;

pub struct AgentFilter {
    pub labels: Vec<String>,
    pub annotations: Vec<(String, String)>,
    pub agent_targets: Vec<Uuid>,
    pub filter_type: FilterType,
}

/// Data Access Layer for Agent operations.
pub struct AgentsDAL<'a> {
    /// Reference to the main DAL instance.
    pub dal: &'a DAL,
}

impl<'a> AgentsDAL<'a> {
    /// Creates a new agent in the database.
    ///
    /// # Arguments
    ///
    /// * `new_agent` - A reference to the NewAgent struct containing the agent details.
    ///
    /// # Returns
    ///
    /// Returns a Result containing the created Agent on success, or a diesel::result::Error on failure.
    pub fn create(&self, new_agent: &NewAgent) -> Result<Agent, diesel::result::Error> {
        let conn = &mut self.dal.pool.get().expect("Failed to get DB connection");
        diesel::insert_into(agents::table)
            .values(new_agent)
            .get_result(conn)
    }

    /// Retrieves a non-deleted agent by its UUID.
    ///
    /// # Arguments
    ///
    /// * `agent_uuid` - The UUID of the agent to retrieve.
    ///
    /// # Returns
    ///
    /// Returns a Result containing an Option<Agent> if found (and not deleted), or a diesel::result::Error on failure.
    pub fn get(&self, agent_uuid: Uuid) -> Result<Option<Agent>, diesel::result::Error> {
        let conn = &mut self.dal.pool.get().expect("Failed to get DB connection");
        agents::table
            .filter(agents::id.eq(agent_uuid))
            .filter(agents::deleted_at.is_null())
            .first(conn)
            .optional()
    }

    /// Retrieves an agent by its UUID, including deleted agents.
    ///
    /// # Arguments
    ///
    /// * `agent_uuid` - The UUID of the agent to retrieve.
    ///
    /// # Returns
    ///
    /// Returns a Result containing an Option<Agent> if found (including deleted agents), or a diesel::result::Error on failure.
    pub fn get_including_deleted(&self, agent_uuid: Uuid) -> Result<Option<Agent>, diesel::result::Error> {
        let conn = &mut self.dal.pool.get().expect("Failed to get DB connection");
        agents::table
            .filter(agents::id.eq(agent_uuid))
            .first(conn)
            .optional()
    }

    /// Lists all non-deleted agents from the database.
    ///
    /// # Returns
    ///
    /// Returns a Result containing a Vec of all non-deleted Agents on success, or a diesel::result::Error on failure.
    pub fn list(&self) -> Result<Vec<Agent>, diesel::result::Error> {
        let conn = &mut self.dal.pool.get().expect("Failed to get DB connection");
        agents::table
            .filter(agents::deleted_at.is_null())
            .load::<Agent>(conn)
    }

    /// Lists all agents from the database, including deleted ones.
    ///
    /// # Returns
    ///
    /// Returns a Result containing a Vec of all Agents (including deleted ones) on success, or a diesel::result::Error on failure.
    pub fn list_all(&self) -> Result<Vec<Agent>, diesel::result::Error> {
        let conn = &mut self.dal.pool.get().expect("Failed to get DB connection");
        agents::table.load::<Agent>(conn)
    }

    /// Updates an existing agent in the database.
    ///
    /// # Arguments
    ///
    /// * `agent_uuid` - The UUID of the agent to update.
    /// * `updated_agent` - A reference to the Agent struct containing the updated details.
    ///
    /// # Returns
    ///
    /// Returns a Result containing the updated Agent on success, or a diesel::result::Error on failure.
    pub fn update(&self, agent_uuid: Uuid, updated_agent: &Agent) -> Result<Agent, diesel::result::Error> {
        let conn = &mut self.dal.pool.get().expect("Failed to get DB connection");
        diesel::update(agents::table.filter(agents::id.eq(agent_uuid)))
            .set(updated_agent)
            .get_result(conn)
    }

    /// Soft deletes an agent by setting its deleted_at timestamp to the current time.
    ///
    /// # Arguments
    ///
    /// * `agent_uuid` - The UUID of the agent to soft delete.
    ///
    /// # Returns
    ///
    /// Returns a Result containing the number of affected rows (0 or 1) on success, or a diesel::result::Error on failure.
    pub fn soft_delete(&self, agent_uuid: Uuid) -> Result<usize, diesel::result::Error> {
        let conn = &mut self.dal.pool.get().expect("Failed to get DB connection");
        diesel::update(agents::table.filter(agents::id.eq(agent_uuid)))
            .set(agents::deleted_at.eq(Utc::now()))
            .execute(conn)
    }

    /// Hard deletes an agent from the database.
    ///
    /// # Arguments
    ///
    /// * `agent_uuid` - The UUID of the agent to hard delete.
    ///
    /// # Returns
    ///
    /// Returns a Result containing the number of affected rows (0 or 1) on success, or a diesel::result::Error on failure.
    pub fn hard_delete(&self, agent_uuid: Uuid) -> Result<usize, diesel::result::Error> {
        let conn = &mut self.dal.pool.get().expect("Failed to get DB connection");
        diesel::delete(agents::table.filter(agents::id.eq(agent_uuid)))
            .execute(conn)
    }

    /// Filters agents by labels.
    ///
    /// # Arguments
    ///
    /// * `labels` - A vector of label strings to filter by.
    /// * `filter_type` - Specifies whether to use AND or OR logic for multiple labels.
    ///
    /// # Returns
    ///
    /// Returns a Result containing a Vec of matching Agents on success, or a diesel::result::Error on failure.
    ///
    /// # SQL Queries
    ///
    /// For FilterType::And:
    /// ```sql
    /// SELECT DISTINCT agents.*
    /// FROM agents
    /// WHERE agents.deleted_at IS NULL
    ///   AND EXISTS (SELECT 1 FROM agent_labels WHERE agent_labels.agent_id = agents.id AND agent_labels.label = ?)
    ///   AND EXISTS (SELECT 1 FROM agent_labels WHERE agent_labels.agent_id = agents.id AND agent_labels.label = ?)
    ///   -- ... (repeated for each label)
    /// ```
    ///
    /// For FilterType::Or:
    /// ```sql
    /// SELECT DISTINCT agents.*
    /// FROM agents
    /// INNER JOIN agent_labels ON agents.id = agent_labels.agent_id
    /// WHERE agents.deleted_at IS NULL
    ///   AND agent_labels.label IN (?, ?, ...)
    /// ```
    pub fn filter_by_labels(&self, labels: Vec<String>, filter_type: FilterType) -> Result<Vec<Agent>, diesel::result::Error> {
        let conn = &mut self.dal.pool.get().expect("Failed to get DB connection");
        
        match filter_type {
            FilterType::And => {
                let mut query = agents::table
                    .filter(agents::deleted_at.is_null())
                    .into_boxed();

                for label in &labels {
                    let subquery = agent_labels::table
                        .filter(agent_labels::agent_id.eq(agents::id))
                        .filter(agent_labels::label.eq(label));
                    query = query.filter(diesel::dsl::exists(subquery));
                }

                query
                    .select(agents::all_columns)
                    .distinct()
                    .load::<Agent>(conn)
            },
            FilterType::Or => {
                agents::table
                    .inner_join(agent_labels::table)
                    .filter(agents::deleted_at.is_null())
                    .filter(agent_labels::label.eq_any(labels))
                    .select(agents::all_columns)
                    .distinct()
                    .load::<Agent>(conn)
            }
        }
    }

    /// Filters agents by annotations.
    ///
    /// # Arguments
    ///
    /// * `annotations` - A vector of (key, value) pairs to filter by.
    /// * `filter_type` - Specifies whether to use AND or OR logic for multiple annotations.
    ///
    /// # Returns
    ///
    /// Returns a Result containing a Vec of matching Agents on success, or a diesel::result::Error on failure.
    ///
    /// # SQL Generated (Roughly Equivalent)
    ///
    /// For FilterType::Or:
    /// ```sql
    /// SELECT DISTINCT agents.*
    /// FROM agents
    /// INNER JOIN agent_annotations ON agents.id = agent_annotations.agent_id
    /// WHERE agents.deleted_at IS NULL
    ///   AND ((agent_annotations.key = ? AND agent_annotations.value = ?)
    ///     OR (agent_annotations.key = ? AND agent_annotations.value = ?)
    ///     OR ...)
    /// ```
    ///
    /// For FilterType::And:
    /// ```sql
    /// SELECT DISTINCT agents.*
    /// FROM agents
    /// WHERE agents.deleted_at IS NULL
    ///   AND EXISTS (SELECT 1 FROM agent_annotations WHERE agent_annotations.agent_id = agents.id AND agent_annotations.key = ? AND agent_annotations.value = ?)
    ///   AND EXISTS (SELECT 1 FROM agent_annotations WHERE agent_annotations.agent_id = agents.id AND agent_annotations.key = ? AND agent_annotations.value = ?)
    ///   -- ... (repeated for each annotation pair)
    /// ```
    ///
    /// Note: The actual implementation uses Rust code to perform the filtering,
    /// which may not directly translate to a single SQL query.
    pub fn filter_by_annotations(&self, annotations: Vec<(String, String)>, filter_type: FilterType) -> Result<Vec<Agent>, diesel::result::Error> {
        let conn = &mut self.dal.pool.get().expect("Failed to get DB connection");
        
        match filter_type {
            FilterType::Or => {
                let mut all_matching_agents = HashSet::new();

                for (key, value) in annotations {
                    let matching_agents: Vec<Agent> = agents::table
                        .inner_join(agent_annotations::table)
                        .filter(agents::deleted_at.is_null())
                        .filter(agent_annotations::key.eq(key))
                        .filter(agent_annotations::value.eq(value))
                        .select(agents::all_columns)
                        .load(conn)?;

                    all_matching_agents.extend(matching_agents);
                }

                Ok(all_matching_agents.into_iter().collect())
            },
            FilterType::And => {
                if annotations.is_empty() {
                    return Ok(Vec::new());
                }

                let mut all_matching_agents: Option<HashSet<Agent>> = None;

                for (key, value) in annotations {
                    let matching_agents: HashSet<Agent> = agents::table
                        .inner_join(agent_annotations::table)
                        .filter(agents::deleted_at.is_null())
                        .filter(agent_annotations::key.eq(key))
                        .filter(agent_annotations::value.eq(value))
                        .select(agents::all_columns)
                        .load(conn)?
                        .into_iter()
                        .collect();

                    all_matching_agents = match all_matching_agents {
                        Some(agents) => Some(agents.intersection(&matching_agents).cloned().collect()),
                        None => Some(matching_agents),
                    };

                    if let Some(ref agents) = all_matching_agents {
                        if agents.is_empty() {
                            break;
                        }
                    }
                }

                Ok(all_matching_agents.map_or_else(Vec::new, |agents| agents.into_iter().collect()))
            }
        }
    }

    /// Retrieves an agent by its target ID.
    ///
    /// # Arguments
    ///
    /// * `agent_target_id` - The UUID of the agent target to retrieve.
    ///
    /// # Returns
    ///
    /// Returns a Result containing an Option<Agent> if found, or a diesel::result::Error on failure.
    pub fn get_agent_by_target_id(&self, agent_target_id: Uuid) -> Result<Option<Agent>, diesel::result::Error> {
        let conn = &mut self.dal.pool.get().expect("Failed to get DB connection");
        agents::table
            .inner_join(agent_targets::table)
            .filter(agents::deleted_at.is_null())
            .filter(agent_targets::stack_id.eq(agent_target_id))
            .select(agents::all_columns)
            .first(conn)
            .optional()
    }

    /// Retrieves labels, targets, and annotations associated with a specific agent.
    ///
    /// # Arguments
    ///
    /// * `agent_id` - The UUID of the agent to retrieve details for.
    ///
    /// # Returns
    ///
    /// Returns a Result containing a tuple of (Vec<AgentLabel>, Vec<AgentTarget>, Vec<AgentAnnotation>)
    /// on success, or a diesel::result::Error on failure.
    pub fn get_agent_details(&self, agent_id: Uuid) -> Result<(Vec<AgentLabel>, Vec<AgentTarget>, Vec<AgentAnnotation>), diesel::result::Error> {
        let conn = &mut self.dal.pool.get().expect("Failed to get DB connection");

        let labels = agent_labels::table
            .filter(agent_labels::agent_id.eq(agent_id))
            .load::<AgentLabel>(conn)?;

        let targets = agent_targets::table
            .filter(agent_targets::agent_id.eq(agent_id))
            .load::<AgentTarget>(conn)?;

        let annotations = agent_annotations::table
            .filter(agent_annotations::agent_id.eq(agent_id))
            .load::<AgentAnnotation>(conn)?;

        Ok((labels, targets, annotations))
    }


    pub fn record_heartbeat(&self, agent_id: Uuid) -> Result<(), diesel::result::Error> {
        let conn = &mut self.dal.pool.get().expect("Failed to get DB connection");
    
        diesel::update(agents::table.filter(agents::id.eq(agent_id)))
            .set(agents::last_heartbeat.eq(diesel::dsl::now))
            .execute(conn)?;
    
        Ok(())
    }
}