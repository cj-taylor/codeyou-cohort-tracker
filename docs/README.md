# Cohort Tracker Documentation

Hey there! This is the documentation for Cohort Tracker, a Rust project that helps coding bootcamp mentors keep tabs on student progress from OpenClass.ai.

## What's in here

- [**Getting Started with Rust**](./rust-basics.md) - Never touched Rust before? Start here
- [**Project Architecture**](./architecture.md) - How we organized the code
- [**Why Rust?**](./why-rust.md) - Why we picked Rust (and what that means)
- [**API Integration**](./openclass-api.md) - How we talk to OpenClass
- [**Database Design**](./database.md) - Our SQLite setup and queries
- [**Development Guide**](./development.md) - Want to contribute? This is your guide

## Quick Overview

This thing pulls student data from OpenClass.ai, saves it to a local SQLite database, and gives you CLI tools to analyze it. We're building it in three phases:

1. **Phase 1 (Done)**: CLI sync tool
2. **Phase 2 (Coming)**: REST API server
3. **Phase 3 (Future)**: Analytics and metrics

## If you're new to Rust

I'd suggest reading these in order:

1. [Rust Basics](./rust-basics.md) - The core concepts you'll see everywhere
2. [Architecture](./architecture.md) - How we structured everything
3. [Why Rust?](./why-rust.md) - Why we made this choice (spoiler: it wasn't easy)

## Need help?

- Main setup instructions are in the [README.md](../README.md)
- Hit a wall? Check the [Troubleshooting](../README.md#troubleshooting) section
- The source code has comments explaining the tricky bits
