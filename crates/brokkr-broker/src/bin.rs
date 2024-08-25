use axum::Server;
use std::net::SocketAddr;
use tokio;

use brokkr_broker::api;
use brokkr_broker::dal::DAL;

use brokkr_broker::db::create_shared_connection_pool;


use brokkr_utils::config::Settings;
use brokkr_utils::logging::prelude::*;

#[tokio::main]
async fn main() {
    // Load configuration
    let config = Settings::new(None).expect("Failed to load configuration");

    // Initialize logger
    brokkr_utils::logging::init(&config.log.level).expect("Failed to initialize logger");

    info!("Starting application");
    let connection_pool = create_shared_connection_pool(&config.database.url, "brokkr", 5);
    let dal = DAL::new(connection_pool.pool.clone());    

    

    // Configure API routes
    let app = api::configure_api_routes(dal);

    // Set up the server address
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));

    info!("Starting server on {}", addr);

    // Start the server
    match Server::bind(&addr).serve(app.into_make_service()).await {
        Ok(_) => info!("Server shut down gracefully"),
        Err(e) => error!("Server error: {}", e),
    }
}