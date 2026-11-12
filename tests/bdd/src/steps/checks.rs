/*
 * Copyright (c) 2025-2026 Dylan Storey
 * Licensed under the Elastic License 2.0.
 * See LICENSE file in the project root for full license text.
 */

//! What the people see: the command's output, the broker's answer, and the
//! records the admin can read afterwards.

use std::time::{Duration, Instant};

use cucumber::then;
use reqwest::StatusCode;
use serde_json::Value;
use uuid::Uuid;

use crate::api;
use crate::world::World;

/// How long the cluster scenario waits for the agent's first report: one
/// poll (10 s in the test stack) and one health interval (60 s), with slack.
const REPORT_WAIT: Duration = Duration::from_secs(180);

// The command

#[then("the command succeeds")]
async fn command_succeeds(w: &mut World) {
    w.last_ok();
}

#[then("the command is refused")]
async fn command_refused(w: &mut World) {
    w.expect_error("error");
}

#[then("the command says it changed nothing")]
async fn command_unchanged(w: &mut World) {
    w.expect_output("unchanged:");
}

#[then("the broker refuses")]
async fn broker_refuses(w: &mut World) {
    let answer = w.answer();
    assert_eq!(
        answer.status,
        StatusCode::FORBIDDEN,
        "the broker answered {} ({}) with {}, not 403",
        answer.status,
        answer.code().unwrap_or("no code"),
        answer.body
    );
}

#[then("the broker answers")]
async fn broker_answers(w: &mut World) {
    let answer = w.answer();
    assert!(
        answer.ok(),
        "the broker answered {} ({}) with {}",
        answer.status,
        answer.code().unwrap_or("no code"),
        answer.body
    );
}

#[then("the output shows the PAK once")]
async fn output_shows_pak_once(w: &mut World) {
    w.printed_pak();
    w.expect_output("shown once");
}

#[then("the output names the next step")]
async fn output_names_next_step(w: &mut World) {
    w.expect_output("Next: brokkr register");
}

// Generators

#[then(expr = "the generator {string} exists")]
async fn generator_exists(w: &mut World, name: String) {
    let generator = w.generator(&name).clone();
    let listed = w
        .api
        .list_generators(w.admin())
        .await
        .expect("list the generators");
    let found = api::named(&listed, &generator.name).and_then(api::id_of);
    assert_eq!(
        found,
        Some(generator.id),
        "the broker does not list the generator {:?} with the id the command printed",
        generator.name
    );
}

#[then(expr = "there is one generator {string}")]
async fn one_generator(w: &mut World, name: String) {
    let real = w.real(&name);
    let listed = w
        .api
        .list_generators(w.admin())
        .await
        .expect("list the generators");
    let count = listed
        .iter()
        .filter(|g| g.get("name").and_then(Value::as_str) == Some(real.as_str()))
        .count();
    assert_eq!(count, 1, "{count} generators have the name {real:?}");
}

#[then(expr = "there is no generator {string}")]
async fn no_generator(w: &mut World, name: String) {
    let real = w.real(&name);
    let listed = w
        .api
        .list_generators(w.admin())
        .await
        .expect("list the generators");
    assert!(
        api::named(&listed, &real).is_none(),
        "the broker lists a generator {real:?}"
    );
}

#[then(expr = "the list has the generator {string}")]
async fn list_has_generator(w: &mut World, name: String) {
    let real = w.real(&name);
    w.expect_output(&real);
}

#[then("the list has no system generator")]
async fn list_has_no_system_generator(w: &mut World) {
    let run = w.last_ok();
    let system = run
        .stdout
        .lines()
        .any(|l| l.starts_with("system ") || l.starts_with("system\t"));
    assert!(
        !system,
        "the list shows the system generator:\n{}",
        run.show()
    );
}

#[then(expr = "the answer names the generator {string}")]
async fn answer_names_generator(w: &mut World, name: String) {
    let generator = w.generator(&name);
    let line = format!("PAK: generator \"{}\" ({})", generator.name, generator.id);
    w.expect_output(&line);
}

