use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Class {
    pub id: String,
    pub name: String,
    pub friendly_id: String,
    pub is_active: bool,
    pub synced_at: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct Student {
    pub id: String,
    pub class_id: String,
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
    pub class_id: String,
    pub name: String,
    #[serde(rename = "type")]
    pub assignment_type: String,
    pub section: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ProgressionRecord {
    pub id: String,
    pub class_id: String,
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
    pub section: Option<String>,
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
    pub night: Option<String>,
    pub last_activity: Option<String>,
    pub days_inactive: Option<i64>,
    pub total_completions: i64,
    pub total_assignments: i64,
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

#[derive(Debug, Clone, Serialize)]
pub struct StudentDetail {
    pub id: String,
    pub first_name: String,
    pub last_name: String,
    pub email: String,
    pub region: Option<String>,
    pub night: Option<String>,
    pub total_assignments: i64,
    pub completed: i64,
    pub completion_pct: f64,
    pub avg_grade: Option<f64>,
    pub risk: String,
    pub last_activity: Option<String>,
    pub days_inactive: Option<i64>,
}

#[derive(Debug, Clone, Serialize)]
pub struct StudentAssignmentStatus {
    pub assignment_id: String,
    pub name: String,
    pub assignment_type: String,
    pub section: Option<String>,
    pub completed: bool,
    pub grade: Option<f64>,
    pub completed_at: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct StudentProgressPoint {
    pub week: String,
    pub completed: i64,
    pub cumulative: i64,
    pub avg_grade: Option<f64>,
}

#[derive(Debug, Clone, Serialize)]
pub struct DayOfWeekStats {
    pub day: String,
    pub count: i64,
}

#[derive(Debug, Clone, Serialize)]
pub struct SectionProgress {
    pub section: String,
    pub total_students: i64,
    pub students_started: i64,
    pub students_completed: i64,
}
