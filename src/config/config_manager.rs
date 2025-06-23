use std::path::Path;
use std::fs;
use std::rc::Rc;
use crate::errors::{AicedError, AicedResult};
use crate::structs::config::config::Config;

pub struct ConfigManager;

impl ConfigManager {

    pub fn load() -> AicedResult<Rc<Config>> {
        let config_locations = dirs::home_dir().map(|d| d.join("aiced/config.toml")).unwrap_or_default();

        if config_locations.exists() {
            log::info!("📋 Loading config from: {}", config_locations.display());
            let content = fs::read_to_string(&config_locations)?;
            let config: Config = toml::from_str(&content)?;
            return Ok(Rc::new(config));
        }

        Ok(Rc::new(Config::default()))
    }

    pub fn create_sample_multi_repo_config() -> AicedResult<()> {
        let sample_config = r#"# Aiced Multi-Repository Configuration

[global]
# How often to scan repositories: "hourly", "daily", "weekly", "manual"
scan_interval = "daily"

# Repository definitions
[[repositories]]
name = "backend-api"
path = "/home/user/projects/backend-api"
branch = "main"
auto_pull = true
auto_pr = true

[[repositories]]
name = "frontend-app"
path = "/home/user/projects/frontend-app"
branch = "develop"
auto_pull = false
auto_pr = true

# Output Configuration
[output]
# Directory to store all analysis results
output_dir = "./aiced-results"

# Summary dashboard
generate_dashboard = true
dashboard_port = 8080

# Notifications
[notifications]
on_critical_only = false
summary_report = true
"#;
        let config_file_dir_path = dirs::home_dir().map(|d| d.join("aiced")).unwrap_or_default();
        let config_file_path = dirs::home_dir().map(|d| d.join("aiced/config.toml")).unwrap_or_default();
        fs::create_dir(&config_file_dir_path)?;
        fs::write(&config_file_path, sample_config)?;
        Ok(())
    }

    pub fn validate_config(config: Rc<Config>) -> AicedResult<()>  {
        let mut errors = Vec::new();

        for repo in &config.repositories {
            if !Path::new(&repo.path).exists() {
                errors.push(format!("Repository '{}' path does not exist: {}", repo.name, repo.path));
            }
           
        }

        let mut names = std::collections::HashSet::new();
        for repo in &config.repositories {
            if !names.insert(&repo.name) {
                errors.push(format!("Duplicate repository name: {}", repo.name));
            }
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(AicedError::config_error("Config Error", Some(""), Some("")))
        }
    }
    
}
