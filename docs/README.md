# Cohort Tracker Documentation

Howdy, folks! This is the documentation for Cohort Tracker, a Rust project that helps mentors keep tabs on student progress from OpenClass.ai.

## What's in here

- [**Getting Started with Rust**](./rust-basics.md) - Never touched Rust before? Start here
- [**Project Architecture**](./architecture.md) - How we organized the code
- [**Why Rust?**](./why-rust.md) - Why we picked Rust (and what that means)
- [**API Integration**](./openclass-api.md) - How we talk to OpenClass
- [**Database Design**](./database.md) - Our SQLite setup and queries
- [**Development Guide**](./development.md) - Want to contribute? This is your guide
- [**Roadmap**](./roadmap.md) - Ideas for where to take this next

## Quick Overview

This thing pulls student data from OpenClass.ai, saves it to a local SQLite database, and gives you a REST API to query it. All three phases are done:

1. CLI sync tool
2. REST API server
3. Analytics endpoints

## If you're new to Rust

I'd suggest reading these in order:

1. [Rust Basics](./rust-basics.md) - The core concepts you'll see everywhere
2. [Architecture](./architecture.md) - How we structured everything
3. [Why Rust?](./why-rust.md) - Why we made this choice (spoiler: it wasn't easy)

## Need help?

- Setup instructions are in the [README](../README.md)
- Source code has comments explaining the tricky bits
- Check the [Roadmap](./roadmap.md) if you're looking for what to work on next
