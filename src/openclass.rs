use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: String,
    pub first_name: String,
    pub last_name: String,
    pub email: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Assignment {
    pub id: String,
    pub name: String,
    #[serde(rename = "type")]
    pub assignment_type: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Progression {
    #[serde(rename = "_id")]
    pub id: serde_json::Value, // MongoDB ObjectId
    pub user: User,
    pub assignment: Assignment,
    #[serde(default)]
    pub grade: Option<f64>,
    pub started_assignment_at: String,
    pub completed_assignment_at: String,
    pub reviewed_at: Option<String>,
}

impl Progression {
    pub fn started_assignment_at_rfc3339(&self) -> String {
        self.started_assignment_at.clone()
    }
    
    pub fn completed_assignment_at_rfc3339(&self) -> String {
        self.completed_assignment_at.clone()
    }
    
    pub fn reviewed_at_rfc3339(&self) -> Option<String> {
        self.reviewed_at.clone()
    }
}

#[derive(Debug, Deserialize)]
pub struct ProgressionResponse {
    pub metadata: Metadata,
    pub data: Vec<Progression>,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
pub struct Metadata {
    total: i32,
    page: i32,
    results_per_page: i32,
    pub can_load_more: bool,
}

