use anyhow::Result;
use serde::Serialize;
use sqlite::Connection;

pub struct Database {
    conn: Connection,
}

#[derive(Debug, Clone, Serialize)]
pub struct Student {
    pub id: String,
    pub first_name: String,
    pub last_name: String,
    pub email: String,
    pub region: Option<String>,
    pub night: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct Mentor {
    pub id: i64,
    pub name: String,
    pub night: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct Assignment {
    pub id: String,
    pub name: String,
    #[serde(rename = "type")]
    pub assignment_type: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct ProgressionRecord {
    pub id: String,
    pub student_id: String,
    pub assignment_id: String,
    pub grade: Option<f64>,
    pub started_at: String,
    pub completed_at: String,
    pub reviewed_at: Option<String>,
    pub synced_at: i64,
}

#[derive(Debug, Clone, Serialize)]
pub struct ProgressSummary {
    pub total_students: i64,
    pub total_assignments: i64,
    pub total_progressions: i64,
    pub avg_grade: Option<f64>,
    pub completion_rate: f64,
}

// Analytics types
#[derive(Debug, Clone, Serialize)]
pub struct CompletionMetrics {
    pub total_assignments: i64,
    pub assignments_with_zero_completions: i64,
    pub avg_students_per_assignment: f64,
    pub assignments: Vec<AssignmentCompletion>,
}

#[derive(Debug, Clone, Serialize)]
pub struct AssignmentCompletion {
    pub assignment_id: String,
    pub name: String,
    pub assignment_type: String,
    pub completions: i64,
    pub completion_rate: f64,
    pub avg_grade: Option<f64>,
}

#[derive(Debug, Clone, Serialize)]
pub struct BlockerAssignment {
    pub assignment_id: String,
    pub name: String,
    pub completion_rate: f64,
    pub avg_grade: Option<f64>,
    pub completions: i64,
    pub total_students: i64,
}

#[derive(Debug, Clone, Serialize)]
pub struct StudentHealth {
    pub student_id: String,
    pub first_name: String,
    pub last_name: String,
    pub email: String,
    pub completed: i64,
    pub total_assignments: i64,
    pub completion_pct: f64,
    pub avg_grade: Option<f64>,
    pub risk: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct WeeklyProgress {
    pub week: String,
    pub completed: i64,
    pub cumulative: i64,
}

#[derive(Debug, Clone, Serialize)]
pub struct StudentActivity {
    pub student_id: String,
    pub first_name: String,
    pub last_name: String,
    pub email: String,
    pub last_activity: Option<String>,
    pub days_inactive: Option<i64>,
    pub total_completions: i64,
}

#[derive(Debug, Clone, Serialize)]
pub struct NightSummary {
    pub night: String,
    pub student_count: i64,
    pub total_completions: i64,
    pub avg_completion_pct: f64,
    pub avg_grade: Option<f64>,
    pub mentors: Vec<String>,
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
                email TEXT UNIQUE NOT NULL,
                region TEXT,
                night TEXT
            )",
        )?;

        // Add columns if they don't exist (for migration)
        let _ = conn.execute("ALTER TABLE students ADD COLUMN region TEXT");
        let _ = conn.execute("ALTER TABLE students ADD COLUMN night TEXT");

        conn.execute(
            "CREATE TABLE IF NOT EXISTS mentors (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                name TEXT NOT NULL,
                night TEXT NOT NULL
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

    pub fn get_last_sync_timestamp(&self) -> Result<Option<i64>> {
        let mut stmt = self.conn.prepare("SELECT synced_at FROM sync_history ORDER BY synced_at DESC LIMIT 1")?;
        match stmt.next()? {
            sqlite::State::Row => {
                let ts = stmt.read::<i64>(0)?;
                Ok(Some(ts))
            }
            sqlite::State::Done => Ok(None),
        }
    }

    pub fn get_all_students(&self) -> Result<Vec<Student>> {
        let mut stmt = self.conn.prepare("SELECT id, first_name, last_name, email, region, night FROM students ORDER BY last_name, first_name")?;
        let mut students = Vec::new();

        while let sqlite::State::Row = stmt.next()? {
            students.push(Student {
                id: stmt.read::<String>(0)?,
                first_name: stmt.read::<String>(1)?,
                last_name: stmt.read::<String>(2)?,
                email: stmt.read::<String>(3)?,
                region: stmt.read::<Option<String>>(4)?,
                night: stmt.read::<Option<String>>(5)?,
            });
        }

        Ok(students)
    }

    pub fn get_all_assignments(&self) -> Result<Vec<Assignment>> {
        let mut stmt = self.conn.prepare("SELECT id, name, type FROM assignments ORDER BY name")?;
        let mut assignments = Vec::new();

        while let sqlite::State::Row = stmt.next()? {
            assignments.push(Assignment {
                id: stmt.read::<String>(0)?,
                name: stmt.read::<String>(1)?,
                assignment_type: stmt.read::<String>(2)?,
            });
        }

        Ok(assignments)
    }

    pub fn get_all_progressions(&self) -> Result<Vec<ProgressionRecord>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, student_id, assignment_id, grade, started_at, completed_at, reviewed_at, synced_at
             FROM progressions ORDER BY completed_at DESC"
        )?;
        let mut progressions = Vec::new();

        while let sqlite::State::Row = stmt.next()? {
            let grade: Option<f64> = stmt.read::<Option<f64>>(3)?;
            let reviewed_at: Option<String> = stmt.read::<Option<String>>(6)?;

            progressions.push(ProgressionRecord {
                id: stmt.read::<String>(0)?,
                student_id: stmt.read::<String>(1)?,
                assignment_id: stmt.read::<String>(2)?,
                grade,
                started_at: stmt.read::<String>(4)?,
                completed_at: stmt.read::<String>(5)?,
                reviewed_at,
                synced_at: stmt.read::<i64>(7)?,
            });
        }

        Ok(progressions)
    }

    pub fn get_progress_summary(&self) -> Result<ProgressSummary> {
        let total_students = self.get_student_count()?;
        let total_assignments = self.get_assignment_count()?;
        let total_progressions = self.get_progression_count()?;

        // Calculate average grade
        let mut stmt = self.conn.prepare("SELECT AVG(grade) FROM progressions WHERE grade IS NOT NULL")?;
        let avg_grade = match stmt.next()? {
            sqlite::State::Row => stmt.read::<Option<f64>>(0)?,
            sqlite::State::Done => None,
        };

        // Calculate completion rate (progressions / (students * assignments))
        let expected_total = total_students * total_assignments;
        let completion_rate = if expected_total > 0 {
            total_progressions as f64 / expected_total as f64
        } else {
            0.0
        };

        Ok(ProgressSummary {
            total_students,
            total_assignments,
            total_progressions,
            avg_grade,
            completion_rate,
        })
    }

    // Analytics methods
    pub fn get_completion_metrics(&self) -> Result<CompletionMetrics> {
        let total_students = self.get_student_count()?;
        let total_assignments = self.get_assignment_count()?;

        // Get completion stats per assignment
        let mut stmt = self.conn.prepare(
            "SELECT a.id, a.name, a.type,
                    COUNT(p.id) as completions,
                    AVG(p.grade) as avg_grade
             FROM assignments a
             LEFT JOIN progressions p ON a.id = p.assignment_id
             GROUP BY a.id, a.name, a.type
             ORDER BY completions DESC"
        )?;

        let mut assignments = Vec::new();
        let mut zero_completions = 0i64;
        let mut total_completions = 0i64;

        while let sqlite::State::Row = stmt.next()? {
            let completions = stmt.read::<i64>(3)?;
            let avg_grade = stmt.read::<Option<f64>>(4)?;
            let completion_rate = if total_students > 0 {
                completions as f64 / total_students as f64
            } else {
                0.0
            };

            if completions == 0 {
                zero_completions += 1;
            }
            total_completions += completions;

            assignments.push(AssignmentCompletion {
                assignment_id: stmt.read::<String>(0)?,
                name: stmt.read::<String>(1)?,
                assignment_type: stmt.read::<String>(2)?,
                completions,
                completion_rate,
                avg_grade,
            });
        }

        let avg_students_per_assignment = if total_assignments > 0 {
            total_completions as f64 / total_assignments as f64
        } else {
            0.0
        };

        Ok(CompletionMetrics {
            total_assignments,
            assignments_with_zero_completions: zero_completions,
            avg_students_per_assignment,
            assignments,
        })
    }

    pub fn get_blockers(&self, limit: usize) -> Result<Vec<BlockerAssignment>> {
        let total_students = self.get_student_count()?;

        // Find assignments with lowest completion rates
        let stmt = self.conn.prepare(
            "SELECT a.id, a.name,
                    COUNT(p.id) as completions,
                    AVG(p.grade) as avg_grade
             FROM assignments a
             LEFT JOIN progressions p ON a.id = p.assignment_id
             GROUP BY a.id, a.name
             ORDER BY completions ASC, avg_grade ASC
             LIMIT ?"
        )?;
        let mut stmt = stmt.bind(1, limit as i64)?;

        let mut blockers = Vec::new();

        while let sqlite::State::Row = stmt.next()? {
            let completions = stmt.read::<i64>(2)?;
            let avg_grade = stmt.read::<Option<f64>>(3)?;
            let completion_rate = if total_students > 0 {
                completions as f64 / total_students as f64
            } else {
                0.0
            };

            blockers.push(BlockerAssignment {
                assignment_id: stmt.read::<String>(0)?,
                name: stmt.read::<String>(1)?,
                completion_rate,
                avg_grade,
                completions,
                total_students,
            });
        }

        Ok(blockers)
    }

    pub fn get_student_health(&self) -> Result<Vec<StudentHealth>> {
        let total_assignments = self.get_assignment_count()?;

        // Get completion stats per student
        let mut stmt = self.conn.prepare(
            "SELECT s.id, s.first_name, s.last_name, s.email,
                    COUNT(p.id) as completed,
                    AVG(p.grade) as avg_grade
             FROM students s
             LEFT JOIN progressions p ON s.id = p.student_id
             GROUP BY s.id, s.first_name, s.last_name, s.email
             ORDER BY completed ASC, avg_grade ASC"
        )?;

        let mut students = Vec::new();

        while let sqlite::State::Row = stmt.next()? {
            let completed = stmt.read::<i64>(4)?;
            let avg_grade = stmt.read::<Option<f64>>(5)?;
            let completion_pct = if total_assignments > 0 {
                completed as f64 / total_assignments as f64
            } else {
                0.0
            };

            // Determine risk level
            let risk = if completion_pct < 0.25 {
                "critical"
            } else if completion_pct < 0.50 {
                "high"
            } else if completion_pct < 0.75 {
                "medium"
            } else {
                "low"
            };

            students.push(StudentHealth {
                student_id: stmt.read::<String>(0)?,
                first_name: stmt.read::<String>(1)?,
                last_name: stmt.read::<String>(2)?,
                email: stmt.read::<String>(3)?,
                completed,
                total_assignments,
                completion_pct,
                avg_grade,
                risk: risk.to_string(),
            });
        }

        Ok(students)
    }

    pub fn get_progress_over_time(&self) -> Result<Vec<WeeklyProgress>> {
        // Group completions by week
        let mut stmt = self.conn.prepare(
            "SELECT strftime('%Y-%W', completed_at) as week,
                    COUNT(*) as completed
             FROM progressions
             WHERE completed_at IS NOT NULL AND completed_at != ''
             GROUP BY week
             ORDER BY week ASC"
        )?;

        let mut weekly = Vec::new();
        let mut cumulative = 0i64;

        while let sqlite::State::Row = stmt.next()? {
            let week = stmt.read::<String>(0)?;
            let completed = stmt.read::<i64>(1)?;
            cumulative += completed;

            weekly.push(WeeklyProgress {
                week,
                completed,
                cumulative,
            });
        }

        Ok(weekly)
    }

    pub fn get_student_activity(&self) -> Result<Vec<StudentActivity>> {
        // Get last activity date and total completions per student
        let mut stmt = self.conn.prepare(
            "SELECT s.id, s.first_name, s.last_name, s.email,
                    MAX(p.completed_at) as last_activity,
                    COUNT(p.id) as total_completions
             FROM students s
             LEFT JOIN progressions p ON s.id = p.student_id
             GROUP BY s.id, s.first_name, s.last_name, s.email
             ORDER BY last_activity ASC NULLS FIRST"
        )?;

        let mut activities = Vec::new();

        while let sqlite::State::Row = stmt.next()? {
            let last_activity: Option<String> = stmt.read::<Option<String>>(4)?;

            // Calculate days inactive
            let days_inactive = if let Some(ref date_str) = last_activity {
                // Parse ISO date and calculate days since
                self.calculate_days_since(date_str).ok()
            } else {
                None
            };

            activities.push(StudentActivity {
                student_id: stmt.read::<String>(0)?,
                first_name: stmt.read::<String>(1)?,
                last_name: stmt.read::<String>(2)?,
                email: stmt.read::<String>(3)?,
                last_activity,
                days_inactive,
                total_completions: stmt.read::<i64>(5)?,
            });
        }

        Ok(activities)
    }

    fn calculate_days_since(&self, date_str: &str) -> Result<i64> {
        use std::time::{SystemTime, UNIX_EPOCH};

        // Parse ISO 8601 date (e.g., "2025-01-04T15:30:00.000Z")
        let date_part = date_str.split('T').next().unwrap_or(date_str);
        let parts: Vec<&str> = date_part.split('-').collect();

        if parts.len() >= 3 {
            let year: i64 = parts[0].parse().unwrap_or(2025);
            let month: i64 = parts[1].parse().unwrap_or(1);
            let day: i64 = parts[2].parse().unwrap_or(1);

            // Approximate days since epoch for the date
            let date_days = (year - 1970) * 365 + (month - 1) * 30 + day;

            // Current days since epoch
            let now = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs() as i64;
            let now_days = now / 86400;

            Ok(now_days - date_days)
        } else {
            Ok(0)
        }
    }

    // Night/Region methods
    pub fn update_student_night(&self, first_name: &str, last_name: &str, region: &str, night: &str) -> Result<bool> {
        let stmt = self.conn.prepare(
            "UPDATE students SET region = ?, night = ? WHERE LOWER(first_name) = LOWER(?) AND LOWER(last_name) = LOWER(?)"
        )?;
        let mut stmt = stmt
            .bind(1, region)?
            .bind(2, night)?
            .bind(3, first_name)?
            .bind(4, last_name)?;
        stmt.next()?;

        // Check if any row was updated
        let mut check = self.conn.prepare("SELECT changes()")?;
        check.next()?;
        let changes = check.read::<i64>(0)?;
        Ok(changes > 0)
    }

    pub fn import_mentor(&self, name: &str, night: &str) -> Result<()> {
        let stmt = self.conn.prepare(
            "INSERT INTO mentors (name, night) VALUES (?, ?)"
        )?;
        let mut stmt = stmt
            .bind(1, name)?
            .bind(2, night)?;
        stmt.next()?;
        Ok(())
    }

    pub fn clear_mentors(&self) -> Result<()> {
        self.conn.execute("DELETE FROM mentors")?;
        Ok(())
    }

    pub fn get_all_mentors(&self) -> Result<Vec<Mentor>> {
        let mut stmt = self.conn.prepare("SELECT id, name, night FROM mentors ORDER BY night, name")?;
        let mut mentors = Vec::new();

        while let sqlite::State::Row = stmt.next()? {
            mentors.push(Mentor {
                id: stmt.read::<i64>(0)?,
                name: stmt.read::<String>(1)?,
                night: stmt.read::<String>(2)?,
            });
        }

        Ok(mentors)
    }

    pub fn get_students_by_night(&self, night: &str) -> Result<Vec<Student>> {
        let stmt = self.conn.prepare(
            "SELECT id, first_name, last_name, email, region, night FROM students WHERE LOWER(night) = LOWER(?) ORDER BY last_name, first_name"
        )?;
        let mut stmt = stmt.bind(1, night)?;
        let mut students = Vec::new();

        while let sqlite::State::Row = stmt.next()? {
            students.push(Student {
                id: stmt.read::<String>(0)?,
                first_name: stmt.read::<String>(1)?,
                last_name: stmt.read::<String>(2)?,
                email: stmt.read::<String>(3)?,
                region: stmt.read::<Option<String>>(4)?,
                night: stmt.read::<Option<String>>(5)?,
            });
        }

        Ok(students)
    }

    pub fn get_night_summary(&self) -> Result<Vec<NightSummary>> {
        let total_assignments = self.get_assignment_count()?;

        // Get stats grouped by night
        let mut stmt = self.conn.prepare(
            "SELECT s.night,
                    COUNT(DISTINCT s.id) as student_count,
                    COUNT(p.id) as total_completions,
                    AVG(p.grade) as avg_grade
             FROM students s
             LEFT JOIN progressions p ON s.id = p.student_id
             WHERE s.night IS NOT NULL
             GROUP BY s.night
             ORDER BY s.night"
        )?;

        let mut summaries = Vec::new();

        while let sqlite::State::Row = stmt.next()? {
            let night = stmt.read::<String>(0)?;
            let student_count = stmt.read::<i64>(1)?;
            let total_completions = stmt.read::<i64>(2)?;
            let avg_grade = stmt.read::<Option<f64>>(3)?;

            // Calculate average completion percentage
            let expected = student_count * total_assignments;
            let avg_completion_pct = if expected > 0 {
                total_completions as f64 / expected as f64
            } else {
                0.0
            };

            // Get mentors for this night
            let mentor_stmt = self.conn.prepare("SELECT name FROM mentors WHERE LOWER(night) = LOWER(?)")?;
            let mut mentor_stmt = mentor_stmt.bind(1, night.as_str())?;
            let mut mentors = Vec::new();
            while let sqlite::State::Row = mentor_stmt.next()? {
                mentors.push(mentor_stmt.read::<String>(0)?);
            }

            summaries.push(NightSummary {
                night,
                student_count,
                total_completions,
                avg_completion_pct,
                avg_grade,
                mentors,
            });
        }

        Ok(summaries)
    }

    #[allow(dead_code)]
    pub fn get_mentor_count(&self) -> Result<i64> {
        let mut stmt = self.conn.prepare("SELECT COUNT(*) FROM mentors")?;
        stmt.next()?;
        let count = stmt.read::<i64>(0)?;
        Ok(count)
    }
}
