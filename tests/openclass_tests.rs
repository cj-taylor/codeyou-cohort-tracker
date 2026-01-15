use cohort_tracker::lms::openclass::{
    OpenClassAssignment as Assignment, OpenClassProgression as Progression, ProgressionResponse,
    User,
};
use serde_json::json;

#[test]
fn test_progression_deserialization() {
    let json_data = json!({
        "_id": {"$oid": "507f1f77bcf86cd799439011"},
        "user": {
            "id": "user123",
            "first_name": "John",
            "last_name": "Doe",
            "email": "john@example.com"
        },
        "assignment": {
            "id": "assign123",
            "name": "Test Assignment",
            "type": "lesson"
        },
        "grade": 0.85,
        "started_assignment_at": "2025-01-01T10:00:00Z",
        "completed_assignment_at": "2025-01-01T11:00:00Z",
        "reviewed_at": null
    });

    let progression: Progression = serde_json::from_value(json_data).unwrap();

    assert_eq!(progression.user.id, "user123");
    assert_eq!(progression.user.first_name, "John");
    assert_eq!(progression.assignment.name, "Test Assignment");
    assert_eq!(progression.grade, Some(0.85));
    assert_eq!(progression.reviewed_at, None);
}

#[test]
fn test_progression_response_deserialization() {
    let json_data = json!({
        "metadata": {
            "total": 100,
            "page": 0,
            "results_per_page": 30,
            "can_load_more": true
        },
        "data": [{
            "_id": {"$oid": "507f1f77bcf86cd799439011"},
            "user": {
                "id": "user123",
                "first_name": "John",
                "last_name": "Doe",
                "email": "john@example.com"
            },
            "assignment": {
                "id": "assign123",
                "name": "Test Assignment",
                "type": "lesson"
            },
            "grade": 0.85,
            "started_assignment_at": "2025-01-01T10:00:00Z",
            "completed_assignment_at": "2025-01-01T11:00:00Z",
            "reviewed_at": null
        }]
    });

    let response: ProgressionResponse = serde_json::from_value(json_data).unwrap();

    assert_eq!(response.data.len(), 1);
    assert!(response.metadata.can_load_more);
    assert_eq!(response.data[0].user.first_name, "John");
}

#[test]
fn test_progression_without_grade() {
    let json_data = json!({
        "_id": {"$oid": "507f1f77bcf86cd799439011"},
        "user": {
            "id": "user123",
            "first_name": "John",
            "last_name": "Doe",
            "email": "john@example.com"
        },
        "assignment": {
            "id": "assign123",
            "name": "Test Assignment",
            "type": "lesson"
        },
        "started_assignment_at": "2025-01-01T10:00:00Z",
        "completed_assignment_at": "2025-01-01T11:00:00Z",
        "reviewed_at": null
    });

    let progression: Progression = serde_json::from_value(json_data).unwrap();
    assert_eq!(progression.grade, None);
}

#[test]
fn test_rfc3339_methods() {
    let progression = Progression {
        id: serde_json::Value::String("test".to_string()),
        user: User {
            id: "user123".to_string(),
            first_name: "John".to_string(),
            last_name: "Doe".to_string(),
            email: "john@example.com".to_string(),
        },
        assignment: Assignment {
            id: "assign123".to_string(),
            name: "Test Assignment".to_string(),
            assignment_type: "lesson".to_string(),
        },
        grade: Some(0.85),
        started_assignment_at: Some("2025-01-01T10:00:00Z".to_string()),
        completed_assignment_at: Some("2025-01-01T11:00:00Z".to_string()),
        reviewed_at: Some("2025-01-01T12:00:00Z".to_string()),
    };

    assert_eq!(
        progression.started_assignment_at_rfc3339(),
        "2025-01-01T10:00:00Z"
    );
    assert_eq!(
        progression.completed_assignment_at_rfc3339(),
        "2025-01-01T11:00:00Z"
    );
    assert_eq!(
        progression.reviewed_at_rfc3339(),
        Some("2025-01-01T12:00:00Z".to_string())
    );
}
