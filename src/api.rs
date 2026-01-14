use anyhow::Result;
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    routing::get,
    Json, Router,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::Mutex;
use tower_http::cors::{Any, CorsLayer};
use tower_http::services::ServeDir;

use crate::config::Config;
#[allow(unused_imports)]
use crate::db::{
    Assignment, BlockerAssignment, Class, CompletionMetrics, Database, Mentor, NightSummary,
    ProgressSummary, ProgressionRecord, Student, StudentActivity, StudentAssignmentStatus,
    StudentDetail, StudentHealth, StudentProgressPoint, WeeklyProgress,
};

// Application state shared across all handlers
#[allow(dead_code)]
pub struct AppState {
    pub db: Mutex<Database>,
    pub config: Config,  // Reserved for future use (e.g., triggering sync)
}

// Response types
#[derive(Serialize)]
pub struct HealthResponse {
    pub status: String,
    pub last_sync: Option<i64>,
    pub students: i64,
    pub assignments: i64,
    pub progressions: i64,
}

#[derive(Serialize)]
pub struct ErrorResponse {
    pub error: String,
}

// Error handling for API
pub struct ApiError(anyhow::Error);

impl IntoResponse for ApiError {
    fn into_response(self) -> axum::response::Response {
        let body = Json(ErrorResponse {
            error: self.0.to_string(),
        });
        (StatusCode::INTERNAL_SERVER_ERROR, body).into_response()
    }
}

impl<E> From<E> for ApiError
where
    E: Into<anyhow::Error>,
{
    fn from(err: E) -> Self {
        Self(err.into())
    }
}

