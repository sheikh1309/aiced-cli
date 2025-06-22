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

    pub fn get_line_changes(&self) -> Option<&Vec<LineChange>> {
        match self {
            FileChange::ModifyFile { line_changes, .. } => Some(line_changes),
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

    pub fn is_bug_fix(&self) -> bool {
        self.get_category() == Some("BUGS")
    }

    pub fn is_performance_improvement(&self) -> bool {
        self.get_category() == Some("PERFORMANCE")
    }

    pub fn is_duplicate_code_fix(&self) -> bool {
        self.get_category() == Some("DUPLICATE_CODE")
    }

}