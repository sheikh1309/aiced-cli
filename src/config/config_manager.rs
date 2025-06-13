use std::path::Path;
use std::fs;
use crate::structs::config::config::Config;

pub struct ConfigManager;

impl ConfigManager {

    pub fn load() -> Result<Config, Box<dyn std::error::Error>> {
        let config_locations = dirs::home_dir().map(|d| d.join("ailyzer/config.toml")).unwrap_or_default();

        if config_locations.exists() {
            println!("📋 Loading config from: {}", config_locations.display());
            let content = fs::read_to_string(&config_locations)?;
            let config: Config = toml::from_str(&content)?;
            return Ok(config);
        }

        Ok(Config::default())
    }

    pub fn create_sample_multi_repo_config() -> Result<(), Box<dyn std::error::Error>> {
        let sample_config = r#"# AiLyzer Multi-Repository Configuration

[global]
# How often to scan repositories: "hourly", "daily", "weekly", "manual"
scan_interval = "daily"

# Default profile to use for repositories
default_profile = "standard"

# Number of repositories to analyze in parallel
parallel_repos = 2

# Cache analysis results
cache_results = true

# How long to keep historical results (days)
results_retention_days = 30

# Repository definitions
[[repositories]]
name = "backend-api"
path = "/home/user/projects/backend-api"
enabled = true
branch = "main"
auto_pull = true
schedule = "0 2 * * *"  # Cron: 2 AM daily
tags = ["backend", "api", "critical"]
priority = "high"
profile = "strict"

[[repositories]]
name = "frontend-app"
path = "/home/user/projects/frontend-app"
enabled = true
branch = "develop"
auto_pull = false
tags = ["frontend", "react"]
priority = "medium"
profile = "standard"

[[repositories]]
name = "microservice-auth"
path = "/home/user/projects/services/auth"
enabled = true
remote_url = "git@github.com:company/auth-service.git"
auto_pull = true
tags = ["microservice", "security", "critical"]
priority = "critical"
profile = "security-focused"


# Profile definitions
[profiles.default]
[profiles.default.analysis]
exclude_patterns = ["tests/", "node_modules/", "target/", "dist/"]
include_patterns = ["src/"]
max_file_size = "1MB"
max_files = 100
languages = ["rust", "javascript", "typescript", "python"]
chunk_strategy = "smart"

[profiles.default.security]
check_secrets = true
severity_threshold = "medium"

[profiles.strict]
[profiles.strict.analysis]
exclude_patterns = ["node_modules/", "target/"]
include_patterns = ["src/", "tests/"]  # Include tests
max_file_size = "500KB"
max_files = 200
skip_tests = false

[profiles.standard]
[profiles.standard.analysis]
exclude_patterns = ["node_modules/", "target/"]
include_patterns = ["src/", "tests/"]  # Include tests
max_file_size = "500KB"
max_files = 200
skip_tests = false

[profiles.strict.security]
check_secrets = true
severity_threshold = "low"  # Report everything

[profiles.security-focused]
[profiles.security-focused.analysis]
focus_areas = ["security", "authentication", "authorization", "crypto"]

[profiles.security-focused.security]
check_secrets = true
severity_threshold = "low"
secret_patterns = [
    "api[_-]?key",
    "secret",
    "password",
    "token",
    "private[_-]?key",
    "auth",
    "credential"
]

[profiles.lenient]
[profiles.lenient.analysis]
exclude_patterns = ["tests/", "examples/", "docs/", "migrations/"]
max_file_size = "2MB"
skip_tests = true

[profiles.lenient.security]
severity_threshold = "high"  # Only critical issues

# AI Configuration
[ai]
provider = "anthropic"
model = "claude-3-5-sonnet-20241022"
max_tokens = 8192
temperature = 0.0
api_key_env = "ANTHROPIC_API_KEY"

# Rate limiting per repository
rate_limit_per_minute = 10
rate_limit_concurrent = 3

# Output Configuration
[output]
# Directory to store all analysis results
output_dir = "./ailyzer-results"

# Organize by: "date", "repository", "severity"
organization = "repository"

# Formats to generate
formats = ["json", "markdown", "html"]

# Summary dashboard
generate_dashboard = true
dashboard_port = 8080

# Notifications
[notifications]
on_critical_only = false
summary_report = true
"#;
        let config_file_dir_path = dirs::home_dir().map(|d| d.join("ailyzer")).unwrap_or_default();
        let config_file_path = dirs::home_dir().map(|d| d.join("ailyzer/config.toml")).unwrap_or_default();
        fs::create_dir(&config_file_dir_path)?;
        fs::write(&config_file_path, sample_config)?;
        println!("✅ Created sample multi-repo config at: {}", config_file_path.display());
        Ok(())
    }

    pub fn validate_config(config: &Config) -> Result<(), Vec<String>> {
        let mut errors = Vec::new();

        for repo in &config.repositories {
            if repo.enabled && !Path::new(&repo.path).exists() {
                errors.push(format!("Repository '{}' path does not exist: {}", repo.name, repo.path));
            }

            if let Some(profile) = &repo.profile {
                if !config.profiles.contains_key(profile) {
                    errors.push(format!("Repository '{}' references unknown profile: {}", repo.name, profile));
                }
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
            Err(errors)
        }
    }
    
}
