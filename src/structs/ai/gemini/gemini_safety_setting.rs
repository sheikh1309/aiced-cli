use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct GeminiSafetySetting {
    pub category: String,
    pub threshold: String,
}