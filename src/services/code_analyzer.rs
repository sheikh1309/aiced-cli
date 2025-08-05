use std::rc::Rc;
use std::sync::Arc;
use crate::adapters::aiced_adapter::AicedAdapter;
use crate::config::constants::ANTHROPIC_API_KEY_ENV;
use crate::errors::{AicedError, AicedResult};
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

    pub fn new(repository_config: Arc<RepositoryConfig>) -> AicedResult<Self> {
        let api_key = std::env::var(ANTHROPIC_API_KEY_ENV)
            .map_err(|_| AicedError::configuration_error(
                "ANTHROPIC_API_KEY environment variable not set",
                Some("environment"),
                Some("Set your Anthropic API key: export ANTHROPIC_API_KEY=your_key_here")
            ))?;
        
        if api_key.trim().is_empty() {
            return Err(AicedError::configuration_error(
                "ANTHROPIC_API_KEY cannot be empty",
                Some("environment"),
                Some("Provide a valid Anthropic API key")
            ));
        }

        let ai_provider = Arc::new(AnthropicProvider::new(api_key));
        let adapter = Arc::new(AicedAdapter::new(ai_provider));
        Ok(Self { 
            repo_scanner: RepoScanner::new(Arc::clone(&repository_config), Arc::clone(&adapter)), 
            repository_config, 
            adapter 
        })
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