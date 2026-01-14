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
    db.insert_student("student1", "class1", "John", "Doe", "john@example.com").unwrap();
    
    // get_student_count counts DISTINCT student IDs across all classes
    assert_eq!(db.get_student_count().unwrap(), 1);
    
    // Insert duplicate should not increase count (IGNORE)
    db.insert_student("student1", "class1", "John", "Doe", "john@example.com").unwrap();
    assert_eq!(db.get_student_count().unwrap(), 1);
    
    // Insert same student in different class - still 1 unique student
    db.insert_student("student1", "class2", "John", "Doe", "john@example.com").unwrap();
    assert_eq!(db.get_student_count().unwrap(), 1);
    
    // But class-specific count should be 1 for each class
    assert_eq!(db.get_student_count_by_class("class1").unwrap(), 1);
    assert_eq!(db.get_student_count_by_class("class2").unwrap(), 1);
    
    // Insert different student
    db.insert_student("student2", "class1", "Jane", "Smith", "jane@example.com").unwrap();
    assert_eq!(db.get_student_count().unwrap(), 2);
    assert_eq!(db.get_student_count_by_class("class1").unwrap(), 2);
}
