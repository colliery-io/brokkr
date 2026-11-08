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

#[then("the output shows the PAK once")]
async fn output_shows_pak_once(w: &mut World) {
    w.printed_pak();
    w.expect_output("shown once");
}

#[then("the output names the next step")]
async fn output_names_next_step(w: &mut World) {
    w.expect_output("Next: brokkr register");
}

// Tenants

#[then(expr = "the tenant {string} exists")]
async fn tenant_exists(w: &mut World, name: String) {
    let tenant = w.tenant(&name).clone();
    let listed = w
        .api
        .list_generators(w.admin())
        .await
        .expect("list the tenants");
    let found = api::named(&listed, &tenant.name).and_then(api::id_of);
    assert_eq!(
        found,
        Some(tenant.id),
        "the broker does not list the tenant {:?} with the id the command printed",
        tenant.name
    );
}

#[then(expr = "there is one tenant {string}")]
async fn one_tenant(w: &mut World, name: String) {
    let real = w.real(&name);
    let listed = w
        .api
        .list_generators(w.admin())
        .await
        .expect("list the tenants");
    let count = listed
        .iter()
        .filter(|g| g.get("name").and_then(Value::as_str) == Some(real.as_str()))
        .count();
    assert_eq!(count, 1, "{count} tenants have the name {real:?}");
}

#[then(expr = "there is no tenant {string}")]
async fn no_tenant(w: &mut World, name: String) {
    let real = w.real(&name);
    let listed = w
        .api
        .list_generators(w.admin())
        .await
        .expect("list the tenants");
    assert!(
        api::named(&listed, &real).is_none(),
        "the broker lists a tenant {real:?}"
    );
}

#[then(expr = "the list has the tenant {string}")]
async fn list_has_tenant(w: &mut World, name: String) {
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

#[then(expr = "the answer names the tenant {string}")]
async fn answer_names_tenant(w: &mut World, name: String) {
    let tenant = w.tenant(&name);
    let line = format!("PAK: tenant \"{}\" ({})", tenant.name, tenant.id);
    w.expect_output(&line);
}

#[then(expr = "the tenant {string} lists the agent {string}")]
async fn tenant_lists_agent(w: &mut World, tenant: String, agent: String) {
    let pak = w.tenant(&tenant).pak.clone();
    let agent = w.agent(&agent).name.clone();
    w.brokkr(&pak, &["agent", "list"]).await;
    w.expect_output(&agent);
}

#[then(expr = "the tenant {string} lists the stack {string}")]
async fn tenant_lists_stack(w: &mut World, tenant: String, stack: String) {
    let pak = w.tenant(&tenant).pak.clone();
    let stack = w.real(&w.stack(&stack).name);
    w.brokkr(&pak, &["stack", "list"]).await;
    w.expect_output(&stack);
}

#[then(expr = "the tenant {string} no longer finds the stack {string}")]
async fn tenant_no_longer_finds_stack(w: &mut World, tenant: String, stack: String) {
    let pak = w.tenant(&tenant).pak.clone();
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

#[then(expr = "the previous PAK of the tenant {string} is refused")]
async fn previous_pak_refused(w: &mut World, tenant: String) {
    let previous = w
        .tenant(&tenant)
        .previous_paks
        .last()
        .cloned()
        .expect("the tenant has rotated its PAK");
    w.brokkr(&previous, &["whoami"]).await;
    w.last_refused();
}

#[then(expr = "the new PAK of the tenant {string} names the tenant")]
async fn new_pak_names_tenant(w: &mut World, tenant: String) {
    let pak = w.tenant(&tenant).pak.clone();
    let name = w.tenant(&tenant).name.clone();
    w.brokkr(&pak, &["whoami"]).await;
    w.expect_output(&format!("PAK: tenant \"{name}\""));
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

#[then(expr = "the agent {string} is registered with the tenant {string}")]
async fn agent_registered(w: &mut World, agent: String, tenant: String) {
    let tenant_id = w.tenant(&tenant).id;
    let regs = registrations(w, &agent).await;
    assert!(
        regs.contains(&tenant_id),
        "the agent {agent:?} is not registered with the tenant {tenant:?}: {regs:?}"
    );
}

#[then(expr = "the agent {string} is not registered with the tenant {string}")]
async fn agent_not_registered(w: &mut World, agent: String, tenant: String) {
    let tenant_id = w.tenant(&tenant).id;
    let regs = registrations(w, &agent).await;
    assert!(
        !regs.contains(&tenant_id),
        "the agent {agent:?} is registered with the tenant {tenant:?}"
    );
}

/// Every agent is registered with the system generator at creation. It is
/// the one registration that is with no tenant of this scenario.
#[then(expr = "the agent {string} is registered with the system generator")]
async fn agent_registered_with_system(w: &mut World, agent: String) {
    let tenants: Vec<Uuid> = w.tenants.values().map(|t| t.id).collect();
    let regs = registrations(w, &agent).await;
    assert!(
        regs.iter().any(|g| !tenants.contains(g)),
        "the agent {agent:?} has no registration beyond the scenario's tenants: {regs:?}"
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
    expr = "the output shows the Helm command for the agent {string} in cluster {string} with the tenant {string}"
)]
async fn output_shows_helm(w: &mut World, agent: String, cluster: String, tenant: String) {
    let agent = w.agent(&agent).name.clone();
    let tenant_id = w.tenant(&tenant).id;
    w.expect_output("helm install brokkr-agent");
    w.expect_output(&format!("--set broker.agentName={agent}"));
    w.expect_output(&format!("--set broker.clusterName={cluster}"));
    w.expect_output(&format!("--set broker.generatorIds=\"{{{tenant_id}}}\""));
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
    expr = "the error says the agent {string} is not registered with the tenant that owns the stack {string}"
)]
async fn error_not_registered(w: &mut World, agent: String, stack: String) {
    let agent = w.agent(&agent).name.clone();
    let stack = w.real(&stack);
    w.expect_error(&format!(
        "agent \"{agent}\" is not registered with the tenant that owns stack \"{stack}\""
    ));
}

#[then("the error names the command that registers it")]
async fn error_names_register(w: &mut World) {
    w.expect_error("brokkr register --agent");
}

/// Poll `brokkr stack status` as the tenant until the table names the
/// cluster agent, or the wait is over.
#[then(expr = "the status of the stack {string} names the cluster agent within three minutes")]
async fn status_names_cluster_agent(w: &mut World, stack: String) {
    let owner = w.stack(&stack).owner.clone();
    let pak = w.tenant(&owner).pak.clone();
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
