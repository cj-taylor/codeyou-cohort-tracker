# Cohort Progress Tracker: A Collaborative Rust Project

## Overview

A multi-phase Rust project that mentors from the Software Development and Python Data Analysis cohorts will build together. The tool syncs student progression data from OpenClass.ai and provides APIs for querying, analyzing, and visualizing cohort health.

---

## Data Model

### From OpenClass API Response

```json
{
  "user": {
    "id": "...",
    "first_name": "...",
    "last_name": "...",
    "email": "..."
  },
  "assignment": {
    "id": "...",
    "name": "...",
    "type": "lesson" | "quiz"
  },
  "grade": 0.0..1.0,
  "started_assignment_at": "ISO8601",
  "completed_assignment_at": "ISO8601",
  "reviewed_at": "ISO8601 | null"
}
```

### Database Schema (SQLite)

```sql
-- Students
CREATE TABLE students (
  id TEXT PRIMARY KEY,
  first_name TEXT,
  last_name TEXT,
  email TEXT UNIQUE
);

-- Assignments
CREATE TABLE assignments (
  id TEXT PRIMARY KEY,
  name TEXT,
  type TEXT -- "lesson" or "quiz"
);

-- Progressions (student-assignment records)
CREATE TABLE progressions (
  id TEXT PRIMARY KEY,
  student_id TEXT FOREIGN KEY,
  assignment_id TEXT FOREIGN KEY,
  grade REAL,
  started_at DATETIME,
  completed_at DATETIME,
  reviewed_at DATETIME,
  synced_at DATETIME -- when we fetched this from OpenClass
);

-- Sync metadata
CREATE TABLE sync_history (
  id INTEGER PRIMARY KEY,
  synced_at DATETIME,
  class_id TEXT,
  page INTEGER,
  records_processed INTEGER
);
```

---

## Project Phases

### Phase 1: CLI + Database (Weeks 1-2)

**Lead: Person 2 (Software Dev Mentor)**
**Duration: 1-2 weeks**

**Goals:**

- Build a CLI that authenticates with OpenClass.ai
- Fetch progressions for a given class ID
- Store data in SQLite with proper schema
- Handle pagination (use `can_load_more`)

**Key Learning:**

- HTTP clients in Rust (`reqwest` or `hyper`)
- Authentication (bearer tokens, custom headers)
- JSON parsing (`serde_json`)
- Database interactions (`sqlx` or `rusqlite`)
- Error handling (API errors, DB errors, parsing)
- Pagination logic

**Deliverable:**

```bash
cohort-tracker sync --class-id 68e594f320442cbbe62a18dc
# Fetches all progressions from OpenClass, stores in SQLite
# Output: "Synced 3781 records in 45 seconds"
```

**Commands to support:**

- `init` — set up local config file with credentials
- `sync` — fetch latest data from OpenClass
- `status` — show when last sync was, how many records

---

### Phase 2: REST API (Weeks 3-4)

**Lead: Person 2 (Software Dev Mentor)**
**Duration: 1-2 weeks**

**Goals:**

- Build a REST API on top of the CLI logic
- Define endpoints mentors actually need
- Return JSON responses with proper error handling

**Key Learning:**

- Web frameworks in Rust (`Axum` or `Actix`)
- Route handling
- Request/response serialization
- Status codes and error responses
- Async/await patterns
- Testing API endpoints

**Endpoints to implement:**

```
GET /health
  → {"status": "ok", "last_sync": "2025-01-04T15:30:00Z"}

GET /classes/{classId}/students
  → List all unique students in class

GET /classes/{classId}/assignments
  → List all unique assignments in class

GET /classes/{classId}/progressions?student_id=X&completed_after=2025-01-01
  → Filter progressions by student, date range, etc.

GET /classes/{classId}/progress-summary
  → {"total_students": 40, "avg_completion": 0.85, "completed_count": 34, "in_progress_count": 6}

POST /sync
  → Trigger manual sync from API (optional)
```

