# OpenClass API Integration

This document explains how Cohort Tracker integrates with the OpenClass.ai API to fetch student progression data.

## Table of Contents

- [API Overview](#api-overview)
- [Authentication Flow](#authentication-flow)
- [Classes API](#classes-api)
- [Progressions API](#progressions-api)
- [Rust Type Definitions](#rust-type-definitions)
- [Pagination Handling](#pagination-handling)
- [Data Transformation](#data-transformation)
- [Error Handling](#error-handling)
- [Rate Limiting and Best Practices](#rate-limiting-and-best-practices)

## API Overview

OpenClass provides a REST API for accessing student data. We use three main endpoints:

1. **Authentication** - Get bearer token for API access
2. **Classes** - List all classes the user has access to
3. **Progressions** - Fetch student progress data with pagination

## Authentication Flow

### Login Endpoint

```http
POST https://api.openclass.ai/v1/auth/login
Content-Type: application/x-www-form-urlencoded

email=mentor@example.com&password=your-password&invite_code=&instructor_invite_code=&mentor_invite_code=
```

**Required Headers:**
- `Content-Type: application/x-www-form-urlencoded`
- `Accept: */*`
- `Origin: https://classroom.code-you.org`
- `X-OpenClass-App-Id: <app-id>` (hardcoded in source)

**Response:**
```json
{
  "result": {
    "objects": ["{\"first_name\": \"John\", \"last_name\": \"Doe\", \"email\": \"john@example.com\", \"license\": \"school\"}"],
    "token": "eyJhbGciOiJIUzUxMiIsImlhdCI6MTc2ODM0NjEyNywiZXhwIjoxNzcwOTM4MTI3fQ..."
  }
}
```

## Classes API

### List Classes Endpoint

```http
GET https://api.openclass.ai/v1/classes
Authorization: Bearer {token}
```

**Required Headers:**
- `bearer: {token}` (note: lowercase "bearer", not "Authorization")
- `Content-Type: application/json; charset=ISO-8859-1`
- `Accept: */*`
- `Origin: https://classroom.code-you.org`
- `X-OpenClass-App-Id: <app-id>` (hardcoded in source)

**Response Structure:**
```json
{
  "result": {
    "objects": "{\"metadata\": {\"total\": 5, \"page\": 0, \"results_per_page\": 999}, \"data\": [{\"_id\": {\"$oid\": \"6913dda091c226449a91e0d4\"}, \"id\": \"6913dda091c226449a91e0d4\", \"friendly_id\": \"data-analysis-pathway-module-2-aug-2\", \"name\": \"Data Analysis Pathway - Module 2 | AUG 25\", ...}]}"
  }
}
```

**Note:** The API wraps JSON data as a string inside `result.objects`. You must parse it twice:
1. Parse outer JSON to get `result.objects` string
2. Parse inner JSON string to get actual class data

**Class Object Fields:**
- `id` - Unique class identifier (MongoDB ObjectId)
- `friendly_id` - URL-friendly identifier (used in CLI)
- `name` - Display name of the class
- `is_published` - Whether class is published
- `school` - School identifier
- `assignments` - Array of assignment IDs
- `units` - Array of unit objects with assignments
- `mentors` - Array of mentor assignments
- `instructors` - Array of instructor IDs

### Implementation in Rust

```rust
pub async fn fetch_classes(&self) -> Result<Vec<Class>> {
    let token = self.token.as_ref().ok_or_else(|| anyhow!("Not authenticated"))?;

    let response = self.client
        .get(format!("{}/v1/classes", self.config.api_base))
        .header("bearer", token)
        .header("Content-Type", "application/json; charset=ISO-8859-1")
        .header("Accept", "*/*")
        .header("Origin", "https://classroom.code-you.org")
        .header("X-OpenClass-App-Id", OPENCLASS_APP_ID)
        .send()
        .await?;

    let text = response.text().await?;
    let outer_json: serde_json::Value = serde_json::from_str(&text)?;
    
    // Extract nested JSON string
    let inner_json_str = outer_json
        .get("result")
        .and_then(|r| r.get("objects"))
        .and_then(|o| o.as_str())
        .ok_or_else(|| anyhow!("Invalid response structure"))?;
    
    let inner_json: serde_json::Value = serde_json::from_str(inner_json_str)?;
    let classes_data = inner_json
        .get("data")
        .and_then(|d| d.as_array())
        .ok_or_else(|| anyhow!("No classes data found"))?;

    // Parse each class object
    let mut classes = Vec::new();
    for class_obj in classes_data {
        classes.push(Class {
            id: class_obj.get("id").and_then(|v| v.as_str()).unwrap_or("").to_string(),
            name: class_obj.get("name").and_then(|v| v.as_str()).unwrap_or("").to_string(),
            friendly_id: class_obj.get("friendly_id").and_then(|v| v.as_str()).unwrap_or("").to_string(),
            is_active: false,
            synced_at: None,
        });
    }

    Ok(classes)
}
```

### Implementation in Rust (Old - Incorrect)

```rust
#[derive(Serialize)]
struct LoginRequest {
    email: String,
    password: String,
}

#[derive(Deserialize)]
struct LoginResponse {
    token: String,
    user: User,
}

impl OpenClassClient {
    pub async fn authenticate(&mut self, email: &str, password: &str) -> Result<(), SyncError> {
        let login_request = LoginRequest {
            email: email.to_string(),
            password: password.to_string(),
        };
        
        let response = self.client
            .post(&format!("{}/v1/auth/login", self.base_url))
            .json(&login_request)
            .send()
            .await?;
            
        if response.status() == 401 {
            return Err(SyncError::AuthenticationFailed);
        }
        
        let login_response: LoginResponse = response.json().await?;
        self.token = Some(login_response.token);
        
        Ok(())
    }
}
```

## Progressions API

### Endpoint Details

```http
GET https://api.openclass.ai/v1/classes/{classId}/progressions?page=0&return_count=30
Authorization: Bearer {token}
```

**Parameters:**
- `classId` - The OpenClass class identifier
- `page` - Page number (0-based)
- `return_count` - Records per page (default: 30, max: 100)

### Response Structure

```json
{
  "metadata": {
    "total": 3781,
    "page": 0,
    "results_per_page": 30,
    "can_load_more": true
  },
  "data": [
    {
      "_id": { "$oid": "693b4d9ba039325fb0b89f92" },
      "user": {
        "id": "686d10387bbf0124aac02088",
        "first_name": "Jane",
        "last_name": "Doe", 
        "email": "jane.doe@example.com"
      },
      "assignment": {
        "id": "68e594f520442cbbe62a19c9",
        "name": "Bring It Into Focus",
        "type": "lesson"
      },
      "grade": 1,
      "started_assignment_at": "2025-12-11T23:02:51.781Z",
      "completed_assignment_at": "2025-12-11T23:02:51.781Z",
      "reviewed_at": null
    }
  ]
}
```

## Rust Type Definitions

### Core Response Types

```rust
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Debug)]
pub struct ProgressionResponse {
    pub metadata: Metadata,
    pub data: Vec<ProgressionData>,
}

#[derive(Deserialize, Debug)]
pub struct Metadata {
    pub total: u32,
    pub page: u32,
    pub results_per_page: u32,
    pub can_load_more: bool,
}

#[derive(Deserialize, Debug)]
pub struct ProgressionData {
    #[serde(rename = "_id")]
    pub id: ObjectId,
    pub user: User,
    pub assignment: Assignment,
    pub grade: Option<f64>,
    pub started_assignment_at: String,
    pub completed_assignment_at: String,
    pub reviewed_at: Option<String>,
}
```

### Nested Types

```rust
#[derive(Deserialize, Debug)]
pub struct ObjectId {
    #[serde(rename = "$oid")]
    pub oid: String,
}

#[derive(Deserialize, Debug)]
pub struct User {
    pub id: String,
    pub first_name: String,
    pub last_name: String,
    pub email: String,
}

#[derive(Deserialize, Debug)]
pub struct Assignment {
    pub id: String,
    pub name: String,
    #[serde(rename = "type")]
    pub assignment_type: String,  // "lesson" or "quiz"
}
```

## Pagination Handling

### Fetching All Pages

```rust
impl OpenClassClient {
    pub async fn fetch_all_progressions(&self, class_id: &str) -> Result<Vec<ProgressionData>, SyncError> {
        let mut all_progressions = Vec::new();
        let mut page = 0;
        
        loop {
            println!("Fetching page {}...", page);
            
            let response = self.fetch_progressions(class_id, page).await?;
            
            println!("Got {} progressions from page {}", response.data.len(), page);
            all_progressions.extend(response.data);
            
            if !response.metadata.can_load_more {
                break;
            }
            
            page += 1;
            
            // Rate limiting - be respectful to OpenClass
            tokio::time::sleep(Duration::from_millis(500)).await;
        }
        
        println!("Total progressions fetched: {}", all_progressions.len());
        Ok(all_progressions)
    }
    
    async fn fetch_progressions(&self, class_id: &str, page: u32) -> Result<ProgressionResponse, SyncError> {
        let token = self.token.as_ref().ok_or(SyncError::NotAuthenticated)?;
        
        let url = format!(
            "{}/v1/classes/{}/progressions?page={}&return_count=30",
            self.base_url, class_id, page
        );
        
        let response = self.client
            .get(&url)
            .bearer_auth(token)
            .send()
            .await?;
            
        if response.status() == 401 {
            return Err(SyncError::AuthenticationFailed);
        }
        
        if response.status() == 404 {
            return Err(SyncError::ClassNotFound(class_id.to_string()));
        }
        
        let progression_response: ProgressionResponse = response.json().await?;
        Ok(progression_response)
    }
}
```

## Data Transformation

### Converting API Data to Database Models

```rust
// Convert OpenClass API response to our database models
impl From<&ProgressionData> for crate::db::Student {
    fn from(data: &ProgressionData) -> Self {
        Self {
            id: data.user.id.clone(),
            first_name: data.user.first_name.clone(),
            last_name: data.user.last_name.clone(),
            email: data.user.email.clone(),
        }
    }
}

impl From<&ProgressionData> for crate::db::Assignment {
    fn from(data: &ProgressionData) -> Self {
        Self {
            id: data.assignment.id.clone(),
            name: data.assignment.name.clone(),
            assignment_type: data.assignment.assignment_type.clone(),
        }
    }
}

impl From<&ProgressionData> for crate::db::Progression {
    fn from(data: &ProgressionData) -> Self {
        Self {
            id: data.id.oid.clone(),
            student_id: data.user.id.clone(),
            assignment_id: data.assignment.id.clone(),
            grade: data.grade,
            started_at: data.started_assignment_at.clone(),
            completed_at: data.completed_assignment_at.clone(),
            reviewed_at: data.reviewed_at.clone(),
            synced_at: chrono::Utc::now().to_rfc3339(),
        }
    }
}
```

## Error Handling

### Custom Error Types

```rust
#[derive(Debug)]
pub enum SyncError {
    AuthenticationFailed,
    NotAuthenticated,
    ClassNotFound(String),
    NetworkError(reqwest::Error),
    JsonParseError(serde_json::Error),
    DatabaseError(rusqlite::Error),
}

impl std::fmt::Display for SyncError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            SyncError::AuthenticationFailed => write!(f, "Authentication failed - check email/password"),
            SyncError::NotAuthenticated => write!(f, "Not authenticated - call authenticate() first"),
            SyncError::ClassNotFound(id) => write!(f, "Class not found: {}", id),
            SyncError::NetworkError(e) => write!(f, "Network error: {}", e),
            SyncError::JsonParseError(e) => write!(f, "JSON parse error: {}", e),
            SyncError::DatabaseError(e) => write!(f, "Database error: {}", e),
        }
    }
}

impl std::error::Error for SyncError {}

// Error conversions
impl From<reqwest::Error> for SyncError {
    fn from(error: reqwest::Error) -> Self {
        SyncError::NetworkError(error)
    }
}

impl From<serde_json::Error> for SyncError {
    fn from(error: serde_json::Error) -> Self {
        SyncError::JsonParseError(error)
    }
}
```

## Rate Limiting and Best Practices

### Respectful API Usage

```rust
pub struct OpenClassClient {
    client: reqwest::Client,
    base_url: String,
    token: Option<String>,
    rate_limiter: RateLimiter,
}

impl OpenClassClient {
    pub fn new(base_url: &str) -> Self {
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(30))
            .user_agent("cohort-tracker/1.0")
            .build()
            .expect("Failed to create HTTP client");
            
        Self {
            client,
            base_url: base_url.to_string(),
            token: None,
            rate_limiter: RateLimiter::new(Duration::from_millis(500)),
        }
    }
    
    async fn make_request<T>(&self, request: reqwest::RequestBuilder) -> Result<T, SyncError> 
    where
        T: serde::de::DeserializeOwned,
    {
        // Wait for rate limiter
        self.rate_limiter.wait().await;
        
        let response = request.send().await?;
        
        // Handle common HTTP errors
        match response.status() {
            reqwest::StatusCode::OK => {},
            reqwest::StatusCode::UNAUTHORIZED => return Err(SyncError::AuthenticationFailed),
            reqwest::StatusCode::NOT_FOUND => return Err(SyncError::ClassNotFound("unknown".to_string())),
            reqwest::StatusCode::TOO_MANY_REQUESTS => {
                // Exponential backoff
                tokio::time::sleep(Duration::from_secs(5)).await;
                return Err(SyncError::NetworkError(reqwest::Error::from(response.error_for_status().unwrap_err())));
            },
            _ => return Err(SyncError::NetworkError(reqwest::Error::from(response.error_for_status().unwrap_err()))),
        }
        
        let data: T = response.json().await?;
        Ok(data)
    }
}
```

### Simple Rate Limiter

```rust
use std::time::{Duration, Instant};
use tokio::time::sleep;

pub struct RateLimiter {
    min_interval: Duration,
    last_request: Option<Instant>,
}

impl RateLimiter {
    pub fn new(min_interval: Duration) -> Self {
        Self {
            min_interval,
            last_request: None,
        }
    }
    
    pub async fn wait(&mut self) {
        if let Some(last) = self.last_request {
            let elapsed = last.elapsed();
            if elapsed < self.min_interval {
                let wait_time = self.min_interval - elapsed;
                sleep(wait_time).await;
            }
        }
        
        self.last_request = Some(Instant::now());
    }
}
```

## Testing API Integration

### Mock Server for Testing

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use wiremock::{MockServer, Mock, ResponseTemplate};
    use wiremock::matchers::{method, path, header};
    
    #[tokio::test]
    async fn test_authentication() {
        let mock_server = MockServer::start().await;
        
        Mock::given(method("POST"))
            .and(path("/v1/auth/login"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "token": "test-token-123",
                "user": {
                    "id": "user123",
                    "email": "test@example.com",
                    "role": "mentor"
                }
            })))
            .mount(&mock_server)
            .await;
            
        let mut client = OpenClassClient::new(&mock_server.uri());
        let result = client.authenticate("test@example.com", "password").await;
        
        assert!(result.is_ok());
        assert_eq!(client.token, Some("test-token-123".to_string()));
    }
    
    #[tokio::test]
    async fn test_fetch_progressions() {
        let mock_server = MockServer::start().await;
        
        Mock::given(method("GET"))
            .and(path("/v1/classes/test-class/progressions"))
            .and(header("authorization", "Bearer test-token"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "metadata": {
                    "total": 1,
                    "page": 0,
                    "results_per_page": 30,
                    "can_load_more": false
                },
                "data": [{
                    "_id": { "$oid": "test-progression-id" },
                    "user": {
                        "id": "test-user-id",
                        "first_name": "Test",
                        "last_name": "User",
                        "email": "test@example.com"
                    },
                    "assignment": {
                        "id": "test-assignment-id",
                        "name": "Test Assignment",
                        "type": "lesson"
                    },
                    "grade": 0.85,
                    "started_assignment_at": "2025-12-11T10:00:00.000Z",
                    "completed_assignment_at": "2025-12-11T10:30:00.000Z",
                    "reviewed_at": null
                }]
            })))
            .mount(&mock_server)
            .await;
            
        let mut client = OpenClassClient::new(&mock_server.uri());
        client.token = Some("test-token".to_string());
        
        let result = client.fetch_progressions("test-class", 0).await;
        assert!(result.is_ok());
        
        let response = result.unwrap();
        assert_eq!(response.data.len(), 1);
        assert_eq!(response.data[0].user.first_name, "Test");
    }
}
```

## Monitoring and Debugging

### Request Logging

```rust
use log::{info, warn, error, debug};

impl OpenClassClient {
    pub async fn fetch_progressions(&self, class_id: &str, page: u32) -> Result<ProgressionResponse, SyncError> {
        let token = self.token.as_ref().ok_or(SyncError::NotAuthenticated)?;
        
        let url = format!(
            "{}/v1/classes/{}/progressions?page={}&return_count=30",
            self.base_url, class_id, page
        );
        
        debug!("Making request to: {}", url);
        
        let start_time = std::time::Instant::now();
        let response = self.client
            .get(&url)
            .bearer_auth(token)
            .send()
            .await?;
            
        let duration = start_time.elapsed();
        info!("API request completed in {:?}", duration);
        
        if !response.status().is_success() {
            warn!("API request failed with status: {}", response.status());
        }
        
        let progression_response: ProgressionResponse = response.json().await?;
        
        info!("Fetched {} progressions from page {}", 
              progression_response.data.len(), page);
              
        Ok(progression_response)
    }
}
```

### Sync Statistics

```rust
pub struct SyncStats {
    pub total_progressions: usize,
    pub new_students: usize,
    pub new_assignments: usize,
    pub pages_fetched: u32,
    pub duration: Duration,
    pub errors: Vec<String>,
}

pub async fn sync_with_stats(config: &Config) -> Result<SyncStats, SyncError> {
    let start_time = Instant::now();
    let mut stats = SyncStats::default();
    
    // ... perform sync
    
    stats.duration = start_time.elapsed();
    Ok(stats)
}
```

## API Changes and Versioning

### Handling API Evolution

```rust
// Use optional fields for new API features
#[derive(Deserialize, Debug)]
pub struct ProgressionData {
    #[serde(rename = "_id")]
    pub id: ObjectId,
    pub user: User,
    pub assignment: Assignment,
    pub grade: Option<f64>,
    pub started_assignment_at: String,
    pub completed_assignment_at: String,
    pub reviewed_at: Option<String>,
    
    // New fields added in API v2 (optional for backward compatibility)
    #[serde(default)]
    pub difficulty_rating: Option<u8>,
    #[serde(default)]
    pub time_spent_minutes: Option<u32>,
}
```

### Version Detection

```rust
impl OpenClassClient {
    pub async fn get_api_version(&self) -> Result<String, SyncError> {
        let response = self.client
            .get(&format!("{}/v1/version", self.base_url))
            .send()
            .await?;
            
        #[derive(Deserialize)]
        struct VersionResponse {
            version: String,
        }
        
        let version_info: VersionResponse = response.json().await?;
        Ok(version_info.version)
    }
}
```

## Next Steps

- Read [Database Design](./database.md) for data storage details
- Check [Development Guide](./development.md) for contributing
- Review [Architecture](./architecture.md) for overall system design
