use anyhow::{anyhow, Result};
use super::OpenClassProvider;

impl OpenClassProvider {
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
}
