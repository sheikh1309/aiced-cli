use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct RepositoryConfig {
    pub name: String,

    pub path: String,

    #[serde(default)]
    pub branch: Option<String>,

    #[serde(default)]
    pub auto_pull: bool,

    #[serde(default)]
    pub auto_pr: bool,
}