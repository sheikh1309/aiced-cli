use serde::{Deserialize, Serialize};
use crate::helpers::config_helper::ConfigHelper;

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct GlobalConfig {
    #[serde(default = "ConfigHelper::default_scan_interval")]
    pub scan_interval: String,
}


impl Default for GlobalConfig {
    fn default() -> Self {
        Self {
            scan_interval: ConfigHelper::default_scan_interval()
        }
    }
}