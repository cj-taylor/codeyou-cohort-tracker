# Cohort Tracker

A Rust CLI application and REST API for syncing and tracking student progress from OpenClass.ai.

## Overview

Cohort Tracker pulls student progression data directly from OpenClass, stores it in a local SQLite database, and provides powerful APIs for analyzing cohort health, identifying struggling students, and understanding common blockers.

**Perfect for:** Coding bootcamp mentors who need to track multiple cohorts across weeks of curriculum.

## Features

- **CLI-based sync** from OpenClass API with authentication
- **SQLite database** for persistent storage of all progression data
- **REST API** (Phase 2) for querying student progress
- **Analytics endpoints** (Phase 3) for cohort metrics and insights
- **Rate-limited API calls** to be respectful to OpenClass
- **Comprehensive error handling** for robust operation

## Quick Start

### Prerequisites

- Rust 1.80+ ([Install Rust](https://rustup.rs/))
- OpenClass.ai account with admin access
- A class ID from OpenClass

### Installation

```bash
# Clone or extract the project
git clone <repo-url>
cd cohort-tracker

# Build the project
cargo build --release

# Binary will be at ./target/release/cohort-tracker
```

### Initialize Configuration

```bash
cargo run -- init \
  --email your-email@example.com \
  --password your-password \
  --class-id 68e594f320442cbbe62a18dc
```

This creates `~/.cohort-tracker.toml` with your credentials.

### Sync Data

```bash
cargo run -- sync
```

This fetches all progressions from OpenClass and stores them locally. First sync typically takes 1-2 minutes for a full cohort.

### Check Status

```bash
cargo run -- status
```

Shows:

- Configuration location
- Student count
- Assignment count
- Total progressions
- Last sync timestamp

## Architecture

### Phase 1: CLI + Sync (Complete)

The core syncing engine that pulls data from OpenClass.

**Commands:**

- `init` — Set up configuration with OpenClass credentials
- `sync` — Fetch all data from OpenClass and store locally
- `status` — Display database stats and sync history

**Modules:**

- `config.rs` — Configuration management and persistence
- `openclass.rs` — API type definitions for OpenClass responses
- `db.rs` — SQLite database operations
- `sync.rs` — OpenClass API client and sync logic
- `cli.rs` — Command-line interface
- `main.rs` — Application entry point

### Phase 2: REST API (Planned)

Web server with endpoints for querying student data.

**Planned endpoints:**

- `GET /health` — Server status
- `GET /classes/{id}/students` — List all students
- `GET /classes/{id}/assignments` — List all assignments
- `GET /classes/{id}/progressions` — Query progressions with filters
- `GET /classes/{id}/progress-summary` — Overview stats

### Phase 3: Analytics (Planned)

Metrics and insights for mentors.

**Planned endpoints:**

- `GET /classes/{id}/metrics/completion` — Assignment completion rates
- `GET /classes/{id}/metrics/blockers` — Ranked difficulty analysis
- `GET /classes/{id}/metrics/student-health` — At-risk student identification
- `GET /classes/{id}/metrics/progress-over-time` — Cumulative progress trends

## Database Schema

### students

```sql
CREATE TABLE students (
    id TEXT PRIMARY KEY,
    first_name TEXT NOT NULL,
    last_name TEXT NOT NULL,
    email TEXT UNIQUE NOT NULL
);
```

### assignments

```sql
CREATE TABLE assignments (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    type TEXT NOT NULL  -- "lesson" or "quiz"
);
```

### progressions

```sql
CREATE TABLE progressions (
    id TEXT PRIMARY KEY,
    student_id TEXT NOT NULL,
    assignment_id TEXT NOT NULL,
    grade REAL,  -- 0.0 to 1.0, or NULL
    started_at TEXT NOT NULL,
    completed_at TEXT NOT NULL,
    reviewed_at TEXT,  -- NULL if not reviewed
    synced_at TEXT NOT NULL,
    FOREIGN KEY (student_id) REFERENCES students(id),
    FOREIGN KEY (assignment_id) REFERENCES assignments(id)
);
```

### sync_history

```sql
CREATE TABLE sync_history (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    synced_at TEXT NOT NULL,
    class_id TEXT NOT NULL,
    page INTEGER NOT NULL,
    records_processed INTEGER NOT NULL
);
```

## Configuration

Configuration is stored in `~/.cohort-tracker.toml`:

```toml
email = "your-email@example.com"
password = "your-password"
class_id = "68e594f320442cbbe62a18dc"
api_base = "https://api.openclass.ai"
```

You can use a different config file:

```bash
cargo run -- --config /path/to/config.toml sync
```

## OpenClass API Integration

Cohort Tracker uses the OpenClass API to fetch student progression data:

**Authentication:**

- POST `/v1/auth/login` with email/password
- Returns bearer token for authenticated requests

**Data Fetching:**

- GET `/v1/classes/{classId}/progressions`
- Supports pagination with `page` and `return_count` parameters
- Returns metadata with `can_load_more` flag

**Response Format:**

```json
{
  "metadata": {
    "total": 3781,
    "page": 0,
    "results_per_page": 30,
    "can_load_more": true
  },
  "data": [
    {
      "_id": { "$oid": "693b4d9ba039325fb0b89f92" },
      "user": {
        "id": "686d10387bbf0124aac02088",
        "first_name": "Jane",
        "last_name": "Doe",
        "email": "jane@example.com"
      },
      "assignment": {
        "id": "68e594f520442cbbe62a19c9",
        "name": "Bring It Into Focus",
        "type": "lesson"
      },
      "grade": 1,
      "started_assignment_at": "2025-12-11T23:02:51.781Z",
      "completed_assignment_at": "2025-12-11T23:02:51.781Z",
      "reviewed_at": null
    }
  ]
}
```

## Building & Running

### Development Build

```bash
cargo build
./target/debug/cohort-tracker status
```

### Release Build (Optimized)

```bash
cargo build --release
./target/release/cohort-tracker status
```

### Running Tests

```bash
cargo test
```

## Dependencies

- **tokio** — Async runtime for concurrent operations
- **reqwest** — HTTP client for API calls
- **serde/serde_json** — JSON serialization/deserialization
- **rusqlite** — SQLite database operations
- **clap** — Command-line argument parsing
- **axum** — Web framework (Phase 2)
- **anyhow** — Error handling

See `Cargo.toml` for complete dependency list and versions.

## Troubleshooting

### Authentication Fails

```
Error: Authentication failed: 401
```

**Solutions:**

- Verify email and password are correct
- Ensure OpenClass account has admin/mentor permissions
- Check that the OpenClass API endpoint is accessible

### Class ID Not Found

```
Error: Failed to fetch progressions: 404
```

**Solutions:**

- Double-check the class ID is correct
- Verify the class exists and you have access to it
- Check OpenClass UI for the correct class ID

### Database Locked

```
Error: database is locked
```

**Solutions:**

- Wait for any running sync to complete
- Delete `cohort-tracker.db` to start fresh (data will be re-synced)
- Check no other process is using the database

### Compilation Errors

```
error: failed to resolve: use of undeclared crate
```

**Solutions:**

- Run `rustup update` to get the latest Rust
- Run `cargo clean` then `cargo build` to rebuild
- Verify Rust version is 1.80+: `rustc --version`

## Development Workflow

### For Building Phase 2 (REST API)

1. Add Axum dependencies to `Cargo.toml`
2. Create `src/api/` module with handlers
3. Create `src/api/responses.rs` for JSON types
4. Update `main.rs` to start web server
5. Test endpoints with `curl` or Postman

### For Building Phase 3 (Analytics)

1. Add analytics functions to `src/db.rs`
2. Create `src/analytics.rs` for complex queries
3. Add analytics endpoints to API
4. Test with real data from Phase 1 sync

## Performance Considerations

- **First sync:** 3,000-4,000 records takes ~2 minutes (with rate limiting)
- **Subsequent syncs:** ~1 minute (fetches updated data only)
- **Rate limiting:** 500ms delay between API pages (respectful to OpenClass)
- **Database:** SQLite is sufficient for cohorts up to ~50 students, 200+ assignments

For larger deployments, consider PostgreSQL in Phase 2/3.

## Contributing

This is designed as a learning project. To extend:

1. Create a feature branch
2. Implement your changes
3. Test thoroughly with real OpenClass data
4. Document any new APIs or commands

## Mentor Notes

### Person 1 (Data Analysis Cohort Mentor)

You'll focus on:

- Phase 3: Analytics queries and metrics
- Understanding data aggregation and filtering
- Designing useful questions mentors need answered

### Person 2 (Software Development Cohort Mentor)

You'll focus on:

- Phase 1: CLI, HTTP client, auth (foundational)
- Phase 2: REST API, web frameworks
- API design and error handling

### You (Overall Lead)

- Ensure architecture is sound
- Guide them through Rust's learning curve
- Review code for correctness and style
- Debug blocking issues

## Roadmap

- [x] Phase 1: CLI + Sync
- [ ] Phase 2: REST API with Axum
- [ ] Phase 3: Analytics and metrics
- [ ] Phase 4: Dashboard UI
- [ ] Phase 5: Multi-cohort support
- [ ] Phase 6: Role-based access control
- [ ] Phase 7: Slack/Discord integration

## License

Educational use. Build with it, learn from it, share what you build.

## Support

For issues or questions:

1. Check the IMPLEMENTATION_GUIDE.md for detailed setup
2. Review cohort_tracker_project_plan.md for architecture
3. Check Troubleshooting section above
4. Review OpenClass API integration section

---

**Happy tracking! 🎓**
