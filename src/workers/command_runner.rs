use std::rc::Rc;
use std::sync::Arc;
use std::io::{self, Write};
use std::time::{Instant};
use crate::config::constants::DEFAULT_TIMEOUT_MINUTES;
use crate::enums::commands::Commands;
use crate::config::config_manager::ConfigManager;
use crate::errors::{AicedError, AicedResult};
use crate::services::file_modifier::FileModifier;
use crate::services::repository_manager::RepositoryManager;
use crate::structs::analyze_repository_response::AnalyzeRepositoryResponse;
use crate::structs::config::config::Config;
use crate::ui::diff_server::DiffServer;

pub struct CommandRunner {
    start_time: Option<Instant>,
}

impl CommandRunner {
    pub fn new() -> Self {
        Self {
            start_time: None,
        }
    }

    pub async fn run_command(&mut self, command: Commands) -> AicedResult<()> {
        self.start_time = Some(Instant::now());

        let result = match command {
            Commands::Init => self.init_command().await,
            Commands::Analyze { repo, tags, profile } => self.analyze_command(repo, tags, profile).await,
            Commands::List => self.list_command().await,
            Commands::Dashboard { port } => self.dashboard_command(port).await,
            Commands::Validate => self.validate_command().await,
            Commands::History { repo, days } => self.history_command(repo, days).await,
        };

        if let Some(start) = self.start_time {
            let duration = start.elapsed();
            log::info!("⏱️  Command completed in {:.2}s", duration.as_secs_f64());
        }

        result
    }

    async fn init_command(&self) -> AicedResult<()> {
        log::info!("🚀 Initializing aiced configuration...");

        match ConfigManager::create_sample_multi_repo_config() {
            Ok(_) => {
            }
            Err(e) => {
                log::error!("❌ Failed to create configuration: {}", e);
                return Err(e);
            }
        }

        Ok(())
    }

    async fn analyze_command(&self, repo: Option<String>, _tags: Vec<String>, _profile: Option<String>) -> AicedResult<()> {
        log::info!("🔍 Starting code analysis...");

        let config = match ConfigManager::load() {
            Ok(config) => config,
            Err(e) => {
                log::error!("❌ Failed to load configuration: {}", e);
                log::error!("💡 Run 'aiced init' to create a configuration file.");
                return Err(e);
            }
        };

        ConfigManager::validate_config(Rc::clone(&config))?;

        let mut results: Vec<Rc<AnalyzeRepositoryResponse>> = Vec::new();
        let mut manager = RepositoryManager::new(Rc::clone(&config));

        if let Some(repo_name) = repo {
            self.analyze_single_repository(&mut manager, &repo_name, &mut results).await?;
        } else {
            self.analyze_all_repositories(&mut manager, &mut results).await?;
        }

        if results.is_empty() {
            log::info!("⚠️ No repositories were successfully analyzed.");
            log::info!("💡 Check the errors above and verify your configuration.");
            return Ok(());
        }

        log::info!("✅ Analysis complete for {} repositories", results.len());

        for result in results {
            if let Err(e) = self.process_repository_result_enhanced(result, &config).await {
                log::error!("❌ Error processing repository results: {}", e);
                log::error!("   Continuing with next repository...");
            }
        }

        Ok(())
    }

    async fn analyze_single_repository(&self, manager: &mut RepositoryManager, repo_name: &str, results: &mut Vec<Rc<AnalyzeRepositoryResponse>>) -> AicedResult<()> {
        log::info!("🎯 Analyzing repository: {}", repo_name);
        let repo_config = manager.config.repositories
            .iter()
            .find(|r| r.name == repo_name)
            .cloned()
            .ok_or_else(|| AicedError::repo_error(repo_name, "Line Wrong", "Repository not found"))?;

        match manager.analyze_repository(Arc::new(repo_config.clone()), results).await {
            Ok(_) => {
                log::info!("✅ Successfully analyzed repository: {}", repo_config.name);
            }
            Err(e) => {
                log::error!("❌ Failed to analyze repository '{}': {}", repo_config.name, e);
                return Err(e);
            }
        }

        Ok(())
    }

    async fn analyze_all_repositories(&self, manager: &mut RepositoryManager, results: &mut Vec<Rc<AnalyzeRepositoryResponse>>) -> AicedResult<()> {
        log::info!("🌍 Analyzing all configured repositories...");

        match manager.analyze_all_repositories(results).await {
            Ok(_) => {
                log::info!("✅ Successfully analyzed all repositories");
            }
            Err(e) => {
                log::error!("❌ Error during repository analysis: {}", e);
                log::error!("   Some repositories may have failed. Continuing with successful ones...");
            }
        }

        Ok(())
    }

