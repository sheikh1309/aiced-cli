use crate::helpers::config_helper::ConfigHelper;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct AnalysisConfig {
    #[serde(default = "ConfigHelper::default_exclude_patterns")]
    pub exclude_patterns: Vec<String>,

    #[serde(default = "ConfigHelper::default_include_patterns")]
    pub include_patterns: Vec<String>,

    #[serde(default = "ConfigHelper::default_max_file_size")]
    pub max_file_size: String,

    #[serde(default = "ConfigHelper::default_max_files")]
    pub max_files: usize,

    #[serde(default = "ConfigHelper::default_languages")]
    pub languages: Vec<String>,

    #[serde(default)]
    pub skip_tests: bool,

    #[serde(default)]
    pub focus_areas: Vec<String>,

    #[serde(default = "ConfigHelper::default_chunk_strategy")]
    pub chunk_strategy: String,

    #[serde(default)]
    pub file_extensions: Vec<String>,
}

impl Default for AnalysisConfig {
    fn default() -> Self {
        Self {
            exclude_patterns: ConfigHelper::default_exclude_patterns(),
            include_patterns: ConfigHelper::default_include_patterns(),
            max_file_size: ConfigHelper::default_max_file_size(),
            max_files: ConfigHelper::default_max_files(),
            languages: ConfigHelper::default_languages(),
            skip_tests: ConfigHelper::default_skip_tests(),
            focus_areas: Vec::new(),
            chunk_strategy: ConfigHelper::default_chunk_strategy(),
            file_extensions: vec![],
        }
    }
}