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

    // Insert student
    db.insert_student("student1", "John", "Doe", "john@example.com").unwrap();
    
    assert_eq!(db.get_student_count().unwrap(), 1);
    
    // Insert duplicate should not increase count (REPLACE)
    db.insert_student("student1", "John", "Doe", "john@example.com").unwrap();
    assert_eq!(db.get_student_count().unwrap(), 1);
    
    // Insert different student
    db.insert_student("student2", "Jane", "Smith", "jane@example.com").unwrap();
    assert_eq!(db.get_student_count().unwrap(), 2);
}
