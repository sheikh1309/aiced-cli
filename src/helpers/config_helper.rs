pub struct ConfigHelper;

impl ConfigHelper {
    pub fn default_exclude_patterns() -> Vec<String> {
        vec![
            "tests/".to_string(),
            "test/".to_string(),
            "benches/".to_string(),
            "examples/".to_string(),
            "target/".to_string(),
            "node_modules/".to_string(),
            "dist/".to_string(),
            "build/".to_string(),
            ".git/".to_string(),
            "*.lock".to_string(),
            "*.log".to_string(),
        ]
    }

    pub fn default_scan_interval() -> String {
        "daily".to_string()
    }

    pub fn default_include_patterns() -> Vec<String> {
        vec!["src/".to_string()]
    }

    pub fn default_max_file_size() -> String {
        "1MB".to_string()
    }

    pub fn default_max_files() -> usize {
        100
    }

    pub fn default_languages() -> Vec<String> {
        vec![
            "rust".to_string(),
            "javascript".to_string(),
            "typescript".to_string(),
            "python".to_string(),
        ]
    }

    pub fn default_skip_tests() -> bool {
        true
    }

    pub fn default_chunk_strategy() -> String {
        "smart".to_string()
    }

    pub fn default_model() -> String {
        "claude-3-5-sonnet-20241022".to_string()
    }

    pub fn default_max_tokens() -> u32 {
        8192
    }

    pub fn default_temperature() -> f32 {
        0.0
    }

    pub fn default_provider() -> String {
        "anthropic".to_string()
    }

    pub fn default_format() -> String {
        "custom".to_string()
    }

    pub fn default_save_analysis() -> bool {
        true
    }

    pub fn default_verbose() -> bool {
        false
    }

    pub fn default_check_secrets() -> bool {
        true
    }

    pub fn default_severity_threshold() -> String {
        "low".to_string()
    }
}