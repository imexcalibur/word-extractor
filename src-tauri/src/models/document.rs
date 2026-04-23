use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Document {
    pub id: String,
    pub file_path: String,
    pub file_name: String,
    pub file_size: u64,
    pub added_at: DateTime<Utc>,
}

impl Document {
    pub fn new(file_path: String) -> Self {
        let file_name = std::path::Path::new(&file_path)
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown")
            .to_string();

        let file_size = std::fs::metadata(&file_path)
            .map(|m| m.len())
            .unwrap_or(0);

        Self {
            id: Uuid::new_v4().to_string(),
            file_path,
            file_name,
            file_size,
            added_at: Utc::now(),
        }
    }
}