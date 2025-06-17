use serde::Deserialize;
use crate::structs::ai::anthropic::anthropic_start_usage_info::AnthropicStartUsageInfo;

#[derive(Debug, Deserialize, Clone)]
pub struct AnthropicMessageStartInfo {
    pub usage: AnthropicStartUsageInfo,
}