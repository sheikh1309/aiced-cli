use serde::Deserialize;
use crate::structs::ai::api_error::ApiError;
use crate::structs::ai::anthropic::anthropic_message_delta::AnthropicMessageDelta;
use crate::structs::ai::anthropic::anthropic_content_delta::AnthropicContentDelta;
use crate::structs::ai::anthropic::anthropic_finish_usage_info::AnthropicFinishUsageInfo;
use crate::structs::ai::anthropic::anthropic_message_start_info::AnthropicMessageStartInfo;

#[derive(Deserialize, Debug, Clone)]
#[serde(tag = "type")]
pub enum StreamEventData {
    #[serde(rename = "message_start")]
    MessageStart {
        message: AnthropicMessageStartInfo,
    },
    #[serde(rename = "content_block_start")]
    ContentBlockStart,
    #[serde(rename = "content_block_stop")]
    ContentBlockStop,
    #[serde(rename = "message_stop")]
    MessageStop,
    #[serde(rename = "ping")]
    Ping,
    #[serde(rename = "content_block_delta")]
    ContentBlockDelta {
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