#[then(expr = "the generator {string} lists the agent {string}")]
async fn generator_lists_agent(w: &mut World, generator: String, agent: String) {
    let pak = w.generator(&generator).pak.clone();
    let agent = w.agent(&agent).name.clone();
    w.brokkr(&pak, &["agent", "list"]).await;
    w.expect_output(&agent);
}

#[then(expr = "the generator {string} lists the stack {string}")]
async fn generator_lists_stack(w: &mut World, generator: String, stack: String) {
    let pak = w.generator(&generator).pak.clone();
    let stack = w.real(&w.stack(&stack).name);
    w.brokkr(&pak, &["stack", "list"]).await;
    w.expect_output(&stack);
}

#[then(expr = "the generator {string} no longer finds the stack {string}")]
async fn generator_no_longer_finds_stack(w: &mut World, generator: String, stack: String) {
    let pak = w.generator(&generator).pak.clone();
    let stack = w.real(&w.stack(&stack).name);
    w.brokkr(&pak, &["stack", "status", &stack]).await;
    w.expect_error("no stack has the name or id");
}

#[then("the output shows the PAK once and says the previous one stopped")]
async fn output_shows_rotated_pak(w: &mut World) {
    w.printed_pak();
    w.expect_output("shown once");
    w.expect_output("previous PAK stopped");
}

#[then(expr = "the previous PAK of the generator {string} is refused")]
async fn previous_pak_refused(w: &mut World, generator: String) {
    let previous = w
        .generator(&generator)
        .previous_paks
        .last()
        .cloned()
        .expect("the generator has rotated its PAK");
    w.brokkr(&previous, &["whoami"]).await;
    w.last_refused();
}

#[then(expr = "the new PAK of the generator {string} names the generator")]
async fn new_pak_names_generator(w: &mut World, generator: String) {
    let pak = w.generator(&generator).pak.clone();
    let name = w.generator(&generator).name.clone();
    w.brokkr(&pak, &["whoami"]).await;
    w.expect_output(&format!("PAK: generator \"{name}\""));
}

// Agents

#[then(expr = "the agent {string} exists")]
async fn agent_exists(w: &mut World, name: String) {
    let agent = w.agent(&name).clone();
    let record = w
        .api
        .get_agent(w.admin(), agent.id)
        .await
        .expect("read the agent");
    assert_eq!(
        record.get("name").and_then(Value::as_str),
        Some(agent.name.as_str()),
        "the record {} is not the agent {:?}: {record}",
        agent.id,
        agent.name
    );
}

#[then(expr = "there is no agent {string}")]
async fn no_agent(w: &mut World, name: String) {
    let real = w.real(&name);
    let listed = w.api.list_agents(w.admin()).await.expect("list the agents");
    assert!(
        api::named(&listed, &real).is_none(),
        "the broker lists an agent {real:?}"
    );
}

/// The generator ids an agent is registered with.
async fn registrations(w: &World, agent: &str) -> Vec<Uuid> {
    let id = w.agent(agent).id;
    w.api
        .agent_registrations(w.admin(), id)
        .await
        .expect("list the registrations")
        .iter()
        .filter_map(|r| {
            r.get("generator_id")
                .and_then(Value::as_str)
                .and_then(|s| Uuid::parse_str(s).ok())
        })
        .collect()
}

#[then(expr = "the agent {string} is registered with the generator {string}")]
async fn agent_registered(w: &mut World, agent: String, generator: String) {
    let generator_id = w.generator(&generator).id;
    let regs = registrations(w, &agent).await;
    assert!(
        regs.contains(&generator_id),
        "the agent {agent:?} is not registered with the generator {generator:?}: {regs:?}"
    );
}

#[then(expr = "the agent {string} is not registered with the generator {string}")]
async fn agent_not_registered(w: &mut World, agent: String, generator: String) {
    let generator_id = w.generator(&generator).id;
    let regs = registrations(w, &agent).await;
    assert!(
        !regs.contains(&generator_id),
        "the agent {agent:?} is registered with the generator {generator:?}"
    );
}

