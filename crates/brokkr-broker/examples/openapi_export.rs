/*
 * Copyright (c) 2025-2026 Dylan Storey
 * Licensed under the Elastic License 2.0.
 * See LICENSE file in the project root for full license text.
 */

//! Exports the broker's OpenAPI v1 spec as JSON to `openapi/brokkr-v1.json`
//! (relative to the workspace root) so it can be diffed, validated, and fed
//! into client SDK generators.
//!
//! utoipa 5.x emits OpenAPI 3.1.0 by default, but progenitor and most other
//! SDK generators currently only consume 3.0.x. This binary post-processes
//! the raw schema to a 3.0-compatible form: downgrades the version string and
//! rewrites `type: [X, "null"]` nullable shorthand into the `nullable: true`
//! form 3.0 generators expect.
//!
//! Run with: `cargo run -p brokkr-broker --example openapi_export`

use std::fs;
use std::path::PathBuf;

use brokkr_broker::api::v1::openapi::ApiDoc;
use serde_json::Value;
use utoipa::OpenApi;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut doc: Value = serde_json::from_str(&ApiDoc::openapi().to_json()?)?;
    downgrade_to_openapi_3_0(&mut doc);
    let json = serde_json::to_string_pretty(&doc)?;

    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .ancestors()
        .nth(2)
        .expect("workspace root above crates/brokkr-broker");
    let out_dir = workspace_root.join("openapi");
    fs::create_dir_all(&out_dir)?;
    let out_path = out_dir.join("brokkr-v1.json");
    fs::write(&out_path, &json)?;

    eprintln!("wrote {} ({} bytes)", out_path.display(), json.len());
    Ok(())
}

/// Rewrites the OpenAPI document in-place to be compatible with OpenAPI 3.0
/// tooling. Two transforms:
///
/// 1. Set the top-level `openapi` field to `"3.0.3"`.
/// 2. Convert every `type: [<primitive>, "null"]` to `type: <primitive>` plus
///    `nullable: true`. This is the 3.0 spelling of 3.1's `["X", "null"]`
///    union shorthand and is the form progenitor / openapi-generator expect.
fn downgrade_to_openapi_3_0(doc: &mut Value) {
    if let Some(obj) = doc.as_object_mut() {
        obj.insert("openapi".into(), Value::String("3.0.3".into()));
    }
    rewrite_nullable_types(doc);
}

fn rewrite_nullable_types(value: &mut Value) {
    match value {
        Value::Object(map) => {
            // OpenAPI 3.0 doesn't support `propertyNames`; JSON object keys
            // are always strings anyway, and the only constraint utoipa emits
            // here is `{type: "string"}`. Drop the field.
            map.remove("propertyNames");
            // Case 1: `type: [<primitive>, "null"]` -> `type: <primitive>` + `nullable: true`.
            let nullable_type = map.get("type").and_then(|t| t.as_array()).and_then(|arr| {
                if arr.len() != 2 {
                    return None;
                }
                let has_null = arr.iter().any(|v| v.as_str() == Some("null"));
                if !has_null {
                    return None;
                }
                arr.iter()
                    .find(|v| v.as_str().is_some() && v.as_str() != Some("null"))
                    .and_then(|v| v.as_str())
                    .map(String::from)
            });
            if let Some(primitive) = nullable_type {
                map.insert("type".into(), Value::String(primitive));
                map.insert("nullable".into(), Value::Bool(true));
            }

            // Case 2: nullable-via-union for $refs.
            //
            // utoipa 3.1 emits `oneOf: [{type: "null"}, X]` (or anyOf) for any
            // optional field whose inner type is a $ref. 3.0 doesn't allow
            // nullable on $refs at all; the closest compatible spelling
            // (and what progenitor accepts) is to unwrap to just X. We lose
            // the nullable hint at the type-system level, but the field is
            // already optional via `required`, so callers handle absence the
            // same way.
            for key in ["oneOf", "anyOf"] {
                if let Some(arr) = map.get(key).and_then(|v| v.as_array()) {
                    if arr.len() == 2 {
                        let null_idx = arr.iter().position(|v| {
                            v.as_object()
                                .and_then(|o| o.get("type"))
                                .and_then(|t| t.as_str())
                                == Some("null")
                        });
                        if let Some(idx) = null_idx {
                            let other = arr[1 - idx].clone();
                            map.remove(key);
                            if let Some(other_obj) = other.as_object() {
                                for (k, v) in other_obj {
                                    map.insert(k.clone(), v.clone());
                                }
                            }
                            break;
                        }
                    }
                }
            }

            for child in map.values_mut() {
                rewrite_nullable_types(child);
            }
        }
        Value::Array(arr) => {
            for child in arr {
                rewrite_nullable_types(child);
            }
        }
        _ => {}
    }
}
