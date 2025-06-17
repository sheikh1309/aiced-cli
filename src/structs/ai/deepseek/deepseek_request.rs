use serde::{Deserialize, Serialize};
use crate::structs::ai::deepseek::deepseek_message::DeepSeekMessage;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeepSeekRequest {
    pub model: String,
    
    pub messages: Vec<DeepSeekMessage>,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_tokens: Option<i32>,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f32>,
    
    pub stream: bool,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub top_p: Option<f32>,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub frequency_penalty: Option<f32>,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub presence_penalty: Option<f32>,
}