/// Every agent is registered with the system generator at creation. It is
/// the one registration that is with no generator of this scenario.
#[then(expr = "the agent {string} is registered with the system generator")]
async fn agent_registered_with_system(w: &mut World, agent: String) {
    let generators: Vec<Uuid> = w.generators.values().map(|t| t.id).collect();
    let regs = registrations(w, &agent).await;
    assert!(
        regs.iter().any(|g| !generators.contains(g)),
        "the agent {agent:?} has no registration beyond the scenario's generators: {regs:?}"
    );
}

#[then(expr = "the agent {string} is {word}")]
async fn agent_has_status(w: &mut World, agent: String, status: String) {
    let id = w.agent(&agent).id;
    let record = w
        .api
        .get_agent(w.admin(), id)
        .await
        .expect("read the agent");
    assert_eq!(
        record.get("status").and_then(Value::as_str),
        Some(status.as_str()),
        "the agent {agent:?} is not {status}: {record}"
    );
}

#[then(
    expr = "the output shows the Helm command for the agent {string} in cluster {string} with the generator {string}"
)]
async fn output_shows_helm(w: &mut World, agent: String, cluster: String, generator: String) {
    let agent = w.agent(&agent).name.clone();
    let generator_id = w.generator(&generator).id;
    w.expect_output("helm install brokkr-agent");
    w.expect_output(&format!("--set broker.agentName={agent}"));
    w.expect_output(&format!("--set broker.clusterName={cluster}"));
    w.expect_output(&format!("--set broker.generatorIds=\"{{{generator_id}}}\""));
}

#[then("the output says the agent is inactive and how to activate it")]
async fn output_says_inactive(w: &mut World) {
    w.expect_output("The agent is INACTIVE");
    w.expect_output("brokkr agent activate");
}

#[then(expr = "the answer names the agent {string} in cluster {string}")]
async fn answer_names_agent(w: &mut World, agent: String, cluster: String) {
    let agent = w.agent(&agent).clone();
    let line = format!(
        "PAK: agent \"{}\" ({}) in cluster \"{cluster}\"",
        agent.name, agent.id
    );
    w.expect_output(&line);
}

// The real agent at start-up

#[then(expr = "the agent finds the record {string}")]
async fn agent_finds_record(w: &mut World, agent: String) {
    let name = w.agent(&agent).name.clone();
    let run = w.last();
    let line = format!("Agent details fetched successfully for agent: {name}");
    assert!(
        run.output().contains(&line),
        "the agent did not report its record {name:?}:\n{}",
        run.show()
    );
}

#[then("the agent stops")]
async fn agent_stops(w: &mut World) {
    let run = w.last();
    assert!(!run.timed_out, "the agent kept running:\n{}", run.show());
    assert!(!run.ok, "the agent exited 0:\n{}", run.show());
}

fn expect_identity_error(w: &World, field: &str, record: &str, configured: &str) {
    let run = w.last();
    let line = format!(
        "The agent record of this PAK has the {field} \"{record}\", but the configuration says \"{configured}\""
    );
    assert!(
        run.output().contains(&line),
        "the agent's error does not say {line:?}:\n{}",
        run.show()
    );
}

#[then(
    expr = "the error says the record has the name {string} and the configuration says {string}"
)]
async fn error_names_name(w: &mut World, record: String, configured: String) {
    let record = w.agent(&record).name.clone();
    let configured = w.real(&configured);
    expect_identity_error(w, "name", &record, &configured);
}

#[then(
    expr = "the error says the record has the cluster {string} and the configuration says {string}"
)]
async fn error_names_cluster(w: &mut World, record: String, configured: String) {
    expect_identity_error(w, "cluster", &record, &configured);
}

#[then(expr = "the error names the Helm value {string}")]
async fn error_names_helm_value(w: &mut World, helm: String) {
    let run = w.last();
    let needle = format!("Set broker.{helm}");
    assert!(
        run.output().contains(&needle),
        "the agent's error does not name {needle:?}:\n{}",
        run.show()
    );
}

// Stacks

#[then(expr = "the command says it created the stack {string} with a first revision")]
async fn created_stack(w: &mut World, stack: String) {
    let real = w.real(&stack);
    w.expect_output(&format!("created stack \"{real}\": first revision"));
}

