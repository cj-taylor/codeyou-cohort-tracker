# Development Guide

This guide helps you contribute to and extend the Cohort Tracker project.

## Getting Started

### Prerequisites

```bash
# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source ~/.cargo/env

# Verify installation
rustc --version  # Should be 1.80+
cargo --version
```

### Clone and Build

```bash
git clone <repo-url>
cd cohort-tracker
cargo build
cargo test
```

## Development Workflow

### 1. Make Changes

```bash
# Create a feature branch
git checkout -b feature/new-analytics

# Make your changes
# Edit src/db.rs, src/cli.rs, etc.
```

### 2. Test Your Changes

```bash
# Run all tests
cargo test

# Test specific module
cargo test db::tests

# Run with output
cargo test -- --nocapture
```

### 3. Check Code Quality

```bash
# Format code
cargo fmt

# Check for common issues
cargo clippy

# Check without building
cargo check
```

### 4. Test CLI Commands

```bash
# Test in development mode
cargo run -- status
cargo run -- sync

# Test release build
cargo build --release
./target/release/cohort-tracker status
```

## Code Organization

### Adding New CLI Commands

1. **Update `cli.rs`:**

```rust
#[derive(Subcommand)]
pub enum Commands {
    Init { email: String, password: String, class_id: String, api_base: Option<String> },
    Sync,
    Status,
    Analytics { student_id: Option<String> },
}
```

2. **Handle in `main.rs`:**

```rust
match cli.command {
    cli::Commands::Analytics { student_id } => {
        cli::handle_analytics(student_id).await?;
    },
    // ... other commands
}
```

3. **Add the handler in `cli.rs`:**

```rust
pub async fn handle_analytics(student_id: Option<String>) -> Result<()> {
    let db = db::Database::new()?;
    
    match student_id {
        Some(id) => show_student_analytics(&db, &id)?,
        None => show_cohort_analytics(&db)?,
    }
    
    Ok(())
}
```

### Adding Database Operations

1. **Add to `db.rs`:**

```rust
impl Database {
    pub fn get_student_progress(&self, student_id: &str) -> Result<Vec<Progression>, rusqlite::Error> {
        let mut stmt = self.conn.prepare(
            "SELECT * FROM progressions WHERE student_id = ?1 ORDER BY completed_at"
        )?;
        
        let progression_iter = stmt.query_map([student_id], |row| {
            Ok(Progression {
                id: row.get(0)?,
                student_id: row.get(1)?,
                assignment_id: row.get(2)?,
                grade: row.get(3)?,
                // ... other fields
            })
        })?;
        
        let mut progressions = Vec::new();
        for progression in progression_iter {
            progressions.push(progression?);
        }
        
        Ok(progressions)
    }
}
```

2. **Add tests:**

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_get_student_progress() {
        let db = Database::new_in_memory().unwrap();
        db.create_tables().unwrap();
        
        // Insert test data
        let student = Student { /* ... */ };
        db.insert_student(&student).unwrap();
        
        // Test the query
        let progress = db.get_student_progress(&student.id).unwrap();
        assert_eq!(progress.len(), 0);  // No progressions yet
    }
}
```

### Adding API Endpoints (Phase 2)

1. **Create handler in `src/api/handlers.rs`:**

```rust
use axum::{Json, extract::Path};

pub async fn get_student_progress(
    Path(student_id): Path<String>
) -> Result<Json<Vec<Progression>>, ApiError> {
    let db = Database::new().map_err(ApiError::DatabaseError)?;
    let progressions = db.get_student_progress(&student_id)
        .map_err(ApiError::DatabaseError)?;
    
    Ok(Json(progressions))
}
```

2. **Add route in `src/api/mod.rs`:**

```rust
use axum::{Router, routing::get};

pub fn create_router() -> Router {
    Router::new()
        .route("/health", get(health_check))
        .route("/students/:id/progress", get(get_student_progress))
}
```

## Testing Strategy

### Unit Tests

Test individual functions:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_config_loading() {
        let config = Config {
            email: "test@example.com".to_string(),
            password: "password".to_string(),
            class_id: "valid_id".to_string(),
            api_base: "https://api.openclass.ai".to_string(),
        };
        
        // Test serialization roundtrip
        let toml_str = toml::to_string(&config).unwrap();
        let parsed: Config = toml::from_str(&toml_str).unwrap();
        assert_eq!(config.email, parsed.email);
    }
}
```

### Integration Tests

Test complete workflows:

```rust
// tests/integration_test.rs
#[tokio::test]
async fn test_sync_workflow() {
    // Setup test database
    let db = Database::new_in_memory().unwrap();
    
    // Mock API responses with wiremock
    let mock_server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/v1/auth/login"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "token": "test-token"
        })))
        .mount(&mock_server)
        .await;
    
    // Run sync and verify
    let config = test_config_with_mock_url(&mock_server.uri());
    let result = sync::sync_all_data(&config).await;
    assert!(result.is_ok());
}
```

### Manual Testing

```bash
# Test with real OpenClass data (use test account)
cargo run -- init --email test@example.com --password testpass --class-id testid
cargo run -- sync
cargo run -- status

# Test error cases
cargo run -- sync  # Without config
cargo run -- init --email invalid  # Invalid email
```

## Error Handling Patterns

### Custom Error Types

