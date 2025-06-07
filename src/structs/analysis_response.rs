use serde::{Deserialize, Serialize};
use crate::enums::file_change::FileChange;
use crate::structs::performance_improvement::PerformanceImprovement;
use crate::structs::security_issue::SecurityIssue;

#[derive(Debug, Deserialize, Serialize)]
pub struct AnalysisResponse {
    pub analysis_summary: String,
    pub changes: Vec<FileChange>,
    #[serde(default)]
    pub security_issues: Vec<SecurityIssue>,
    #[serde(default)]
    pub performance_improvements: Vec<PerformanceImprovement>,
}