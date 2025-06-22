use std::collections::HashSet;
use std::sync::Arc;
use dashmap::DashMap;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use crate::enums::file_change::FileChange;
use crate::enums::line_change::LineChange;
use crate::structs::config::repository_config::RepositoryConfig;
use crate::errors::AilyzerResult;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiffSession {
    pub id: String,
    pub repository_name: String,
    pub repository_path: String,
    pub files: Vec<FileDiff>,
    pub applied_changes: HashSet<String>,
    pub status: SessionStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileDiff {
    pub file_path: String,
    pub changes: Vec<ChangeItem>,
    pub original_content: String,
    pub preview_content: String,
    pub file_type: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChangeItem {
    pub id: String,
    pub change_type: String,
    pub line_number: usize,
    pub old_content: Option<String>,
    pub new_content: Option<String>,
    pub applied: bool,
    pub reason: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SessionStatus {
    Active,
    Completed,
    Cancelled,
}

pub struct SessionManager {
    sessions: Arc<DashMap<String, DiffSession>>,
}

impl SessionManager {
    pub fn new() -> Self {
        Self {
            sessions: Arc::new(DashMap::new()),
        }
    }

    pub fn create_session(
        &self,
        repository_config: &RepositoryConfig,
        changes: &[FileChange],
    ) -> AilyzerResult<String> {
        let session_id = Uuid::new_v4().to_string();

        let mut files = Vec::new();

        for change in changes {
            match change {
                FileChange::ModifyFile { file_path, reason, line_changes, .. } => {
                    let file_diff = self.create_file_diff(
                        repository_config,
                        file_path,
                        reason,
                        line_changes,
                    )?;
                    files.push(file_diff);
                }
                FileChange::CreateFile { file_path, reason, content, .. } => {
                    let file_diff = self.create_new_file_diff(file_path, reason, content)?;
                    files.push(file_diff);
                }
                FileChange::DeleteFile { file_path, reason, .. } => {
                    let file_diff = self.create_delete_file_diff(
                        repository_config,
                        file_path,
                        reason,
                    )?;
                    files.push(file_diff);
                }
            }
        }

        let session = DiffSession {
            id: session_id.clone(),
            repository_name: repository_config.name.clone(),
            repository_path: repository_config.path.clone(),
            files,
            applied_changes: HashSet::new(),
            status: SessionStatus::Active,
        };

        self.sessions.insert(session_id.clone(), session);
        Ok(session_id)
    }

    pub fn get_session(&self, session_id: &str) -> Option<DiffSession> {
        self.sessions.get(session_id).map(|entry| entry.clone())
    }

    pub fn apply_change(&self, session_id: &str, change_id: &str) -> AilyzerResult<bool> {
        if let Some(mut session) = self.sessions.get_mut(session_id) {
            session.applied_changes.insert(change_id.to_string());

            // Update the change item status
            for file in &mut session.files {
                for change in &mut file.changes {
                    if change.id == change_id {
                        change.applied = true;
                        return Ok(true);
                    }
                }
            }
        }
        Ok(false)
    }

    pub fn unapply_change(&self, session_id: &str, change_id: &str) -> AilyzerResult<bool> {
        if let Some(mut session) = self.sessions.get_mut(session_id) {
            session.applied_changes.remove(change_id);

            // Update the change item status
            for file in &mut session.files {
                for change in &mut file.changes {
                    if change.id == change_id {
                        change.applied = false;
                        return Ok(true);
                    }
                }
            }
        }
        Ok(false)
    }

    pub fn complete_session(&self, session_id: &str) -> AilyzerResult<Vec<String>> {
        if let Some(mut session) = self.sessions.get_mut(session_id) {
            session.status = SessionStatus::Completed;
            Ok(session.applied_changes.iter().cloned().collect())
        } else {
            Ok(Vec::new())
        }
    }

    pub fn cancel_session(&self, session_id: &str) -> AilyzerResult<()> {
        if let Some(mut session) = self.sessions.get_mut(session_id) {
            session.status = SessionStatus::Cancelled;
        }
        Ok(())
    }

    fn create_file_diff(
        &self,
        repository_config: &RepositoryConfig,
        file_path: &str,
        reason: &str,
        line_changes: &[LineChange],
    ) -> AilyzerResult<FileDiff> {
        let full_path = format!("{}/{}", repository_config.path, file_path).replace("//", "/");
        let original_content = std::fs::read_to_string(&full_path)?;

        let mut changes = Vec::new();

        for line_change in line_changes {
            let change_item = self.line_change_to_change_item(line_change, reason)?;
            changes.push(change_item);
        }

        let preview_content = self.apply_changes_to_content(&original_content, line_changes)?;

        let file_type = self.detect_file_type(file_path);

        Ok(FileDiff {
            file_path: file_path.to_string(),
            changes,
            original_content,
            preview_content,
            file_type,
        })
    }

    fn create_new_file_diff(
        &self,
        file_path: &str,
        reason: &str,
        content: &str,
    ) -> AilyzerResult<FileDiff> {
        let change_item = ChangeItem {
            id: Uuid::new_v4().to_string(),
            change_type: "create_file".to_string(),
            line_number: 0,
            old_content: None,
            new_content: Some(content.to_string()),
            applied: false,
            reason: reason.to_string(),
        };

        let file_type = self.detect_file_type(file_path);

        Ok(FileDiff {
            file_path: file_path.to_string(),
            changes: vec![change_item],
            original_content: String::new(),
            preview_content: content.to_string(),
            file_type,
        })
    }

    fn create_delete_file_diff(
        &self,
        repository_config: &RepositoryConfig,
        file_path: &str,
        reason: &str,
    ) -> AilyzerResult<FileDiff> {
        let full_path = format!("{}/{}", repository_config.path, file_path).replace("//", "/");
        let original_content = std::fs::read_to_string(&full_path).unwrap_or_default();

        let change_item = ChangeItem {
            id: Uuid::new_v4().to_string(),
            change_type: "delete_file".to_string(),
            line_number: 0,
            old_content: Some(original_content.clone()),
            new_content: None,
            applied: false,
            reason: reason.to_string(),
        };

        let file_type = self.detect_file_type(file_path);

        Ok(FileDiff {
            file_path: file_path.to_string(),
            changes: vec![change_item],
            original_content,
            preview_content: String::new(),
            file_type,
        })
    }

    fn line_change_to_change_item(
        &self,
        line_change: &LineChange,
        reason: &str,
    ) -> AilyzerResult<ChangeItem> {
        let (change_type, line_number, old_content, new_content) = match line_change {
            LineChange::Replace { line_number, old_content, new_content } => {
                ("replace".to_string(), *line_number, Some(old_content.clone()), Some(new_content.clone()))
            }
            LineChange::InsertAfter { line_number, new_content } => {
                ("insert_after".to_string(), *line_number, None, Some(new_content.clone()))
            }
            LineChange::InsertBefore { line_number, new_content } => {
                ("insert_before".to_string(), *line_number, None, Some(new_content.clone()))
            }
            LineChange::Delete { line_number } => {
                ("delete".to_string(), *line_number, Some("".to_string()), None)
            }
            LineChange::ReplaceRange { start_line, old_content, new_content, .. } => {
                ("replace_range".to_string(), *start_line, Some(old_content.join("\n")), Some(new_content.join("\n")))
            }
            LineChange::InsertManyAfter { line_number, new_lines } => {
                ("insert_many_after".to_string(), *line_number, None, Some(new_lines.join("\n")))
            }
            LineChange::InsertManyBefore { line_number, new_lines } => {
                ("insert_many_before".to_string(), *line_number, None, Some(new_lines.join("\n")))
            }
            LineChange::DeleteMany { start_line, .. } => {
                ("delete_many".to_string(), *start_line, Some("".to_string()), None)
            }
        };

        Ok(ChangeItem {
            id: Uuid::new_v4().to_string(),
            change_type,
            line_number,
            old_content,
            new_content,
            applied: false,
            reason: reason.to_string(),
        })
    }

    fn apply_changes_to_content(
        &self,
        original_content: &str,
        line_changes: &[LineChange],
    ) -> AilyzerResult<String> {
        let mut lines: Vec<String> = original_content.lines().map(|s| s.to_string()).collect();

        // Sort changes by line number in reverse order to avoid index shifting issues
        let mut sorted_changes = line_changes.to_vec();
        sorted_changes.sort_by(|a, b| {
            let line_a = match a {
                LineChange::Replace { line_number, .. } => *line_number,
                LineChange::InsertAfter { line_number, .. } => *line_number,
                LineChange::InsertBefore { line_number, .. } => *line_number,
                LineChange::Delete { line_number } => *line_number,
                LineChange::ReplaceRange { start_line, .. } => *start_line,
                LineChange::InsertManyAfter { line_number, .. } => *line_number,
                LineChange::InsertManyBefore { line_number, .. } => *line_number,
                LineChange::DeleteMany { start_line, .. } => *start_line,
            };
            let line_b = match b {
                LineChange::Replace { line_number, .. } => *line_number,
                LineChange::InsertAfter { line_number, .. } => *line_number,
                LineChange::InsertBefore { line_number, .. } => *line_number,
                LineChange::Delete { line_number } => *line_number,
                LineChange::ReplaceRange { start_line, .. } => *start_line,
                LineChange::InsertManyAfter { line_number, .. } => *line_number,
                LineChange::InsertManyBefore { line_number, .. } => *line_number,
                LineChange::DeleteMany { start_line, .. } => *start_line,
            };
            line_b.cmp(&line_a) // Reverse order
        });

        for change in sorted_changes {
            match change {
                LineChange::Replace { line_number, new_content, .. } => {
                    if line_number > 0 && line_number <= lines.len() {
                        lines[line_number - 1] = new_content;
                    }
                }
                LineChange::InsertAfter { line_number, new_content } => {
                    if line_number <= lines.len() {
                        lines.insert(line_number, new_content);
                    }
                }
                LineChange::InsertBefore { line_number, new_content } => {
                    if line_number > 0 && line_number <= lines.len() {
                        lines.insert(line_number - 1, new_content);
                    }
                }
                LineChange::Delete { line_number } => {
                    if line_number > 0 && line_number <= lines.len() {
                        lines.remove(line_number - 1);
                    }
                }
                _ => {
                    // Handle other change types as needed
                }
            }
        }

        Ok(lines.join("\n"))
    }

    fn detect_file_type(&self, file_path: &str) -> String {
        if let Some(extension) = std::path::Path::new(file_path).extension() {
            match extension.to_str().unwrap_or("") {
                "rs" => "rust".to_string(),
                "js" | "jsx" => "javascript".to_string(),
                "ts" | "tsx" => "typescript".to_string(),
                "py" => "python".to_string(),
                "java" => "java".to_string(),
                "cpp" | "cc" | "cxx" => "cpp".to_string(),
                "c" => "c".to_string(),
                "h" | "hpp" => "c".to_string(),
                "go" => "go".to_string(),
                "php" => "php".to_string(),
                "rb" => "ruby".to_string(),
                "html" => "html".to_string(),
                "css" => "css".to_string(),
                "json" => "json".to_string(),
                "xml" => "xml".to_string(),
                "yaml" | "yml" => "yaml".to_string(),
                "toml" => "toml".to_string(),
                "md" => "markdown".to_string(),
                _ => "text".to_string(),
            }
        } else {
            "text".to_string()
        }
    }
}