use serde::{Deserialize, Serialize};
use crate::enums::priority::Priority;

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct RepositoryConfig {
    pub name: String,
    pub path: String,

    #[serde(default)]
    pub profile: Option<String>,

    #[serde(default)]
    pub enabled: bool,

    #[serde(default)]
    pub branch: Option<String>,

    #[serde(default)]
    pub remote_url: Option<String>,

    #[serde(default)]
    pub auto_pull: bool,

    #[serde(default)]
    pub schedule: Option<String>,

    #[serde(default)]
    pub tags: Vec<String>,

    #[serde(default)]
    pub priority: Priority,
}