use anyhow::{Result, anyhow};
use clap::{Parser, Subcommand};
use crate::config::Config;
use crate::db::Database;
use std::io::{self, Write};

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
        /// API base URL (default: https://api.openclass.ai)
        #[arg(short, long, default_value = "https://api.openclass.ai")]
        api_base: String,
    },

    List {
        #[arg(short, long)]
        all: bool,
    },

    Activate {
        friendly_ids: Vec<String>,
    },

    Deactivate {
        friendly_ids: Vec<String>,
    },

    Sync {
        #[arg(long)]
        class: Option<String>,
        /// Full sync: fetch all data. Incremental sync: stop after 3 pages of duplicates
        #[arg(long)]
        full: bool,
    },
    
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
    api_base: String,
) -> Result<()> {
    // Save credentials
    let config = Config {
        email: email.clone(),
        password: password.clone(),
        api_base: api_base.clone(),
    };

    let config_path = Config::default_path();
    config.save(config_path.to_str().unwrap())?;

    println!("✓ Configuration saved to {}", config_path.display());

    // Authenticate and fetch classes
    println!("\nAuthenticating...");
    let mut provider = crate::lms::openclass::OpenClassProvider::new(config);
    provider.authenticate().await?;
    
    println!("Fetching available classes...");
    let classes = provider.fetch_classes().await?;

    if classes.is_empty() {
        println!("No classes found for this account.");
        return Ok(());
    }

    // Display classes
    println!("\nFound {} classes:", classes.len());
    for class in &classes {
        println!("  - {}", class.friendly_id);
    }

    // Get user selection
    println!("\nEnter friendly IDs to activate (comma-separated, or 'all'):");
    println!("Example: data-analysis-pathway-module-1-aug-2,data-analysis-pathway-module-2-aug-2");
    print!("> ");
    io::stdout().flush()?;

    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    let input = input.trim();

    // Parse selection
    let active_friendly_ids: Vec<String> = if input == "all" {
        classes.iter().map(|c| c.friendly_id.clone()).collect()
    } else {
        input.split(',').map(|s| s.trim().to_string()).collect()
    };

    // Store classes in database
    let db_path = crate::config::home_dir()
        .unwrap_or_else(|| std::path::PathBuf::from("."))
        .join(".cohort-tracker.db");
    let db = Database::new(db_path.to_str().unwrap())?;

    println!();
    for class in &classes {
        let is_active = active_friendly_ids.contains(&class.friendly_id);
        let mut class_to_insert = class.clone();
        class_to_insert.is_active = is_active;
        db.insert_class(&class_to_insert)?;
        
        println!("  {} - {} ({})", 
            if is_active { "✓" } else { "○" }, 
            class.name,
            class.friendly_id
        );
    }

    println!("\n✓ Setup complete! Run 'cargo run -- sync' to fetch data.");

    Ok(())
}

pub async fn handle_sync(config_path: Option<String>, class_friendly_id: Option<String>, full: bool) -> Result<()> {
    let path = config_path
        .unwrap_or_else(|| Config::default_path().to_str().unwrap().to_string());

    let config = Config::from_file(&path)?;
    println!("Loading config from: {}", path);

    // Create database
    let db_path = crate::config::home_dir()
        .unwrap_or_else(|| std::path::PathBuf::from("."))
        .join(".cohort-tracker.db");
    let db = Database::new(db_path.to_str().unwrap())?;
    println!("✓ Database initialized: {}", db_path.display());

    // Create provider and sync engine
    let mut provider = Box::new(crate::lms::openclass::OpenClassProvider::new(config.clone()));
    println!("Authenticating with OpenClass...");
    provider.authenticate().await?;
    println!("✓ Authenticated");

    let mut engine = crate::sync::SyncEngine::new(provider);

    // Sync specific class or all active classes
    let mode = if full { "full" } else { "incremental" };
    println!("Starting {} sync...", mode);
    let start = std::time::Instant::now();
    
    let stats = if let Some(friendly_id) = class_friendly_id {
        // Sync specific class
        let class = db.get_class_by_friendly_id(&friendly_id)?;
        println!("Syncing class: {}", class.name);
        engine.sync_class(&class.id, &db, full).await?
    } else {
        // Sync all active classes
        engine.sync_all(&db, full).await?
    };
    
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
    println!("Email: {}", config.email);

    let db_path = crate::config::home_dir()
        .unwrap_or_else(|| std::path::PathBuf::from("."))
        .join(".cohort-tracker.db");
    let db = Database::new(db_path.to_str().unwrap())?;

    // Show per-class stats
    let classes = db.get_active_classes()?;
    
    if classes.is_empty() {
        println!("\nNo active classes. Run 'init' or 'activate' first.");
        return Ok(());
    }

    println!("\n=== Active Classes ===");
    for class in classes {
        let student_count = db.get_student_count_by_class(&class.id)?;
        let assignment_count = db.get_assignment_count_by_class(&class.id)?;
        let progression_count = db.get_progression_count_by_class(&class.id)?;

        println!("\n{} ({})", class.name, class.friendly_id);
        println!("  Students: {}", student_count);
        println!("  Assignments: {}", assignment_count);
        println!("  Progressions: {}", progression_count);
        println!("  Last sync: {}", class.synced_at.as_deref().unwrap_or("never"));
    }

    Ok(())
}

