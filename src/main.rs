mod api;
mod cli;
mod config;
mod db;
mod openclass;
mod sync;

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
        cli::Commands::Server { port } => {
            cli::handle_server(cli.config, port).await?;
        }
        cli::Commands::Import { students, mentors } => {
            cli::handle_import(students, mentors).await?;
        }
    }

    Ok(())
}
