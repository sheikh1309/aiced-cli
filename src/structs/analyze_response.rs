use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug)]
pub struct AnalyzeResponse {
    pub content: String,
    pub input_tokens: u32,
    pub output_tokens: u32,
}