use serde::Serialize;
use crate::enums::file_change::FileChange;

#[derive(Debug, Serialize)]
pub struct AnalysisResponse {
    pub analysis_summary: String,
    pub changes: Vec<FileChange>,
}