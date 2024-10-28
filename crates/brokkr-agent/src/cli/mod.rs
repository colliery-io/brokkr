pub mod commands;
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    Register {
        #[arg(long)]
        #[arg(env = "BROKKR__AGENT__ADMIN_PAK")]
        admin_pak: String,
        #[arg(long)]
        #[arg(env = "BROKKR__AGENT__NAME")]
        agent_name: String,
        #[arg(long)]
        #[arg(env = "BROKKR__AGENT__CLUSTER_NAME")]
        cluster_name: String,
    },
    Start,
}

pub fn parse_cli() -> Cli {
    Cli::parse()
}
