/*
 * Copyright (c) 2025-2026 Dylan Storey
 * Licensed under the Elastic License 2.0.
 * See LICENSE file in the project root for full license text.
 */

/// Utility functions for the Brokkr agent.
use serde::de::Deserialize;
use std::error::Error;

/// Deserializes a multi-document YAML string into a vector of YAML values.
///
/// # Arguments
/// * `multi_doc_str` - String containing multiple YAML documents
///
/// # Returns
/// * `Result<Vec<serde_yaml::Value>, Box<dyn Error>>` - Vector of parsed YAML values or error
pub fn multidoc_deserialize(multi_doc_str: &str) -> Result<Vec<serde_yaml::Value>, Box<dyn Error>> {
    let mut docs = vec![];
    for d in serde_yaml::Deserializer::from_str(multi_doc_str) {
        docs.push(serde_yaml::Value::deserialize(d)?);
    }
    Ok(docs)
}

/// Extracts the unique Kubernetes namespaces referenced by a multi-document
/// YAML manifest, in first-seen order. Documents without an explicit
/// `metadata.namespace` contribute `"default"` (BROKKR-T-0190).
///
/// # Arguments
/// * `multi_doc_str` - String containing multiple YAML documents
///
/// # Returns
/// * `Vec<String>` - Unique namespaces; `["default"]` when the manifest is
///   empty or unparseable so callers always have somewhere to search
pub fn manifest_namespaces(multi_doc_str: &str) -> Vec<String> {
    let mut namespaces: Vec<String> = Vec::new();
    if let Ok(docs) = multidoc_deserialize(multi_doc_str) {
        for doc in docs {
            if doc.is_null() {
                continue;
            }
            let ns = doc
                .get("metadata")
                .and_then(|m| m.get("namespace"))
                .and_then(|n| n.as_str())
                .unwrap_or("default")
                .to_string();
            if !namespaces.contains(&ns) {
                namespaces.push(ns);
            }
        }
    }
    if namespaces.is_empty() {
        namespaces.push("default".to_string());
    }
    namespaces
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_manifest_namespaces() {
        let yaml = "---\napiVersion: v1\nkind: Service\nmetadata:\n  name: a\n  namespace: prod\n---\napiVersion: v1\nkind: ConfigMap\nmetadata:\n  name: b\n---\napiVersion: apps/v1\nkind: Deployment\nmetadata:\n  name: c\n  namespace: prod\n";
        assert_eq!(manifest_namespaces(yaml), vec!["prod", "default"]);
        assert_eq!(manifest_namespaces(""), vec!["default"]);
        assert_eq!(manifest_namespaces("not: [valid"), vec!["default"]);
    }

    #[test]
    fn test_multidoc_deserialize_success() {
        let multi_doc_yaml = r#"
---
key1: value1
key2: value2
---
- item1
- item2
- item3
"#;
        let result = multidoc_deserialize(multi_doc_yaml);
        assert!(result.is_ok());
        let docs = result.unwrap();
        assert_eq!(docs.len(), 2);
        assert_eq!(docs[0]["key1"], "value1");
        assert_eq!(docs[0]["key2"], "value2");
        assert_eq!(docs[1][0], "item1");
        assert_eq!(docs[1][1], "item2");
        assert_eq!(docs[1][2], "item3");
    }

    #[test]
    fn test_multidoc_deserialize_failure() {
        let invalid_yaml = r#"
---
key1: value1
key2: value2
---
- item1
- item2
- : invalid
"#;
        let result = multidoc_deserialize(invalid_yaml);
        assert!(result.is_err());
    }
}
