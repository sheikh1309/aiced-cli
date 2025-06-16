use serde::{Deserialize, Serialize};
use crate::structs::config::global_config::GlobalConfig;
use crate::structs::config::notification_config::NotificationConfig;
use crate::structs::config::output_config::OutputConfig;
use crate::structs::config::repository_config::RepositoryConfig;

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Config {
    #[serde(default)]
    pub global: GlobalConfig,

    #[serde(default)]
    pub repositories: Vec<RepositoryConfig>,

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
            output: OutputConfig::default(),
            notifications: Default::default(),
        }
    }
}