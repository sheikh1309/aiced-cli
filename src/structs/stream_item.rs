use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StreamItem {
    pub content: String,
    pub is_complete: bool,
    pub finish_reason: Option<String>,
}

impl StreamItem {
    pub fn new(content: String) -> Self {
        Self {
            content,
            is_complete: false,
            finish_reason: None,
        }
    }

    pub fn complete(content: String, finish_reason: Option<String>) -> Self {
        Self {
            content,
            is_complete: true,
            finish_reason,
        }
    }
}