```rust
// Only when you need specific error handling
#[derive(Debug)]
pub enum SyncError {
    AuthFailed,
    NetworkError(reqwest::Error),
    DatabaseError(rusqlite::Error),
}

impl std::fmt::Display for SyncError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            SyncError::AuthFailed => write!(f, "Authentication failed"),
            SyncError::NetworkError(e) => write!(f, "Network error: {}", e),
            SyncError::DatabaseError(e) => write!(f, "Database error: {}", e),
        }
    }
}

impl std::error::Error for SyncError {}

// Auto-convert from other error types
impl From<reqwest::Error> for SyncError {
    fn from(error: reqwest::Error) -> Self {
        SyncError::NetworkError(error)
    }
}
```

## Performance Considerations

### Database Optimization

```rust
// Use transactions for bulk operations
pub fn insert_many_progressions(&self, progressions: &[Progression]) -> Result<(), rusqlite::Error> {
    let tx = self.conn.transaction()?;
    
    {
        let mut stmt = tx.prepare(
            "INSERT OR REPLACE INTO progressions (id, student_id, assignment_id, grade, started_at, completed_at, reviewed_at, synced_at) 
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)"
        )?;
        
        for progression in progressions {
            stmt.execute(params![
                progression.id,
                progression.student_id,
                progression.assignment_id,
                progression.grade,
                progression.started_at,
                progression.completed_at,
                progression.reviewed_at,
                progression.synced_at,
            ])?;
        }
    }
    
    tx.commit()?;
    Ok(())
}
```

### HTTP Client Optimization

```rust
// Reuse HTTP client and connections
pub struct OpenClassClient {
    client: reqwest::Client,
    base_url: String,
    token: Option<String>,
}

impl OpenClassClient {
    pub fn new(base_url: &str) -> Self {
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(30))
            .connection_verbose(true)
            .build()
            .expect("Failed to create HTTP client");
            
        Self {
            client,
            base_url: base_url.to_string(),
            token: None,
        }
    }
}
```

## Debugging Tips

### Enable Logging

```rust
// Add to Cargo.toml
[dependencies]
env_logger = "0.10"
log = "0.4"

// In main.rs
fn main() {
    env_logger::init();
    // ... rest of main
}

// In your code
use log::{info, warn, error};

pub async fn sync_progressions(&self, class_id: &str) -> Result<Vec<Progression>, SyncError> {
    info!("Starting sync for class {}", class_id);
    
    let mut all_progressions = Vec::new();
    let mut page = 0;
    
    loop {
        let response = self.fetch_progressions(class_id, page).await?;
        info!("Fetched {} progressions from page {}", response.data.len(), page);
        
        all_progressions.extend(response.data);
        
        if !response.metadata.can_load_more {
            break;
        }
        page += 1;
    }
    
    info!("Sync complete: {} total progressions", all_progressions.len());
    Ok(all_progressions)
}
```

### Run with Logging

```bash
# Enable all logs
RUST_LOG=debug cargo run -- sync

# Enable only info and above
RUST_LOG=info cargo run -- sync
```

### Database Debugging

```rust
// Add helper method to inspect database
impl Database {
    pub fn debug_tables(&self) -> Result<(), rusqlite::Error> {
        let tables = ["students", "assignments", "progressions", "sync_history"];
        
        for table in &tables {
            let count: i64 = self.conn.query_row(
                &format!("SELECT COUNT(*) FROM {}", table),
                [],
                |row| row.get(0)
            )?;
            println!("{}: {} records", table, count);
        }
        
        Ok(())
    }
}
```

## Common Issues and Solutions

### Compilation Errors

**Error:** `cannot borrow as mutable`
```rust
// Problem
let mut client = OpenClassClient::new();
client.authenticate().await?;
client.fetch_data().await?;  // Error if authenticate takes &mut self

// Solution: structure borrows properly
let mut client = OpenClassClient::new();
client.authenticate().await?;
drop(client);  // Or restructure to avoid multiple mutable borrows
```

**Error:** `future cannot be sent between threads safely`
```rust
// Problem: non-Send types in async functions
async fn bad_function() {
    let rc = Rc::new(5);  // Rc is not Send
    some_async_call().await;
    println!("{}", rc);
}

// Solution: use Send types or restructure
async fn good_function() {
    let arc = Arc::new(5);  // Arc is Send
    some_async_call().await;
    println!("{}", arc);
}
```

### Runtime Issues

**Issue:** Database locked errors
```rust
// Solution: ensure connections are properly closed
impl Database {
    pub fn close(self) -> Result<(), rusqlite::Error> {
        self.conn.close().map_err(|(_, err)| err)
    }
}
```

**Issue:** HTTP timeout errors
```rust
// Solution: configure appropriate timeouts
let client = reqwest::Client::builder()
    .timeout(Duration::from_secs(60))  // Longer timeout
    .build()?;
```

## Contributing Guidelines

### Code Style

- Use `cargo fmt` for formatting
- Follow Rust naming conventions (snake_case for functions, PascalCase for types)
- Add documentation comments for public functions
- Keep functions focused and small

### Commit Messages

```
feat: add student analytics command
fix: handle null grades in progression data
docs: update API documentation
test: add integration tests for sync workflow
```

### Pull Request Process

1. Create feature branch from main
2. Make changes with tests
3. Run `cargo test` and `cargo clippy`
4. Update documentation if needed
5. Submit PR with clear description

## Next Steps

- Read [Database Design](./database.md) for schema details
- Check [OpenClass API](./openclass-api.md) for integration details
- Review the main [README.md](../README.md) for project overview
