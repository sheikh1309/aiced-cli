use serde::{Deserialize, Serialize};
use crate::errors::{AilyzerError, AilyzerResult};

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(tag = "action")]
pub enum LineChange {
    #[serde(rename = "replace")]
    Replace {
        line_number: usize,
        old_content: String,
        new_content: String,
    },
    #[serde(rename = "insert_after")]
    InsertAfter {
        line_number: usize,
        new_content: String,
    },
    #[serde(rename = "insert_before")]
    InsertBefore {
        line_number: usize,
        new_content: String,
    },
    #[serde(rename = "delete")]
    Delete {
        line_number: usize,
    },
    #[serde(rename = "replace_range")]
    ReplaceRange {
        start_line: usize,
        end_line: usize,
        old_content: Vec<String>,
        new_content: Vec<String>,
    },
    // NEW: Multi-line actions
    #[serde(rename = "insert_many_after")]
    InsertManyAfter {
        line_number: usize,
        new_lines: Vec<String>,
    },
    #[serde(rename = "insert_many_before")]
    InsertManyBefore {
        line_number: usize,
        new_lines: Vec<String>,
    },
    #[serde(rename = "delete_many")]
    DeleteMany {
        start_line: usize,
        end_line: usize,
    },
}

impl LineChange {
    pub fn is_multi_line(&self) -> bool {
        match self {
            LineChange::Replace { new_content, .. } => {
                // Multi-line if new content contains newlines
                new_content.contains('\n')
            }
            LineChange::InsertAfter { new_content, .. } |
            LineChange::InsertBefore { new_content, .. } => {
                // Multi-line if new content contains newlines
                new_content.contains('\n')
            }
            LineChange::ReplaceRange { start_line, end_line, .. } => {
                // Multi-line if range spans more than one line
                end_line > start_line
            }
            LineChange::Delete { .. } => {
                // Delete only affects one line
                false
            }
            // NEW: Multi-line actions are always multi-line
            LineChange::InsertManyAfter { .. } |
            LineChange::InsertManyBefore { .. } |
            LineChange::DeleteMany { .. } => true,
        }
    }

    pub fn get_affected_line_range(&self) -> (usize, usize) {
        match self {
            LineChange::Replace { line_number, new_content, .. } => {
                if new_content.contains('\n') {
                    let line_count = new_content.lines().count();
                    (*line_number, *line_number + line_count - 1)
                } else {
                    (*line_number, *line_number)
                }
            }
            LineChange::InsertAfter { line_number, new_content } |
            LineChange::InsertBefore { line_number, new_content } => {
                if new_content.contains('\n') {
                    let line_count = new_content.lines().count();
                    (*line_number, *line_number + line_count - 1)
                } else {
                    (*line_number, *line_number)
                }
            }
            LineChange::Delete { line_number } => (*line_number, *line_number),
            LineChange::ReplaceRange { start_line, end_line, .. } => (*start_line, *end_line),
            // NEW: Multi-line actions
            LineChange::InsertManyAfter { line_number, new_lines } => {
                (*line_number, *line_number + new_lines.len() - 1)
            }
            LineChange::InsertManyBefore { line_number, new_lines } => {
                (*line_number, *line_number + new_lines.len() - 1)
            }
            LineChange::DeleteMany { start_line, end_line } => (*start_line, *end_line),
        }
    }

    pub fn get_line_count_delta(&self) -> i32 {
        match self {
            LineChange::Replace { new_content, .. } => {
                if new_content.contains('\n') {
                    new_content.lines().count() as i32 - 1 // Replace 1 line with N lines
                } else {
                    0 // Replace 1 line with 1 line
                }
            }
            LineChange::InsertAfter { new_content, .. } |
            LineChange::InsertBefore { new_content, .. } => {
                if new_content.contains('\n') {
                    new_content.lines().count() as i32
                } else {
                    1 // Insert 1 line
                }
            }
            LineChange::Delete { .. } => -1, // Remove 1 line
            LineChange::ReplaceRange { start_line, end_line, new_content, .. } => {
                let old_line_count = (end_line - start_line + 1) as i32;
                let new_line_count = new_content.len() as i32;
                new_line_count - old_line_count
            }
            // NEW: Multi-line actions
            LineChange::InsertManyAfter { new_lines, .. } |
            LineChange::InsertManyBefore { new_lines, .. } => {
                new_lines.len() as i32
            }
            LineChange::DeleteMany { start_line, end_line } => {
                let deleted_count = end_line - start_line + 1;
                -(deleted_count as i32)
            }
        }
    }

    pub fn get_action_type(&self) -> &str {
        match self {
            LineChange::Replace { .. } => "replace",
            LineChange::InsertAfter { .. } => "insert_after",
            LineChange::InsertBefore { .. } => "insert_before",
            LineChange::Delete { .. } => "delete",
            LineChange::ReplaceRange { .. } => "replace_range",
            // NEW: Multi-line actions
            LineChange::InsertManyAfter { .. } => "insert_many_after",
            LineChange::InsertManyBefore { .. } => "insert_many_before",
            LineChange::DeleteMany { .. } => "delete_many",
        }
    }

