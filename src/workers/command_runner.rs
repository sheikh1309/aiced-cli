use std::rc::Rc;
use std::sync::Arc;
use std::io::{self, Write};
use std::time::{Instant};
use crate::enums::commands::Commands;
use crate::config::config_manager::ConfigManager;
use crate::enums::application_mode::ApplicationMode;
use crate::errors::{AilyzerError, AilyzerResult};
use crate::logger::file_change_logger::FileChangeLogger;
use crate::services::file_modifier::FileModifier;
use crate::services::repository_manager::RepositoryManager;
use crate::structs::analyze_repository_response::AnalyzeRepositoryResponse;
use crate::structs::config::config::Config;
use crate::structs::change_statistics::ChangeStatistics;

pub struct CommandRunner {
    start_time: Option<Instant>,
}

impl CommandRunner {
    pub fn new() -> Self {
        Self {
            start_time: None,
        }
    }

    pub async fn run_command(&mut self, command: Commands) -> AilyzerResult<()> {
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

    async fn init_command(&self) -> AilyzerResult<()> {
        log::info!("🚀 Initializing ailyzer configuration...");

        match ConfigManager::create_sample_multi_repo_config() {
            Ok(_) => {
                log::info!("✅ Configuration file created successfully!");
                log::info!("📝 Edit the configuration file to add your repositories.");
                log::info!("🔧 Run 'ailyzer validate' to check your configuration.");
            }
            Err(e) => {
                log::error!("❌ Failed to create configuration: {}", e);
                return Err(e);
            }
        }

        Ok(())
    }

    async fn analyze_command(&self, repo: Option<String>, _tags: Vec<String>, _profile: Option<String>) -> AilyzerResult<()> {
        log::info!("🔍 Starting code analysis...");

        let config = match ConfigManager::load() {
            Ok(config) => config,
            Err(e) => {
                log::error!("❌ Failed to load configuration: {}", e);
                log::error!("💡 Run 'ailyzer init' to create a configuration file.");
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

    async fn analyze_single_repository(&self, manager: &mut RepositoryManager, repo_name: &str, results: &mut Vec<Rc<AnalyzeRepositoryResponse>>) -> AilyzerResult<()> {
        log::info!("🎯 Analyzing repository: {}", repo_name);
        let repo_config = manager.config.repositories
            .iter()
            .find(|r| r.name == repo_name)
            .cloned()
            .ok_or_else(|| AilyzerError::repo_error(repo_name, "Line Wrong", "Repository not found"))?;

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

    async fn analyze_all_repositories(&self, manager: &mut RepositoryManager, results: &mut Vec<Rc<AnalyzeRepositoryResponse>>) -> AilyzerResult<()> {
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

    async fn process_repository_result_enhanced(&self, result: Rc<AnalyzeRepositoryResponse>, config: &Config) -> AilyzerResult<()> {
        log::info!("\n{}", "=".repeat(60).as_str());
        log::info!("📊 Processing results for: {}", result.repository_config.name);
        log::info!("{}", "=".repeat(60));

        FileChangeLogger::print_analysis_report(Rc::clone(&result));

        let stats = FileModifier::get_change_statistics(&result.repository_analysis.changes);
        stats.print_summary();

        log::info!("🔍 Validating changes...");
        let validation_result = FileModifier::validate_changes_batch(
            &result.repository_config,
            &result.repository_analysis.changes
        )?;

        validation_result.print_summary();

        if !validation_result.is_valid {
            log::info!("❌ Validation failed. Skipping this repository.");
            return Ok(());
        }

        let application_mode = self.get_application_mode(&stats)?;

        let mut is_there_applied_changes = false;

        match application_mode {
            ApplicationMode::Individual => {
                is_there_applied_changes = self.apply_changes_individually(&result).await?;
            }
            ApplicationMode::Priority => {
                is_there_applied_changes = self.apply_changes_by_priority(&result).await?;
            }
            ApplicationMode::Category => {
                is_there_applied_changes = self.apply_changes_by_category(&result).await?;
            }
            ApplicationMode::Severity => {
                is_there_applied_changes = self.apply_changes_by_severity(&result).await?;
            }
            ApplicationMode::All => {
                is_there_applied_changes = self.apply_all_changes(&result).await?;
            }
            ApplicationMode::Skip => {
                log::info!("⏭️ Skipping changes for this repository.");
                return Ok(());
            }
        }

        if is_there_applied_changes {
            self.handle_post_application_workflow(result, config).await?;
        }

        Ok(())
    }

    fn get_application_mode(&self, stats: &ChangeStatistics) -> AilyzerResult<ApplicationMode> {
        log::info!("\n🎯 How would you like to apply changes?");

        match stats.get_application_strategy() {
            crate::structs::change_statistics::ApplicationStrategy::PriorityBased => {
                log::info!("💡 Recommended: Priority-based application (security/bugs first)");
            }
            crate::structs::change_statistics::ApplicationStrategy::SecurityFirst => {
                log::info!("💡 Recommended: Security-first application");
            }
            crate::structs::change_statistics::ApplicationStrategy::CategoryBased => {
                log::info!("💡 Recommended: Category-based application");
            }
            crate::structs::change_statistics::ApplicationStrategy::AllAtOnce => {
                log::info!("💡 Recommended: Apply all changes at once");
            }
        }

        log::info!("\nOptions:");
        log::info!("  1. 🎯 Priority-based (security → bugs → severity)");
        log::info!("  2. 🏷️ Category-based (group by type)");
        log::info!("  3. ⚡ Severity-based (high severity first)");
        log::info!("  4. 📝 Individual review (one by one)");
        log::info!("  5. 🚀 Apply all at once");
        log::info!("  6. ⏭️ Skip this repository");

        print!("\nSelect option (1-6): ");
        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;

        match input.trim() {
            "1" => Ok(ApplicationMode::Priority),
            "2" => Ok(ApplicationMode::Category),
            "3" => Ok(ApplicationMode::Severity),
            "4" => Ok(ApplicationMode::Individual),
            "5" => Ok(ApplicationMode::All),
            "6" => Ok(ApplicationMode::Skip),
            _ => {
                log::info!("Invalid option, defaulting to priority-based application.");
                Ok(ApplicationMode::Priority)
            }
        }
    }

    async fn handle_post_application_workflow(&self, result: Rc<AnalyzeRepositoryResponse>, config: &Config) -> AilyzerResult<()> {
        log::info!("\n🔄 Post-application workflow...");

        // Create PR if enabled
        if result.repository_config.auto_pr {
            if let Err(e) = self.handle_pr_creation(Rc::clone(&result)).await {
                log::error!("❌ Failed to create PR: {}", e);
            }
        }

        // Save analysis results
        if let Err(e) = self.save_analysis_results(Rc::clone(&result)).await {
            log::error!("❌ Failed to save analysis results: {}", e);
        }

        // Send notifications
        if config.notifications.enabled {
            if let Err(e) = self.send_notifications(Rc::clone(&result)).await {
                log::error!("❌ Failed to send notifications: {}", e);
            }
        }

        Ok(())
    }

    async fn handle_pr_creation(&self, result: Rc<AnalyzeRepositoryResponse>) -> AilyzerResult<()> {
        print!("\n🌿 PR Branch name? (default: \"improvements/ailyzer-apply-changes\"): ");
        io::stdout().flush()?;

        let mut branch = String::new();
        io::stdin().read_line(&mut branch)?;

        if branch.trim().is_empty() {
            branch = "improvements/ailyzer-apply-changes".to_string();
        }

        self.create_pr(result, branch.trim().to_string()).await
    }

    async fn list_command(&self) -> AilyzerResult<()> {
        log::info!("📋 Loading repository configuration...");

        let config = ConfigManager::load()?;

        log::info!("\n📋 Configured Repositories:");
        log::info!("{}", "=".repeat(50));

        if config.repositories.is_empty() {
            log::info!("⚠️ No repositories configured.");
            log::info!("💡 Run 'ailyzer init' to create a configuration file.");
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

    async fn dashboard_command(&self, port: u16) -> AilyzerResult<()> {
        log::info!("🌐 Starting ailyzer dashboard...");
        log::info!("🚀 Dashboard will be available at: http://localhost:{}", port);
        log::info!("⏹️ Press Ctrl+C to stop the dashboard");

        // TODO: Implement web dashboard
        // This would start a web server showing:
        // - Repository analysis history
        // - Change statistics
        // - Real-time analysis progress
        // - Configuration management UI

        log::info!("🚧 Dashboard feature coming soon!");
        log::info!("💡 For now, use the CLI commands to interact with ailyzer.");

        Ok(())
    }

    async fn validate_command(&self) -> AilyzerResult<()> {
        log::info!("🔍 Validating ailyzer configuration...");

        let config = match ConfigManager::load() {
            Ok(config) => {
                log::info!("✅ Configuration file loaded successfully");
                config
            }
            Err(e) => {
                log::error!("❌ Failed to load configuration: {}", e);
                log::error!("💡 Run 'ailyzer init' to create a configuration file.");
                return Err(e);
            }
        };
        ConfigManager::validate_config(Rc::clone(&config))?;
        log::info!("✅ Configuration is valid");
        log::info!("📊 Found {} configured repositories", config.repositories.len());

        self.perform_extended_validation(&config).await?;

        Ok(())
    }

    async fn perform_extended_validation(&self, config: &Config) -> AilyzerResult<()> {
        log::info!("\n🔍 Performing extended validation...");

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

        // Print results
        if issues.is_empty() && warnings.is_empty() {
            log::info!("✅ Extended validation passed - no issues found");
        } else {
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
        }

        Ok(())
    }

    async fn history_command(&self, repo: Option<String>, days: u32) -> AilyzerResult<()> {
        log::info!("📜 Showing analysis history...");

        match repo {
            Some(repo_name) => {
                log::info!("🎯 Repository: {}", repo_name);
            }
            None => {
                log::info!("🌍 All repositories");
            }
        }

        log::info!("📅 Last {} days", days);

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

    async fn create_pr(&self, analyze_repository_response: Rc<AnalyzeRepositoryResponse>, branch: String) -> AilyzerResult<()> {
        log::info!("  📨 Creating PR branch: {}", branch);
        // TODO: Implement PR creation
        Ok(())
    }

    pub async fn save_analysis_results(&self, analyze_repository_response: Rc<AnalyzeRepositoryResponse>) -> AilyzerResult<()> {
        log::info!("  💾 Saving analysis results...");
        // TODO: Implement result saving
        Ok(())
    }

    async fn send_notifications(&self, analyze_repository_response: Rc<AnalyzeRepositoryResponse>) -> AilyzerResult<()> {
        log::info!("  📨 Sending notifications...");
        // TODO: Implement notifications (Slack, email, webhook)
        Ok(())
    }

    async fn apply_changes_individually(&self, result: &AnalyzeRepositoryResponse) -> AilyzerResult<bool> {
        log::info!("\n📝 Individual change review mode");
        let mut approved_changes = Vec::new();

        for (i, change) in result.repository_analysis.changes.iter().enumerate() {
            log::info!("\n{}", "-".repeat(50).as_str());
            log::info!("Change {} of {}", i + 1, result.repository_analysis.changes.len());

            if let Err(e) = FileChangeLogger::print_change_summary(Rc::clone(&result.repository_config), change) {
                log::error!("❌ Error printing change summary: {}", e);
                continue;
            }

            print!("\nApprove this change? (y/N/q to quit): ");
            io::stdout().flush()?;

            let mut input = String::new();
            io::stdin().read_line(&mut input)?;

            match input.trim().to_lowercase().as_str() {
                "y" | "yes" => {
                    approved_changes.push(change);
                    log::info!("✅ Change approved");
                }
                "q" | "quit" => {
                    log::info!("🛑 Stopping individual review.");
                    break;
                }
                _ => {
                    log::info!("⏭️ Skipping this change.");
                }
            }
        }

        if approved_changes.is_empty() {
            log::info!("📊 No changes approved for application");
            return Ok(false);
        }

        log::info!("\n🔧 Applying {} approved changes...", approved_changes.len());

        match FileModifier::apply_changes_grouped_by_file(Arc::new(result.repository_config.as_ref().clone()), approved_changes) {
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

    async fn apply_changes_by_priority(&self, result: &AnalyzeRepositoryResponse) -> AilyzerResult<bool> {
        log::info!("\n🎯 Applying changes in priority order...");
        match FileModifier::apply_changes_grouped_by_file(Arc::new(result.repository_config.as_ref().clone()), result.repository_analysis.changes.iter().collect()) {
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

    async fn apply_changes_by_category(&self, result: &AnalyzeRepositoryResponse) -> AilyzerResult<bool> {
        log::info!("\n🏷️ Category-based application");

        let categories = ["SECURITY", "BUGS", "PERFORMANCE", "ARCHITECTURE", "CLEAN_CODE", "DUPLICATE_CODE"];
        let mut all_approved_changes = Vec::new();

        for category in &categories {
            let category_changes: Vec<_> = result.repository_analysis.changes.iter()
                .filter(|c| c.get_category().as_deref() == Some(category))
                .collect();

            if category_changes.is_empty() {
                continue;
            }

            log::info!("\n📋 Category: {} ({} changes)", category, category_changes.len());

            // Show changes in this category
            for change in &category_changes.clone() {
                if let Err(e) = FileChangeLogger::print_change_summary(Rc::clone(&result.repository_config), change) {
                    log::error!("❌ Error printing change summary: {}", e);
                }
            }

            print!("Apply all {} changes in category {}? (y/N): ", category_changes.len(), category);
            io::stdout().flush()?;

            let mut input = String::new();
            io::stdin().read_line(&mut input)?;

            if input.trim().to_lowercase() == "y" {
                all_approved_changes.extend(category_changes.clone());
                log::info!("✅ Approved {} changes for category {}", category_changes.len(), category);
            }
        }

        if all_approved_changes.is_empty() {
            log::info!("📊 No changes approved for application");
            return Ok(false);
        }

        log::info!("\n🔧 Applying {} approved changes...", all_approved_changes.len());

        match FileModifier::apply_changes_grouped_by_file(Arc::new(result.repository_config.as_ref().clone()), all_approved_changes) {
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

    async fn apply_changes_by_severity(&self, result: &AnalyzeRepositoryResponse) -> AilyzerResult<bool> {
        log::info!("\n⚡ Severity-based application");

        let severities = ["critical", "high", "medium", "low"];
        let mut all_approved_changes = Vec::new();

        // First, get user approval for each severity level
        for severity in &severities {
            let severity_changes: Vec<_> = result.repository_analysis.changes.iter()
                .filter(|c| c.get_severity() == *severity)
                .collect();

            if severity_changes.is_empty() {
                continue;
            }

            log::info!("\n📊 Severity: {} ({} changes)", severity, severity_changes.len());

            // Show changes in this severity level
            for change in &severity_changes {
                if let Err(e) = FileChangeLogger::print_change_summary(Rc::clone(&result.repository_config), change) {
                    log::error!("❌ Error printing change summary: {}", e);
                }
            }

            print!("Apply all {} changes with {} severity? (y/N): ", severity_changes.len(), severity);
            io::stdout().flush()?;

            let mut input = String::new();
            io::stdin().read_line(&mut input)?;

            if input.trim().to_lowercase() == "y" {
                all_approved_changes.extend(severity_changes.clone());
                log::info!("✅ Approved {} changes for severity {}", severity_changes.len(), severity);
            }
        }

        if all_approved_changes.is_empty() {
            log::info!("📊 No changes approved for application");
            return Ok(false);
        }

        log::info!("\n🔧 Applying {} approved changes...", all_approved_changes.len());

        match FileModifier::apply_changes_grouped_by_file(Arc::new(result.repository_config.as_ref().clone()), all_approved_changes) {
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

    async fn apply_all_changes(&self, result: &AnalyzeRepositoryResponse) -> AilyzerResult<bool> {
        log::info!("\n🚀 Applying all changes at once...");

        for change in &result.repository_analysis.changes {
            if let Err(e) = FileChangeLogger::print_change_summary(Rc::clone(&result.repository_config), change) {
                log::error!("❌ Error printing change summary: {}", e);
                continue;
            }
        }

        print!("\nApply all {} changes? (y/N): ", result.repository_analysis.changes.len());
        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;

        if input.trim().to_lowercase() == "y" {
            match FileModifier::apply_changes_grouped_by_file(Arc::new(result.repository_config.as_ref().clone()), result.repository_analysis.changes.iter().collect()) {
                Ok(applied_count) => {
                    log::info!("\n📊 Bulk application complete: {} changes applied", applied_count);
                    Ok(applied_count > 0)
                }
                Err(e) => {
                    log::error!("❌ Failed to apply changes: {}", e);
                    Err(e)
                }
            }
        } else {
            log::info!("⏭️ Skipping all changes.");
            Ok(false)
        }
    }
}