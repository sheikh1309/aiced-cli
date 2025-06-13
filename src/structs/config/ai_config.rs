use serde::{Deserialize, Serialize};
use crate::helpers::config_helper::ConfigHelper;

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct AiConfig {
    #[serde(default = "ConfigHelper::default_model")]
    pub model: String,

    #[serde(default = "ConfigHelper::default_max_tokens")]
    pub max_tokens: u32,

    #[serde(default = "ConfigHelper::default_temperature")]
    pub temperature: f32,

    #[serde(default)]
    pub api_key_env: Option<String>,

    #[serde(default = "ConfigHelper::default_provider")]
    pub provider: String,

    #[serde(default)]
    pub custom_prompt: Option<String>,
}

impl Default for AiConfig {
    fn default() -> Self {
        Self {
            model: ConfigHelper::default_model(),
            max_tokens: ConfigHelper::default_max_tokens(),
            temperature: ConfigHelper::default_temperature(),
            api_key_env: Some("ANTHROPIC_API_KEY".to_string()),
            provider: ConfigHelper::default_provider(),
            custom_prompt: None,
        }
    }
}