# Cohort Tracker

Pulls student progress data from OpenClass.ai and stores it locally. Built in Rust.

Track your students' progress, identify who's struggling, and see where they get stuck - all from your own local database.

---

## Quick Start

**New here?** Check out the [Getting Started Guide](./docs/getting-started.md) for a detailed walkthrough.

**Already know what you're doing?** Here's the speed run:

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

Open http://localhost:3000 and you're off to the races.

## What You Get

- **📊 Dashboard** - Visual overview of student progress, completion rates, and risk levels
- **🔍 Analytics** - Identify struggling students and blocker assignments
- **🗄️ Local Database** - All your data in SQLite, query it however you want
- **🚀 REST API** - Build your own tools on top of the data
- **⚡ Fast Sync** - Incremental updates, only fetches what's new

## Daily Usage

```bash
# Sync new data (run this daily or before checking the dashboard)
cargo run -- sync

# Check status
cargo run -- status

# Start the dashboard
make serve
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
```

## Documentation

**Start here:**
- 👋 [Getting Started](./docs/getting-started.md) - Detailed setup and usage guide
- 🏗️ [Architecture](./docs/architecture.md) - How the code is organized
- 🦀 [Rust Basics](./docs/rust-basics.md) - Rust concepts used in this project

**Dive deeper:**
- [Database Schema](./docs/database.md) - What data we store and how to query it
- [Development Guide](./docs/development.md) - Contributing and making changes
- [OpenClass API](./docs/openclass-api.md) - Understanding the data source
- [Testing](./docs/testing.md) - How to write and run tests
- [Why Rust?](./docs/why-rust.md) - Our technology decisions
- [Roadmap](./docs/roadmap.md) - Future ideas

**Full index:** [docs/README.md](./docs/README.md)

## How It Works

1. **Sync** - Fetches student progress from OpenClass API
2. **Store** - Saves to local SQLite database (`~/.cohort-tracker.db`)
3. **Serve** - REST API provides data to the dashboard
4. **Visualize** - Dashboard shows progress, blockers, and risk levels

The sync is incremental by default - it only fetches new data. First sync takes a couple minutes, subsequent syncs are much faster.

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

- ✅ Multi-class support
- ✅ Incremental sync (fast updates)
- ✅ Full sync option (complete refresh)
- ✅ Student progress tracking
- ✅ Risk level identification
- ✅ Blocker assignment detection
- ✅ Activity monitoring
- ✅ REST API
- ✅ Interactive dashboard
- ✅ Local SQLite database

## Contributing

We welcome contributions! Check out the [Development Guide](./docs/development.md) to get started.

The codebase is organized to be approachable for Rust learners. Each module has a clear purpose, and we've documented the tricky bits.

## License

MIT

## Questions?

- Check the [docs/](./docs/README.md) folder
- Look at the code - it has comments where things get interesting
- Open an issue - we're friendly!

Built with 🦀 and ❤️ for educators tracking student progress.

