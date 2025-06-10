use crate::enums::line_change::LineChange;

pub enum FileChange {
    ModifyFile {
        file_path: String,
        reason: String,
        severity: String,
        line_changes: Vec<LineChange>,
    },
    CreateFile {
        file_path: String,
        reason: String,
        severity: String,
        content: String,
    },
    DeleteFile {
        file_path: String,
        reason: String,
        severity: String,
    },
}