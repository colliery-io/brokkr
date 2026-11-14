/*
 * Copyright (c) 2025-2026 Dylan Storey
 * Licensed under the Elastic License 2.0.
 * See LICENSE file in the project root for full license text.
 */

//! What a generator does, with its own PAK.

use cucumber::{given, when};

use crate::world::{Stack, World};

/// `brokkr apply` of `bundle` as the generator; the stack is kept with its owner.
async fn apply(w: &mut World, generator: &str, stack: &str, bundle: std::path::PathBuf) {
    let real = w.real(stack);
    let pak = w.generator(generator).pak.clone();
    let folder = bundle.to_string_lossy().into_owned();
    w.brokkr(&pak, &["apply", "-f", &folder, "--stack", &real])
        .await;
    if w.last().ok {
        w.stacks.insert(
            stack.to_string(),
            Stack {
                name: stack.to_string(),
                owner: generator.to_string(),
            },
        );
    }
}

#[given(expr = "the generator {string} has applied a folder as the stack {string}")]
async fn has_applied(w: &mut World, generator: String, stack: String) {
    let bundle = w.new_bundle();
    apply(w, &generator, &stack, bundle).await;
    w.last_ok();
}

#[when(expr = "the generator {string} applies a folder as the stack {string}")]
async fn applies(w: &mut World, generator: String, stack: String) {
    let bundle = w.new_bundle();
    apply(w, &generator, &stack, bundle).await;
}

#[when(expr = "the generator {string} applies the same folder as the stack {string}")]
async fn applies_same(w: &mut World, generator: String, stack: String) {
    let bundle = w.same_bundle();
    apply(w, &generator, &stack, bundle).await;
}

#[when(expr = "the generator {string} asks who it is")]
async fn asks_who(w: &mut World, generator: String) {
    let pak = w.generator(&generator).pak.clone();
    w.brokkr(&pak, &["whoami"]).await;
}

#[when(expr = "the generator {string} lists the generators")]
async fn lists_generators(w: &mut World, generator: String) {
    let pak = w.generator(&generator).pak.clone();
    w.brokkr(&pak, &["generator", "list"]).await;
}

#[when(expr = "the generator {string} lists its agents")]
async fn lists_agents(w: &mut World, generator: String) {
    let pak = w.generator(&generator).pak.clone();
    w.brokkr(&pak, &["agent", "list"]).await;
}

#[when(expr = "the generator {string} lists its stacks")]
async fn lists_stacks(w: &mut World, generator: String) {
    let pak = w.generator(&generator).pak.clone();
    w.brokkr(&pak, &["stack", "list"]).await;
}

#[when(expr = "the generator {string} asks for the status of the stack {string}")]
async fn stack_status(w: &mut World, generator: String, stack: String) {
    let pak = w.generator(&generator).pak.clone();
    let stack = w.real(&w.stack(&stack).name);
    w.brokkr(&pak, &["stack", "status", &stack]).await;
}

#[when(expr = "the generator {string} reads the events of the stack {string}")]
async fn stack_events(w: &mut World, generator: String, stack: String) {
    let pak = w.generator(&generator).pak.clone();
    let stack = w.real(&w.stack(&stack).name);
    w.brokkr(&pak, &["stack", "events", &stack, "--limit", "5"])
        .await;
}

#[when(expr = "the generator {string} reads the logs of the stack {string}")]
async fn stack_logs(w: &mut World, generator: String, stack: String) {
    let pak = w.generator(&generator).pak.clone();
    let stack = w.real(&w.stack(&stack).name);
    w.brokkr(&pak, &["stack", "logs", &stack, "--limit", "5"])
        .await;
}

#[when(expr = "the generator {string} labels the stack {string} with {string}")]
async fn labels_stack(w: &mut World, generator: String, stack: String, label: String) {
    let pak = w.generator(&generator).pak.clone();
    let stack = w.real(&w.stack(&stack).name);
    w.brokkr(&pak, &["stack", "label", &stack, &label]).await;
}

