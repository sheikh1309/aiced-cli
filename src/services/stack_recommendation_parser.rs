use crate::structs::stack_recommendation::StackRecommendation;
use std::collections::HashMap;
use crate::errors::{AilyzerError, AilyzerResult};

const RECOMMENDED_STACK_MARKER: &str = "RECOMMENDED_STACK:";
const END_RECOMMENDED_STACK_MARKER: &str = "END_RECOMMENDED_STACK";
const PRIMARY_LANGUAGE_FIELD: &str = "PRIMARY_LANGUAGE:";
const LANGUAGE_REASON_FIELD: &str = "LANGUAGE_REASON:";
const FRAMEWORK_FIELD: &str = "FRAMEWORK:";
const FRAMEWORK_REASON_FIELD: &str = "FRAMEWORK_REASON:";
const RUNTIME_FIELD: &str = "RUNTIME:";
const PACKAGE_MANAGER_FIELD: &str = "PACKAGE_MANAGER:";
const DATABASE_FIELD: &str = "DATABASE:";
const DATABASE_REASON_FIELD: &str = "DATABASE_REASON:";
const ORM_FIELD: &str = "ORM:";
const TESTING_FIELD: &str = "TESTING:";
const BUILD_TOOLS_FIELD: &str = "BUILD_TOOLS:";
const LINTING_FIELD: &str = "LINTING:";
const CONTAINERIZATION_FIELD: &str = "CONTAINERIZATION:";
const CLOUD_SERVICES_FIELD: &str = "CLOUD_SERVICES:";
const AUTHENTICATION_FIELD: &str = "AUTHENTICATION:";
const API_TYPE_FIELD: &str = "API_TYPE:";
const API_REASON_FIELD: &str = "API_REASON:";
const ARCHITECTURE_PATTERN_FIELD: &str = "ARCHITECTURE_PATTERN:";
const ARCHITECTURE_REASON_FIELD: &str = "ARCHITECTURE_REASON:";
const SCALABILITY_CONSIDERATIONS_FIELD: &str = "SCALABILITY_CONSIDERATIONS:";
const SECURITY_RECOMMENDATIONS_FIELD: &str = "SECURITY_RECOMMENDATIONS:";
const DEPLOYMENT_STRATEGY_FIELD: &str = "DEPLOYMENT_STRATEGY:";
const LEARNING_CURVE_FIELD: &str = "LEARNING_CURVE:";
const MAINTENANCE_EFFORT_FIELD: &str = "MAINTENANCE_EFFORT:";
const RECOMMENDED_DEPENDENCIES_MARKER: &str = "RECOMMENDED_DEPENDENCIES:";
const END_RECOMMENDED_DEPENDENCIES_MARKER: &str = "END_RECOMMENDED_DEPENDENCIES";
const ESSENTIAL_CONFIGS_MARKER: &str = "ESSENTIAL_CONFIGS:";
const END_ESSENTIAL_CONFIGS_MARKER: &str = "END_ESSENTIAL_CONFIGS";
const PROJECT_STRUCTURE_MARKER: &str = "PROJECT_STRUCTURE:";
const END_PROJECT_STRUCTURE_MARKER: &str = "END_PROJECT_STRUCTURE";
const DEVELOPMENT_WORKFLOW_MARKER: &str = "DEVELOPMENT_WORKFLOW:";
const END_DEVELOPMENT_WORKFLOW_MARKER: &str = "END_DEVELOPMENT_WORKFLOW";

pub struct StackRecommendationParser {
    lines: Vec<String>,
    current: usize,
}

impl StackRecommendationParser {
    pub fn new(input: &str) -> Self {
        Self {
            lines: input.lines().map(|s| s.to_string()).collect(),
            current: 0,
        }
    }

