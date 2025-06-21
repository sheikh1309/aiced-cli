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
use crate::structs::apply_result::ApplyResult;
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
            println!("\n⏱️ Command completed in {:.2}s", duration.as_secs_f64());
        }

        result
    }

    async fn init_command(&self) -> AilyzerResult<()> {
        println!("🚀 Initializing ailyzer configuration...");

        match ConfigManager::create_sample_multi_repo_config() {
            Ok(_) => {
                println!("✅ Configuration file created successfully!");
                println!("📝 Edit the configuration file to add your repositories.");
                println!("🔧 Run 'ailyzer validate' to check your configuration.");
            }
            Err(e) => {
                eprintln!("❌ Failed to create configuration: {}", e);
                return Err(e);
            }
        }

        Ok(())
    }

    async fn analyze_command(&self, repo: Option<String>, _tags: Vec<String>, _profile: Option<String>) -> AilyzerResult<()> {
        println!("🔍 Starting code analysis...");

        let config = match ConfigManager::load() {
            Ok(config) => config,
            Err(e) => {
                eprintln!("❌ Failed to load configuration: {}", e);
                eprintln!("💡 Run 'ailyzer init' to create a configuration file.");
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
            println!("⚠️ No repositories were successfully analyzed.");
            println!("💡 Check the errors above and verify your configuration.");
            return Ok(());
        }

        println!("\n✅ Analysis complete for {} repositories", results.len());
        
        for result in results {
            if let Err(e) = self.process_repository_result_enhanced(result, &config).await {
                eprintln!("❌ Error processing repository results: {}", e);
                eprintln!("   Continuing with next repository...");
            }
        }

        Ok(())
    }

    async fn analyze_single_repository(&self, manager: &mut RepositoryManager, repo_name: &str, results: &mut Vec<Rc<AnalyzeRepositoryResponse>>) -> AilyzerResult<()> {
        println!("🎯 Analyzing repository: {}", repo_name);
        let repo_config = manager.config.repositories
            .iter()
            .find(|r| r.name == repo_name)
            .cloned()
            .ok_or_else(|| AilyzerError::repo_error(repo_name, "Line Wrong", "Repository not found"))?;

        match manager.analyze_repository(Arc::new(repo_config.clone()), results).await {
            Ok(_) => {
                println!("✅ Successfully analyzed repository: {}", repo_config.name);
            }
            Err(e) => {
                eprintln!("❌ Failed to analyze repository '{}': {}", repo_config.name, e);
                return Err(e);
            }
        }

        Ok(())
    }

    async fn analyze_all_repositories(&self, manager: &mut RepositoryManager, results: &mut Vec<Rc<AnalyzeRepositoryResponse>>) -> AilyzerResult<()> {
        println!("🌍 Analyzing all configured repositories...");

        match manager.analyze_all_repositories(results).await {
            Ok(_) => {
                println!("✅ Successfully analyzed all repositories");
            }
            Err(e) => {
                eprintln!("❌ Error during repository analysis: {}", e);
                eprintln!("   Some repositories may have failed. Continuing with successful ones...");
            }
        }

        Ok(())
    }

    async fn process_repository_result_enhanced(&self, result: Rc<AnalyzeRepositoryResponse>, config: &Config) -> AilyzerResult<()> {
        println!("\n{}", "=".repeat(60).as_str());
        println!("📊 Processing results for: {}", result.repository_config.name);
        println!("{}", "=".repeat(60));

        // Print analysis report
        FileChangeLogger::print_analysis_report(Rc::clone(&result));

        // Get comprehensive statistics
        let stats = FileModifier::get_change_statistics(&result.repository_analysis.changes);
        stats.print_summary();

        // Validate changes before applying
        println!("\n🔍 Validating changes...");
        let validation_result = FileModifier::validate_changes_batch(
            &result.repository_config,
            &result.repository_analysis.changes
        )?;

        validation_result.print_summary();

        if !validation_result.is_valid {
            println!("❌ Validation failed. Skipping this repository.");
            return Ok(());
        }

        // Get user's application preference
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
                println!("⏭️ Skipping changes for this repository.");
                return Ok(());
            }
        }

        if is_there_applied_changes {
            self.handle_post_application_workflow(result, config).await?;
        }

        Ok(())
    }

    fn get_application_mode(&self, stats: &ChangeStatistics) -> AilyzerResult<ApplicationMode> {
        println!("\n🎯 How would you like to apply changes?");

        match stats.get_application_strategy() {
            crate::structs::change_statistics::ApplicationStrategy::PriorityBased => {
                println!("💡 Recommended: Priority-based application (security/bugs first)");
            }
            crate::structs::change_statistics::ApplicationStrategy::SecurityFirst => {
                println!("💡 Recommended: Security-first application");
            }
            crate::structs::change_statistics::ApplicationStrategy::CategoryBased => {
                println!("💡 Recommended: Category-based application");
            }
            crate::structs::change_statistics::ApplicationStrategy::AllAtOnce => {
                println!("💡 Recommended: Apply all changes at once");
            }
        }

        println!("\nOptions:");
        println!("  1. 🎯 Priority-based (security → bugs → severity)");
        println!("  2. 🏷️ Category-based (group by type)");
        println!("  3. ⚡ Severity-based (high severity first)");
        println!("  4. 📝 Individual review (one by one)");
        println!("  5. 🚀 Apply all at once");
        println!("  6. ⏭️ Skip this repository");

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
                println!("Invalid option, defaulting to priority-based application.");
                Ok(ApplicationMode::Priority)
            }
        }
    }

    async fn apply_changes_individually(&self, result: &AnalyzeRepositoryResponse) -> AilyzerResult<bool> {
        println!("\n📝 Individual change review mode");
        let mut applied_count = 0;

        for (i, change) in result.repository_analysis.changes.iter().enumerate() {
            println!("\n{}", "-".repeat(50).as_str());
            println!("Change {} of {}", i + 1, result.repository_analysis.changes.len());

            if let Err(e) = FileChangeLogger::print_change_summary(Rc::clone(&result.repository_config), change) {
                eprintln!("❌ Error printing change summary: {}", e);
                continue;
            }

            print!("\nApply this change? (y/N/q to quit): ");
            io::stdout().flush()?;

            let mut input = String::new();
            io::stdin().read_line(&mut input)?;

            match input.trim().to_lowercase().as_str() {
                "y" | "yes" => {
                    match FileModifier::apply_change_with_logging(Arc::new(result.repository_config.as_ref().clone()), change) {
                        Ok(_) => {
                            applied_count += 1;
                            println!("✅ Change applied successfully");
                        }
                        Err(e) => {
                            eprintln!("❌ Failed to apply change: {}", e);
                        }
                    }
                }
                "q" | "quit" => {
                    println!("🛑 Stopping individual review.");
                    break;
                }
                _ => {
                    println!("⏭️ Skipping this change.");
                }
            }
        }

        println!("\n📊 Individual review complete: {} changes applied", applied_count);
        Ok(applied_count > 0)
    }

    async fn apply_changes_by_priority(&self, result: &AnalyzeRepositoryResponse) -> AilyzerResult<bool> {
        println!("\n🎯 Applying changes in priority order...");

        let apply_result = FileModifier::apply_changes_by_priority(
            Arc::new(result.repository_config.as_ref().clone()),
            &result.repository_analysis.changes
        )?;

        self.print_apply_result(&apply_result);
        Ok(apply_result.total_applied > 0)
    }

    async fn apply_changes_by_category(&self, result: &AnalyzeRepositoryResponse) -> AilyzerResult<bool> {
        println!("\n🏷️ Category-based application");

        let categories = ["SECURITY", "BUGS", "PERFORMANCE", "ARCHITECTURE", "CLEAN_CODE", "DUPLICATE_CODE"];
        let mut total_applied = 0;

        for category in &categories {
            let category_changes: Vec<_> = result.repository_analysis.changes.iter()
                .filter(|c| c.get_category().as_deref() == Some(category))
                .collect();

            if category_changes.is_empty() {
                continue;
            }

            println!("\n📋 Category: {} ({} changes)", category, category_changes.len());
            print!("Apply all {} changes? (y/N): ", category);
            io::stdout().flush()?;

            let mut input = String::new();
            io::stdin().read_line(&mut input)?;

            if input.trim().to_lowercase() == "y" {
                match FileModifier::apply_changes_by_category(
                    Arc::new(result.repository_config.as_ref().clone()),
                    &result.repository_analysis.changes,
                    category
                ) {
                    Ok(applied) => {
                        total_applied += applied;
                        println!("✅ Applied {} changes for category {}", applied, category);
                    }
                    Err(e) => {
                        eprintln!("❌ Failed to apply changes for category {}: {}", category, e);
                    }
                }
            }
        }

        println!("\n📊 Category-based application complete: {} total changes applied", total_applied);
        Ok(total_applied > 0)
    }

    async fn apply_changes_by_severity(&self, result: &AnalyzeRepositoryResponse) -> AilyzerResult<bool> {
        println!("\n⚡ Severity-based application");

        let severities = ["critical", "high", "medium", "low"];
        let mut total_applied = 0;

        for severity in &severities {
            let severity_changes: Vec<_> = result.repository_analysis.changes.iter()
                .filter(|c| c.get_severity() == *severity)
                .collect();

            if severity_changes.is_empty() {
                continue;
            }

            println!("\n📊 Severity: {} ({} changes)", severity, severity_changes.len());
            print!("Apply all {} changes? (y/N): ", severity);
            io::stdout().flush()?;

            let mut input = String::new();
            io::stdin().read_line(&mut input)?;

            if input.trim().to_lowercase() == "y" {
                match FileModifier::apply_changes_by_severity(
                    Arc::new(result.repository_config.as_ref().clone()),
                    &result.repository_analysis.changes,
                    severity
                ) {
                    Ok(applied) => {
                        total_applied += applied;
                        println!("✅ Applied {} changes for severity {}", applied, severity);
                    }
                    Err(e) => {
                        eprintln!("❌ Failed to apply changes for severity {}: {}", severity, e);
                    }
                }
            }
        }

        println!("\n📊 Severity-based application complete: {} total changes applied", total_applied);
        Ok(total_applied > 0)
    }

    async fn apply_all_changes(&self, result: &AnalyzeRepositoryResponse) -> AilyzerResult<bool> {
        println!("\n🚀 Applying all changes at once...");

        // Show summary first
        for change in &result.repository_analysis.changes {
            if let Err(e) = FileChangeLogger::print_change_summary(Rc::clone(&result.repository_config), change) {
                eprintln!("❌ Error printing change summary: {}", e);
                continue;
            }
        }

        print!("\nApply all {} changes? (y/N): ", result.repository_analysis.changes.len());
        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;

        if input.trim().to_lowercase() == "y" {
            let mut applied_count = 0;

            for change in &result.repository_analysis.changes {
                match FileModifier::apply_change_with_logging(Arc::new(result.repository_config.as_ref().clone()), change) {
                    Ok(_) => {
                        applied_count += 1;
                    }
                    Err(e) => {
                        eprintln!("❌ Failed to apply change: {}", e);
                    }
                }
            }

            println!("\n📊 Bulk application complete: {} of {} changes applied",
                     applied_count, result.repository_analysis.changes.len());
            Ok(applied_count > 0)
        } else {
            println!("⏭️ Skipping all changes.");
            Ok(false)
        }
    }

    fn print_apply_result(&self, result: &ApplyResult) {
        println!("\n📊 Priority Application Results:");
        println!("   🔒 Security: {}", result.security_applied);
        println!("   🐛 Bugs: {}", result.bugs_applied);
        println!("   🚀 Performance: {}", result.performance_applied);
        println!("   🏗️ Architecture: {}", result.architecture_applied);
        println!("   ✨ Clean Code: {}", result.clean_code_applied);
        println!("   🔄 Duplicate Code: {}", result.duplicate_code_applied);
        println!("   ❌ Failed: {}", result.failed);
        println!("   📈 Total Applied: {}", result.total_applied);
    }

    async fn handle_post_application_workflow(&self, result: Rc<AnalyzeRepositoryResponse>, config: &Config) -> AilyzerResult<()> {
        println!("\n🔄 Post-application workflow...");

        // Create PR if enabled
        if result.repository_config.auto_pr {
            if let Err(e) = self.handle_pr_creation(Rc::clone(&result)).await {
                eprintln!("❌ Failed to create PR: {}", e);
            }
        }

        // Save analysis results
        if let Err(e) = self.save_analysis_results(Rc::clone(&result)).await {
            eprintln!("❌ Failed to save analysis results: {}", e);
        }

        // Send notifications
        if config.notifications.enabled {
            if let Err(e) = self.send_notifications(Rc::clone(&result)).await {
                eprintln!("❌ Failed to send notifications: {}", e);
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
        println!("📋 Loading repository configuration...");

        let config = ConfigManager::load()?;

        println!("\n📋 Configured Repositories:");
        println!("{}", "=".repeat(50));

        if config.repositories.is_empty() {
            println!("⚠️ No repositories configured.");
            println!("💡 Run 'ailyzer init' to create a configuration file.");
            return Ok(());
        }

        for (i, repo) in config.repositories.iter().enumerate() {
            println!("{}. ✅ {}", i + 1, repo.name);
            println!("   📁 Path: {}", repo.path);
            println!("   🔧 Auto PR: {}", if repo.auto_pr { "✅" } else { "❌" });

            println!();
        }

        println!("📊 Total repositories: {}", config.repositories.len());
        Ok(())
    }

    async fn dashboard_command(&self, port: u16) -> AilyzerResult<()> {
        println!("🌐 Starting ailyzer dashboard...");
        println!("🚀 Dashboard will be available at: http://localhost:{}", port);
        println!("⏹️ Press Ctrl+C to stop the dashboard");

        // TODO: Implement web dashboard
        // This would start a web server showing:
        // - Repository analysis history
        // - Change statistics
        // - Real-time analysis progress
        // - Configuration management UI

        println!("🚧 Dashboard feature coming soon!");
        println!("💡 For now, use the CLI commands to interact with ailyzer.");

        Ok(())
    }

    async fn validate_command(&self) -> AilyzerResult<()> {
        println!("🔍 Validating ailyzer configuration...");

        let config = match ConfigManager::load() {
            Ok(config) => {
                println!("✅ Configuration file loaded successfully");
                config
            }
            Err(e) => {
                eprintln!("❌ Failed to load configuration: {}", e);
                eprintln!("💡 Run 'ailyzer init' to create a configuration file.");
                return Err(e);
            }
        };
        ConfigManager::validate_config(Rc::clone(&config))?;
        println!("✅ Configuration is valid");
        println!("📊 Found {} configured repositories", config.repositories.len());

        self.perform_extended_validation(&config).await?;

        Ok(())
    }

    async fn perform_extended_validation(&self, config: &Config) -> AilyzerResult<()> {
        println!("\n🔍 Performing extended validation...");

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
            println!("✅ Extended validation passed - no issues found");
        } else {
            if !issues.is_empty() {
                println!("❌ Issues found:");
                for issue in &issues {
                    println!("   - {}", issue);
                }
            }

            if !warnings.is_empty() {
                println!("⚠️ Warnings:");
                for warning in &warnings {
                    println!("   - {}", warning);
                }
            }
        }

        Ok(())
    }

    async fn history_command(&self, repo: Option<String>, days: u32) -> AilyzerResult<()> {
        println!("📜 Showing analysis history...");

        match repo {
            Some(repo_name) => {
                println!("🎯 Repository: {}", repo_name);
            }
            None => {
                println!("🌍 All repositories");
            }
        }

        println!("📅 Last {} days", days);

        // TODO: Implement history functionality
        // This would show:
        // - Previous analysis results
        // - Changes applied over time
        // - Success/failure rates
        // - Performance metrics

        println!("🚧 History feature coming soon!");
        println!("💡 Analysis results will be stored and displayed here.");

        Ok(())
    }

    async fn create_pr(&self, analyze_repository_response: Rc<AnalyzeRepositoryResponse>, branch: String) -> AilyzerResult<()> {
        println!("  📨 Creating PR branch: {}", branch);
        // TODO: Implement PR creation
        Ok(())
    }

    pub async fn save_analysis_results(&self, analyze_repository_response: Rc<AnalyzeRepositoryResponse>) -> AilyzerResult<()> {
        println!("  💾 Saving analysis results...");
        // TODO: Implement result saving
        Ok(())
    }

    async fn send_notifications(&self, analyze_repository_response: Rc<AnalyzeRepositoryResponse>) -> AilyzerResult<()> {
        println!("  📨 Sending notifications...");
        // TODO: Implement notifications (Slack, email, webhook)
        Ok(())
    }
}

impl Default for CommandRunner {
    fn default() -> Self {
        Self::new()
    }
}