#[then(expr = "the command says the stack {string} is unchanged")]
async fn stack_unchanged(w: &mut World, stack: String) {
    let real = w.real(&stack);
    w.expect_output(&format!("unchanged: stack \"{real}\" already current"));
}

#[then("the status shows the latest revision and the health")]
async fn status_shows_revision_and_health(w: &mut World) {
    w.expect_output("latest revision:");
    w.expect_output("health:");
}

#[then("the status says no agent has reported yet")]
async fn status_says_no_report(w: &mut World) {
    w.expect_output("No agent has reported on this stack yet");
}

#[then(expr = "the list shows the agent {string} with the label {string}")]
async fn list_shows_agent_with_label(w: &mut World, agent: String, label: String) {
    let agent = w.agent(&agent).name.clone();
    let run = w.last_ok();
    assert!(
        run.stdout
            .lines()
            .next()
            .is_some_and(|h| h.contains("LABELS")),
        "the list has no LABELS column:\n{}",
        run.show()
    );
    let row = run
        .stdout
        .lines()
        .find(|l| l.starts_with(&agent))
        .unwrap_or_else(|| panic!("the list has no row for {agent:?}:\n{}", run.show()));
    assert!(
        row.contains(&label),
        "the row of {agent:?} lacks the label {label:?}: {row:?}"
    );
}

#[then(expr = "the list has the agent {string}")]
async fn list_has_agent(w: &mut World, agent: String) {
    let agent = w.agent(&agent).name.clone();
    w.expect_output(&agent);
}

#[then(expr = "the list does not have the agent {string}")]
async fn list_lacks_agent(w: &mut World, agent: String) {
    let agent = w.agent(&agent).name.clone();
    let run = w.last_ok();
    assert!(
        !run.stdout.contains(&agent),
        "the list has the agent {agent:?}:\n{}",
        run.show()
    );
}

#[then(expr = "the list has the stack {string}")]
async fn list_has_stack(w: &mut World, stack: String) {
    let real = w.real(&stack);
    w.expect_output(&real);
}

#[then(expr = "the list does not have the stack {string}")]
async fn list_lacks_stack(w: &mut World, stack: String) {
    let real = w.real(&stack);
    let run = w.last_ok();
    assert!(
        !run.stdout.contains(&real),
        "the list has the stack {real:?}:\n{}",
        run.show()
    );
}

#[then(expr = "the command says the labels of the stack {string} match the agent {string}")]
async fn labels_match_agent(w: &mut World, stack: String, agent: String) {
    let stack = w.real(&stack);
    let agent = w.agent(&agent).name.clone();
    w.expect_output(&format!("the labels of stack \"{stack}\" match: {agent}"));
}

#[then(expr = "the command says no agent matches the labels of the stack {string}")]
async fn no_agent_matches(w: &mut World, stack: String) {
    let stack = w.real(&stack);
    w.expect_output(&format!("no agent matches the labels of stack \"{stack}\""));
}

#[then("the command says each agent removes the stack's resources on its next poll")]
async fn deleted_stack(w: &mut World) {
    w.expect_output("removes its resources on its next poll");
}

#[then(expr = "the command says it sent the stack {string} to the agent {string}")]
async fn targeted_stack(w: &mut World, stack: String, agent: String) {
    let stack = w.real(&stack);
    let agent = w.agent(&agent).name.clone();
    w.expect_output(&format!("targeted stack \"{stack}\" at agent \"{agent}\""));
}

#[then(
    expr = "the error says the agent {string} is not registered with the generator that owns the stack {string}"
)]
async fn error_not_registered(w: &mut World, agent: String, stack: String) {
    let agent = w.agent(&agent).name.clone();
    let stack = w.real(&stack);
    w.expect_error(&format!(
        "agent \"{agent}\" is not registered with the generator that owns stack \"{stack}\""
    ));
}

#[then("the error names the command that registers it")]
async fn error_names_register(w: &mut World) {
    w.expect_error("brokkr register --agent");
}

