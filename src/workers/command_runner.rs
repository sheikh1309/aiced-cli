use crate::config::config_manager::ConfigManager;
use crate::enums::commands::Commands;
use crate::services::repository_manager::RepositoryManager;

pub struct CommandRunner;

impl CommandRunner {

    pub async fn run_command(command: Commands) -> Result<(), Box<dyn std::error::Error>> {
        match command {
            Commands::Init => {
                ConfigManager::create_sample_multi_repo_config()?;
            }

            Commands::Analyze { repo, tags, profile } => {
                let config = ConfigManager::load()?;
                if let Err(errors) = ConfigManager::validate_config(&config) {
                    return Err(errors.join("\n").into());
                }

                let mut manager = RepositoryManager::new(config);

                if let Some(repo_name) = repo {
                    let repo_config = manager.config.repositories
                        .iter()
                        .find(|r| r.name == repo_name)
                        .cloned()
                        .ok_or_else(|| format!("Repository not found: {}", repo_name))?;

                    let result = manager.analyze_repository(&repo_config).await?;
                    println!("\n✅ Analysis complete: {:?}", result);
                } else {
                    // Analyze all repositories
                    let results = manager.analyze_all_repositories().await?;

                    // Print summary
                    println!("\n📊 Analysis Summary:");
                    for result in results {
                        println!("  - {}: {} issues ({} critical)",
                                 result.repository,
                                 result.issues_found,
                                 result.critical_issues
                        );
                    }
                }
            }

            Commands::List { all } => {
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
            }

            Commands::Dashboard { port } => {
                println!("🌐 Starting dashboard on http://localhost:{}", port);
                // Implement web dashboard here
            }

            Commands::Validate => {
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
            }

            Commands::History { repo, days } => {
                // Implement history viewing
                println!("📜 Showing history for last {} days", days);
            }
        }

        Ok(())
    }
}