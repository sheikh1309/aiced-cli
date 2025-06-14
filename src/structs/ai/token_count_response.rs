use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
#[derive(Debug)]
pub struct TokenCountResponse {
    pub input_tokens: usize,
}