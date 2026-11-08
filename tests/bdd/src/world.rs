/*
 * Copyright (c) 2025-2026 Dylan Storey
 * Licensed under the Elastic License 2.0.
 * See LICENSE file in the project root for full license text.
 */

//! The state of one scenario: the tenants, agents and stacks it created,
//! the last command it ran, and the last answer of the broker.
//!
//! A feature names things plainly (`team-robin`, `prod-1`, `payments`). The
//! world gives each a unique real name for this scenario, so scenarios run
//! at the same time and the suite runs twice against one broker.

use std::collections::BTreeMap;
use std::fmt;
use std::path::{Path, PathBuf};
use std::time::Duration;

use tempfile::TempDir;
use uuid::Uuid;

use crate::api::{self, Answer, Api};
use crate::process::{self, Run};

/// The development broker's admin PAK (crates/brokkr-utils/default.toml).
/// Publicly known; it opens only a broker that runs the embedded default hash.
const DEV_ADMIN_PAK: &str = "brokkr_BR3rVsDa_GK3QN7CDUzYc6iKgMkJ98M2WSimM5t6U8";

/// What the suite is pointed at.
#[derive(Clone)]
pub struct Env {
    pub broker_url: String,
    pub admin_pak: String,
    /// The `brokkr` binary.
    pub cli: PathBuf,
    /// The `brokkr-agent` binary, for the scenarios that start an agent.
    pub agent_bin: Option<PathBuf>,
    /// The name of the agent that runs in the test stack's cluster.
    pub cluster_agent: Option<String>,
}

impl Env {
    pub fn from_process_env() -> Self {
        let var = |name: &str| std::env::var(name).ok().filter(|v| !v.trim().is_empty());
        Self {
            broker_url: var("BROKER_URL").unwrap_or_else(|| "http://localhost:3000".to_string()),
            admin_pak: var("ADMIN_PAK").unwrap_or_else(|| DEV_ADMIN_PAK.to_string()),
            cli: PathBuf::from(var("BROKKR_CLI").unwrap_or_else(|| {
                panic!(
                    "BROKKR_CLI is not set. Run `angreal tests bdd`, or set BROKKR_CLI to the \
                     path of the brokkr binary"
                )
            })),
            agent_bin: var("BROKKR_AGENT").map(PathBuf::from),
            cluster_agent: var("BROKKR_CLUSTER_AGENT"),
        }
    }
}

impl fmt::Debug for Env {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Env")
            .field("broker_url", &self.broker_url)
            .field("admin_pak", &masked(&self.admin_pak))
            .field("cli", &self.cli)
            .field("agent_bin", &self.agent_bin)
            .field("cluster_agent", &self.cluster_agent)
            .finish()
    }
}

/// A PAK as it may appear in a log: its last four characters.
pub fn masked(pak: &str) -> String {
    let tail: String = pak
        .chars()
        .rev()
        .take(4)
        .collect::<Vec<_>>()
        .into_iter()
        .rev()
        .collect();
    format!("…{tail}")
}

/// A tenant the scenario created. `pak` is its current PAK; a rotation moves
/// the old one to `previous_paks`.
#[derive(Clone)]
pub struct Tenant {
    pub id: Uuid,
    pub name: String,
    pub pak: String,
    pub previous_paks: Vec<String>,
}

impl fmt::Debug for Tenant {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Tenant")
            .field("id", &self.id)
            .field("name", &self.name)
            .field("pak", &masked(&self.pak))
            .field("previous_paks", &self.previous_paks.len())
            .finish()
    }
}

/// An agent record the scenario created.
#[derive(Clone)]
pub struct AgentRecord {
    pub id: Uuid,
    pub name: String,
    pub cluster: String,
    pub pak: String,
}

impl fmt::Debug for AgentRecord {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("AgentRecord")
            .field("id", &self.id)
            .field("name", &self.name)
            .field("cluster", &self.cluster)
            .field("pak", &masked(&self.pak))
            .finish()
    }
}

/// A stack a tenant of the scenario applied.
#[derive(Debug, Clone)]
pub struct Stack {
    pub name: String,
    /// The tenant that owns it, by its feature name.
    pub owner: String,
}

