/*
 * Copyright (c) 2025-2026 Dylan Storey
 * Licensed under the Elastic License 2.0.
 * See LICENSE file in the project root for full license text.
 */

//! Reading the operator console with a browser (BROKKR-T-0400).
//!
//! `console/read.mjs` opens one view of the console that the broker serves,
//! reads what it shows and prints it as JSON. This module runs it and parses
//! the JSON; the steps in `steps/console.rs` check it. The script uses the
//! Playwright of `console/package.json`: `npm install` in that folder once.
//!
//! The console polls the broker, and other scenarios change the fleet at the
//! same time, so the steps read again until the view shows what they expect,
//! or until [`WAIT`] runs out.

use std::path::PathBuf;
use std::time::Duration;

use serde::Deserialize;
use tokio::process::Command;

/// How long a step reads the console again before it fails.
pub const WAIT: Duration = Duration::from_secs(45);

/// The pause between two reads.
pub const POLL: Duration = Duration::from_secs(2);

/// The folder of `read.mjs` and its `node_modules`.
fn console_dir() -> PathBuf {
    PathBuf::from(concat!(env!("CARGO_MANIFEST_DIR"), "/console"))
}

/// One agent row of the Fleet view.
#[derive(Debug, Clone, Deserialize)]
pub struct FleetRow {
    pub name: String,
    /// Shown in a failure message.
    #[allow(dead_code)]
    pub status: Option<String>,
    pub health: Option<String>,
}

/// What the Fleet view shows: its KPI tiles by label, and the agent rows.
#[derive(Debug, Clone, Deserialize)]
pub struct Fleet {
    pub tiles: std::collections::BTreeMap<String, String>,
    pub agents: Vec<FleetRow>,
}

impl Fleet {
    pub fn agent(&self, name: &str) -> Option<&FleetRow> {
        self.agents.iter().find(|a| a.name == name)
    }
}

/// One row of the Work orders panel of the Overview.
#[derive(Debug, Clone, Deserialize)]
pub struct WorkOrderRow {
    pub id8: String,
    /// Shown in a failure message.
    #[allow(dead_code)]
    pub work_type: Option<String>,
    /// Shown in a failure message.
    #[allow(dead_code)]
    pub status: Option<String>,
}

/// One row of the Fleet by cluster panel of the Overview.
#[derive(Debug, Clone, Deserialize)]
pub struct ClusterRow {
    pub name: String,
    /// "<up>/<total> up".
    pub caption: String,
}

/// What the Overview shows: its KPI tiles by label, the active work orders
/// and the cluster rows.
#[derive(Debug, Clone, Deserialize)]
pub struct Overview {
    pub tiles: std::collections::BTreeMap<String, String>,
    pub work_orders: Vec<WorkOrderRow>,
    pub clusters: Vec<ClusterRow>,
}

impl Overview {
    pub fn cluster(&self, name: &str) -> Option<&ClusterRow> {
        self.clusters.iter().find(|c| c.name == name)
    }
}

/// Run `node read.mjs <view>` against `broker_url` and parse its JSON.
async fn read(broker_url: &str, view: &str) -> Result<serde_json::Value, String> {
    let node = std::env::var("BDD_NODE").unwrap_or_else(|_| "node".to_string());
    let dir = console_dir();
    if !dir.join("node_modules").exists() {
        return Err(format!(
            "no node_modules in {}: run `npm install` there once",
            dir.display()
        ));
    }
    let out = tokio::time::timeout(
        Duration::from_secs(90),
        Command::new(&node)
            .arg("read.mjs")
            .arg(view)
            .current_dir(&dir)
            .env("BROKER_URL", broker_url)
            .kill_on_drop(true)
            .output(),
    )
    .await
    .map_err(|_| format!("`{node} read.mjs {view}` did not end in 90 seconds"))?
    .map_err(|e| format!("cannot run `{node} read.mjs {view}`: {e}"))?;
    if !out.status.success() {
        return Err(format!(
            "`{node} read.mjs {view}` failed:\n{}",
            String::from_utf8_lossy(&out.stderr)
        ));
    }
    serde_json::from_slice(&out.stdout).map_err(|e| {
        format!(
            "`{node} read.mjs {view}` printed no JSON ({e}):\n{}",
            String::from_utf8_lossy(&out.stdout)
        )
    })
}

pub async fn fleet(broker_url: &str) -> Result<Fleet, String> {
    let v = read(broker_url, "fleet").await?;
    serde_json::from_value(v).map_err(|e| format!("the Fleet JSON: {e}"))
}

pub async fn overview(broker_url: &str) -> Result<Overview, String> {
    let v = read(broker_url, "overview").await?;
    serde_json::from_value(v).map_err(|e| format!("the Overview JSON: {e}"))
}

/// Read with `read` until `check` gives `Ok`, or until [`WAIT`] runs out.
/// Panics with the last reason and the last read.
pub async fn until<T, R, F, C>(what: &str, mut read: R, check: C) -> T
where
    T: std::fmt::Debug,
    R: FnMut() -> F,
    F: std::future::Future<Output = Result<T, String>>,
    C: Fn(&T) -> Result<(), String>,
{
    let deadline = tokio::time::Instant::now() + WAIT;
    loop {
        let reason = match read().await {
            Ok(seen) => match check(&seen) {
                Ok(()) => return seen,
                Err(why) => format!("{why}\nthe console showed: {seen:?}"),
            },
            Err(e) => e,
        };
        if tokio::time::Instant::now() >= deadline {
            panic!(
                "{what}: not true after {} seconds.\n{reason}",
                WAIT.as_secs()
            );
        }
        tokio::time::sleep(POLL).await;
    }
}
