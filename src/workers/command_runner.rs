use std::rc::Rc;
use std::sync::Arc;
use std::io::{self, Write};
use crate::enums::commands::Commands;
use crate::config::config_manager::ConfigManager;
use crate::logger::file_change_logger::FileChangeLogger;
use crate::services::file_modifier::FileModifier;
use crate::services::repository_manager::RepositoryManager;
use crate::structs::analyze_repository_response::AnalyzeRepositoryResponse;
use crate::structs::config::config::Config;

pub struct CommandRunner;

impl CommandRunner {

    pub fn new() -> Self {
        Self {}
    }

    pub async fn run_command(&self, command: Commands) -> Result<(), Box<dyn std::error::Error>> {
        match command {
            Commands::Init => ConfigManager::create_sample_multi_repo_config()?,
            Commands::Analyze { repo, .. } => self.analyze_repositories(repo).await?,
            Commands::List { .. } => self.list()?,
            Commands::Dashboard { port } => self.dashboard(port)?,
            Commands::Validate => self.validate()?,
            Commands::History { repo, days } => println!("📜 Showing history for {:?} in last {} days", repo, days)
        }

        Ok(())
    }

    async fn analyze_repositories(&self, repo: Option<String>) -> Result<(), Box<dyn std::error::Error>> {
        let config = ConfigManager::load()?;
        if let Err(errors) = ConfigManager::validate_config(Rc::clone(&config)) {
            return Err(errors.join("\n").into());
        }

        let mut results: Vec<Rc<AnalyzeRepositoryResponse>> = Vec::new();
        let mut manager = RepositoryManager::new(Rc::clone(&config));

        if let Some(repo_name) = repo {
            let repo_config = manager.config.repositories
                .iter()
                .find(|r| r.name == repo_name)
                .cloned()
                .ok_or_else(|| format!("Repository not found: {}", repo_name))?;

            // Handle single repository analysis errors
            match manager.analyze_repository(Arc::new(repo_config.clone()), &mut results).await {
                Ok(_) => {
                    println!("✅ Successfully analyzed repository: {}", repo_config.name);
                }
                Err(e) => {
                    eprintln!("❌ Failed to analyze repository '{}': {}", repo_config.name, e);
                    eprintln!("   Continuing with next operations...");
                    // Don't return error - just log and continue
                }
            }
        } else {
            // Handle multiple repositories analysis errors
            match manager.analyze_all_repositories(&mut results).await {
                Ok(_) => {
                    println!("✅ Successfully analyzed all repositories");
                }
                Err(e) => {
                    eprintln!("❌ Error during repository analysis: {}", e);
                    eprintln!("   Some repositories may have failed. Continuing with successful ones...");
                    // Don't return error - just log and continue with any successful results
                }
            }
        }

        if results.is_empty() {
            println!("⚠️  No repositories were successfully analyzed. Please check the errors above.");
            return Ok(()); // Return success but with no results to process
        }

        println!("\n✅ Analysis complete for {} repositories\n", results.len());

        for result in results {
            if let Err(e) = self.process_repository_result(result, &config).await {
                eprintln!("❌ Error processing repository results: {}", e);
                eprintln!("   Continuing with next repository...");
                // Continue with next repository instead of stopping
            }
        }

        Ok(())
    }

