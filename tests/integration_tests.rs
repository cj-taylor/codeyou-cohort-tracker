use cohort_tracker::{config::Config, db::Database};
use tempfile::NamedTempFile;
use wiremock::MockServer;

#[tokio::test]
async fn test_config_and_database_integration() {
    // Simple integration test without network calls
    let temp_file = NamedTempFile::new().unwrap();
    let db_path = temp_file.path().to_str().unwrap();
    let db = Database::new(db_path).unwrap();

    // Test basic database operations with class_id
    db.insert_student("user123", "class1", "John", "Doe", "john@example.com")
        .unwrap();
    db.insert_assignment("assign123", "class1", "Test Assignment", "lesson", Some("Week 1"))
        .unwrap();
    db.insert_progression(
        "prog123",
        "class1",
        "user123",
        "assign123",
        Some(0.85),
        "2025-01-01T10:00:00Z",
        "2025-01-01T11:00:00Z",
        None,
    )
    .unwrap();

    // Verify data integrity
    assert_eq!(db.get_student_count().unwrap(), 1);
    assert_eq!(db.get_assignment_count().unwrap(), 1);
    assert_eq!(db.get_progression_count().unwrap(), 1);
}

#[tokio::test]
async fn test_config_with_mock_server() {
    let mock_server = MockServer::start().await;

    let config = Config {
        email: "test@example.com".to_string(),
        password: "password".to_string(),
        api_base: mock_server.uri(),
    };

    // Test config serialization
    let temp_file = NamedTempFile::new().unwrap();
    let path = temp_file.path().to_str().unwrap();

    config.save(path).unwrap();
    let loaded = Config::from_file(path).unwrap();

    assert_eq!(config.email, loaded.email);
    assert_eq!(config.api_base, loaded.api_base);
}
