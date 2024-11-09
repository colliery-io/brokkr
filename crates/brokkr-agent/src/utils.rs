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

#[cfg(test)]
mod tests {
    use super::*;

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
