use std::rc::Rc;
use std::sync::Arc;
use tokio::time::{sleep, Duration};
use crate::errors::{AilyzerError, AilyzerResult};
use crate::logger::animated_logger::AnimatedLogger;
use crate::services::code_analyzer::CodeAnalyzer;
use crate::structs::analyze_repository_response::AnalyzeRepositoryResponse;
use crate::structs::config::config::Config;
use crate::structs::config::repository_config::RepositoryConfig;

pub struct RepositoryManager {
    pub config: Rc<Config>
}

impl RepositoryManager {
    pub fn new(config: Rc<Config>) -> Self {
        Self { config }
    }

    pub async fn analyze_all_repositories(&mut self, results: &mut Vec<Rc<AnalyzeRepositoryResponse>>) -> AilyzerResult<()> {
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

    pub async fn analyze_repository(&mut self, repository_config: Arc<RepositoryConfig>, results: &mut Vec<Rc<AnalyzeRepositoryResponse>>) -> AilyzerResult<()> {
        println!("\n🔍 Analyzing repository: {}", repository_config.name);
        if repository_config.auto_pull {
            self.pull_repository(Arc::clone(&repository_config)).await?;
        }

        let analyzer = CodeAnalyzer::new(Arc::clone(&repository_config));
        let analyze_repository_response = analyzer.analyze_repository().await?;
        results.push(Rc::clone(&analyze_repository_response));

        Ok(())
    }

    async fn pull_repository(&self, repo: Arc<RepositoryConfig>) -> AilyzerResult<()> {
        use std::process::Command;

        println!("  📥 Pulling latest changes...");

        let output = Command::new("git")
            .args(&["pull", "origin", repo.branch.as_deref().unwrap_or("main")])
            .current_dir(&repo.path)
            .output()?;

        if !output.status.success() {
            return Err(AilyzerError::system_error("git pull", "Failed to pull latest changes").into());
        }

        Ok(())
    }
}