/*
 * Copyright (c) 2025-2026 Dylan Storey
 * Licensed under the Elastic License 2.0.
 * See LICENSE file in the project root for full license text.
 */

//! What an agent does, with its own PAK: the CLI as the agent, the real
//! `brokkr-agent` binary at start-up, and the requests a running agent sends
//! (a heartbeat, a health report, a read of what it serves), which the
//! scenario sends in its place.

use std::time::Duration;

use cucumber::{given, when};
use serde_json::Value;
use uuid::Uuid;

use crate::api;
use crate::world::World;

/// The deployment object the agent serves for `stack`: the one of that stack
/// in the agent's full target state.
pub async fn served_object(w: &World, agent: &str, stack: &str) -> Option<Uuid> {
    let stack_id = w.stack_id(stack).await.to_string();
    let record = w.agent(agent);
    let state = w
        .api
        .target_state(&record.pak, record.id)
        .await
        .expect("the agent reads its target state");
    state
        .iter()
        .find(|o| o.get("stack_id").and_then(Value::as_str) == Some(stack_id.as_str()))
        .and_then(api::id_of)
}

#[when(expr = "the agent {string} asks who it is")]
async fn asks_who(w: &mut World, agent: String) {
    let pak = w.agent(&agent).pak.clone();
    w.brokkr(&pak, &["whoami"]).await;
}

#[when(expr = "the agent {string} starts with its PAK and no name or cluster")]
async fn starts_with_pak_only(w: &mut World, agent: String) {
    let pak = w.agent(&agent).pak.clone();
    w.brokkr_agent(&pak, None, None).await;
}

#[when(expr = "the agent {string} starts with its PAK and the name {string}")]
async fn starts_with_name(w: &mut World, agent: String, name: String) {
    let pak = w.agent(&agent).pak.clone();
    let name = w.real(&name);
    w.brokkr_agent(&pak, Some(&name), None).await;
}

#[when(expr = "the agent {string} starts with its PAK and the cluster {string}")]
async fn starts_with_cluster(w: &mut World, agent: String, cluster: String) {
    let pak = w.agent(&agent).pak.clone();
    w.brokkr_agent(&pak, None, Some(&cluster)).await;
}

#[when(expr = "the agent {string} sends a heartbeat")]
async fn sends_heartbeat(w: &mut World, agent: String) {
    let record = w.agent(&agent);
    w.api
        .heartbeat(&record.pak, record.id)
        .await
        .expect("the agent sends a heartbeat");
}

#[when(expr = "the agent {string} stays silent for {int} seconds")]
async fn stays_silent(_w: &mut World, _agent: String, seconds: u64) {
    tokio::time::sleep(Duration::from_secs(seconds)).await;
}

#[given(expr = "the agent {string} has reported the stack {string} as {word}")]
async fn has_reported(w: &mut World, agent: String, stack: String, status: String) {
    let object = served_object(w, &agent, &stack)
        .await
        .unwrap_or_else(|| panic!("the agent {agent:?} does not serve the stack {stack:?}"));
    let record = w.agent(&agent);
    w.api
        .report_health(&record.pak, record.id, object, &status)
        .await
        .expect("the agent reports the health");
}

/// The id comes from the admin's view of the stack, so the step asks one
/// thing: does the broker let this agent read the object.
#[when(expr = "the agent {string} reads the deployment object of the stack {string} by its id")]
async fn reads_object_by_id(w: &mut World, agent: String, stack: String) {
    let object = served_object(w, &agent, &stack)
        .await
        .unwrap_or_else(|| panic!("the agent {agent:?} does not serve the stack {stack:?}"));
    let pak = w.agent(&agent).pak.clone();
    let answer = w
        .api
        .deployment_object(&pak, object)
        .await
        .expect("ask the broker for the deployment object");
    w.answer = Some(answer);
}
