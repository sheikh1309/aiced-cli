use serde::{Deserialize, Serialize};
use crate::structs::config::analysis_config::AnalysisConfig;
use crate::structs::config::security_config::SecurityConfig;

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ProfileConfig {
    #[serde(default)]
    pub analysis: AnalysisConfig,

    #[serde(default)]
    pub security: SecurityConfig,
}