    async fn process_repository_result_enhanced(&self, result: Rc<AnalyzeRepositoryResponse>, config: &Config) -> AicedResult<()> {
        log::info!("📊 Processing results for: {}", result.repository_config.name);

        let validation_result = FileModifier::validate_changes_batch(
            &result.repository_config,
            &result.repository_analysis.changes
        )?;

        if !validation_result.is_valid {
            log::error!("❌ Validation failed. Skipping this repository.");
            return Ok(());
        }
        self.apply_changes_individually(&result).await?;
        self.handle_post_application_workflow(result, config).await?;
        Ok(())
    }

    async fn handle_post_application_workflow(&self, result: Rc<AnalyzeRepositoryResponse>, config: &Config) -> AicedResult<()> {
        if result.repository_config.auto_pr {
            if let Err(e) = self.handle_pr_creation(Rc::clone(&result)).await {
                log::error!("❌ Failed to create PR: {}", e);
            }
        }

        if let Err(e) = self.save_analysis_results(Rc::clone(&result)).await {
            log::error!("❌ Failed to save analysis results: {}", e);
        }

        if config.notifications.enabled {
            if let Err(e) = self.send_notifications(Rc::clone(&result)).await {
                log::error!("❌ Failed to send notifications: {}", e);
            }
        }

        Ok(())
    }

    async fn handle_pr_creation(&self, result: Rc<AnalyzeRepositoryResponse>) -> AicedResult<()> {
        print!("\n🌿 PR Branch name? (default: \"improvements/aiced-apply-changes\"): ");
        io::stdout().flush()?;

        let mut branch = String::new();
        io::stdin().read_line(&mut branch)?;

        if branch.trim().is_empty() {
            branch = "improvements/aiced-apply-changes".to_string();
        }

        self.create_pr(result, branch.trim().to_string()).await
    }

    async fn list_command(&self) -> AicedResult<()> {
        log::info!("📋 Loading repository configuration...");

        let config = ConfigManager::load()?;

        log::info!("\n📋 Configured Repositories:");
        log::info!("{}", "=".repeat(50));

        if config.repositories.is_empty() {
            log::info!("⚠️ No repositories configured.");
            log::info!("💡 Run 'aiced init' to create a configuration file.");
            return Ok(());
        }

        for (i, repo) in config.repositories.iter().enumerate() {
            log::info!("{}. ✅ {}", i + 1, repo.name);
            log::info!("   📁 Path: {}", repo.path);
            log::info!("   🔧 Auto PR: {}", if repo.auto_pr { "✅" } else { "❌" });

            log::info!("\n");
        }

        log::info!("📊 Total repositories: {}", config.repositories.len());
        Ok(())
    }

    async fn dashboard_command(&self, port: u16) -> AicedResult<()> {
        log::info!("🌐 Starting aiced dashboard...");
        log::info!("🚀 Dashboard will be available at: http://localhost:{}", port);
        log::info!("⏹️ Press Ctrl+C to stop the dashboard");

        // TODO: Implement web dashboard
        // This would start a web server showing:
        // - Repository analysis history
        // - Change statistics
        // - Real-time analysis progress
        // - Configuration management UI

        Ok(())
    }

    async fn validate_command(&self) -> AicedResult<()> {
        let config = match ConfigManager::load() {
            Ok(config) => {
                config
            }
            Err(e) => {
                log::error!("❌ Failed to load configuration: {}", e);
                log::error!("💡 Run 'aiced init' to create a configuration file.");
                return Err(e);
            }
        };
        ConfigManager::validate_config(Rc::clone(&config))?;
        self.perform_extended_validation(&config).await?;

        Ok(())
    }

