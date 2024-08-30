use crate::dal::DAL;
use brokkr_models::models::agents::{Agent, NewAgent};
use brokkr_models::schema::agents;
use chrono::Utc;
use diesel::prelude::*;
use uuid::Uuid;

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

    /// Searches for non-deleted agents by name or cluster name.
    ///
    /// # Arguments
    ///
    /// * `query` - The search string to match against agent names or cluster names.
    ///
    /// # Returns
    ///
    /// Returns a Result containing a Vec of matching non-deleted Agents on success, or a diesel::result::Error on failure.
    pub fn search(&self, query: &str) -> Result<Vec<Agent>, diesel::result::Error> {
        let conn = &mut self.dal.pool.get().expect("Failed to get DB connection");
        agents::table
            .filter(agents::name.ilike(format!("%{}%", query)).or(agents::cluster_name.ilike(format!("%{}%", query))))
            .filter(agents::deleted_at.is_null())
            .load::<Agent>(conn)
    }

    /// Searches for all agents by name or cluster name, including deleted ones.
    ///
    /// # Arguments
    ///
    /// * `query` - The search string to match against agent names or cluster names.
    ///
    /// # Returns
    ///
    /// Returns a Result containing a Vec of all matching Agents (including deleted ones) on success, or a diesel::result::Error on failure.
    pub fn search_all(&self, query: &str) -> Result<Vec<Agent>, diesel::result::Error> {
        let conn = &mut self.dal.pool.get().expect("Failed to get DB connection");
        agents::table
            .filter(agents::name.ilike(format!("%{}%", query)).or(agents::cluster_name.ilike(format!("%{}%", query))))
            .load::<Agent>(conn)
    }
}