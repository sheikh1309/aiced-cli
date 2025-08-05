use crate::enums::file_change::FileChange;
use crate::enums::line_change::LineChange;
use crate::structs::analysis_response::AnalysisResponse;
use crate::structs::technology_stack::TechnologyStack;
use std::collections::HashMap;
use crate::errors::{AicedError, AicedResult};

const ANALYSIS_SUMMARY_MARKER: &str = "ANALYSIS_SUMMARY:";
const CHANGE_MARKER: &str = "CHANGE:";
const END_CHANGE_MARKER: &str = "END_CHANGE";
const FILE_FIELD: &str = "FILE:";
const REASON_FIELD: &str = "REASON:";
const SEVERITY_FIELD: &str = "SEVERITY:";
const ACTION_FIELD: &str = "ACTION:";
const LINE_FIELD: &str = "LINE:";
const START_LINE_FIELD: &str = "START_LINE:";
const END_LINE_FIELD: &str = "END_LINE:";
const OLD_FIELD: &str = "OLD:";
const NEW_FIELD: &str = "NEW:";
const OLD_LINES_MARKER: &str = "OLD_LINES:";
const NEW_LINES_MARKER: &str = "NEW_LINES:";
const END_OLD_LINES_MARKER: &str = "END_OLD_LINES";
const END_NEW_LINES_MARKER: &str = "END_NEW_LINES";
const CONTENT_FIELD: &str = "CONTENT:";
const END_CONTENT_MARKER: &str = "END_CONTENT";
const TECHNOLOGY_STACK_MARKER: &str = "TECHNOLOGY_STACK:";
const END_TECHNOLOGY_STACK_MARKER: &str = "END_TECHNOLOGY_STACK";
const CATEGORY_FIELD: &str = "CATEGORY:";
const DEPENDENCIES_MARKER: &str = "DEPENDENCIES:";
const END_DEPENDENCIES_MARKER: &str = "END_DEPENDENCIES";
const CRITICAL_CONFIGS_MARKER: &str = "CRITICAL_CONFIGS:";
const END_CRITICAL_CONFIGS_MARKER: &str = "END_CRITICAL_CONFIGS";
const PRIMARY_LANGUAGE_FIELD: &str = "PRIMARY_LANGUAGE:";
const FRAMEWORK_FIELD: &str = "FRAMEWORK:";
const RUNTIME_FIELD: &str = "RUNTIME:";
const PACKAGE_MANAGER_FIELD: &str = "PACKAGE_MANAGER:";
const DATABASE_FIELD: &str = "DATABASE:";
const ORM_FIELD: &str = "ORM:";
const TESTING_FIELD: &str = "TESTING:";
const BUILD_TOOLS_FIELD: &str = "BUILD_TOOLS:";
const LINTING_FIELD: &str = "LINTING:";
const CONTAINERIZATION_FIELD: &str = "CONTAINERIZATION:";
const CLOUD_SERVICES_FIELD: &str = "CLOUD_SERVICES:";
const AUTHENTICATION_FIELD: &str = "AUTHENTICATION:";
const API_TYPE_FIELD: &str = "API_TYPE:";
const ARCHITECTURE_PATTERN_FIELD: &str = "ARCHITECTURE_PATTERN:";
const MODIFY_FILE_REQUIRED_FIELDS: &[&str] = &[FILE_FIELD, REASON_FIELD, SEVERITY_FIELD, CATEGORY_FIELD];
const CREATE_FILE_REQUIRED_FIELDS: &[&str] = &[FILE_FIELD, REASON_FIELD, SEVERITY_FIELD, CATEGORY_FIELD];
const DELETE_FILE_REQUIRED_FIELDS: &[&str] = &[FILE_FIELD, REASON_FIELD, SEVERITY_FIELD, CATEGORY_FIELD];

pub struct AnalysisParser {
    lines: Vec<String>,
    current: usize,
}

impl AnalysisParser {
    pub fn new(input: &str) -> Self {
        Self {
            lines: input.lines().map(|s| s.to_string()).collect(),
            current: 0,
        }
    }

    pub fn parse(&mut self) -> AicedResult<AnalysisResponse> {
        let mut response = AnalysisResponse {
            technology_stack: None,
            analysis_summary: String::new(),
            changes: Vec::new()
        };

        if self.has_technology_stack() {
            response.technology_stack = Some(self.parse_technology_stack()?);
        }

        response.analysis_summary = self.parse_summary()?;
        while self.current < self.lines.len() {
            if self.current_line().trim().is_empty() {
                self.advance();
                continue;
            }

            if self.current_line().starts_with(CHANGE_MARKER) {
                match self.parse_change() {
                    Ok(change) => {
                        response.changes.push(change);
                    }
                    Err(e) => {
                        log::error!("❌ Error parsing change at line {}: {}", self.current + 1, e);
                        self.skip_to_next_change();
                    }
                }
            } else {
                self.advance();
            }
        }

        Ok(response)
    }

