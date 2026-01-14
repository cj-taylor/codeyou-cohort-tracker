use anyhow::{anyhow, Result};
use super::OpenClassProvider;
use crate::models::Class;
use super::types::ProgressionResponse;

impl OpenClassProvider {
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
                is_active: false,
                synced_at: None,
            });
        }

        Ok(classes)
    }

    pub async fn fetch_class_details(&self, class_id: &str) -> Result<std::collections::HashMap<String, String>> {
        let token = self.token.as_ref().ok_or_else(|| anyhow!("Not authenticated"))?;

        let url = format!("{}/v1/classes/{}", self.config.api_base, class_id);

        let response = self.client
            .get(&url)
            .header("bearer", token)
            .header("Content-Type", "application/json; charset=ISO-8859-1")
            .header("Accept", "*/*")
            .header("Origin", "https://classroom.code-you.org")
            .header("X-OpenClass-App-Id", "38e8433f3fd003aa0f650125e9ff1e9427d476796e37803cea9942ff7cc31cd0")
            .send()
            .await?;

        let text = response.text().await?;
        
        let outer_json: serde_json::Value = serde_json::from_str(&text)
            .map_err(|e| anyhow!("Failed to parse class details JSON: {}", e))?;
        
        let class_data: serde_json::Value = if let Some(result) = outer_json.get("result") {
            if let Some(objects_array) = result.get("objects").and_then(|o| o.as_array()) {
                if let Some(objects_str) = objects_array.get(0).and_then(|s| s.as_str()) {
                    let nested: serde_json::Value = serde_json::from_str(objects_str)
                        .map_err(|e| anyhow!("Failed to parse nested class data: {}", e))?;
                    
                    if let Some(data_array) = nested.get("data").and_then(|d| d.as_array()) {
                        if let Some(class_obj) = data_array.get(0) {
                            class_obj.clone()
                        } else {
                            return Ok(std::collections::HashMap::new());
                        }
                    } else {
                        return Ok(std::collections::HashMap::new());
                    }
                } else {
                    return Ok(std::collections::HashMap::new());
                }
            } else {
                return Ok(std::collections::HashMap::new());
            }
        } else {
            return Ok(std::collections::HashMap::new());
        };
        
        let mut assignment_sections = std::collections::HashMap::new();
        
        if let Some(units) = class_data.get("units").and_then(|u| u.as_array()) {
            println!("Found {} units", units.len());
            for unit in units {
                let section_name = unit.get("name").and_then(|n| n.as_str()).unwrap_or("Unknown Section");
                
                if let Some(assignments) = unit.get("assignments").and_then(|a| a.as_array()) {
                    for assignment in assignments {
                        if let Some(assignment_id) = assignment.as_str() {
                            assignment_sections.insert(assignment_id.to_string(), section_name.to_string());
                        }
                    }
                }
            }
        } else {
            println!("No units array found in class data");
        }
        
        Ok(assignment_sections)
    }

    pub async fn fetch_progressions(&self, class_id: &str, page: i32) -> Result<ProgressionResponse> {
        let token = self
            .token
            .as_ref()
            .ok_or_else(|| anyhow!("Not authenticated"))?;

        let url = format!(
            "{}/v1/classes/{}/progressions?return_count=200&page={}&sort_by_completed_at=-1",
            self.config.api_base, class_id, page
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
        
        let outer_json: serde_json::Value = serde_json::from_str(&text)?;
        
        let inner_json_str = outer_json
            .get("result")
            .and_then(|r| r.get("objects"))
            .and_then(|o| o.as_array())
            .and_then(|arr| arr.get(0))
            .and_then(|s| s.as_str())
            .ok_or_else(|| anyhow!("Invalid response structure"))?;
        
        let body: ProgressionResponse = serde_json::from_str(inner_json_str)?;
        Ok(body)
    }
}
