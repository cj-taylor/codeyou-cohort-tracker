use cohort_tracker::{config::Config, sync::OpenClassClient};
use serde_json::json;
use wiremock::{
    matchers::{header, method, path},
    Mock, MockServer, ResponseTemplate,
};

#[tokio::test]
async fn test_authentication_success() {
    let mock_server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/v1/auth/login"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "result": {
                "token": "test-token-123"
            }
        })))
        .mount(&mock_server)
        .await;

    let config = Config {
        email: "test@example.com".to_string(),
        password: "password".to_string(),
        api_base: mock_server.uri(),
    };

    let mut client = OpenClassClient::new(config);
    let result = client.authenticate().await;

    assert!(result.is_ok());
}

#[tokio::test]
async fn test_authentication_failure() {
    let mock_server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/v1/auth/login"))
        .respond_with(ResponseTemplate::new(401).set_body_string("Unauthorized"))
        .mount(&mock_server)
        .await;

    let config = Config {
        email: "wrong@example.com".to_string(),
        password: "wrongpass".to_string(),
        api_base: mock_server.uri(),
    };

    let mut client = OpenClassClient::new(config);
    let result = client.authenticate().await;

    assert!(result.is_err());
}

#[tokio::test]
async fn test_fetch_progressions() {
    let mock_server = MockServer::start().await;

    // Mock auth endpoint
    Mock::given(method("POST"))
        .and(path("/v1/auth/login"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "result": {
                "token": "test-token-123"
            }
        })))
        .mount(&mock_server)
        .await;

    // Mock progressions endpoint with the weird nested JSON format
    let progressions_data = json!({
        "metadata": {
            "total": 1,
            "page": 0,
            "results_per_page": 30,
            "can_load_more": false
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

    Mock::given(method("GET"))
        .and(path("/v1/classes/class123/progressions"))
        .and(header("bearer", "test-token-123"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "result": {
                "objects": [serde_json::to_string(&progressions_data).unwrap()]
            }
        })))
        .mount(&mock_server)
        .await;

    let config = Config {
        email: "test@example.com".to_string(),
        password: "password".to_string(),
        api_base: mock_server.uri(),
    };

    let mut client = OpenClassClient::new(config);
    client.authenticate().await.unwrap();

    let result = client.fetch_progressions("class123", 0).await;
    assert!(result.is_ok());

    let response = result.unwrap();
    assert_eq!(response.data.len(), 1);
    assert_eq!(response.data[0].user.first_name, "John");
    assert!(!response.metadata.can_load_more);
}
