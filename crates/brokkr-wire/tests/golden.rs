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
            owner_generator_id: None,
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
            stack_id: Some(stack_id),
            work_order_id: None,
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
            stack_id: Some(stack_id),
            work_order_id: None,
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
        WsMessage::FleetUpdate(FleetAgentRecord {
            agent_id,
            name: "demo-agent".to_string(),
            cluster_name: "demo-cluster".to_string(),
            status: "ACTIVE".to_string(),
            ws_connected: true,
            connected_since: Some(ts),
            last_heartbeat: Some(ts),
            heartbeat_age_seconds: Some(0),
            pending_object_count: 0,
            pending_work_orders: 0,
            claimed_work_orders: 0,
            last_event_at: Some(ts),
            seconds_since_last_event: Some(0),
            health_failing: 0,
            health_degraded: 0,
            k8s_reachable: Some(true),
            k8s_api_latency_ms: Some(12),
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
        "fleet_update",
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

/// A broker from before 0.12.0 sends a work order with no owner field
/// (BROKKR-T-0402). It still parses: the owner reads as an admin's.
#[test]
fn a_work_order_with_no_owner_field_still_parses() {
    let old = serde_json::json!({
        "id": "33333333-3333-3333-3333-333333333333",
        "created_at": "2026-01-01T00:00:00Z",
        "updated_at": "2026-01-01T00:00:00Z",
        "work_type": "build",
        "yaml_content": "kind: Build\n",
        "status": "PENDING",
        "claimed_by": null,
        "claimed_at": null,
        "claim_timeout_seconds": 3600,
        "max_retries": 3,
        "retry_count": 0,
        "backoff_seconds": 60,
        "next_retry_after": null,
        "last_error": null,
        "last_error_at": null
    });
    let work_order: WorkOrder = serde_json::from_value(old).expect("the old shape parses");
    assert_eq!(work_order.owner_generator_id, None);
}

/// An agent from before 0.12.0 sends a kube event and a pod log line with a
/// stack id and no work order id (BROKKR-T-0405). Both still parse, and they
/// are about the stack.
#[test]
fn telemetry_frames_of_the_old_shape_still_parse() {
    let stack_id = Uuid::parse_str("22222222-2222-2222-2222-222222222222").unwrap();
    let old_event = serde_json::json!({
        "type": "k8s_event",
        "body": {
            "agent_id": "11111111-1111-1111-1111-111111111111",
            "stack_id": stack_id,
            "observed_at": "2026-01-01T00:00:00Z",
            "reason": "Pulled",
            "message": "Successfully pulled image",
            "event_type": "Normal",
            "source": "kubelet",
            "involved_object": {
                "api_version": "v1",
                "kind": "Pod",
                "namespace": "demo",
                "name": "demo-abc",
                "uid": null
            }
        }
    });
    let old_line = serde_json::json!({
        "type": "pod_log_line",
        "body": {
            "agent_id": "11111111-1111-1111-1111-111111111111",
            "stack_id": stack_id,
            "namespace": "demo",
            "pod": "demo-abc",
            "container": "app",
            "ts": "2026-01-01T00:00:00Z",
            "line": "starting up"
        }
    });

    let WsMessage::K8sEvent(event) = serde_json::from_value(old_event).expect("the old event")
    else {
        panic!("wrong variant")
    };
    assert_eq!(event.stack_id, Some(stack_id));
    assert_eq!(event.work_order_id, None);

    let WsMessage::PodLogLine(line) = serde_json::from_value(old_line).expect("the old line")
    else {
        panic!("wrong variant")
    };
    assert_eq!(line.stack_id, Some(stack_id));
    assert_eq!(line.work_order_id, None);
}

/// A frame of a work order has a work order id and no stack id.
#[test]
fn a_telemetry_frame_of_a_work_order_round_trips() {
    let work_order_id = Uuid::parse_str("33333333-3333-3333-3333-333333333333").unwrap();
    let frame = WsMessage::PodLogLine(PodLogLine {
        agent_id: Uuid::parse_str("11111111-1111-1111-1111-111111111111").unwrap(),
        stack_id: None,
        work_order_id: Some(work_order_id),
        namespace: "jobs".to_string(),
        pod: "job-x".to_string(),
        container: "main".to_string(),
        ts: chrono::Utc.with_ymd_and_hms(2026, 1, 1, 0, 0, 0).unwrap(),
        line: "hello".to_string(),
    });
    let json = serde_json::to_value(&frame).expect("serialize");
    assert!(json["body"]["stack_id"].is_null());
    assert_eq!(json["body"]["work_order_id"], work_order_id.to_string());
    let WsMessage::PodLogLine(back) = serde_json::from_value(json).expect("parse") else {
        panic!("wrong variant")
    };
    assert_eq!(back.stack_id, None);
    assert_eq!(back.work_order_id, Some(work_order_id));
    assert_eq!(back.line, "hello");
}
