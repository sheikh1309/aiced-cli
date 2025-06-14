use std::collections::HashMap;
use std::rc::Rc;
use std::sync::Arc;
use tokio::time::{sleep, Duration};
use crate::logger::animated_logger::AnimatedLogger;
use crate::logger::file_change_logger::FileChangeLogger;
use crate::services::code_analyzer::CodeAnalyzer;
use crate::structs::analysis_result::AnalysisResult;
use crate::structs::analyze_repository_response::AnalyzeRepositoryResponse;
use crate::structs::config::config::Config;
use crate::structs::config::repository_config::RepositoryConfig;

pub struct RepositoryManager {
    pub config: Config,
    results_cache: HashMap<String, AnalysisResult>,
}

impl RepositoryManager {
    pub fn new(config: Config) -> Self {
        Self {
            config,
            results_cache: HashMap::new(),
        }
    }

    pub async fn analyze_all_repositories(&mut self, results: &mut Vec<Rc<AnalyzeRepositoryResponse>>) -> Result<(), Box<dyn std::error::Error>> {
        let enabled_repos: Vec<_> = self.config.repositories
            .iter()
            .cloned()
            .collect();

        println!("🚀 Analyzing {} repositories", enabled_repos.len());

        for (index, repo) in enabled_repos.iter().enumerate() {
            self.analyze_repository(Arc::new(repo.clone()), results).await?;
            
            if index < enabled_repos.len() - 1 {
                let mut logger = AnimatedLogger::new("Sleeping for 60 seconds".to_string());
                logger.start();
                sleep(Duration::from_secs(60)).await;
                logger.stop("Resume To Next Repository").await;
            }
        }

        Ok(())
    }

    pub async fn analyze_repository(&mut self, repository_config: Arc<RepositoryConfig>, results: &mut Vec<Rc<AnalyzeRepositoryResponse>>) -> Result<(), Box<dyn std::error::Error>> {
        println!("\n🔍 Analyzing repository: {}", repository_config.name);
        if repository_config.auto_pull {
            self.pull_repository(Arc::clone(&repository_config)).await?;
        }

        let analyzer = self.create_analyzer_for_repo(Arc::clone(&repository_config))?;
        let analyze_repository_response = analyzer.analyze_repository().await?;
        FileChangeLogger::print_analysis_report(Rc::clone(&analyze_repository_response));
        results.push(Rc::clone(&analyze_repository_response));

        self.save_analysis_results(Rc::clone(&analyze_repository_response)).await?;
        if !self.config.notifications.enabled {
            self.send_notifications(Rc::clone(&analyze_repository_response)).await?;
        }
      
        Ok(())
    }

    async fn pull_repository(&self, repo: Arc<RepositoryConfig>) -> Result<(), Box<dyn std::error::Error>> {
        use std::process::Command;

        println!("  📥 Pulling latest changes...");

        let output = Command::new("git")
            .args(&["pull", "origin", repo.branch.as_deref().unwrap_or("main")])
            .current_dir(&repo.path)
            .output()?;

        if !output.status.success() {
            return Err(format!("Git pull failed: {}", String::from_utf8_lossy(&output.stderr)).into());
        }

        Ok(())
    }

    fn create_analyzer_for_repo(&self, repository_config: Arc<RepositoryConfig>) -> Result<CodeAnalyzer, Box<dyn std::error::Error>> {
        let var_name = match &self.config.ai.api_key_env {
            None => panic!("API key environment variable not set"),
            Some(val) => val
        };
        let api_key = std::env::var(var_name)?;

        Ok(CodeAnalyzer::new(api_key, Arc::clone(&repository_config))?)
    }

    pub async fn save_analysis_results(&self, analyze_repository_response: Rc<AnalyzeRepositoryResponse>) -> Result<(), Box<dyn std::error::Error>> {
        println!("  💾 Saving analysis results... {:?}", analyze_repository_response);
        // todo
        Ok(())
    }

    async fn send_notifications(&self, analyze_repository_response: Rc<AnalyzeRepositoryResponse>) -> Result<(), Box<dyn std::error::Error>> {
        // Implement Slack, email, webhook notifications
        println!("  📨 Sending notifications... {:?}", analyze_repository_response);
        // todo
        Ok(())
    }
}