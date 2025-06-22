use crate::enums::file_change::FileChange;
use crate::structs::technology_stack::TechnologyStack;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalysisResponse {
    pub technology_stack: Option<TechnologyStack>,
    pub analysis_summary: String,
    pub changes: Vec<FileChange>,
}