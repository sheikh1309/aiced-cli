use serde::{Deserialize, Serialize};
use crate::structs::ai::deepseek::deepseek_delta::DeepSeekDelta;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeepSeekStreamChoice {
    pub index: i32,
    pub delta: DeepSeekDelta,
    pub finish_reason: Option<String>,
}