    async fn process_repository_result(&self, result: Rc<AnalyzeRepositoryResponse>, config: &Config) -> Result<(), Box<dyn std::error::Error>> {
        FileChangeLogger::print_analysis_report(Rc::clone(&result));

        print!("\nReview changes individually? (y/N): ");
        io::stdout().flush()?;

        let mut review_mode = String::new();
        io::stdin().read_line(&mut review_mode)?;
        let individual_review = review_mode.trim().to_lowercase() == "y";

        let mut is_there_applied_changes = false;

        if individual_review {
            // Individual review mode
            for change in &result.repository_analysis.changes {
                if let Err(e) = FileChangeLogger::print_change_summary(Rc::clone(&result.repository_config), change) {
                    eprintln!("❌ Error printing change summary: {}", e);
                    continue; // Skip this change and continue with next
                }

                print!("\nApply this change? (y/N): ");
                io::stdout().flush()?;

                let mut input = String::new();
                io::stdin().read_line(&mut input)?;

                if input.trim().to_lowercase() == "y" {
                    match FileModifier::apply_change(Arc::new(result.repository_config.as_ref().clone()), change) {
                        Ok(_) => {
                            is_there_applied_changes = true;
                            println!("✅ Change applied successfully");
                        }
                        Err(e) => {
                            eprintln!("❌ Failed to apply change: {}", e);
                            eprintln!("   Continuing with next change...");
                            // Continue with next change instead of failing completely
                        }
                    }
                }
            }
        } else {
            // Bulk review mode (original behavior)
            for change in &result.repository_analysis.changes {
                if let Err(e) = FileChangeLogger::print_change_summary(Rc::clone(&result.repository_config), change) {
                    eprintln!("❌ Error printing change summary: {}", e);
                    continue; // Skip this change and continue with next
                }
            }

            print!("\nApply all changes? (y/N): ");
            io::stdout().flush()?;

            let mut input = String::new();
            io::stdin().read_line(&mut input)?;

            if input.trim().to_lowercase() == "y" {
                for change in &result.repository_analysis.changes {
                    match FileModifier::apply_change(Arc::new(result.repository_config.as_ref().clone()), change) {
                        Ok(_) => {
                            is_there_applied_changes = true;
                            println!("✅ Change applied successfully");
                        }
                        Err(e) => {
                            eprintln!("❌ Failed to apply change: {}", e);
                            eprintln!("   Continuing with next change...");
                            // Continue with next change instead of failing completely
                        }
                    }
                }
            }
        }

        if is_there_applied_changes {
            if result.repository_config.auto_pr {
                print!("\nPR Branch name? (default: \"improvements/aiLyzer-apply-changes\"): ");
                io::stdout().flush()?;

                let mut branch = String::new();
                io::stdin().read_line(&mut branch)?;
                if branch.trim().is_empty() {
                    branch = "improvements/aiLyzer-apply-changes".to_string();
                }

                if let Err(e) = self.create_pr(Rc::clone(&result), branch).await {
                    eprintln!("❌ Failed to create PR: {}", e);
                    eprintln!("   Continuing with other operations...");
                }
            }

            if let Err(e) = self.save_analysis_results(Rc::clone(&result)).await {
                eprintln!("❌ Failed to save analysis results: {}", e);
                eprintln!("   Continuing with other operations...");
            }

            if config.notifications.enabled {
                if let Err(e) = self.send_notifications(Rc::clone(&result)).await {
                    eprintln!("❌ Failed to send notifications: {}", e);
                    eprintln!("   Continuing with other operations...");
                }
            }
        }

        Ok(())
    }

    fn list(&self) -> Result<(), Box<dyn std::error::Error>> {
        let config = ConfigManager::load()?;

        println!("📋 Configured Repositories:\n");
        for repo in &config.repositories {
            println!("  ✅ {}", repo.name);
            println!("     Path: {}", repo.path);
            println!();
        }

        Ok(())
    }

    fn dashboard(&self, port: u16) -> Result<(), Box<dyn std::error::Error>> {
        println!("🌐 Starting dashboard on http://localhost:{}", port);
        // Implement web dashboard here
        Ok(())
    }

    fn validate(&self) -> Result<(), Box<dyn std::error::Error>> {
        let config = ConfigManager::load()?;
        match ConfigManager::validate_config(Rc::clone(&config)) {
            Ok(_) => println!("✅ Configuration is valid"),
            Err(errors) => {
                println!("❌ Configuration errors:");
                for error in errors {
                    println!("  - {}", error);
                }
                return Ok(());
            }
        }

        Ok(())
    }

    // todo - set this in other file
    async fn create_pr(&self, analyze_repository_response: Rc<AnalyzeRepositoryResponse>, branch: String) -> Result<(), Box<dyn std::error::Error>> {
        println!("  📨 Creating PR branch: {}", branch);
        // todo
        Ok(())
    }

    pub async fn save_analysis_results(&self, analyze_repository_response: Rc<AnalyzeRepositoryResponse>) -> Result<(), Box<dyn std::error::Error>> {
        println!("  💾 Saving analysis results...");
        // todo
        Ok(())
    }

    async fn send_notifications(&self, analyze_repository_response: Rc<AnalyzeRepositoryResponse>) -> Result<(), Box<dyn std::error::Error>> {
        // Implement Slack, email, webhook notifications
        println!("  📨 Sending notifications...");
        // todo
        Ok(())
    }
}