use std::rc::Rc;
use std::sync::Arc;
use crate::adapters::aiced_adapter::AicedAdapter;
use crate::errors::AicedResult;
use crate::helpers::prompt_generator;
use crate::logger::animated_logger::AnimatedLogger;
use crate::prompts::system_analysis_prompt::SYSTEM_ANALYSIS_PROMPT;
use crate::services::ai::anthropic::AnthropicProvider;
use crate::services::analysis_parser::AnalysisParser;
use crate::services::repo_scanner::RepoScanner;
use crate::structs::analyze_repository_response::AnalyzeRepositoryResponse;
use crate::structs::config::repository_config::RepositoryConfig;

pub struct CodeAnalyzer {
    repo_scanner: RepoScanner,
    repository_config: Arc<RepositoryConfig>,
    adapter: Arc<AicedAdapter>,
}

impl CodeAnalyzer {

    pub fn new(repository_config: Arc<RepositoryConfig>) -> Self {
        let ai_provider = Arc::new(AnthropicProvider::new(std::env::var("ANTHROPIC_API_KEY").unwrap()));
        let adapter = Arc::new(AicedAdapter::new(ai_provider));
        Self { repo_scanner: RepoScanner::new(Arc::clone(&repository_config), Arc::clone(&adapter)), repository_config, adapter }
    }

    pub async fn analyze_repository(&self) -> AicedResult<Rc<AnalyzeRepositoryResponse>> {
        let files = self.repo_scanner.scan_files().await?;
        let user_prompt = prompt_generator::generate_analysis_user_prompt(files, &self.repository_config.path);
        let mut logger = AnimatedLogger::new("Analyzing Repository".to_string());
        logger.start();

        let analyze_data = self.adapter.stream_llm_chat(user_prompt, SYSTEM_ANALYSIS_PROMPT.to_string()).await;
        logger.stop("Analysis complete").await;
        let mut analysis_parser = AnalysisParser::new(&analyze_data?.content);
        let analysis = analysis_parser.parse()?;

        Ok(Rc::new(AnalyzeRepositoryResponse {
            repository_analysis: Rc::new(analysis),
            repository_config: Rc::new((*self.repository_config).clone())
        }))
    }

}