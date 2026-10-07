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
//! no-op. A generator PAK applies for its own generator; an admin PAK names
//! the owner with `--generator`.

mod config;
mod fleet;

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

    /// Prefixed API Key (PAK) to authenticate with.
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

    /// Register an agent with a tenant (admin bootstrap).
    ///
    /// Agents usually register themselves when they start. Use this command to
    /// register an agent for it: for example, before the agent is live, or to
    /// add a tenant. Give the agent and the tenant by name or id. Requires an
    /// admin PAK. If the agent is already registered
    /// with the tenant, the broker returns 409 `already_registered` and the
    /// command exits with status 1.
    Register(RegisterArgs),

    /// Remove an agent's registration from a tenant (admin).
    ///
    /// DESTRUCTIVE: the broker also removes the agent's targets for that
    /// tenant and notifies the agent, which then prunes the corresponding
    /// Kubernetes resources on its next reconcile. Give the agent and the
    /// tenant by name or id. Requires an admin PAK.
    Deregister(RegisterArgs),

    /// List tenant registrations: for one agent or for one tenant.
    ///
    /// Give the agent or the tenant by name or id. A name lookup needs an
    /// admin PAK. With an agent PAK or a tenant PAK, give the id.
    Registrations(RegistrationsArgs),

    /// Activate, pause, label and list agents. Give each agent by name or id.
    #[command(subcommand)]
    Agent(fleet::AgentCommand),

    /// Label, target and list stacks. Give each stack by name or id.
    #[command(subcommand)]
    Stack(fleet::StackCommand),
}

#[derive(Debug, Args)]
struct RegisterArgs {
    /// The name or the id of the agent to register or deregister.
    #[arg(long, value_name = "NAME_OR_ID")]
    agent: String,

    /// The name or the id of the tenant. The API calls a tenant a generator.
    #[arg(long, value_name = "NAME_OR_ID")]
    generator: String,
}

#[derive(Debug, Args)]
#[command(group(ArgGroup::new("subject").required(true).args(["agent", "generator"])))]
struct RegistrationsArgs {
    /// List the tenants this agent is registered with. Give the name or the
    /// id of the agent.
    #[arg(long, value_name = "NAME_OR_ID")]
    agent: Option<String>,

    /// List the agents registered with this tenant. Give the name or the id
    /// of the tenant. The API calls a tenant a generator.
    #[arg(long, value_name = "NAME_OR_ID")]
    generator: Option<String>,
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

    /// Tenant that owns the stack, by name or id. The API calls a tenant a
    /// generator. Required with an admin PAK. With a tenant PAK, you can omit
    /// it; if you give it, it must name the tenant of that PAK.
    #[arg(long, value_name = "NAME_OR_ID")]
    generator: Option<String>,
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
        Command::Agent(command) => fleet::agent(&client, command).await,
        Command::Stack(command) => fleet::stack(&client, command).await,
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
    let outcome = match &args.generator {
        Some(generator) => {
            client
                .apply_for_generator(generator, &args.stack, &args.filename, &args.target_label)
                .await
        }
        None => {
            client
                .apply(&args.stack, &args.filename, &args.target_label)
                .await
        }
    }
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

/// Find the ids of the agent and the tenant of a register or deregister
/// command. Each one is a name or an id.
async fn resolve_pair(client: &BrokkrClient, args: &RegisterArgs) -> Result<(Uuid, Uuid), String> {
    let agent = client
        .resolve_agent_id(&args.agent)
        .await
        .map_err(|e| e.to_string())?;
    let generator = client
        .resolve_generator_id(&args.generator)
        .await
        .map_err(|e| e.to_string())?;
    Ok((agent, generator))
}

/// Show a name-or-id argument with its id: `"edge-1" (<id>)` for a name,
/// only the id for an id.
fn shown(given: &str, id: Uuid) -> String {
    if Uuid::parse_str(given).is_ok() {
        id.to_string()
    } else {
        format!("\"{given}\" ({id})")
    }
}

async fn register(client: &BrokkrClient, args: RegisterArgs) -> Result<(), String> {
    let (agent, generator) = resolve_pair(client, &args).await?;
    let reg = client
        .register_agent(generator, Some(agent))
        .await
        .map_err(|e| e.to_string())?;
    println!(
        "registered agent {} with tenant {} (registration {})",
        shown(&args.agent, reg.agent_id),
        shown(&args.generator, reg.generator_id),
        reg.id
    );
    Ok(())
}

async fn deregister(client: &BrokkrClient, args: RegisterArgs) -> Result<(), String> {
    let (agent, generator) = resolve_pair(client, &args).await?;
    client
        .deregister_agent(generator, Some(agent))
        .await
        .map_err(|e| e.to_string())?;
    println!(
        "deregistered agent {} from tenant {}",
        shown(&args.agent, agent),
        shown(&args.generator, generator)
    );
    println!(
        "note: the agent's targets for this tenant were removed; it will prune \
         those resources on its next reconcile"
    );
    Ok(())
}

async fn registrations(client: &BrokkrClient, args: RegistrationsArgs) -> Result<(), String> {
    // ArgGroup guarantees exactly one of --agent / --generator is set.
    if let Some(given) = args.agent {
        let id = client
            .resolve_agent_id(&given)
            .await
            .map_err(|e| e.to_string())?;
        let agent = shown(&given, id);
        let regs = client
            .list_agent_registrations(id)
            .await
            .map_err(|e| e.to_string())?;
        if regs.is_empty() {
            println!("agent {agent} has no tenant registrations");
        } else {
            println!("agent {agent} is registered with {} tenant(s):", regs.len());
            for r in regs {
                println!("  tenant {}  (registered {})", r.generator_id, r.registered_at);
            }
        }
    } else if let Some(given) = args.generator {
        let id = client
            .resolve_generator_id(&given)
            .await
            .map_err(|e| e.to_string())?;
        let generator = shown(&given, id);
        let regs = client
            .list_generator_registered_agents(id)
            .await
            .map_err(|e| e.to_string())?;
        if regs.is_empty() {
            println!("tenant {generator} has no registered agents");
        } else {
            println!("tenant {generator} has {} registered agent(s):", regs.len());
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
    fn registration_flags_take_a_name_or_an_id() {
        for (command, tenant) in [("register", "acme"), ("deregister", NIL)] {
            let args = [
                "brokkr",
                command,
                "--agent",
                "edge-1",
                "--generator",
                tenant,
            ];
            let cli = Cli::try_parse_from(args).unwrap();
            let (Command::Register(a) | Command::Deregister(a)) = cli.command else {
                panic!("expected register or deregister");
            };
            assert_eq!(a.agent, "edge-1");
            assert_eq!(a.generator, tenant);
        }
        assert!(Cli::try_parse_from(["brokkr", "registrations", "--agent", "edge-1"]).is_ok());
        assert!(Cli::try_parse_from(["brokkr", "registrations", "--generator", "acme"]).is_ok());
    }

    #[test]
    fn an_id_is_shown_alone_and_a_name_with_its_id() {
        let id = Uuid::from_u128(7);
        assert_eq!(shown(&id.to_string(), id), id.to_string());
        assert_eq!(shown("edge-1", id), format!("\"edge-1\" ({id})"));
    }
}
