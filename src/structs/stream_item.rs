use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StreamItem {
    pub content: String,
    pub is_complete: bool,
    pub stop_reason: Option<String>,
    pub input_tokens: Option<u32>,
    pub output_tokens: Option<u32>,
}

impl StreamItem {
    pub fn new(content: String) -> Self {
        Self {
            content,
            is_complete: false,
            stop_reason: None,
            input_tokens: None,
            output_tokens: None,
        }
    }

    pub fn complete(content: String, stop_reason: Option<String>, output_tokens: u32) -> Self {
        Self {
            content,
            is_complete: true,
            stop_reason,
            input_tokens: Some(0),
            output_tokens: Some(output_tokens),
        }
    }

    pub fn with_tokens(content: String, input_tokens: Option<u32>, output_tokens: Option<u32>) -> Self {
        Self {
            content,
            is_complete: false,
            stop_reason: None,
            input_tokens,
            output_tokens,
        }
    }
}