    pub fn parse(&mut self) -> AilyzerResult<StackRecommendation> {
        while !self.is_eof() && !self.current_line().trim().starts_with(RECOMMENDED_STACK_MARKER) {
            self.advance();
        }

        if self.is_eof() {
            return Err(AilyzerError::parse_error("Recommended stack", Some(self.current), "Recommended stack", Some(&"Recommended stack marker not found")));
        }

        self.expect_line(RECOMMENDED_STACK_MARKER)?;
        self.advance();

        let mut recommendation = StackRecommendation::default();

        while !self.is_eof() && !self.current_line().trim().starts_with(END_RECOMMENDED_STACK_MARKER) {
            let line = self.current_line().trim();

            if line.is_empty() {
                self.advance();
                continue;
            }

            // Parse individual fields
            if line.starts_with(PRIMARY_LANGUAGE_FIELD) {
                recommendation.primary_language = Some(self.parse_field(PRIMARY_LANGUAGE_FIELD)?);
            } else if line.starts_with(LANGUAGE_REASON_FIELD) {
                recommendation.language_reason = Some(self.parse_field(LANGUAGE_REASON_FIELD)?);
            } else if line.starts_with(FRAMEWORK_FIELD) {
                recommendation.framework = Some(self.parse_field(FRAMEWORK_FIELD)?);
            } else if line.starts_with(FRAMEWORK_REASON_FIELD) {
                recommendation.framework_reason = Some(self.parse_field(FRAMEWORK_REASON_FIELD)?);
            } else if line.starts_with(RUNTIME_FIELD) {
                recommendation.runtime = Some(self.parse_field(RUNTIME_FIELD)?);
            } else if line.starts_with(PACKAGE_MANAGER_FIELD) {
                recommendation.package_manager = Some(self.parse_field(PACKAGE_MANAGER_FIELD)?);
            } else if line.starts_with(DATABASE_FIELD) {
                recommendation.database = Some(self.parse_field(DATABASE_FIELD)?);
            } else if line.starts_with(DATABASE_REASON_FIELD) {
                recommendation.database_reason = Some(self.parse_field(DATABASE_REASON_FIELD)?);
            } else if line.starts_with(ORM_FIELD) {
                recommendation.orm = Some(self.parse_field(ORM_FIELD)?);
            } else if line.starts_with(TESTING_FIELD) {
                recommendation.testing = Some(self.parse_field(TESTING_FIELD)?);
            } else if line.starts_with(BUILD_TOOLS_FIELD) {
                recommendation.build_tools = Some(self.parse_field(BUILD_TOOLS_FIELD)?);
            } else if line.starts_with(LINTING_FIELD) {
                recommendation.linting = Some(self.parse_field(LINTING_FIELD)?);
            } else if line.starts_with(CONTAINERIZATION_FIELD) {
                recommendation.containerization = Some(self.parse_field(CONTAINERIZATION_FIELD)?);
            } else if line.starts_with(CLOUD_SERVICES_FIELD) {
                recommendation.cloud_services = Some(self.parse_field(CLOUD_SERVICES_FIELD)?);
            } else if line.starts_with(AUTHENTICATION_FIELD) {
                recommendation.authentication = Some(self.parse_field(AUTHENTICATION_FIELD)?);
            } else if line.starts_with(API_TYPE_FIELD) {
                recommendation.api_type = Some(self.parse_field(API_TYPE_FIELD)?);
            } else if line.starts_with(API_REASON_FIELD) {
                recommendation.api_reason = Some(self.parse_field(API_REASON_FIELD)?);
            } else if line.starts_with(ARCHITECTURE_PATTERN_FIELD) {
                recommendation.architecture_pattern = Some(self.parse_field(ARCHITECTURE_PATTERN_FIELD)?);
            } else if line.starts_with(ARCHITECTURE_REASON_FIELD) {
                recommendation.architecture_reason = Some(self.parse_field(ARCHITECTURE_REASON_FIELD)?);
            } else if line.starts_with(SCALABILITY_CONSIDERATIONS_FIELD) {
                recommendation.scalability_considerations = Some(self.parse_field(SCALABILITY_CONSIDERATIONS_FIELD)?);
            } else if line.starts_with(SECURITY_RECOMMENDATIONS_FIELD) {
                recommendation.security_recommendations = Some(self.parse_field(SECURITY_RECOMMENDATIONS_FIELD)?);
            } else if line.starts_with(DEPLOYMENT_STRATEGY_FIELD) {
                recommendation.deployment_strategy = Some(self.parse_field(DEPLOYMENT_STRATEGY_FIELD)?);
            } else if line.starts_with(LEARNING_CURVE_FIELD) {
                recommendation.learning_curve = Some(self.parse_field(LEARNING_CURVE_FIELD)?);
            } else if line.starts_with(MAINTENANCE_EFFORT_FIELD) {
                recommendation.maintenance_effort = Some(self.parse_field(MAINTENANCE_EFFORT_FIELD)?);
            } else if line.starts_with(RECOMMENDED_DEPENDENCIES_MARKER) {
                self.advance();
                recommendation.recommended_dependencies = self.parse_dependency_section(END_RECOMMENDED_DEPENDENCIES_MARKER)?;
            } else if line.starts_with(ESSENTIAL_CONFIGS_MARKER) {
                self.advance();
                recommendation.essential_configs = self.parse_key_value_section(END_ESSENTIAL_CONFIGS_MARKER)?;
            } else if line.starts_with(PROJECT_STRUCTURE_MARKER) {
                self.advance();
                recommendation.project_structure = self.parse_key_value_section(END_PROJECT_STRUCTURE_MARKER)?;
            } else if line.starts_with(DEVELOPMENT_WORKFLOW_MARKER) {
                self.advance();
                recommendation.development_workflow = self.parse_key_value_section(END_DEVELOPMENT_WORKFLOW_MARKER)?;
            } else {
                self.advance();
            }
        }

        if !self.is_eof() {
            self.expect_line(END_RECOMMENDED_STACK_MARKER)?;
        }

        Ok(recommendation)
    }

