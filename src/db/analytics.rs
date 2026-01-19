use super::Database;
use anyhow::Result;

// Import models from parent crate
use crate::models::*;

impl Database {
    pub fn get_progress_summary(&self, class_id: &str, night: Option<&str>) -> Result<ProgressSummary> {
        let total_students = if let Some(night_val) = night {
            let stmt = self.conn.prepare("SELECT COUNT(*) FROM students WHERE class_id = ? AND night = ?")?;
            let mut stmt = stmt.bind(1, class_id)?.bind(2, night_val)?;
            match stmt.next()? {
                sqlite::State::Row => stmt.read::<i64>(0)?,
                sqlite::State::Done => 0,
            }
        } else {
            self.get_student_count_by_class(class_id)?
        };
        
        let total_assignments = self.get_assignment_count_by_class(class_id)?;
        
        let total_progressions = if let Some(night_val) = night {
            let stmt = self.conn.prepare(
                "SELECT COUNT(*) FROM progressions p 
                 JOIN students s ON p.student_id = s.id AND p.class_id = s.class_id
                 WHERE p.class_id = ? AND s.night = ?"
            )?;
            let mut stmt = stmt.bind(1, class_id)?.bind(2, night_val)?;
            match stmt.next()? {
                sqlite::State::Row => stmt.read::<i64>(0)?,
                sqlite::State::Done => 0,
            }
        } else {
            self.get_progression_count_by_class(class_id)?
        };

        let avg_grade = if let Some(night_val) = night {
            let stmt = self.conn.prepare(
                "SELECT AVG(p.grade) FROM progressions p 
                 JOIN students s ON p.student_id = s.id AND p.class_id = s.class_id
                 WHERE p.grade IS NOT NULL AND p.class_id = ? AND s.night = ?"
            )?;
            let mut stmt = stmt.bind(1, class_id)?.bind(2, night_val)?;
            match stmt.next()? {
                sqlite::State::Row => stmt.read::<Option<f64>>(0)?,
                sqlite::State::Done => None,
            }
        } else {
            let stmt = self.conn.prepare(
                "SELECT AVG(grade) FROM progressions WHERE grade IS NOT NULL AND class_id = ?",
            )?;
            let mut stmt = stmt.bind(1, class_id)?;
            match stmt.next()? {
                sqlite::State::Row => stmt.read::<Option<f64>>(0)?,
                sqlite::State::Done => None,
            }
        };

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

    pub fn get_completion_metrics(&self) -> Result<CompletionMetrics> {
        let total_students = self.get_student_count()?;
        let total_assignments = self.get_assignment_count()?;

        let mut stmt = self.conn.prepare(
            "SELECT a.id, a.name, a.type,
                    COUNT(p.id) as completions,
                    AVG(p.grade) as avg_grade
             FROM assignments a
             LEFT JOIN progressions p ON a.id = p.assignment_id
             GROUP BY a.id, a.name, a.type
             ORDER BY completions DESC",
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

    pub fn get_blockers(&self, class_id: &str, limit: usize, night: Option<&str>) -> Result<Vec<BlockerAssignment>> {
        let total_students = if let Some(night_val) = night {
            let stmt = self.conn.prepare("SELECT COUNT(*) FROM students WHERE class_id = ? AND night = ?")?;
            let mut stmt = stmt.bind(1, class_id)?.bind(2, night_val)?;
            match stmt.next()? {
                sqlite::State::Row => stmt.read::<i64>(0)?,
                sqlite::State::Done => 0,
            }
        } else {
            self.get_student_count_by_class(class_id)?
        };

        let query = if night.is_some() {
            "SELECT a.id, a.name, a.section,
                    COUNT(CASE WHEN s.night = ? AND s.class_id = a.class_id THEN p.id END) as completions,
                    AVG(CASE WHEN s.night = ? AND s.class_id = a.class_id THEN p.grade END) as avg_grade
             FROM assignments a
             LEFT JOIN progressions p ON a.id = p.assignment_id AND a.class_id = p.class_id
             LEFT JOIN students s ON p.student_id = s.id
             WHERE a.class_id = ?
             GROUP BY a.id, a.name, a.section
             ORDER BY completions ASC, avg_grade ASC
             LIMIT ?"
        } else {
            "SELECT a.id, a.name, a.section,
                    COUNT(p.id) as completions,
                    AVG(p.grade) as avg_grade
             FROM assignments a
             LEFT JOIN progressions p ON a.id = p.assignment_id AND a.class_id = p.class_id
             WHERE a.class_id = ?
             GROUP BY a.id, a.name, a.section
             ORDER BY completions ASC, avg_grade ASC
             LIMIT ?"
        };

        let stmt = self.conn.prepare(query)?;
        let mut stmt = if let Some(night_val) = night {
            stmt.bind(1, night_val)?.bind(2, night_val)?.bind(3, class_id)?.bind(4, limit as i64)?
        } else {
            stmt.bind(1, class_id)?.bind(2, limit as i64)?
        };

        let mut blockers = Vec::new();

        while let sqlite::State::Row = stmt.next()? {
            let completions = stmt.read::<i64>(3)?;
            let avg_grade = stmt.read::<Option<f64>>(4)?;
            let completion_rate = if total_students > 0 {
                completions as f64 / total_students as f64
            } else {
                0.0
            };

            blockers.push(BlockerAssignment {
                assignment_id: stmt.read::<String>(0)?,
                name: stmt.read::<String>(1)?,
                section: stmt.read::<Option<String>>(2)?,
                completion_rate,
                avg_grade,
                completions,
                total_students,
            });
        }

        Ok(blockers)
    }

    pub fn get_student_health(&self, class_id: &str, night: Option<&str>) -> Result<Vec<StudentHealth>> {
        let total_assignments = self.get_assignment_count_by_class(class_id)?;

        let query = if night.is_some() {
            "SELECT s.id, s.first_name, s.last_name, s.email,
                    COUNT(p.id) as completed,
                    AVG(p.grade) as avg_grade
             FROM students s
             LEFT JOIN progressions p ON s.id = p.student_id AND s.class_id = p.class_id
             WHERE s.class_id = ? AND s.night = ?
             GROUP BY s.id, s.first_name, s.last_name, s.email
             ORDER BY completed ASC, avg_grade ASC"
        } else {
            "SELECT s.id, s.first_name, s.last_name, s.email,
                    COUNT(p.id) as completed,
                    AVG(p.grade) as avg_grade
             FROM students s
             LEFT JOIN progressions p ON s.id = p.student_id AND s.class_id = p.class_id
             WHERE s.class_id = ?
             GROUP BY s.id, s.first_name, s.last_name, s.email
             ORDER BY completed ASC, avg_grade ASC"
        };

        let stmt = self.conn.prepare(query)?;
        let mut stmt = if let Some(night_val) = night {
            stmt.bind(1, class_id)?.bind(2, night_val)?
        } else {
            stmt.bind(1, class_id)?
        };

        let mut students = Vec::new();

        while let sqlite::State::Row = stmt.next()? {
            let completed = stmt.read::<i64>(4)?;
            let avg_grade = stmt.read::<Option<f64>>(5)?;
            let completion_pct = if total_assignments > 0 {
                completed as f64 / total_assignments as f64
            } else {
                0.0
            };

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

    pub fn get_progress_over_time(&self, class_id: &str, night: Option<&str>) -> Result<Vec<WeeklyProgress>> {
        let query = if night.is_some() {
            "SELECT strftime('%Y-%W', p.completed_at) as week,
                    COUNT(*) as completed
             FROM progressions p
             JOIN students s ON p.student_id = s.id
             WHERE p.completed_at IS NOT NULL AND p.completed_at != '' AND p.class_id = ? AND s.night = ?
             GROUP BY week
             ORDER BY week ASC"
        } else {
            "SELECT strftime('%Y-%W', completed_at) as week,
                    COUNT(*) as completed
             FROM progressions
             WHERE completed_at IS NOT NULL AND completed_at != '' AND class_id = ?
             GROUP BY week
             ORDER BY week ASC"
        };

        let stmt = self.conn.prepare(query)?;
        let mut stmt = if let Some(night_val) = night {
            stmt.bind(1, class_id)?.bind(2, night_val)?
        } else {
            stmt.bind(1, class_id)?
        };

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

    pub fn get_student_activity(&self, class_id: &str) -> Result<Vec<StudentActivity>> {
        self.get_student_activity_filtered(class_id, None)
    }

    pub fn get_student_activity_filtered(
        &self,
        class_id: &str,
        night: Option<&str>,
    ) -> Result<Vec<StudentActivity>> {
        // First get total assignments for the class
        let total_assignments = self.get_assignment_count_by_class(class_id)?;

        let query = match night {
            Some(_) => {
                "SELECT s.id, s.first_name, s.last_name, s.email, s.night,
                        MAX(p.completed_at) as last_activity,
                        COUNT(p.id) as total_completions
                 FROM students s
                 LEFT JOIN progressions p ON s.id = p.student_id AND s.class_id = p.class_id
                 WHERE s.class_id = ? AND LOWER(s.night) = LOWER(?)
                 GROUP BY s.id, s.first_name, s.last_name, s.email, s.night
                 ORDER BY last_activity ASC NULLS FIRST"
            }
            None => {
                "SELECT s.id, s.first_name, s.last_name, s.email, s.night,
                        MAX(p.completed_at) as last_activity,
                        COUNT(p.id) as total_completions
                 FROM students s
                 LEFT JOIN progressions p ON s.id = p.student_id AND s.class_id = p.class_id
                 WHERE s.class_id = ?
                 GROUP BY s.id, s.first_name, s.last_name, s.email, s.night
                 ORDER BY last_activity ASC NULLS FIRST"
            }
        };

        let mut stmt = if let Some(n) = night {
            let stmt = self.conn.prepare(query)?;
            stmt.bind(1, class_id)?.bind(2, n)?
        } else {
            let stmt = self.conn.prepare(query)?;
            stmt.bind(1, class_id)?
        };

        let mut activities = Vec::new();

        while let sqlite::State::Row = stmt.next()? {
            let last_activity: Option<String> = stmt.read::<Option<String>>(5)?;

            let days_inactive = if let Some(ref date_str) = last_activity {
                self.calculate_days_since(date_str).ok()
            } else {
                None
            };

            activities.push(StudentActivity {
                student_id: stmt.read::<String>(0)?,
                first_name: stmt.read::<String>(1)?,
                last_name: stmt.read::<String>(2)?,
                email: stmt.read::<String>(3)?,
                night: stmt.read::<Option<String>>(4)?,
                last_activity,
                days_inactive,
                total_completions: stmt.read::<i64>(6)?,
                total_assignments,
            });
        }

        Ok(activities)
    }

    pub fn get_night_summary(&self, class_id: &str) -> Result<Vec<NightSummary>> {
        let total_assignments = self.get_assignment_count_by_class(class_id)?;

        let stmt = self.conn.prepare(
            "SELECT s.night,
                    COUNT(DISTINCT s.id) as student_count,
                    COUNT(p.id) as total_completions,
                    AVG(p.grade) as avg_grade
             FROM students s
             LEFT JOIN progressions p ON s.id = p.student_id AND s.class_id = p.class_id
             WHERE s.night IS NOT NULL AND s.class_id = ?
             GROUP BY s.night
             ORDER BY CASE s.night
                 WHEN 'Mon' THEN 1
                 WHEN 'Tues' THEN 2
                 WHEN 'Wed' THEN 3
                 WHEN 'Thurs' THEN 4
                 WHEN 'Fri' THEN 5
                 WHEN 'Sat' THEN 6
                 WHEN 'Sun' THEN 7
                 ELSE 8
             END",
        )?;
        let mut stmt = stmt.bind(1, class_id)?;

        let mut summaries = Vec::new();

        while let sqlite::State::Row = stmt.next()? {
            let night = stmt.read::<String>(0)?;
            let student_count = stmt.read::<i64>(1)?;
            let total_completions = stmt.read::<i64>(2)?;
            let avg_grade = stmt.read::<Option<f64>>(3)?;

            let expected = student_count * total_assignments;
            let avg_completion_pct = if expected > 0 {
                total_completions as f64 / expected as f64
            } else {
                0.0
            };

            let mentor_stmt = self
                .conn
                .prepare("SELECT name FROM mentors WHERE LOWER(night) = LOWER(?)")?;
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

    pub fn get_student_detail(
        &self,
        class_id: &str,
        student_id: &str,
    ) -> Result<Option<StudentDetail>> {
        let total_assignments = self.get_assignment_count_by_class(class_id)?;

        let stmt = self.conn.prepare(
            "SELECT s.id, s.first_name, s.last_name, s.email, s.region, s.night,
                    COUNT(p.id) as completed,
                    AVG(p.grade) as avg_grade,
                    MAX(p.completed_at) as last_activity
             FROM students s
             LEFT JOIN progressions p ON s.id = p.student_id AND s.class_id = p.class_id
             WHERE s.id = ? AND s.class_id = ?
             GROUP BY s.id, s.first_name, s.last_name, s.email, s.region, s.night",
        )?;
        let mut stmt = stmt.bind(1, student_id)?.bind(2, class_id)?;

        match stmt.next()? {
            sqlite::State::Row => {
                let completed = stmt.read::<i64>(6)?;
                let avg_grade = stmt.read::<Option<f64>>(7)?;
                let last_activity: Option<String> = stmt.read::<Option<String>>(8)?;

                let completion_pct = if total_assignments > 0 {
                    completed as f64 / total_assignments as f64
                } else {
                    0.0
                };

                let risk = if completion_pct < 0.25 {
                    "critical"
                } else if completion_pct < 0.50 {
                    "high"
                } else if completion_pct < 0.75 {
                    "medium"
                } else {
                    "low"
                };

                let days_inactive = if let Some(ref date_str) = last_activity {
                    self.calculate_days_since(date_str).ok()
                } else {
                    None
                };

                Ok(Some(StudentDetail {
                    id: stmt.read::<String>(0)?,
                    first_name: stmt.read::<String>(1)?,
                    last_name: stmt.read::<String>(2)?,
                    email: stmt.read::<String>(3)?,
                    region: stmt.read::<Option<String>>(4)?,
                    night: stmt.read::<Option<String>>(5)?,
                    total_assignments,
                    completed,
                    completion_pct,
                    avg_grade,
                    risk: risk.to_string(),
                    last_activity,
                    days_inactive,
                }))
            }
            sqlite::State::Done => Ok(None),
        }
    }

    pub fn get_student_assignments(
        &self,
        class_id: &str,
        student_id: &str,
    ) -> Result<Vec<StudentAssignmentStatus>> {
        let stmt = self.conn.prepare(
            "SELECT a.id, a.name, a.type, a.section,
                    p.grade, p.completed_at,
                    CASE WHEN p.id IS NOT NULL THEN 1 ELSE 0 END as completed
             FROM assignments a
             LEFT JOIN progressions p ON a.id = p.assignment_id AND a.class_id = p.class_id AND p.student_id = ?
             WHERE a.class_id = ?
             ORDER BY a.section, a.name"
        )?;
        let mut stmt = stmt.bind(1, student_id)?.bind(2, class_id)?;

        let mut assignments = Vec::new();

        while let sqlite::State::Row = stmt.next()? {
            let completed_flag = stmt.read::<i64>(6)?;
            assignments.push(StudentAssignmentStatus {
                assignment_id: stmt.read::<String>(0)?,
                name: stmt.read::<String>(1)?,
                assignment_type: stmt.read::<String>(2)?,
                section: stmt.read::<Option<String>>(3)?,
                completed: completed_flag == 1,
                grade: stmt.read::<Option<f64>>(4)?,
                completed_at: stmt.read::<Option<String>>(5)?,
            });
        }

        Ok(assignments)
    }

    pub fn get_student_progress_timeline(
        &self,
        class_id: &str,
        student_id: &str,
    ) -> Result<Vec<StudentProgressPoint>> {
        let stmt = self.conn.prepare(
            "SELECT strftime('%Y-%W', completed_at) as week,
                    COUNT(*) as completed,
                    AVG(grade) as avg_grade
             FROM progressions
             WHERE student_id = ? AND class_id = ? AND completed_at IS NOT NULL AND completed_at != ''
             GROUP BY week
             ORDER BY week ASC"
        )?;
        let mut stmt = stmt.bind(1, student_id)?.bind(2, class_id)?;

        let mut timeline = Vec::new();
        let mut cumulative = 0i64;

        while let sqlite::State::Row = stmt.next()? {
            let week = stmt.read::<String>(0)?;
            let completed = stmt.read::<i64>(1)?;
            let avg_grade = stmt.read::<Option<f64>>(2)?;
            cumulative += completed;

            timeline.push(StudentProgressPoint {
                week,
                completed,
                cumulative,
                avg_grade,
            });
        }

        Ok(timeline)
    }

    fn calculate_days_since(&self, date_str: &str) -> Result<i64> {
        use chrono::{NaiveDateTime, Utc};

        // Parse ISO 8601 datetime string
        let parsed = NaiveDateTime::parse_from_str(date_str, "%Y-%m-%dT%H:%M:%S")
            .or_else(|_| NaiveDateTime::parse_from_str(date_str, "%Y-%m-%d %H:%M:%S"))
            .or_else(|_| {
                // Try parsing just the date part
                let date_part = date_str.split('T').next().unwrap_or(date_str);
                chrono::NaiveDate::parse_from_str(date_part, "%Y-%m-%d")
                    .map(|d| d.and_hms_opt(0, 0, 0).unwrap())
            })?;

        let now = Utc::now().naive_utc();
        let duration = now.signed_duration_since(parsed);
        Ok(duration.num_days())
    }

    pub fn get_completions_by_day_of_week(&self, class_id: &str, night: Option<&str>) -> Result<Vec<DayOfWeekStats>> {
        let query = if night.is_some() {
            "SELECT 
                CASE CAST(strftime('%w', p.completed_at) AS INTEGER)
                    WHEN 0 THEN 'Sunday'
                    WHEN 1 THEN 'Monday'
                    WHEN 2 THEN 'Tuesday'
                    WHEN 3 THEN 'Wednesday'
                    WHEN 4 THEN 'Thursday'
                    WHEN 5 THEN 'Friday'
                    WHEN 6 THEN 'Saturday'
                END as day_name,
                CAST(strftime('%w', p.completed_at) AS INTEGER) as day_num,
                COUNT(*) as count
             FROM progressions p
             JOIN students s ON p.student_id = s.id
             WHERE p.class_id = ? AND s.night = ? AND p.completed_at IS NOT NULL AND p.completed_at != ''
             GROUP BY day_num
             ORDER BY day_num"
        } else {
            "SELECT 
                CASE CAST(strftime('%w', completed_at) AS INTEGER)
                    WHEN 0 THEN 'Sunday'
                    WHEN 1 THEN 'Monday'
                    WHEN 2 THEN 'Tuesday'
                    WHEN 3 THEN 'Wednesday'
                    WHEN 4 THEN 'Thursday'
                    WHEN 5 THEN 'Friday'
                    WHEN 6 THEN 'Saturday'
                END as day_name,
                CAST(strftime('%w', completed_at) AS INTEGER) as day_num,
                COUNT(*) as count
             FROM progressions
             WHERE class_id = ? AND completed_at IS NOT NULL AND completed_at != ''
             GROUP BY day_num
             ORDER BY day_num"
        };

        let stmt = self.conn.prepare(query)?;
        let mut stmt = if let Some(night_val) = night {
            stmt.bind(1, class_id)?.bind(2, night_val)?
        } else {
            stmt.bind(1, class_id)?
        };

        let mut results = Vec::new();
        while let sqlite::State::Row = stmt.next()? {
            results.push(DayOfWeekStats {
                day: stmt.read::<String>(0)?,
                count: stmt.read::<i64>(2)?,
            });
        }
        Ok(results)
    }

    pub fn get_student_completions_by_day_of_week(
        &self,
        class_id: &str,
        student_id: &str,
    ) -> Result<Vec<DayOfWeekStats>> {
        let stmt = self.conn.prepare(
            "SELECT 
                CASE CAST(strftime('%w', completed_at) AS INTEGER)
                    WHEN 0 THEN 'Sunday'
                    WHEN 1 THEN 'Monday'
                    WHEN 2 THEN 'Tuesday'
                    WHEN 3 THEN 'Wednesday'
                    WHEN 4 THEN 'Thursday'
                    WHEN 5 THEN 'Friday'
                    WHEN 6 THEN 'Saturday'
                END as day_name,
                CAST(strftime('%w', completed_at) AS INTEGER) as day_num,
                COUNT(*) as count
             FROM progressions
             WHERE class_id = ? AND student_id = ? AND completed_at IS NOT NULL AND completed_at != ''
             GROUP BY day_num
             ORDER BY day_num"
        )?;
        let mut stmt = stmt.bind(1, class_id)?.bind(2, student_id)?;

        let mut results = Vec::new();
        while let sqlite::State::Row = stmt.next()? {
            results.push(DayOfWeekStats {
                day: stmt.read::<String>(0)?,
                count: stmt.read::<i64>(2)?,
            });
        }
        Ok(results)
    }

    pub fn get_completions_by_time_of_day(&self, class_id: &str, night: Option<&str>) -> Result<Vec<DayOfWeekStats>> {
        let query = if night.is_some() {
            "SELECT 
                CASE 
                    WHEN CAST(strftime('%H', p.completed_at) AS INTEGER) BETWEEN 6 AND 11 THEN 'Morning (6am-12pm)'
                    WHEN CAST(strftime('%H', p.completed_at) AS INTEGER) BETWEEN 12 AND 17 THEN 'Afternoon (12pm-6pm)'
                    WHEN CAST(strftime('%H', p.completed_at) AS INTEGER) BETWEEN 18 AND 23 THEN 'Evening (6pm-12am)'
                    ELSE 'Night (12am-6am)'
                END as time_period,
                COUNT(*) as count
             FROM progressions p
             JOIN students s ON p.student_id = s.id
             WHERE p.class_id = ? AND s.night = ? AND p.completed_at IS NOT NULL AND p.completed_at != ''
             GROUP BY time_period
             ORDER BY 
                CASE time_period
                    WHEN 'Morning (6am-12pm)' THEN 1
                    WHEN 'Afternoon (12pm-6pm)' THEN 2
                    WHEN 'Evening (6pm-12am)' THEN 3
                    ELSE 4
                END"
        } else {
            "SELECT 
                CASE 
                    WHEN CAST(strftime('%H', completed_at) AS INTEGER) BETWEEN 6 AND 11 THEN 'Morning (6am-12pm)'
                    WHEN CAST(strftime('%H', completed_at) AS INTEGER) BETWEEN 12 AND 17 THEN 'Afternoon (12pm-6pm)'
                    WHEN CAST(strftime('%H', completed_at) AS INTEGER) BETWEEN 18 AND 23 THEN 'Evening (6pm-12am)'
                    ELSE 'Night (12am-6am)'
                END as time_period,
                COUNT(*) as count
             FROM progressions
             WHERE class_id = ? AND completed_at IS NOT NULL AND completed_at != ''
             GROUP BY time_period
             ORDER BY 
                CASE time_period
                    WHEN 'Morning (6am-12pm)' THEN 1
                    WHEN 'Afternoon (12pm-6pm)' THEN 2
                    WHEN 'Evening (6pm-12am)' THEN 3
                    ELSE 4
                END"
        };

        let stmt = self.conn.prepare(query)?;
        let mut stmt = if let Some(night_val) = night {
            stmt.bind(1, class_id)?.bind(2, night_val)?
        } else {
            stmt.bind(1, class_id)?
        };

        let mut results = Vec::new();
        while let sqlite::State::Row = stmt.next()? {
            results.push(DayOfWeekStats {
                day: stmt.read::<String>(0)?,
                count: stmt.read::<i64>(1)?,
            });
        }
        Ok(results)
    }

    pub fn get_student_completions_by_time_of_day(
        &self,
        class_id: &str,
        student_id: &str,
    ) -> Result<Vec<DayOfWeekStats>> {
        let stmt = self.conn.prepare(
            "SELECT 
                CASE 
                    WHEN CAST(strftime('%H', completed_at) AS INTEGER) BETWEEN 6 AND 11 THEN 'Morning (6am-12pm)'
                    WHEN CAST(strftime('%H', completed_at) AS INTEGER) BETWEEN 12 AND 17 THEN 'Afternoon (12pm-6pm)'
                    WHEN CAST(strftime('%H', completed_at) AS INTEGER) BETWEEN 18 AND 23 THEN 'Evening (6pm-12am)'
                    ELSE 'Night (12am-6am)'
                END as time_period,
                COUNT(*) as count
             FROM progressions
             WHERE class_id = ? AND student_id = ? AND completed_at IS NOT NULL AND completed_at != ''
             GROUP BY time_period
             ORDER BY 
                CASE time_period
                    WHEN 'Morning (6am-12pm)' THEN 1
                    WHEN 'Afternoon (12pm-6pm)' THEN 2
                    WHEN 'Evening (6pm-12am)' THEN 3
                    ELSE 4
                END"
        )?;
        let mut stmt = stmt.bind(1, class_id)?.bind(2, student_id)?;

        let mut results = Vec::new();
        while let sqlite::State::Row = stmt.next()? {
            results.push(DayOfWeekStats {
                day: stmt.read::<String>(0)?,
                count: stmt.read::<i64>(1)?,
            });
        }
        Ok(results)
    }

    pub fn get_assignment_type_stats(&self, class_id: &str, night: Option<&str>) -> Result<Vec<AssignmentTypeStats>> {
        let total_students = if let Some(night_val) = night {
            let stmt = self.conn.prepare("SELECT COUNT(*) FROM students WHERE class_id = ? AND night = ?")?;
            let mut stmt = stmt.bind(1, class_id)?.bind(2, night_val)?;
            match stmt.next()? {
                sqlite::State::Row => stmt.read::<i64>(0)?,
                sqlite::State::Done => 0,
            }
        } else {
            self.get_student_count_by_class(class_id)?
        };

        let query = if night.is_some() {
            "SELECT 
                a.type,
                COUNT(DISTINCT a.id) as total_assignments,
                COUNT(CASE WHEN s.night = ? AND s.class_id = a.class_id THEN p.id END) as total_completions,
                AVG(CASE WHEN s.night = ? AND s.class_id = a.class_id THEN p.grade END) as avg_grade
             FROM assignments a
             LEFT JOIN progressions p ON a.id = p.assignment_id AND a.class_id = p.class_id
             LEFT JOIN students s ON p.student_id = s.id
             WHERE a.class_id = ?
             GROUP BY a.type
             ORDER BY a.type"
        } else {
            "SELECT 
                a.type,
                COUNT(DISTINCT a.id) as total_assignments,
                COUNT(p.id) as total_completions,
                AVG(p.grade) as avg_grade
             FROM assignments a
             LEFT JOIN progressions p ON a.id = p.assignment_id AND a.class_id = p.class_id
             WHERE a.class_id = ?
             GROUP BY a.type
             ORDER BY a.type"
        };

        let stmt = self.conn.prepare(query)?;
        let mut stmt = if let Some(night_val) = night {
            stmt.bind(1, night_val)?.bind(2, night_val)?.bind(3, class_id)?
        } else {
            stmt.bind(1, class_id)?
        };

        let mut results = Vec::new();
        while let sqlite::State::Row = stmt.next()? {
            let assignment_type = stmt.read::<String>(0)?;
            let total_assignments = stmt.read::<i64>(1)?;
            let total_completions = stmt.read::<i64>(2)?;
            let avg_grade = stmt.read::<Option<f64>>(3)?;
            
            let expected_completions = total_students * total_assignments;
            let avg_completion_rate = if expected_completions > 0 {
                total_completions as f64 / expected_completions as f64
            } else {
                0.0
            };

            results.push(AssignmentTypeStats {
                assignment_type,
                total_assignments,
                avg_completion_rate,
                avg_grade,
                total_completions,
            });
        }
        Ok(results)
    }

    pub fn get_grade_distribution(&self, class_id: &str, night: Option<&str>) -> Result<Vec<GradeDistribution>> {
        let query = if night.is_some() {
            "SELECT p.grade
             FROM progressions p
             JOIN students s ON p.student_id = s.id AND p.class_id = s.class_id
             WHERE p.class_id = ? AND s.night = ? AND p.grade IS NOT NULL"
        } else {
            "SELECT grade
             FROM progressions
             WHERE class_id = ? AND grade IS NOT NULL"
        };

        let stmt = self.conn.prepare(query)?;
        let mut stmt = if let Some(night_val) = night {
            stmt.bind(1, class_id)?.bind(2, night_val)?
        } else {
            stmt.bind(1, class_id)?
        };

        // Collect all grades
        let mut grades = Vec::new();
        while let sqlite::State::Row = stmt.next()? {
            grades.push(stmt.read::<f64>(0)?);
        }

        let total = grades.len() as f64;
        if total == 0.0 {
            return Ok(Vec::new());
        }

        // Create grade buckets
        let ranges = vec![
            ("0-50%", 0.0, 0.5),
            ("50-60%", 0.5, 0.6),
            ("60-70%", 0.6, 0.7),
            ("70-80%", 0.7, 0.8),
            ("80-90%", 0.8, 0.9),
            ("90-100%", 0.9, 1.01), // 1.01 to include 1.0
        ];

        let mut distribution = Vec::new();
        for (label, min, max) in ranges {
            let count = grades.iter().filter(|&&g| g >= min && g < max).count() as i64;
            let percentage = (count as f64 / total) * 100.0;
            distribution.push(GradeDistribution {
                range: label.to_string(),
                count,
                percentage,
            });
        }

        Ok(distribution)
    }

    pub fn get_velocity_stats(&self, class_id: &str, night: Option<&str>) -> Result<Vec<VelocityStats>> {
        let query = if night.is_some() {
            "SELECT 
                strftime('%Y-%W', p.completed_at) as week,
                COUNT(*) as total_completions,
                COUNT(DISTINCT p.student_id) as active_students
             FROM progressions p
             JOIN students s ON p.student_id = s.id AND p.class_id = s.class_id
             WHERE p.completed_at IS NOT NULL AND p.completed_at != '' 
                   AND p.class_id = ? AND s.night = ?
             GROUP BY week
             ORDER BY week ASC"
        } else {
            "SELECT 
                strftime('%Y-%W', completed_at) as week,
                COUNT(*) as total_completions,
                COUNT(DISTINCT student_id) as active_students
             FROM progressions
             WHERE completed_at IS NOT NULL AND completed_at != '' AND class_id = ?
             GROUP BY week
             ORDER BY week ASC"
        };

        let stmt = self.conn.prepare(query)?;
        let mut stmt = if let Some(night_val) = night {
            stmt.bind(1, class_id)?.bind(2, night_val)?
        } else {
            stmt.bind(1, class_id)?
        };

        let mut results = Vec::new();
        while let sqlite::State::Row = stmt.next()? {
            let week = stmt.read::<String>(0)?;
            let total_completions = stmt.read::<i64>(1)?;
            let active_students = stmt.read::<i64>(2)?;
            
            let avg_completions_per_student = if active_students > 0 {
                total_completions as f64 / active_students as f64
            } else {
                0.0
            };

            results.push(VelocityStats {
                week,
                avg_completions_per_student,
                total_completions,
                active_students,
            });
        }

        Ok(results)
    }

    pub fn get_engagement_gaps(&self, class_id: &str, night: Option<&str>) -> Result<Vec<EngagementGap>> {
        let total_assignments = self.get_assignment_count_by_class(class_id)?;
        
        let query = if night.is_some() {
            "SELECT s.id, s.first_name, s.last_name, s.email, s.night,
                    MAX(p.completed_at) as last_activity,
                    COUNT(p.id) as completed
             FROM students s
             LEFT JOIN progressions p ON s.id = p.student_id AND s.class_id = p.class_id
             WHERE s.class_id = ? AND s.night = ?
             GROUP BY s.id, s.first_name, s.last_name, s.email, s.night"
        } else {
            "SELECT s.id, s.first_name, s.last_name, s.email, s.night,
                    MAX(p.completed_at) as last_activity,
                    COUNT(p.id) as completed
             FROM students s
             LEFT JOIN progressions p ON s.id = p.student_id AND s.class_id = p.class_id
             WHERE s.class_id = ?
             GROUP BY s.id, s.first_name, s.last_name, s.email, s.night"
        };

        let stmt = self.conn.prepare(query)?;
        let mut stmt = if let Some(night_val) = night {
            stmt.bind(1, class_id)?.bind(2, night_val)?
        } else {
            stmt.bind(1, class_id)?
        };

        let mut gaps = Vec::new();

        while let sqlite::State::Row = stmt.next()? {
            let last_activity: Option<String> = stmt.read::<Option<String>>(5)?;
            let completed = stmt.read::<i64>(6)?;
            let completion_pct = if total_assignments > 0 {
                completed as f64 / total_assignments as f64
            } else {
                0.0
            };

            // Only include students who are:
            // 1. Have >50% completion (not at-risk)
            // 2. Have been inactive for 7-14 days (engagement gap)
            if completion_pct > 0.5 {
                if let Some(ref date_str) = last_activity {
                    if let Ok(days_inactive) = self.calculate_days_since(date_str) {
                        if days_inactive >= 7 && days_inactive <= 14 {
                            gaps.push(EngagementGap {
                                student_id: stmt.read::<String>(0)?,
                                first_name: stmt.read::<String>(1)?,
                                last_name: stmt.read::<String>(2)?,
                                email: stmt.read::<String>(3)?,
                                night: stmt.read::<Option<String>>(4)?,
                                days_inactive,
                                completion_pct,
                                last_activity: last_activity.clone(),
                            });
                        }
                    }
                }
            }
        }

        // Sort by days inactive (most concerning first)
        gaps.sort_by(|a, b| b.days_inactive.cmp(&a.days_inactive));

        Ok(gaps)
    }

    pub fn get_assignment_difficulty(&self, class_id: &str, night: Option<&str>) -> Result<Vec<AssignmentDifficulty>> {
        let total_students = if let Some(night_val) = night {
            let stmt = self.conn.prepare("SELECT COUNT(*) FROM students WHERE class_id = ? AND night = ?")?;
            let mut stmt = stmt.bind(1, class_id)?.bind(2, night_val)?;
            match stmt.next()? {
                sqlite::State::Row => stmt.read::<i64>(0)?,
                sqlite::State::Done => 0,
            }
        } else {
            self.get_student_count_by_class(class_id)?
        };

        let query = if night.is_some() {
            "SELECT a.id, a.name, a.section, a.type,
                    COUNT(CASE WHEN s.night = ? AND s.class_id = a.class_id THEN p.id END) as completions,
                    AVG(CASE WHEN s.night = ? AND s.class_id = a.class_id THEN p.grade END) as avg_grade
             FROM assignments a
             LEFT JOIN progressions p ON a.id = p.assignment_id AND a.class_id = p.class_id
             LEFT JOIN students s ON p.student_id = s.id
             WHERE a.class_id = ?
             GROUP BY a.id, a.name, a.section, a.type"
        } else {
            "SELECT a.id, a.name, a.section, a.type,
                    COUNT(p.id) as completions,
                    AVG(p.grade) as avg_grade
             FROM assignments a
             LEFT JOIN progressions p ON a.id = p.assignment_id AND a.class_id = p.class_id
             WHERE a.class_id = ?
             GROUP BY a.id, a.name, a.section, a.type"
        };

        let stmt = self.conn.prepare(query)?;
        let mut stmt = if let Some(night_val) = night {
            stmt.bind(1, night_val)?.bind(2, night_val)?.bind(3, class_id)?
        } else {
            stmt.bind(1, class_id)?
        };

        let mut difficulties = Vec::new();

        while let sqlite::State::Row = stmt.next()? {
            let completions = stmt.read::<i64>(4)?;
            let avg_grade = stmt.read::<Option<f64>>(5)?;
            
            let completion_rate = if total_students > 0 {
                completions as f64 / total_students as f64
            } else {
                0.0
            };

            // Difficulty score: higher = more difficult
            // Formula: (1 - completion_rate) * 0.6 + (1 - avg_grade) * 0.4
            // Weights completion more heavily than grade
            let grade_component = if let Some(grade) = avg_grade {
                (1.0 - grade) * 0.4
            } else {
                0.4 // Assume difficult if no grades
            };
            let completion_component = (1.0 - completion_rate) * 0.6;
            let difficulty_score = completion_component + grade_component;

            difficulties.push(AssignmentDifficulty {
                assignment_id: stmt.read::<String>(0)?,
                name: stmt.read::<String>(1)?,
                section: stmt.read::<Option<String>>(2)?,
                assignment_type: stmt.read::<String>(3)?,
                difficulty_score,
                completion_rate,
                avg_grade,
                total_students,
                completions,
            });
        }

        // Sort by difficulty score (highest first)
        difficulties.sort_by(|a, b| b.difficulty_score.partial_cmp(&a.difficulty_score).unwrap());

        Ok(difficulties)
    }

    pub fn get_section_progress(&self, class_id: &str, night: Option<&str>) -> Result<Vec<SectionProgress>> {
        let query = if night.is_some() {
            "SELECT 
                a.section,
                COUNT(DISTINCT s.id) as total_students,
                COUNT(DISTINCT CASE WHEN p.id IS NOT NULL THEN s.id END) as students_started,
                COUNT(DISTINCT CASE WHEN p.id IS NOT NULL AND p.grade >= 0.7 THEN s.id END) as students_completed
             FROM assignments a
             CROSS JOIN students s ON s.class_id = ? AND s.night = ?
             LEFT JOIN progressions p ON p.assignment_id = a.id AND p.student_id = s.id AND p.class_id = a.class_id
             WHERE a.class_id = ? AND a.section IS NOT NULL AND a.section != ''
             GROUP BY a.section
             ORDER BY a.section"
        } else {
            "SELECT 
                a.section,
                COUNT(DISTINCT s.id) as total_students,
                COUNT(DISTINCT CASE WHEN p.id IS NOT NULL THEN s.id END) as students_started,
                COUNT(DISTINCT CASE WHEN p.id IS NOT NULL AND p.grade >= 0.7 THEN s.id END) as students_completed
             FROM assignments a
             CROSS JOIN students s ON s.class_id = ?
             LEFT JOIN progressions p ON p.assignment_id = a.id AND p.student_id = s.id AND p.class_id = a.class_id
             WHERE a.class_id = ? AND a.section IS NOT NULL AND a.section != ''
             GROUP BY a.section
             ORDER BY a.section"
        };

        let stmt = self.conn.prepare(query)?;
        let mut stmt = if let Some(night_val) = night {
            stmt.bind(1, class_id)?.bind(2, night_val)?.bind(3, class_id)?
        } else {
            stmt.bind(1, class_id)?.bind(2, class_id)?
        };

        let mut results = Vec::new();
        while let sqlite::State::Row = stmt.next()? {
            results.push(SectionProgress {
                section: stmt.read::<String>(0)?,
                total_students: stmt.read::<i64>(1)?,
                students_started: stmt.read::<i64>(2)?,
                students_completed: stmt.read::<i64>(3)?,
            });
        }
        Ok(results)
    }
}
