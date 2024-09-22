//! Brokkr Broker CLI application
//!
//! This module provides the command-line interface for the Brokkr Broker application.
//! It includes functionality for serving the broker, rotating keys, and managing the application.

use brokkr_broker::api;
use brokkr_broker::dal::DAL;
use brokkr_broker::db::create_shared_connection_pool;
use brokkr_broker::utils;
use brokkr_utils::config::Settings;
use brokkr_utils::logging::prelude::*;
use clap::{Parser, Subcommand};
use diesel::deserialize::QueryableByName;
use diesel::prelude::*;
use diesel::result::Error as DieselError;
use diesel::sql_query;
use diesel::sql_types::BigInt;
use diesel_migrations::{embed_migrations, EmbeddedMigrations, MigrationHarness};
use tokio::signal;
use uuid::Uuid;

/// Embedded migrations for the database
pub const MIGRATIONS: EmbeddedMigrations = embed_migrations!("../brokkr-models/migrations");

/// Struct to hold the count result from SQL query
#[derive(QueryableByName, Debug)]
struct Count {
    #[diesel(sql_type = BigInt)]
    count: i64,
}

/// Command-line interface structure
#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

/// Subcommands for the CLI
#[derive(Subcommand)]
enum Commands {
    /// Start the Brokkr Broker server
    Serve,
    /// Rotate the admin key
    RotateAdmin,
    /// Rotate an agent key
    RotateAgentKey {
        #[arg(long)]
        uuid: Uuid,
    },
    /// Rotate a generator key
    RotateGeneratorKey {
        #[arg(long)]
        uuid: Uuid,
    },
}

/// Main function to run the Brokkr Broker application
///
/// This function initializes the application, parses command-line arguments,
/// and executes the appropriate command based on user input.
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    // Load configuration
    let config = Settings::new(None).expect("Failed to load configuration");

    // Initialize logger
    brokkr_utils::logging::init(&config.log.level).expect("Failed to initialize logger");

    // Create PAK controller
    let _ =
        utils::pak::create_pak_controller(Some(&config)).expect("Failed to create PAK controller");

    // Execute the appropriate command
    match cli.command {
        Commands::Serve => serve(&config).await?,
        Commands::RotateAdmin => rotate_admin(&config)?,
        Commands::RotateAgentKey { uuid } => rotate_agent_key(&config, uuid)?,
        Commands::RotateGeneratorKey { uuid } => rotate_generator_key(&config, uuid)?,
    }
    Ok(())
}

/// Function to start the Brokkr Broker server
///
/// This function initializes the database, runs migrations, checks for first-time setup,
/// configures API routes, and starts the server with graceful shutdown support.
async fn serve(config: &Settings) -> Result<(), Box<dyn std::error::Error>> {
    info!("Starting Brokkr Broker application");

    // Create database connection pool
    info!("Creating database connection pool");
    let connection_pool = create_shared_connection_pool(&config.database.url, "brokkr", 5);
    info!("Database connection pool created successfully");

    // Run pending migrations
    info!("Running pending database migrations");
    let mut conn = connection_pool
        .pool
        .get()
        .expect("Failed to get DB connection");
    conn.run_pending_migrations(MIGRATIONS)
        .expect("Failed to run migrations");
    info!("Database migrations completed successfully");

    // Check if this is the first time running the application
    let is_first_run = conn
        .transaction(|conn| {
            let result: Count =
                sql_query("SELECT COUNT(*) as count FROM app_initialization").get_result(conn)?;
            if result.count == 0 {
                // If it's the first run, insert a record into app_initialization
                sql_query("INSERT INTO app_initialization DEFAULT VALUES").execute(conn)?;
                Ok::<bool, DieselError>(true)
            } else {
                Ok::<bool, DieselError>(false)
            }
        })
        .expect("Failed to check initialization status");

    // Perform first-time setup if necessary
    if is_first_run {
        info!("First time application startup detected. Creating admin role...");
        utils::first_startup(&mut conn)?;
    } else {
        info!("Existing application detected. Proceeding with normal startup.");
    }

    // Initialize Data Access Layer
    info!("Initializing Data Access Layer");
    let dal = DAL::new(connection_pool.pool.clone());

    // Configure API routes
    info!("Configuring API routes");
    let app = api::configure_api_routes(dal.clone()).with_state(dal);

    // Set up the server address
    let addr = "0.0.0.0:3000";
    info!("Starting server on {}", addr);
    let listener = tokio::net::TcpListener::bind(addr).await?;

    // Set up shutdown signal handler
    let (shutdown_tx, shutdown_rx) = tokio::sync::oneshot::channel::<()>();
    tokio::spawn(async move {
        signal::ctrl_c().await.expect("Failed to listen for ctrl+c");
        shutdown_tx.send(()).ok();
    });

    // Start the server with graceful shutdown
    info!("Brokkr Broker is now running");
    axum::serve(listener, app)
        .with_graceful_shutdown(utils::shutdown(shutdown_rx))
        .await?;

    Ok(())
}

/// Function to rotate the admin key
///
/// This function generates a new admin key and updates it in the database.
fn rotate_admin(config: &Settings) -> Result<(), Box<dyn std::error::Error>> {
    info!("Rotating admin key");

    // Create database connection
    let mut conn = PgConnection::establish(&config.database.url)
        .expect("Failed to establish database connection");

    // Run the first_startup function to generate a new admin key
    utils::upsert_admin(&mut conn)?;

    info!("Admin key rotated successfully");
    Ok(())
}

fn rotate_agent_key(config: &Settings, uuid: Uuid) -> Result<(), Box<dyn std::error::Error>> {
    info!("Rotating agent key");

    let pool = create_shared_connection_pool(&config.database.url, "brokkr", 1);
    let dal = DAL::new(pool.pool.clone());

    let agent = dal.agents().get(uuid)?.ok_or("Agent not found")?;
    let new_pak_hash = utils::pak::create_pak()?.1;
    dal.agents().update_pak_hash(agent.id, new_pak_hash)?;

    info!("Agent key rotated successfully for agent: {}", agent.name);
    Ok(())
}

fn rotate_generator_key(config: &Settings, uuid: Uuid) -> Result<(), Box<dyn std::error::Error>> {
    info!("Rotating generator key");

    let pool = create_shared_connection_pool(&config.database.url, "brokkr", 1);
    let dal = DAL::new(pool.pool.clone());

    let generator = dal.generators().get(uuid)?.ok_or("Generator not found")?;

    let new_pak_hash = utils::pak::create_pak()?.1;
    dal.generators()
        .update_pak_hash(generator.id, new_pak_hash)?;

    info!(
        "Generator key rotated successfully for generator: {}",
        generator.name
    );
    Ok(())
}
