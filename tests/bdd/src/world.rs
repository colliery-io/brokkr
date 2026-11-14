/*
 * Copyright (c) 2025-2026 Dylan Storey
 * Licensed under the Elastic License 2.0.
 * See LICENSE file in the project root for full license text.
 */

//! The state of one scenario: the generators, agents and stacks it created,
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

/// A generator the scenario created. `pak` is its current PAK; a rotation moves
/// the old one to `previous_paks`.
#[derive(Clone)]
pub struct Generator {
    pub id: Uuid,
    pub name: String,
    pub pak: String,
    pub previous_paks: Vec<String>,
}

impl fmt::Debug for Generator {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Generator")
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

/// A stack a generator of the scenario applied.
#[derive(Debug, Clone)]
pub struct Stack {
    pub name: String,
    /// The generator that owns it, by its feature name.
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
    pub generators: BTreeMap<String, Generator>,
    pub agents: BTreeMap<String, AgentRecord>,
    pub stacks: BTreeMap<String, Stack>,
    /// The work orders the admin created, by the name the feature uses.
    pub work_orders: BTreeMap<String, Uuid>,
    /// The folder of manifests the generator applies.
    pub bundle: Option<TempDir>,
    /// The last binary run: `brokkr` or `brokkr-agent`.
    pub last: Option<Run>,
    /// The last answer of the broker to a direct request.
    pub answer: Option<Answer>,
    /// The cluster agent, once a step looked it up, and the generators it was
    /// registered with, for the cleanup.
    pub cluster_agent_id: Option<Uuid>,
    pub cluster_registrations: Vec<Uuid>,
    /// The stacks a step noted before their delete, by their feature name.
    pub noted_stacks: BTreeMap<String, NotedStack>,
    /// The folders of the YAML files that `brokkr work-order create` reads.
    pub work_order_files: Vec<TempDir>,
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
            generators: BTreeMap::new(),
            agents: BTreeMap::new(),
            stacks: BTreeMap::new(),
            work_orders: BTreeMap::new(),
            bundle: None,
            last: None,
            answer: None,
            cluster_agent_id: None,
            cluster_registrations: Vec::new(),
            noted_stacks: BTreeMap::new(),
            work_order_files: Vec::new(),
        }
    }

    /// The real name of a thing the feature calls `name`.
    pub fn real(&self, name: &str) -> String {
        format!("{name}-{}", self.run)
    }

    pub fn admin(&self) -> &str {
        &self.env.admin_pak
    }

    pub fn generator(&self, name: &str) -> &Generator {
        self.generators
            .get(name)
            .unwrap_or_else(|| panic!("the scenario has not created the generator {name:?} yet"))
    }

    pub fn generator_mut(&mut self, name: &str) -> &mut Generator {
        self.generators
            .get_mut(name)
            .unwrap_or_else(|| panic!("the scenario has not created the generator {name:?} yet"))
    }

    pub fn agent(&self, name: &str) -> &AgentRecord {
        self.agents
            .get(name)
            .unwrap_or_else(|| panic!("the scenario has not created the agent {name:?} yet"))
    }

    pub fn stack(&self, name: &str) -> &Stack {
        self.stacks.get(name).unwrap_or_else(|| {
            panic!("no generator of the scenario has applied the stack {name:?} yet")
        })
    }

    /// The id of a stack of the scenario, as the admin finds it by its name.
    pub async fn stack_id(&self, name: &str) -> Uuid {
        let real = self.real(&self.stack(name).name);
        let stacks = self
            .api
            .list_stacks(self.admin())
            .await
            .expect("the admin lists the stacks");
        api::named(&stacks, &real)
            .and_then(api::id_of)
            .unwrap_or_else(|| panic!("the admin does not see the stack {real:?}"))
    }

    /// The name of the Job that the work order `name` of this scenario runs
    /// in the cluster. It is unique, so a later run does not find an old Job.
    pub fn job_name(&self, name: &str) -> String {
        format!("bdd-{}-{}", name, self.run)
    }

    /// The line that the Job of the work order `name` prints.
    pub fn job_line(&self, name: &str) -> String {
        format!("brokkr bdd line of {}", self.job_name(name))
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

    /// The id in the `created work order <id> (type …)` line of the last run.
    pub fn printed_work_order_id(&self) -> Uuid {
        let run = self.last_ok();
        run.stdout
            .lines()
            .find_map(|l| l.strip_prefix("created work order "))
            .and_then(|rest| rest.split_whitespace().next())
            .and_then(|id| Uuid::parse_str(id).ok())
            .unwrap_or_else(|| {
                panic!(
                    "no `created work order <id>` line in the output:\n{}",
                    run.show()
                )
            })
    }

    /// A file with the YAML of a work order, for `brokkr work-order create -f`.
    /// It lives as long as the scenario.
    pub fn work_order_file(&mut self, yaml: &str) -> PathBuf {
        let dir = tempfile::tempdir().expect("a temp dir for the work order");
        let path = dir.path().join("work-order.yaml");
        std::fs::write(&path, yaml).expect("write the work order file");
        self.work_order_files.push(dir);
        path
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

    /// The folder the generator applied before, for a second apply.
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

        // A work order that ended is in the log and answers 404 here.
        for (name, id) in &self.work_orders {
            report(
                format!("delete work order {name}"),
                api.delete_work_order(&admin, *id).await,
            );
        }
        // Stacks first: a stack of a deleted generator has no owner to find it by.
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
        // scenario's generators go, which also removes its targets for them.
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
        for generator in self.generators.values() {
            report(
                format!("delete generator {}", generator.name),
                api.delete_generator(&admin, generator.id).await,
            );
        }
    }
}

// The stack lifecycle (BROKKR-T-0398)

/// What a scenario knows of a stack after its delete: the admin no longer
/// finds the stack by its name, so the ids are read before.
#[derive(Debug, Clone, Copy)]
pub struct NotedStack {
    pub id: Uuid,
    /// The latest deployment object of the stack before the delete.
    pub last_revision: Option<Uuid>,
}

impl World {
    /// Read the id and the latest revision of a stack, and keep them for the
    /// steps after its delete. Best effort: a failed read notes nothing, and
    /// only a step that needs the note fails.
    pub async fn note_stack(&mut self, name: &str) {
        let real = self.real(&self.stack(name).name);
        let Ok(stacks) = self.api.list_stacks(self.admin()).await else {
            return;
        };
        let Some(id) = api::named(&stacks, &real).and_then(api::id_of) else {
            return;
        };
        let last_revision = self
            .api
            .revisions(self.admin(), id)
            .await
            .ok()
            .and_then(|revisions| api::latest(&revisions));
        self.noted_stacks
            .insert(name.to_string(), NotedStack { id, last_revision });
    }

    /// The note of a deleted stack.
    pub fn noted(&self, name: &str) -> NotedStack {
        *self.noted_stacks.get(name).unwrap_or_else(|| {
            panic!("the scenario did not read the stack {name:?} before its delete")
        })
    }

    /// The last revision of a deleted stack.
    pub fn noted_revision(&self, name: &str) -> Uuid {
        self.noted(name)
            .last_revision
            .unwrap_or_else(|| panic!("the stack {name:?} had no revision before its delete"))
    }
}
