use serde::Serialize;
use crate::structs::ai::anthropic_message::AnthropicMessage;

#[derive(Serialize)]
#[derive(Debug)]
pub struct TokenCountRequest {
    pub model: String,
    pub system: String,
    pub messages: Vec<AnthropicMessage>
}