/// Poll `brokkr stack status` as the generator until the table names the
/// cluster agent, or the wait is over.
#[then(expr = "the status of the stack {string} names the cluster agent within three minutes")]
async fn status_names_cluster_agent(w: &mut World, stack: String) {
    let owner = w.stack(&stack).owner.clone();
    let pak = w.generator(&owner).pak.clone();
    let real = w.real(&stack);
    let agent = w.cluster_agent_name().to_string();
    let start = Instant::now();
    loop {
        w.brokkr(&pak, &["stack", "status", &real]).await;
        let run = w.last_ok();
        let reported = run
            .stdout
            .lines()
            .any(|l| l.starts_with(&agent) && !l.contains("No agent has reported"));
        if reported {
            return;
        }
        assert!(
            start.elapsed() < REPORT_WAIT,
            "the cluster agent {agent:?} has not reported on the stack {real:?} after {REPORT_WAIT:?}:\n{}",
            run.show()
        );
        tokio::time::sleep(Duration::from_secs(5)).await;
    }
}

// The agent's lifecycle: the fleet record, the stack health, the registrations

/// The fleet record of an agent of the scenario, as the admin reads it.
async fn fleet_record(w: &World, agent: &str) -> Value {
    w.api
        .fleet_record(w.admin(), w.agent(agent).id)
        .await
        .unwrap_or_else(|e| panic!("read the fleet record of the agent {agent:?}: {e}"))
}

#[then(expr = "the fleet record of the agent {string} is {word}")]
async fn fleet_record_status(w: &mut World, agent: String, status: String) {
    let record = fleet_record(w, &agent).await;
    assert_eq!(
        record.get("status").and_then(Value::as_str),
        Some(status.as_str()),
        "the fleet record of the agent {agent:?} is not {status}: {record}"
    );
}

#[then(expr = "the fleet record of the agent {string} has a heartbeat under {int} seconds old")]
async fn fleet_record_fresh(w: &mut World, agent: String, seconds: i64) {
    let record = fleet_record(w, &agent).await;
    let age = record.get("heartbeat_age_seconds").and_then(Value::as_i64);
    assert!(
        age.is_some_and(|a| a < seconds),
        "the heartbeat of the agent {agent:?} is not under {seconds} s old: {record}"
    );
}

#[then(expr = "the fleet record of the agent {string} has a heartbeat at least {int} seconds old")]
async fn fleet_record_aged(w: &mut World, agent: String, seconds: i64) {
    let record = fleet_record(w, &agent).await;
    let age = record.get("heartbeat_age_seconds").and_then(Value::as_i64);
    assert!(
        age.is_some_and(|a| a >= seconds),
        "the heartbeat of the agent {agent:?} is not at least {seconds} s old: {record}"
    );
}

#[then(expr = "the fleet record of the agent {string} has no open connection")]
async fn fleet_record_not_connected(w: &mut World, agent: String) {
    let record = fleet_record(w, &agent).await;
    assert_eq!(
        record.get("ws_connected").and_then(Value::as_bool),
        Some(false),
        "the fleet record of the agent {agent:?} shows an open connection: {record}"
    );
}

#[then(expr = "the fleet record of the agent {string} counts {int} failing record(s)")]
async fn fleet_record_failing(w: &mut World, agent: String, count: i64) {
    let record = fleet_record(w, &agent).await;
    assert_eq!(
        record.get("health_failing").and_then(Value::as_i64),
        Some(count),
        "the agent {agent:?} does not count {count} failing: {record}"
    );
}

#[then(expr = "the fleet does not list the agent {string}")]
async fn fleet_has_no_agent(w: &mut World, agent: String) {
    let id = w.agent(&agent).id.to_string();
    let fleet = w.api.fleet(w.admin()).await.expect("read the fleet");
    assert!(
        !fleet
            .iter()
            .any(|r| r.get("agent_id").and_then(Value::as_str) == Some(id.as_str())),
        "the fleet still lists the agent {agent:?} ({id})"
    );
}

#[then(expr = "the PAK of the agent {string} is refused")]
async fn agent_pak_refused(w: &mut World, agent: String) {
    let pak = w.agent(&agent).pak.clone();
    w.brokkr(&pak, &["whoami"]).await;
    w.last_refused();
}

