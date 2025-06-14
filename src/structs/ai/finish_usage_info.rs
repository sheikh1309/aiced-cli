use serde::Deserialize;

#[derive(Debug, Deserialize, Clone)]
pub struct FinishUsageInfo {
    pub output_tokens: u32,
}