/*
 * Copyright (c) 2025 Dylan Storey
 * Licensed under the Elastic License 2.0.
 * See LICENSE file in the project root for full license text.
 */

use crate::api;
use crate::dal::DAL;
use crate::db::create_shared_connection_pool;
use crate::utils;
use crate::utils::pak;
use brokkr_models::models::agents::NewAgent;
use brokkr_models::models::generator::NewGenerator;
use brokkr_utils::config::Settings;
use brokkr_utils::logging::prelude::*;
use diesel::prelude::*;
use diesel::result::Error as DieselError;
use diesel::sql_query;
use diesel::sql_types::BigInt;
use diesel_migrations::{embed_migrations, EmbeddedMigrations, MigrationHarness};
use tokio::signal;
use uuid::Uuid;

// Assuming MIGRATIONS is defined in the bin.rs file, we need to import it
pub const MIGRATIONS: EmbeddedMigrations = embed_migrations!("../brokkr-models/migrations");

// Struct to hold the count result from SQL query
#[derive(QueryableByName, Debug)]
struct Count {
    #[diesel(sql_type = BigInt)]
    count: i64,
}

/// Function to start the Brokkr Broker server
///
/// This function initializes the database, runs migrations, checks for first-time setup,
/// configures API routes, and starts the server with graceful shutdown support.
pub async fn serve(config: &Settings) -> Result<(), Box<dyn std::error::Error>> {
    info!("Starting Brokkr Broker application");

    // Create database connection pool
    info!("Creating database connection pool");
    let connection_pool = create_shared_connection_pool(
        &config.database.url,
        "brokkr",
        5,
        config.database.schema.as_deref(),
    );
    info!("Database connection pool created successfully");

    // Set up schema if configured (for multi-tenant deployments)
    if let Some(ref schema) = config.database.schema {
        info!("Setting up schema: {}", schema);
        connection_pool
            .setup_schema(schema)
            .expect("Failed to set up schema");
        info!("Schema '{}' set up successfully", schema);
    }

    // Run pending migrations
    info!("Running pending database migrations");
    let mut conn = connection_pool.get().expect("Failed to get DB connection");

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
        utils::first_startup(&mut conn, config)?;
    } else {
        info!("Existing application detected. Proceeding with normal startup.");
    }

    // Initialize Data Access Layer
    info!("Initializing Data Access Layer");
    let dal = DAL::new(connection_pool.clone());

    // Initialize encryption key for webhooks
    info!("Initializing encryption key");
    utils::encryption::init_encryption_key(config.broker.webhook_encryption_key.as_deref())
        .expect("Failed to initialize encryption key");

    // Initialize event bus for webhook notifications
    info!("Initializing event bus");
    utils::event_bus::init_event_bus(dal.clone())
        .expect("Failed to initialize event bus");

    // Start background tasks
    info!("Starting background tasks");
    let cleanup_config = utils::background_tasks::DiagnosticCleanupConfig {
        interval_seconds: config
            .broker
            .diagnostic_cleanup_interval_seconds
            .unwrap_or(900),
        max_age_hours: config.broker.diagnostic_max_age_hours.unwrap_or(1),
    };
    utils::background_tasks::start_diagnostic_cleanup_task(dal.clone(), cleanup_config);

    // Start work order maintenance task (retry processing and stale claim detection)
    let work_order_config = utils::background_tasks::WorkOrderMaintenanceConfig::default();
    utils::background_tasks::start_work_order_maintenance_task(dal.clone(), work_order_config);

    // Start webhook delivery worker
    let webhook_delivery_config = utils::background_tasks::WebhookDeliveryConfig {
        interval_seconds: config
            .broker
            .webhook_delivery_interval_seconds
            .unwrap_or(5),
        batch_size: config.broker.webhook_delivery_batch_size.unwrap_or(50),
    };
    utils::background_tasks::start_webhook_delivery_task(dal.clone(), webhook_delivery_config);

    // Start webhook cleanup task
    let webhook_cleanup_config = utils::background_tasks::WebhookCleanupConfig {
        interval_seconds: 3600, // Every hour
        retention_days: config.broker.webhook_cleanup_retention_days.unwrap_or(7),
    };
    utils::background_tasks::start_webhook_cleanup_task(dal.clone(), webhook_cleanup_config);

    // Configure API routes
    info!("Configuring API routes");
    let app = api::configure_api_routes(dal.clone(), &config.cors).with_state(dal);

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
pub fn rotate_admin(config: &Settings) -> Result<(), Box<dyn std::error::Error>> {
    info!("Rotating admin key");

    // Create database connection
    let mut conn = PgConnection::establish(&config.database.url)
        .expect("Failed to establish database connection");

    // Run the first_startup function to generate a new admin key
    utils::upsert_admin(&mut conn, config)?;

    info!("Admin key rotated successfully");
    Ok(())
}

