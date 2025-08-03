use serde::Serialize;
use crate::structs::ai::anthropic::anthropic_message::AnthropicMessage;

#[derive(Serialize)]
#[derive(Debug)]
pub struct AnthropicTokenCountRequest {
    pub model: String,
    pub system: String,
    pub messages: Vec<AnthropicMessage>
}