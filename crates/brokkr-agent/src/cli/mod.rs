/*
 * Copyright (c) 2025-2026 Dylan Storey
 * Licensed under the Elastic License 2.0.
 * See LICENSE file in the project root for full license text.
 */

/// Command-line interface module for the Brokkr agent.
pub mod commands;
use clap::{Parser, Subcommand};

/// CLI configuration structure.
#[derive(Parser)]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    /// Command to execute
    #[command(subcommand)]
    pub command: Commands,
}

/// Available CLI commands.
#[derive(Subcommand)]
pub enum Commands {
    /// Start the Brokkr agent
    Start {
        /// Comma-separated generator UUIDs to self-register with on startup.
        /// Overrides `BROKKR__AGENT__GENERATOR_IDS` / the config file; when
        /// omitted, falls back to config and then the legacy
        /// `BROKKR_GENERATOR_IDS` env var. Empty = system/fleet scope only.
        #[arg(long, value_name = "UUID,UUID,...")]
        generator_ids: Option<String>,
    },
}

/// Parses command-line arguments into the Cli structure.
///
/// # Returns
/// * `Cli` - Parsed CLI configuration
pub fn parse_cli() -> Cli {
    Cli::parse()
}
