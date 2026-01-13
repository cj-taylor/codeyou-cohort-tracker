# Cohort Tracker

Pulls student progress data from OpenClass.ai and stores it locally. Built in Rust.

## Getting started

You'll need Rust 1.80+ installed. If you don't have it, grab it from [rustup.rs](https://rustup.rs/).

```bash
cargo build --release

# Point it at your OpenClass account
cargo run -- init --email you@example.com --password your-password --class-id YOUR_CLASS_ID

# Pull down all the data
cargo run -- sync

# See what you've got
cargo run -- status

# Start the dashboard
make serve
```

The `sync` command grabs everything from OpenClass and saves it to a local SQLite database. First sync takes a minute or two depending on how many students you have.

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
