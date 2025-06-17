use serde::Deserialize;
use crate::structs::ai::api_error::ApiError;
use crate::structs::ai::anthropic::anthropic_content_delta::AnthropicContentDelta;
use crate::structs::ai::anthropic::anthropic_message_delta::AnthropicMessageDelta;
use crate::structs::ai::anthropic::anthropic_message_start_info::AnthropicMessageStartInfo;
use crate::structs::ai::anthropic::anthropic_finish_usage_info::AnthropicFinishUsageInfo;

#[derive(Deserialize, Debug, Clone)]
#[serde(tag = "type")]
pub enum StreamEventData {
    #[serde(rename = "message_start")]
    MessageStart {
        message: AnthropicMessageStartInfo,
    },
    #[serde(rename = "content_block_delta")]
    ContentBlockDelta {
        index: usize,
        delta: AnthropicContentDelta,
    },
    #[serde(rename = "message_delta")]
    MessageDelta {
        delta: AnthropicMessageDelta,
        usage: AnthropicFinishUsageInfo,
    },
    #[serde(rename = "error")]
    Error {
        error: ApiError,
    },
}