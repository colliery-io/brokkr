/*
 * Copyright (c) 2025 Dylan Storey
 * Licensed under the Elastic License 2.0.
 * See LICENSE file in the project root for full license text.
 */

//! Brokkr Broker CLI application
//!
//! This module provides the command-line interface for the Brokkr Broker application.
//! It includes functionality for serving the broker, rotating keys, and managing the application.

use brokkr_broker::cli::{parse_cli, Commands, CreateSubcommands, RotateSubcommands};

use brokkr_broker::utils;
use brokkr_utils::config::Settings;

use brokkr_broker::cli::commands;

/// Main function to run the Brokkr Broker application
///
/// This function initializes the application, parses command-line arguments,
/// and executes the appropriate command based on user input.
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = parse_cli();

    // Load configuration
    let config = Settings::new(None).expect("Failed to load configuration");

    // Initialize telemetry (includes tracing/logging setup)
    let telemetry_config = config.telemetry.for_broker();
    brokkr_utils::telemetry::init(&telemetry_config, &config.log.level, &config.log.format)
        .expect("Failed to initialize telemetry");

    // Create PAK controller
    let _ =
        utils::pak::create_pak_controller(Some(&config)).expect("Failed to create PAK controller");

    // Execute the appropriate command
    match cli.command {
        Commands::Serve => commands::serve(&config).await?,
        Commands::Create(create_commands) => match create_commands.command {
            CreateSubcommands::Agent { name, cluster_name } => {
                commands::create_agent(&config, name, cluster_name)?
            }
            CreateSubcommands::Generator { name, description } => {
                commands::create_generator(&config, name, description)?
            }
        },
        Commands::Rotate(rotate_commands) => match rotate_commands.command {
            RotateSubcommands::Admin => commands::rotate_admin(&config)?,
            RotateSubcommands::Agent { uuid } => commands::rotate_agent_key(&config, uuid)?,
            RotateSubcommands::Generator { uuid } => commands::rotate_generator_key(&config, uuid)?,
        },
    }

    // Shutdown telemetry on exit
    brokkr_utils::telemetry::shutdown();

    Ok(())
}
