use std::collections::HashMap;
use std::rc::Rc;
use std::sync::Arc;
use crate::enums::analysis_status::AnalysisStatus;
use crate::enums::file_change::FileChange;
use crate::enums::priority::Priority;
use crate::services::code_analyzer::CodeAnalyzer;
use crate::structs::analysis_response::AnalysisResponse;
use crate::structs::analysis_result::AnalysisResult;
use crate::structs::analyze_repository_response::AnalyzeRepositoryResponse;
use crate::structs::config::config::Config;
use crate::structs::config::profile_config::ProfileConfig;
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
            .filter(|r| r.enabled)
            .cloned()
            .collect();

        println!("🚀 Analyzing {} repositories", enabled_repos.len());

        for repo in enabled_repos {
            self.analyze_repository(Arc::new(repo), results).await?;
            tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
        }

        Ok(())
    }

    pub async fn analyze_repository(&mut self, repository_config: Arc<RepositoryConfig>, results: &mut Vec<Rc<AnalyzeRepositoryResponse>>) -> Result<(), Box<dyn std::error::Error>> {
        println!("\n🔍 Analyzing repository: {}", repository_config.name);
        if repository_config.auto_pull {
            self.pull_repository(Arc::clone(&repository_config)).await?;
        }

        let profile = repository_config.profile.as_ref().unwrap_or(&self.config.global.default_profile);
        let profile_config = self.config.profiles.get(profile).ok_or_else(|| format!("Profile not found: {}", profile))?;

        let analyzer = self.create_analyzer_for_repo(Arc::clone(&repository_config), profile_config)?;
        let analyze_repository_response = analyzer.analyze_repository().await?;
        analyzer.print_analysis_report(Rc::clone(&analyze_repository_response));
        results.push(Rc::clone(&analyze_repository_response));

        self.save_analysis_results(Rc::clone(&analyze_repository_response)).await?;
        if !self.config.notifications.enabled {
            self.send_notifications(Rc::clone(&analyze_repository_response)).await?;
        }
        
        // todo - add sleep for rate limit

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

    fn create_analyzer_for_repo(
        &self,
        repository_config: Arc<RepositoryConfig>,
        profile: &ProfileConfig
    ) -> Result<CodeAnalyzer, Box<dyn std::error::Error>> {
        let var_name = match &self.config.ai.api_key_env {
            None => panic!("API key environment variable not set"),
            Some(val) => val
        };
        let api_key = std::env::var(var_name)?;

        Ok(CodeAnalyzer::new(api_key, Arc::clone(&repository_config))?)
    }

    pub async fn save_analysis_results(
        &self,
        analyze_repository_response: Rc<AnalyzeRepositoryResponse>
    ) -> Result<(), Box<dyn std::error::Error>> {
        println!("  💾 Saving analysis results...");
        Ok(())
    }

    async fn send_notifications(
        &self,
        analyze_repository_response: Rc<AnalyzeRepositoryResponse>
    ) -> Result<(), Box<dyn std::error::Error>> {
        // Implement Slack, email, webhook notifications
        println!("  📨 Sending notifications...");

        Ok(())
    }

    fn generate_markdown_report(&self, repo_name: &str, analysis: &AnalysisResponse) -> String {
        format!(
            "# ailyzer Analysis Report: {}\n\n\
            **Date**: {}\n\
            **Issues Found**: {}\n\n\
            ## Summary\n\
            {}\n\n\
            ## Changes Required\n\
            ...",
            repo_name,
            chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC"),
            analysis.changes.len(),
            analysis.analysis_summary
        )
    }
}