# Cohort Tracker

Pulls student progress data from OpenClass.ai and stores it locally in SQLite. Built in Rust.

## Why This Exists

As a mentor, it's hard to know where students are in their progression without manually clicking through OpenClass pages and loading paginated results. This tool solves that by:

- Automatically syncing all student progress data via the OpenClass API
- Storing it locally so you can query and analyze it however you want
- Providing a dashboard to quickly see who's stuck, who's active, and when they're working
- Helping you prepare for office hours by knowing what assignments students are on

Built for CodeYou mentors who want better visibility into their cohorts.

## Quick Start

New to the project? Check out the [Getting Started Guide](./docs/getting-started.md).

Otherwise:

```bash
# Install Rust from https://rustup.rs if you haven't already

# Build it
cargo build --release

# Set up your credentials and select classes to track
cargo run -- init --email you@example.com --password your-password

# Fetch the data
cargo run -- sync

# Start the dashboard
make serve
```

Open http://localhost:3000 to view the dashboard.

## What You Get

### Core Features
- **Dashboard** with visual overview of student progress and completion rates
- **Analytics** to identify struggling students and blocker assignments
- **Night Filtering** to compare performance across different cohort nights
- **Local SQLite database** you can query directly
- **REST API** for building your own tools
- **Fast incremental sync** (only fetches new data)

### Analytics & Diagnostics
- **Assignment Type Breakdown** - Compare performance on lessons vs quizzes
- **Grade Distribution** - Visualize grade spread to identify struggling subgroups
- **Velocity Tracking** - Monitor student pace (assignments/week) to catch slowdowns early
- **Engagement Gap Detection** - Early warning for students who go silent (7-14 days inactive)
- **Assignment Difficulty Ranking** - Composite scoring to prioritize curriculum fixes
- **Students at Risk** - Automatic risk scoring based on completion and grades
- **Activity Monitoring** - Track last activity and days inactive per student
- **Performance by Night** - Compare mentor groups and cohort nights
- **Progress Over Time** - Weekly completion trends with drill-down
- **Section Progress** - See which course sections students are stuck on

## Daily Usage

```bash
# Sync new data (run this daily or before checking the dashboard)
cargo run -- sync

# Check status
cargo run -- status

# Start the dashboard
make serve

# Update to the latest version
cargo run -- update
```

## Common Commands

```bash
# Sync specific class
cargo run -- sync --class data-analysis-pathway-module-2-aug-2

# Force full refresh (fetches everything)
cargo run -- sync --full

# List all classes
cargo run -- list

# Activate/deactivate classes
cargo run -- activate data-analysis-pathway-module-1-aug-2
cargo run -- deactivate old-class-name

# Update to latest version
cargo run -- update
```

## Documentation

**Start here:**
- [Getting Started](./docs/getting-started.md) - Detailed setup and usage guide
- [Dashboard Guide](./docs/dashboard-guide.md) - Complete walkthrough of the dashboard features
- [Architecture](./docs/architecture.md) - How the code is organized
- [Rust Basics](./docs/rust-basics.md) - Rust concepts used in this project

**Dive deeper:**
- [Database Schema](./docs/database.md) - What data we store and how to query it
- [Development Guide](./docs/development.md) - Contributing and making changes
- [OpenClass API](./docs/openclass-api.md) - Understanding the data source
- [Testing](./docs/testing.md) - How to write and run tests
- [Why Rust?](./docs/why-rust.md) - Our technology decisions
- [Roadmap](./docs/roadmap.md) - Future ideas

**Full index:** [docs/README.md](./docs/README.md)

## How It Works

1. Sync - Fetches student progress from OpenClass API
2. Store - Saves to local SQLite database (`~/.cohort-tracker.db`)
3. Serve - REST API provides data to the dashboard
4. Update - Automatically checks for new versions (once per day)

The sync is incremental by default - it only fetches new data. First sync takes a couple minutes, subsequent syncs are faster.

## Configuration

The configuration file is stored at `~/.cohort-tracker.toml`. You can disable automatic update checks by setting:

```toml
check_for_updates = false
```
4. Visualize - Dashboard shows progress, blockers, and risk levels

The sync is incremental by default - it only fetches new data. First sync takes a couple minutes, subsequent syncs are faster.

## Project Structure

```
src/
├── models.rs        # Data structures (Class, Student, Assignment)
├── cli.rs           # Command-line interface
├── api.rs           # REST API server
├── db/              # Database layer (queries + analytics)
├── lms/             # LMS provider abstraction
│   └── openclass/   # OpenClass implementation
└── sync/            # Sync engine
```

See [Architecture](./docs/architecture.md) for the full picture.

## Requirements

- Rust 1.80+ ([install from rustup.rs](https://rustup.rs/))
- OpenClass instructor/mentor account
- Access to at least one class

## Features

- Multi-class support
- Incremental sync (fast updates)
- Full sync option (complete refresh)
- Student progress tracking
- Risk level identification
- Blocker assignment detection
- Activity monitoring
- REST API
- Interactive dashboard
- Local SQLite database
- Real-time sync progress with Server-Sent Events

## Contributing

Check out the [Development Guide](./docs/development.md) to get started. The codebase is organized to be approachable for Rust learners.

## License

MIT

## Questions?

Check the [docs/](./docs/README.md) folder or open an issue.

Built with Rust for educators tracking student progress.

