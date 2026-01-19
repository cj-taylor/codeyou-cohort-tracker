use cohort_tracker::db::Database;
use tempfile::NamedTempFile;

#[test]
fn test_database_creation() {
    let temp_file = NamedTempFile::new().unwrap();
    let path = temp_file.path().to_str().unwrap();

    let db = Database::new(path).unwrap();

    // Should be able to get counts from empty database
    assert_eq!(db.get_student_count().unwrap(), 0);
    assert_eq!(db.get_assignment_count().unwrap(), 0);
    assert_eq!(db.get_progression_count().unwrap(), 0);
}

#[test]
fn test_student_operations() {
    let temp_file = NamedTempFile::new().unwrap();
    let path = temp_file.path().to_str().unwrap();
    let db = Database::new(path).unwrap();

    // Insert student with class_id
    db.insert_student("student1", "class1", "John", "Doe", "john@example.com")
        .unwrap();

    // get_student_count counts DISTINCT student IDs across all classes
    assert_eq!(db.get_student_count().unwrap(), 1);

    // Insert duplicate should not increase count (IGNORE)
    db.insert_student("student1", "class1", "John", "Doe", "john@example.com")
        .unwrap();
    assert_eq!(db.get_student_count().unwrap(), 1);

    // Insert same student in different class - still 1 unique student
    db.insert_student("student1", "class2", "John", "Doe", "john@example.com")
        .unwrap();
    assert_eq!(db.get_student_count().unwrap(), 1);

    // But class-specific count should be 1 for each class
    assert_eq!(db.get_student_count_by_class("class1").unwrap(), 1);
    assert_eq!(db.get_student_count_by_class("class2").unwrap(), 1);

    // Insert different student
    db.insert_student("student2", "class1", "Jane", "Smith", "jane@example.com")
        .unwrap();
    assert_eq!(db.get_student_count().unwrap(), 2);
    assert_eq!(db.get_student_count_by_class("class1").unwrap(), 2);
}

#[test]
fn test_assignment_type_stats() {
    let temp_file = NamedTempFile::new().unwrap();
    let path = temp_file.path().to_str().unwrap();
    let db = Database::new(path).unwrap();

    // Setup: Create students and assignments
    db.insert_student("s1", "class1", "John", "Doe", "john@example.com")
        .unwrap();
    db.insert_student("s2", "class1", "Jane", "Smith", "jane@example.com")
        .unwrap();

    db.insert_assignment("a1", "class1", "Lesson 1", "lesson", None)
        .unwrap();
    db.insert_assignment("a2", "class1", "Quiz 1", "quiz", None)
        .unwrap();
    db.insert_assignment("a3", "class1", "Lesson 2", "lesson", None)
        .unwrap();

    // Add progressions
    db.insert_progression(
        "p1",
        "class1",
        "s1",
        "a1",
        Some(0.9),
        "2024-01-01T09:00:00",
        "2024-01-01T10:00:00",
        None,
    )
    .unwrap();
    db.insert_progression(
        "p2",
        "class1",
        "s1",
        "a2",
        Some(0.8),
        "2024-01-02T09:00:00",
        "2024-01-02T10:00:00",
        None,
    )
    .unwrap();
    db.insert_progression(
        "p3",
        "class1",
        "s2",
        "a1",
        Some(0.95),
        "2024-01-01T10:00:00",
        "2024-01-01T11:00:00",
        None,
    )
    .unwrap();

    let stats = db.get_assignment_type_stats("class1", None).unwrap();

    // Should have 2 types: lesson and quiz
    assert_eq!(stats.len(), 2);

    // Find lesson stats
    let lesson_stats = stats
        .iter()
        .find(|s| s.assignment_type == "lesson")
        .unwrap();
    assert_eq!(lesson_stats.total_assignments, 2); // 2 lessons
    assert_eq!(lesson_stats.total_completions, 2); // s1 and s2 completed a1

    // Find quiz stats
    let quiz_stats = stats.iter().find(|s| s.assignment_type == "quiz").unwrap();
    assert_eq!(quiz_stats.total_assignments, 1); // 1 quiz
    assert_eq!(quiz_stats.total_completions, 1); // only s1 completed it
}

#[test]
fn test_grade_distribution() {
    let temp_file = NamedTempFile::new().unwrap();
    let path = temp_file.path().to_str().unwrap();
    let db = Database::new(path).unwrap();

    // Setup
    db.insert_student("s1", "class1", "John", "Doe", "john@example.com")
        .unwrap();
    db.insert_assignment("a1", "class1", "Test", "lesson", None)
        .unwrap();

    // Add progressions with various grades
    db.insert_progression(
        "p1",
        "class1",
        "s1",
        "a1",
        Some(0.45),
        "2024-01-01T09:00:00",
        "2024-01-01T10:00:00",
        None,
    )
    .unwrap();
    db.insert_progression(
        "p2",
        "class1",
        "s1",
        "a1",
        Some(0.65),
        "2024-01-02T09:00:00",
        "2024-01-02T10:00:00",
        None,
    )
    .unwrap();
    db.insert_progression(
        "p3",
        "class1",
        "s1",
        "a1",
        Some(0.85),
        "2024-01-03T09:00:00",
        "2024-01-03T10:00:00",
        None,
    )
    .unwrap();
    db.insert_progression(
        "p4",
        "class1",
        "s1",
        "a1",
        Some(0.95),
        "2024-01-04T09:00:00",
        "2024-01-04T10:00:00",
        None,
    )
    .unwrap();

    let distribution = db.get_grade_distribution("class1", None).unwrap();

    // Should have 6 buckets
    assert_eq!(distribution.len(), 6);

    // Check that we have grades in the right buckets
    let bucket_45 = distribution.iter().find(|d| d.range == "0-50%").unwrap();
    assert_eq!(bucket_45.count, 1);

    let bucket_65 = distribution.iter().find(|d| d.range == "60-70%").unwrap();
    assert_eq!(bucket_65.count, 1);
}

#[test]
fn test_velocity_stats() {
    let temp_file = NamedTempFile::new().unwrap();
    let path = temp_file.path().to_str().unwrap();
    let db = Database::new(path).unwrap();

    // Setup
    db.insert_student("s1", "class1", "John", "Doe", "john@example.com")
        .unwrap();
    db.insert_student("s2", "class1", "Jane", "Smith", "jane@example.com")
        .unwrap();
    db.insert_assignment("a1", "class1", "Test", "lesson", None)
        .unwrap();

    // Add progressions in same week
    db.insert_progression(
        "p1",
        "class1",
        "s1",
        "a1",
        Some(0.9),
        "2024-01-01T09:00:00",
        "2024-01-01T10:00:00",
        None,
    )
    .unwrap();
    db.insert_progression(
        "p2",
        "class1",
        "s2",
        "a1",
        Some(0.8),
        "2024-01-02T09:00:00",
        "2024-01-02T10:00:00",
        None,
    )
    .unwrap();
    db.insert_progression(
        "p3",
        "class1",
        "s1",
        "a1",
        Some(0.95),
        "2024-01-03T09:00:00",
        "2024-01-03T10:00:00",
        None,
    )
    .unwrap();

    let velocity = db.get_velocity_stats("class1", None).unwrap();

    // Should have at least one week
    assert!(!velocity.is_empty());

    let week_stats = &velocity[0];
    assert_eq!(week_stats.total_completions, 3);
    assert_eq!(week_stats.active_students, 2); // s1 and s2
    assert_eq!(week_stats.avg_completions_per_student, 1.5); // 3 completions / 2 students
}
