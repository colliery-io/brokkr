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
}

/// Main function to run the Brokkr Broker application
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
    }

    Ok(())
}

/// Function to start the Brokkr Broker server
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
    let app = api::configure_api_routes();

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