#[derive(Debug, cucumber::World)]
#[world(init = Self::new)]
pub struct World {
    pub env: Env,
    pub api: Api,
    /// The unique suffix of this scenario's names.
    pub run: String,
    /// By the name the feature uses.
    pub tenants: BTreeMap<String, Tenant>,
    pub agents: BTreeMap<String, AgentRecord>,
    pub stacks: BTreeMap<String, Stack>,
    /// The folder of manifests the tenant applies.
    pub bundle: Option<TempDir>,
    /// The last binary run: `brokkr` or `brokkr-agent`.
    pub last: Option<Run>,
    /// The last answer of the broker to a direct request.
    pub answer: Option<Answer>,
    /// The cluster agent, once a step looked it up, and the tenants it was
    /// registered with, for the cleanup.
    pub cluster_agent_id: Option<Uuid>,
    pub cluster_registrations: Vec<Uuid>,
}

impl World {
    fn new() -> Self {
        let env = Env::from_process_env();
        let api = Api::new(&env.broker_url);
        let run = Uuid::new_v4().simple().to_string()[..8].to_string();
        Self {
            env,
            api,
            run,
            tenants: BTreeMap::new(),
            agents: BTreeMap::new(),
            stacks: BTreeMap::new(),
            bundle: None,
            last: None,
            answer: None,
            cluster_agent_id: None,
            cluster_registrations: Vec::new(),
        }
    }

    /// The real name of a thing the feature calls `name`.
    pub fn real(&self, name: &str) -> String {
        format!("{name}-{}", self.run)
    }

    pub fn admin(&self) -> &str {
        &self.env.admin_pak
    }

    pub fn tenant(&self, name: &str) -> &Tenant {
        self.tenants
            .get(name)
            .unwrap_or_else(|| panic!("the scenario has not created the tenant {name:?} yet"))
    }

    pub fn tenant_mut(&mut self, name: &str) -> &mut Tenant {
        self.tenants
            .get_mut(name)
            .unwrap_or_else(|| panic!("the scenario has not created the tenant {name:?} yet"))
    }

    pub fn agent(&self, name: &str) -> &AgentRecord {
        self.agents
            .get(name)
            .unwrap_or_else(|| panic!("the scenario has not created the agent {name:?} yet"))
    }

    pub fn stack(&self, name: &str) -> &Stack {
        self.stacks.get(name).unwrap_or_else(|| {
            panic!("no tenant of the scenario has applied the stack {name:?} yet")
        })
    }

    /// The name of the agent in the test stack's cluster.
    pub fn cluster_agent_name(&self) -> &str {
        self.env.cluster_agent.as_deref().unwrap_or_else(|| {
            panic!(
                "BROKKR_CLUSTER_AGENT is not set: this scenario needs the agent that runs in \
                 the test stack's cluster"
            )
        })
    }

    pub fn agent_bin(&self) -> &Path {
        self.env.agent_bin.as_deref().unwrap_or_else(|| {
            panic!(
                "BROKKR_AGENT is not set: this scenario starts the real brokkr-agent binary. \
                 Run `angreal tests bdd`, or set BROKKR_AGENT to its path"
            )
        })
    }

    // Running the binaries

    /// Run `brokkr <args>` with `pak` and keep the run as the last one.
    pub async fn brokkr(&mut self, pak: &str, args: &[&str]) -> &Run {
        let run = process::brokkr(&self.env.cli, &self.env.broker_url, pak, args).await;
        println!(
            "  [{}] $ {}  [{}]",
            self.run,
            run.command,
            if run.ok { "ok" } else { "refused" }
        );
        self.last = Some(run);
        self.last()
    }

    /// Run `brokkr-agent start` with `pak` and the given identity.
    pub async fn brokkr_agent(
        &mut self,
        pak: &str,
        name: Option<&str>,
        cluster: Option<&str>,
    ) -> &Run {
        let run = process::brokkr_agent(
            self.agent_bin(),
            &self.env.broker_url,
            pak,
            name,
            cluster,
            Duration::from_secs(60),
        )
        .await;
        println!(
            "  [{}] $ {}  [{}]",
            self.run,
            run.command,
            if run.ok { "ok" } else { "stopped" }
        );
        self.last = Some(run);
        self.last()
    }

    pub fn last(&self) -> &Run {
        self.last
            .as_ref()
            .expect("no command has run yet in this scenario")
    }

    /// The last run, which must have succeeded.
    pub fn last_ok(&self) -> &Run {
        let run = self.last();
        assert!(run.ok, "the command failed:\n{}", run.show());
        run
    }

    /// The last run, which must have been refused.
    pub fn last_refused(&self) -> &Run {
        let run = self.last();
        assert!(
            !run.ok,
            "the command succeeded, but should have been refused:\n{}",
            run.show()
        );
        run
    }