    fn parse_dependency_section(&mut self, end_marker: &str) -> AilyzerResult<HashMap<String, String>> {
        let mut dependencies = HashMap::new();

        while !self.is_eof() && !self.current_line().trim().starts_with(end_marker) {
            let line = self.current_line().trim();

            if line.is_empty() {
                self.advance();
                continue;
            }

            // Parse format: package_name: version - purpose
            if let Some(colon_pos) = line.find(':') {
                let package_name = line[..colon_pos].trim().to_string();
                let rest = line[colon_pos + 1..].trim();

                // Split by " - " to separate version from purpose
                if let Some(dash_pos) = rest.find(" - ") {
                    let version = rest[..dash_pos].trim().to_string();
                    let purpose = rest[dash_pos + 3..].trim().to_string();
                    dependencies.insert(package_name, format!("{} - {}", version, purpose));
                } else {
                    // Just version without purpose
                    dependencies.insert(package_name, rest.to_string());
                }
            }

            self.advance();
        }

        if !self.is_eof() {
            self.expect_line(end_marker)?;
            self.advance();
        }

        Ok(dependencies)
    }

    fn parse_key_value_section(&mut self, end_marker: &str) -> AilyzerResult<HashMap<String, String>> {
        let mut map = HashMap::new();

        while !self.is_eof() && !self.current_line().trim().starts_with(end_marker) {
            let line = self.current_line().trim();

            if line.is_empty() {
                self.advance();
                continue;
            }

            // Parse key: value format
            if let Some(colon_pos) = line.find(':') {
                let key = line[..colon_pos].trim().to_string();
                let value = line[colon_pos + 1..].trim().to_string();
                map.insert(key, value);
            }

            self.advance();
        }

        if !self.is_eof() {
            self.expect_line(end_marker)?;
            self.advance();
        }

        Ok(map)
    }

    fn current_line(&self) -> &str {
        self.lines.get(self.current).map(|s| s.as_str()).unwrap_or("")
    }

    fn advance(&mut self) {
        self.current += 1;
    }

    fn is_eof(&self) -> bool {
        self.current >= self.lines.len()
    }

    fn parse_field(&mut self, prefix: &str) -> AilyzerResult<String> {
        let line = self.current_line();
        let value = line
            .strip_prefix(prefix)
            .ok_or_else(|| AilyzerError::parse_error("InvalidFormat", Some(self.current), "InvalidFormat", Some(&line)))?
            .trim()
            .to_string();

        self.advance();
        Ok(value)
    }

    fn expect_line(&self, expected: &str) -> AilyzerResult<()> {
        let line = self.current_line().trim();
        if !line.starts_with(expected) {
            return Err(AilyzerError::parse_error("InvalidFormat", Some(self.current), "InvalidFormat", Some(&line)));
        }
        Ok(())
    }
}

