// src/dal/deployment_objects.rs

use diesel::prelude::*;
use uuid::Uuid;
use brokkr_models::models::deployment_objects::{DeploymentObject, NewDeploymentObject};
use crate::dal::DAL;

pub struct DeploymentObjectsDAL<'a> {
    pub(crate) dal: &'a DAL,
}

impl<'a> DeploymentObjectsDAL<'a> {
    /// Create a new deployment object in the database
    pub fn create(&self, new_deployment_object: &NewDeploymentObject) -> QueryResult<DeploymentObject> {
        use brokkr_models::schema::deployment_objects::dsl::*;

        let conn = &mut self.dal.pool.get().expect("Failed to get DB connection");

        diesel::insert_into(deployment_objects)
            .values(new_deployment_object)
            .get_result(conn)
    }

    /// Retrieve a deployment object by its UUID
    pub fn get_by_id(&self, object_uuid: Uuid) -> QueryResult<DeploymentObject> {
        use brokkr_models::schema::deployment_objects::dsl::*;

        let conn = &mut self.dal.pool.get().expect("Failed to get DB connection");

        deployment_objects.filter(uuid.eq(object_uuid)).first(conn)
    }

    /// Retrieve all deployment objects for a given stack
    pub fn get_by_stack_id(&self, stack_id_param: Uuid) -> QueryResult<Vec<DeploymentObject>> {
        use brokkr_models::schema::deployment_objects::dsl::*;

        let conn = &mut self.dal.pool.get().expect("Failed to get DB connection");

        deployment_objects
            .filter(stack_id.eq(stack_id_param))
            .order(sequence_id.asc())
            .load(conn)
    }

    /// Update an existing deployment object
    pub fn update(&self, object_uuid: Uuid, updated_object: &DeploymentObject) -> QueryResult<DeploymentObject> {
        use brokkr_models::schema::deployment_objects::dsl::*;

        let conn = &mut self.dal.pool.get().expect("Failed to get DB connection");

        diesel::update(deployment_objects.filter(uuid.eq(object_uuid)))
            .set(updated_object)
            .get_result(conn)
    }

    /// Soft delete a deployment object by setting its deleted_at timestamp
    pub fn soft_delete(&self, object_uuid: Uuid) -> QueryResult<DeploymentObject> {
        use brokkr_models::schema::deployment_objects::dsl::*;
        use diesel::dsl::now;

        let conn = &mut self.dal.pool.get().expect("Failed to get DB connection");

        diesel::update(deployment_objects.filter(uuid.eq(object_uuid)))
            .set(deleted_at.eq(now))
            .get_result(conn)
    }

    /// Retrieve all active (non-deleted) deployment objects
    pub fn get_active(&self) -> QueryResult<Vec<DeploymentObject>> {
        use brokkr_models::schema::deployment_objects::dsl::*;

        let conn = &mut self.dal.pool.get().expect("Failed to get DB connection");

        deployment_objects
            .filter(deleted_at.is_null())
            .order(sequence_id.asc())
            .load(conn)
    }
}