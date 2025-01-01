use brokkr_agent::cli::commands;
use brokkr_agent::cli::{parse_cli, Commands};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = parse_cli();

    match cli.command {
        Commands::Start => {
            commands::start().await?;
        }
    }

    Ok(())
}
