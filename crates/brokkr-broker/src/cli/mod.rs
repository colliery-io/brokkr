pub mod commands;

use clap::{Parser, Subcommand, Args};
use uuid::Uuid;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
#[command(propagate_version = true)]
/// Brokkr Broker CLI
///
/// This CLI provides commands to manage the Brokkr Broker, including serving the broker,
/// creating agents and generators, and rotating keys.
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Start the Brokkr Broker server
    Serve,

    /// Create new entities
    Create(CreateCommands),

    /// Rotate keys
    Rotate(RotateCommands),
}

#[derive(Args)]
pub struct CreateCommands {
    #[command(subcommand)]
    pub command: CreateSubcommands,
}

#[derive(Subcommand)]
pub enum CreateSubcommands {
    /// Create a new agent
    Agent {
        /// Name of the agent
        #[arg(long)]
        name: String,
        /// Name of the cluster the agent belongs to
        #[arg(long)]
        cluster_name: String,
    },

    /// Create a new generator
    Generator {
        /// Name of the generator
        #[arg(long)]
        name: String,
        /// Optional description of the generator
        #[arg(long)]
        description: Option<String>,
    },
}

#[derive(Args)]
pub struct RotateCommands {
    #[command(subcommand)]
    pub command: RotateSubcommands,
}

#[derive(Subcommand)]
pub enum RotateSubcommands {
    /// Rotate an agent key
    Agent {
        /// UUID of the agent
        #[arg(long)]
        uuid: Uuid,
    },

    /// Rotate a generator key
    Generator {
        /// UUID of the generator
        #[arg(long)]
        uuid: Uuid,
    },

    /// Rotate the admin key
    Admin,
}

pub fn parse_cli() -> Cli {
    Cli::parse()
}
