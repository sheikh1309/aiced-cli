use serde::Serialize;
use crate::structs::ai::anthropic_message::AnthropicMessage;
use crate::structs::ai::thinking::Thinking;

#[derive(Serialize)]
pub struct MessageRequest {
    pub model: String,
    pub system: String,
    pub max_tokens: u32,
    pub temperature: Option<f32>,
    pub messages: Vec<AnthropicMessage>,
    pub stream: bool,
    pub thinking: Thinking,
}