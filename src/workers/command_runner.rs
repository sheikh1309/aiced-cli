use std::rc::Rc;
use std::sync::Arc;
use std::io::{self, Write};
use crate::enums::commands::Commands;
use crate::config::config_manager::ConfigManager;
use crate::logger::file_change_logger::FileChangeLogger;
use crate::services::file_modifier::FileModifier;
use crate::services::repository_manager::RepositoryManager;
use crate::structs::analyze_repository_response::AnalyzeRepositoryResponse;

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

            manager.analyze_repository(Arc::new(repo_config), &mut results).await?;
        } else {
            manager.analyze_all_repositories(&mut results).await?;
        }
        
        println!("\n✅ Analysis complete for {} repositories\n", results.len());
        
        for result in results {
            FileChangeLogger::print_analysis_report(Rc::clone(&result));
            let mut is_there_applied_changes = false;
            for change in &result.repository_analysis.changes {
                FileChangeLogger::print_change_summary(Rc::clone(&result.repository_config), change)?;
            }

            print!("\nApply changes? (y/N): ");
            io::stdout().flush()?;

            let mut input = String::new();
            io::stdin().read_line(&mut input)?;

            if input.trim().to_lowercase() == "y" {
                for change in &result.repository_analysis.changes {
                    FileModifier::apply_change(Arc::new(result.repository_config.as_ref().clone()), change)?;
                    is_there_applied_changes = true;
                }
            }
            
            if is_there_applied_changes {
                if result.repository_config.auto_pr {
                    print!("\nPR Branch name? (default: \"improvements/aiLyzer-apply-changes\"): ");
                    io::stdout().flush()?;

                    let mut branch = String::new();
                    io::stdin().read_line(&mut branch)?;
                    if branch.trim().to_lowercase() == "" {
                        branch = "improvements/aiLyzer-apply-changes".to_string();
                    }
                    self.create_pr(Rc::clone(&result), branch).await?;
                }

                self.save_analysis_results(Rc::clone(&result)).await?;

                if config.notifications.enabled {
                    self.send_notifications(Rc::clone(&result)).await?;
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