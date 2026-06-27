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
use clap::{ArgGroup, Args, Parser, Subcommand};
use config::{ConfigLayer, ResolvedConfig};
use std::path::PathBuf;
use std::process::ExitCode;
use uuid::Uuid;

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

    /// Register an agent with a generator scope (admin bootstrap).
    ///
    /// Agents normally self-register on startup; use this to register an agent
    /// on its behalf — e.g. before it is live, or to add an application scope.
    /// Requires an admin PAK. Registering an already-registered pair is a no-op.
    Register(RegisterArgs),

    /// Remove an agent's registration from a generator scope (admin).
    ///
    /// DESTRUCTIVE: the broker also removes the agent's targets for that
    /// generator and notifies the agent, which then prunes the corresponding
    /// Kubernetes resources on its next reconcile. Requires an admin PAK.
    Deregister(RegisterArgs),

    /// List generator registrations — for one agent or one generator.
    Registrations(RegistrationsArgs),
}

#[derive(Debug, Args)]
struct RegisterArgs {
    /// UUID of the agent to (de)register.
    #[arg(long)]
    agent: Uuid,

    /// UUID of the generator scope to (de)register the agent with.
    #[arg(long)]
    generator: Uuid,
}

#[derive(Debug, Args)]
#[command(group(ArgGroup::new("subject").required(true).args(["agent", "generator"])))]
struct RegistrationsArgs {
    /// List the generator scopes this agent is registered with.
    #[arg(long)]
    agent: Option<Uuid>,

    /// List the agents registered with this generator scope.
    #[arg(long)]
    generator: Option<Uuid>,
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
        Command::Register(args) => register(&client, args).await,
        Command::Deregister(args) => deregister(&client, args).await,
        Command::Registrations(args) => registrations(&client, args).await,
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

async fn register(client: &BrokkrClient, args: RegisterArgs) -> Result<(), String> {
    let reg = client
        .register_agent(args.generator, Some(args.agent))
        .await
        .map_err(|e| e.to_string())?;
    println!(
        "registered agent {} with generator {} (registration {})",
        reg.agent_id, reg.generator_id, reg.id
    );
    Ok(())
}

async fn deregister(client: &BrokkrClient, args: RegisterArgs) -> Result<(), String> {
    client
        .deregister_agent(args.generator, Some(args.agent))
        .await
        .map_err(|e| e.to_string())?;
    println!(
        "deregistered agent {} from generator {}",
        args.agent, args.generator
    );
    println!(
        "note: the agent's targets for this generator were removed; it will prune \
         those resources on its next reconcile"
    );
    Ok(())
}

async fn registrations(client: &BrokkrClient, args: RegistrationsArgs) -> Result<(), String> {
    // ArgGroup guarantees exactly one of --agent / --generator is set.
    if let Some(agent) = args.agent {
        let regs = client
            .list_agent_registrations(agent)
            .await
            .map_err(|e| e.to_string())?;
        if regs.is_empty() {
            println!("agent {agent} has no generator registrations");
        } else {
            println!("agent {agent} is registered with {} generator(s):", regs.len());
            for r in regs {
                println!("  generator {}  (registered {})", r.generator_id, r.registered_at);
            }
        }
    } else if let Some(generator) = args.generator {
        let regs = client
            .list_generator_registered_agents(generator)
            .await
            .map_err(|e| e.to_string())?;
        if regs.is_empty() {
            println!("generator {generator} has no registered agents");
        } else {
            println!("generator {generator} has {} registered agent(s):", regs.len());
            for r in regs {
                println!("  agent {}  (registered {})", r.agent_id, r.registered_at);
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::CommandFactory;

    const NIL: &str = "00000000-0000-0000-0000-000000000000";

    #[test]
    fn cli_command_tree_is_valid() {
        Cli::command().debug_assert();
    }

    #[test]
    fn register_requires_both_agent_and_generator() {
        assert!(Cli::try_parse_from(["brokkr", "register", "--agent", NIL]).is_err());
        assert!(
            Cli::try_parse_from(["brokkr", "register", "--agent", NIL, "--generator", NIL]).is_ok()
        );
    }

    #[test]
    fn registrations_accepts_exactly_one_subject() {
        // exactly one subject: ok
        assert!(Cli::try_parse_from(["brokkr", "registrations", "--agent", NIL]).is_ok());
        assert!(Cli::try_parse_from(["brokkr", "registrations", "--generator", NIL]).is_ok());
        // neither: error
        assert!(Cli::try_parse_from(["brokkr", "registrations"]).is_err());
        // both: error (mutually exclusive group)
        assert!(
            Cli::try_parse_from(["brokkr", "registrations", "--agent", NIL, "--generator", NIL])
                .is_err()
        );
    }

    #[test]
    fn malformed_uuid_is_rejected() {
        assert!(Cli::try_parse_from(["brokkr", "register", "--agent", "nope", "--generator", NIL])
            .is_err());
    }
}
