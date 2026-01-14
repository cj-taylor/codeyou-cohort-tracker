# Cohort Tracker

Pulls student progress data from OpenClass.ai and stores it locally. Built in Rust.

## ⚠️ Breaking Changes - Multi-Class Support

**If upgrading from a previous version:**

The application now supports multiple classes! This requires a fresh setup:

1. **Backup your data** (optional):
   ```bash
   cp ~/.cohort-tracker.db ~/.cohort-tracker.db.backup
   cp ~/.cohort-tracker.toml ~/.cohort-tracker.toml.backup
   ```

2. **Remove old config and database**:
   ```bash
   rm ~/.cohort-tracker.toml
   rm ~/.cohort-tracker.db
   ```

3. **Re-initialize with new workflow**:
   ```bash
   cargo run -- init --email your@email.com --password yourpass
   # Follow the interactive prompts to select which classes to track
   ```

4. **Sync your data**:
   ```bash
   cargo run -- sync
   ```

---

## Getting started

You'll need Rust 1.80+ installed. If you don't have it, grab it from [rustup.rs](https://rustup.rs/).

```bash
cargo build --release

# Initialize with your OpenClass credentials
cargo run -- init --email you@example.com --password your-password

# The init command will:
# 1. Save your credentials
# 2. Fetch all classes you have access to
# 3. Let you select which classes to track (active vs inactive)

# Pull down all the data for active classes
cargo run -- sync

# See what you've got
cargo run -- status

# List all classes
cargo run -- list

# Activate/deactivate classes
cargo run -- activate data-analysis-pathway-module-1-aug-2
cargo run -- deactivate data-analysis-pathway-module-3-may25

# Start the dashboard
make serve
```

The `sync` command grabs everything from OpenClass and saves it to a local SQLite database. First sync takes a minute or two depending on how many students and classes you have.

`make serve` starts the API server and opens the dashboard in your browser.

## Docs

All the details live in [docs/](./docs/README.md):

- [Rust basics](./docs/rust-basics.md) if you're new to the language
- [Architecture](./docs/architecture.md) for how the code is organized
- [Database](./docs/database.md) for the schema
- [OpenClass API](./docs/openclass-api.md) for how we talk to their backend
- [Development](./docs/development.md) for contributing
- [Testing](./docs/testing.md) for the test suite
- [Roadmap](./docs/roadmap.md) for what could come next

## Status

- [x] CLI + sync
- [x] REST API
- [x] Analytics endpoints