/// The agents that the health of a stack lists, over each of its objects.
async fn stack_health_agents(w: &World, stack: &str) -> Vec<String> {
    let id = w.stack_id(stack).await;
    let answer = w
        .api
        .stack_health(w.admin(), id)
        .await
        .expect("ask the broker for the health");
    assert!(
        answer.ok(),
        "the broker did not give the health of the stack {stack:?}: {} {}",
        answer.status,
        answer.body
    );
    answer
        .body
        .get("deployment_objects")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(|o| o.get("agents").and_then(Value::as_array))
        .flatten()
        .filter_map(|a| a.get("agent_id").and_then(Value::as_str))
        .map(str::to_string)
        .collect()
}

#[then(expr = "the health of the stack {string} lists the agent {string}")]
async fn stack_health_lists(w: &mut World, stack: String, agent: String) {
    let id = w.agent(&agent).id.to_string();
    let agents = stack_health_agents(w, &stack).await;
    assert!(
        agents.contains(&id),
        "the health of the stack {stack:?} does not list the agent {agent:?}: {agents:?}"
    );
}

#[then(expr = "the health of the stack {string} does not list the agent {string}")]
async fn stack_health_omits(w: &mut World, stack: String, agent: String) {
    let id = w.agent(&agent).id.to_string();
    let agents = stack_health_agents(w, &stack).await;
    assert!(
        !agents.contains(&id),
        "the health of the stack {stack:?} still lists the agent {agent:?}"
    );
}

#[then(expr = "the registrations of the generator {string} do not name the agent {string}")]
async fn registrations_omit(w: &mut World, generator: String, agent: String) {
    let id = w.agent(&agent).id.to_string();
    let generator_id = w.generator(&generator).id;
    let registered = w
        .api
        .registered_agents(w.admin(), generator_id)
        .await
        .expect("list the registrations of the generator");
    assert!(
        !registered
            .iter()
            .any(|r| r.get("agent_id").and_then(Value::as_str) == Some(id.as_str())),
        "the generator {generator:?} still has a registration of the deleted agent {agent:?} ({id})"
    );
}

#[then(expr = "the target state of the agent {string} has the stack {string}")]
async fn target_state_has_stack(w: &mut World, agent: String, stack: String) {
    assert!(
        crate::steps::agent::served_object(w, &agent, &stack)
            .await
            .is_some(),
        "the target state of the agent {agent:?} does not have the stack {stack:?}"
    );
}

// The stack lifecycle (BROKKR-T-0398)

#[then(expr = "the stack {string} has {int} revision(s)")]
async fn stack_has_revisions(w: &mut World, stack: String, count: usize) {
    let id = w.stack_id(&stack).await;
    let revisions = w
        .api
        .revisions(w.admin(), id)
        .await
        .expect("list the revisions of the stack");
    assert_eq!(
        revisions.len(),
        count,
        "the stack {stack:?} does not have {count} revisions: {revisions:?}"
    );
}

#[then(expr = "the agent {string} serves the latest revision of the stack {string}")]
async fn agent_serves_latest(w: &mut World, agent: String, stack: String) {
    let id = w.stack_id(&stack).await;
    let revisions = w
        .api
        .revisions(w.admin(), id)
        .await
        .expect("list the revisions of the stack");
    let latest =
        api::latest(&revisions).unwrap_or_else(|| panic!("the stack {stack:?} has no revision"));
    let served = crate::steps::agent::served_object(w, &agent, &stack).await;
    assert_eq!(
        served,
        Some(latest),
        "the agent {agent:?} does not serve the latest revision of the stack {stack:?}"
    );
}

#[then(expr = "the health of the stack {string} is {word}")]
async fn stack_health_is(w: &mut World, stack: String, status: String) {
    let id = w.stack_id(&stack).await;
    let answer = w
        .api
        .stack_health(w.admin(), id)
        .await
        .expect("ask the broker for the health");
    assert_eq!(
        answer.body.get("overall_status").and_then(Value::as_str),
        Some(status.as_str()),
        "the stack {stack:?} is not {status}: {} {}",
        answer.status,
        answer.body
    );
}

