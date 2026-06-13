/*
 * Copyright (c) 2025-2026 Dylan Storey
 * Licensed under the Elastic License 2.0.
 * See LICENSE file in the project root for full license text.
 */

//! Golden + round-trip tests for the `WsMessage` enum.
//!
//! The golden fixture in `tests/fixtures/ws_message_v1.json` is the canonical
//! on-wire shape for v1. Changes to it are intentional and require a release
//! bump per `project_release_versioning`. Accidental changes (a field rename
//! or a tag rename) will fail this test.

use brokkr_wire::*;
use chrono::TimeZone;
use serde_json::Value;
use uuid::Uuid;

/// Build a deterministic sample of every `WsMessage` variant. Values are
/// fixed (no `now()`, no random UUIDs) so the serialized output is stable.
fn sample_messages() -> Vec<WsMessage> {
    let agent_id = Uuid::parse_str("11111111-1111-1111-1111-111111111111").unwrap();
    let stack_id = Uuid::parse_str("22222222-2222-2222-2222-222222222222").unwrap();
    let work_order_id = Uuid::parse_str("33333333-3333-3333-3333-333333333333").unwrap();
    let target_id = Uuid::parse_str("44444444-4444-4444-4444-444444444444").unwrap();
    let generator_id = Uuid::parse_str("55555555-5555-5555-5555-555555555555").unwrap();
    let event_id = Uuid::parse_str("66666666-6666-6666-6666-666666666666").unwrap();
    let deployment_object_id = Uuid::parse_str("77777777-7777-7777-7777-777777777777").unwrap();
    let health_id = Uuid::parse_str("88888888-8888-8888-8888-888888888888").unwrap();
    let ts = chrono::Utc.with_ymd_and_hms(2026, 1, 1, 0, 0, 0).unwrap();

    vec![
        WsMessage::WorkOrder(WorkOrder {
            id: work_order_id,
            created_at: ts,
            updated_at: ts,
            work_type: "build".to_string(),
            yaml_content: "kind: Build\n".to_string(),
            status: "PENDING".to_string(),
            claimed_by: None,
            claimed_at: None,
            claim_timeout_seconds: 3600,
            max_retries: 3,
            retry_count: 0,
            backoff_seconds: 60,
            next_retry_after: None,
            last_error: None,
            last_error_at: None,
        }),
        WsMessage::TargetChanged(AgentTarget {
            id: target_id,
            agent_id,
            stack_id,
        }),
        WsMessage::StackChanged(Stack {
            id: stack_id,
            created_at: ts,
            updated_at: ts,
            deleted_at: None,
            name: "demo".to_string(),
            description: None,
            generator_id,
        }),
        WsMessage::Heartbeat(Heartbeat {
            agent_id,
            sent_at: ts,
            k8s_reachable: None,
            k8s_api_latency_ms: None,
        }),
        WsMessage::AgentEvent(AgentEvent {
            id: event_id,
            created_at: ts,
            updated_at: ts,
            deleted_at: None,
            agent_id,
            deployment_object_id,
            event_type: "DEPLOYMENT".to_string(),
            status: "SUCCESS".to_string(),
            message: Some("ok".to_string()),
        }),
        WsMessage::AgentHealth(DeploymentHealth {
            id: health_id,
            agent_id,
            deployment_object_id,
            status: "healthy".to_string(),
            summary: None,
            checked_at: ts,
            created_at: ts,
            updated_at: ts,
        }),
        WsMessage::K8sEvent(K8sEvent {
            agent_id,
            stack_id,
            observed_at: ts,
            reason: "Pulled".to_string(),
            message: "Successfully pulled image".to_string(),
            event_type: "Normal".to_string(),
            source: Some("kubelet".to_string()),
            involved_object: ObjectRef {
                api_version: "v1".to_string(),
                kind: "Pod".to_string(),
                namespace: Some("demo".to_string()),
                name: "demo-abc".to_string(),
                uid: Some("99999999-9999-9999-9999-999999999999".to_string()),
            },
        }),
        WsMessage::PodLogLine(PodLogLine {
            agent_id,
            stack_id,
            namespace: "demo".to_string(),
            pod: "demo-abc".to_string(),
            container: "app".to_string(),
            ts,
            line: "starting up".to_string(),
        }),
        WsMessage::LogGap(LogGap {
            agent_id,
            stack_id,
            since_ts: ts,
            dropped_count: 42,
            reason: GapReason::RateLimit,
        }),
    ]
}

#[test]
fn every_variant_roundtrips() {
    for msg in sample_messages() {
        let encoded = serde_json::to_value(&msg).expect("serialize");
        let decoded: WsMessage = serde_json::from_value(encoded.clone()).expect("deserialize");
        let re_encoded = serde_json::to_value(&decoded).expect("re-serialize");
        assert_eq!(
            encoded, re_encoded,
            "round-trip diverged for variant {:?}",
            msg
        );
    }
}

#[test]
fn variant_tags_are_snake_case() {
    // Spot-check that the external tag matches the documented wire shape.
    // If anyone renames a variant without intending to change the wire,
    // this fails loudly.
    let expected_tags = [
        "work_order",
        "target_changed",
        "stack_changed",
        "heartbeat",
        "agent_event",
        "agent_health",
        "k8s_event",
        "pod_log_line",
        "log_gap",
    ];

    let actual_tags: Vec<String> = sample_messages()
        .iter()
        .map(|m| {
            serde_json::to_value(m).unwrap()["type"]
                .as_str()
                .unwrap()
                .to_string()
        })
        .collect();

    assert_eq!(actual_tags, expected_tags);
}

#[test]
fn golden_fixture_matches_current_serialization() {
    let golden_raw = include_str!("fixtures/ws_message_v1.json");
    let golden: Value = serde_json::from_str(golden_raw).expect("golden parses");

    let current: Value = serde_json::to_value(sample_messages()).expect("serialize sample");

    if golden != current {
        let pretty = serde_json::to_string_pretty(&current).unwrap();
        panic!(
            "Golden wire fixture drifted. If this change is intentional, update \
             tests/fixtures/ws_message_v1.json and bump the wire/release version \
             per project_release_versioning.\n\n--- current serialization ---\n{}\n",
            pretty
        );
    }
}

#[test]
fn wire_version_is_pinned() {
    // Sanity: the protocol version comes from CARGO_PKG_VERSION; if someone
    // ever hardcodes it, this catches the divergence.
    assert_eq!(WIRE_VERSION, env!("CARGO_PKG_VERSION"));
    assert!(!WIRE_VERSION.is_empty());
}
