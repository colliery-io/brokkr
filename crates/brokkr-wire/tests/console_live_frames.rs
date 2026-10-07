//! The operator console (`crates/brokkr-web`, outside this workspace) parses
//! the live stack stream with its own small serde structs, and its host tests
//! read `fixtures/console_live_frames.json`. This test keeps that fixture
//! equal to what this crate's serializer writes, so the console cannot drift
//! from the wire format without a failing test here.

use brokkr_wire::{GapReason, LogGap, PodLogLine, WsMessage};
use chrono::{DateTime, Utc};
use uuid::Uuid;

fn at(s: &str) -> DateTime<Utc> {
    DateTime::parse_from_rfc3339(s)
        .expect("time")
        .with_timezone(&Utc)
}

#[test]
fn the_console_fixture_is_what_the_wire_serializer_writes() {
    let stack_id = Uuid::parse_str("0b1c2d3e-4f5a-4b6c-8d7e-9f0a1b2c3d4e").expect("uuid");
    let frames = vec![
        WsMessage::PodLogLine(PodLogLine {
            agent_id: Uuid::parse_str("6f1d2c3b-4a59-4e8f-9b0a-1c2d3e4f5a6b").expect("uuid"),
            stack_id,
            namespace: "payments".into(),
            pod: "payments-api-7d9f4-x2k1".into(),
            container: "api".into(),
            ts: at("2026-10-07T12:04:53.123456789Z"),
            line: "POST /charge 201 48ms".into(),
        }),
        // The broker's synthetic gap for a lagging subscriber has a nil agent.
        WsMessage::LogGap(LogGap {
            agent_id: Uuid::nil(),
            stack_id,
            since_ts: at("2026-10-07T12:04:54Z"),
            dropped_count: 17,
            reason: GapReason::BufferFull,
        }),
    ];
    let written = serde_json::to_value(&frames).expect("serialize");
    let fixture: serde_json::Value =
        serde_json::from_str(include_str!("fixtures/console_live_frames.json")).expect("fixture");
    assert_eq!(written, fixture);
}
