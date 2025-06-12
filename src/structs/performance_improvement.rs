use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceImprovement {
    pub file_path: String,
    pub line_number: usize,
    pub issue: String,
    pub current_code: String,
    pub suggested_code: String,
    pub impact: String,
}