#[given(expr = "the generator {string} has labelled the stack {string} with {string}")]
async fn has_labelled_stack(w: &mut World, generator: String, stack: String, label: String) {
    labels_stack(w, generator, stack, label).await;
    w.last_ok();
}

#[when(expr = "the generator {string} sends the stack {string} to the agent {string}")]
async fn sends_stack(w: &mut World, generator: String, stack: String, agent: String) {
    let pak = w.generator(&generator).pak.clone();
    let stack = w.real(&w.stack(&stack).name);
    let agent = w.agent(&agent).name.clone();
    w.brokkr(&pak, &["stack", "target", &stack, &agent]).await;
}

#[when(expr = "the generator {string} sends the stack {string} to the agent {string} by its id")]
async fn sends_stack_by_id(w: &mut World, generator: String, stack: String, agent: String) {
    let pak = w.generator(&generator).pak.clone();
    let stack = w.real(&w.stack(&stack).name);
    let agent = w.agent(&agent).id.to_string();
    w.brokkr(&pak, &["stack", "target", &stack, &agent]).await;
}

#[when(expr = "the generator {string} sends the stack {string} to the cluster agent")]
async fn sends_stack_to_cluster_agent(w: &mut World, generator: String, stack: String) {
    let pak = w.generator(&generator).pak.clone();
    let stack = w.real(&w.stack(&stack).name);
    let agent = w.cluster_agent_name().to_string();
    w.brokkr(&pak, &["stack", "target", &stack, &agent]).await;
    w.last_ok();
}

#[when(expr = "the generator {string} rotates its PAK")]
async fn rotates_pak(w: &mut World, generator: String) {
    let pak = w.generator(&generator).pak.clone();
    w.brokkr(&pak, &["generator", "rotate"]).await;
    if w.last().ok {
        let new_pak = w.printed_pak();
        let t = w.generator_mut(&generator);
        let old = std::mem::replace(&mut t.pak, new_pak);
        t.previous_paks.push(old);
    }
}

#[when(expr = "the generator {string} deletes the stack {string}")]
async fn deletes_stack(w: &mut World, generator: String, stack: String) {
    w.note_stack(&stack).await;
    let pak = w.generator(&generator).pak.clone();
    let stack = w.real(&w.stack(&stack).name);
    w.brokkr(&pak, &["stack", "delete", &stack]).await;
}

// What a generator may not do

#[when(expr = "the generator {string} registers the agent {string} with itself")]
async fn registers_agent(w: &mut World, generator: String, agent: String) {
    // By id on both sides: a generator PAK cannot read the generator list, and it
    // does not see an agent that is not registered with it.
    let pak = w.generator(&generator).pak.clone();
    let generator_id = w.generator(&generator).id.to_string();
    let agent_id = w.agent(&agent).id.to_string();
    w.brokkr(
        &pak,
        &[
            "register",
            "--agent",
            &agent_id,
            "--generator",
            &generator_id,
        ],
    )
    .await;
}

#[when(expr = "the generator {string} creates the agent {string} in cluster {string}")]
async fn creates_agent(w: &mut World, generator: String, agent: String, cluster: String) {
    let pak = w.generator(&generator).pak.clone();
    let agent = w.real(&agent);
    w.brokkr(&pak, &["agent", "create", &agent, "--cluster", &cluster])
        .await;
}

#[when(expr = "the generator {string} creates the generator {string}")]
async fn creates_generator(w: &mut World, generator: String, other: String) {
    let pak = w.generator(&generator).pak.clone();
    let other = w.real(&other);
    w.brokkr(&pak, &["generator", "create", &other]).await;
}

// Reads that go to the broker directly, because no command makes them

#[when(expr = "the generator {string} reads the record of the generator {string}")]
async fn reads_generator_record(w: &mut World, generator: String, other: String) {
    let pak = w.generator(&generator).pak.clone();
    let other = w.generator(&other).id;
    let answer = w
        .api
        .get_generator(&pak, other)
        .await
        .expect("ask the broker for the record");
    w.answer = Some(answer);
}

