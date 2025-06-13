use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct PatternRule {
    pub pattern: String,
    pub message: String,
    pub severity: String,
    pub category: String,
}