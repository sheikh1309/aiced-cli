use serde::{Deserialize, Serialize};
use crate::enums::line_change::LineChange;

#[derive(Debug, Deserialize, Serialize)]
#[serde(tag = "type")]
pub enum FileChange {
    #[serde(rename = "modify_file")]
    ModifyFile {
        file_path: String,
        reason: String,
        severity: String,
        line_changes: Vec<LineChange>,
    },
    #[serde(rename = "create_file")]
    CreateFile {
        file_path: String,
        reason: String,
        severity: String,
        content: String,
    },
    #[serde(rename = "delete_file")]
    DeleteFile {
        file_path: String,
        reason: String,
        severity: String,
    },
}