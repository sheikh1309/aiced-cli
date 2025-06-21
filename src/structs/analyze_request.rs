use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
pub struct AnalyzeRequest {
    pub prompt: String,
}