**Deliverable:**

```bash
cargo run -- server
# Starts on http://localhost:3000
# Can call: curl http://localhost:3000/classes/68e594.../students
```

---

### Phase 3: Analytics & Insights (Week 5)

**Lead: Person 1 (Data Analysis Mentor)**
**Duration: 1 week**

**Goals:**

- Add endpoints that answer mentorship questions
- Compute metrics over time
- Identify patterns and blockers

**Key Learning:**

- SQL aggregation queries
- Time-series analysis
- Data transformation logic
- Filtering and ranking
- What questions mentors actually need answered

**New endpoints:**

```
GET /classes/{classId}/metrics/completion
  → {
      "total_assignments": 30,
      "assignments_with_0_completions": 2,
      "avg_students_per_assignment": 38.5
    }

GET /classes/{classId}/metrics/blockers
  → Rank assignments by: fewest completions, lowest avg grade, longest time-to-completion
  → [
      {"assignment_id": "...", "name": "Week 5 Quiz", "completion_rate": 0.65, "avg_grade": 0.72},
      ...
    ]

GET /classes/{classId}/metrics/student-health
  → Identify students falling behind:
  → [
      {"student_id": "...", "name": "...", "completed": 12, "total": 30, "pct": 0.40, "risk": "high"},
      ...
    ]

GET /classes/{classId}/metrics/progress-over-time
  → Show cumulative completion by week
  → [
      {"week": "2025-11-24", "completed": 150, "in_progress": 30},
      ...
    ]
```

**Mentors can use these to:**

- See which assignments need reteaching
- Identify students who need 1-on-1 help
- Understand cohort velocity
- Plan interventions

**Deliverable:**

```bash
curl http://localhost:3000/classes/68e594.../metrics/blockers
# [{"assignment_id": "...", "name": "Week 5 Quiz", "completion_rate": 0.65}, ...]
```

---

### Phase 4: Dashboard (Week 6+, Optional)

**Lead: You**

**Goals:**

- Simple web UI to visualize metrics
- No heavy frontend framework needed (HTML + fetch + minimal JS)
- Shows cohort health at a glance

**Pages:**

- Overview: total students, completion %, last sync
- Blockers: ranked assignments by difficulty
- Student Risk: which students are behind
- Trends: completion over time (simple chart)

**Deliverable:**

- Visit <http://localhost:3000/dashboard>
- See cohort health in real time

---

## Technology Stack

### Core

- **Language:** Rust
- **Web Framework:** Axum (lightweight, modern, good for APIs)
- **Database:** SQLite + SQLx (compile-time SQL checking)
- **HTTP Client:** reqwest (simple, well-documented)
- **JSON:** serde + serde_json

### Development

- Cargo for builds/tests
- Simple error handling (anyhow crate)
- Optional: sqlx-cli for migrations

### Project Structure

```
cohort-tracker/
├── Cargo.toml
├── src/
│   ├── main.rs              # CLI entry point
│   ├── api/
│   │   ├── mod.rs           # Web server setup
│   │   ├── handlers.rs      # Route handlers
│   │   └── responses.rs     # JSON response types
│   ├── db/
│   │   ├── mod.rs           # Database setup
│   │   └── queries.rs       # SQL queries
│   ├── openclass/
│   │   ├── mod.rs           # OpenClass API client
│   │   ├── auth.rs          # Authentication
│   │   └── types.rs         # API response types
│   ├── sync.rs              # Sync logic (fetch + store)
│   └── config.rs            # Config file handling
├── migrations/              # SQLx migrations (optional)
└── tests/                   # Integration tests
```

---

## Week-by-Week Breakdown

### Week 1-2: Phase 1 (CLI + Sync)

