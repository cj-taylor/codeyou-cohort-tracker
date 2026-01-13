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
    Init {
        #[arg(short, long)]
        email: String,
        #[arg(short, long)]
        password: String,
        #[arg(long)]
        class_id: String,
        /// API base URL (default: https://api.openclass.ai)
        #[arg(short, long, default_value = "https://api.openclass.ai")]
        api_base: String,
    },

    Sync,
    Status,

    Server {
        #[arg(short, long, default_value = "3000")]
        port: u16,
    },

    /// Import student nights and mentor data from CSV files
    Import {
        /// Path to students CSV (columns: First Name, Last Name, Region, Night)
        #[arg(long)]
        students: Option<String>,
        /// Path to mentors CSV (columns: Mentor Name, Night)
        #[arg(long)]
        mentors: Option<String>,
    },
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

pub async fn handle_server(config_path: Option<String>, port: u16) -> Result<()> {
    let path = config_path
        .unwrap_or_else(|| Config::default_path().to_str().unwrap().to_string());

    let config = Config::from_file(&path)?;
    println!("Config: {}", path);
    println!("Class ID: {}", config.class_id);

    crate::api::start_server(config, "cohort-tracker.db", port).await
}

pub async fn handle_import(students_path: Option<String>, mentors_path: Option<String>) -> Result<()> {
    use std::fs::File;
    use std::io::{BufRead, BufReader};

    let db = Database::new("cohort-tracker.db")?;
    println!("Database: cohort-tracker.db");

    // Import students CSV
    if let Some(path) = students_path {
        println!("\nImporting students from: {}", path);
        let file = File::open(&path)?;
        let reader = BufReader::new(file);
        let mut updated = 0;
        let mut not_found = 0;
        let mut skipped = 0;

        for (i, line) in reader.lines().enumerate() {
            let line = line?;
            if i == 0 {
                // Skip header row
                continue;
            }

            // TODO: use a proper CSV parser instead of split(',')
            let fields: Vec<&str> = line.split(',').collect();
            if fields.len() >= 4 {
                let first_name = fields[0].trim();
                let last_name = fields[1].trim();
                let region = fields[2].trim();
                let night = fields[3].trim();

                if first_name.is_empty() || last_name.is_empty() {
                    skipped += 1;
                    continue;
                }

                match db.update_student_night(first_name, last_name, region, night)? {
                    true => {
                        updated += 1;
                        println!("  Updated: {} {} -> {} ({})", first_name, last_name, night, region);
                    }
                    false => {
                        not_found += 1;
                        println!("  Not found: {} {}", first_name, last_name);
                    }
                }
            } else {
                skipped += 1;
            }
        }

        println!("\nStudent import complete:");
        println!("  Updated: {}", updated);
        println!("  Not found: {}", not_found);
        println!("  Skipped: {}", skipped);
    }

    // Import mentors CSV
    if let Some(path) = mentors_path {
        println!("\nImporting mentors from: {}", path);

        // Clear existing mentors
        db.clear_mentors()?;
        println!("  Cleared existing mentors");

        let file = File::open(&path)?;
        let reader = BufReader::new(file);
        let mut imported = 0;

        for (i, line) in reader.lines().enumerate() {
            let line = line?;
            if i == 0 {
                // Skip header row
                continue;
            }

            let fields: Vec<&str> = line.split(',').collect();
            if fields.len() >= 2 {
                let name = fields[0].trim();
                let night = fields[1].trim();

                if name.is_empty() || night.is_empty() {
                    continue;
                }

                db.import_mentor(name, night)?;
                imported += 1;
                println!("  Imported: {} ({})", name, night);
            }
        }

        println!("\nMentor import complete:");
        println!("  Imported: {}", imported);
    }

    // Show summary
    println!("\n=== Current Night/Region Summary ===");
    let summaries = db.get_night_summary()?;
    if summaries.is_empty() {
        println!("No night data assigned yet.");
    } else {
        for summary in summaries {
            println!(
                "\n{}: {} students, {} completions ({:.1}% avg)",
                summary.night,
                summary.student_count,
                summary.total_completions,
                summary.avg_completion_pct * 100.0
            );
            if !summary.mentors.is_empty() {
                println!("  Mentors: {}", summary.mentors.join(", "));
            }
        }
    }

    Ok(())
}
