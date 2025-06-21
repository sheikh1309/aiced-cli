use crate::enums::file_change::FileChange;
use crate::enums::line_change::LineChange;
use crate::structs::analysis_response::AnalysisResponse;
use std::collections::HashMap;

pub struct Parser {
    lines: Vec<String>,
    current: usize,
}

impl Parser {
    pub fn new(input: &str) -> Self {
        Self {
            lines: input.lines().map(|s| s.to_string()).collect(),
            current: 0,
        }
    }

    pub fn parse(&mut self) -> Result<AnalysisResponse, String> {
        let mut response = AnalysisResponse {
            analysis_summary: String::new(),
            changes: Vec::new()
        };

        response.analysis_summary = self.parse_summary()?;

        while self.current < self.lines.len() {
            if self.current_line().trim().is_empty() {
                self.current += 1;
                continue;
            }

            if self.current_line().starts_with("CHANGE:") {
                match self.parse_change() {
                    Ok(change) => {
                        response.changes.push(change);
                    }
                    Err(e) => {
                        eprintln!("❌ Error parsing change at line {}: {}", self.current + 1, e);
                        // Try to recover by finding the next CHANGE: or END_CHANGE
                        self.skip_to_next_change();
                    }
                }
            } else {
                self.current += 1;
            }
        }

        Ok(response)
    }

    fn skip_to_next_change(&mut self) {
        while self.current < self.lines.len() {
            if self.current_line().starts_with("CHANGE:") ||
                self.current_line().starts_with("END_CHANGE") {
                break;
            }
            self.current += 1;
        }
        if self.current_line().starts_with("END_CHANGE") {
            self.current += 1; // Move past END_CHANGE
        }
    }

    fn current_line(&self) -> &str {
        self.lines.get(self.current).map(|s| s.as_str()).unwrap_or("")
    }

    fn peek_line(&self, offset: usize) -> &str {
        self.lines.get(self.current + offset).map(|s| s.as_str()).unwrap_or("")
    }

    fn parse_summary(&mut self) -> Result<String, String> {
        self.expect_line("ANALYSIS_SUMMARY:")?;
        self.advance();

        let mut summary = Vec::new();
        while self.current < self.lines.len() && !self.current_line().starts_with("CHANGE:") {
            if !self.current_line().trim().is_empty() {
                summary.push(self.current_line().to_string());
            }
            self.advance();
        }

        Ok(summary.join("\n"))
    }

    fn parse_change(&mut self) -> Result<FileChange, String> {
        let line = self.current_line();
        let change_type = line
            .strip_prefix("CHANGE:")
            .ok_or_else(|| format!("Expected 'CHANGE:' at line {}", self.current + 1))?
            .trim()
            .to_string(); // Convert to owned String to extend lifetime

        self.advance();

        match change_type.as_str() {
            "modify_file" => self.parse_modify_file(),
            "create_file" => self.parse_create_file(),
            "delete_file" => self.parse_delete_file(),
            _ => Err(format!("Unknown change type: {}", change_type)),
        }
    }

    fn parse_modify_file(&mut self) -> Result<FileChange, String> {
        let mut fields = HashMap::new();
        let mut line_changes = Vec::new();

        while self.current < self.lines.len() && !self.current_line().starts_with("END_CHANGE") {
            let line = self.current_line().trim();

            if line.starts_with("FILE:") {
                fields.insert("file_path", self.parse_field("FILE:")?);
            } else if line.starts_with("REASON:") {
                fields.insert("reason", self.parse_field("REASON:")?);
            } else if line.starts_with("SEVERITY:") {
                fields.insert("severity", self.parse_field("SEVERITY:")?);
            } else if line.starts_with("ACTION:") {
                match self.parse_line_action() {
                    Ok(action) => line_changes.push(action),
                    Err(e) => {
                        eprintln!("⚠️ Warning: Failed to parse action at line {}: {}", self.current + 1, e);
                        // Skip to next ACTION or END_CHANGE
                        self.skip_to_next_action();
                    }
                }
            } else {
                self.advance();
            }
        }

        self.expect_line("END_CHANGE")?;
        self.advance();

        Ok(FileChange::ModifyFile {
            file_path: fields.get("file_path")
                .ok_or("Missing FILE field")?
                .to_string(),
            reason: fields.get("reason")
                .ok_or("Missing REASON field")?
                .to_string(),
            severity: fields.get("severity")
                .ok_or("Missing SEVERITY field")?
                .to_string(),
            line_changes,
        })
    }

    fn skip_to_next_action(&mut self) {
        self.advance();
        while self.current < self.lines.len() {
            let line = self.current_line().trim();
            if line.starts_with("ACTION:") || line.starts_with("END_CHANGE") {
                break;
            }
            self.advance();
        }
    }

