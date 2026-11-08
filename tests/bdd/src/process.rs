/*
 * Copyright (c) 2025-2026 Dylan Storey
 * Licensed under the Elastic License 2.0.
 * See LICENSE file in the project root for full license text.
 */

//! Running the real binaries: `brokkr` (the CLI) and `brokkr-agent`.

use std::path::Path;
use std::process::Stdio;
use std::time::Duration;

use tokio::io::AsyncReadExt as _;
use tokio::process::Command;

/// A home no binary has a config in, so no `~/.brokkr/config` or kubeconfig
/// of the machine leaks into a run.
const EMPTY_HOME: &str = "/nonexistent-brokkr-home";

/// What one run of a binary did.
#[derive(Debug, Clone)]
pub struct Run {
    /// The command line, for the failure message.
    pub command: String,
    /// The process exited 0.
    pub ok: bool,
    /// The process did not exit in time and was stopped.
    pub timed_out: bool,
    pub stdout: String,
    pub stderr: String,
}

impl Run {
    /// stdout and stderr together, for checks that do not care which.
    pub fn output(&self) -> String {
        format!("{}{}", self.stdout, self.stderr)
    }

    /// The run, shown for a failed step.
    pub fn show(&self) -> String {
        let status = if self.timed_out {
            "stopped after the timeout"
        } else if self.ok {
            "exit 0"
        } else {
            "exit non-zero"
        };
        format!(
            "$ {}\n[{status}]\n--- stdout ---\n{}--- stderr ---\n{}",
            self.command, self.stdout, self.stderr
        )
    }
}

/// Run `brokkr <args>` with a broker URL and a PAK from the environment.
pub async fn brokkr(cli: &Path, broker_url: &str, pak: &str, args: &[&str]) -> Run {
    let command = format!("brokkr {}", args.join(" "));
    let out = Command::new(cli)
        .args(args)
        .env_clear()
        .env("PATH", std::env::var("PATH").unwrap_or_default())
        .env("HOME", EMPTY_HOME)
        .env("BROKKR_BROKER_URL", broker_url)
        .env("BROKKR_PAK", pak)
        .output()
        .await
        .unwrap_or_else(|e| panic!("cannot run {}: {e}", cli.display()));
    Run {
        command,
        ok: out.status.success(),
        timed_out: false,
        stdout: String::from_utf8_lossy(&out.stdout).into_owned(),
        stderr: String::from_utf8_lossy(&out.stderr).into_owned(),
    }
}

/// Start `brokkr-agent start` with a PAK and, when given, a configured name
/// and cluster, and wait for it to stop or for `timeout`. The agent has no
/// cluster here: its kubeconfig points at a missing file, so a run that gets
/// past the identity check ends on its own at the Kubernetes client.
pub async fn brokkr_agent(
    bin: &Path,
    broker_url: &str,
    pak: &str,
    name: Option<&str>,
    cluster: Option<&str>,
    timeout: Duration,
) -> Run {
    let mut shown = vec!["brokkr-agent start".to_string()];
    let mut cmd = Command::new(bin);
    cmd.arg("start")
        .env_clear()
        .env("PATH", std::env::var("PATH").unwrap_or_default())
        .env("HOME", EMPTY_HOME)
        .env("BROKKR__AGENT__BROKER_URL", broker_url)
        .env("BROKKR__AGENT__PAK", pak)
        .env(
            "BROKKR__AGENT__KUBECONFIG_PATH",
            "/nonexistent-brokkr-kubeconfig",
        )
        .env("BROKKR__AGENT__HEALTH_PORT", "0")
        .env("BROKKR__AGENT__MAX_RETRIES", "3")
        .env("BROKKR__AGENT__WS_FORCE_REST", "true")
        .env("BROKKR__LOG__LEVEL", "info")
        .env("BROKKR__LOG__FORMAT", "text")
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .kill_on_drop(true);
    if let Some(name) = name {
        cmd.env("BROKKR__AGENT__AGENT_NAME", name);
        shown.push(format!("(BROKKR__AGENT__AGENT_NAME={name})"));
    }
    if let Some(cluster) = cluster {
        cmd.env("BROKKR__AGENT__CLUSTER_NAME", cluster);
        shown.push(format!("(BROKKR__AGENT__CLUSTER_NAME={cluster})"));
    }
    let command = shown.join(" ");

    let mut child = cmd
        .spawn()
        .unwrap_or_else(|e| panic!("cannot run {}: {e}", bin.display()));
    let mut stdout = child.stdout.take().expect("piped stdout");
    let mut stderr = child.stderr.take().expect("piped stderr");
    let out_task = tokio::spawn(async move {
        let mut buf = Vec::new();
        let _ = stdout.read_to_end(&mut buf).await;
        buf
    });
    let err_task = tokio::spawn(async move {
        let mut buf = Vec::new();
        let _ = stderr.read_to_end(&mut buf).await;
        buf
    });

    let (ok, timed_out) = match tokio::time::timeout(timeout, child.wait()).await {
        Ok(Ok(status)) => (status.success(), false),
        Ok(Err(e)) => panic!("waiting for brokkr-agent: {e}"),
        Err(_) => {
            let _ = child.kill().await;
            (false, true)
        }
    };
    let stdout = out_task.await.unwrap_or_default();
    let stderr = err_task.await.unwrap_or_default();
    Run {
        command,
        ok,
        timed_out,
        stdout: String::from_utf8_lossy(&stdout).into_owned(),
        stderr: String::from_utf8_lossy(&stderr).into_owned(),
    }
}
