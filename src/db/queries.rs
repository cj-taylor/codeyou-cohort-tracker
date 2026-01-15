use super::Database;
use anyhow::{anyhow, Result};

// Import models from parent crate
use crate::models::*;

impl Database {
    // Class operations
    pub fn insert_class(&self, class: &Class) -> Result<()> {
        let stmt = self.conn.prepare(
            "INSERT OR REPLACE INTO classes (id, name, friendly_id, is_active, synced_at) VALUES (?, ?, ?, ?, ?)"
        )?;
        let stmt = stmt
            .bind(1, class.id.as_str())?
            .bind(2, class.name.as_str())?
            .bind(3, class.friendly_id.as_str())?
            .bind(4, if class.is_active { 1 } else { 0 })?;

        let stmt = match &class.synced_at {
            Some(s) => stmt.bind(5, s.as_str())?,
            None => stmt.bind(5, ())?,
        };

        let mut stmt = stmt;
        stmt.next()?;
        Ok(())
    }

    pub fn get_classes(&self) -> Result<Vec<Class>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, name, friendly_id, is_active, synced_at FROM classes ORDER BY name",
        )?;
        let mut classes = Vec::new();

        while let sqlite::State::Row = stmt.next()? {
            classes.push(Class {
                id: stmt.read::<String>(0)?,
                name: stmt.read::<String>(1)?,
                friendly_id: stmt.read::<String>(2)?,
                is_active: stmt.read::<i64>(3)? == 1,
                synced_at: stmt.read::<Option<String>>(4)?,
            });
        }

        Ok(classes)
    }

    pub fn get_active_classes(&self) -> Result<Vec<Class>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, name, friendly_id, is_active, synced_at FROM classes WHERE is_active = 1 ORDER BY name"
        )?;
        let mut classes = Vec::new();

        while let sqlite::State::Row = stmt.next()? {
            classes.push(Class {
                id: stmt.read::<String>(0)?,
                name: stmt.read::<String>(1)?,
                friendly_id: stmt.read::<String>(2)?,
                is_active: true,
                synced_at: stmt.read::<Option<String>>(4)?,
            });
        }

        Ok(classes)
    }

    pub fn get_class_by_friendly_id(&self, friendly_id: &str) -> Result<Class> {
        let stmt = self.conn.prepare(
            "SELECT id, name, friendly_id, is_active, synced_at FROM classes WHERE friendly_id = ?",
        )?;
        let mut stmt = stmt.bind(1, friendly_id)?;

        match stmt.next()? {
            sqlite::State::Row => Ok(Class {
                id: stmt.read::<String>(0)?,
                name: stmt.read::<String>(1)?,
                friendly_id: stmt.read::<String>(2)?,
                is_active: stmt.read::<i64>(3)? == 1,
                synced_at: stmt.read::<Option<String>>(4)?,
            }),
            sqlite::State::Done => Err(anyhow!("Class not found: {}", friendly_id)),
        }
    }

    pub fn set_class_active(&self, id: &str, is_active: bool) -> Result<()> {
        let stmt = self
            .conn
            .prepare("UPDATE classes SET is_active = ? WHERE id = ?")?;
        let mut stmt = stmt.bind(1, if is_active { 1 } else { 0 })?.bind(2, id)?;
        stmt.next()?;
        Ok(())
    }

    pub fn update_class_sync_time(&self, id: &str, synced_at: &str) -> Result<()> {
        let stmt = self
            .conn
            .prepare("UPDATE classes SET synced_at = ? WHERE id = ?")?;
        let mut stmt = stmt.bind(1, synced_at)?.bind(2, id)?;
        stmt.next()?;
        Ok(())
    }

    // Student operations
    pub fn insert_student(
        &self,
        id: &str,
        class_id: &str,
        first_name: &str,
        last_name: &str,
        email: &str,
    ) -> Result<()> {
        let stmt = self.conn.prepare(
            "INSERT OR IGNORE INTO students (id, class_id, first_name, last_name, email) VALUES (?, ?, ?, ?, ?)"
        )?;
        let mut stmt = stmt
            .bind(1, id)?
            .bind(2, class_id)?
            .bind(3, first_name)?
            .bind(4, last_name)?
            .bind(5, email)?;
        stmt.next()?;
        Ok(())
    }

    pub fn get_students_by_class(&self, class_id: &str) -> Result<Vec<Student>> {
        let stmt = self.conn.prepare("SELECT id, class_id, first_name, last_name, email, region, night FROM students WHERE class_id = ? ORDER BY last_name, first_name")?;
        let mut stmt = stmt.bind(1, class_id)?;
        let mut students = Vec::new();

        while let sqlite::State::Row = stmt.next()? {
            students.push(Student {
                id: stmt.read::<String>(0)?,
                class_id: stmt.read::<String>(1)?,
                first_name: stmt.read::<String>(2)?,
                last_name: stmt.read::<String>(3)?,
                email: stmt.read::<String>(4)?,
                region: stmt.read::<Option<String>>(5)?,
                night: stmt.read::<Option<String>>(6)?,
            });
        }

        Ok(students)
    }

    pub fn get_students_by_night(&self, class_id: &str, night: &str) -> Result<Vec<Student>> {
        let stmt = self.conn.prepare(
            "SELECT id, class_id, first_name, last_name, email, region, night FROM students WHERE class_id = ? AND LOWER(night) = LOWER(?) ORDER BY last_name, first_name"
        )?;
        let mut stmt = stmt.bind(1, class_id)?.bind(2, night)?;
        let mut students = Vec::new();

        while let sqlite::State::Row = stmt.next()? {
            students.push(Student {
                id: stmt.read::<String>(0)?,
                class_id: stmt.read::<String>(1)?,
                first_name: stmt.read::<String>(2)?,
                last_name: stmt.read::<String>(3)?,
                email: stmt.read::<String>(4)?,
                region: stmt.read::<Option<String>>(5)?,
                night: stmt.read::<Option<String>>(6)?,
            });
        }

        Ok(students)
    }

    pub fn update_student_night(
        &self,
        first_name: &str,
        last_name: &str,
        region: &str,
        night: &str,
    ) -> Result<bool> {
        let stmt = self.conn.prepare(
            "UPDATE students SET region = ?, night = ? WHERE LOWER(first_name) = LOWER(?) AND LOWER(last_name) = LOWER(?)"
        )?;
        let mut stmt = stmt
            .bind(1, region)?
            .bind(2, night)?
            .bind(3, first_name)?
            .bind(4, last_name)?;
        stmt.next()?;

        let mut check = self.conn.prepare("SELECT changes()")?;
        check.next()?;
        let changes = check.read::<i64>(0)?;
        Ok(changes > 0)
    }

    // Assignment operations
    pub fn insert_assignment(
        &self,
        id: &str,
        class_id: &str,
        name: &str,
        assignment_type: &str,
        section: Option<&str>,
    ) -> Result<()> {
        let stmt = self.conn.prepare(
            "INSERT OR REPLACE INTO assignments (id, class_id, name, type, section) VALUES (?, ?, ?, ?, ?)"
        )?;
        let stmt = stmt
            .bind(1, id)?
            .bind(2, class_id)?
            .bind(3, name)?
            .bind(4, assignment_type)?;

        let mut stmt = if let Some(s) = section {
            stmt.bind(5, s)?
        } else {
            stmt.bind(5, ())?
        };

        stmt.next()?;
        Ok(())
    }

    pub fn get_assignments_by_class(&self, class_id: &str) -> Result<Vec<Assignment>> {
        let stmt = self.conn.prepare("SELECT id, class_id, name, type, section FROM assignments WHERE class_id = ? ORDER BY section, name")?;
        let mut stmt = stmt.bind(1, class_id)?;
        let mut assignments = Vec::new();

        while let sqlite::State::Row = stmt.next()? {
            assignments.push(Assignment {
                id: stmt.read::<String>(0)?,
                class_id: stmt.read::<String>(1)?,
                name: stmt.read::<String>(2)?,
                assignment_type: stmt.read::<String>(3)?,
                section: stmt.read::<Option<String>>(4)?,
            });
        }

        Ok(assignments)
    }

    // Progression operations
    #[allow(clippy::too_many_arguments)]
    pub fn insert_progression(
        &self,
        id: &str,
        class_id: &str,
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
            (id, class_id, student_id, assignment_id, grade, started_at, completed_at, reviewed_at, synced_at)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)"
        )?;
        let stmt = stmt
            .bind(1, id)?
            .bind(2, class_id)?
            .bind(3, student_id)?
            .bind(4, assignment_id)?;
        let stmt = match grade {
            Some(g) => stmt.bind(5, g)?,
            None => stmt.bind(5, ())?,
        };
        let stmt = stmt.bind(6, started_at)?.bind(7, completed_at)?;
        let stmt = match reviewed_at {
            Some(r) => stmt.bind(8, r)?,
            None => stmt.bind(8, ())?,
        };
        let mut stmt = stmt.bind(9, now as i64)?;
        stmt.next()?;
        Ok(())
    }

    pub fn get_progressions_by_class(&self, class_id: &str) -> Result<Vec<ProgressionRecord>> {
        let stmt = self.conn.prepare(
            "SELECT id, class_id, student_id, assignment_id, grade, started_at, completed_at, reviewed_at, synced_at
             FROM progressions WHERE class_id = ? ORDER BY completed_at DESC"
        )?;
        let mut stmt = stmt.bind(1, class_id)?;
        let mut progressions = Vec::new();

        while let sqlite::State::Row = stmt.next()? {
            let grade: Option<f64> = stmt.read::<Option<f64>>(4)?;
            let reviewed_at: Option<String> = stmt.read::<Option<String>>(7)?;

            progressions.push(ProgressionRecord {
                id: stmt.read::<String>(0)?,
                class_id: stmt.read::<String>(1)?,
                student_id: stmt.read::<String>(2)?,
                assignment_id: stmt.read::<String>(3)?,
                grade,
                started_at: stmt.read::<String>(5)?,
                completed_at: stmt.read::<String>(6)?,
                reviewed_at,
                synced_at: stmt.read::<i64>(8)?,
            });
        }

        Ok(progressions)
    }

    pub fn get_progression_ids_by_class(
        &self,
        class_id: &str,
    ) -> Result<std::collections::HashSet<String>> {
        let stmt = self
            .conn
            .prepare("SELECT id FROM progressions WHERE class_id = ?")?;
        let mut stmt = stmt.bind(1, class_id)?;
        let mut ids = std::collections::HashSet::new();

        while let sqlite::State::Row = stmt.next()? {
            ids.insert(stmt.read::<String>(0)?);
        }

        Ok(ids)
    }

    // Mentor operations
    pub fn import_mentor(&self, name: &str, night: &str) -> Result<()> {
        let stmt = self
            .conn
            .prepare("INSERT INTO mentors (name, night) VALUES (?, ?)")?;
        let mut stmt = stmt.bind(1, name)?.bind(2, night)?;
        stmt.next()?;
        Ok(())
    }

    pub fn clear_mentors(&self) -> Result<()> {
        self.conn.execute("DELETE FROM mentors")?;
        Ok(())
    }

    pub fn get_all_mentors(&self) -> Result<Vec<Mentor>> {
        let mut stmt = self
            .conn
            .prepare("SELECT id, name, night FROM mentors ORDER BY night, name")?;
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

    // Count operations
    pub fn get_student_count(&self) -> Result<i64> {
        let mut stmt = self
            .conn
            .prepare("SELECT COUNT(DISTINCT id) FROM students")?;
        stmt.next()?;
        let count = stmt.read::<i64>(0)?;
        Ok(count)
    }

    pub fn get_student_count_by_class(&self, class_id: &str) -> Result<i64> {
        let stmt = self
            .conn
            .prepare("SELECT COUNT(*) FROM students WHERE class_id = ?")?;
        let mut stmt = stmt.bind(1, class_id)?;
        stmt.next()?;
        let count = stmt.read::<i64>(0)?;
        Ok(count)
    }

    pub fn get_assignment_count(&self) -> Result<i64> {
        let mut stmt = self
            .conn
            .prepare("SELECT COUNT(DISTINCT id) FROM assignments")?;
        stmt.next()?;
        let count = stmt.read::<i64>(0)?;
        Ok(count)
    }

    pub fn get_assignment_count_by_class(&self, class_id: &str) -> Result<i64> {
        let stmt = self
            .conn
            .prepare("SELECT COUNT(*) FROM assignments WHERE class_id = ?")?;
        let mut stmt = stmt.bind(1, class_id)?;
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

    pub fn get_progression_count_by_class(&self, class_id: &str) -> Result<i64> {
        let stmt = self
            .conn
            .prepare("SELECT COUNT(*) FROM progressions WHERE class_id = ?")?;
        let mut stmt = stmt.bind(1, class_id)?;
        stmt.next()?;
        let count = stmt.read::<i64>(0)?;
        Ok(count)
    }

    // Sync tracking
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

    pub fn get_last_sync_timestamp(&self) -> Result<Option<i64>> {
        let mut stmt = self
            .conn
            .prepare("SELECT synced_at FROM sync_history ORDER BY synced_at DESC LIMIT 1")?;
        match stmt.next()? {
            sqlite::State::Row => {
                let ts = stmt.read::<i64>(0)?;
                Ok(Some(ts))
            }
            sqlite::State::Done => Ok(None),
        }
    }
}
