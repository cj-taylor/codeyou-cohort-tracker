use anyhow::Result;
use clap::{Parser, Subcommand};
use crate::config::Config;
use crate::db::Database;

#[derive(Parser)]
#[command(name = "cohort-tracker")]
#[command(about = "Sync and track student progress from OpenClass", long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,

    #[arg(short, long, global = true)]
    pub config: Option<String>,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Initialize configuration
    Init {
        /// Email for OpenClass login
        #[arg(short, long)]
        email: String,

        /// Password for OpenClass login
        #[arg(short, long)]
        password: String,

        /// Class ID to track
        #[arg(long)]
        class_id: String,

        /// API base URL (default: https://api.openclass.ai)
        #[arg(short, long, default_value = "https://api.openclass.ai")]
        api_base: String,
    },

    /// Sync data from OpenClass
    Sync,

    /// Show status and stats
    Status,
}

pub async fn handle_init(
    email: String,
    password: String,
    class_id: String,
    api_base: String,
) -> Result<()> {
    let config = Config {
        email,
        password,
        class_id,
        api_base,
    };

    let config_path = Config::default_path();
    config.save(config_path.to_str().unwrap())?;

    println!(
        "✓ Configuration saved to {}",
        config_path.display()
    );
    println!("  Email: {}", config.email);
    println!("  Class ID: {}", config.class_id);
    println!("\nRun 'cohort-tracker sync' to start syncing data");

    Ok(())
}

pub async fn handle_sync(config_path: Option<String>) -> Result<()> {
    let path = config_path
        .unwrap_or_else(|| Config::default_path().to_str().unwrap().to_string());

    let config = Config::from_file(&path)?;
    println!("Loading config from: {}", path);

    // Create database
    let db = Database::new("cohort-tracker.db")?;
    println!("✓ Database initialized: cohort-tracker.db");

    // Create and authenticate client
    let mut client = crate::sync::OpenClassClient::new(config.clone());
    println!("Authenticating with OpenClass...");
    client.authenticate().await?;
    println!("✓ Authenticated");

    // Sync all data
    println!("Starting sync...");
    let start = std::time::Instant::now();
    let stats = client.sync_all(&db).await?;
    let duration = start.elapsed();

    println!("\n=== Sync Complete ===");
    println!("Total records: {}", stats.total_records);
    println!("Pages fetched: {}", stats.pages_fetched);
    println!("Unique students: {}", stats.students_inserted);
    println!("Unique assignments: {}", stats.assignments_inserted);
    println!("Progressions: {}", stats.progressions_inserted);
    println!("Time elapsed: {:.2}s", duration.as_secs_f64());

    Ok(())
}

pub async fn handle_status(config_path: Option<String>) -> Result<()> {
    let path = config_path
        .unwrap_or_else(|| Config::default_path().to_str().unwrap().to_string());

    let config = Config::from_file(&path)?;
    println!("Config: {}", path);
    println!("Class ID: {}", config.class_id);
    println!("Email: {}", config.email);

    let db = Database::new("cohort-tracker.db")?;

    let student_count = db.get_student_count()?;
    let assignment_count = db.get_assignment_count()?;
    let progression_count = db.get_progression_count()?;
    let last_sync = db.get_last_sync()?;

    println!("\n=== Database Stats ===");
    println!("Students: {}", student_count);
    println!("Assignments: {}", assignment_count);
    println!("Progressions: {}", progression_count);
    println!(
        "Last sync: {}",
        last_sync.unwrap_or_else(|| "Never".to_string())
    );

    Ok(())
}
