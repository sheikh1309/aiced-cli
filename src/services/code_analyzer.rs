use std::rc::Rc;
use std::sync::Arc;
use std::time::Duration;
use reqwest::Client;
use crate::errors::{AilyzerError, AilyzerResult};
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
    client: Client
}

impl CodeAnalyzer {

    pub fn new(repository_config: Arc<RepositoryConfig>) -> Self {
        Self {
            repo_scanner: RepoScanner::new(Arc::clone(&repository_config)),
            repository_config,
            client: Client::builder()
                .connect_timeout(Duration::from_secs(20))
                .timeout(Duration::from_secs(60 * 30)) // Increase from default
                .build().unwrap(),
        }
    }

    pub async fn analyze_repository(&self) -> AilyzerResult<Rc<AnalyzeRepositoryResponse>> {
        let files = self.repo_scanner.scan_files().await?;
        let user_prompt = prompt_generator::generate_analysis_user_prompt(files, &self.repository_config.path);
        let mut logger = AnimatedLogger::new("Analyzing Repository".to_string());
        logger.start();

        let request_body = AnalyzeRequest { prompt: user_prompt };

        let response = match self.client
            .post("http://localhost:3000/api/analyze")
            .header("x-api-key", "api-key-123456")
            .header("Content-Type", "application/json")
            .json(&request_body)
            .send()
            .await
        {
            Ok(resp) => resp,
            Err(e) => {
                logger.stop("Analysis failed").await;
                eprintln!("Network error during API request: {}", e);
                return Err(AilyzerError::system_error("analysis Error", "Failed to connect to analysis server").into());
            }
        };

        let body: ApiResponse<AnalyzeResponse> = match response.status() {
            reqwest::StatusCode::OK => {
                match response.json().await {
                    Ok(data) => data,
                    Err(e) => {
                        logger.stop("Analysis failed").await;
                        eprintln!("Failed to parse JSON response: {}", e);
                        return Err(AilyzerError::system_error("analysis Error", &"Invalid response format from server").into());
                    }
                }
            },
            reqwest::StatusCode::REQUEST_TIMEOUT => {
                logger.stop("Analysis failed").await;
                eprintln!("Request timed out (408)");
                return Err(AilyzerError::system_error("analysis Error", &"Request timed out (408)").into());
            },
            status => {
                let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
                logger.stop("Analysis failed").await;
                eprintln!("Request failed with status {}: {}", status, error_text);
                return Err(AilyzerError::system_error("analysis Error", &format!("Request failed with status {}: {}", status, error_text)).into());
            }
        };

        if !body.success {
            logger.stop("Analysis failed").await;
            eprintln!("API returned error: {}", body.message);
            return Err(AilyzerError::system_error("analysis Error", &format!("Analysis failed: {}", body.message)).into());
        }

        let analyze_data = match &body.data {
            Some(data) => data,
            None => {
                logger.stop("Analysis failed").await;
                eprintln!("API response missing data field");
                return Err(AilyzerError::system_error("analysis Error", &"API response missing data field").into());
            }
        };

        logger.stop("Analysis complete").await;

        let full_content = &analyze_data.content;
        let mut analysis_parser = AnalysisParser::new(&full_content);
        let analysis = match analysis_parser.parse() {
            Ok(analysis) => analysis,
            Err(e) => {
                eprintln!("Failed to parse custom format: {}", e);
                return Err(AilyzerError::system_error("analysis Error", &format!("Failed to parse custom format: {}", e)).into());
            }
        };

        Ok(Rc::new(AnalyzeRepositoryResponse {
            repository_analysis: Rc::new(analysis),
            repository_config: Rc::new((*self.repository_config).clone())
        }))
    }

}