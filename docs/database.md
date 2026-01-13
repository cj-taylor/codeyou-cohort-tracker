# Database Design

This document explains the SQLite database schema used by Cohort Tracker and the reasoning behind design decisions.

## Schema Overview

The database consists of four main tables that store student progression data from OpenClass:

```sql
-- Core entities
students      -- Student information
assignments   -- Assignment/lesson definitions  
progressions  -- Student progress on assignments

-- Metadata
sync_history  -- Track sync operations
```

## Table Definitions

### `students` Table

```sql
CREATE TABLE students (
    id TEXT PRIMARY KEY,           -- OpenClass user ID
    first_name TEXT NOT NULL,      -- Student's first name
    last_name TEXT NOT NULL,       -- Student's last name  
    email TEXT UNIQUE NOT NULL     -- Student's email (unique)
);
```

**Purpose:** Store basic student information from OpenClass user data.

**Key Design Decisions:**
- `id` is OpenClass's user ID (string format)
- `email` has UNIQUE constraint to prevent duplicate students
- Names are separate fields for flexible display/sorting

**Example Data:**
```sql
INSERT INTO students VALUES 
('686d10387bbf0124aac02088', 'Jane', 'Doe', 'jane.doe@example.com'),
('686d10387bbf0124aac02089', 'John', 'Smith', 'john.smith@example.com');
```

### `assignments` Table

```sql
CREATE TABLE assignments (
    id TEXT PRIMARY KEY,     -- OpenClass assignment ID
    name TEXT NOT NULL,      -- Assignment/lesson name
    type TEXT NOT NULL       -- "lesson" or "quiz"
);
```

**Purpose:** Store assignment metadata from OpenClass.

**Key Design Decisions:**
- `type` distinguishes between lessons and quizzes for analytics
- `name` is the display name from OpenClass
- Simple structure - more fields can be added later

**Example Data:**
```sql
INSERT INTO assignments VALUES 
('68e594f520442cbbe62a19c9', 'Bring It Into Focus', 'lesson'),
('68e594f520442cbbe62a19ca', 'JavaScript Basics Quiz', 'quiz');
```

### `progressions` Table

```sql
CREATE TABLE progressions (
    id TEXT PRIMARY KEY,              -- OpenClass progression ID
    student_id TEXT NOT NULL,         -- Foreign key to students.id
    assignment_id TEXT NOT NULL,      -- Foreign key to assignments.id
    grade REAL,                       -- 0.0 to 1.0, or NULL if not graded
    started_at TEXT NOT NULL,         -- ISO 8601 timestamp
    completed_at TEXT NOT NULL,       -- ISO 8601 timestamp  
    reviewed_at TEXT,                 -- ISO 8601 timestamp, NULL if not reviewed
    synced_at TEXT NOT NULL,          -- When we fetched this record
    FOREIGN KEY (student_id) REFERENCES students(id),
    FOREIGN KEY (assignment_id) REFERENCES assignments(id)
);
```

**Purpose:** Core table storing student progress on assignments.

**Key Design Decisions:**
- `grade` is nullable - assignments may not be graded yet
- All timestamps stored as TEXT in ISO 8601 format for simplicity
- `synced_at` tracks when we last updated this record
- Foreign keys ensure referential integrity

**Example Data:**
```sql
INSERT INTO progressions VALUES 
('693b4d9ba039325fb0b89f92', '686d10387bbf0124aac02088', '68e594f520442cbbe62a19c9', 
 1.0, '2025-12-11T23:02:51.781Z', '2025-12-11T23:02:51.781Z', NULL, '2025-12-12T10:00:00.000Z');
```

### `sync_history` Table

```sql
CREATE TABLE sync_history (
    id INTEGER PRIMARY KEY AUTOINCREMENT,  -- Auto-incrementing ID
    synced_at TEXT NOT NULL,                -- When sync occurred
    class_id TEXT NOT NULL,                 -- Which class was synced
    page INTEGER NOT NULL,                  -- Which API page was processed
    records_processed INTEGER NOT NULL      -- How many records in this page
);
```

**Purpose:** Track sync operations for debugging and monitoring.

**Key Design Decisions:**
- Auto-incrementing integer ID for simple ordering
- Track page-level sync for debugging pagination issues
- `records_processed` helps identify API changes or issues

**Example Data:**
```sql
INSERT INTO sync_history VALUES 
(1, '2025-12-12T10:00:00.000Z', '68e594f320442cbbe62a18dc', 0, 30),
(2, '2025-12-12T10:00:05.000Z', '68e594f320442cbbe62a18dc', 1, 30),
(3, '2025-12-12T10:00:10.000Z', '68e594f320442cbbe62a18dc', 2, 15);
```

