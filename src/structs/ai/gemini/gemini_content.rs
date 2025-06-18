use serde::{Deserialize, Serialize};
use crate::structs::ai::gemini::gemini_part::GeminiPart;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct GeminiContent {
    pub role: String,
    pub parts: Vec<GeminiPart>,
}