pub fn rotate_agent_key(config: &Settings, uuid: Uuid) -> Result<(), Box<dyn std::error::Error>> {
    info!("Rotating agent key");

    let pool = create_shared_connection_pool(
        &config.database.url,
        "brokkr",
        1,
        config.database.schema.as_deref(),
    );
    let dal = DAL::new(pool.clone());

    let agent = dal.agents().get(uuid)?.ok_or("Agent not found")?;
    let new_pak_hash = utils::pak::create_pak()?.1;
    dal.agents().update_pak_hash(agent.id, new_pak_hash)?;

    info!("Agent key rotated successfully for agent: {}", agent.name);
    Ok(())
}

pub fn rotate_generator_key(
    config: &Settings,
    uuid: Uuid,
) -> Result<(), Box<dyn std::error::Error>> {
    info!("Rotating generator key");

    let pool = create_shared_connection_pool(
        &config.database.url,
        "brokkr",
        1,
        config.database.schema.as_deref(),
    );
    let dal = DAL::new(pool.clone());

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

pub fn create_agent(
    config: &Settings,
    name: String,
    cluster_name: String,
) -> Result<(), Box<dyn std::error::Error>> {
    info!("Creating new agent: {}", name);

    let pool = create_shared_connection_pool(
        &config.database.url,
        "brokkr",
        1,
        config.database.schema.as_deref(),
    );
    let dal = DAL::new(pool.clone());

    let new_agent = NewAgent::new(name, cluster_name)
        .map_err(|e| format!("Failed to create NewAgent: {}", e))?;

    let (pak, pak_hash) = pak::create_pak()?;

    let agent = dal.agents().create(&new_agent)?;
    dal.agents().update_pak_hash(agent.id, pak_hash)?;

    info!("Successfully created agent with ID: {}", agent.id);
    println!("Agent created successfully:");
    println!("ID: {}", agent.id);
    println!("Name: {}", agent.name);
    println!("Cluster: {}", agent.cluster_name);
    println!("Initial PAK: {}", pak);

    Ok(())
}

pub fn create_generator(
    config: &Settings,
    name: String,
    description: Option<String>,
) -> Result<(), Box<dyn std::error::Error>> {
    info!("Creating new generator: {}", name);

    let pool = create_shared_connection_pool(
        &config.database.url,
        "brokkr",
        1,
        config.database.schema.as_deref(),
    );
    let dal = DAL::new(pool.clone());

    let new_generator = NewGenerator::new(name, description)
        .map_err(|e| format!("Failed to create NewGenerator: {}", e))?;

    let (pak, pak_hash) = pak::create_pak()?;

    let generator = dal.generators().create(&new_generator)?;
    dal.generators().update_pak_hash(generator.id, pak_hash)?;

    info!("Successfully created generator with ID: {}", generator.id);
    println!("Generator created successfully:");
    println!("ID: {}", generator.id);
    println!("Name: {}", generator.name);
    println!("Initial PAK: {}", pak);

    Ok(())
}
