use anyhow::Result;
use clap::Parser;
use cohort_tracker::cli;
use cohort_tracker::config::Config;

#[tokio::main]
async fn main() -> Result<()> {
    dotenv::dotenv().ok();

    let cli = cli::Cli::parse();

    // Check for updates in background (non-blocking)
    let config_path = cli.config.clone().unwrap_or_else(|| {
        Config::default_path()
            .to_str()
            .unwrap_or(".cohort-tracker.toml")
            .to_string()
    });

    if let Ok(config) = Config::from_file(&config_path) {
        if config.check_for_updates {
            tokio::spawn(async {
                cohort_tracker::update::check_and_notify().await;
            });
        }
    }

    match cli.command {
        cli::Commands::Init {
            email,
            password,
            api_base,
        } => {
            cli::handle_init(email, password, api_base).await?;
        }
        cli::Commands::List { all } => {
            cli::handle_list(all).await?;
        }
        cli::Commands::Activate { friendly_ids } => {
            cli::handle_activate(friendly_ids).await?;
        }
        cli::Commands::Deactivate { friendly_ids } => {
            cli::handle_deactivate(friendly_ids).await?;
        }
        cli::Commands::Sync { class, full } => {
            cli::handle_sync(cli.config, class, full).await?;
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
        cli::Commands::Update => {
            cohort_tracker::update::perform_update().await?;
        }
    }

    Ok(())
}
