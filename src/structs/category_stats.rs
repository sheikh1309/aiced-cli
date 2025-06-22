use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CategoryStats {
    pub category: String,
    pub count: usize,
    pub percentage: usize,
    pub is_high_impact: bool,
}