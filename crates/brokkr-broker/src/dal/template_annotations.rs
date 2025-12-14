/*
 * Copyright (c) 2025 Dylan Storey
 * Licensed under the Elastic License 2.0.
 * See LICENSE file in the project root for full license text.
 */

//! Data Access Layer for Template Annotation operations.
//!
//! This module provides functionality to interact with template annotations in the database,
//! including creating, retrieving, listing, and deleting annotations.

use crate::dal::DAL;
use brokkr_models::models::template_annotations::{NewTemplateAnnotation, TemplateAnnotation};
use brokkr_models::schema::template_annotations;
use diesel::prelude::*;
use uuid::Uuid;

/// Handles database operations for Template Annotations.
pub struct TemplateAnnotationsDAL<'a> {
    /// Reference to the main DAL instance.
    pub dal: &'a DAL,
}

impl TemplateAnnotationsDAL<'_> {
    /// Creates a new template annotation in the database.
    ///
    /// # Arguments
    ///
    /// * `new_annotation` - The new annotation details to be inserted.
    ///
    /// # Returns
    ///
    /// The created `TemplateAnnotation` or a database error.
    ///
    /// # Errors
    ///
    /// Returns a `diesel::result::Error` if the database operation fails.
    pub fn create(
        &self,
        new_annotation: &NewTemplateAnnotation,
    ) -> Result<TemplateAnnotation, diesel::result::Error> {
        let conn = &mut self.dal.pool.get().expect("Failed to get DB connection");
        diesel::insert_into(template_annotations::table)
            .values(new_annotation)
            .get_result(conn)
    }

    /// Retrieves a template annotation by its ID.
    ///
    /// # Arguments
    ///
    /// * `annotation_id` - The UUID of the annotation to retrieve.
    ///
    /// # Returns
    ///
    /// An `Option<TemplateAnnotation>` if found, or `None` if not found.
    ///
    /// # Errors
    ///
    /// Returns a `diesel::result::Error` if the database operation fails.
    pub fn get(
        &self,
        annotation_id: Uuid,
    ) -> Result<Option<TemplateAnnotation>, diesel::result::Error> {
        let conn = &mut self.dal.pool.get().expect("Failed to get DB connection");
        template_annotations::table
            .filter(template_annotations::id.eq(annotation_id))
            .first(conn)
            .optional()
    }

    /// Lists all annotations for a specific template.
    ///
    /// # Arguments
    ///
    /// * `template_id` - The UUID of the template whose annotations to retrieve.
    ///
    /// # Returns
    ///
    /// A vector of `TemplateAnnotation`s associated with the specified template.
    ///
    /// # Errors
    ///
    /// Returns a `diesel::result::Error` if the database operation fails.
    pub fn list_for_template(
        &self,
        template_id: Uuid,
    ) -> Result<Vec<TemplateAnnotation>, diesel::result::Error> {
        let conn = &mut self.dal.pool.get().expect("Failed to get DB connection");
        template_annotations::table
            .filter(template_annotations::template_id.eq(template_id))
            .load::<TemplateAnnotation>(conn)
    }

    /// Deletes a template annotation from the database.
    ///
    /// # Arguments
    ///
    /// * `annotation_id` - The UUID of the annotation to delete.
    ///
    /// # Returns
    ///
    /// The number of affected rows (0 or 1).
    ///
    /// # Errors
    ///
    /// Returns a `diesel::result::Error` if the database operation fails.
    pub fn delete(&self, annotation_id: Uuid) -> Result<usize, diesel::result::Error> {
        let conn = &mut self.dal.pool.get().expect("Failed to get DB connection");
        diesel::delete(
            template_annotations::table.filter(template_annotations::id.eq(annotation_id)),
        )
        .execute(conn)
    }

    /// Deletes all annotations for a specific template.
    ///
    /// # Arguments
    ///
    /// * `template_id` - The UUID of the template whose annotations to delete.
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
        diesel::delete(
            template_annotations::table.filter(template_annotations::template_id.eq(template_id)),
        )
        .execute(conn)
    }
}
