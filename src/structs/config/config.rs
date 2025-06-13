use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use crate::structs::config::ai_config::AiConfig;
use crate::structs::config::global_config::GlobalConfig;
use crate::structs::config::notification_config::NotificationConfig;
use crate::structs::config::output_config::OutputConfig;
use crate::structs::config::profile_config::ProfileConfig;
use crate::structs::config::repository_config::RepositoryConfig;

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Config {
    #[serde(default)]
    pub global: GlobalConfig,

    #[serde(default)]
    pub repositories: Vec<RepositoryConfig>,

    #[serde(default)]
    pub profiles: HashMap<String, ProfileConfig>,

    #[serde(default)]
    pub ai: AiConfig,

    #[serde(default)]
    pub output: OutputConfig,

    #[serde(default)]
    pub notifications: NotificationConfig,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            global: Default::default(),
            repositories: vec![],
            ai: AiConfig::default(),
            output: OutputConfig::default(),
            profiles: Default::default(),
            notifications: Default::default(),
        }
    }
}