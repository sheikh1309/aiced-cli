use std::collections::HashMap;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TechnologyStack {
    pub primary_language: Option<String>,
    pub framework: Option<String>,
    pub runtime: Option<String>,
    pub package_manager: Option<String>,
    pub database: Option<String>,
    pub orm: Option<String>,
    pub testing: Option<String>,
    pub build_tools: Option<String>,
    pub linting: Option<String>,
    pub containerization: Option<String>,
    pub cloud_services: Option<String>,
    pub authentication: Option<String>,
    pub api_type: Option<String>,
    pub architecture_pattern: Option<String>,
    pub dependencies: HashMap<String, String>,
    pub critical_configs: HashMap<String, String>,
}


