use serde::Deserialize;

#[derive(Debug, Deserialize, Clone)]
pub struct AnthropicFinishUsageInfo {
    pub output_tokens: u32,
}