#[then(expr = "the fleet record of the agent {string} counts {int} degraded record(s)")]
async fn fleet_record_degraded(w: &mut World, agent: String, count: i64) {
    let record = fleet_record(w, &agent).await;
    assert_eq!(
        record.get("health_degraded").and_then(Value::as_i64),
        Some(count),
        "the agent {agent:?} does not count {count} degraded: {record}"
    );
}

/// A deleted stack has no health: the broker does not find it, or it finds
/// no deployment object with a report.
#[then(expr = "the deleted stack {string} has no health")]
async fn deleted_stack_has_no_health(w: &mut World, stack: String) {
    let id = w.noted(&stack).id;
    let answer = w
        .api
        .stack_health(w.admin(), id)
        .await
        .expect("ask the broker for the health");
    let objects = answer
        .body
        .get("deployment_objects")
        .and_then(Value::as_array)
        .map_or(0, Vec::len);
    assert!(
        !answer.ok() || objects == 0,
        "the deleted stack {stack:?} still has health: {}",
        answer.body
    );
}

#[then(expr = "the targets of the agent {string} do not name the deleted stack {string}")]
async fn targets_omit_deleted_stack(w: &mut World, agent: String, stack: String) {
    let id = w.noted(&stack).id.to_string();
    let targets = w
        .api
        .agent_targets(w.admin(), w.agent(&agent).id)
        .await
        .expect("list the targets of the agent");
    assert!(
        !targets
            .iter()
            .any(|t| t.get("stack_id").and_then(Value::as_str) == Some(id.as_str())),
        "the agent {agent:?} still has a target for the deleted stack {stack:?} ({id})"
    );
}

/// The health records of the last revision of a deleted stack. A refusal of
/// the broker is no record.
async fn deleted_revision_records(w: &World, stack: &str) -> Vec<Value> {
    let object = w.noted_revision(stack);
    let answer = w
        .api
        .object_health(w.admin(), object)
        .await
        .expect("ask the broker for the health of the deployment object");
    if !answer.ok() {
        return Vec::new();
    }
    answer
        .body
        .get("health_records")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default()
}

#[then(expr = "the last revision of the deleted stack {string} has no health records")]
async fn deleted_revision_has_no_records(w: &mut World, stack: String) {
    let records = deleted_revision_records(w, &stack).await;
    assert!(
        records.is_empty(),
        "the broker still returns the health of a revision of the deleted stack {stack:?}: {records:?}"
    );
}

#[then(expr = "the last revision of the deleted stack {string} has no {word} record")]
async fn deleted_revision_has_no_status(w: &mut World, stack: String, status: String) {
    let records = deleted_revision_records(w, &stack).await;
    assert!(
        !records
            .iter()
            .any(|r| r.get("status").and_then(Value::as_str) == Some(status.as_str())),
        "the broker kept a {status} report on a revision of the deleted stack {stack:?}: {records:?}"
    );
}

// Work orders (BROKKR-T-0399)

/// The status of a work order as the admin reads it: `PENDING`, `CLAIMED` or
/// `RETRY_PENDING` while it is open, `COMPLETED` or `FAILED` from the log,
/// and `None` when the broker no longer has it.
async fn work_order_status(w: &World, name: &str) -> Option<String> {
    let id = *w
        .work_orders
        .get(name)
        .unwrap_or_else(|| panic!("the scenario has not created the work order {name:?} yet"));
    let answer = w
        .api
        .get_work_order(w.admin(), id)
        .await
        .expect("read the work order");
    if answer.status == StatusCode::NOT_FOUND {
        return None;
    }
    assert!(
        answer.ok(),
        "the broker did not give the work order {name:?}: {} {}",
        answer.status,
        answer.body
    );
    answer
        .body
        .get("status")
        .and_then(Value::as_str)
        .map(str::to_string)
}

#[then(expr = "the work order {string} is {word}")]
async fn work_order_is(w: &mut World, name: String, status: String) {
    assert_eq!(
        work_order_status(w, &name).await.as_deref(),
        Some(status.as_str()),
        "the work order {name:?} is not {status}"
    );
}