    /// Fail unless the last run's stdout holds `needle`.
    pub fn expect_output(&self, needle: &str) {
        let run = self.last_ok();
        assert!(
            run.stdout.contains(needle),
            "the output does not say {needle:?}:\n{}",
            run.show()
        );
    }

    /// Fail unless the last run's stderr holds `needle`.
    pub fn expect_error(&self, needle: &str) {
        let run = self.last_refused();
        assert!(
            run.stderr.contains(needle),
            "the error does not say {needle:?}:\n{}",
            run.show()
        );
    }

    pub fn answer(&self) -> &Answer {
        self.answer
            .as_ref()
            .expect("the scenario has not asked the broker directly yet")
    }

    // Parsing what the CLI prints

    /// The `PAK: …` line of the last run.
    pub fn printed_pak(&self) -> String {
        let run = self.last_ok();
        run.stdout
            .lines()
            .find_map(|l| l.strip_prefix("PAK: "))
            .map(|p| p.trim().to_string())
            .unwrap_or_else(|| panic!("no PAK line in the output:\n{}", run.show()))
    }

    /// The id in a `created <kind> "<name>" (<id>)…` line of the last run.
    pub fn printed_id(&self, kind: &str, name: &str) -> Uuid {
        let run = self.last_ok();
        let prefix = format!("created {kind} \"{name}\" (");
        run.stdout
            .lines()
            .find_map(|l| l.strip_prefix(&prefix))
            .and_then(|rest| rest.split(')').next())
            .and_then(|id| Uuid::parse_str(id).ok())
            .unwrap_or_else(|| panic!("no `{prefix}<id>)` line in the output:\n{}", run.show()))
    }

    // The folder of manifests

    /// A folder with one small manifest, new for this scenario. The ConfigMap
    /// name is unique, so an agent that applies it leaves nothing that a later
    /// run trips over.
    pub fn new_bundle(&mut self) -> PathBuf {
        let dir = tempfile::tempdir().expect("a temp dir for the manifests");
        let yaml = format!(
            "apiVersion: v1\nkind: ConfigMap\nmetadata:\n  name: bdd-{}\n  namespace: default\ndata:\n  run: \"{}\"\n",
            self.run, self.run
        );
        std::fs::write(dir.path().join("configmap.yaml"), yaml).expect("write the manifest");
        let path = dir.path().to_path_buf();
        self.bundle = Some(dir);
        path
    }

    /// The folder the tenant applied before, for a second apply.
    pub fn same_bundle(&self) -> PathBuf {
        self.bundle
            .as_ref()
            .map(|d| d.path().to_path_buf())
            .expect("the scenario has not applied a folder yet")
    }

    // Cleanup

    /// Remove what the scenario created, as the admin. Best effort: a failed
    /// delete is printed, not raised, so one leftover does not hide the
    /// scenario's own result.
    pub async fn cleanup(&mut self) {
        let admin = self.env.admin_pak.clone();
        let api = self.api.clone();
        let report = |what: String, answer: anyhow::Result<Answer>| match answer {
            Ok(a) if a.ok() || a.status == reqwest::StatusCode::NOT_FOUND => {}
            Ok(a) => println!("  cleanup: {what} answered {}: {}", a.status, a.body),
            Err(e) => println!("  cleanup: {what} failed: {e}"),
        };

        // Stacks first: a stack of a deleted tenant has no owner to find it by.
        if !self.stacks.is_empty()
            && let Ok(stacks) = api.list_stacks(&admin).await
        {
            for stack in self.stacks.values() {
                let real = self.real(&stack.name);
                if let Some(id) = api::named(&stacks, &real).and_then(api::id_of) {
                    report(
                        format!("delete stack {real}"),
                        api.delete_stack(&admin, id).await,
                    );
                }
            }
        }
        // The cluster agent stays: only its registrations with this
        // scenario's tenants go, which also removes its targets for them.
        if let Some(agent_id) = self.cluster_agent_id {
            for generator in std::mem::take(&mut self.cluster_registrations) {
                report(
                    format!("deregister the cluster agent from {generator}"),
                    api.deregister(&admin, generator, agent_id).await,
                );
            }
        }
        for agent in self.agents.values() {
            report(
                format!("delete agent {}", agent.name),
                api.delete_agent(&admin, agent.id).await,
            );
        }
        for tenant in self.tenants.values() {
            report(
                format!("delete tenant {}", tenant.name),
                api.delete_generator(&admin, tenant.id).await,
            );
        }
    }
}
