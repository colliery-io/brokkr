use crate::dal::DAL;
use brokkr_models::models::deployment_objects::{DeploymentObject, NewDeploymentObject};
use brokkr_models::schema::deployment_objects;
use chrono::Utc;
use diesel::prelude::*;
use uuid::Uuid;

/// Data Access Layer for DeploymentObject operations.
pub struct DeploymentObjectsDAL<'a> {
    /// Reference to the main DAL instance.
    pub dal: &'a DAL,
}

impl<'a> DeploymentObjectsDAL<'a> {
    /// Creates a new deployment object in the database.
    ///
    /// # Arguments
    ///
    /// * `new_deployment_object` - A reference to the NewDeploymentObject struct containing the deployment object details.
    ///
    /// # Returns
    ///
    /// Returns a Result containing the created DeploymentObject on success, or a diesel::result::Error on failure.
    pub fn create(&self, new_deployment_object: &NewDeploymentObject) -> Result<DeploymentObject, diesel::result::Error> {
        let conn = &mut self.dal.pool.get().expect("Failed to get DB connection");
        diesel::insert_into(deployment_objects::table)
            .values(new_deployment_object)
            .get_result(conn)
    }

    /// Retrieves a non-deleted deployment object by its UUID.
    ///
    /// # Arguments
    ///
    /// * `deployment_object_uuid` - The UUID of the deployment object to retrieve.
    ///
    /// # Returns
    ///
    /// Returns a Result containing an Option<DeploymentObject> if found (and not deleted), or a diesel::result::Error on failure.
    pub fn get(&self, deployment_object_uuid: Uuid) -> Result<Option<DeploymentObject>, diesel::result::Error> {
        let conn = &mut self.dal.pool.get().expect("Failed to get DB connection");
        deployment_objects::table
            .filter(deployment_objects::id.eq(deployment_object_uuid))
            .filter(deployment_objects::deleted_at.is_null())
            .first(conn)
            .optional()
    }

    /// Retrieves a deployment object by its UUID, including deleted objects.
    ///
    /// # Arguments
    ///
    /// * `deployment_object_uuid` - The UUID of the deployment object to retrieve.
    ///
    /// # Returns
    ///
    /// Returns a Result containing an Option<DeploymentObject> if found (including deleted objects), or a diesel::result::Error on failure.
    pub fn get_including_deleted(&self, deployment_object_uuid: Uuid) -> Result<Option<DeploymentObject>, diesel::result::Error> {
        let conn = &mut self.dal.pool.get().expect("Failed to get DB connection");
        deployment_objects::table
            .filter(deployment_objects::id.eq(deployment_object_uuid))
            .first(conn)
            .optional()
    }

    /// Lists all non-deleted deployment objects for a specific stack.
    ///
    /// # Arguments
    ///
    /// * `stack_id` - The UUID of the stack to list deployment objects for.
    ///
    /// # Returns
    ///
    /// Returns a Result containing a Vec of all non-deleted DeploymentObjects for the specified stack on success, or a diesel::result::Error on failure.
    pub fn list_for_stack(&self, stack_id: Uuid) -> Result<Vec<DeploymentObject>, diesel::result::Error> {
        let conn = &mut self.dal.pool.get().expect("Failed to get DB connection");
        deployment_objects::table
            .filter(deployment_objects::stack_id.eq(stack_id))
            .filter(deployment_objects::deleted_at.is_null())
            .order(deployment_objects::sequence_id.desc())
            .load::<DeploymentObject>(conn)
    }

    /// Lists all deployment objects for a specific stack, including deleted ones.
    ///
    /// # Arguments
    ///
    /// * `stack_id` - The UUID of the stack to list deployment objects for.
    ///
    /// # Returns
    ///
    /// Returns a Result containing a Vec of all DeploymentObjects for the specified stack (including deleted ones) on success, or a diesel::result::Error on failure.
    pub fn list_all_for_stack(&self, stack_id: Uuid) -> Result<Vec<DeploymentObject>, diesel::result::Error> {
        let conn = &mut self.dal.pool.get().expect("Failed to get DB connection");
        deployment_objects::table
            .filter(deployment_objects::stack_id.eq(stack_id))
            .order(deployment_objects::sequence_id.desc())
            .load::<DeploymentObject>(conn)
    }

    /// Soft deletes a deployment object by setting its deleted_at timestamp to the current time.
    ///
    /// # Arguments
    ///
    /// * `deployment_object_uuid` - The UUID of the deployment object to soft delete.
    ///
    /// # Returns
    ///
    /// Returns a Result containing the number of affected rows (0 or 1) on success, or a diesel::result::Error on failure.
    pub fn soft_delete(&self, deployment_object_uuid: Uuid) -> Result<usize, diesel::result::Error> {
        let conn = &mut self.dal.pool.get().expect("Failed to get DB connection");
        diesel::update(deployment_objects::table.filter(deployment_objects::id.eq(deployment_object_uuid)))
            .set(deployment_objects::deleted_at.eq(Utc::now()))
            .execute(conn)
    }

    /// Retrieves the latest non-deleted deployment object for a specific stack.
    ///
    /// # Arguments
    ///
    /// * `stack_id` - The UUID of the stack to retrieve the latest deployment object for.
    ///
    /// # Returns
    ///
    /// Returns a Result containing an Option<DeploymentObject> if found, or a diesel::result::Error on failure.
    pub fn get_latest_for_stack(&self, stack_id: Uuid) -> Result<Option<DeploymentObject>, diesel::result::Error> {
        let conn = &mut self.dal.pool.get().expect("Failed to get DB connection");
        deployment_objects::table
            .filter(deployment_objects::stack_id.eq(stack_id))
            .filter(deployment_objects::deleted_at.is_null())
            .order(deployment_objects::sequence_id.desc())
            .first(conn)
            .optional()
    }
}