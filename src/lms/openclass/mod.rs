use reqwest::Client;
use crate::config::Config;
use std::time::Duration;
use async_trait::async_trait;
use anyhow::Result;
use crate::lms::{LmsProvider, ProgressionBatch, Progression};
use crate::models;

mod auth;
mod fetch;
pub mod types;

pub use types::{ProgressionResponse, Progression as OpenClassProgression, User, Assignment as OpenClassAssignment, Metadata};

pub struct OpenClassProvider {
    pub(crate) client: Client,
    pub(crate) config: Config,
    pub(crate) token: Option<String>,
}

impl OpenClassProvider {
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
}

#[async_trait]
impl LmsProvider for OpenClassProvider {
    async fn authenticate(&mut self) -> Result<()> {
        // Delegate to auth module
        OpenClassProvider::authenticate(self).await
    }

    async fn fetch_classes(&self) -> Result<Vec<models::Class>> {
        OpenClassProvider::fetch_classes(self).await
    }

    async fn fetch_class_structure(&self, class_id: &str) -> Result<std::collections::HashMap<String, String>> {
        OpenClassProvider::fetch_class_details(self, class_id).await
    }

    async fn fetch_progressions(&self, class_id: &str, page: i32) -> Result<ProgressionBatch> {
        let response = OpenClassProvider::fetch_progressions(self, class_id, page).await?;
        
        let progressions = response.data.into_iter().map(|p| {
            let progression_id = match &p.id {
                serde_json::Value::Object(obj) => {
                    obj.get("$oid")
                        .and_then(|v| v.as_str())
                        .unwrap_or("unknown")
                        .to_string()
                }
                serde_json::Value::String(s) => s.clone(),
                _ => "unknown".to_string(),
            };

            Progression {
                id: progression_id,
                student: models::Student {
                    id: p.user.id.clone(),
                    class_id: class_id.to_string(),
                    first_name: p.user.first_name.clone(),
                    last_name: p.user.last_name.clone(),
                    email: p.user.email.clone(),
                    region: None,
                    night: None,
                },
                assignment: models::Assignment {
                    id: p.assignment.id.clone(),
                    class_id: class_id.to_string(),
                    name: p.assignment.name.clone(),
                    assignment_type: p.assignment.assignment_type.clone(),
                    section: None,
                },
                grade: p.grade,
                started_at: p.started_assignment_at_rfc3339(),
                completed_at: p.completed_assignment_at_rfc3339(),
                reviewed_at: p.reviewed_at_rfc3339(),
            }
        }).collect();

        Ok(ProgressionBatch {
            progressions,
            can_load_more: response.metadata.can_load_more,
        })
    }

    fn provider_name(&self) -> &str {
        "OpenClass"
    }
}