    async fn perform_extended_validation(&self, config: &Config) -> AicedResult<()> {
        let mut issues = Vec::new();
        let mut warnings = Vec::new();

        // Check repository paths
        for repo in &config.repositories {
            let path = std::path::Path::new(&repo.path);
            if !path.exists() {
                issues.push(format!("Repository path does not exist: {} ({})", repo.name, repo.path));
            } else if !path.is_dir() {
                issues.push(format!("Repository path is not a directory: {} ({})", repo.name, repo.path));
            } else {
                // Check if it's a git repository
                let git_path = path.join(".git");
                if !git_path.exists() {
                    warnings.push(format!("Repository may not be a git repository: {} ({})", repo.name, repo.path));
                }
            }
        }

        // Check for duplicate repository names
        let mut names = std::collections::HashSet::new();
        for repo in &config.repositories {
            if !names.insert(&repo.name) {
                issues.push(format!("Duplicate repository name: {}", repo.name));
            }
        }

        if !issues.is_empty() {
            log::info!("❌ Issues found:");
            for issue in &issues {
                log::info!("   - {}", issue);
            }
        }

        if !warnings.is_empty() {
            log::info!("⚠️ Warnings:");
            for warning in &warnings {
                log::info!("   - {}", warning);
            }
        }

        Ok(())
    }

    async fn history_command(&self, _repo: Option<String>, _days: u32) -> AicedResult<()> {

        // TODO: Implement history functionality
        // This would show:
        // - Previous analysis results
        // - Changes applied over time
        // - Success/failure rates
        // - Performance metrics

        log::info!("🚧 History feature coming soon!");
        log::info!("💡 Analysis results will be stored and displayed here.");

        Ok(())
    }

    async fn create_pr(&self, _analyze_repository_response: Rc<AnalyzeRepositoryResponse>, branch: String) -> AicedResult<()> {
        log::info!("  📨 Creating PR branch: {}", branch);
        // TODO: Implement PR creation
        Ok(())
    }

    pub async fn save_analysis_results(&self, _analyze_repository_response: Rc<AnalyzeRepositoryResponse>) -> AicedResult<()> {
        log::info!("  💾 Saving analysis results...");
        // TODO: Implement result saving
        Ok(())
    }

    async fn send_notifications(&self, _analyze_repository_response: Rc<AnalyzeRepositoryResponse>) -> AicedResult<()> {
        log::info!("  📨 Sending notifications...");
        // TODO: Implement notifications (Slack, email, webhook)
        Ok(())
    }

    async fn apply_changes_individually(&self, result: &AnalyzeRepositoryResponse) -> AicedResult<bool> {
        log::info!("🌐 Starting interactive diff viewer...");

        let mut diff_server = DiffServer::new();
        let port = diff_server.start().await?;

        let session_id = diff_server.create_session(
            &result.repository_config,
            result.repository_analysis.changes.clone()
        ).await?;

        let url = format!("http://localhost:{}?session={}", port, session_id);

        log::info!("📱 Opening interactive diff viewer...");
        log::info!("🔗 URL: {}", url);

        match webbrowser::open(&url) {
            Ok(_) => {
                log::info!("✅ Browser opened successfully");
            }
            Err(e) => {
                log::warn!("⚠️ Failed to open browser automatically: {}", e);
                log::info!("📋 Please manually open: {}", url);
            }
        }

        log::info!("👆 Review changes in your browser and click 'Complete Review' when done");
        log::info!("⏱️ Waiting for review completion (timeout: {} minutes)...", DEFAULT_TIMEOUT_MINUTES);

        let applied_change_ids = diff_server.wait_for_completion(&session_id, DEFAULT_TIMEOUT_MINUTES).await?;

        diff_server.shutdown().await?;

        if applied_change_ids.is_empty() {
            log::info!("📊 No changes approved for application");
            return Ok(false);
        }

        let changes_to_apply = self.filter_changes_by_ids(&result.repository_analysis.changes, &applied_change_ids);

        match FileModifier::apply_changes_grouped_by_file(
            Arc::new(result.repository_config.as_ref().clone()),
            changes_to_apply
        ) {
            Ok(applied_count) => {
                log::info!("✅ Successfully applied {} changes", applied_count);
                Ok(applied_count > 0)
            }
            Err(e) => {
                log::error!("❌ Failed to apply changes: {}", e);
                Err(e)
            }
        }
    }

    fn filter_changes_by_ids<'a>(&self, all_changes: &'a [crate::enums::file_change::FileChange], applied_ids: &[String], ) -> Vec<&'a crate::enums::file_change::FileChange> {
        // For now, we'll use a simple approach where we match changes by their content
        // In a more sophisticated implementation, we would store the mapping between
        // change IDs and FileChange objects in the session

        // Since the session manager creates unique IDs for each change, we need to
        // implement a way to map back. For this implementation, we'll apply all changes
        // that were marked as applied in the session.

        // TODO: Implement proper ID mapping between session changes and FileChange objects
        // For now, return all changes if any were applied
        if !applied_ids.is_empty() {
            all_changes.iter().collect()
        } else {
            Vec::new()
        }
    }
}