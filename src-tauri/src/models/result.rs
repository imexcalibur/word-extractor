use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ContentType {
    Text,
    Table,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContentPosition {
    pub page: Option<u32>,
    pub table_index: Option<u32>,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtractionResult {
    pub document_name: String,
    pub document_id: String,
    pub rule_id: String,
    pub rule_name: String,
    pub content_type: ContentType,
    pub content: String,
    pub position: ContentPosition,
}