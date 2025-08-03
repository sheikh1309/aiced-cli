use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use dashmap::DashMap;
use uuid::Uuid;
use crate::enums::file_change::FileChange;
use crate::enums::line_change::LineChange;
use crate::enums::session_status::SessionStatus;
use crate::structs::config::repository_config::RepositoryConfig;
use crate::errors::AicedResult;
use crate::structs::diff::change_item::ChangeItem;
use crate::structs::diff::diff_session::DiffSession;
use crate::structs::diff::file_diff::FileDiff;

pub struct SessionManager {
    sessions: Arc<DashMap<String, DiffSession>>,
}

impl SessionManager {
    pub fn new() -> Self {
        Self {
            sessions: Arc::new(DashMap::new()),
        }
    }

    pub fn create_session(&self, repository_config: &RepositoryConfig, changes: &[FileChange]) -> AicedResult<String> {
        let session_id = Uuid::new_v4().to_string();

        let mut files_map: HashMap<String, FileDiff> = HashMap::new();
        
        for change in changes {
            match change {
                FileChange::ModifyFile { file_path, reason, line_changes, .. } => {
                    if files_map.contains_key(file_path) {
                        if let Some(file_diff) = files_map.get_mut(file_path) {
                            // Add the new change items
                            for line_change in line_changes {
                                let change_item = self.line_change_to_change_item(line_change, reason)?;
                                file_diff.changes.push(change_item);
                            }
                            
                            // Apply all changes cumulatively
                            // First, collect all line changes from all change items
                            let mut all_line_changes = Vec::new();
                            for change_item in &file_diff.changes {
                                match change_item.change_type.as_str() {
                                    "replace" => {
                                        if let (Some(new_content), line_number) = (&change_item.new_content, change_item.line_number) {
                                            all_line_changes.push(LineChange::Replace {
                                                line_number,
                                                old_content: "".to_string(), // We don't need this for applying changes
                                                new_content: new_content.clone(),
                                            });
                                        }
                                    }
                                    "insert_after" => {
                                        if let (Some(new_content), line_number) = (&change_item.new_content, change_item.line_number) {
                                            all_line_changes.push(LineChange::InsertAfter {
                                                line_number,
                                                new_content: new_content.clone(),
                                            });
                                        }
                                    }
                                    "insert_before" => {
                                        if let (Some(new_content), line_number) = (&change_item.new_content, change_item.line_number) {
                                            all_line_changes.push(LineChange::InsertBefore {
                                                line_number,
                                                new_content: new_content.clone(),
                                            });
                                        }
                                    }
                                    "delete" => {
                                        all_line_changes.push(LineChange::Delete {
                                            line_number: change_item.line_number,
                                        });
                                    }
                                    "replace_range" => {
                                        if let (Some(old_content), Some(new_content), line_number) = (&change_item.old_content, &change_item.new_content, change_item.line_number) {
                                            // Parse the old content to determine end_line
                                            let old_lines = old_content.lines().count();
                                            let end_line = line_number + old_lines - 1;
                                            
                                            // Parse the new content into lines
                                            let new_lines: Vec<String> = new_content.lines().map(String::from).collect();
                                            
                                            all_line_changes.push(LineChange::ReplaceRange {
                                                start_line: line_number,
                                                end_line,
                                                old_content: Vec::new(), // Not needed for applying
                                                new_content: new_lines,
                                            });
                                        }
                                    }
                                    "insert_many_after" => {
                                        if let (Some(new_content), line_number) = (&change_item.new_content, change_item.line_number) {
                                            let new_lines: Vec<String> = new_content.lines().map(String::from).collect();
                                            
                                            all_line_changes.push(LineChange::InsertManyAfter {
                                                line_number,
                                                new_lines,
                                            });
                                        }
                                    }
                                    "insert_many_before" => {
                                        if let (Some(new_content), line_number) = (&change_item.new_content, change_item.line_number) {
                                            let new_lines: Vec<String> = new_content.lines().map(String::from).collect();
                                            
                                            all_line_changes.push(LineChange::InsertManyBefore {
                                                line_number,
                                                new_lines,
                                            });
                                        }
                                    }
                                    "delete_many" => {
                                        // Determine end_line based on old_content
                                        let end_line = if let Some(old_content) = &change_item.old_content {
                                            change_item.line_number + old_content.lines().count() - 1
                                        } else {
                                            change_item.line_number
                                        };

                                        all_line_changes.push(LineChange::DeleteMany {
                                            start_line: change_item.line_number,
                                            end_line,
                                        });
                                    }
                                    _ => {}
                                }
                            }
                            
                            // Then apply all accumulated changes to the original content
                            file_diff.preview_content = self.apply_changes_to_content(&file_diff.original_content, &all_line_changes)?;
                        }
                    } else {
                        let diff = self.create_file_diff(
                            repository_config,
                            file_path,
                            reason,
                            line_changes,
                        )?;
                        files_map.insert(file_path.to_string(), diff);
                    }
                }
                FileChange::CreateFile { file_path, reason, content, .. } => {
                    let diff = self.create_new_file_diff(file_path, reason, content)?;
                    files_map.insert(file_path.to_string(), diff);
                }
                FileChange::DeleteFile { file_path, reason, .. } => {
                    let diff = self.create_delete_file_diff(
                        repository_config,
                        file_path,
                        reason,
                    )?;
                    files_map.insert(file_path.to_string(), diff);
                }
            };
        }
        
        let session = DiffSession {
            id: session_id.clone(),
            repository_name: repository_config.name.clone(),
            repository_path: repository_config.path.clone(),
            files: files_map.into_iter().map(|(_, file_diff)| file_diff).collect(),
            applied_changes: HashSet::new(),
            status: SessionStatus::Active,
        };

        self.sessions.insert(session_id.clone(), session);
        Ok(session_id)
    }

