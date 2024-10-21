use brokkr_agent::cli::commands;
use brokkr_agent::cli::{parse_cli, Commands};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = parse_cli();

    match cli.command {
        Commands::Register {
            admin_pak,
            agent_name,
            cluster_name,
        } => {
            commands::register(admin_pak, agent_name, cluster_name).await?;
        }
        Commands::Start => {
            commands::start().await?;
        }
    }

    Ok(())
}
