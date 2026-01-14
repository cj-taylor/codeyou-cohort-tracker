use crate::config::Config;
use crate::openclass::ProgressionResponse;
use crate::db::{Database, Class};
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
        
        // TODO: move these headers to a config or constant
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
            // FIXME: this should probably be configurable or at least not hardcoded
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

    pub async fn fetch_classes(&self) -> Result<Vec<Class>> {
        let token = self
            .token
            .as_ref()
            .ok_or_else(|| anyhow!("Not authenticated"))?;

        let url = format!("{}/v1/classes", self.config.api_base);

        let response = self
            .client
            .get(&url)
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
                "Failed to fetch classes: {} - {}",
                status,
                error_text
            ));
        }

        let text = response.text().await?;
        let outer_json: serde_json::Value = serde_json::from_str(&text)?;
        
        // Extract nested JSON string
        let inner_json_str = outer_json
            .get("result")
            .and_then(|r| r.get("objects"))
            .and_then(|o| o.as_str())
            .ok_or_else(|| anyhow!("Invalid classes response structure"))?;
        
        let inner_json: serde_json::Value = serde_json::from_str(inner_json_str)?;
        let classes_data = inner_json
            .get("data")
            .and_then(|d| d.as_array())
            .ok_or_else(|| anyhow!("No classes data found"))?;

        let mut classes = Vec::new();
        for class_obj in classes_data {
            classes.push(Class {
                id: class_obj.get("id").and_then(|v| v.as_str()).unwrap_or("").to_string(),
                name: class_obj.get("name").and_then(|v| v.as_str()).unwrap_or("").to_string(),
                friendly_id: class_obj.get("friendly_id").and_then(|v| v.as_str()).unwrap_or("").to_string(),
                is_active: false, // Will be set by user during init
                synced_at: None,
            });
        }

        Ok(classes)
    }

    pub async fn fetch_progressions(&self, class_id: &str, page: i32) -> Result<ProgressionResponse> {
        let token = self
            .token
            .as_ref()
            .ok_or_else(|| anyhow!("Not authenticated"))?;

        let url = format!(
            "{}/v1/classes/{}/progressions?return_count=30&page={}&sort_by_completed_at=-1",
            self.config.api_base, class_id, page
        );

        let response = self
            .client
            .get(url)
            .header("bearer", token)
            .header("Content-Type", "application/json; charset=ISO-8859-1")
            .header("Accept", "*/*")
            .header("Origin", "https://classroom.code-you.org")
            // TODO: same hardcoded app ID as above
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
        
        // FIXME: this API design is bonkers - they wrap JSON in JSON strings
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
        let active_classes = db.get_active_classes()?;
        
        if active_classes.is_empty() {
            println!("No active classes found. Run 'init' or 'activate' first.");
            return Ok(SyncStats::default());
        }

        println!("Found {} active class(es) to sync\n", active_classes.len());

        let mut total_stats = SyncStats::default();

        for (i, class) in active_classes.iter().enumerate() {
            println!("=== [{}/{}] Syncing: {} ===", i + 1, active_classes.len(), class.name);
            let class_stats = self.sync_class(&class.id, db).await?;
            total_stats.merge(class_stats);
            
            // Update sync timestamp
            let now = chrono::Utc::now().to_rfc3339();
            db.update_class_sync_time(&class.id, &now)?;
        }

        println!("\n=== All Classes Synced ===");
        println!("Total pages fetched: {}", total_stats.pages_fetched);
        println!("Total progressions: {}", total_stats.progressions_inserted);

        Ok(total_stats)
    }

    pub async fn sync_class(&self, class_id: &str, db: &Database) -> Result<SyncStats> {
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
            print!("Fetching page {}...", page);
            std::io::Write::flush(&mut std::io::stdout()).ok();

            let response = self.fetch_progressions(class_id, page).await?;
            can_load_more = response.metadata.can_load_more;
            let records_count = response.data.len();
            
            println!(" {} records", records_count);

            if records_count == 0 {
                println!("No more records to fetch.");
                break;
            }

            for progression in response.data {
                // Insert student with class_id
                db.insert_student(
                    &progression.user.id,
                    class_id,
                    &progression.user.first_name,
                    &progression.user.last_name,
                    &progression.user.email,
                )?;
                stats.students_inserted += 1;

                // Insert assignment with class_id
                db.insert_assignment(
                    &progression.assignment.id,
                    class_id,
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

                // Insert progression with class_id
                db.insert_progression(
                    &progression_id,
                    class_id,
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
            db.record_sync(class_id, page, records_count as i32)?;

            stats.pages_fetched += 1;
            page += 1;

            // TODO: make this configurable
            // Be nice to the API
            tokio::time::sleep(Duration::from_millis(500)).await;
        }

        println!("\n✓ Class sync complete:");
        println!("  Pages fetched: {}", stats.pages_fetched);
        println!("  Total records: {}", stats.total_records);
        println!("  Students: {} (unique)", db.get_student_count_by_class(class_id)?);
        println!("  Assignments: {} (unique)", db.get_assignment_count_by_class(class_id)?);
        println!("  Progressions: {}", stats.progressions_inserted);

        Ok(stats)
    }
}

#[derive(Debug, Default)]
pub struct SyncStats {
    pub total_records: i32,
    pub students_inserted: i32,
    pub assignments_inserted: i32,
    pub progressions_inserted: i32,
    pub pages_fetched: i32,
}

impl SyncStats {
    pub fn merge(&mut self, other: SyncStats) {
        self.total_records += other.total_records;
        self.students_inserted += other.students_inserted;
        self.assignments_inserted += other.assignments_inserted;
        self.progressions_inserted += other.progressions_inserted;
        self.pages_fetched += other.pages_fetched;
    }
}
