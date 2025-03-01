/*
 * Copyright (c) 2025 Dylan Storey
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
    Start,
}

/// Parses command-line arguments into the Cli structure.
///
/// # Returns
/// * `Cli` - Parsed CLI configuration
pub fn parse_cli() -> Cli {
    Cli::parse()
}
