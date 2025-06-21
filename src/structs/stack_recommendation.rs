use std::collections::HashMap;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct StackRecommendation {
    // Core technology choices
    pub primary_language: Option<String>,
    pub language_reason: Option<String>,
    pub framework: Option<String>,
    pub framework_reason: Option<String>,
    pub runtime: Option<String>,
    pub package_manager: Option<String>,
    pub database: Option<String>,
    pub database_reason: Option<String>,
    pub orm: Option<String>,
    pub testing: Option<String>,
    pub build_tools: Option<String>,
    pub linting: Option<String>,
    pub containerization: Option<String>,
    pub cloud_services: Option<String>,
    pub authentication: Option<String>,
    pub api_type: Option<String>,
    pub api_reason: Option<String>,
    pub architecture_pattern: Option<String>,
    pub architecture_reason: Option<String>,

    // Dependencies with versions and purposes
    pub recommended_dependencies: HashMap<String, String>,

    // Configuration files and their purposes
    pub essential_configs: HashMap<String, String>,

    // Project structure recommendations
    pub project_structure: HashMap<String, String>,

    // Development workflow steps
    pub development_workflow: HashMap<String, String>,

    // Additional considerations
    pub scalability_considerations: Option<String>,
    pub security_recommendations: Option<String>,
    pub deployment_strategy: Option<String>,
    pub learning_curve: Option<String>,
    pub maintenance_effort: Option<String>,
}

impl StackRecommendation {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn is_empty(&self) -> bool {
        self.primary_language.is_none() &&
            self.framework.is_none() &&
            self.runtime.is_none() &&
            self.database.is_none() &&
            self.recommended_dependencies.is_empty()
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
        self.recommended_dependencies.len()
    }

    pub fn get_config_count(&self) -> usize {
        self.essential_configs.len()
    }

    pub fn get_structure_count(&self) -> usize {
        self.project_structure.len()
    }

    pub fn get_workflow_steps(&self) -> usize {
        self.development_workflow.len()
    }

    pub fn has_reasoning(&self) -> bool {
        self.language_reason.is_some() ||
            self.framework_reason.is_some() ||
            self.database_reason.is_some() ||
            self.api_reason.is_some() ||
            self.architecture_reason.is_some()
    }

    pub fn get_core_technologies(&self) -> Vec<String> {
        let mut technologies = Vec::new();

        if let Some(lang) = &self.primary_language {
            technologies.push(format!("Language: {}", lang));
        }

        if let Some(framework) = &self.framework {
            technologies.push(format!("Framework: {}", framework));
        }

        if let Some(runtime) = &self.runtime {
            technologies.push(format!("Runtime: {}", runtime));
        }

        if let Some(db) = &self.database {
            technologies.push(format!("Database: {}", db));
        }

        if let Some(api) = &self.api_type {
            technologies.push(format!("API: {}", api));
        }

        technologies
    }

    pub fn get_development_tools(&self) -> Vec<String> {
        let mut tools = Vec::new();

        if let Some(pm) = &self.package_manager {
            tools.push(format!("Package Manager: {}", pm));
        }

        if let Some(build) = &self.build_tools {
            tools.push(format!("Build Tools: {}", build));
        }

        if let Some(testing) = &self.testing {
            tools.push(format!("Testing: {}", testing));
        }

        if let Some(linting) = &self.linting {
            tools.push(format!("Linting: {}", linting));
        }

        tools
    }

    pub fn get_infrastructure_tools(&self) -> Vec<String> {
        let mut tools = Vec::new();

        if let Some(container) = &self.containerization {
            tools.push(format!("Containerization: {}", container));
        }

        if let Some(cloud) = &self.cloud_services {
            tools.push(format!("Cloud Services: {}", cloud));
        }

        if let Some(auth) = &self.authentication {
            tools.push(format!("Authentication: {}", auth));
        }

        tools
    }

    pub fn get_all_reasons(&self) -> HashMap<String, String> {
        let mut reasons = HashMap::new();

        if let Some(reason) = &self.language_reason {
            reasons.insert("Language".to_string(), reason.clone());
        }

        if let Some(reason) = &self.framework_reason {
            reasons.insert("Framework".to_string(), reason.clone());
        }

        if let Some(reason) = &self.database_reason {
            reasons.insert("Database".to_string(), reason.clone());
        }

        if let Some(reason) = &self.api_reason {
            reasons.insert("API Type".to_string(), reason.clone());
        }

        if let Some(reason) = &self.architecture_reason {
            reasons.insert("Architecture".to_string(), reason.clone());
        }

        reasons
    }

    pub fn get_considerations(&self) -> HashMap<String, String> {
        let mut considerations = HashMap::new();

        if let Some(scalability) = &self.scalability_considerations {
            considerations.insert("Scalability".to_string(), scalability.clone());
        }

        if let Some(security) = &self.security_recommendations {
            considerations.insert("Security".to_string(), security.clone());
        }

        if let Some(deployment) = &self.deployment_strategy {
            considerations.insert("Deployment".to_string(), deployment.clone());
        }

        if let Some(learning) = &self.learning_curve {
            considerations.insert("Learning Curve".to_string(), learning.clone());
        }

        if let Some(maintenance) = &self.maintenance_effort {
            considerations.insert("Maintenance".to_string(), maintenance.clone());
        }

        considerations
    }

    pub fn is_complete(&self) -> bool {
        self.primary_language.is_some() &&
            self.framework.is_some() &&
            self.database.is_some() &&
            !self.recommended_dependencies.is_empty() &&
            !self.essential_configs.is_empty()
    }

    pub fn get_completeness_score(&self) -> f32 {
        let mut score = 0.0;
        let total_fields = 10.0; // Adjust based on critical fields

        if self.primary_language.is_some() { score += 1.0; }
        if self.framework.is_some() { score += 1.0; }
        if self.database.is_some() { score += 1.0; }
        if self.api_type.is_some() { score += 1.0; }
        if self.architecture_pattern.is_some() { score += 1.0; }
        if !self.recommended_dependencies.is_empty() { score += 1.0; }
        if !self.essential_configs.is_empty() { score += 1.0; }
        if !self.project_structure.is_empty() { score += 1.0; }
        if self.deployment_strategy.is_some() { score += 1.0; }
        if self.security_recommendations.is_some() { score += 1.0; }

        score / total_fields
    }
}