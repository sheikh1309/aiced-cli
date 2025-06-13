use crate::enums::analysis_status::AnalysisStatus;

#[derive(Debug, Clone)]
pub struct AnalysisResult {
    pub repository: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub issues_found: usize,
    pub critical_issues: usize,
    pub duration_seconds: u64,
    pub status: AnalysisStatus,
}