- Day 1-2: Project setup, Cargo scaffold, basic file structure
- Day 3-4: OpenClass auth + HTTP client
- Day 5-6: JSON parsing (serde types matching OpenClass response)
- Day 7-8: SQLite schema + sqlx setup
- Day 9-10: Sync logic (fetch all pages, insert into DB)
- Day 11-12: CLI commands (init, sync, status)
- Day 13-14: Testing, error handling, documentation

**Person 2 leads**, you review code and handle tricky parts (error handling, pagination logic).

### Week 3-4: Phase 2 (API)

- Day 1-2: Axum setup, basic routing
- Day 3-4: Implement GET /health, /students, /assignments
- Day 5-6: Implement GET /progressions (with filtering)
- Day 7-8: Implement GET /progress-summary
- Day 9-10: Error handling (proper HTTP responses)
- Day 11-12: Testing endpoints with curl/Postman
- Day 13-14: Documentation, refactoring

**Person 2 leads**, you handle architectural decisions.

### Week 5: Phase 3 (Analytics)

- Day 1-2: Design SQL queries for metrics
- Day 3-4: Implement /metrics/completion, /metrics/blockers
- Day 5-6: Implement /metrics/student-health, /metrics/progress-over-time
- Day 7-8: Test metrics with real data
- Day 9-10: Documentation

**Person 1 leads** (they understand what questions mentors need). You help with Rust/SQL implementation.

### Week 6+: Phase 4 (Dashboard)

- Simple HTML template
- Fetch from API endpoints
- Basic charts (Chart.js or similar)
- Deploy somewhere accessible

---

## Collaboration Points

**Code Review Cycle:**

1. Person 2 pushes code to a branch
2. You and Person 1 review (check for correctness, error handling, style)
3. Iterate and merge
4. Switch leads for next phase

**Weekly Check-in:**

- 30 min sync on progress
- Blockers/questions
- Next week's focus

**Shared Responsibility:**

- Error handling: all three
- Testing: all three
- Documentation: Person 2 or Person 1 write, you review

---

## Success Criteria

### Phase 1

- ✅ CLI successfully syncs data from OpenClass
- ✅ SQLite database has all student progressions
- ✅ Pagination works (all 3781+ records fetched)
- ✅ Handles auth errors, network errors gracefully

### Phase 2

- ✅ API starts and serves requests
- ✅ GET /progressions filters work correctly
- ✅ Responses are consistent JSON
- ✅ Error responses have proper status codes

### Phase 3

- ✅ Metrics endpoints return sensible data
- ✅ Metrics match what mentors actually need
- ✅ Person 1 can use these to plan lessons

### Phase 4 (Optional)

- ✅ Dashboard loads and shows data
- ✅ Easy for mentors to use

---

## Getting Started

1. Create a GitHub repo (or use what you have)
2. Set up Cargo project: `cargo new cohort-tracker`
3. Add dependencies to Cargo.toml:

   ```toml
   [dependencies]
   reqwest = { version = "0.11", features = ["json"] }
   serde = { version = "1.0", features = ["derive"] }
   serde_json = "1.0"
   sqlx = { version = "0.7", features = ["sqlite", "runtime-tokio-rustls"] }
   tokio = { version = "1", features = ["full"] }
   axum = "0.7"
   anyhow = "1.0"
   dotenv = "0.15"
   ```

4. Start with Phase 1: define types and auth

---

## Future Ideas (Post-April)

- Support multiple classes/cohorts simultaneously
- Role-based access (admin, mentor, read-only)
- Webhooks (notify mentors when student falls behind)
- Export metrics as CSV/PDF reports
- Custom dashboards per mentor
- Student self-service view (see my own progress)
- Integration with Slack (daily digest of at-risk students)

---

## Notes

- **Timeline is flexible:** adjust phases based on actual progress
- **Complexity can scale:** add features as confidence grows
- **Real-world useful:** mentors will actually use this in their cohorts
- **Learning-focused:** each phase teaches specific Rust patterns
- **Collaboration is the goal:** being "better together" means code review, pairing, and shared problem-solving
