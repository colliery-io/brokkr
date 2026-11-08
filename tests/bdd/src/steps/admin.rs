/*
 * Copyright (c) 2025-2026 Dylan Storey
 * Licensed under the Elastic License 2.0.
 * See LICENSE file in the project root for full license text.
 */

//! What the admin does, with the admin PAK.

use cucumber::{given, when};

use crate::api;
use crate::world::{AgentRecord, Tenant, World};

/// `brokkr tenant create`; the created tenant is kept with its PAK.
async fn create_tenant(w: &mut World, name: &str) {
    let real = w.real(name);
    let admin = w.admin().to_string();
    w.brokkr(&admin, &["tenant", "create", &real, "--description", "bdd"])
        .await;
    if w.last().ok {
        let id = w.printed_id("tenant", &real);
        let pak = w.printed_pak();
        w.tenants.insert(
            name.to_string(),
            Tenant {
                id,
                name: real,
                pak,
                previous_paks: Vec::new(),
            },
        );
    }
}

#[given(expr = "the admin has created the tenant {string}")]
async fn has_created_tenant(w: &mut World, name: String) {
    create_tenant(w, &name).await;
    w.last_ok();
}

#[when(expr = "the admin creates the tenant {string}")]
async fn creates_tenant(w: &mut World, name: String) {
    create_tenant(w, &name).await;
}

#[when(expr = "the admin creates the tenant {string} again")]
async fn creates_tenant_again(w: &mut World, name: String) {
    let real = w.real(&name);
    let admin = w.admin().to_string();
    w.brokkr(&admin, &["tenant", "create", &real]).await;
}

/// `brokkr agent create`, with `--tenant` for each tenant; the created agent
/// is kept with its PAK.
async fn create_agent(w: &mut World, name: &str, cluster: &str, tenants: &[&str]) {
    let real = w.real(name);
    let admin = w.admin().to_string();
    let mut args = vec!["agent", "create", real.as_str(), "--cluster", cluster];
    let tenant_names: Vec<String> = tenants.iter().map(|t| w.tenant(t).name.clone()).collect();
    for t in &tenant_names {
        args.push("--tenant");
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
    expr = "the admin has created the agent {string} in cluster {string} for the tenant {string}"
)]
async fn has_created_agent_for_tenant(
    w: &mut World,
    name: String,
    cluster: String,
    tenant: String,
) {
    create_agent(w, &name, &cluster, &[&tenant]).await;
    w.last_ok();
}

#[when(expr = "the admin creates the agent {string} in cluster {string} for the tenant {string}")]
async fn creates_agent_for_tenant(w: &mut World, name: String, cluster: String, tenant: String) {
    create_agent(w, &name, &cluster, &[&tenant]).await;
}

#[given(expr = "the admin has registered the agent {string} with the tenant {string}")]
#[when(expr = "the admin registers the agent {string} with the tenant {string}")]
async fn registers_agent(w: &mut World, agent: String, tenant: String) {
    let agent = w.agent(&agent).name.clone();
    let tenant = w.tenant(&tenant).name.clone();
    let admin = w.admin().to_string();
    w.brokkr(
        &admin,
        &["register", "--agent", &agent, "--generator", &tenant],
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

#[when(expr = "the admin activates the agent {string}")]
async fn activates_agent(w: &mut World, agent: String) {
    let agent = w.agent(&agent).name.clone();
    let admin = w.admin().to_string();
    w.brokkr(&admin, &["agent", "activate", &agent]).await;
    w.last_ok();
}

#[when(expr = "the admin lists the tenants")]
async fn lists_tenants(w: &mut World) {
    let admin = w.admin().to_string();
    w.brokkr(&admin, &["tenant", "list"]).await;
}

#[when(expr = "the admin sends the stack {string} to the agent {string}")]
async fn sends_stack_to_agent(w: &mut World, stack: String, agent: String) {
    let stack = w.real(&w.stack(&stack).name);
    let agent = w.agent(&agent).name.clone();
    let admin = w.admin().to_string();
    w.brokkr(&admin, &["stack", "target", &stack, &agent]).await;
}

/// The agent that runs in the test stack's cluster: the admin registers it
/// with the tenant and activates it, so the tenant's stacks can reach it.
#[given(expr = "the cluster agent is registered with the tenant {string} and active")]
async fn cluster_agent_registered(w: &mut World, tenant: String) {
    let name = w.cluster_agent_name().to_string();
    let admin = w.admin().to_string();
    let agents = w.api.list_agents(&admin).await.expect("list the agents");
    let id = api::named(&agents, &name)
        .and_then(api::id_of)
        .unwrap_or_else(|| panic!("the cluster agent {name:?} is not in the broker's agent list"));
    w.cluster_agent_id = Some(id);
    let tenant_id = w.tenant(&tenant).id;
    let tenant_name = w.tenant(&tenant).name.clone();
    w.brokkr(
        &admin,
        &["register", "--agent", &name, "--generator", &tenant_name],
    )
    .await;
    w.last_ok();
    w.cluster_registrations.push(tenant_id);
    w.brokkr(&admin, &["agent", "activate", &name]).await;
    w.last_ok();
}
