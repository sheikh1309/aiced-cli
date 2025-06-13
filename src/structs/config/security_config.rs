use serde::{Deserialize, Serialize};
use crate::helpers::config_helper::ConfigHelper;

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct SecurityConfig {
    #[serde(default = "ConfigHelper::default_check_secrets")]
    pub check_secrets: bool,

    #[serde(default)]
    pub secret_patterns: Vec<String>,

    #[serde(default = "ConfigHelper::default_severity_threshold")]
    pub severity_threshold: String,
}

impl Default for SecurityConfig {
    fn default() -> Self {
        Self {
            check_secrets: ConfigHelper::default_check_secrets(),
            secret_patterns: vec![
                r"api[_-]?key".to_string(),
                r"secret[_-]?key".to_string(),
                r"password".to_string(),
                r"token".to_string(),
            ],
            severity_threshold: ConfigHelper::default_severity_threshold(),
        }
    }
}