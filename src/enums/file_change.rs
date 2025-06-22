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
}