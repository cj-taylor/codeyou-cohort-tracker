# Project Architecture

This document explains how the Cohort Tracker code is organized and how the different parts work together.

## Project Structure

```
cohort-tracker/
├── src/
│   ├── main.rs          # Entry point and CLI routing
│   ├── cli.rs           # Command-line interface definitions
│   ├── config.rs        # Configuration management
│   ├── db.rs            # Database operations
│   ├── openclass.rs     # OpenClass API types
│   └── sync.rs          # API client and sync logic
├── docs/                # Documentation (this folder)
├── Cargo.toml          # Dependencies and project metadata
└── README.md           # Main project documentation
```

## Module Breakdown

### `main.rs` - Application Entry Point

```rust
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
        // ... other commands
    }

    Ok(())
}
```

**What it does:** Routes CLI commands to handlers. Uses `anyhow::Result` for simple error handling.

### `cli.rs` - Command Line Interface

```rust
#[derive(Parser)]
pub struct Cli {
    #[arg(long)]
    pub config: Option<PathBuf>,
    
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    Init { email: String, password: String, class_id: String },
    Sync,
    Status,
}
```

**Purpose:** Defines CLI structure using `clap`. Handles argument parsing and validation.

### `config.rs` - Configuration Management

```rust
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
        Ok(())
    }
}
```

**What it does:** Loads/saves config from `~/.cohort-tracker.toml`. Uses `anyhow` for error handling instead of custom error types.

### `db.rs` - Database Operations

```rust
pub struct Database {
    conn: Connection,
}

impl Database {
    pub fn new() -> Result<Self, rusqlite::Error> { /* ... */ }
    pub fn create_tables(&self) -> Result<(), rusqlite::Error> { /* ... */ }
    pub fn insert_student(&self, student: &Student) -> Result<(), rusqlite::Error> { /* ... */ }
    pub fn get_stats(&self) -> Result<DatabaseStats, rusqlite::Error> { /* ... */ }
}
```

**Purpose:** All SQLite database interactions. Handles schema creation, data insertion, and queries.

### `openclass.rs` - API Type Definitions

```rust
#[derive(Deserialize)]
pub struct ProgressionResponse {
    pub metadata: Metadata,
    pub data: Vec<ProgressionData>,
}

#[derive(Deserialize)]
pub struct ProgressionData {
    #[serde(rename = "_id")]
    pub id: ObjectId,
    pub user: User,
    pub assignment: Assignment,
    pub grade: Option<f64>,
    // ... more fields
}
```

**Purpose:** Rust structs that match OpenClass API JSON responses. Uses `serde` for deserialization.

### `sync.rs` - API Client and Sync Logic

```rust
pub struct OpenClassClient {
    client: reqwest::Client,
    base_url: String,
    token: Option<String>,
}

impl OpenClassClient {
    pub async fn authenticate(&mut self, email: &str, password: &str) -> Result<(), SyncError> { /* ... */ }
    pub async fn fetch_progressions(&self, class_id: &str, page: u32) -> Result<ProgressionResponse, SyncError> { /* ... */ }
}

pub async fn sync_all_data(config: &Config) -> Result<(), SyncError> { /* ... */ }
```

**Purpose:** HTTP client for OpenClass API. Handles authentication, pagination, and data fetching.

## Data Flow

### 1. Initialization (`init` command)

```
User Input → CLI Parser → Config Creation → Save to ~/.cohort-tracker.toml
```

### 2. Sync Process (`sync` command)

```
Config Load → API Authentication → Paginated Data Fetch → Database Storage → Sync History
```

### 3. Status Check (`status` command)

```
Config Load → Database Query → Display Stats
```

## Key Design Decisions

### Async/Await for HTTP Calls

```rust
// Non-blocking HTTP requests
let response = self.client
    .get(&url)
    .bearer_auth(token)
    .send()
    .await?;
```

**Why:** Allows concurrent operations and better resource utilization during network I/O.

### Result Types for Error Handling

```rust
pub enum SyncError {
    AuthenticationFailed,
    NetworkError(reqwest::Error),
    DatabaseError(rusqlite::Error),
}
```

**Why:** Explicit error handling prevents crashes and provides clear error messages.

### SQLite for Local Storage

```rust
// Simple, file-based database
let conn = Connection::open("cohort-tracker.db")?;
```

**Why:** No server setup required, perfect for single-user CLI tool.

### Pagination Handling

```rust
let mut page = 0;
loop {
    let response = client.fetch_progressions(class_id, page).await?;
    process_data(&response.data);
    
    if !response.metadata.can_load_more {
        break;
    }
    page += 1;
}
```

**Why:** OpenClass API returns data in pages, we need to fetch all pages for complete data.

### Error Handling Strategy

### Using `anyhow` for Simple Error Handling

```rust
use anyhow::{anyhow, Result};

// Simple error propagation - no custom error types needed
pub fn from_file(path: &str) -> Result<Self> {
    let content = fs::read_to_string(path)
        .map_err(|e| anyhow!("Failed to read config file: {}", e))?;
    toml::from_str(&content).map_err(|e| anyhow!("Failed to parse config: {}", e))
}
```

### Custom Errors Only When Needed

```rust
// Only create custom errors when you need specific handling
#[derive(Debug)]
pub enum SyncError {
    AuthFailed,
    NetworkError(reqwest::Error),
    DatabaseError(rusqlite::Error),
}

// Use ? operator to propagate errors
pub async fn sync_all_data(config: &Config) -> Result<(), SyncError> {
    let mut client = OpenClassClient::new(&config.api_base);
    client.authenticate(&config.email, &config.password).await?;
    // ... rest of sync
}
```

## Testing Strategy

### Unit Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_config_serialization() {
        let config = Config { /* ... */ };
        let toml = toml::to_string(&config).unwrap();
        let parsed: Config = toml::from_str(&toml).unwrap();
        assert_eq!(config.email, parsed.email);
    }
}
```

### Integration Tests

```rust
#[tokio::test]
async fn test_full_sync_flow() {
    // Test with mock HTTP server
    let mock_server = MockServer::start().await;
    // ... setup mock responses
    let result = sync_all_data(&test_config).await;
    assert!(result.is_ok());
}
```

## Future Architecture (Phases 2 & 3)

### Phase 2: REST API

```
CLI Tool ← → SQLite ← → REST API Server ← → Web Dashboard
```

### Phase 3: Analytics

```
Raw Data → Analytics Engine → Metrics API → Visualization
```

## Performance Considerations

- **Rate Limiting:** 500ms delay between API calls
- **Batch Inserts:** Group database operations for efficiency  
- **Connection Pooling:** Reuse HTTP connections
- **Incremental Sync:** Only fetch new/updated data

## Next Steps

- Read [Why Rust?](./why-rust.md) to understand our technology choices
- Check [Database Design](./database.md) for schema details
- Review [Development Guide](./development.md) for contributing
