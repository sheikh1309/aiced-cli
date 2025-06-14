use serde::Deserialize;
use crate::structs::ai::api_error::ApiError;
use crate::structs::ai::content_delta::ContentDelta;
use crate::structs::ai::message_delta::MessageDelta;
use crate::structs::ai::message_start_info::MessageStartInfo;
use crate::structs::ai::finish_usage_info::FinishUsageInfo;

#[derive(Deserialize, Debug, Clone)]
#[serde(tag = "type")]
pub enum StreamEventData {
    #[serde(rename = "message_start")]
    MessageStart {
        message: MessageStartInfo,
    },
    #[serde(rename = "content_block_delta")]
    ContentBlockDelta {
        index: usize,
        delta: ContentDelta,
    },
    #[serde(rename = "message_delta")]
    MessageDelta {
        delta: MessageDelta,
        usage: FinishUsageInfo,
    },
    #[serde(rename = "error")]
    Error {
        error: ApiError,
    },
}