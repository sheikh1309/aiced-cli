use std::rc::Rc;
use crate::structs::analysis_response::AnalysisResponse;
use crate::structs::config::repository_config::RepositoryConfig;

#[derive(Debug)]
pub struct AnalyzeRepositoryResponse {
    pub repository_analysis: Rc<AnalysisResponse>,
    pub repository_config: Rc<RepositoryConfig>,
} 