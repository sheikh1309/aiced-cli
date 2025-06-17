use serde::{Deserialize, Serialize};
use crate::structs::ai::deepseek::deepseek_usage::DeepSeekUsage;
use crate::structs::ai::deepseek::deepseek_choice::DeepSeekChoice;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeepSeekResponse {
    pub id: String,
    pub object: String,
    pub created: i64,
    pub model: String,
    pub choices: Vec<DeepSeekChoice>,
    pub usage: DeepSeekUsage,
}