// src/dal/deployment_objects.rs

//! This module provides a Data Access Layer (DAL) for managing DeploymentObject entities in the database.
//!
//! It uses Diesel ORM for database operations and includes functionality for creating,
//! retrieving, updating, and soft-deleting deployment objects.

use diesel::prelude::*;
use uuid::Uuid;
use brokkr_models::models::deployment_objects::{DeploymentObject, NewDeploymentObject};
use crate::dal::DAL;

/// Represents the Data Access Layer for DeploymentObject-related operations.
pub struct DeploymentObjectsDAL<'a> {
    /// Reference to the main DAL instance.
    pub(crate) dal: &'a DAL,
}

impl<'a> DeploymentObjectsDAL<'a> {
    /// Creates a new deployment object in the database.
    ///
    /// # Arguments
    ///
    /// * `new_deployment_object` - A reference to the NewDeploymentObject struct containing the object details.
    ///
    /// # Returns
    ///
    /// Returns a QueryResult containing the created DeploymentObject on success, or an error on failure.
    pub fn create(&self, new_deployment_object: &NewDeploymentObject) -> QueryResult<DeploymentObject> {
        use brokkr_models::schema::deployment_objects::dsl::*;

        let conn = &mut self.dal.pool.get().expect("Failed to get DB connection");

        diesel::insert_into(deployment_objects)
            .values(new_deployment_object)
            .get_result(conn)
    }

    /// Retrieves a deployment object by its UUID.
    ///
    /// # Arguments
    ///
    /// * `object_uuid` - The UUID of the deployment object to retrieve.
    ///
    /// # Returns
    ///
    /// Returns a QueryResult containing the DeploymentObject if found, or an error if not found or on failure.
    pub fn get_by_id(&self, object_uuid: Uuid) -> QueryResult<DeploymentObject> {
        use brokkr_models::schema::deployment_objects::dsl::*;

        let conn = &mut self.dal.pool.get().expect("Failed to get DB connection");

        deployment_objects.filter(id.eq(object_uuid)).first(conn)
    }

    /// Retrieves all deployment objects for a given stack, ordered by sequence_id.
    ///
    /// # Arguments
    ///
    /// * `stack_id_param` - The UUID of the stack to retrieve deployment objects for.
    ///
    /// # Returns
    ///
    /// Returns a QueryResult containing a Vec of DeploymentObjects for the given stack on success, or an error on failure.
    pub fn get_by_stack_id(&self, stack_uuid: Uuid) -> QueryResult<Vec<DeploymentObject>> {
        use brokkr_models::schema::deployment_objects::dsl::*;

        let conn = &mut self.dal.pool.get().expect("Failed to get DB connection");

        deployment_objects
            .filter(stack_id.eq(stack_uuid))
            .order(sequence_id.asc())
            .load(conn)
    }

    /// Updates an existing deployment object.
    ///
    /// # Arguments
    ///
    /// * `object_uuid` - The UUID of the deployment object to update.
    /// * `updated_object` - A reference to the DeploymentObject struct containing the updated details.
    ///
    /// # Returns
    ///
    /// Returns a QueryResult containing the updated DeploymentObject on success, or an error on failure.
    pub fn update(&self, object_uuid: Uuid, updated_object: &DeploymentObject) -> QueryResult<DeploymentObject> {
        use brokkr_models::schema::deployment_objects::dsl::*;

        let conn = &mut self.dal.pool.get().expect("Failed to get DB connection");

        diesel::update(deployment_objects.filter(id.eq(object_uuid)))
            .set(updated_object)
            .get_result(conn)
    }

    /// Soft deletes a deployment object by setting its deleted_at timestamp to the current time.
    ///
    /// # Arguments
    ///
    /// * `object_uuid` - The UUID of the deployment object to soft delete.
    ///
    /// # Returns
    ///
    /// Returns a QueryResult containing the soft-deleted DeploymentObject on success, or an error on failure.
    pub fn soft_delete(&self, object_uuid: Uuid) -> QueryResult<DeploymentObject> {
        use brokkr_models::schema::deployment_objects::dsl::*;
        use diesel::dsl::now;

        let conn = &mut self.dal.pool.get().expect("Failed to get DB connection");

        diesel::update(deployment_objects.filter(id.eq(object_uuid)))
            .set(deleted_at.eq(now))
            .get_result(conn)
    }

    /// Retrieves all active (non-deleted) deployment objects, ordered by sequence_id.
    ///
    /// # Returns
    ///
    /// Returns a QueryResult containing a Vec of active DeploymentObjects on success, or an error on failure.
    pub fn get_active(&self) -> QueryResult<Vec<DeploymentObject>> {
        use brokkr_models::schema::deployment_objects::dsl::*;

        let conn = &mut self.dal.pool.get().expect("Failed to get DB connection");

        deployment_objects
            .filter(deleted_at.is_null())
            .order(sequence_id.asc())
            .load(conn)
    }
}