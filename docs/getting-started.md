# Getting Started

This guide will help you get the Cohort Tracker running. No Rust experience required to start.

## What Does This Thing Do?

Cohort Tracker pulls student progress data from OpenClass.ai and stores it locally in a database. Then it gives you:

- **A dashboard** to visualize student progress
- **Analytics** to identify struggling students
- **An API** to build your own tools

Think of it as your own copy of the class data that you can query however you want.

## Before You Start

You need:

1. **Rust** - The programming language this is built with
   - Install from [rustup.rs](https://rustup.rs/)
   - Takes about 5 minutes
   - Works on Mac, Linux, and Windows

2. **OpenClass credentials** - Email and password for your instructor/mentor account
   - You need access to at least one class

3. **A terminal** - Command line interface
   - Mac: Terminal app
   - Windows: PowerShell or Command Prompt
   - Linux: You know what to do

## First Time Setup

### 1. Get the Code

```bash
git clone https://github.com/yourusername/cohort-tracker.git
cd cohort-tracker
```

### 2. Build It

```bash
cargo build --release
```

This compiles the Rust code into an executable. First time takes a few minutes while it downloads dependencies.

Cargo (Rust's package manager) downloads libraries and compiles everything. The `--release` flag makes it optimized and fast.

### 3. Initialize with Your Credentials

```bash
cargo run -- init --email your@email.com --password yourpassword
```

This does three things:
1. Saves your credentials to `~/.cohort-tracker.toml`
2. Connects to OpenClass and fetches your classes
3. Asks which classes you want to track

You'll see something like:

```
Found 3 classes:
  - data-analysis-pathway-module-1-aug-2
  - data-analysis-pathway-module-2-aug-2
  - web-dev-fundamentals-sep-1

Enter friendly IDs to activate (comma-separated, or 'all'):
>
```

Type the class IDs you want (separated by commas) or just type `all`.

### 4. Sync the Data

```bash
cargo run -- sync
```

This fetches student progress from OpenClass and stores it in `~/.cohort-tracker.db` (SQLite database). It shows progress as it goes.

First sync takes a minute or two. Subsequent syncs are faster (they only fetch new data).

### 5. Start the Dashboard

```bash
make serve
```

Or manually:

```bash
cargo run -- server &
open http://localhost:3000
```

**What's happening?**
- Starts a web server on port 3000
- Opens the dashboard in your browser
- Dashboard reads from your local database

## Daily Usage

### Check Status

```bash
cargo run -- status
```

Shows what classes you're tracking and when they were last synced.

### Sync New Data

```bash
cargo run -- sync
```

Fetches any new student progress since last sync. Run this daily or before checking the dashboard.

### Sync a Specific Class

```bash
cargo run -- sync --class data-analysis-pathway-module-1-aug-2
```

### Force a Full Refresh

```bash
cargo run -- sync --full
```

Fetches everything, not just new data. Use this if something seems off.

### Manage Classes

```bash
# List all classes
cargo run -- list

# Activate a class
cargo run -- activate data-analysis-pathway-module-3-jan-1

# Deactivate a class
cargo run -- deactivate old-class-name
```

## Understanding the Dashboard

Once you open http://localhost:3000, you'll see:

**Overview Tab**
- Total students, assignments, completion rates
- Quick health check of your cohort

**Students Tab**
- List of all students
- Completion percentages
- Risk levels (who's falling behind)
- Click a student to see their detailed progress

**Blockers Tab**
- Assignments with low completion rates
- These are where students get stuck
- Helps you know what to review in class

**Activity Tab**
- Who's been active recently
- Who hasn't submitted anything in a while
- Filter by night/cohort

## Common Tasks

### I Want to See Who's Struggling

1. Open dashboard → Students tab
2. Look at "Students at Risk" table
3. Click a student's name for details
4. See exactly which assignments they're missing

### I Want to Know What to Review in Class

1. Open dashboard → Blockers tab
2. Top assignments are where students get stuck
3. These are good candidates for review sessions

### I Want to Export Data

The database is just SQLite. You can query it directly:

```bash
sqlite3 ~/.cohort-tracker.db "SELECT * FROM students"
```

Or use any SQLite tool. The schema is documented in [database.md](./database.md).

### I Want to Build My Own Tool

The API is running at http://localhost:3000. Try:

```bash
curl http://localhost:3000/classes
```

See [api.rs](../src/api.rs) for all endpoints.

## Troubleshooting

### "Authentication failed"

- Check your email/password
- Make sure you have instructor/mentor access
- Try logging into OpenClass.ai in a browser first

### "No classes found"

- Your account might not have access to any classes yet
- Contact your program coordinator

### "Database is locked"

- Another instance is running
- Kill it: `pkill cohort-tracker`
- Or just restart your terminal

### Sync is Slow

- First sync is always slow (fetching everything)
- Subsequent syncs are faster (incremental)
- Use `--full` flag sparingly

### Dashboard Shows Old Data

- Run `cargo run -- sync` to fetch new data
- Refresh your browser
- Check when it was last synced: `cargo run -- status`

## What's Next?

Now that you have it running:

- **Explore the code** - Start with [architecture.md](./architecture.md)
- **Learn Rust** - Check out [rust-basics.md](./rust-basics.md)
- **Add features** - See [development.md](./development.md)
- **Understand the data** - Read [database.md](./database.md)

## Tips for Learning

**Don't try to understand everything at once.** Pick one piece:

- Want to add a CLI command? Look at `cli.rs`
- Want to add a database query? Look at `db/queries.rs`
- Want to understand the API? Look at `lms/openclass/`

**Run the code with changes.** The best way to learn is to modify something small and see what happens. The compiler will tell you if you break something.

**Ask questions.** If something's confusing, that's a documentation bug. Let us know!

## Using the Makefile (Optional but Easier)

We've included a Makefile to make common commands easier. Instead of typing long `cargo run` commands, you can use short `make` commands:

```bash
# See all available commands
make help

# Common tasks
make sync              # Instead of: cargo run -- sync
make status            # Instead of: cargo run -- status
make serve             # Instead of: cargo run -- server

# With parameters
make sync-class CLASS=data-analysis-pathway-module-2-aug-2
make activate CLASS=my-class-id
```

**Why both?** The Makefile is convenient, but showing the full `cargo run` commands helps you understand what's actually happening. Use whichever you prefer!

**Learning tip:** Run `make help` to see all available commands with their actual cargo commands shown.

## Quick Reference

```bash
# Setup
cargo build --release
cargo run -- init --email you@example.com --password pass
# Or: make init EMAIL=you@example.com PASSWORD=pass

# Daily use
cargo run -- sync          # Fetch new data
cargo run -- status        # Check what's tracked
make serve                 # Start dashboard

# Management
cargo run -- list          # Show all classes
cargo run -- activate ID   # Track a class
cargo run -- deactivate ID # Stop tracking a class

# Development
cargo test                 # Run tests
cargo build                # Build (debug mode)
cargo run -- server        # Start API server
```

## Need Help?

- Check the [docs/](.) folder for detailed guides
- Look at the code - it has comments where things get tricky
- The Rust compiler is helpful with error messages
