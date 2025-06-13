use crate::enums::file_change::FileChange;
use crate::structs::performance_improvement::PerformanceImprovement;
use crate::structs::security_issue::SecurityIssue;

#[derive(Debug)]
pub struct AnalysisResponse {
    pub analysis_summary: String,
    pub changes: Vec<FileChange>,
    pub security_issues: Vec<SecurityIssue>,
    pub performance_improvements: Vec<PerformanceImprovement>,
}