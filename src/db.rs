use anyhow::Result;
use sqlite::Connection;

pub struct Database {
    conn: Connection,
}

impl Database {
    pub fn new(path: &str) -> Result<Self> {
        let conn = sqlite::open(path)?;

        // Create tables
        conn.execute(
            "CREATE TABLE IF NOT EXISTS students (
                id TEXT PRIMARY KEY,
                first_name TEXT NOT NULL,
                last_name TEXT NOT NULL,
                email TEXT UNIQUE NOT NULL
            )",
        )?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS assignments (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                type TEXT NOT NULL
            )",
        )?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS progressions (
                id TEXT PRIMARY KEY,
                student_id TEXT NOT NULL,
                assignment_id TEXT NOT NULL,
                grade REAL,
                started_at TEXT NOT NULL,
                completed_at TEXT NOT NULL,
                reviewed_at TEXT,
                synced_at TEXT NOT NULL,
                FOREIGN KEY (student_id) REFERENCES students(id),
                FOREIGN KEY (assignment_id) REFERENCES assignments(id)
            )",
        )?;

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

    pub fn insert_student(&self, id: &str, first_name: &str, last_name: &str, email: &str) -> Result<()> {
        let stmt = self.conn.prepare(
            "INSERT OR IGNORE INTO students (id, first_name, last_name, email) VALUES (?, ?, ?, ?)"
        )?;
        let mut stmt = stmt
            .bind(1, id)?
            .bind(2, first_name)?
            .bind(3, last_name)?
            .bind(4, email)?;
        stmt.next()?;
        Ok(())
    }

    pub fn insert_assignment(&self, id: &str, name: &str, assignment_type: &str) -> Result<()> {
        let stmt = self.conn.prepare(
            "INSERT OR IGNORE INTO assignments (id, name, type) VALUES (?, ?, ?)"
        )?;
        let mut stmt = stmt
            .bind(1, id)?
            .bind(2, name)?
            .bind(3, assignment_type)?;
        stmt.next()?;
        Ok(())
    }

    pub fn insert_progression(
        &self,
        id: &str,
        student_id: &str,
        assignment_id: &str,
        grade: Option<f64>,
        started_at: &str,
        completed_at: &str,
        reviewed_at: Option<&str>,
    ) -> Result<()> {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let stmt = self.conn.prepare(
            "INSERT OR REPLACE INTO progressions
            (id, student_id, assignment_id, grade, started_at, completed_at, reviewed_at, synced_at)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?)"
        )?;
        let stmt = stmt
            .bind(1, id)?
            .bind(2, student_id)?
            .bind(3, assignment_id)?;
        let stmt = match grade {
            Some(g) => stmt.bind(4, g)?,
            None => stmt.bind(4, ())?,
        };
        let stmt = stmt
            .bind(5, started_at)?
            .bind(6, completed_at)?;
        let stmt = match reviewed_at {
            Some(r) => stmt.bind(7, r)?,
            None => stmt.bind(7, ())?,
        };
        let mut stmt = stmt.bind(8, now as i64)?;
        stmt.next()?;
        Ok(())
    }

    pub fn record_sync(&self, class_id: &str, page: i32, records_processed: i32) -> Result<()> {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let stmt = self.conn.prepare(
            "INSERT INTO sync_history (synced_at, class_id, page, records_processed) VALUES (?, ?, ?, ?)"
        )?;
        let mut stmt = stmt
            .bind(1, now as i64)?
            .bind(2, class_id)?
            .bind(3, page as i64)?
            .bind(4, records_processed as i64)?;
        stmt.next()?;
        Ok(())
    }

    pub fn get_student_count(&self) -> Result<i64> {
        let mut stmt = self.conn.prepare("SELECT COUNT(*) FROM students")?;
        stmt.next()?;
        let count = stmt.read::<i64>(0)?;
        Ok(count)
    }

    pub fn get_assignment_count(&self) -> Result<i64> {
        let mut stmt = self.conn.prepare("SELECT COUNT(*) FROM assignments")?;
        stmt.next()?;
        let count = stmt.read::<i64>(0)?;
        Ok(count)
    }

    pub fn get_progression_count(&self) -> Result<i64> {
        let mut stmt = self.conn.prepare("SELECT COUNT(*) FROM progressions")?;
        stmt.next()?;
        let count = stmt.read::<i64>(0)?;
        Ok(count)
    }

    pub fn get_last_sync(&self) -> Result<Option<String>> {
        let mut stmt = self.conn.prepare("SELECT synced_at FROM sync_history ORDER BY synced_at DESC LIMIT 1")?;
        match stmt.next()? {
            sqlite::State::Row => {
                let ts = stmt.read::<i64>(0)?;
                Ok(Some(format!("Timestamp: {}", ts)))
            }
            sqlite::State::Done => Ok(None),
        }
    }
}
