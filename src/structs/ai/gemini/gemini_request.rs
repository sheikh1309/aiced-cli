use serde::{Deserialize, Serialize};
use crate::structs::ai::gemini::gemini_content::GeminiContent;
use crate::structs::ai::gemini::gemini_generation_config::GeminiGenerationConfig;
use crate::structs::ai::gemini::gemini_safety_setting::GeminiSafetySetting;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct GeminiRequest {
    pub contents: Vec<GeminiContent>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub generation_config: Option<GeminiGenerationConfig>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub safety_settings: Option<Vec<GeminiSafetySetting>>,
}