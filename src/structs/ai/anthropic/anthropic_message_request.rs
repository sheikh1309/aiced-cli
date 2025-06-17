use serde::Serialize;
use crate::structs::ai::anthropic::anthropic_message::AnthropicMessage;
use crate::structs::ai::anthropic::anthropic_thinking::AnthropicThinking;

#[derive(Serialize)]
pub struct AnthropicMessageRequest {
    pub model: String,
    pub system: String,
    pub max_tokens: u32,
    pub temperature: Option<f32>,
    pub messages: Vec<AnthropicMessage>,
    pub stream: bool,
    pub thinking: AnthropicThinking,
}