    pub fn get_session(&self, session_id: &str) -> Option<DiffSession> {
        self.sessions.get(session_id).map(|entry| entry.clone())
    }

    pub fn apply_change(&self, session_id: &str, change_id: &str) -> AicedResult<bool> {
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

    pub fn unapply_change(&self, session_id: &str, change_id: &str) -> AicedResult<bool> {
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

    pub fn complete_session(&self, session_id: &str) -> AicedResult<Vec<String>> {
        if let Some(mut session) = self.sessions.get_mut(session_id) {
            session.status = SessionStatus::Completed;
            Ok(session.applied_changes.iter().cloned().collect())
        } else {
            Ok(Vec::new())
        }
    }

    pub fn cancel_session(&self, session_id: &str) -> AicedResult<()> {
        if let Some(mut session) = self.sessions.get_mut(session_id) {
            session.status = SessionStatus::Cancelled;
        }
        Ok(())
    }

    fn create_file_diff(&self, repository_config: &RepositoryConfig, file_path: &str, reason: &str, line_changes: &[LineChange]) -> AicedResult<FileDiff> {
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

    fn create_new_file_diff(&self, file_path: &str, reason: &str, content: &str) -> AicedResult<FileDiff> {
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

    fn create_delete_file_diff(&self, repository_config: &RepositoryConfig, file_path: &str, reason: &str) -> AicedResult<FileDiff> {
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

    fn line_change_to_change_item(&self, line_change: &LineChange, reason: &str) -> AicedResult<ChangeItem> {
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

    fn apply_changes_to_content(&self, original_content: &str, line_changes: &[LineChange]) -> AicedResult<String> {
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
                LineChange::ReplaceRange { start_line, end_line, new_content, .. } => {
                    if start_line > 0 && end_line <= lines.len() {
                        lines.splice(start_line - 1..end_line, new_content.iter().cloned());
                    }
                }
                LineChange::InsertManyAfter { line_number, new_lines } => {
                    if line_number <= lines.len() {
                        lines.splice(line_number..line_number, new_lines.iter().cloned());
                    }
                }
                LineChange::InsertManyBefore { line_number, new_lines } => {
                    if line_number > 0 && line_number <= lines.len() {
                        lines.splice(line_number - 1..line_number - 1, new_lines.iter().cloned());
                    }
                }
                LineChange::DeleteMany { start_line, end_line } => {
                    if start_line > 0 && end_line <= lines.len() {
                        lines.splice(start_line - 1..end_line, []);
                    }
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