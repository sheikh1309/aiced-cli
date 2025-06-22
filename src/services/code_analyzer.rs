use std::rc::Rc;
use std::sync::Arc;
use crate::adapters::ailyzer_adapter::AiLyzerAdapter;
use crate::errors::AilyzerResult;
use crate::helpers::prompt_generator;
use crate::logger::animated_logger::AnimatedLogger;
use crate::services::analysis_parser::AnalysisParser;
use crate::services::repo_scanner::RepoScanner;
use crate::structs::analyze_repository_response::AnalyzeRepositoryResponse;
use crate::structs::analyze_request::AnalyzeRequest;
use crate::structs::analyze_response::AnalyzeResponse;
use crate::structs::api_response::ApiResponse;
use crate::structs::config::repository_config::RepositoryConfig;

pub struct CodeAnalyzer {
    repo_scanner: RepoScanner,
    repository_config: Arc<RepositoryConfig>,
    adapter: Arc<AiLyzerAdapter>,
}

impl CodeAnalyzer {

    pub fn new(repository_config: Arc<RepositoryConfig>) -> Self {
        let adapter = Arc::new(AiLyzerAdapter::new("http://localhost:3000".to_string(), "api-key-123456".to_string()));
        Self { repo_scanner: RepoScanner::new(Arc::clone(&repository_config), Arc::clone(&adapter)), repository_config, adapter }
    }

    pub async fn analyze_repository(&self) -> AilyzerResult<Rc<AnalyzeRepositoryResponse>> {
        let files = self.repo_scanner.scan_files().await?;
        let user_prompt = prompt_generator::generate_analysis_user_prompt(files, &self.repository_config.path);
        let mut logger = AnimatedLogger::new("Analyzing Repository".to_string());
        logger.start();

        let request_body = AnalyzeRequest { prompt: user_prompt };
        let analyze_data: ApiResponse<AnalyzeResponse> = self.adapter.post_json_extract_data("api/analyze", &request_body, &mut logger, "Analysis").await?;
        logger.stop("Analysis complete").await;
        let mut analysis_parser = AnalysisParser::new(&analyze_data.data.unwrap().content);
        let analysis = analysis_parser.parse()?;

        Ok(Rc::new(AnalyzeRepositoryResponse {
            repository_analysis: Rc::new(analysis),
            repository_config: Rc::new((*self.repository_config).clone())
        }))
    }

}