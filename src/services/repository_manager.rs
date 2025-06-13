use std::collections::HashMap;
use crate::enums::analysis_status::AnalysisStatus;
use crate::enums::file_change::FileChange;
use crate::enums::priority::Priority;
use crate::services::code_analyzer::CodeAnalyzer;
use crate::structs::analysis_response::AnalysisResponse;
use crate::structs::analysis_result::AnalysisResult;
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

    pub async fn analyze_all_repositories(&mut self) -> Result<Vec<AnalysisResult>, Box<dyn std::error::Error>> {
        // Clone the repositories to avoid borrowing self while iterating
        let enabled_repos: Vec<_> = self.config.repositories
            .iter()
            .filter(|r| r.enabled)
            .cloned()
            .collect();

        println!("🚀 Analyzing {} repositories", enabled_repos.len());

        let mut results = Vec::new();

        // Group by priority
        let mut priority_groups: HashMap<Priority, Vec<RepositoryConfig>> = HashMap::new();
        for repo in enabled_repos {
            priority_groups.entry(repo.priority.clone()).or_insert(Vec::new()).push(repo);
        }

        // Analyze in priority order
        for priority in [Priority::Critical, Priority::High, Priority::Medium, Priority::Low] {
            if let Some(repos) = priority_groups.get(&priority) {
                println!("\n📊 Analyzing {} priority repositories:", format!("{:?}", priority).to_lowercase());

                for repo in repos {
                    let result = self.analyze_repository(&repo).await?;
                    results.push(result);

                    // Add delay between repos to avoid rate limits
                    tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
                }
            }
        }

        Ok(results)
    }

    pub async fn analyze_repository(&mut self, repo: &RepositoryConfig) -> Result<AnalysisResult, Box<dyn std::error::Error>> {
        println!("\n🔍 Analyzing repository: {}", repo.name);
        let start_time = std::time::Instant::now();

        // Pull latest changes if configured
        if repo.auto_pull {
            self.pull_repository(repo).await?;
        }

        let profile = repo.profile.as_ref()
            .unwrap_or(&self.config.global.default_profile);
        
        let profile_config = self.config.profiles.get(profile)
            .ok_or_else(|| format!("Profile not found: {}", profile))?;
        
        let analyzer = self.create_analyzer_for_repo(repo, profile_config)?;

        match analyzer.analyze_repository().await {
            Ok(analysis) => {
                let critical_count = analysis.changes.iter()
                    .filter(|c| matches!(c,
                        FileChange::ModifyFile { severity, .. } |
                        FileChange::CreateFile { severity, .. } |
                        FileChange::DeleteFile { severity, .. }
                        if severity == "critical"
                    ))
                    .count();

                let result = AnalysisResult {
                    repository: repo.name.clone(),
                    timestamp: chrono::Utc::now(),
                    issues_found: analysis.changes.len(),
                    critical_issues: critical_count,
                    duration_seconds: start_time.elapsed().as_secs(),
                    status: AnalysisStatus::Success,
                };

                // Save results
                self.save_analysis_results(&repo.name, &analysis).await?;

                // Send notifications if needed
                if critical_count > 0 || !self.config.notifications.on_critical_only {
                    self.send_notifications(&result, &analysis).await?;
                }

                Ok(result)
            }
            Err(e) => {
                let result = AnalysisResult {
                    repository: repo.name.clone(),
                    timestamp: chrono::Utc::now(),
                    issues_found: 0,
                    critical_issues: 0,
                    duration_seconds: start_time.elapsed().as_secs(),
                    status: AnalysisStatus::Failed(()),
                };

                Ok(result)
            }
        }
    }

    async fn pull_repository(&self, repo: &RepositoryConfig) -> Result<(), Box<dyn std::error::Error>> {
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
        repo: &RepositoryConfig,
        profile: &ProfileConfig
    ) -> Result<CodeAnalyzer, Box<dyn std::error::Error>> {
        let var_name = match &self.config.ai.api_key_env {
            None => panic!("API key environment variable not set"),
            Some(val) => val
        };
        let api_key = std::env::var(var_name)?;
        
        let repo_config = Config {
            global: self.config.global.clone(),
            repositories: vec![repo.clone()],
            profiles: self.config.profiles.clone(),
            ai: self.config.ai.clone(),
            output: self.config.output.clone(),
            notifications: self.config.notifications.clone(),
        };

        Ok(CodeAnalyzer::new(api_key, repo.path.clone(), &repo_config)?)
    }

    async fn save_analysis_results(
        &self,
        repo_name: &str,
        analysis: &AnalysisResponse
    ) -> Result<(), Box<dyn std::error::Error>> {
        todo!();
        Ok(())
    }

    async fn send_notifications(
        &self,
        result: &AnalysisResult,
        analysis: &AnalysisResponse
    ) -> Result<(), Box<dyn std::error::Error>> {
        // Implement Slack, email, webhook notifications
        println!("  📨 Sending notifications...");

        // This would contain actual notification logic

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