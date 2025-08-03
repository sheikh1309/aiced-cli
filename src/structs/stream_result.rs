use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct StreamResult {
    pub content: String,
    pub input_tokens: u32,
    pub output_tokens: u32,
}