    fn has_technology_stack(&self) -> bool {
        self.lines.iter().any(|line| line.trim().starts_with(TECHNOLOGY_STACK_MARKER))
    }

    fn parse_technology_stack(&mut self) -> AicedResult<TechnologyStack> {
        while !self.is_eof() && !self.current_line().trim().starts_with(TECHNOLOGY_STACK_MARKER) {
            self.advance();
        }

        if self.is_eof() {
            return Err(AicedError::parse_error("ParseError", Some(self.current), "ParseError", Some(&"Technology stack marker not found")));
        }

        self.expect_line(TECHNOLOGY_STACK_MARKER)?;
        self.advance();

        let mut stack = TechnologyStack::default();
        let mut dependencies = HashMap::new();
        let mut critical_configs = HashMap::new();

        while !self.is_eof() && !self.current_line().trim().starts_with(END_TECHNOLOGY_STACK_MARKER) {
            let line = self.current_line().trim();

            if line.is_empty() {
                self.advance();
                continue;
            }

            
            if line.starts_with(PRIMARY_LANGUAGE_FIELD) {
                stack.primary_language = Some(self.parse_field(PRIMARY_LANGUAGE_FIELD)?);
            } else if line.starts_with(FRAMEWORK_FIELD) {
                stack.framework = Some(self.parse_field(FRAMEWORK_FIELD)?);
            } else if line.starts_with(RUNTIME_FIELD) {
                stack.runtime = Some(self.parse_field(RUNTIME_FIELD)?);
            } else if line.starts_with(PACKAGE_MANAGER_FIELD) {
                stack.package_manager = Some(self.parse_field(PACKAGE_MANAGER_FIELD)?);
            } else if line.starts_with(DATABASE_FIELD) {
                stack.database = Some(self.parse_field(DATABASE_FIELD)?);
            } else if line.starts_with(ORM_FIELD) {
                stack.orm = Some(self.parse_field(ORM_FIELD)?);
            } else if line.starts_with(TESTING_FIELD) {
                stack.testing = Some(self.parse_field(TESTING_FIELD)?);
            } else if line.starts_with(BUILD_TOOLS_FIELD) {
                stack.build_tools = Some(self.parse_field(BUILD_TOOLS_FIELD)?);
            } else if line.starts_with(LINTING_FIELD) {
                stack.linting = Some(self.parse_field(LINTING_FIELD)?);
            } else if line.starts_with(CONTAINERIZATION_FIELD) {
                stack.containerization = Some(self.parse_field(CONTAINERIZATION_FIELD)?);
            } else if line.starts_with(CLOUD_SERVICES_FIELD) {
                stack.cloud_services = Some(self.parse_field(CLOUD_SERVICES_FIELD)?);
            } else if line.starts_with(AUTHENTICATION_FIELD) {
                stack.authentication = Some(self.parse_field(AUTHENTICATION_FIELD)?);
            } else if line.starts_with(API_TYPE_FIELD) {
                stack.api_type = Some(self.parse_field(API_TYPE_FIELD)?);
            } else if line.starts_with(ARCHITECTURE_PATTERN_FIELD) {
                stack.architecture_pattern = Some(self.parse_field(ARCHITECTURE_PATTERN_FIELD)?);
            } else if line.starts_with(DEPENDENCIES_MARKER) {
                self.advance();
                dependencies = self.parse_key_value_section(END_DEPENDENCIES_MARKER)?;
            } else if line.starts_with(CRITICAL_CONFIGS_MARKER) {
                self.advance();
                critical_configs = self.parse_key_value_section(END_CRITICAL_CONFIGS_MARKER)?;
            } else {
                self.advance();
            }
        }

        self.expect_line(END_TECHNOLOGY_STACK_MARKER)?;
        self.advance();

        stack.dependencies = dependencies;
        stack.critical_configs = critical_configs;

        Ok(stack)
    }

