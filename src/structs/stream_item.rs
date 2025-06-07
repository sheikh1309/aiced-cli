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

impl crate::traits::stream_processor::StreamItemLike for StreamItem {
    fn content(&self) -> &str {
        &self.content
    }

    fn is_complete(&self) -> bool {
        self.is_complete
    }

    fn finish_reason(&self) -> &Option<String> {
        &self.finish_reason
    }

    fn create_complete(content: String, finish_reason: Option<String>) -> Self {
        Self::complete(content, finish_reason)
    }
}