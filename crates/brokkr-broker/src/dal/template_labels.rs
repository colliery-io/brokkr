/*
 * Copyright (c) 2025 Dylan Storey
 * Licensed under the Elastic License 2.0.
 * See LICENSE file in the project root for full license text.
 */

//! Data Access Layer for Template Label operations.
//!
//! This module provides functionality to interact with template labels in the database,
//! including creating, retrieving, listing, and deleting labels.

use crate::dal::DAL;
use brokkr_models::models::template_labels::{NewTemplateLabel, TemplateLabel};
use brokkr_models::schema::template_labels;
use diesel::prelude::*;
use uuid::Uuid;

/// Handles database operations for Template Labels.
pub struct TemplateLabelsDAL<'a> {
    /// Reference to the main DAL instance.
    pub dal: &'a DAL,
}

impl TemplateLabelsDAL<'_> {
    /// Creates a new template label in the database.
    ///
    /// # Arguments
    ///
    /// * `new_label` - The new label details to be inserted.
    ///
    /// # Returns
    ///
    /// The created `TemplateLabel` or a database error.
    ///
    /// # Errors
    ///
    /// Returns a `diesel::result::Error` if the database operation fails.
    pub fn create(
        &self,
        new_label: &NewTemplateLabel,
    ) -> Result<TemplateLabel, diesel::result::Error> {
        let conn = &mut self.dal.pool.get().expect("Failed to get DB connection");
        diesel::insert_into(template_labels::table)
            .values(new_label)
            .get_result(conn)
    }

    /// Retrieves a template label by its ID.
    ///
    /// # Arguments
    ///
    /// * `label_id` - The UUID of the label to retrieve.
    ///
    /// # Returns
    ///
    /// An `Option<TemplateLabel>` if found, or `None` if not found.
    ///
    /// # Errors
    ///
    /// Returns a `diesel::result::Error` if the database operation fails.
    pub fn get(&self, label_id: Uuid) -> Result<Option<TemplateLabel>, diesel::result::Error> {
        let conn = &mut self.dal.pool.get().expect("Failed to get DB connection");
        template_labels::table
            .filter(template_labels::id.eq(label_id))
            .first(conn)
            .optional()
    }

    /// Lists all labels for a specific template.
    ///
    /// # Arguments
    ///
    /// * `template_id` - The UUID of the template whose labels to retrieve.
    ///
    /// # Returns
    ///
    /// A vector of `TemplateLabel`s associated with the specified template.
    ///
    /// # Errors
    ///
    /// Returns a `diesel::result::Error` if the database operation fails.
    pub fn list_for_template(
        &self,
        template_id: Uuid,
    ) -> Result<Vec<TemplateLabel>, diesel::result::Error> {
        let conn = &mut self.dal.pool.get().expect("Failed to get DB connection");
        template_labels::table
            .filter(template_labels::template_id.eq(template_id))
            .load::<TemplateLabel>(conn)
    }

    /// Deletes a template label from the database.
    ///
    /// # Arguments
    ///
    /// * `label_id` - The UUID of the label to delete.
    ///
    /// # Returns
    ///
    /// The number of affected rows (0 or 1).
    ///
    /// # Errors
    ///
    /// Returns a `diesel::result::Error` if the database operation fails.
    pub fn delete(&self, label_id: Uuid) -> Result<usize, diesel::result::Error> {
        let conn = &mut self.dal.pool.get().expect("Failed to get DB connection");
        diesel::delete(template_labels::table.filter(template_labels::id.eq(label_id)))
            .execute(conn)
    }

    /// Deletes all labels for a specific template.
    ///
    /// # Arguments
    ///
    /// * `template_id` - The UUID of the template whose labels to delete.
    ///
    /// # Returns
    ///
    /// The number of affected rows.
    ///
    /// # Errors
    ///
    /// Returns a `diesel::result::Error` if the database operation fails.
    pub fn delete_all_for_template(
        &self,
        template_id: Uuid,
    ) -> Result<usize, diesel::result::Error> {
        let conn = &mut self.dal.pool.get().expect("Failed to get DB connection");
        diesel::delete(template_labels::table.filter(template_labels::template_id.eq(template_id)))
            .execute(conn)
    }
}