    fn parse_key_value_section(&mut self, end_marker: &str) -> AicedResult<HashMap<String, String>> {
        let mut map = HashMap::new();

        while !self.is_eof() && !self.current_line().trim().starts_with(end_marker) {
            let line = self.current_line().trim();

            if line.is_empty() {
                self.advance();
                continue;
            }

            
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

    fn skip_to_next_change(&mut self) {
        while self.current < self.lines.len() {
            let line = self.current_line().trim();
            if line.starts_with(CHANGE_MARKER) || line.starts_with(END_CHANGE_MARKER) {
                break;
            }
            self.advance();
        }

        if self.current < self.lines.len() && self.current_line().starts_with(END_CHANGE_MARKER) {
            self.advance();
        }
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

    fn parse_summary(&mut self) -> AicedResult<String> {
        
        while !self.is_eof() && !self.current_line().trim().starts_with(ANALYSIS_SUMMARY_MARKER) {
            self.advance();
        }

        if self.is_eof() {
            return Err(AicedError::parse_error("ParseError", Some(self.current), "ParseError", Some(&"Analysis summary marker not found")));
        }

        self.expect_line(ANALYSIS_SUMMARY_MARKER)?;
        self.advance();

        let mut summary_lines = Vec::new();

        
        while !self.is_eof() && !self.current_line().starts_with(CHANGE_MARKER) {
            let line = self.current_line().trim();
            if !line.is_empty() {
                summary_lines.push(line.to_string());
            }
            self.advance();
        }

        if summary_lines.is_empty() {
            return Err(AicedError::parse_error("ParseError", Some(self.current), "ParseError", Some(&"Analysis summary marker not found")));
        }

        Ok(summary_lines.join("\n"))
    }

    fn parse_change(&mut self) -> AicedResult<FileChange> {
        let line = self.current_line().trim().to_string();
        let change_type = line
            .strip_prefix(CHANGE_MARKER)
            .ok_or_else(|| AicedError::parse_error("InvalidFormat", Some(self.current), "InvalidFormat", Some(&line)))?
            .trim();

        let current_line = self.current + 1; 
        self.advance();

        match change_type {
            "modify_file" => self.parse_modify_file(),
            "create_file" => self.parse_create_file(),
            "delete_file" => self.parse_delete_file(),
            _ => Err(AicedError::parse_error("UnknownChangeType", Some(current_line), "UnknownChangeType", Some(&change_type))),
        }
    }

    fn parse_modify_file(&mut self) -> AicedResult<FileChange> {
        let fields = self.parse_required_fields(MODIFY_FILE_REQUIRED_FIELDS)?;
        let mut line_changes = Vec::new();

        
        while !self.is_eof() && !self.current_line().starts_with(END_CHANGE_MARKER) {
            let line = self.current_line().trim();

            if line.starts_with(ACTION_FIELD) {
                match self.parse_line_action() {
                    Ok(action) => line_changes.push(action),
                    Err(e) => {
                        log::error!("⚠️  Warning: Failed to parse action at line {}: {}", self.current + 1, e);
                        self.skip_to_next_action();
                    }
                }
            } else {
                self.advance();
            }
        }

        self.expect_line(END_CHANGE_MARKER)?;
        self.advance();

        Ok(FileChange::ModifyFile {
            file_path: fields.get(FILE_FIELD).unwrap().clone(),
            reason: fields.get(REASON_FIELD).unwrap().clone(),
            severity: fields.get(SEVERITY_FIELD).unwrap().clone(),
            category: fields.get(CATEGORY_FIELD).unwrap().clone(),
            line_changes,
        })
    }

    fn parse_create_file(&mut self) -> AicedResult<FileChange> {
        let fields = self.parse_required_fields(CREATE_FILE_REQUIRED_FIELDS)?;
        let mut content = String::new();

        while !self.is_eof() && !self.current_line().starts_with(END_CHANGE_MARKER) {
            let line = self.current_line().trim();

            if line.starts_with(CONTENT_FIELD) {
                self.advance();

                content = self.parse_content_until_fixed(END_CONTENT_MARKER)?;
                break;
            } else {
                self.advance();
            }
        }

        self.expect_line(END_CHANGE_MARKER)?;
        self.advance();

        Ok(FileChange::CreateFile {
            file_path: fields.get(FILE_FIELD).unwrap().clone(),
            reason: fields.get(REASON_FIELD).unwrap().clone(),
            severity: fields.get(SEVERITY_FIELD).unwrap().clone(),
            category: fields.get(CATEGORY_FIELD).unwrap().clone(),
            content,
        })
    }

    fn parse_content_until_fixed(&mut self, end_marker: &str) -> AicedResult<String> {
        let mut content_lines = Vec::new();
        let start_line = self.current; 

        while !self.is_eof() {
            let line = self.current_line();
            let trimmed_line = line.trim();

            if trimmed_line == end_marker {
                self.advance(); 
                break;
            }

            
            content_lines.push(line.to_string());
            self.advance();

            
            if self.current - start_line > 10000 {
                return Err(AicedError::parse_error(
                    "ContentTooLarge",
                    Some(self.current),
                    "ContentTooLarge",
                    Some(&format!("Content block too large (>10000 lines), possible missing {}", end_marker))
                ));
            }
        }

        if self.is_eof() {
            return Err(AicedError::parse_error(
                "UnexpectedEof",
                Some(self.current),
                "UnexpectedEof",
                Some(&format!("looking for {} (started at line {})", end_marker, start_line + 1))
            ));
        }

        let content = content_lines.join("\n");
        Ok(content)
    }

    fn parse_delete_file(&mut self) -> AicedResult<FileChange> {
        let fields = self.parse_required_fields(DELETE_FILE_REQUIRED_FIELDS)?;

        
        while !self.is_eof() && !self.current_line().starts_with(END_CHANGE_MARKER) {
            self.advance();
        }

        self.expect_line(END_CHANGE_MARKER)?;
        self.advance();

        Ok(FileChange::DeleteFile {
            file_path: fields.get(FILE_FIELD).unwrap().clone(),
            reason: fields.get(REASON_FIELD).unwrap().clone(),
            severity: fields.get(SEVERITY_FIELD).unwrap().clone(),
            category: fields.get(CATEGORY_FIELD).unwrap().clone(),
        })
    }

    fn parse_required_fields(&mut self, required_fields: &[&str]) -> AicedResult<HashMap<String, String>> {
        let mut fields = HashMap::new();
        while !self.is_eof() && !self.is_terminator() {
            let line = self.current_line().trim().to_string(); 
            let mut found_field = false;

            for &field in required_fields {
                if line.starts_with(field) {
                    let value = self.parse_field(field)?;
                    fields.insert(field.to_string(), value);
                    found_field = true;
                    break;
                }
            }

            if !found_field && !Self::is_recognized_field_static(&line) {
                self.advance();
            }
        }

        for &field in required_fields {
            if !fields.contains_key(field) {
                return Err(AicedError::parse_error("MissingField", Some(self.current), "MissingField", Some(&field)));
            }
        }

        Ok(fields)
    }

    fn is_terminator(&self) -> bool {
        let line = self.current_line().trim();
        line.starts_with(END_CHANGE_MARKER) ||
            line.starts_with(ACTION_FIELD) ||
            line.starts_with(CONTENT_FIELD)
    }

    fn is_recognized_field_static(line: &str) -> bool {
        const RECOGNIZED_FIELDS: &[&str] = &[
            FILE_FIELD, REASON_FIELD, SEVERITY_FIELD, CATEGORY_FIELD, ACTION_FIELD,
            LINE_FIELD, START_LINE_FIELD, END_LINE_FIELD,
            OLD_FIELD, NEW_FIELD, CONTENT_FIELD, NEW_LINES_MARKER
        ];

        RECOGNIZED_FIELDS.iter().any(|&field| line.starts_with(field))
    }

    fn skip_to_next_action(&mut self) {
        self.advance();
        while !self.is_eof() {
            let line = self.current_line().trim();
            if line.starts_with(ACTION_FIELD) || line.starts_with(END_CHANGE_MARKER) {
                break;
            }
            self.advance();
        }
    }

    fn parse_line_action(&mut self) -> AicedResult<LineChange> {
        let action_type = self.parse_field(ACTION_FIELD)?;
        let current_line = self.current; 

        match action_type.as_str() {
            "replace" => self.parse_replace_action(),
            "insert_after" => self.parse_insert_after_action(),
            "insert_before" => self.parse_insert_before_action(),
            "insert_many_after" => self.parse_insert_many_after_action(),
            "insert_many_before" => self.parse_insert_many_before_action(),
            "delete" => self.parse_delete_action(),
            "delete_many" => self.parse_delete_many_action(),
            "replace_range" => self.parse_replace_range_action(),
            _ => Err(AicedError::parse_error("UnknownActionType", Some(current_line), "UnknownActionType", Some(&action_type))),
        }
    }

    fn parse_replace_action(&mut self) -> AicedResult<LineChange> {
        let line_number = self.parse_number_field(LINE_FIELD)?;
        let old_content = self.parse_field(OLD_FIELD)?;
        let new_content = self.parse_field(NEW_FIELD)?;

        Ok(LineChange::Replace {
            line_number,
            old_content,
            new_content,
        })
    }

    fn parse_insert_after_action(&mut self) -> AicedResult<LineChange> {
        let line_number = self.parse_number_field(LINE_FIELD)?;
        let new_content = self.parse_field(NEW_FIELD)?;

        Ok(LineChange::InsertAfter {
            line_number,
            new_content,
        })
    }

    fn parse_insert_before_action(&mut self) -> AicedResult<LineChange> {
        let line_number = self.parse_number_field(LINE_FIELD)?;
        let new_content = self.parse_field(NEW_FIELD)?;

        Ok(LineChange::InsertBefore {
            line_number,
            new_content,
        })
    }

    fn parse_insert_many_after_action(&mut self) -> AicedResult<LineChange> {
        let line_number = self.parse_number_field(LINE_FIELD)?;

        
        self.expect_line(NEW_LINES_MARKER)?;
        self.advance();
        let new_lines = self.parse_lines_until(END_NEW_LINES_MARKER)?;

        Ok(LineChange::InsertManyAfter {
            line_number,
            new_lines,
        })
    }

    fn parse_insert_many_before_action(&mut self) -> AicedResult<LineChange> {
        let line_number = self.parse_number_field(LINE_FIELD)?;

        
        self.expect_line(NEW_LINES_MARKER)?;
        self.advance();
        let new_lines = self.parse_lines_until(END_NEW_LINES_MARKER)?;

        Ok(LineChange::InsertManyBefore {
            line_number,
            new_lines,
        })
    }

    fn parse_delete_action(&mut self) -> AicedResult<LineChange> {
        let line_number = self.parse_number_field(LINE_FIELD)?;

        Ok(LineChange::Delete {
            line_number,
        })
    }

    fn parse_delete_many_action(&mut self) -> AicedResult<LineChange> {
        let start_line = self.parse_number_field(START_LINE_FIELD)?;
        let end_line = self.parse_number_field(END_LINE_FIELD)?;

        if start_line > end_line {
            return Err(AicedError::parse_error(
                "ParseError",
                Some(self.current),
                "InvalidFormat",
                Some(&format!("Invalid line range: start_line ({}) > end_line ({})", start_line, end_line)))
            );
        }

        Ok(LineChange::DeleteMany {
            start_line,
            end_line,
        })
    }

    fn parse_replace_range_action(&mut self) -> AicedResult<LineChange> {
        let start_line = self.parse_number_field(START_LINE_FIELD)?;
        let end_line = self.parse_number_field(END_LINE_FIELD)?;

        
        if start_line > end_line {
            return Err(AicedError::parse_error(
                "ParseError",
                Some(self.current),
                "InvalidFormat",
                Some(&format!("Invalid line range: start_line ({}) > end_line ({})", start_line, end_line)))
            );
        }

        
        self.expect_line(OLD_LINES_MARKER)?;
        self.advance();
        let old_content = self.parse_lines_until(END_OLD_LINES_MARKER)?;

        
        self.expect_line(NEW_LINES_MARKER)?;
        self.advance();
        let new_content = self.parse_lines_until(END_NEW_LINES_MARKER)?;

        Ok(LineChange::ReplaceRange {
            start_line,
            end_line,
            old_content,
            new_content,
        })
    }

    fn parse_field(&mut self, prefix: &str) -> AicedResult<String> {
        let line = self.current_line();

        let value = line
            .strip_prefix(prefix)
            .ok_or_else(|| AicedError::parse_error("InvalidFormat", Some(self.current + 1), "InvalidFormat", Some(&prefix.to_string())))?
            .trim()
            .to_string();

        self.advance();
        Ok(value)
    }

    fn parse_number_field(&mut self, prefix: &str) -> AicedResult<usize> {
        let value = self.parse_field(prefix)?;
        value.parse::<usize>()
            .map_err(|_| AicedError::parse_error("InvalidNumber", Some(self.current + 1), "InvalidNumber", Some(&value)))
    }

    fn parse_lines_until(&mut self, end_marker: &str) -> AicedResult<Vec<String>> {
        let mut lines = Vec::new();

        while !self.is_eof() && !self.current_line().trim().starts_with(end_marker) {
            lines.push(self.current_line().to_string());
            self.advance();
        }

        if self.is_eof() {
            return Err(AicedError::parse_error("UnexpectedEof", Some(self.current + 1), "UnexpectedEof", Some(&format!("looking for {}", end_marker))));
        }

        self.expect_line(end_marker)?;
        self.advance();

        Ok(lines)
    }

    fn expect_line(&self, expected: &str) -> AicedResult<()> {
        let line = self.current_line().trim();
        if !line.starts_with(expected) {
            return Err(AicedError::parse_error("expect_line", Some(self.current + 1), expected, Some(line)));
        }
        Ok(())
    }
}

