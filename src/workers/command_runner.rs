use std::rc::Rc;
use std::sync::Arc;
use crate::config::config_manager::ConfigManager;
use crate::enums::commands::Commands;
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
            Commands::List { all } => self.list(all)?,
            Commands::Dashboard { port } => self.dashboard(port)?,
            Commands::Validate => self.validate()?,
            Commands::History { repo, days } => println!("📜 Showing history for {:?} in last {} days", repo, days)
        }

        Ok(())
    }

    async fn analyze_repositories(&self, repo: Option<String>) -> Result<(), Box<dyn std::error::Error>> {
        let config = ConfigManager::load()?;
        if let Err(errors) = ConfigManager::validate_config(&config) {
            return Err(errors.join("\n").into());
        }

        let mut results: Vec<Rc<AnalyzeRepositoryResponse>> = Vec::new();
        let mut manager = RepositoryManager::new(config);

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
        
        println!("\n✅ Analysis complete: {:?}", results);
        
        // apply changes

        Ok(())
    }

    fn list(&self, all: bool) -> Result<(), Box<dyn std::error::Error>> {
        let config = ConfigManager::load()?;

        println!("📋 Configured Repositories:\n");
        for repo in &config.repositories {
            if !all && !repo.enabled {
                continue;
            }

            println!("  {} {}",
                     if repo.enabled { "✅" } else { "❌" },
                     repo.name
            );
            println!("     Path: {}", repo.path);
            println!("     Priority: {:?}", repo.priority);
            println!("     Tags: {:?}", repo.tags);
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
        match ConfigManager::validate_config(&config) {
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
}