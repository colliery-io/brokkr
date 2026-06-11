/*
 * Copyright (c) 2025-2026 Dylan Storey
 * Licensed under the Elastic License 2.0.
 * See LICENSE file in the project root for full license text.
 */

//! `brokkr` — command-line client for the Brokkr control plane.
//!
//! The headline command is `brokkr apply`: point it at a folder of Kubernetes
//! manifests and a stack name, and it becomes that stack's desired state. It is
//! a thin shell over the Rust SDK's idempotent [`BrokkrClient::apply`], so a CI
//! job or a developer loop can re-run it cheaply — an unchanged folder is a
//! no-op.

mod config;

use brokkr_client::{ApplyOutcome, BrokkrClient};
use clap::{Args, Parser, Subcommand};
use config::{ConfigLayer, ResolvedConfig};
use std::path::PathBuf;
use std::process::ExitCode;

/// Brokkr control-plane CLI.
#[derive(Debug, Parser)]
#[command(name = "brokkr", version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Command,

    #[command(flatten)]
    connection: ConnectionArgs,
}

/// Connection settings shared by every command. Each is also resolvable from an
/// environment variable or `~/.brokkr/config`; see [`config`].
#[derive(Debug, Args)]
struct ConnectionArgs {
    /// Broker base URL (the `/api/v1` suffix is added if omitted).
    #[arg(long, global = true)]
    broker_url: Option<String>,

    /// Project Access Key (PAK) to authenticate with.
    #[arg(long, global = true)]
    pak: Option<String>,

    /// Path to the config file (default: ~/.brokkr/config).
    #[arg(long, global = true, value_name = "PATH")]
    config: Option<PathBuf>,
}

#[derive(Debug, Subcommand)]
enum Command {
    /// Make a folder of manifests the desired state of a stack (idempotent).
    Apply(ApplyArgs),
}

#[derive(Debug, Args)]
struct ApplyArgs {
    /// Folder of manifests (top-level `*.yaml`/`*.yml`) or a single file.
    #[arg(short = 'f', long = "filename", value_name = "PATH")]
    filename: PathBuf,

    /// Name of the stack; created if it does not exist.
    #[arg(long)]
    stack: String,

    /// Targeting label for agent fan-out, e.g. `env:prod`. Repeatable.
    #[arg(long = "target-label", value_name = "LABEL")]
    target_label: Vec<String>,
}

#[tokio::main]
async fn main() -> ExitCode {
    let cli = Cli::parse();
    match run(cli).await {
        Ok(()) => ExitCode::SUCCESS,
        Err(message) => {
            eprintln!("error: {message}");
            ExitCode::FAILURE
        }
    }
}

async fn run(cli: Cli) -> Result<(), String> {
    let resolved = resolve_connection(&cli.connection)?;
    let client = BrokkrClient::builder(resolved.broker_url)
        .token(resolved.pak)
        .build()
        .map_err(|e| format!("failed to build client: {e}"))?;

    match cli.command {
        Command::Apply(args) => apply(&client, args).await,
    }
}

/// Layer the CLI flags over the environment and the config file.
fn resolve_connection(args: &ConnectionArgs) -> Result<ResolvedConfig, String> {
    let flag = ConfigLayer {
        broker_url: args.broker_url.clone(),
        pak: args.pak.clone(),
    };
    let env = config::env_layer();
    let file = match args.config.clone().or_else(config::default_config_path) {
        Some(path) => config::load_file(&path)?,
        None => ConfigLayer::default(),
    };
    config::resolve(&flag, &env, &file)
}

async fn apply(client: &BrokkrClient, args: ApplyArgs) -> Result<(), String> {
    let outcome = client
        .apply(&args.stack, &args.filename, &args.target_label)
        .await
        .map_err(|e| e.to_string())?;

    match outcome {
        ApplyOutcome::Created(obj) => println!(
            "created stack \"{}\": first revision (sequence {})",
            args.stack, obj.sequence_id
        ),
        ApplyOutcome::Updated(obj) => println!(
            "updated stack \"{}\": new revision (sequence {})",
            args.stack, obj.sequence_id
        ),
        ApplyOutcome::Unchanged => {
            println!("unchanged: stack \"{}\" already current", args.stack)
        }
    }
    Ok(())
}
