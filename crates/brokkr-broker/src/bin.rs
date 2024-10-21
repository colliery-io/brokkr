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

    // Initialize logger
    brokkr_utils::logging::init(&config.log.level).expect("Failed to initialize logger");

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
    Ok(())
}
