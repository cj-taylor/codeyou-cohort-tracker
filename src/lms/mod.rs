use anyhow::Result;
use async_trait::async_trait;
use crate::models::{Class, Student, Assignment};

pub mod openclass;

#[async_trait]
pub trait LmsProvider: Send + Sync {
    async fn authenticate(&mut self) -> Result<()>;
    async fn fetch_classes(&self) -> Result<Vec<Class>>;
    async fn fetch_class_structure(&self, class_id: &str) -> Result<std::collections::HashMap<String, String>>;
    async fn fetch_progressions(&self, class_id: &str, page: i32) -> Result<ProgressionBatch>;
    fn provider_name(&self) -> &str;
}

pub struct ProgressionBatch {
    pub progressions: Vec<Progression>,
    pub can_load_more: bool,
}

pub struct Progression {
    pub id: String,
    pub student: Student,
    pub assignment: Assignment,
    pub grade: Option<f64>,
    pub started_at: String,
    pub completed_at: String,
    pub reviewed_at: Option<String>,
}
