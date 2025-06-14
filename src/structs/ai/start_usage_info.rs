use serde::Deserialize;

#[derive(Debug, Deserialize, Clone)]
pub struct StartUsageInfo {
    pub input_tokens: u32,
    pub output_tokens: u32,
}