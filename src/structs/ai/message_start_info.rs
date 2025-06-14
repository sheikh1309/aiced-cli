use serde::Deserialize;
use crate::structs::ai::start_usage_info::StartUsageInfo;

#[derive(Debug, Deserialize, Clone)]
pub struct MessageStartInfo {
    pub usage: StartUsageInfo,
}