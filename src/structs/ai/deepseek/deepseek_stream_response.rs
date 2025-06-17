use serde::{Deserialize, Serialize};
use crate::structs::ai::deepseek::deepseek_stream_choice::DeepSeekStreamChoice;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeepSeekStreamResponse {
    pub id: String,
    pub object: String,
    pub created: i64,
    pub model: String,
    pub choices: Vec<DeepSeekStreamChoice>,
}