## Indexes for Performance

```sql
-- Speed up common queries
CREATE INDEX idx_progressions_student_id ON progressions(student_id);
CREATE INDEX idx_progressions_assignment_id ON progressions(assignment_id);
CREATE INDEX idx_progressions_completed_at ON progressions(completed_at);
CREATE INDEX idx_sync_history_class_id ON sync_history(class_id);
```

**Purpose:** Optimize queries for:
- Student progress lookups
- Assignment completion analysis  
- Time-based filtering
- Sync history by class

## Common Queries

### Student Progress Summary

```sql
-- Get all progress for a specific student
SELECT 
    s.first_name,
    s.last_name,
    a.name AS assignment_name,
    a.type AS assignment_type,
    p.grade,
    p.completed_at
FROM progressions p
JOIN students s ON p.student_id = s.id
JOIN assignments a ON p.assignment_id = a.id
WHERE s.id = '686d10387bbf0124aac02088'
ORDER BY p.completed_at;
```

### Assignment Completion Rates

```sql
-- Get completion rates by assignment
SELECT 
    a.name,
    a.type,
    COUNT(*) as total_attempts,
    COUNT(p.grade) as graded_attempts,
    AVG(p.grade) as average_grade
FROM assignments a
LEFT JOIN progressions p ON a.id = p.assignment_id
GROUP BY a.id, a.name, a.type
ORDER BY total_attempts DESC;
```

### Student Performance Analytics

```sql
-- Identify struggling students (low average grades)
SELECT 
    s.first_name,
    s.last_name,
    s.email,
    COUNT(p.id) as total_assignments,
    AVG(p.grade) as average_grade,
    COUNT(CASE WHEN p.grade < 0.7 THEN 1 END) as failing_assignments
FROM students s
JOIN progressions p ON s.id = p.student_id
WHERE p.grade IS NOT NULL
GROUP BY s.id, s.first_name, s.last_name, s.email
HAVING average_grade < 0.75
ORDER BY average_grade ASC;
```

### Sync Monitoring

```sql
-- Check recent sync activity
SELECT 
    class_id,
    COUNT(*) as pages_synced,
    SUM(records_processed) as total_records,
    MIN(synced_at) as sync_start,
    MAX(synced_at) as sync_end
FROM sync_history 
WHERE synced_at > datetime('now', '-1 day')
GROUP BY class_id
ORDER BY sync_start DESC;
```

## Data Types and Constraints

### Text vs. Integer IDs

**Decision:** Use TEXT for OpenClass IDs
```sql
id TEXT PRIMARY KEY  -- Not INTEGER
```

**Reasoning:** 
- OpenClass uses MongoDB ObjectIds (24-character hex strings)
- TEXT preserves exact format from API
- No risk of integer overflow or conversion issues

### Nullable vs. Required Fields

**Nullable Fields:**
- `progressions.grade` - assignments may not be graded
- `progressions.reviewed_at` - not all assignments are reviewed

**Required Fields:**
- All names, emails, timestamps for core functionality
- Foreign keys to maintain referential integrity

### Timestamp Format

**Decision:** Store as TEXT in ISO 8601 format
```sql
completed_at TEXT NOT NULL  -- '2025-12-11T23:02:51.781Z'
```

**Reasoning:**
- SQLite has limited native date/time support
- ISO 8601 is human-readable and sortable as text
- Easy to parse in Rust with `chrono` crate
- Preserves timezone information from API

## Database Operations in Code

### Connection Management

```rust
use rusqlite::{Connection, Result};

pub struct Database {
    conn: Connection,
}

impl Database {
    pub fn new() -> Result<Self> {
        let conn = Connection::open("cohort-tracker.db")?;
        Ok(Database { conn })
    }
    
    pub fn new_in_memory() -> Result<Self> {
        let conn = Connection::open(":memory:")?;
        Ok(Database { conn })
    }
}
```

### Schema Creation

```rust
impl Database {
    pub fn create_tables(&self) -> Result<()> {
        self.conn.execute_batch(r#"
            CREATE TABLE IF NOT EXISTS students (
                id TEXT PRIMARY KEY,
                first_name TEXT NOT NULL,
                last_name TEXT NOT NULL,
                email TEXT UNIQUE NOT NULL
            );
            
            CREATE TABLE IF NOT EXISTS assignments (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                type TEXT NOT NULL
            );
            
            -- ... other tables
            
            -- Create indexes
            CREATE INDEX IF NOT EXISTS idx_progressions_student_id 
                ON progressions(student_id);
        "#)?;
        
        Ok(())
    }
}
```

### Data Insertion