#[when(expr = "the generator {string} reads the health of the stack {string} by its id")]
async fn reads_stack_health(w: &mut World, generator: String, stack: String) {
    let pak = w.generator(&generator).pak.clone();
    let owner = w.stack(&stack).owner.clone();
    let owner_pak = w.generator(&owner).pak.clone();
    let real = w.real(&w.stack(&stack).name);
    let stacks = w
        .api
        .list_stacks(&owner_pak)
        .await
        .expect("the owner lists its stacks");
    let id = crate::api::named(&stacks, &real)
        .and_then(crate::api::id_of)
        .unwrap_or_else(|| panic!("the owner does not see its stack {real:?}"));
    let answer = w
        .api
        .stack_health(&pak, id)
        .await
        .expect("ask the broker for the health");
    w.answer = Some(answer);
}

#[when(expr = "the generator {string} reads the labels of the agent {string}")]
async fn reads_agent_labels(w: &mut World, generator: String, agent: String) {
    let pak = w.generator(&generator).pak.clone();
    let agent = w.agent(&agent).id;
    let answer = w
        .api
        .agent_labels(&pak, agent)
        .await
        .expect("ask the broker for the labels");
    w.answer = Some(answer);
}

// The stack lifecycle (BROKKR-T-0398)

/// A second apply of the scenario's folder with a changed manifest, so the
/// stack gets a new revision.
#[given(expr = "the generator {string} has applied a changed folder as the stack {string}")]
#[when(expr = "the generator {string} applies a changed folder as the stack {string}")]
async fn applies_changed_folder(w: &mut World, generator: String, stack: String) {
    let bundle = w.same_bundle();
    let manifest = bundle.join("configmap.yaml");
    let yaml = std::fs::read_to_string(&manifest).expect("read the manifest");
    std::fs::write(&manifest, format!("{yaml}  changed: \"{}\"\n", yaml.len()))
        .expect("write the changed manifest");
    apply(w, &generator, &stack, bundle).await;
}

// Work orders of a generator (BROKKR-T-0410). The generator uses the
// `brokkr work-order` commands with its own PAK.

/// The id of a work order of the scenario, as text for a command.
fn work_order_arg(w: &World, name: &str) -> String {
    w.work_orders
        .get(name)
        .unwrap_or_else(|| panic!("the scenario has not created the work order {name:?} yet"))
        .to_string()
}

/// `brokkr work-order create` as the generator, with `targets` after the
/// file. A created work order is kept on the `World` by its feature name, so
/// the cleanup removes it.
async fn create_work_order(
    w: &mut World,
    generator: &str,
    name: &str,
    work_type: &str,
    yaml: &str,
    targets: &[&str],
) {
    let pak = w.generator(generator).pak.clone();
    let file = w.work_order_file(yaml).to_string_lossy().into_owned();
    let mut args = vec!["work-order", "create", "-f", &file, "--type", work_type];
    args.extend_from_slice(targets);
    w.brokkr(&pak, &args).await;
    if w.last().ok {
        let id = w.printed_work_order_id();
        w.work_orders.insert(name.to_string(), id);
    }
}

/// A work order that no real agent runs: the scenario plays the agent.
fn plain_yaml(w: &World, name: &str) -> String {
    format!("# bdd work order {}\n", w.real(name))
}

#[given(expr = "the generator {string} has created the work order {string} for the agent {string}")]
async fn has_created_work_order(w: &mut World, generator: String, name: String, agent: String) {
    let agent = w.agent(&agent).name.clone();
    let yaml = plain_yaml(w, &name);
    create_work_order(w, &generator, &name, "bdd", &yaml, &["--agent", &agent]).await;
    w.last_ok();
}

/// By the id of the agent: a generator cannot look up the name of an agent
/// that is not registered with it.
#[when(
    expr = "the generator {string} creates the work order {string} for the agent {string} by its id"
)]
async fn creates_work_order_by_id(w: &mut World, generator: String, name: String, agent: String) {
    let agent = w.agent(&agent).id.to_string();
    let yaml = plain_yaml(w, &name);
    create_work_order(w, &generator, &name, "bdd", &yaml, &["--agent", &agent]).await;
}

