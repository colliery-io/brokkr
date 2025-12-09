/*
 * Copyright (c) 2025 Dylan Storey
 * Licensed under the Elastic License 2.0.
 * See LICENSE file in the project root for full license text.
 */

//! Data Access Layer for Rendered Deployment Object operations.
//!
//! This module provides functionality to interact with the rendered_deployment_objects
//! table in the database, tracking the provenance of deployment objects created from
//! templates.

use crate::dal::DAL;
use brokkr_models::models::rendered_deployment_objects::{
    NewRenderedDeploymentObject, RenderedDeploymentObject,
};
use brokkr_models::schema::rendered_deployment_objects;
use diesel::prelude::*;
use uuid::Uuid;

/// Handles database operations for RenderedDeploymentObject entities.
pub struct RenderedDeploymentObjectsDAL<'a> {
    /// Reference to the main DAL instance.
    pub dal: &'a DAL,
}

impl RenderedDeploymentObjectsDAL<'_> {
    /// Creates a new rendered deployment object provenance record in the database.
    ///
    /// # Arguments
    ///
    /// * `new_record` - The new provenance record details to be inserted.
    ///
    /// # Returns
    ///
    /// The created `RenderedDeploymentObject` on success, or a `diesel::result::Error` on failure.
    pub fn create(
        &self,
        new_record: &NewRenderedDeploymentObject,
    ) -> Result<RenderedDeploymentObject, diesel::result::Error> {
        let conn = &mut self.dal.pool.get().expect("Failed to get DB connection");
        diesel::insert_into(rendered_deployment_objects::table)
            .values(new_record)
            .get_result(conn)
    }

    /// Retrieves a rendered deployment object provenance record by its ID.
    ///
    /// # Arguments
    ///
    /// * `record_id` - The UUID of the record to retrieve.
    ///
    /// # Returns
    ///
    /// An `Option<RenderedDeploymentObject>` if found, or a `diesel::result::Error` on failure.
    pub fn get(
        &self,
        record_id: Uuid,
    ) -> Result<Option<RenderedDeploymentObject>, diesel::result::Error> {
        let conn = &mut self.dal.pool.get().expect("Failed to get DB connection");
        rendered_deployment_objects::table
            .filter(rendered_deployment_objects::id.eq(record_id))
            .first(conn)
            .optional()
    }

    /// Retrieves the provenance record for a specific deployment object.
    ///
    /// # Arguments
    ///
    /// * `deployment_object_id` - The UUID of the deployment object.
    ///
    /// # Returns
    ///
    /// An `Option<RenderedDeploymentObject>` if found, or a `diesel::result::Error` on failure.
    pub fn get_by_deployment_object(
        &self,
        deployment_object_id: Uuid,
    ) -> Result<Option<RenderedDeploymentObject>, diesel::result::Error> {
        let conn = &mut self.dal.pool.get().expect("Failed to get DB connection");
        rendered_deployment_objects::table
            .filter(rendered_deployment_objects::deployment_object_id.eq(deployment_object_id))
            .first(conn)
            .optional()
    }

    /// Lists all provenance records for a specific template.
    ///
    /// # Arguments
    ///
    /// * `template_id` - The UUID of the template.
    /// * `version` - Optional version filter. If provided, only returns records for that version.
    ///
    /// # Returns
    ///
    /// A `Vec<RenderedDeploymentObject>` for the specified template, or a `diesel::result::Error` on failure.
    pub fn list_by_template(
        &self,
        template_id: Uuid,
        version: Option<i32>,
    ) -> Result<Vec<RenderedDeploymentObject>, diesel::result::Error> {
        let conn = &mut self.dal.pool.get().expect("Failed to get DB connection");

        let mut query = rendered_deployment_objects::table
            .filter(rendered_deployment_objects::template_id.eq(template_id))
            .into_boxed();

        if let Some(v) = version {
            query = query.filter(rendered_deployment_objects::template_version.eq(v));
        }

        query
            .order(rendered_deployment_objects::created_at.desc())
            .load::<RenderedDeploymentObject>(conn)
    }

    /// Lists all provenance records from the database.
    ///
    /// # Returns
    ///
    /// A `Vec<RenderedDeploymentObject>` containing all records, or a `diesel::result::Error` on failure.
    pub fn list(&self) -> Result<Vec<RenderedDeploymentObject>, diesel::result::Error> {
        let conn = &mut self.dal.pool.get().expect("Failed to get DB connection");
        rendered_deployment_objects::table
            .order(rendered_deployment_objects::created_at.desc())
            .load::<RenderedDeploymentObject>(conn)
    }

    /// Deletes a provenance record from the database.
    ///
    /// # Arguments
    ///
    /// * `record_id` - The UUID of the record to delete.
    ///
    /// # Returns
    ///
    /// The number of affected rows (0 or 1) on success, or a `diesel::result::Error` on failure.
    pub fn delete(&self, record_id: Uuid) -> Result<usize, diesel::result::Error> {
        let conn = &mut self.dal.pool.get().expect("Failed to get DB connection");
        diesel::delete(
            rendered_deployment_objects::table.filter(rendered_deployment_objects::id.eq(record_id)),
        )
        .execute(conn)
    }

    /// Deletes all provenance records for a specific deployment object.
    ///
    /// # Arguments
    ///
    /// * `deployment_object_id` - The UUID of the deployment object.
    ///
    /// # Returns
    ///
    /// The number of affected rows on success, or a `diesel::result::Error` on failure.
    pub fn delete_for_deployment_object(
        &self,
        deployment_object_id: Uuid,
    ) -> Result<usize, diesel::result::Error> {
        let conn = &mut self.dal.pool.get().expect("Failed to get DB connection");
        diesel::delete(
            rendered_deployment_objects::table
                .filter(rendered_deployment_objects::deployment_object_id.eq(deployment_object_id)),
        )
        .execute(conn)
    }

    /// Deletes all provenance records for a specific template.
    ///
    /// # Arguments
    ///
    /// * `template_id` - The UUID of the template.
    ///
    /// # Returns
    ///
    /// The number of affected rows on success, or a `diesel::result::Error` on failure.
    pub fn delete_for_template(&self, template_id: Uuid) -> Result<usize, diesel::result::Error> {
        let conn = &mut self.dal.pool.get().expect("Failed to get DB connection");
        diesel::delete(
            rendered_deployment_objects::table
                .filter(rendered_deployment_objects::template_id.eq(template_id)),
        )
        .execute(conn)
    }
}
