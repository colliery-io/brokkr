/*
 * Copyright (c) 2025 Dylan Storey
 * Licensed under the Elastic License 2.0.
 * See LICENSE file in the project root for full license text.
 */

//! Data Access Layer for TemplateTarget operations.
//!
//! This module provides functionality to interact with the template_targets table
//! in the database, allowing CRUD operations on TemplateTarget entities.

use crate::dal::DAL;
use brokkr_models::models::template_targets::{NewTemplateTarget, TemplateTarget};
use brokkr_models::schema::template_targets;
use diesel::prelude::*;
use uuid::Uuid;

/// Handles database operations for TemplateTarget entities.
pub struct TemplateTargetsDAL<'a> {
    /// Reference to the main DAL instance.
    pub dal: &'a DAL,
}

impl TemplateTargetsDAL<'_> {
    /// Creates a new template target in the database.
    ///
    /// # Arguments
    ///
    /// * `new_target` - The new template target details to be inserted.
    ///
    /// # Returns
    ///
    /// The created `TemplateTarget` on success, or a `diesel::result::Error` on failure.
    pub fn create(
        &self,
        new_target: &NewTemplateTarget,
    ) -> Result<TemplateTarget, diesel::result::Error> {
        let conn = &mut self.dal.pool.get().expect("Failed to get DB connection");
        diesel::insert_into(template_targets::table)
            .values(new_target)
            .get_result(conn)
    }

    /// Retrieves a template target by its ID.
    ///
    /// # Arguments
    ///
    /// * `target_id` - The UUID of the template target to retrieve.
    ///
    /// # Returns
    ///
    /// An `Option<TemplateTarget>` if found, or a `diesel::result::Error` on failure.
    pub fn get(&self, target_id: Uuid) -> Result<Option<TemplateTarget>, diesel::result::Error> {
        let conn = &mut self.dal.pool.get().expect("Failed to get DB connection");
        template_targets::table
            .filter(template_targets::id.eq(target_id))
            .first(conn)
            .optional()
    }

    /// Lists all template targets from the database.
    ///
    /// # Returns
    ///
    /// A `Vec<TemplateTarget>` containing all template targets, or a `diesel::result::Error` on failure.
    pub fn list(&self) -> Result<Vec<TemplateTarget>, diesel::result::Error> {
        let conn = &mut self.dal.pool.get().expect("Failed to get DB connection");
        template_targets::table.load::<TemplateTarget>(conn)
    }

    /// Lists all template targets for a specific template.
    ///
    /// # Arguments
    ///
    /// * `template_id` - The UUID of the template to list targets for.
    ///
    /// # Returns
    ///
    /// A `Vec<TemplateTarget>` for the specified template, or a `diesel::result::Error` on failure.
    pub fn list_for_template(
        &self,
        template_id: Uuid,
    ) -> Result<Vec<TemplateTarget>, diesel::result::Error> {
        let conn = &mut self.dal.pool.get().expect("Failed to get DB connection");
        template_targets::table
            .filter(template_targets::template_id.eq(template_id))
            .load::<TemplateTarget>(conn)
    }

    /// Lists all template targets for a specific stack.
    ///
    /// # Arguments
    ///
    /// * `stack_id` - The UUID of the stack to list targets for.
    ///
    /// # Returns
    ///
    /// A `Vec<TemplateTarget>` for the specified stack, or a `diesel::result::Error` on failure.
    pub fn list_for_stack(
        &self,
        stack_id: Uuid,
    ) -> Result<Vec<TemplateTarget>, diesel::result::Error> {
        let conn = &mut self.dal.pool.get().expect("Failed to get DB connection");
        template_targets::table
            .filter(template_targets::stack_id.eq(stack_id))
            .load::<TemplateTarget>(conn)
    }

    /// Checks if a specific template-stack association exists.
    ///
    /// # Arguments
    ///
    /// * `template_id` - The UUID of the template.
    /// * `stack_id` - The UUID of the stack.
    ///
    /// # Returns
    ///
    /// `true` if the association exists, `false` otherwise.
    pub fn exists(
        &self,
        template_id: Uuid,
        stack_id: Uuid,
    ) -> Result<bool, diesel::result::Error> {
        let conn = &mut self.dal.pool.get().expect("Failed to get DB connection");
        let count: i64 = template_targets::table
            .filter(template_targets::template_id.eq(template_id))
            .filter(template_targets::stack_id.eq(stack_id))
            .count()
            .get_result(conn)?;
        Ok(count > 0)
    }

    /// Deletes a template target from the database.
    ///
    /// # Arguments
    ///
    /// * `target_id` - The UUID of the template target to delete.
    ///
    /// # Returns
    ///
    /// The number of affected rows (0 or 1) on success, or a `diesel::result::Error` on failure.
    pub fn delete(&self, target_id: Uuid) -> Result<usize, diesel::result::Error> {
        let conn = &mut self.dal.pool.get().expect("Failed to get DB connection");
        diesel::delete(template_targets::table.filter(template_targets::id.eq(target_id)))
            .execute(conn)
    }

    /// Deletes all template targets for a specific template.
    ///
    /// # Arguments
    ///
    /// * `template_id` - The UUID of the template to delete targets for.
    ///
    /// # Returns
    ///
    /// The number of affected rows on success, or a `diesel::result::Error` on failure.
    pub fn delete_for_template(&self, template_id: Uuid) -> Result<usize, diesel::result::Error> {
        let conn = &mut self.dal.pool.get().expect("Failed to get DB connection");
        diesel::delete(
            template_targets::table.filter(template_targets::template_id.eq(template_id)),
        )
        .execute(conn)
    }

    /// Deletes all template targets for a specific stack.
    ///
    /// # Arguments
    ///
    /// * `stack_id` - The UUID of the stack to delete targets for.
    ///
    /// # Returns
    ///
    /// The number of affected rows on success, or a `diesel::result::Error` on failure.
    pub fn delete_for_stack(&self, stack_id: Uuid) -> Result<usize, diesel::result::Error> {
        let conn = &mut self.dal.pool.get().expect("Failed to get DB connection");
        diesel::delete(template_targets::table.filter(template_targets::stack_id.eq(stack_id)))
            .execute(conn)
    }
}
