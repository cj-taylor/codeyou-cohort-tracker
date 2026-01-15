use anyhow::Result;
use sqlite::Connection;

mod analytics;
mod queries;

pub struct Database {
    pub(crate) conn: Connection,
}

impl Database {
    pub fn new(path: &str) -> Result<Self> {
        let conn = sqlite::open(path)?;

        // Create classes table
        conn.execute(
            "CREATE TABLE IF NOT EXISTS classes (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                friendly_id TEXT NOT NULL UNIQUE,
                is_active INTEGER DEFAULT 1,
                synced_at TEXT
            )",
        )?;

        // Create tables with class_id
        conn.execute(
            "CREATE TABLE IF NOT EXISTS students (
                id TEXT NOT NULL,
                class_id TEXT NOT NULL,
                first_name TEXT NOT NULL,
                last_name TEXT NOT NULL,
                email TEXT NOT NULL,
                region TEXT,
                night TEXT,
                PRIMARY KEY (id, class_id)
            )",
        )?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS mentors (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                name TEXT NOT NULL,
                night TEXT NOT NULL
            )",
        )?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS assignments (
                id TEXT NOT NULL,
                class_id TEXT NOT NULL,
                name TEXT NOT NULL,
                type TEXT NOT NULL,
                section TEXT,
                PRIMARY KEY (id, class_id)
            )",
        )?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS progressions (
                id TEXT PRIMARY KEY,
                class_id TEXT NOT NULL,
                student_id TEXT NOT NULL,
                assignment_id TEXT NOT NULL,
                grade REAL,
                started_at TEXT NOT NULL,
                completed_at TEXT NOT NULL,
                reviewed_at TEXT,
                synced_at TEXT NOT NULL
            )",
        )?;

        // Create indexes
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_progressions_class ON progressions(class_id)",
        )?;
        conn.execute("CREATE INDEX IF NOT EXISTS idx_students_class ON students(class_id)")?;
        conn.execute("CREATE INDEX IF NOT EXISTS idx_assignments_class ON assignments(class_id)")?;

        // Migration: Add section column to assignments if it doesn't exist
        let has_section = conn
            .prepare("SELECT section FROM assignments LIMIT 1")
            .is_err();
        if has_section {
            println!("Migrating database: Adding section column to assignments table...");
            conn.execute("ALTER TABLE assignments ADD COLUMN section TEXT")?;
            println!("✓ Migration complete");
        }

        conn.execute(
            "CREATE TABLE IF NOT EXISTS sync_history (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                synced_at TEXT NOT NULL,
                class_id TEXT NOT NULL,
                page INTEGER NOT NULL,
                records_processed INTEGER NOT NULL
            )",
        )?;

        Ok(Self { conn })
    }
}
