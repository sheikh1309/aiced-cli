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

impl TechnologyStack {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn is_empty(&self) -> bool {
        self.primary_language.is_none() &&
            self.framework.is_none() &&
            self.runtime.is_none() &&
            self.package_manager.is_none() &&
            self.database.is_none() &&
            self.orm.is_none() &&
            self.testing.is_none() &&
            self.build_tools.is_none() &&
            self.linting.is_none() &&
            self.containerization.is_none() &&
            self.cloud_services.is_none() &&
            self.authentication.is_none() &&
            self.api_type.is_none() &&
            self.architecture_pattern.is_none() &&
            self.dependencies.is_empty() &&
            self.critical_configs.is_empty()
    }

    pub fn get_main_stack(&self) -> String {
        let mut stack_parts = Vec::new();

        if let Some(lang) = &self.primary_language {
            stack_parts.push(lang.clone());
        }

        if let Some(framework) = &self.framework {
            stack_parts.push(framework.clone());
        }

        if let Some(db) = &self.database {
            stack_parts.push(db.clone());
        }

        if stack_parts.is_empty() {
            "Unknown".to_string()
        } else {
            stack_parts.join(" + ")
        }
    }

    pub fn get_dependency_count(&self) -> usize {
        self.dependencies.len()
    }

    pub fn get_config_count(&self) -> usize {
        self.critical_configs.len()
    }
}