    pub fn adds_content(&self) -> bool {
        matches!(self,
            LineChange::Replace { .. } |
            LineChange::InsertAfter { .. } |
            LineChange::InsertBefore { .. } |
            LineChange::ReplaceRange { .. } |
            LineChange::InsertManyAfter { .. } |
            LineChange::InsertManyBefore { .. }
        )
    }

    pub fn removes_content(&self) -> bool {
        matches!(self,
            LineChange::Delete { .. } |
            LineChange::DeleteMany { .. }
        )
    }

    pub fn get_new_content(&self) -> Option<&str> {
        match self {
            LineChange::Replace { new_content, .. } => Some(new_content),
            LineChange::InsertAfter { new_content, .. } => Some(new_content),
            LineChange::InsertBefore { new_content, .. } => Some(new_content),
            LineChange::Delete { .. } => None,
            LineChange::ReplaceRange { .. } => None, // ReplaceRange has Vec<String>, not single string
            // NEW: Multi-line actions don't have single string content
            LineChange::InsertManyAfter { .. } |
            LineChange::InsertManyBefore { .. } |
            LineChange::DeleteMany { .. } => None,
        }
    }

    pub fn get_new_content_lines(&self) -> Option<&Vec<String>> {
        match self {
            LineChange::ReplaceRange { new_content, .. } => Some(new_content),
            // NEW: Multi-line actions
            LineChange::InsertManyAfter { new_lines, .. } |
            LineChange::InsertManyBefore { new_lines, .. } => Some(new_lines),
            _ => None,
        }
    }

    pub fn get_old_content(&self) -> Option<&str> {
        match self {
            LineChange::Replace { old_content, .. } => Some(old_content),
            _ => None,
        }
    }

    pub fn get_old_content_lines(&self) -> Option<&Vec<String>> {
        match self {
            LineChange::ReplaceRange { old_content, .. } => Some(old_content),
            _ => None,
        }
    }

    pub fn conflicts_with(&self, other: &LineChange) -> bool {
        let (self_start, self_end) = self.get_affected_line_range();
        let (other_start, other_end) = other.get_affected_line_range();

        // Check for overlap
        !(self_end < other_start || other_end < self_start)
    }

    pub fn get_description(&self) -> String {
        match self {
            LineChange::Replace { line_number, .. } => {
                if self.is_multi_line() {
                    format!("Replace line {} with multiple lines", line_number)
                } else {
                    format!("Replace line {}", line_number)
                }
            }
            LineChange::InsertAfter { line_number, .. } => {
                if self.is_multi_line() {
                    format!("Insert multiple lines after line {}", line_number)
                } else {
                    format!("Insert line after line {}", line_number)
                }
            }
            LineChange::InsertBefore { line_number, .. } => {
                if self.is_multi_line() {
                    format!("Insert multiple lines before line {}", line_number)
                } else {
                    format!("Insert line before line {}", line_number)
                }
            }
            LineChange::Delete { line_number } => {
                format!("Delete line {}", line_number)
            }
            LineChange::ReplaceRange { start_line, end_line, new_content, .. } => {
                format!("Replace lines {}-{} with {} lines", start_line, end_line, new_content.len())
            }
            // NEW: Multi-line actions
            LineChange::InsertManyAfter { line_number, new_lines } => {
                format!("Insert {} lines after line {}", new_lines.len(), line_number)
            }
            LineChange::InsertManyBefore { line_number, new_lines } => {
                format!("Insert {} lines before line {}", new_lines.len(), line_number)
            }
            LineChange::DeleteMany { start_line, end_line } => {
                format!("Delete lines {}-{}", start_line, end_line)
            }
        }
    }

    pub fn validate(&self) -> AilyzerResult<()> {
        match self {
            LineChange::Replace { line_number, old_content, new_content } => {
                if *line_number == 0 {
                    return Err(AilyzerError::system_error("validate line", "Line number cannot be 0"));
                }
                if old_content.is_empty() {
                    return Err(AilyzerError::system_error("validate line", "Old content cannot be empty for replace operation"));
                }
                if new_content.is_empty() {
                    return Err(AilyzerError::system_error("validate line", "New content cannot be empty for replace operation"));
                }
            }
            LineChange::InsertAfter { line_number, new_content } |
            LineChange::InsertBefore { line_number, new_content } => {
                if *line_number == 0 && matches!(self, LineChange::InsertBefore { .. }) {
                    return Err(AilyzerError::system_error("validate line", "Line number cannot be 0 for insert_before operation"));
                }
                if new_content.is_empty() {
                    return Err(AilyzerError::system_error("validate line", "New content cannot be empty for insert operation"));
                }
            }
            LineChange::Delete { line_number } => {
                if *line_number == 0 {
                    return Err(AilyzerError::system_error("validate line", "Line number cannot be 0"));
                }
            }
            LineChange::ReplaceRange { start_line, end_line, old_content, new_content: _ } => {
                if *start_line == 0 {
                    return Err(AilyzerError::system_error("validate line", "Start line cannot be 0"));
                }
                if start_line > end_line {
                    return Err(AilyzerError::system_error("validate line", "Start line cannot be greater than end line"));
                }
                if old_content.is_empty() {
                    return Err(AilyzerError::system_error("validate line", "Old content cannot be empty for replace_range operation"));
                }
            }
            LineChange::InsertManyAfter { line_number, new_lines } |
            LineChange::InsertManyBefore { line_number, new_lines } => {
                if *line_number == 0 && matches!(self, LineChange::InsertManyBefore { .. }) {
                    return Err(AilyzerError::system_error("validate line", "Line number cannot be 0 for insert_many_before operation"));
                }
                if new_lines.is_empty() {
                    return Err(AilyzerError::system_error("validate line", "New lines cannot be empty for multi-line insert operation"));
                }
                for (i, line) in new_lines.iter().enumerate() {
                    if line.trim().is_empty() {
                        return Err(AilyzerError::system_error("validate line", &format!("Line {} in new_lines is empty or whitespace-only", i + 1)));
                    }
                }
            }
            LineChange::DeleteMany { start_line, end_line } => {
                if *start_line == 0 {
                    return Err(AilyzerError::system_error("validate line", "Start line cannot be 0"));
                }
                if start_line > end_line {
                    return Err(AilyzerError::system_error("validate line", "Start line cannot be greater than end line"));
                }
            }
        }
        Ok(())
    }