```rust
impl Database {
    pub fn insert_student(&self, student: &Student) -> Result<()> {
        self.conn.execute(
            "INSERT OR REPLACE INTO students (id, first_name, last_name, email) 
             VALUES (?1, ?2, ?3, ?4)",
            params![student.id, student.first_name, student.last_name, student.email],
        )?;
        Ok(())
    }
    
    pub fn insert_progression(&self, progression: &Progression) -> Result<()> {
        self.conn.execute(
            "INSERT OR REPLACE INTO progressions 
             (id, student_id, assignment_id, grade, started_at, completed_at, reviewed_at, synced_at) 
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            params![
                progression.id,
                progression.student_id,
                progression.assignment_id,
                progression.grade,
                progression.started_at,
                progression.completed_at,
                progression.reviewed_at,
                progression.synced_at,
            ],
        )?;
        Ok(())
    }
}
```

## Performance Considerations

### Batch Operations

```rust
// Use transactions for bulk inserts
pub fn insert_many_progressions(&self, progressions: &[Progression]) -> Result<()> {
    let tx = self.conn.transaction()?;
    
    {
        let mut stmt = tx.prepare(
            "INSERT OR REPLACE INTO progressions (...) VALUES (?1, ?2, ...)"
        )?;
        
        for progression in progressions {
            stmt.execute(params![/* ... */])?;
        }
    }
    
    tx.commit()?;
    Ok(())
}
```

### Query Optimization

```rust
// Use prepared statements for repeated queries
pub fn get_student_progressions(&self, student_id: &str) -> Result<Vec<Progression>> {
    let mut stmt = self.conn.prepare_cached(
        "SELECT * FROM progressions WHERE student_id = ?1 ORDER BY completed_at"
    )?;
    
    // ... execute query
}
```

## Migration Strategy

### Schema Versioning

```sql
-- Track schema version
CREATE TABLE IF NOT EXISTS schema_version (
    version INTEGER PRIMARY KEY
);

INSERT OR REPLACE INTO schema_version VALUES (1);
```

### Adding New Columns

```rust
// Example migration for adding new field
pub fn migrate_to_v2(&self) -> Result<()> {
    self.conn.execute(
        "ALTER TABLE assignments ADD COLUMN difficulty_level INTEGER DEFAULT 1",
        [],
    )?;
    
    self.conn.execute(
        "UPDATE schema_version SET version = 2",
        [],
    )?;
    
    Ok(())
}
```

## Backup and Recovery

### Database Backup

```rust
use std::fs;

pub fn backup_database() -> Result<(), Box<dyn std::error::Error>> {
    let timestamp = chrono::Utc::now().format("%Y%m%d_%H%M%S");
    let backup_path = format!("cohort-tracker_backup_{}.db", timestamp);
    
    fs::copy("cohort-tracker.db", &backup_path)?;
    println!("Database backed up to: {}", backup_path);
    
    Ok(())
}
```

### Data Export

```rust
// Export to CSV for external analysis
pub fn export_progressions_csv(&self, path: &str) -> Result<()> {
    let mut wtr = csv::Writer::from_path(path)?;
    
    let mut stmt = self.conn.prepare(
        "SELECT s.email, a.name, p.grade, p.completed_at 
         FROM progressions p 
         JOIN students s ON p.student_id = s.id 
         JOIN assignments a ON p.assignment_id = a.id"
    )?;
    
    let rows = stmt.query_map([], |row| {
        Ok((
            row.get::<_, String>(0)?,  // email
            row.get::<_, String>(1)?,  // assignment name
            row.get::<_, Option<f64>>(2)?,  // grade
            row.get::<_, String>(3)?,  // completed_at
        ))
    })?;
    
    for row in rows {
        let (email, assignment, grade, completed_at) = row?;
        wtr.write_record(&[
            email,
            assignment,
            grade.map_or("".to_string(), |g| g.to_string()),
            completed_at,
        ])?;
    }
    
    wtr.flush()?;
    Ok(())
}
```

## Testing Database Operations

### In-Memory Testing

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_student_insertion() {
        let db = Database::new_in_memory().unwrap();
        db.create_tables().unwrap();
        
        let student = Student {
            id: "test123".to_string(),
            first_name: "Test".to_string(),
            last_name: "User".to_string(),
            email: "test@example.com".to_string(),
        };
        
        db.insert_student(&student).unwrap();
        
        // Verify insertion
        let count: i64 = db.conn.query_row(
            "SELECT COUNT(*) FROM students WHERE id = ?1",
            params![student.id],
            |row| row.get(0)
        ).unwrap();
        
        assert_eq!(count, 1);
    }
}
```

## Next Steps

- Read [OpenClass API](./openclass-api.md) for data source details
- Check [Development Guide](./development.md) for contributing
- Review [Architecture](./architecture.md) for overall system design