#[given(
    expr = "the generator {string} has created the work order {string} for the label {string} of this scenario"
)]
async fn has_created_work_order_for_label(
    w: &mut World,
    generator: String,
    name: String,
    label: String,
) {
    let label = crate::steps::admin::scenario_label(w, &label);
    let yaml = plain_yaml(w, &name);
    create_work_order(w, &generator, &name, "bdd", &yaml, &["--label", &label]).await;
    w.last_ok();
}

/// A `custom` work order for the cluster agent: one Job that prints the line
/// of the scenario. It has no retry, so a failure ends in the log at once.
#[when(
    expr = "the generator {string} launches the work order {string} on the cluster agent with a job that prints a line"
)]
async fn launches_job(w: &mut World, generator: String, name: String) {
    let agent = w.cluster_agent_name().to_string();
    let job = w.job_name(&name);
    let line = w.job_line(&name);
    let yaml = format!(
        "apiVersion: batch/v1\nkind: Job\nmetadata:\n  name: {job}\n  namespace: default\nspec:\n  backoffLimit: 0\n  ttlSecondsAfterFinished: 600\n  template:\n    spec:\n      restartPolicy: Never\n      containers:\n        - name: main\n          image: busybox:1.36\n          command: [\"sh\", \"-c\", \"echo {line}; sleep 2; exit 0\"]\n"
    );
    create_work_order(
        w,
        &generator,
        &name,
        "custom",
        &yaml,
        &["--agent", &agent, "--max-retries", "0"],
    )
    .await;
    w.last_ok();
}

#[when(expr = "the generator {string} asks for the status of the work order {string}")]
async fn asks_work_order_status(w: &mut World, generator: String, name: String) {
    let pak = w.generator(&generator).pak.clone();
    let id = work_order_arg(w, &name);
    w.brokkr(&pak, &["work-order", "status", &id]).await;
}

#[when(expr = "the generator {string} lists its work orders")]
async fn lists_work_orders(w: &mut World, generator: String) {
    let pak = w.generator(&generator).pak.clone();
    w.brokkr(&pak, &["work-order", "list"]).await;
}

#[when(expr = "the generator {string} reads the history of its work orders")]
async fn reads_work_order_history(w: &mut World, generator: String) {
    let pak = w.generator(&generator).pak.clone();
    w.brokkr(&pak, &["work-order", "history"]).await;
}

#[when(expr = "the generator {string} deletes the work order {string}")]
async fn deletes_work_order(w: &mut World, generator: String, name: String) {
    let pak = w.generator(&generator).pak.clone();
    let id = work_order_arg(w, &name);
    w.brokkr(&pak, &["work-order", "delete", &id]).await;
}

#[when(expr = "the generator {string} reads the logs of the work order {string}")]
async fn reads_work_order_logs(w: &mut World, generator: String, name: String) {
    let pak = w.generator(&generator).pak.clone();
    let id = work_order_arg(w, &name);
    w.brokkr(&pak, &["work-order", "logs", &id]).await;
}

#[when(expr = "the generator {string} reads the events of the work order {string}")]
async fn reads_work_order_events(w: &mut World, generator: String, name: String) {
    let pak = w.generator(&generator).pak.clone();
    let id = work_order_arg(w, &name);
    w.brokkr(&pak, &["work-order", "events", &id]).await;
}

/// A generator lists only the agents registered with it, so the CLI cannot
/// find this agent by its name. The error must still give the next step
/// (BROKKR-T-0418).
#[when(
    expr = "the generator {string} creates the work order {string} for the agent {string} by its name"
)]
async fn creates_work_order_by_name(w: &mut World, generator: String, name: String, agent: String) {
    let agent = w.agent(&agent).name.clone();
    let yaml = plain_yaml(w, &name);
    create_work_order(w, &generator, &name, "bdd", &yaml, &["--agent", &agent]).await;
}
