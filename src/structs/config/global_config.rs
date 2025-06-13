use serde::{Deserialize, Serialize};
use crate::helpers::config_helper::ConfigHelper;

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct GlobalConfig {
    #[serde(default = "ConfigHelper::default_scan_interval")]
    pub scan_interval: String,

    #[serde(default)]
    pub default_profile: String,

    #[serde(default)]
    pub parallel_repos: usize,

    #[serde(default)]
    pub cache_results: bool,

    #[serde(default)]
    pub results_retention_days: u32,
}

// Default implementations
impl Default for GlobalConfig {
    fn default() -> Self {
        Self {
            scan_interval: ConfigHelper::default_scan_interval(),
            default_profile: "default".to_string(),
            parallel_repos: 1,
            cache_results: true,
            results_retention_days: 30,
        }
    }
}