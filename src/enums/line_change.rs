use serde::{Deserialize, Serialize};
use crate::errors::{AicedError, AicedResult};

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
                
                new_content.contains('\n')
            }
            LineChange::InsertAfter { new_content, .. } |
            LineChange::InsertBefore { new_content, .. } => {
                
                new_content.contains('\n')
            }
            LineChange::ReplaceRange { start_line, end_line, .. } => {
                
                end_line > start_line
            }
            LineChange::Delete { .. } => {
                
                false
            }
            
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
            
            LineChange::InsertManyAfter { line_number, new_lines } => {
                (*line_number, *line_number + new_lines.len() - 1)
            }
            LineChange::InsertManyBefore { line_number, new_lines } => {
                (*line_number, *line_number + new_lines.len() - 1)
            }
            LineChange::DeleteMany { start_line, end_line } => (*start_line, *end_line),
        }
    }

    pub fn conflicts_with(&self, other: &LineChange) -> bool {
        let (self_start, self_end) = self.get_affected_line_range();
        let (other_start, other_end) = other.get_affected_line_range();

        
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

    pub fn validate(&self) -> AicedResult<()> {
        match self {
            LineChange::Replace { line_number, old_content, new_content } => {
                if *line_number == 0 {
                    return Err(AicedError::system_error("validate line", "Line number cannot be 0"));
                }
                if old_content.is_empty() {
                    return Err(AicedError::system_error("validate line", "Old content cannot be empty for replace operation"));
                }
                if new_content.is_empty() {
                    return Err(AicedError::system_error("validate line", "New content cannot be empty for replace operation"));
                }
            }
            LineChange::InsertAfter { line_number, new_content } |
            LineChange::InsertBefore { line_number, new_content } => {
                if *line_number == 0 && matches!(self, LineChange::InsertBefore { .. }) {
                    return Err(AicedError::system_error("validate line", "Line number cannot be 0 for insert_before operation"));
                }
                if new_content.is_empty() {
                    return Err(AicedError::system_error("validate line", "New content cannot be empty for insert operation"));
                }
            }
            LineChange::Delete { line_number } => {
                if *line_number == 0 {
                    return Err(AicedError::system_error("validate line", "Line number cannot be 0"));
                }
            }
            LineChange::ReplaceRange { start_line, end_line, old_content, new_content: _ } => {
                if *start_line == 0 {
                    return Err(AicedError::system_error("validate line", "Start line cannot be 0"));
                }
                if start_line > end_line {
                    return Err(AicedError::system_error("validate line", "Start line cannot be greater than end line"));
                }
                if old_content.is_empty() {
                    return Err(AicedError::system_error("validate line", "Old content cannot be empty for replace_range operation"));
                }
            }
            LineChange::InsertManyAfter { line_number, new_lines } |
            LineChange::InsertManyBefore { line_number, new_lines } => {
                if *line_number == 0 && matches!(self, LineChange::InsertManyBefore { .. }) {
                    return Err(AicedError::system_error("validate line", "Line number cannot be 0 for insert_many_before operation"));
                }
                if new_lines.is_empty() {
                    return Err(AicedError::system_error("validate line", "New lines cannot be empty for multi-line insert operation"));
                }
                for (i, line) in new_lines.iter().enumerate() {
                    if line.trim().is_empty() {
                        return Err(AicedError::system_error("validate line", &format!("Line {} in new_lines is empty or whitespace-only", i + 1)));
                    }
                }
            }
            LineChange::DeleteMany { start_line, end_line } => {
                if *start_line == 0 {
                    return Err(AicedError::system_error("validate line", "Start line cannot be 0"));
                }
                if start_line > end_line {
                    return Err(AicedError::system_error("validate line", "Start line cannot be greater than end line"));
                }
            }
        }
        Ok(())
    }
}