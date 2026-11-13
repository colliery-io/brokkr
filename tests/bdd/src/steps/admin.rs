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

// Work orders (BROKKR-T-0399). No command makes a work order: the admin asks
// the broker.

/// The label a scenario gives its agents and its work order, in the form
/// key:value that the CLI asks for. It is unique to the scenario, so a work
/// order for it reaches no agent of another scenario.
pub fn scenario_label(w: &World, label: &str) -> String {
    format!("{label}:{}", w.run)
}

/// `POST /work-orders` with `extra` over a small default body. A created work
/// order is kept on the `World`; the answer is kept for the checks.
async fn create_work_order(w: &mut World, name: &str, extra: serde_json::Value) {
    let mut body = serde_json::json!({
        "work_type": "bdd",
        "yaml_content": format!("# bdd work order {}\n", w.real(name)),
    });
    for (key, value) in extra.as_object().expect("an object").clone() {
        body[key] = value;
    }
    let answer = w
        .api
        .create_work_order(w.admin(), body)
        .await
        .expect("ask the broker for a work order");
    if answer.ok() {
        let id = api::id_of(&answer.body)
            .unwrap_or_else(|| panic!("no id in the new work order: {}", answer.body));
        w.work_orders.insert(name.to_string(), id);
    }
    w.answer = Some(answer);
}

/// The last answer, which must be a created work order.
fn work_order_created(w: &World) {
    let answer = w.answer();
    assert!(
        answer.ok(),
        "the broker did not create the work order: {} {}",
        answer.status,
        answer.body
    );
}

#[given(expr = "the admin has created the work order {string} for the agent {string}")]
async fn has_created_work_order(w: &mut World, name: String, agent: String) {
    let agent_id = w.agent(&agent).id;
    create_work_order(
        w,
        &name,
        serde_json::json!({ "targeting": { "agent_ids": [agent_id] } }),
    )
    .await;
    work_order_created(w);
}

#[when(expr = "the admin creates the work order {string} for the agent {string}")]
async fn creates_work_order(w: &mut World, name: String, agent: String) {
    let agent_id = w.agent(&agent).id;
    create_work_order(
        w,
        &name,
        serde_json::json!({ "targeting": { "agent_ids": [agent_id] } }),
    )
    .await;
}

/// The backoff is 1 second, so the first failure waits 2 seconds, and the
/// maintenance task of the broker (each 10 seconds) brings the work order back.
#[given(
    expr = "the admin has created the work order {string} for the agent {string} with {int} retry"
)]
async fn has_created_work_order_with_retries(
    w: &mut World,
    name: String,
    agent: String,
    retries: i32,
) {
    let agent_id = w.agent(&agent).id;
    create_work_order(
        w,
        &name,
        serde_json::json!({
            "targeting": { "agent_ids": [agent_id] },
            "max_retries": retries,
            "backoff_seconds": 1,
        }),
    )
    .await;
    work_order_created(w);
}

#[given(
    expr = "the admin has created the work order {string} for the agent {string} with a claim timeout of {int} second"
)]
async fn has_created_work_order_with_timeout(
    w: &mut World,
    name: String,
    agent: String,
    seconds: i32,
) {
    let agent_id = w.agent(&agent).id;
    create_work_order(
        w,
        &name,
        serde_json::json!({
            "targeting": { "agent_ids": [agent_id] },
            "claim_timeout_seconds": seconds,
        }),
    )
    .await;
    work_order_created(w);
}

#[given(
    expr = "the admin has created the work order {string} for the label {string} of this scenario"
)]
async fn has_created_work_order_for_label(w: &mut World, name: String, label: String) {
    let label = scenario_label(w, &label);
    create_work_order(
        w,
        &name,
        serde_json::json!({ "targeting": { "labels": [label] } }),
    )
    .await;
    work_order_created(w);
}

#[given(expr = "the admin has given the agent {string} the label {string} of this scenario")]
async fn has_given_scenario_label(w: &mut World, agent: String, label: String) {
    let agent = w.agent(&agent).name.clone();
    let label = scenario_label(w, &label);
    let admin = w.admin().to_string();
    w.brokkr(&admin, &["agent", "label", &agent, &label]).await;
    w.last_ok();
}

#[given(expr = "the admin has deleted the agent {string}")]
async fn has_deleted_agent(w: &mut World, agent: String) {
    deletes_agent(w, agent).await;
}

// A work order that runs in the cluster (BROKKR-T-0406).

/// A `custom` work order for the cluster agent: one Job that prints the line
/// of the scenario, then ends with `exit_code`. The work order has no retry,
/// so a failure ends in the log at once.
async fn launch_job_on_cluster_agent(w: &mut World, name: &str, exit_code: i32) {
    let agent_id = w
        .cluster_agent_id
        .expect("the scenario has not looked up the cluster agent yet");
    let job = w.job_name(name);
    let line = w.job_line(name);
    let yaml = format!(
        "apiVersion: batch/v1\nkind: Job\nmetadata:\n  name: {job}\n  namespace: default\nspec:\n  backoffLimit: 0\n  ttlSecondsAfterFinished: 600\n  template:\n    spec:\n      restartPolicy: Never\n      containers:\n        - name: main\n          image: busybox:1.36\n          command: [\"sh\", \"-c\", \"echo {line}; sleep 2; exit {exit_code}\"]\n"
    );
    create_work_order(
        w,
        name,
        serde_json::json!({
            "work_type": "custom",
            "yaml_content": yaml,
            "max_retries": 0,
            "targeting": { "agent_ids": [agent_id] },
        }),
    )
    .await;
    work_order_created(w);
}

#[when(
    expr = "the admin launches the work order {string} on the cluster agent with a job that prints a line"
)]
async fn launches_job(w: &mut World, name: String) {
    launch_job_on_cluster_agent(w, &name, 0).await;
}

#[when(
    expr = "the admin launches the work order {string} on the cluster agent with a job that prints a line and fails"
)]
async fn launches_failing_job(w: &mut World, name: String) {
    launch_job_on_cluster_agent(w, &name, 1).await;
}
