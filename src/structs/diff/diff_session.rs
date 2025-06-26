use std::collections::HashSet;
use serde::{Deserialize, Serialize};
use crate::enums::session_status::SessionStatus;
use crate::structs::diff::file_diff::FileDiff;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiffSession {
    pub id: String,
    pub repository_name: String,
    pub repository_path: String,
    pub files: Vec<FileDiff>,
    pub applied_changes: HashSet<String>,
    pub status: SessionStatus,
}