use std::fs;
use std::rc::Rc;
use std::sync::Arc;
use futures::StreamExt;
use crate::helpers::prompt_generator;
use crate::prompts::system_analysis_prompt::SYSTEM_ANALYSIS_PROMPT;
use crate::logger::animated_logger::AnimatedLogger;
use crate::services::anthropic::AnthropicProvider;
use crate::services::custom_parser::Parser;
use crate::services::repo_scanner::RepoScanner;
use crate::services::rate_limiter::ApiRateLimiter;
use crate::structs::analyze_repository_response::AnalyzeRepositoryResponse;
use crate::structs::config::repository_config::RepositoryConfig;

pub struct CodeAnalyzer {
    anthropic_provider: Arc<AnthropicProvider>,
    repo_scanner: RepoScanner,
    repository_config: Arc<RepositoryConfig>,
}

impl CodeAnalyzer {

    pub fn new(api_key: String, repository_config: Arc<RepositoryConfig>) -> Result<Self, Box<dyn std::error::Error>> {
        let anthropic_provider = Arc::new(AnthropicProvider::new(api_key.clone(), Arc::new(ApiRateLimiter::new())));
        Ok(Self {
            anthropic_provider: Arc::clone(&anthropic_provider),
            repo_scanner: RepoScanner::new(anthropic_provider, Arc::clone(&repository_config)),
            repository_config,
        })
    }

    pub async fn analyze_repository(&self) -> Result<Rc<AnalyzeRepositoryResponse>, Box<dyn std::error::Error>> {
        let files = self.repo_scanner.scan_files().await?;

        let user_prompt = prompt_generator::generate_analysis_user_prompt(files, &self.repository_config.path);
        let mut logger = AnimatedLogger::new("Analyzing Repository".to_string());
        logger.start();
        let mut full_content = String::new();
        let mut input_tokens = 0u32;
        let mut output_tokens = 0u32;
        let mut stream = self.anthropic_provider.trigger_stream_request(SYSTEM_ANALYSIS_PROMPT.to_string(), vec![user_prompt]).await?;

        while let Some(result) = stream.next().await {
            match result {
                Ok(item) => {
                    if !item.content.is_empty() {
                        full_content.push_str(&item.content);
                    }

                    if let Some(input) = item.input_tokens {
                        input_tokens = input;
                    }

                    if let Some(output) = item.output_tokens {
                        output_tokens = output;
                    }

                    if item.is_complete {
                        println!("is_complete {:?}", item);
                        break;
                    }
                }
                Err(_e) => {},
            }
        }

        logger.stop("Analysis complete").await;

        println!("Input tokens: {}", input_tokens);
        println!("Output tokens: {}", output_tokens);
        fs::write(format!("ai_output_{}.txt", self.repository_config.name), &full_content)?;
        
        let mut parser = Parser::new(&full_content);
        let analysis = parser.parse().map_err(|e| { format!("Failed to parse custom format: {}", e) })?;

        Ok(Rc::new(AnalyzeRepositoryResponse { repository_analysis: Rc::new(analysis), repository_config: Rc::new((*self.repository_config).clone()) }))
    }
    
}