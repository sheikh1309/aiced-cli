use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
#[derive(Debug)]
pub struct AnthropicTokenCountResponse {
    pub input_tokens: usize,
}