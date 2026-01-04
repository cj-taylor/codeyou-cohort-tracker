# Cohort Tracker: Full Implementation Guide

## Overview

This guide walks you through building a complete Rust CLI application that syncs student data from OpenClass.ai and provides a REST API for querying cohort progress.

## Prerequisites

- Rust 1.80+ (<https://rustup.rs/>)
- Git
- Basic understanding of async Rust and HTTP clients

## Getting Started

### Step 1: Initialize Project

```bash
cargo new cohort-tracker
cd cohort-tracker
```

### Step 2: Update Cargo.toml

Use these dependencies (all compatible with Rust 1.80+):

```toml
[package]
name = "cohort-tracker"
version = "0.1.0"
edition = "2021"

[dependencies]
# Async runtime
tokio = { version = "1", features = ["macros", "rt-multi-thread"] }

# HTTP client
reqwest = { version = "0.12", features = ["json"] }

# Serialization
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"

# Database
rusqlite = { version = "0.32", features = ["bundled"] }

# Error handling
anyhow = "1.0"
thiserror = "1.0"

# Configuration
dotenv = "0.15"
toml = "0.8"

# CLI
clap = { version = "4.4", features = ["derive"] }

# Web framework (for Phase 2)
axum = "0.7"
tower = "0.4"
tower-http = { version = "0.5", features = ["trace", "cors"] }

# Utilities
tracing = "0.1"
tracing-subscriber = "0.3"
```

---

## Phase 1: CLI + Sync

### Module Structure

```
src/
├── main.rs              # Entry point
├── config.rs            # Configuration management
├── openclass.rs         # OpenClass API types
├── db.rs                # Database operations
├── sync.rs              # Sync logic
└── cli.rs               # CLI commands
```

### 1. config.rs - Configuration Management

```rust
use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Config {
    pub email: String,
    pub password: String,
    pub class_id: String,
    pub api_base: String,
}

impl Config {
    pub fn from_file(path: &str) -> Result<Self> {
        let content = fs::read_to_string(path)
            .map_err(|e| anyhow!("Failed to read config file: {}", e))?;
        toml::from_str(&content).map_err(|e| anyhow!("Failed to parse config: {}", e))
    }

    pub fn save(&self, path: &str) -> Result<()> {
        let content = toml::to_string_pretty(self)?;
        fs::write(path, content)?;
        println!("✓ Config saved to {}", path);
        Ok(())
    }

    pub fn default_path() -> PathBuf {
        if let Some(home) = std::env::var_os("HOME") {
            PathBuf::from(home).join(".cohort-tracker.toml")
        } else {
            PathBuf::from(".cohort-tracker.toml")
        }
    }
}
```

### 2. openclass.rs - API Types

```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: String,
    pub first_name: String,
    pub last_name: String,
    pub email: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Assignment {
    pub id: String,
    pub name: String,
    #[serde(rename = "type")]
    pub assignment_type: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Progression {
    #[serde(rename = "_id")]
    pub id: serde_json::Value,
    pub user: User,
    pub assignment: Assignment,
    #[serde(default)]
    pub grade: Option<f64>,
    pub started_assignment_at: String,
    pub completed_assignment_at: String,
    pub reviewed_at: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ProgressionResponse {
    pub metadata: Metadata,
    pub data: Vec<Progression>,
}

#[derive(Debug, Deserialize)]
pub struct Metadata {
    pub total: i32,
    pub page: i32,
    pub results_per_page: i32,
    pub can_load_more: bool,
}

#[derive(Debug, Serialize)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}
```

### 3. db.rs - Database Operations

```rust
use anyhow::Result;
use rusqlite::{params, Connection, OptionalExtension};

pub struct Database {
    conn: Connection,
}

impl Database {
    pub fn new(path: &str) -> Result<Self> {
        let conn = Connection::open(path)?;

        // Initialize schema
        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS students (
                id TEXT PRIMARY KEY,
                first_name TEXT NOT NULL,
                last_name TEXT NOT NULL,
                email TEXT UNIQUE NOT NULL
            );

            CREATE TABLE IF NOT EXISTS assignments (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                type TEXT NOT NULL
            );

            CREATE TABLE IF NOT EXISTS progressions (
                id TEXT PRIMARY KEY,
                student_id TEXT NOT NULL,
                assignment_id TEXT NOT NULL,
                grade REAL,
                started_at TEXT NOT NULL,
                completed_at TEXT NOT NULL,
                reviewed_at TEXT,
                synced_at TEXT NOT NULL,
                FOREIGN KEY (student_id) REFERENCES students(id),
                FOREIGN KEY (assignment_id) REFERENCES assignments(id)
            );

            CREATE TABLE IF NOT EXISTS sync_history (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                synced_at TEXT NOT NULL,
                class_id TEXT NOT NULL,
                page INTEGER NOT NULL,
                records_processed INTEGER NOT NULL
            );",
        )?;

        Ok(Self { conn })
    }

    pub fn insert_student(&self, id: &str, first_name: &str, last_name: &str, email: &str) -> Result<()> {
        self.conn.execute(
            "INSERT OR IGNORE INTO students (id, first_name, last_name, email) VALUES (?1, ?2, ?3, ?4)",
            params![id, first_name, last_name, email],
        )?;
        Ok(())
    }

    pub fn insert_assignment(&self, id: &str, name: &str, assignment_type: &str) -> Result<()> {
        self.conn.execute(
            "INSERT OR IGNORE INTO assignments (id, name, type) VALUES (?1, ?2, ?3)",
            params![id, name, assignment_type],
        )?;
        Ok(())
    }

    pub fn insert_progression(
        &self,
        id: &str,
        student_id: &str,
        assignment_id: &str,
        grade: Option<f64>,
        started_at: &str,
        completed_at: &str,
        reviewed_at: Option<&str>,
    ) -> Result<()> {
        let now = chrono::Local::now().to_rfc3339();
        self.conn.execute(
            "INSERT OR REPLACE INTO progressions
            (id, student_id, assignment_id, grade, started_at, completed_at, reviewed_at, synced_at)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            params![id, student_id, assignment_id, grade, started_at, completed_at, reviewed_at, now],
        )?;
        Ok(())
    }

    pub fn record_sync(&self, class_id: &str, page: i32, records_processed: i32) -> Result<()> {
        let now = chrono::Local::now().to_rfc3339();
        self.conn.execute(
            "INSERT INTO sync_history (synced_at, class_id, page, records_processed) VALUES (?1, ?2, ?3, ?4)",
            params![now, class_id, page, records_processed],
        )?;
        Ok(())
    }

    pub fn get_student_count(&self) -> Result<i64> {
        let count = self.conn.query_row(
            "SELECT COUNT(*) FROM students",
            [],
            |row| row.get(0),
        )?;
        Ok(count)
    }

    pub fn get_assignment_count(&self) -> Result<i64> {
        let count = self.conn.query_row(
            "SELECT COUNT(*) FROM assignments",
            [],
            |row| row.get(0),
        )?;
        Ok(count)
    }

    pub fn get_progression_count(&self) -> Result<i64> {
        let count = self.conn.query_row(
            "SELECT COUNT(*) FROM progressions",
            [],
            |row| row.get(0),
        )?;
        Ok(count)
    }

    pub fn get_last_sync(&self) -> Result<Option<String>> {
        let result = self.conn.query_row(
            "SELECT synced_at FROM sync_history ORDER BY synced_at DESC LIMIT 1",
            [],
            |row| row.get(0),
        ).optional()?;
        Ok(result)
    }
}
```

### 4. sync.rs - OpenClass API Client & Sync Logic

```rust
use crate::config::Config;
use crate::db::Database;
use crate::openclass::{LoginRequest, ProgressionResponse};
use anyhow::{anyhow, Result};
use std::time::Duration;

pub struct OpenClassClient {
    config: Config,
    token: Option<String>,
}

impl OpenClassClient {
    pub fn new(config: Config) -> Self {
        Self {
            config,
            token: None,
        }
    }

    pub async fn authenticate(&mut self) -> Result<()> {
        let client = reqwest::Client::new();
        let req = LoginRequest {
            email: self.config.email.clone(),
            password: self.config.password.clone(),
        };

        let response = client
            .post(&format!("{}/v1/auth/login", self.config.api_base))
            .json(&req)
            .timeout(Duration::from_secs(10))
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(anyhow!("Authentication failed: {}", response.status()));
        }

        let json: serde_json::Value = response.json().await?;

        // Try to extract token from response
        if let Some(token) = json.get("token").and_then(|v| v.as_str()) {
            self.token = Some(token.to_string());
            return Ok(());
        }

        if let Some(token) = json.get("auth_token").and_then(|v| v.as_str()) {
            self.token = Some(token.to_string());
            return Ok(());
        }

        Err(anyhow!("No token in authentication response"))
    }

    pub async fn fetch_progressions(&self, page: i32) -> Result<ProgressionResponse> {
        let token = self.token.as_ref()
            .ok_or_else(|| anyhow!("Not authenticated"))?;

        let client = reqwest::Client::new();
        let url = format!(
            "{}/v1/classes/{}/progressions?return_count=30&page={}&sort_by_completed_at=-1",
            self.config.api_base, self.config.class_id, page
        );

        let response = client
            .get(&url)
            .header("bearer", token)
            .header("content-type", "application/json")
            .timeout(Duration::from_secs(30))
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(anyhow!("Failed to fetch page {}: {}", page, response.status()));
        }

        Ok(response.json().await?)
    }

    pub async fn sync_all(&self, db: &Database) -> Result<SyncStats> {
        let mut stats = SyncStats::default();
        let mut page = 0;

        loop {
            println!("Fetching page {}...", page);
            let response = self.fetch_progressions(page).await?;
            let can_load_more = response.metadata.can_load_more;

            for prog in response.data {
                // Insert student
                db.insert_student(
                    &prog.user.id,
                    &prog.user.first_name,
                    &prog.user.last_name,
                    &prog.user.email,
                )?;

                // Insert assignment
                db.insert_assignment(
                    &prog.assignment.id,
                    &prog.assignment.name,
                    &prog.assignment.assignment_type,
                )?;

                // Extract progression ID
                let prog_id = match &prog.id {
                    serde_json::Value::Object(obj) => {
                        obj.get("$oid")
                            .and_then(|v| v.as_str())
                            .unwrap_or("unknown")
                            .to_string()
                    }
                    serde_json::Value::String(s) => s.clone(),
                    _ => "unknown".to_string(),
                };

                // Insert progression
                db.insert_progression(
                    &prog_id,
                    &prog.user.id,
                    &prog.assignment.id,
                    prog.grade,
                    &prog.started_assignment_at,
                    &prog.completed_assignment_at,
                    prog.reviewed_at.as_deref(),
                )?;

                stats.total_records += 1;
            }

            db.record_sync(&self.config.class_id, page, response.data.len() as i32)?;
            stats.pages_fetched += 1;
            page += 1;

            if !can_load_more {
                break;
            }

            // Rate limiting
            tokio::time::sleep(Duration::from_millis(500)).await;
        }

        Ok(stats)
    }
}

#[derive(Debug, Default)]
pub struct SyncStats {
    pub total_records: i32,
    pub pages_fetched: i32,
}
```

### 5. cli.rs - Command Handler

```rust
use anyhow::Result;
use clap::{Parser, Subcommand};
use crate::config::Config;
use crate::db::Database;

#[derive(Parser)]
#[command(name = "cohort-tracker", about = "Sync and track student progress from OpenClass")]
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
        #[arg(short, long)]
        class_id: String,
        #[arg(short, long, default_value = "https://api.openclass.ai")]
        api_base: String,
    },
    Sync,
    Status,
}

pub async fn handle_init(email: String, password: String, class_id: String, api_base: String) -> Result<()> {
    let config = Config { email, password, class_id, api_base };
    let path = Config::default_path();
    config.save(path.to_str().unwrap())?;
    println!("Run 'cohort-tracker sync' to start syncing");
    Ok(())
}

pub async fn handle_sync(config_path: Option<String>) -> Result<()> {
    let path = config_path
        .unwrap_or_else(|| Config::default_path().to_str().unwrap().to_string());

    let config = Config::from_file(&path)?;
    let db = Database::new("cohort-tracker.db")?;

    let mut client = crate::sync::OpenClassClient::new(config);
    println!("Authenticating...");
    client.authenticate().await?;
    println!("✓ Authenticated");

    let start = std::time::Instant::now();
    let stats = client.sync_all(&db).await?;
    let duration = start.elapsed();

    println!("\n=== Sync Complete ===");
    println!("Total records: {}", stats.total_records);
    println!("Pages: {}", stats.pages_fetched);
    println!("Time: {:.2}s", duration.as_secs_f64());

    Ok(())
}

pub async fn handle_status(config_path: Option<String>) -> Result<()> {
    let path = config_path
        .unwrap_or_else(|| Config::default_path().to_str().unwrap().to_string());

    let config = Config::from_file(&path)?;
    let db = Database::new("cohort-tracker.db")?;

    println!("Class ID: {}", config.class_id);
    println!("Students: {}", db.get_student_count()?);
    println!("Assignments: {}", db.get_assignment_count()?);
    println!("Progressions: {}", db.get_progression_count()?);
    println!("Last sync: {}", db.get_last_sync()?.unwrap_or_else(|| "Never".to_string()));

    Ok(())
}
```

### 6. main.rs - Entry Point

```rust
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
        cli::Commands::Init { email, password, class_id, api_base } => {
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
```

---

## Running Phase 1

### Build

```bash
cargo build --release
```

### Initialize Config

```bash
cargo run -- init \
  --email your@email.com \
  --password yourpassword \
  --class-id 68e594f320442cbbe62a18dc
```

### Start Sync

```bash
cargo run -- sync
```

### Check Status

```bash
cargo run -- status
```

---

## What You'll Learn

**Phase 1 teaches:**

- HTTP clients and authentication
- JSON serialization/deserialization
- Database operations with Rust
- Error handling
- CLI with clap
- Async/await patterns
- Pagination handling
- Rate limiting

---

## Next Steps: Phase 2 (REST API)

Once Phase 1 works, Phase 2 adds:

- Axum web framework
- GET/POST endpoints
- JSON responses
- Error handling
- Testing

Would recommend implementing Phase 1 completely, testing it with real OpenClass data, then moving to Phase 2.

---

## Troubleshooting

### Authentication Fails

- Double-check email/password
- Verify class ID is correct
- Check that OpenClass API is accessible

### Database Errors

- Delete `cohort-tracker.db` if schema changes
- Ensure disk space available

### Compilation Issues

- Run `rustup update` to get latest Rust
- Check Rust version with `rustc --version`

---

## Notes for Person 1 & Person 2

When teaching them, this is a natural progression:

1. **Person 2 builds Phase 1** (CLI, HTTP client, auth)
2. **You review and fix issues**
3. **Person 2 leads Phase 2** (Axum API)
4. **Person 1 leads Phase 3** (Analytics queries)

Each phase is contained and can be built/tested independently.
