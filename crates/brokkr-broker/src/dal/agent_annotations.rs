use crate::dal::DAL;
use brokkr_models::models::agent_annotations::{AgentAnnotation, NewAgentAnnotation};
use brokkr_models::schema::agent_annotations;
use diesel::prelude::*;
use uuid::Uuid;

/// Data Access Layer for Agent Annotation operations.
pub struct AgentAnnotationsDAL<'a> {
    /// Reference to the main DAL instance.
    pub dal: &'a DAL,
}

impl<'a> AgentAnnotationsDAL<'a> {
    /// Creates a new agent annotation in the database.
    ///
    /// # Arguments
    ///
    /// * `new_annotation` - A reference to the NewAgentAnnotation struct containing the annotation details.
    ///
    /// # Returns
    ///
    /// Returns a Result containing the created AgentAnnotation on success, or a diesel::result::Error on failure.
    pub fn create(&self, new_annotation: &NewAgentAnnotation) -> Result<AgentAnnotation, diesel::result::Error> {
        let conn = &mut self.dal.pool.get().expect("Failed to get DB connection");
        diesel::insert_into(agent_annotations::table)
            .values(new_annotation)
            .get_result(conn)
    }

    /// Retrieves an agent annotation by its ID.
    ///
    /// # Arguments
    ///
    /// * `annotation_id` - The UUID of the annotation to retrieve.
    ///
    /// # Returns
    ///
    /// Returns a Result containing an Option<AgentAnnotation> if found, or a diesel::result::Error on failure.
    pub fn get(&self, annotation_id: Uuid) -> Result<Option<AgentAnnotation>, diesel::result::Error> {
        let conn = &mut self.dal.pool.get().expect("Failed to get DB connection");
        agent_annotations::table
            .filter(agent_annotations::id.eq(annotation_id))
            .first(conn)
            .optional()
    }

    /// Lists all annotations for a specific agent.
    ///
    /// # Arguments
    ///
    /// * `agent_id` - The UUID of the agent whose annotations to retrieve.
    ///
    /// # Returns
    ///
    /// Returns a Result containing a Vec of AgentAnnotations for the specified agent, or a diesel::result::Error on failure.
    pub fn list_for_agent(&self, agent_id: Uuid) -> Result<Vec<AgentAnnotation>, diesel::result::Error> {
        let conn = &mut self.dal.pool.get().expect("Failed to get DB connection");
        agent_annotations::table
            .filter(agent_annotations::agent_id.eq(agent_id))
            .load::<AgentAnnotation>(conn)
    }

    /// Updates an existing agent annotation in the database.
    ///
    /// # Arguments
    ///
    /// * `annotation_id` - The UUID of the annotation to update.
    /// * `updated_annotation` - A reference to the AgentAnnotation struct containing the updated details.
    ///
    /// # Returns
    ///
    /// Returns a Result containing the updated AgentAnnotation on success, or a diesel::result::Error on failure.
    pub fn update(&self, annotation_id: Uuid, updated_annotation: &AgentAnnotation) -> Result<AgentAnnotation, diesel::result::Error> {
        let conn = &mut self.dal.pool.get().expect("Failed to get DB connection");
        diesel::update(agent_annotations::table.filter(agent_annotations::id.eq(annotation_id)))
            .set(updated_annotation)
            .get_result(conn)
    }

    /// Deletes an agent annotation from the database.
    ///
    /// # Arguments
    ///
    /// * `annotation_id` - The UUID of the annotation to delete.
    ///
    /// # Returns
    ///
    /// Returns a Result containing the number of affected rows (0 or 1) on success, or a diesel::result::Error on failure.
    pub fn delete(&self, annotation_id: Uuid) -> Result<usize, diesel::result::Error> {
        let conn = &mut self.dal.pool.get().expect("Failed to get DB connection");
        diesel::delete(agent_annotations::table.filter(agent_annotations::id.eq(annotation_id)))
            .execute(conn)
    }

    /// Deletes all annotations for a specific agent.
    ///
    /// # Arguments
    ///
    /// * `agent_id` - The UUID of the agent whose annotations to delete.
    ///
    /// # Returns
    ///
    /// Returns a Result containing the number of affected rows on success, or a diesel::result::Error on failure.
    pub fn delete_all_for_agent(&self, agent_id: Uuid) -> Result<usize, diesel::result::Error> {
        let conn = &mut self.dal.pool.get().expect("Failed to get DB connection");
        diesel::delete(agent_annotations::table.filter(agent_annotations::agent_id.eq(agent_id)))
            .execute(conn)
    }
}