use serde::{Deserialize, Serialize};
use crate::structs::ai::deepseek::deepseek_message::DeepSeekMessage;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeepSeekChoice {
    pub index: i32,
    pub message: DeepSeekMessage,
    pub finish_reason: Option<String>,
}