/// Poll until the work order has the status: the maintenance task of the
/// broker moves it, each 10 seconds.
#[then(expr = "the work order {string} is {word} within {int} seconds")]
async fn work_order_is_within(w: &mut World, name: String, status: String, seconds: u64) {
    let start = Instant::now();
    loop {
        let now = work_order_status(w, &name).await;
        if now.as_deref() == Some(status.as_str()) {
            return;
        }
        assert!(
            start.elapsed() < Duration::from_secs(seconds),
            "the work order {name:?} is {now:?}, not {status}, after {seconds} s"
        );
        tokio::time::sleep(Duration::from_secs(1)).await;
    }
}

/// Poll until the work order is no longer pending: it ended, it is flagged,
/// or it is gone. The scenario does not say which.
#[then(expr = "the work order {string} is not left PENDING within {int} seconds")]
async fn work_order_not_left_pending(w: &mut World, name: String, seconds: u64) {
    let start = Instant::now();
    loop {
        let now = work_order_status(w, &name).await;
        if now.as_deref() != Some("PENDING") {
            return;
        }
        assert!(
            start.elapsed() < Duration::from_secs(seconds),
            "the work order {name:?} is still PENDING {seconds} s after its only agent was deleted"
        );
        tokio::time::sleep(Duration::from_secs(1)).await;
    }
}

#[then(expr = "the agent {string} has the work order {string} pending")]
async fn agent_has_pending(w: &mut World, agent: String, work_order: String) {
    assert!(
        crate::steps::agent::has_pending(w, &agent, &work_order).await,
        "the agent {agent:?} does not see the work order {work_order:?} pending"
    );
}

#[then(expr = "the agent {string} does not have the work order {string} pending")]
async fn agent_has_not_pending(w: &mut World, agent: String, work_order: String) {
    assert!(
        !crate::steps::agent::has_pending(w, &agent, &work_order).await,
        "the agent {agent:?} still sees the work order {work_order:?} pending"
    );
}

#[then(
    expr = "the fleet record of the agent {string} counts {int} pending and {int} claimed work order(s)"
)]
async fn fleet_record_work_orders(w: &mut World, agent: String, pending: i64, claimed: i64) {
    let record = fleet_record(w, &agent).await;
    assert_eq!(
        (
            record.get("pending_work_orders").and_then(Value::as_i64),
            record.get("claimed_work_orders").and_then(Value::as_i64),
        ),
        (Some(pending), Some(claimed)),
        "the agent {agent:?} does not count {pending} pending and {claimed} claimed: {record}"
    );
}

/// The log entry of a work order that ended, which must be there.
async fn work_order_log_success(w: &World, name: &str) -> Option<bool> {
    let id = *w
        .work_orders
        .get(name)
        .unwrap_or_else(|| panic!("the scenario has not created the work order {name:?} yet"));
    let answer = w
        .api
        .work_order_log(w.admin(), id)
        .await
        .expect("read the work order log");
    assert!(
        answer.ok(),
        "the log does not have the work order {name:?}: {} {}",
        answer.status,
        answer.body
    );
    answer.body.get("success").and_then(Value::as_bool)
}

#[then(expr = "the log has the work order {string} as a success")]
async fn log_has_success(w: &mut World, name: String) {
    assert_eq!(
        work_order_log_success(w, &name).await,
        Some(true),
        "the log does not have the work order {name:?} as a success"
    );
}

#[then(expr = "the log has the work order {string} as a failure")]
async fn log_has_failure(w: &mut World, name: String) {
    assert_eq!(
        work_order_log_success(w, &name).await,
        Some(false),
        "the log does not have the work order {name:?} as a failure"
    );
}

#[then("the broker says the work order is not claimable")]
async fn broker_says_not_claimable(w: &mut World) {
    let answer = w.answer();
    assert_eq!(
        (answer.status, answer.code()),
        (StatusCode::NOT_FOUND, Some("work_order_not_claimable")),
        "the broker answered {} with {}",
        answer.status,
        answer.body
    );
}

#[then("the broker does not accept the work order")]
async fn broker_does_not_accept_work_order(w: &mut World) {
    let answer = w.answer();
    assert!(
        !answer.ok(),
        "the broker created a work order for a deleted agent: {} {}",
        answer.status,
        answer.body
    );
}