    fn parse_line_action(&mut self) -> Result<LineChange, String> {
        let action_type = self.parse_field("ACTION:")?;

        match action_type.as_str() {
            "replace" => {
                let line_number = self.parse_number_field("LINE:")?;
                let old_content = self.parse_field("OLD:")?;
                let new_content = self.parse_field("NEW:")?;

                Ok(LineChange::Replace {
                    line_number,
                    old_content,
                    new_content,
                })
            }
            "insert_after" => {
                let line_number = self.parse_number_field("LINE:")?;
                let new_content = self.parse_field("NEW:")?;

                Ok(LineChange::InsertAfter {
                    line_number,
                    new_content,
                })
            }
            "insert_before" => {
                let line_number = self.parse_number_field("LINE:")?;
                let new_content = self.parse_field("NEW:")?;

                Ok(LineChange::InsertBefore {
                    line_number,
                    new_content,
                })
            }
            "delete" => {
                let line_number = self.parse_number_field("LINE:")?;

                Ok(LineChange::Delete {
                    line_number,
                })
            }
            "replace_range" => {
                let start_line = self.parse_number_field("START_LINE:")?;
                let end_line = self.parse_number_field("END_LINE:")?;

                // Parse OLD_LINES block
                self.expect_line("OLD_LINES:")?;
                self.advance();
                let old_content = self.parse_lines_until("END_OLD_LINES")?;

                // Parse NEW_LINES block
                self.expect_line("NEW_LINES:")?;
                self.advance();
                let new_content = self.parse_lines_until("END_NEW_LINES")?;

                Ok(LineChange::ReplaceRange {
                    start_line,
                    end_line,
                    old_content,
                    new_content,
                })
            }
            _ => Err(format!("Unknown action type: {}", action_type)),
        }
    }

    fn parse_create_file(&mut self) -> Result<FileChange, String> {
        let mut fields = HashMap::new();
        let mut content = String::new();

        while self.current < self.lines.len() && !self.current_line().starts_with("END_CHANGE") {
            let line = self.current_line().trim();

            if line.starts_with("FILE:") {
                fields.insert("file_path", self.parse_field("FILE:")?);
            } else if line.starts_with("REASON:") {
                fields.insert("reason", self.parse_field("REASON:")?);
            } else if line.starts_with("SEVERITY:") {
                fields.insert("severity", self.parse_field("SEVERITY:")?);
            } else if line.starts_with("CONTENT:") {
                self.advance();
                content = self.parse_content_until("END_CONTENT")?;
            } else {
                self.advance();
            }
        }

        self.expect_line("END_CHANGE")?;
        self.advance();

        Ok(FileChange::CreateFile {
            file_path: fields.get("file_path")
                .ok_or("Missing FILE field")?
                .to_string(),
            reason: fields.get("reason")
                .ok_or("Missing REASON field")?
                .to_string(),
            severity: fields.get("severity")
                .ok_or("Missing SEVERITY field")?
                .to_string(),
            content,
        })
    }

    fn parse_delete_file(&mut self) -> Result<FileChange, String> {
        let mut fields = HashMap::new();

        while self.current < self.lines.len() && !self.current_line().starts_with("END_CHANGE") {
            let line = self.current_line().trim();

            if line.starts_with("FILE:") {
                fields.insert("file_path", self.parse_field("FILE:")?);
            } else if line.starts_with("REASON:") {
                fields.insert("reason", self.parse_field("REASON:")?);
            } else if line.starts_with("SEVERITY:") {
                fields.insert("severity", self.parse_field("SEVERITY:")?);
            }
            self.advance();
        }

        self.expect_line("END_CHANGE")?;
        self.advance();

        Ok(FileChange::DeleteFile {
            file_path: fields.get("file_path")
                .ok_or("Missing FILE field")?
                .to_string(),
            reason: fields.get("reason")
                .ok_or("Missing REASON field")?
                .to_string(),
            severity: fields.get("severity")
                .ok_or("Missing SEVERITY field")?
                .to_string(),
        })
    }

    fn parse_field(&mut self, prefix: &str) -> Result<String, String> {
        let line = self.current_line();
        let value = line
            .strip_prefix(prefix)
            .ok_or_else(|| {
                let context = format!(
                    "Expected '{}' at line {}, found '{}'\nNext line: '{}'",
                    prefix,
                    self.current + 1,
                    line,
                    self.peek_line(1)
                );
                context
            })?
            .replacen(" ", "", 1)
            .to_string();

        self.advance();
        Ok(value)
    }

    fn parse_number_field(&mut self, prefix: &str) -> Result<usize, String> {
        let value = self.parse_field(prefix)?;
        value.parse::<usize>()
            .map_err(|_| format!("Invalid number '{}' for field {}", value, prefix))
    }

    fn parse_lines_until(&mut self, end_marker: &str) -> Result<Vec<String>, String> {
        let mut lines = Vec::new();

        while self.current < self.lines.len() && !self.current_line().trim().starts_with(end_marker) {
            lines.push(self.current_line().to_string());
            self.advance();
        }

        self.expect_line(end_marker)?;
        self.advance();

        Ok(lines)
    }

    fn parse_content_until(&mut self, end_marker: &str) -> Result<String, String> {
        let lines = self.parse_lines_until(end_marker)?;
        Ok(lines.join("\n"))
    }

    fn expect_line(&self, expected: &str) -> Result<(), String> {
        let line = self.current_line().trim();
        if !line.starts_with(expected) {
            return Err(format!(
                "Expected '{}' at line {}, found '{}'\nContext: Previous line: '{}', Next line: '{}'",
                expected,
                self.current + 1,
                line,
                if self.current > 0 { self.lines.get(self.current - 1).map(|s| s.as_str()).unwrap_or("") } else { "" },
                self.peek_line(1)
            ));
        }
        Ok(())
    }

    fn advance(&mut self) {
        self.current += 1;
    }
}