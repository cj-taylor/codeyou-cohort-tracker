# Documentation

Documentation for the Cohort Tracker project. Start with Getting Started if you're new, or jump to Architecture to understand how the code is organized.

## Start Here

**New to the project?** → [Getting Started](./getting-started.md)  
Get it running in 10 minutes. No Rust experience needed.

**Want to understand the code?** → [Architecture](./architecture.md)  
Learn how the pieces fit together and why we organized it this way.

**New to Rust?** → [Rust Basics](./rust-basics.md)  
The Rust concepts used in this codebase, explained simply.

## Guides by Topic

### Using the Tool

- **[Getting Started](./getting-started.md)** - Installation, first sync, daily usage
- **[Dashboard Guide](./dashboard-guide.md)** - Complete walkthrough of dashboard features with screenshots
- **[Database Schema](./database.md)** - What data we store and how to query it

### Understanding the Code

- **[Architecture](./architecture.md)** - How the code is organized
- **[Rust Basics](./rust-basics.md)** - Rust concepts used in this project
- **[Why Rust?](./why-rust.md)** - Our technology decisions and trade-offs

### Contributing

- **[Development Guide](./development.md)** - Set up your dev environment, make changes
- **[Testing](./testing.md)** - How to write and run tests
- **[OpenClass API](./openclass-api.md)** - Understanding the data source

### Future Ideas

- **[Roadmap](./roadmap.md)** - Features we're thinking about

## Quick Navigation

**I want to...**

- **Get it running** → [Getting Started](./getting-started.md)
- **Learn to use the dashboard** → [Dashboard Guide](./dashboard-guide.md)
- **Add a new CLI command** → [Development Guide](./development.md) and `src/cli.rs`
- **Add a new database query** → [Database Schema](./database.md) and `src/db/queries.rs`
- **Add a new API endpoint** → `src/api.rs`
- **Support a new LMS** → [Architecture](./architecture.md) (LMS Provider section)
- **Understand the sync logic** → [OpenClass API](./openclass-api.md) and `src/sync/engine.rs`
- **Query the database directly** → [Database Schema](./database.md)
- **Learn Rust** → [Rust Basics](./rust-basics.md) and [Why Rust?](./why-rust.md)

## Documentation Philosophy

We write docs for humans, not robots:

- **Conversational tone** - We say "you" not "the developer"
- **Explain why** - Not just what, but why we did it that way
- **Real examples** - From the actual codebase
- **Acknowledge complexity** - When something's tricky, we say so
- **Encourage experimentation** - The compiler has your back

## What's Where in the Code

```
src/
├── models.rs        # Data structures (Class, Student, etc.)
├── config.rs        # Configuration management
├── cli.rs           # Command-line interface
├── api.rs           # REST API server
├── db/              # Database layer
│   ├── mod.rs       # Schema and setup
│   ├── queries.rs   # CRUD operations
│   └── analytics.rs # Complex queries
├── lms/             # LMS provider abstraction
│   └── openclass/   # OpenClass implementation
└── sync/            # Sync engine
```

See [Architecture](./architecture.md) for the full picture.

## Contributing to Docs

Found something confusing? Here's how to help:

1. **Ask questions** - If you're confused, others probably are too
2. **Suggest improvements** - Open an issue or PR
3. **Add examples** - Real code examples help
4. **Fix typos** - Every bit helps

## Learning Path

If you're new to both Rust and this project, start here:

1. **[Getting Started](./getting-started.md)** - Get it running
2. **[Rust Basics](./rust-basics.md)** - Learn the language concepts
3. **[Architecture](./architecture.md)** - Understand the structure
4. **[Development Guide](./development.md)** - Make your first change
5. **Pick a piece to dive deep** - Database, API, sync logic, etc.

## Getting Help

- **Read the code** - It has comments where things get tricky
- **Check the docs** - You're in the right place
- **Ask questions** - Open an issue
- **Experiment** - Change something and see what happens

## Quick Reference

```bash
# First time setup
cargo build --release
cargo run -- init --email you@example.com --password pass
cargo run -- sync

# Daily use
cargo run -- sync          # Fetch new data
make serve                 # Start dashboard

# Development
cargo test                 # Run tests
cargo build                # Build
cargo run -- server        # Start API
```
