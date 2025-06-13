use serde::{Deserialize, Serialize};
use crate::helpers::config_helper::ConfigHelper;

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct OutputConfig {
    #[serde(default = "ConfigHelper::default_format")]
    pub format: String,

    #[serde(default = "ConfigHelper::default_save_analysis")]
    pub save_analysis: bool,

    #[serde(default)]
    pub output_dir: Option<String>,

    #[serde(default = "ConfigHelper::default_verbose")]
    pub verbose: bool,

    #[serde(default)]
    pub generate_report: bool,

    #[serde(default)]
    pub report_format: String,
}

impl Default for OutputConfig {
    fn default() -> Self {
        Self {
            format: ConfigHelper::default_format(),
            save_analysis: ConfigHelper::default_save_analysis(),
            output_dir: None,
            verbose: ConfigHelper::default_verbose(),
            generate_report: false,
            report_format: "markdown".to_string(),
        }
    }
}