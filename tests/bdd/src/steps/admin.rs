/*
 * Copyright (c) 2025-2026 Dylan Storey
 * Licensed under the Elastic License 2.0.
 * See LICENSE file in the project root for full license text.
 */

//! What the admin does, with the admin PAK.

use cucumber::{given, when};

use crate::api;
use crate::world::{AgentRecord, Generator, World};

/// `brokkr generator create`; the created generator is kept with its PAK.
async fn create_generator(w: &mut World, name: &str) {
    let real = w.real(name);
    let admin = w.admin().to_string();
    w.brokkr(
        &admin,
        &["generator", "create", &real, "--description", "bdd"],
    )
    .await;
    if w.last().ok {
        let id = w.printed_id("generator", &real);
        let pak = w.printed_pak();
        w.generators.insert(
            name.to_string(),
            Generator {
                id,
                name: real,
                pak,
                previous_paks: Vec::new(),
            },
        );
    }
}

#[given(expr = "the admin has created the generator {string}")]
async fn has_created_generator(w: &mut World, name: String) {
    create_generator(w, &name).await;
    w.last_ok();
}

#[when(expr = "the admin creates the generator {string}")]
async fn creates_generator(w: &mut World, name: String) {
    create_generator(w, &name).await;
}

#[when(expr = "the admin creates the generator {string} again")]
async fn creates_generator_again(w: &mut World, name: String) {
    let real = w.real(&name);
    let admin = w.admin().to_string();
    w.brokkr(&admin, &["generator", "create", &real]).await;
}

/// `brokkr agent create`, with `--generator` for each generator; the created agent
/// is kept with its PAK.
async fn create_agent(w: &mut World, name: &str, cluster: &str, generators: &[&str]) {
    let real = w.real(name);
    let admin = w.admin().to_string();
    let mut args = vec!["agent", "create", real.as_str(), "--cluster", cluster];
    let generator_names: Vec<String> = generators
        .iter()
        .map(|t| w.generator(t).name.clone())
        .collect();
    for t in &generator_names {
        args.push("--generator");
        args.push(t);
    }
    w.brokkr(&admin, &args).await;
    if w.last().ok {
        let id = w.printed_id("agent", &real);
        let pak = w.printed_pak();
        w.agents.insert(
            name.to_string(),
            AgentRecord {
                id,
                name: real,
                cluster: cluster.to_string(),
                pak,
            },
        );
    }
}

#[given(expr = "the admin has created the agent {string} in cluster {string}")]
async fn has_created_agent(w: &mut World, name: String, cluster: String) {
    create_agent(w, &name, &cluster, &[]).await;
    w.last_ok();
}

#[given(
    expr = "the admin has created the agent {string} in cluster {string} for the generator {string}"
)]
async fn has_created_agent_for_generator(
    w: &mut World,
    name: String,
    cluster: String,
    generator: String,
) {
    create_agent(w, &name, &cluster, &[&generator]).await;
    w.last_ok();
}

#[when(
    expr = "the admin creates the agent {string} in cluster {string} for the generator {string}"
)]
async fn creates_agent_for_generator(
    w: &mut World,
    name: String,
    cluster: String,
    generator: String,
) {
    create_agent(w, &name, &cluster, &[&generator]).await;
}

#[given(expr = "the admin has registered the agent {string} with the generator {string}")]
#[when(expr = "the admin registers the agent {string} with the generator {string}")]
async fn registers_agent(w: &mut World, agent: String, generator: String) {
    let agent = w.agent(&agent).name.clone();
    let generator = w.generator(&generator).name.clone();
    let admin = w.admin().to_string();
    w.brokkr(
        &admin,
        &["register", "--agent", &agent, "--generator", &generator],
    )
    .await;
}

#[given(expr = "the admin has labelled the agent {string} with {string}")]
async fn has_labelled_agent(w: &mut World, agent: String, label: String) {
    let agent = w.agent(&agent).name.clone();
    let admin = w.admin().to_string();
    w.brokkr(&admin, &["agent", "label", &agent, &label]).await;
    w.last_ok();
}

#[given(expr = "the admin has activated the agent {string}")]
#[when(expr = "the admin activates the agent {string}")]
async fn activates_agent(w: &mut World, agent: String) {
    let agent = w.agent(&agent).name.clone();
    let admin = w.admin().to_string();
    w.brokkr(&admin, &["agent", "activate", &agent]).await;
    w.last_ok();
}

#[when(expr = "the admin lists the generators")]
async fn lists_generators(w: &mut World) {
    let admin = w.admin().to_string();
    w.brokkr(&admin, &["generator", "list"]).await;
}

#[when(expr = "the admin sends the stack {string} to the agent {string}")]
async fn sends_stack_to_agent(w: &mut World, stack: String, agent: String) {
    let stack = w.real(&w.stack(&stack).name);
    let agent = w.agent(&agent).name.clone();
    let admin = w.admin().to_string();
    w.brokkr(&admin, &["stack", "target", &stack, &agent]).await;
}

#[given(expr = "the admin has sent the stack {string} to the agent {string}")]
async fn has_sent_stack_to_agent(w: &mut World, stack: String, agent: String) {
    sends_stack_to_agent(w, stack, agent).await;
    w.last_ok();
}

#[when(expr = "the admin pauses the agent {string}")]
async fn pauses_agent(w: &mut World, agent: String) {
    let agent = w.agent(&agent).name.clone();
    let admin = w.admin().to_string();
    w.brokkr(&admin, &["agent", "pause", &agent]).await;
    w.last_ok();
}

/// No command removes a target: the admin asks the broker.
#[when(expr = "the admin takes the stack {string} from the agent {string}")]
async fn takes_stack_from_agent(w: &mut World, stack: String, agent: String) {
    let stack_id = w.stack_id(&stack).await;
    let agent_id = w.agent(&agent).id;
    w.api
        .remove_target(w.admin(), agent_id, stack_id)
        .await
        .expect("remove the target");
}

/// No command deletes an agent: the admin asks the broker. The record stays
/// on the `World`, so later steps can ask about the deleted agent.
#[when(expr = "the admin deletes the agent {string}")]
async fn deletes_agent(w: &mut World, agent: String) {
    let id = w.agent(&agent).id;
    let answer = w
        .api
        .delete_agent(w.admin(), id)
        .await
        .expect("ask the broker to delete the agent");
    assert!(
        answer.ok(),
        "the broker did not delete the agent {agent:?}: {} {}",
        answer.status,
        answer.body
    );
}

/// The agent that runs in the test stack's cluster: the admin registers it
/// with the generator and activates it, so the generator's stacks can reach it.
#[given(expr = "the cluster agent is registered with the generator {string} and active")]
async fn cluster_agent_registered(w: &mut World, generator: String) {
    let name = w.cluster_agent_name().to_string();
    let admin = w.admin().to_string();
    let agents = w.api.list_agents(&admin).await.expect("list the agents");
    let id = api::named(&agents, &name)
        .and_then(api::id_of)
        .unwrap_or_else(|| panic!("the cluster agent {name:?} is not in the broker's agent list"));
    w.cluster_agent_id = Some(id);
    let generator_id = w.generator(&generator).id;
    let generator_name = w.generator(&generator).name.clone();
    w.brokkr(
        &admin,
        &["register", "--agent", &name, "--generator", &generator_name],
    )
    .await;
    w.last_ok();
    w.cluster_registrations.push(generator_id);
    w.brokkr(&admin, &["agent", "activate", &name]).await;
    w.last_ok();
}
