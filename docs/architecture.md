# Project Architecture

This document explains how the Cohort Tracker code is organized and how the different parts work together. If you're new to the project, start here to get your bearings.

## Table of Contents

- [Project Structure](#project-structure)
- [Why This Structure?](#why-this-structure)
- [Core Concepts](#core-concepts)
- [Data Flow](#data-flow)
- [Key Design Patterns](#key-design-patterns)
- [Module Organization Rules](#module-organization-rules)
- [Where to Add New Code](#where-to-add-new-code)
- [Testing Strategy](#testing-strategy)
- [Common Patterns](#common-patterns)

## Project Structure

```
cohort-tracker/
├── src/
│   ├── main.rs          # Entry point - routes CLI commands
│   ├── lib.rs           # Library exports
│   ├── models.rs        # Domain models (Class, Student, Assignment, etc.)
│   ├── config.rs        # Configuration management
│   ├── cli.rs           # CLI command definitions and handlers
│   ├── api.rs           # REST API server and endpoints
│   ├── db/              # Database layer
│   │   ├── mod.rs       # Database struct and schema
│   │   ├── queries.rs   # CRUD operations
│   │   └── analytics.rs # Analytics queries
│   ├── lms/             # LMS provider abstraction
│   │   ├── mod.rs       # LmsProvider trait
│   │   └── openclass/   # OpenClass implementation
│   │       ├── mod.rs   # Provider implementation
│   │       ├── auth.rs  # Authentication
│   │       ├── fetch.rs # API calls
│   │       └── types.rs # OpenClass-specific types
│   └── sync/            # Sync engine
│       ├── mod.rs       # Module exports
│       ├── types.rs     # SyncStats
│       └── engine.rs    # Provider-agnostic sync logic
├── static/              # Dashboard HTML/CSS/JS
├── docs/                # Documentation
├── tests/               # Integration tests
├── Cargo.toml          # Dependencies
└── README.md           # Quick start guide
```

## Why This Structure?

We started with everything in single files (`db.rs`, `sync.rs`), but as the project grew, we split them into modules:

- **Easier to navigate**: Find what you need faster
- **Clear responsibilities**: Each file has one job
- **Better for learning**: Focus on one piece at a time
- **Ready for growth**: Easy to add new providers or features

## Core Concepts

### 1. Domain Models (`models.rs`)

These are the core data structures used throughout the app:

```rust
pub struct Class {
    pub id: String,
    pub name: String,
    pub friendly_id: String,
    pub is_active: bool,
    pub synced_at: Option<String>,
}

pub struct Student {
    pub id: String,
    pub class_id: String,
    pub first_name: String,
    pub last_name: String,
    pub email: String,
    // ...
}
```

**Why at the top level?** These models are used everywhere - database, API, sync logic. Keeping them in one place makes them easy to import.

### 2. Database Layer (`db/`)

Split into three files for clarity:

**`db/mod.rs`** - Database struct and schema setup
```rust
pub struct Database {
    pub(crate) conn: Connection,
}

impl Database {
    pub fn new(path: &str) -> Result<Self> {
        // Creates tables, runs migrations
    }
}
```

**`db/queries.rs`** - Basic CRUD operations
- Insert/get students, assignments, progressions
- Class management
- Count queries

**`db/analytics.rs`** - Complex analytics
- Student health metrics
- Blocker assignments
- Progress over time
- Activity tracking

**Why split?** A 1000+ line file is hard to navigate. Now you know exactly where to look.

### 3. LMS Provider System (`lms/`)

This is the heart of our multi-provider architecture.

**The Problem:** We started with OpenClass hardcoded everywhere. But what if we want to support TopHat or Canvas later?

**The Solution:** Define a trait that all providers must implement:

```rust
#[async_trait]
pub trait LmsProvider: Send + Sync {
    async fn authenticate(&mut self) -> Result<()>;
    async fn fetch_classes(&self) -> Result<Vec<Class>>;
    async fn fetch_progressions(&self, class_id: &str, page: i32) 
        -> Result<ProgressionBatch>;
    // ...
}
```

Now OpenClass is just one implementation:

```
lms/
├── mod.rs              # LmsProvider trait + common types
└── openclass/          # OpenClass-specific code
    ├── mod.rs          # Implements LmsProvider
    ├── auth.rs         # OpenClass authentication
    ├── fetch.rs        # OpenClass API calls
    └── types.rs        # OpenClass response types
```

**Adding TopHat?** Just create `lms/tophat/` and implement the trait. The sync engine doesn't need to change.

### 4. Sync Engine (`sync/`)

The sync engine is now provider-agnostic:

```rust
pub struct SyncEngine {
    provider: Box<dyn LmsProvider>,  // Works with any provider!
}

impl SyncEngine {
    pub async fn sync_class(&mut self, class_id: &str, db: &Database, full: bool) 
        -> Result<SyncStats> {
        // Fetch from provider
        let batch = self.provider.fetch_progressions(class_id, page).await?;
        
        // Save to database
        for progression in batch.progressions {
            db.insert_student(/* ... */)?;
            db.insert_progression(/* ... */)?;
        }
    }
}
```

**Key insight:** The sync logic doesn't care if data comes from OpenClass, TopHat, or anywhere else. It just uses the `LmsProvider` trait.

### 5. CLI (`cli.rs`)

Command definitions and handlers in one file. Each command has a `handle_*` function:

```rust
pub async fn handle_sync(config_path: Option<String>, class_id: Option<String>, full: bool) 
    -> Result<()> {
    // 1. Load config
    // 2. Create provider
    // 3. Authenticate
    // 4. Run sync engine
}
```

**Why not split?** Only 8 commands, each handler is self-contained. Splitting would add complexity without benefit.

### 6. API Server (`api.rs`)

REST API for the dashboard. Simple handlers that query the database:

```rust
async fn list_students(
    State(state): State<Arc<AppState>>,
    Path(class_id): Path<String>,
) -> Result<Json<Vec<Student>>, ApiError> {
    let db = state.db.lock().await;
    let students = db.get_students_by_class(&class_id)?;
    Ok(Json(students))
}
```

**Why not split?** 18 handlers, but they're all simple (10-20 lines). Current structure is clear.

## Data Flow

Data moves through the system like this:

```
1. CLI Command
   ↓
2. Create LMS Provider (OpenClass)
   ↓
3. Authenticate with Provider
   ↓
4. Sync Engine fetches data via Provider trait
   ↓
5. Sync Engine saves to Database
   ↓
6. API Server reads from Database
   ↓
7. Dashboard displays data
```

## Key Design Patterns

### Trait Objects for Flexibility

```rust
let provider: Box<dyn LmsProvider> = Box::new(OpenClassProvider::new(config));
let engine = SyncEngine::new(provider);
```

This lets us swap providers at runtime without changing the sync engine.

### Result Type for Error Handling

```rust
pub fn get_students(&self, class_id: &str) -> Result<Vec<Student>> {
    // If anything fails, return Err
    // Caller handles with ? operator
}
```

No exceptions, no silent failures. Every error is explicit.

### Async/Await for I/O

```rust
let response = self.client.get(url).await?;
let data = response.json().await?;
```

Non-blocking I/O means we can fetch from multiple classes concurrently.

## Module Organization Rules

We follow these guidelines:

1. **Top-level files** - Domain-wide concerns (models, config)
2. **Directories** - When a file grows past ~400 lines or has multiple concerns
3. **Re-exports** - Make imports clean (`use crate::models::*`)
4. **Provider-specific code** - Always in `lms/<provider>/`

## Where to Add New Code

**New database query?** → `db/queries.rs` (CRUD) or `db/analytics.rs` (complex)

**New CLI command?** → Add to `cli.rs` (Commands enum + handler function)

**New API endpoint?** → Add handler to `api.rs` + route in `create_router()`

**New LMS provider?** → Create `lms/<provider>/` directory, implement `LmsProvider` trait

**New domain model?** → Add to `models.rs`

## Testing Strategy

- **Unit tests** - In same file as code (`#[cfg(test)]` modules)
- **Integration tests** - In `tests/` directory
- **API tests** - Use `wiremock` to mock HTTP calls

See [testing.md](./testing.md) for details.

## Common Patterns

### The `?` Operator

```rust
let config = Config::from_file(&path)?;  // Returns early if error
```

This is Rust's way of saying "if this fails, return the error to the caller."

### Pattern Matching

```rust
match response.status() {
    StatusCode::OK => { /* handle success */ }
    StatusCode::UNAUTHORIZED => { /* handle auth error */ }
    _ => { /* handle other errors */ }
}
```

Rust makes you handle all cases. No surprises.

### Option Handling

```rust
let section = assignment_sections.get(&id).map(|s| s.as_str());
```

`Option<T>` means "might be None." The `?` operator works here too.

## Learning Path

If you're new to the codebase:

1. **Start with `models.rs`** - Understand the core data structures
2. **Read `db/queries.rs`** - See how data is stored/retrieved
3. **Look at `cli.rs`** - See how commands work end-to-end
4. **Explore `lms/openclass/`** - Understand the provider pattern
5. **Check `sync/engine.rs`** - See how it all comes together

## Next Steps

- [Database Schema](./database.md) - Understand the data model
- [Development Guide](./development.md) - Set up your environment
- [Rust Basics](./rust-basics.md) - Learn Rust concepts used here
- [OpenClass API](./openclass-api.md) - Understand the data source

## Questions?

The code has comments where things get tricky. If something's unclear, that's a documentation bug - let us know!
