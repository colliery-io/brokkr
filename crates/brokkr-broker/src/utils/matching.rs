/*
 * Copyright (c) 2025 Dylan Storey
 * Licensed under the Elastic License 2.0.
 * See LICENSE file in the project root for full license text.
 */

//! Template-to-stack matching utilities.
//!
//! This module provides functions for validating that a template's labels and
//! annotations are compatible with a target stack before instantiation.

use serde::Serialize;

/// Result of a template-to-stack matching operation.
#[derive(Debug, Default, Serialize)]
pub struct MatchResult {
    /// Whether the template matches the stack.
    pub matches: bool,
    /// Labels required by the template that are missing from the stack.
    pub missing_labels: Vec<String>,
    /// Annotations required by the template that are missing from the stack.
    pub missing_annotations: Vec<(String, String)>,
}

/// Check if a template can be instantiated into a stack.
///
/// # Matching Rules
///
/// - Template with NO labels/annotations = matches ANY stack (permissive)
/// - Template WITH labels = ALL labels must exist on stack
/// - Template WITH annotations = ALL annotations (key-value pairs) must exist on stack
///
/// # Arguments
///
/// * `template_labels` - Labels attached to the template
/// * `template_annotations` - Annotations attached to the template (key-value pairs)
/// * `stack_labels` - Labels attached to the target stack
/// * `stack_annotations` - Annotations attached to the target stack (key-value pairs)
///
/// # Returns
///
/// A `MatchResult` indicating whether the template matches and details about any
/// missing labels or annotations.
pub fn template_matches_stack(
    template_labels: &[String],
    template_annotations: &[(String, String)],
    stack_labels: &[String],
    stack_annotations: &[(String, String)],
) -> MatchResult {
    // If template has no labels/annotations, it matches everything (go anywhere)
    if template_labels.is_empty() && template_annotations.is_empty() {
        return MatchResult {
            matches: true,
            missing_labels: Vec::new(),
            missing_annotations: Vec::new(),
        };
    }

    // Check all template labels exist on stack
    let missing_labels: Vec<String> = template_labels
        .iter()
        .filter(|tl| !stack_labels.contains(tl))
        .cloned()
        .collect();

    // Check all template annotations exist on stack (exact key-value match)
    let missing_annotations: Vec<(String, String)> = template_annotations
        .iter()
        .filter(|ta| !stack_annotations.contains(ta))
        .cloned()
        .collect();

    MatchResult {
        matches: missing_labels.is_empty() && missing_annotations.is_empty(),
        missing_labels,
        missing_annotations,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_template_no_labels_matches_any_stack() {
        // Template with no labels should match any stack
        let result = template_matches_stack(
            &[],
            &[],
            &["env=prod".to_string(), "team=platform".to_string()],
            &[("region".to_string(), "us-east".to_string())],
        );
        assert!(result.matches);
        assert!(result.missing_labels.is_empty());
        assert!(result.missing_annotations.is_empty());
    }

    #[test]
    fn test_template_no_labels_matches_empty_stack() {
        // Template with no labels should match stack with no labels
        let result = template_matches_stack(&[], &[], &[], &[]);
        assert!(result.matches);
    }

    #[test]
    fn test_template_labels_subset_of_stack_matches() {
        // Template labels are subset of stack labels - should match
        let result = template_matches_stack(
            &["env=prod".to_string()],
            &[],
            &["env=prod".to_string(), "team=platform".to_string()],
            &[],
        );
        assert!(result.matches);
        assert!(result.missing_labels.is_empty());
    }

    #[test]
    fn test_template_labels_exact_match() {
        // Template labels exactly match stack labels - should match
        let result = template_matches_stack(
            &["env=prod".to_string(), "team=platform".to_string()],
            &[],
            &["env=prod".to_string(), "team=platform".to_string()],
            &[],
        );
        assert!(result.matches);
    }

    #[test]
    fn test_template_label_not_on_stack() {
        // Template has label that stack doesn't have - should not match
        let result = template_matches_stack(
            &["env=prod".to_string(), "critical".to_string()],
            &[],
            &["env=prod".to_string()],
            &[],
        );
        assert!(!result.matches);
        assert_eq!(result.missing_labels, vec!["critical".to_string()]);
    }

    #[test]
    fn test_template_multiple_missing_labels() {
        // Template has multiple labels that stack doesn't have
        let result = template_matches_stack(
            &["env=prod".to_string(), "critical".to_string(), "tier=1".to_string()],
            &[],
            &["env=prod".to_string()],
            &[],
        );
        assert!(!result.matches);
        assert_eq!(
            result.missing_labels,
            vec!["critical".to_string(), "tier=1".to_string()]
        );
    }

    #[test]
    fn test_annotation_exact_match() {
        // Template annotation exactly matches stack annotation - should match
        let result = template_matches_stack(
            &[],
            &[("region".to_string(), "us-east".to_string())],
            &[],
            &[("region".to_string(), "us-east".to_string())],
        );
        assert!(result.matches);
    }

    #[test]
    fn test_annotation_key_matches_value_differs() {
        // Annotation key matches but value differs - should not match
        let result = template_matches_stack(
            &[],
            &[("region".to_string(), "us-east".to_string())],
            &[],
            &[("region".to_string(), "us-west".to_string())],
        );
        assert!(!result.matches);
        assert_eq!(
            result.missing_annotations,
            vec![("region".to_string(), "us-east".to_string())]
        );
    }

    #[test]
    fn test_annotation_missing_entirely() {
        // Annotation key doesn't exist on stack - should not match
        let result = template_matches_stack(
            &[],
            &[("region".to_string(), "us-east".to_string())],
            &[],
            &[("environment".to_string(), "production".to_string())],
        );
        assert!(!result.matches);
        assert_eq!(
            result.missing_annotations,
            vec![("region".to_string(), "us-east".to_string())]
        );
    }

    #[test]
    fn test_mixed_labels_and_annotations_all_match() {
        // Both labels and annotations match - should match
        let result = template_matches_stack(
            &["env=prod".to_string()],
            &[("region".to_string(), "us-east".to_string())],
            &["env=prod".to_string(), "team=platform".to_string()],
            &[
                ("region".to_string(), "us-east".to_string()),
                ("owner".to_string(), "alice".to_string()),
            ],
        );
        assert!(result.matches);
    }

    #[test]
    fn test_mixed_labels_match_but_annotations_dont() {
        // Labels match but annotations don't - should not match
        let result = template_matches_stack(
            &["env=prod".to_string()],
            &[("region".to_string(), "us-east".to_string())],
            &["env=prod".to_string()],
            &[("region".to_string(), "us-west".to_string())],
        );
        assert!(!result.matches);
        assert!(result.missing_labels.is_empty());
        assert_eq!(
            result.missing_annotations,
            vec![("region".to_string(), "us-east".to_string())]
        );
    }

    #[test]
    fn test_annotations_match_but_labels_dont() {
        // Annotations match but labels don't - should not match
        let result = template_matches_stack(
            &["env=prod".to_string()],
            &[("region".to_string(), "us-east".to_string())],
            &["env=staging".to_string()],
            &[("region".to_string(), "us-east".to_string())],
        );
        assert!(!result.matches);
        assert_eq!(result.missing_labels, vec!["env=prod".to_string()]);
        assert!(result.missing_annotations.is_empty());
    }

    #[test]
    fn test_both_labels_and_annotations_missing() {
        // Both labels and annotations missing - returns all missing
        let result = template_matches_stack(
            &["env=prod".to_string()],
            &[("region".to_string(), "us-east".to_string())],
            &[],
            &[],
        );
        assert!(!result.matches);
        assert_eq!(result.missing_labels, vec!["env=prod".to_string()]);
        assert_eq!(
            result.missing_annotations,
            vec![("region".to_string(), "us-east".to_string())]
        );
    }
}
