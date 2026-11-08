/*
 * Copyright (c) 2025-2026 Dylan Storey
 * Licensed under the Elastic License 2.0.
 * See LICENSE file in the project root for full license text.
 */

//! What an agent does, with its own PAK: the CLI as the agent, and the real
//! `brokkr-agent` binary at start-up.

use cucumber::when;

use crate::world::World;

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
