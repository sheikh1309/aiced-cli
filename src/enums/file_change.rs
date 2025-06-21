use crate::enums::line_change::LineChange;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FileChange {
    ModifyFile {
        file_path: String,
        reason: String,
        severity: String,
        category: String,
        line_changes: Vec<LineChange>,
    },
    CreateFile {
        file_path: String,
        reason: String,
        severity: String,
        category: String,
        content: String,
    },
    DeleteFile {
        file_path: String,
        reason: String,
        severity: String,
        category: String,
    },
}

impl FileChange {
    pub fn get_file_path(&self) -> &str {
        match self {
            FileChange::ModifyFile { file_path, .. } => file_path,
            FileChange::CreateFile { file_path, .. } => file_path,
            FileChange::DeleteFile { file_path, .. } => file_path,
        }
    }

    pub fn get_reason(&self) -> &str {
        match self {
            FileChange::ModifyFile { reason, .. } => reason,
            FileChange::CreateFile { reason, .. } => reason,
            FileChange::DeleteFile { reason, .. } => reason,
        }
    }

    pub fn get_severity(&self) -> &str {
        match self {
            FileChange::ModifyFile { severity, .. } => severity,
            FileChange::CreateFile { severity, .. } => severity,
            FileChange::DeleteFile { severity, .. } => severity,
        }
    }

    pub fn get_category(&self) -> Option<&str> {
        match self {
            FileChange::ModifyFile { category, .. } => Some(category),
            FileChange::CreateFile { category, .. } => Some(category),
            FileChange::DeleteFile { category, .. } => Some(category),
        }
    }

    pub fn get_change_type(&self) -> &str {
        match self {
            FileChange::ModifyFile { .. } => "modify_file",
            FileChange::CreateFile { .. } => "create_file",
            FileChange::DeleteFile { .. } => "delete_file",
        }
    }

    pub fn get_line_changes(&self) -> Option<&Vec<LineChange>> {
        match self {
            FileChange::ModifyFile { line_changes, .. } => Some(line_changes),
            _ => None,
        }
    }

    pub fn get_content(&self) -> Option<&str> {
        match self {
            FileChange::CreateFile { content, .. } => Some(content),
            _ => None,
        }
    }

    pub fn is_critical(&self) -> bool {
        self.get_severity() == "critical"
    }

    pub fn is_high_priority(&self) -> bool {
        matches!(self.get_severity(), "critical" | "high")
    }

    pub fn is_security_related(&self) -> bool {
        self.get_category() == Some("SECURITY")
    }

    pub fn is_architecture_related(&self) -> bool {
        self.get_category() == Some("ARCHITECTURE")
    }

    pub fn is_clean_code_related(&self) -> bool {
        self.get_category() == Some("CLEAN_CODE")
    }

    pub fn get_line_change_count(&self) -> usize {
        match self {
            FileChange::ModifyFile { line_changes, .. } => line_changes.len(),
            _ => 0,
        }
    }

    pub fn affects_multiple_lines(&self) -> bool {
        match self {
            FileChange::ModifyFile { line_changes, .. } => {
                // Check if any line change affects multiple lines
                line_changes.iter().any(|change| {
                    match change {
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
                })
            }
            FileChange::CreateFile { content, .. } => {
                // Multi-line if content has multiple lines
                content.lines().count() > 1
            }
            FileChange::DeleteFile { .. } => {
                // Delete file doesn't affect multiple lines
                false
            }
        }
    }

    pub fn is_bug_fix(&self) -> bool {
        self.get_category() == Some("BUGS")
    }

    pub fn is_performance_improvement(&self) -> bool {
        self.get_category() == Some("PERFORMANCE")
    }

    pub fn is_duplicate_code_fix(&self) -> bool {
        self.get_category() == Some("DUPLICATE_CODE")
    }

    pub fn get_priority_score(&self) -> u8 {
        // Calculate priority score based on severity and category
        let severity_score = match self.get_severity() {
            "critical" => 4,
            "high" => 3,
            "medium" => 2,
            "low" => 1,
            _ => 0,
        };

        let category_bonus = match self.get_category() {
            Some("SECURITY") => 2,  // Security gets highest bonus
            Some("BUGS") => 1,      // Bugs get medium bonus
            Some("PERFORMANCE") => 1,
            _ => 0,
        };

        severity_score + category_bonus
    }

    pub fn should_apply_immediately(&self) -> bool {
        // Critical security issues and bugs should be applied immediately
        self.is_critical() && (self.is_security_related() || self.is_bug_fix())
    }

    pub fn get_estimated_impact(&self) -> &str {
        match (self.get_severity(), self.get_category()) {
            ("critical", Some("SECURITY")) => "High - Critical security vulnerability",
            ("critical", Some("BUGS")) => "High - Critical bug that may cause system failure",
            ("high", Some("SECURITY")) => "Medium-High - Security issue requiring attention",
            ("high", Some("PERFORMANCE")) => "Medium - Performance bottleneck",
            ("medium", Some("CLEAN_CODE")) => "Low-Medium - Code quality improvement",
            ("low", _) => "Low - Minor improvement",
            _ => "Unknown impact",
        }
    }
}