    pub fn get_line_number(&self) -> usize {
        match self {
            LineChange::Replace { line_number, .. } => *line_number,
            LineChange::InsertAfter { line_number, .. } => *line_number,
            LineChange::InsertBefore { line_number, .. } => *line_number,
            LineChange::InsertManyAfter { line_number, .. } => *line_number,
            LineChange::InsertManyBefore { line_number, .. } => *line_number,
            LineChange::Delete { line_number } => *line_number,
            LineChange::DeleteMany { start_line, .. } => *start_line,
            LineChange::ReplaceRange { start_line, .. } => *start_line,
        }
    }

    pub fn get_line_range(&self) -> (usize, usize) {
        match self {
            LineChange::Replace { line_number, .. } => (*line_number, *line_number),
            LineChange::InsertAfter { line_number, .. } => (*line_number, *line_number),
            LineChange::InsertBefore { line_number, .. } => (*line_number, *line_number),
            LineChange::InsertManyAfter { line_number, .. } => (*line_number, *line_number),
            LineChange::InsertManyBefore { line_number, .. } => (*line_number, *line_number),
            LineChange::Delete { line_number } => (*line_number, *line_number),
            LineChange::DeleteMany { start_line, end_line } => (*start_line, *end_line),
            LineChange::ReplaceRange { start_line, end_line, .. } => (*start_line, *end_line),
        }
    }

    pub fn get_line_delta(&self) -> i32 {
        self.get_line_count_delta()
    }

    pub fn is_multi_line_operation(&self) -> bool {
        matches!(self,
            LineChange::InsertManyAfter { .. } |
            LineChange::InsertManyBefore { .. } |
            LineChange::DeleteMany { .. } |
            LineChange::ReplaceRange { .. }
        )
    }

    pub fn describe(&self) -> String {
        self.get_description()
    }

    pub fn modifies_existing_content(&self) -> bool {
        matches!(self,
            LineChange::Replace { .. } |
            LineChange::ReplaceRange { .. }
        )
    }

    pub fn only_adds_content(&self) -> bool {
        matches!(self,
            LineChange::InsertAfter { .. } |
            LineChange::InsertBefore { .. } |
            LineChange::InsertManyAfter { .. } |
            LineChange::InsertManyBefore { .. }
        )
    }

    pub fn only_removes_content(&self) -> bool {
        matches!(self,
            LineChange::Delete { .. } |
            LineChange::DeleteMany { .. }
        )
    }

    pub fn lines_added(&self) -> usize {
        match self {
            LineChange::Replace { new_content, .. } => {
                if new_content.contains('\n') {
                    new_content.lines().count()
                } else {
                    1
                }
            }
            LineChange::InsertAfter { new_content, .. } |
            LineChange::InsertBefore { new_content, .. } => {
                if new_content.contains('\n') {
                    new_content.lines().count()
                } else {
                    1
                }
            }
            LineChange::InsertManyAfter { new_lines, .. } |
            LineChange::InsertManyBefore { new_lines, .. } => new_lines.len(),
            LineChange::ReplaceRange { new_content, .. } => new_content.len(),
            LineChange::Delete { .. } |
            LineChange::DeleteMany { .. } => 0,
        }
    }

    pub fn lines_removed(&self) -> usize {
        match self {
            LineChange::Replace { .. } => 1,
            LineChange::Delete { .. } => 1,
            LineChange::DeleteMany { start_line, end_line } => end_line - start_line + 1,
            LineChange::ReplaceRange { start_line, end_line, .. } => end_line - start_line + 1,
            LineChange::InsertAfter { .. } |
            LineChange::InsertBefore { .. } |
            LineChange::InsertManyAfter { .. } |
            LineChange::InsertManyBefore { .. } => 0,
        }
    }
}