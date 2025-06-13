use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct SlackConfig {
    pub webhook_url_env: String,
    pub channel: String,
    pub mention_on_critical: Vec<String>,
}