pub async fn handle_server(config_path: Option<String>, port: u16) -> Result<()> {
    let _path = config_path
        .unwrap_or_else(|| Config::default_path().to_str().unwrap().to_string());

    let db_path = crate::config::home_dir()
        .unwrap_or_else(|| std::path::PathBuf::from("."))
        .join(".cohort-tracker.db");
    
    crate::api::start_server(db_path.to_str().unwrap(), port).await
}

pub async fn handle_import(students_path: Option<String>, mentors_path: Option<String>) -> Result<()> {
    use std::fs::File;
    use std::io::{BufRead, BufReader};

    let db_path = crate::config::home_dir()
        .unwrap_or_else(|| std::path::PathBuf::from("."))
        .join(".cohort-tracker.db");
    let db = Database::new(db_path.to_str().unwrap())?;
    println!("Database: {}", db_path.display());

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
    println!("\n=== Import Complete ===");
    println!("Night/region data has been imported. Use 'status' command to view per-class summaries.");

    Ok(())
}

pub async fn handle_list(all: bool) -> Result<()> {
    let db_path = crate::config::home_dir()
        .unwrap_or_else(|| std::path::PathBuf::from("."))
        .join(".cohort-tracker.db");
    let db = Database::new(db_path.to_str().unwrap())?;

    let classes = if all {
        db.get_classes()?
    } else {
        db.get_active_classes()?
    };

    if classes.is_empty() {
        println!("No classes found. Run 'init' first.");
        return Ok(());
    }

    println!("{} classes:", if all { "All" } else { "Active" });
    for class in classes {
        let status = if class.is_active { "✓" } else { "○" };
        let synced = class.synced_at.as_deref().unwrap_or("never");
        println!("  {} {} ({}) - last synced: {}", 
            status, 
            class.name, 
            class.friendly_id,
            synced
        );
    }

    Ok(())
}

pub async fn handle_activate(friendly_ids: Vec<String>) -> Result<()> {
    let db_path = crate::config::home_dir()
        .unwrap_or_else(|| std::path::PathBuf::from("."))
        .join(".cohort-tracker.db");
    let db = Database::new(db_path.to_str().unwrap())?;

    for friendly_id in friendly_ids {
        match db.get_class_by_friendly_id(&friendly_id) {
            Ok(class) => {
                db.set_class_active(&class.id, true)?;
                println!("✓ Activated: {} ({})", class.name, class.friendly_id);
            }
            Err(_) => {
                eprintln!("✗ Error: Class '{}' not found", friendly_id);
                eprintln!("\nAvailable classes:");
                let classes = db.get_classes()?;
                for class in classes {
                    let status = if class.is_active { "active" } else { "inactive" };
                    println!("  - {} ({})", class.friendly_id, status);
                }
                return Err(anyhow!("Invalid class friendly_id"));
            }
        }
    }

    Ok(())
}

pub async fn handle_deactivate(friendly_ids: Vec<String>) -> Result<()> {
    let db_path = crate::config::home_dir()
        .unwrap_or_else(|| std::path::PathBuf::from("."))
        .join(".cohort-tracker.db");
    let db = Database::new(db_path.to_str().unwrap())?;

    for friendly_id in friendly_ids {
        match db.get_class_by_friendly_id(&friendly_id) {
            Ok(class) => {
                db.set_class_active(&class.id, false)?;
                println!("○ Deactivated: {} ({})", class.name, class.friendly_id);
            }
            Err(_) => {
                eprintln!("✗ Error: Class '{}' not found", friendly_id);
                eprintln!("\nAvailable classes:");
                let classes = db.get_classes()?;
                for class in classes {
                    let status = if class.is_active { "active" } else { "inactive" };
                    println!("  - {} ({})", class.friendly_id, status);
                }
                return Err(anyhow!("Invalid class friendly_id"));
            }
        }
    }

    Ok(())
}
