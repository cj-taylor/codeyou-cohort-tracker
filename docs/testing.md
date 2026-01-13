# Test Suite Documentation

## Overview

The Cohort Tracker project includes comprehensive unit and integration tests to ensure functionality and reliability.

## Test Structure

```
tests/
├── config_tests.rs      # Configuration management tests
├── db_tests.rs          # Database operations tests  
├── openclass_tests.rs   # API type serialization tests
├── sync_tests.rs        # HTTP client and sync tests
└── integration_tests.rs # End-to-end workflow tests
```

## Running Tests

### All Tests
```bash
cargo test
```

### Specific Test Files
```bash
cargo test --test config_tests
cargo test --test db_tests
```

### With Output
```bash
cargo test -- --nocapture
```

### Test Script
```bash
./run_tests.sh
```

## Test Coverage

### Unit Tests
- **Config Module**: Serialization, file I/O, error handling
- **Database Module**: CRUD operations, schema creation, data integrity
- **OpenClass Types**: JSON deserialization, type conversions

### Integration Tests  
- **Authentication Flow**: Login success/failure scenarios
- **Data Sync**: Full sync workflow with mock API
- **Error Handling**: Network failures, invalid responses
- **Data Persistence**: Database operations during sync

## Test Dependencies

- `tempfile`: Temporary files for database tests
- `wiremock`: HTTP mocking for API tests
- `serde_json`: JSON manipulation in tests

## Test Data

Tests use realistic but fake data:
- Student emails: `test@example.com`, `john@example.com`
- Class IDs: `class123`, `test-class`
- API tokens: `test-token-123`

## Continuous Integration

Tests are designed to run in CI environments:
- No external dependencies
- Temporary file cleanup
- Deterministic results
- Fast execution (< 30 seconds)

## Adding New Tests

1. Create test file in `tests/` directory
2. Import modules: `use cohort_tracker::module::Type;`
3. Use `#[test]` for unit tests, `#[tokio::test]` for async tests
4. Follow naming convention: `test_feature_scenario`

## Common Test Patterns

### Database Tests
```rust
let temp_file = NamedTempFile::new().unwrap();
let db = Database::new(temp_file.path().to_str().unwrap()).unwrap();
```

### Mock API Tests
```rust
let mock_server = MockServer::start().await;
Mock::given(method("POST"))
    .and(path("/endpoint"))
    .respond_with(ResponseTemplate::new(200).set_body_json(json!({...})))
    .mount(&mock_server)
    .await;
```

### Config Tests
```rust
let temp_file = NamedTempFile::new().unwrap();
config.save(temp_file.path().to_str().unwrap()).unwrap();
```

## Test Philosophy

- **Fast**: Tests run quickly for rapid feedback
- **Isolated**: Each test is independent 
- **Realistic**: Use real-world scenarios and data patterns
- **Comprehensive**: Cover happy path, edge cases, and error conditions
- **Maintainable**: Clear test names and minimal setup code
