use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeepSeekDelta {
    pub role: Option<String>,
    pub content: Option<String>,
}