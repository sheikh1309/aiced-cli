use serde::{Deserialize, Serialize};

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
        }
    }

    pub fn get_action_type(&self) -> &str {
        match self {
            LineChange::Replace { .. } => "replace",
            LineChange::InsertAfter { .. } => "insert_after",
            LineChange::InsertBefore { .. } => "insert_before",
            LineChange::Delete { .. } => "delete",
            LineChange::ReplaceRange { .. } => "replace_range",
        }
    }

    pub fn adds_content(&self) -> bool {
        matches!(self,
            LineChange::Replace { .. } |
            LineChange::InsertAfter { .. } |
            LineChange::InsertBefore { .. } |
            LineChange::ReplaceRange { .. }
        )
    }

    pub fn removes_content(&self) -> bool {
        matches!(self, LineChange::Delete { .. })
    }

    pub fn get_new_content(&self) -> Option<&str> {
        match self {
            LineChange::Replace { new_content, .. } => Some(new_content),
            LineChange::InsertAfter { new_content, .. } => Some(new_content),
            LineChange::InsertBefore { new_content, .. } => Some(new_content),
            LineChange::Delete { .. } => None,
            LineChange::ReplaceRange { .. } => None, // ReplaceRange has Vec<String>, not single string
        }
    }

    pub fn get_new_content_lines(&self) -> Option<&Vec<String>> {
        match self {
            LineChange::ReplaceRange { new_content, .. } => Some(new_content),
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
        }
    }

    pub fn validate(&self) -> Result<(), String> {
        match self {
            LineChange::Replace { line_number, old_content, new_content } => {
                if *line_number == 0 {
                    return Err("Line number cannot be 0".to_string());
                }
                if old_content.is_empty() {
                    return Err("Old content cannot be empty for replace operation".to_string());
                }
                if new_content.is_empty() {
                    return Err("New content cannot be empty for replace operation".to_string());
                }
            }
            LineChange::InsertAfter { line_number, new_content } |
            LineChange::InsertBefore { line_number, new_content } => {
                if *line_number == 0 && matches!(self, LineChange::InsertBefore { .. }) {
                    return Err("Line number cannot be 0 for insert_before operation".to_string());
                }
                if new_content.is_empty() {
                    return Err("New content cannot be empty for insert operation".to_string());
                }
            }
            LineChange::Delete { line_number } => {
                if *line_number == 0 {
                    return Err("Line number cannot be 0".to_string());
                }
            }
            LineChange::ReplaceRange { start_line, end_line, old_content, new_content: _ } => {
                if *start_line == 0 {
                    return Err("Start line cannot be 0".to_string());
                }
                if start_line > end_line {
                    return Err("Start line cannot be greater than end line".to_string());
                }
                if old_content.is_empty() {
                    return Err("Old content cannot be empty for replace_range operation".to_string());
                }
            }
        }
        Ok(())
    }
}