use crate::config::Config;
use crate::openclass::{LoginRequest, LoginRequestAlt, ProgressionResponse};
use crate::db::Database;
use anyhow::{anyhow, Result};
use reqwest::Client;
use std::time::Duration;

pub struct OpenClassClient {
    client: Client,
    config: Config,
    token: Option<String>,
}

impl OpenClassClient {
    pub fn new(config: Config) -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(30))
            .build()
            .expect("Failed to create HTTP client");
        
        Self {
            client,
            config,
            token: None,
        }
    }

    pub async fn authenticate(&mut self) -> Result<()> {
        let url = format!("{}/v1/auth/login", self.config.api_base);
        println!("Attempting to authenticate with URL: {}", url);
        
        let form_data = format!(
            "email={}&password={}&invite_code=&instructor_invite_code=&mentor_invite_code=",
            urlencoding::encode(&self.config.email),
            urlencoding::encode(&self.config.password)
        );

        let response = self
            .client
            .post(&url)
            .header("Content-Type", "application/x-www-form-urlencoded")
            .header("Accept", "*/*")
            .header("Origin", "https://classroom.code-you.org")
            .header("X-OpenClass-App-Id", "38e8433f3fd003aa0f650125e9ff1e9427d476796e37803cea9942ff7cc31cd0")
            .body(form_data)
            .send()
            .await?;

        let status = response.status();
        println!("Response status: {}", status);
        
        if !status.is_success() {
            let error_text = response.text().await.unwrap_or_else(|_| "Unable to read response".to_string());
            println!("Error response body: {}", error_text);
            return Err(anyhow!(
                "Authentication failed: {} - {}",
                status,
                error_text
            ));
        }

        let text = response.text().await?;
        
        if let Ok(json) = serde_json::from_str::<serde_json::Value>(&text) {
            if let Some(result) = json.get("result") {
                if let Some(token) = result.get("token").and_then(|v| v.as_str()) {
                    self.token = Some(token.to_string());
                    println!("✓ Authentication successful");
                    return Ok(());
                }
            }
        }

        Err(anyhow!(
            "Could not extract token from authentication response"
        ))
    }

    pub async fn fetch_progressions(&self, page: i32) -> Result<ProgressionResponse> {
        let token = self
            .token
            .as_ref()
            .ok_or_else(|| anyhow!("Not authenticated"))?;

        let url = format!(
            "{}/v1/classes/{}/progressions?return_count=30&page={}&sort_by_completed_at=-1",
            self.config.api_base, self.config.class_id, page
        );

        let response = self
            .client
            .get(url)
            .header("bearer", token)
            .header("Content-Type", "application/json; charset=ISO-8859-1")
            .header("Accept", "*/*")
            .header("Origin", "https://classroom.code-you.org")
            .header("X-OpenClass-App-Id", "38e8433f3fd003aa0f650125e9ff1e9427d476796e37803cea9942ff7cc31cd0")
            .send()
            .await?;

        let status = response.status();
        if !status.is_success() {
            let error_text = response.text().await.unwrap_or_else(|_| "Unable to read response".to_string());
            return Err(anyhow!(
                "Failed to fetch progressions: {} - {}",
                status,
                error_text
            ));
        }

        let text = response.text().await?;
        
        // Parse the outer response
        let outer_json: serde_json::Value = serde_json::from_str(&text)?;
        
        // Extract the inner JSON string from result.objects[0]
        let inner_json_str = outer_json
            .get("result")
            .and_then(|r| r.get("objects"))
            .and_then(|o| o.as_array())
            .and_then(|arr| arr.get(0))
            .and_then(|s| s.as_str())
            .ok_or_else(|| anyhow!("Invalid response structure"))?;
        
        // Parse the inner JSON string to get the actual progressions data
        let body: ProgressionResponse = serde_json::from_str(inner_json_str)?;
        Ok(body)
    }

    pub async fn sync_all(&self, db: &Database) -> Result<SyncStats> {
        let mut stats = SyncStats {
            total_records: 0,
            students_inserted: 0,
            assignments_inserted: 0,
            progressions_inserted: 0,
            pages_fetched: 0,
        };

        let mut page = 0;
        let mut can_load_more = true;

        while can_load_more {
            println!("Fetching page {}...", page);

            let response = self.fetch_progressions(page).await?;
            can_load_more = response.metadata.can_load_more;
            let records_count = response.data.len() as i32;

            for progression in response.data {
                // Insert student
                db.insert_student(
                    &progression.user.id,
                    &progression.user.first_name,
                    &progression.user.last_name,
                    &progression.user.email,
                )?;
                stats.students_inserted += 1;

                // Insert assignment
                db.insert_assignment(
                    &progression.assignment.id,
                    &progression.assignment.name,
                    &progression.assignment.assignment_type,
                )?;
                stats.assignments_inserted += 1;

                // Extract progression ID (MongoDB ObjectId as string)
                let progression_id = match &progression.id {
                    serde_json::Value::Object(obj) => {
                        obj.get("$oid")
                            .and_then(|v| v.as_str())
                            .unwrap_or("unknown")
                            .to_string()
                    }
                    serde_json::Value::String(s) => s.clone(),
                    _ => "unknown".to_string(),
                };

                // Insert progression
                db.insert_progression(
                    &progression_id,
                    &progression.user.id,
                    &progression.assignment.id,
                    progression.grade,
                    &progression.started_assignment_at_rfc3339(),
                    &progression.completed_assignment_at_rfc3339(),
                    progression.reviewed_at_rfc3339().as_deref(),
                )?;
                stats.progressions_inserted += 1;

                stats.total_records += 1;
            }

            // Record sync page
            db.record_sync(&self.config.class_id, page, records_count)?;

            stats.pages_fetched += 1;
            page += 1;

            // Be nice to the API
            tokio::time::sleep(Duration::from_millis(500)).await;
        }

        Ok(stats)
    }
}

#[derive(Debug)]
pub struct SyncStats {
    pub total_records: i32,
    pub students_inserted: i32,
    pub assignments_inserted: i32,
    pub progressions_inserted: i32,
    pub pages_fetched: i32,
}