// Query parameters
#[derive(Debug, Deserialize)]
pub struct StudentActivityQuery {
    pub night: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ClassListQuery {
    pub all: Option<bool>,
}

// Handler functions
async fn health(State(state): State<Arc<AppState>>) -> Result<Json<HealthResponse>, ApiError> {
    let db = state.db.lock().await;

    let last_sync = db.get_last_sync_timestamp()?;
    let students = db.get_student_count()?;
    let assignments = db.get_assignment_count()?;
    let progressions = db.get_progression_count()?;

    Ok(Json(HealthResponse {
        status: "ok".to_string(),
        last_sync,
        students,
        assignments,
        progressions,
    }))
}

async fn list_classes(
    Query(query): Query<ClassListQuery>,
    State(state): State<Arc<AppState>>,
) -> Result<Json<Vec<Class>>, ApiError> {
    let db = state.db.lock().await;
    let classes = if query.all.unwrap_or(false) {
        db.get_classes()?
    } else {
        db.get_active_classes()?
    };
    Ok(Json(classes))
}

async fn list_students(
    Path(class_id): Path<String>,
    State(state): State<Arc<AppState>>,
) -> Result<Json<Vec<Student>>, ApiError> {
    let db = state.db.lock().await;
    let students = db.get_students_by_class(&class_id)?;
    Ok(Json(students))
}

async fn list_assignments(
    Path(class_id): Path<String>,
    State(state): State<Arc<AppState>>,
) -> Result<Json<Vec<Assignment>>, ApiError> {
    let db = state.db.lock().await;
    let assignments = db.get_assignments_by_class(&class_id)?;
    Ok(Json(assignments))
}

async fn list_progressions(
    Path(class_id): Path<String>,
    State(state): State<Arc<AppState>>,
) -> Result<Json<Vec<ProgressionRecord>>, ApiError> {
    let db = state.db.lock().await;
    let progressions = db.get_progressions_by_class(&class_id)?;
    Ok(Json(progressions))
}

async fn progress_summary(
    Path(class_id): Path<String>,
    State(state): State<Arc<AppState>>,
) -> Result<Json<ProgressSummary>, ApiError> {
    let db = state.db.lock().await;
    let summary = db.get_progress_summary(&class_id)?;
    Ok(Json(summary))
}

// Analytics handlers
async fn metrics_completion(
    Path(_class_id): Path<String>,
    State(state): State<Arc<AppState>>,
) -> Result<Json<CompletionMetrics>, ApiError> {
    let db = state.db.lock().await;
    let metrics = db.get_completion_metrics()?;
    Ok(Json(metrics))
}

async fn metrics_blockers(
    Path(class_id): Path<String>,
    State(state): State<Arc<AppState>>,
) -> Result<Json<Vec<BlockerAssignment>>, ApiError> {
    let db = state.db.lock().await;
    let blockers = db.get_blockers(&class_id, 10)?; // Top 10 blockers
    Ok(Json(blockers))
}

async fn metrics_student_health(
    Path(class_id): Path<String>,
    State(state): State<Arc<AppState>>,
) -> Result<Json<Vec<StudentHealth>>, ApiError> {
    let db = state.db.lock().await;
    let health = db.get_student_health(&class_id)?;
    Ok(Json(health))
}

async fn metrics_progress_over_time(
    Path(class_id): Path<String>,
    State(state): State<Arc<AppState>>,
) -> Result<Json<Vec<WeeklyProgress>>, ApiError> {
    let db = state.db.lock().await;
    let progress = db.get_progress_over_time(&class_id)?;
    Ok(Json(progress))
}

async fn metrics_student_activity(
    Path(class_id): Path<String>,
    Query(query): Query<StudentActivityQuery>,
    State(state): State<Arc<AppState>>,
) -> Result<Json<Vec<StudentActivity>>, ApiError> {
    let db = state.db.lock().await;
    let activity = db.get_student_activity_filtered(&class_id, query.night.as_deref())?;
    Ok(Json(activity))
}

async fn list_mentors(
    State(state): State<Arc<AppState>>,
) -> Result<Json<Vec<Mentor>>, ApiError> {
    let db = state.db.lock().await;
    let mentors = db.get_all_mentors()?;
    Ok(Json(mentors))
}

async fn metrics_night_summary(
    Path(class_id): Path<String>,
    State(state): State<Arc<AppState>>,
) -> Result<Json<Vec<NightSummary>>, ApiError> {
    let db = state.db.lock().await;
    let summary = db.get_night_summary(&class_id)?;
    Ok(Json(summary))
}

async fn students_by_night(
    Path((class_id, night)): Path<(String, String)>,
    State(state): State<Arc<AppState>>,
) -> Result<Json<Vec<Student>>, ApiError> {
    let db = state.db.lock().await;
    let students = db.get_students_by_night(&class_id, &night)?;
    Ok(Json(students))
}

async fn student_detail(
    Path((class_id, student_id)): Path<(String, String)>,
    State(state): State<Arc<AppState>>,
) -> Result<impl IntoResponse, ApiError> {
    let db = state.db.lock().await;
    match db.get_student_detail(&class_id, &student_id)? {
        Some(detail) => Ok((StatusCode::OK, Json(detail)).into_response()),
        None => Ok((
            StatusCode::NOT_FOUND,
            Json(ErrorResponse {
                error: "Student not found".to_string(),
            }),
        )
            .into_response()),
    }
}

async fn student_assignments(
    Path((class_id, student_id)): Path<(String, String)>,
    State(state): State<Arc<AppState>>,
) -> Result<Json<Vec<StudentAssignmentStatus>>, ApiError> {
    let db = state.db.lock().await;
    let assignments = db.get_student_assignments(&class_id, &student_id)?;
    Ok(Json(assignments))
}

async fn student_progress_timeline(
    Path((class_id, student_id)): Path<(String, String)>,
    State(state): State<Arc<AppState>>,
) -> Result<Json<Vec<StudentProgressPoint>>, ApiError> {
    let db = state.db.lock().await;
    let timeline = db.get_student_progress_timeline(&class_id, &student_id)?;
    Ok(Json(timeline))
}

// Build the router with all routes
fn create_router(state: Arc<AppState>) -> Router {
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    // Serve static files from the "static" directory
    let static_service = ServeDir::new("static").not_found_service(ServeDir::new("static"));

    Router::new()
        .route("/health", get(health))
        .route("/classes", get(list_classes))
        .route("/classes/:class_id/students", get(list_students))
        .route("/classes/:class_id/assignments", get(list_assignments))
        .route("/classes/:class_id/progressions", get(list_progressions))
        .route("/classes/:class_id/progress-summary", get(progress_summary))
        // Analytics endpoints
        .route("/classes/:class_id/metrics/completion", get(metrics_completion))
        .route("/classes/:class_id/metrics/blockers", get(metrics_blockers))
        .route("/classes/:class_id/metrics/student-health", get(metrics_student_health))
        .route("/classes/:class_id/metrics/progress-over-time", get(metrics_progress_over_time))
        .route("/classes/:class_id/metrics/student-activity", get(metrics_student_activity))
        .route("/classes/:class_id/metrics/night-summary", get(metrics_night_summary))
        .route("/classes/:class_id/students/night/:night", get(students_by_night))
        // Student detail endpoints
        .route("/classes/:class_id/students/:student_id/detail", get(student_detail))
        .route("/classes/:class_id/students/:student_id/assignments", get(student_assignments))
        .route("/classes/:class_id/students/:student_id/progress-timeline", get(student_progress_timeline))
        // Mentors
        .route("/mentors", get(list_mentors))
        // Dashboard (serve index.html at root)
        .nest_service("/dashboard", ServeDir::new("static"))
        .fallback_service(static_service)
        .layer(cors)
        .with_state(state)
}

// Start the API server
pub async fn start_server(config: Config, db_path: &str, port: u16) -> Result<()> {
    let db = Database::new(db_path)?;

    let state = Arc::new(AppState {
        db: Mutex::new(db),
        config,
    });

    let app = create_router(state);

    let addr = format!("0.0.0.0:{}", port);
    println!("Starting server on http://{}", addr);
    println!();
    println!("Available endpoints:");
    println!("  GET  /health");
    println!("  GET  /classes[?all=true]");
    println!("  GET  /classes/{{class_id}}/students");
    println!("  GET  /classes/{{class_id}}/assignments");
    println!("  GET  /classes/{{class_id}}/progressions");
    println!("  GET  /classes/{{class_id}}/progress-summary");
    println!();
    println!("Analytics endpoints:");
    println!("  GET  /classes/{{class_id}}/metrics/completion");
    println!("  GET  /classes/{{class_id}}/metrics/blockers");
    println!("  GET  /classes/{{class_id}}/metrics/student-health");
    println!("  GET  /classes/{{class_id}}/metrics/progress-over-time");
    println!("  GET  /classes/{{class_id}}/metrics/student-activity[?night=Tues]");
    println!("  GET  /classes/{{class_id}}/metrics/night-summary");
    println!("  GET  /classes/{{class_id}}/students/night/{{night}}");
    println!("  GET  /mentors");
    println!();
    println!("Student detail endpoints:");
    println!("  GET  /classes/{{class_id}}/students/{{student_id}}/detail");
    println!("  GET  /classes/{{class_id}}/students/{{student_id}}/assignments");
    println!("  GET  /classes/{{class_id}}/students/{{student_id}}/progress-timeline");
    println!();
    println!("Dashboard:");
    println!("  http://{}:{}/dashboard/", "localhost", port);
    println!();

    let listener = tokio::net::TcpListener::bind(&addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
