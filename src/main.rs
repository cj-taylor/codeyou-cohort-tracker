mod config;
mod openclass;
mod db;
mod sync;
mod cli;

use anyhow::Result;
use clap::Parser;

#[tokio::main]
async fn main() -> Result<()> {
    dotenv::dotenv().ok();
    
    let cli = cli::Cli::parse();

    match cli.command {
        cli::Commands::Init {
            email,
            password,
            class_id,
            api_base,
        } => {
            cli::handle_init(email, password, class_id, api_base).await?;
        }
        cli::Commands::Sync => {
            cli::handle_sync(cli.config).await?;
        }
        cli::Commands::Status => {
            cli::handle_status(cli.config).await?;
        }
    }

    Ok(())
}
