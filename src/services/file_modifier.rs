use std::fs;
use std::path::Path;
use std::sync::Arc;
use crate::enums::file_change::FileChange;
use crate::enums::line_change::LineChange;
use crate::logger::file_change_logger::FileChangeLogger;
use crate::structs::config::repository_config::RepositoryConfig;

pub struct FileModifier;

impl FileModifier {

    pub fn apply_change(repository_config: Arc<RepositoryConfig>, file_change: &FileChange) -> Result<(), Box<dyn std::error::Error>> {
        match file_change {
            FileChange::ModifyFile { file_path, reason: _, severity: _, line_changes } => {
                FileModifier::validate_file_modifications(&repository_config.path, file_path, line_changes)?;
                FileModifier::apply_file_modifications(&repository_config.path, file_path, line_changes)?;
            }
            FileChange::CreateFile { file_path, reason: _, severity: _, content } => {
                FileChangeLogger::print_new_file_preview(file_path, content);
                FileModifier::create_file(&repository_config.path, file_path, content)?;
            }
            FileChange::DeleteFile { file_path, reason: _, severity: _ } => {
                FileModifier::delete_file(&repository_config.path, file_path)?;
            }
        }
        Ok(())
    }
    
    pub fn apply_file_modifications(repo_path: &str, file_path: &str, changes: &[LineChange]) -> Result<(), Box<dyn std::error::Error>> {
        let str_path = format!("{}/{}", repo_path, file_path).replace("//", "/");
        let full_path = Path::new(&*str_path);

        if !full_path.exists() {
            return Err(format!("File does not exist: {}", full_path.display()).into());
        }

        let content = fs::read_to_string(&full_path)?;
        let original_lines: Vec<String> = content.lines().map(|s| s.to_string()).collect();

        let validated_changes = Self::validate_changes(changes, &original_lines)?;

        let mut sorted_changes = validated_changes;
        sorted_changes.sort_by_key(|change| Self::get_change_line_number(change));

        let mut lines = original_lines.clone();
        let mut cumulative_offset: i32 = 0; // Track total line offset

        for (_, change) in sorted_changes.iter().enumerate() {

            let adjusted_change = Self::adjust_change_line_numbers(change, cumulative_offset);

            let line_offset = match &adjusted_change {
                LineChange::Replace { line_number, old_content, new_content } => {
                    // todo - some times the new content can have multi line
                    Self::apply_replace(&mut lines, *line_number, old_content, new_content)?;
                    0 // Replace doesn't change line count
                }
                LineChange::InsertAfter { line_number, new_content } => {
                    Self::apply_insert_after(&mut lines, *line_number, new_content)?;
                    1 // Added 1 line
                }
                LineChange::InsertBefore { line_number, new_content } => {
                    Self::apply_insert_before(&mut lines, *line_number, new_content)?;
                    1 // Added 1 line
                }
                LineChange::Delete { line_number } => {
                    Self::apply_delete(&mut lines, *line_number)?;
                    -1 // Removed 1 line
                }
                LineChange::ReplaceRange { start_line, end_line, old_content, new_content } => {
                    let old_line_count = end_line - start_line + 1;
                    let new_line_count = new_content.len();
                    Self::apply_replace_range(&mut lines, *start_line, *end_line, old_content, new_content)?;
                    new_line_count as i32 - old_line_count as i32 // Net change in lines
                }
            };

            cumulative_offset += line_offset;
        }

        let new_content = lines.join("\n");
        fs::write(&full_path, new_content)?;

        Ok(())
    }

    fn adjust_change_line_numbers(change: &LineChange, offset: i32) -> LineChange {
        match change {
            LineChange::Replace { line_number, old_content, new_content } => {
                LineChange::Replace {
                    line_number: Self::apply_offset(*line_number, offset),
                    old_content: old_content.clone(),
                    new_content: new_content.clone(),
                }
            }
            LineChange::InsertAfter { line_number, new_content } => {
                LineChange::InsertAfter {
                    line_number: Self::apply_offset(*line_number, offset),
                    new_content: new_content.clone(),
                }
            }
            LineChange::InsertBefore { line_number, new_content } => {
                LineChange::InsertBefore {
                    line_number: Self::apply_offset(*line_number, offset),
                    new_content: new_content.clone(),
                }
            }
            LineChange::Delete { line_number } => {
                LineChange::Delete {
                    line_number: Self::apply_offset(*line_number, offset),
                }
            }
            LineChange::ReplaceRange { start_line, end_line, old_content, new_content } => {
                LineChange::ReplaceRange {
                    start_line: Self::apply_offset(*start_line, offset),
                    end_line: Self::apply_offset(*end_line, offset),
                    old_content: old_content.clone(),
                    new_content: new_content.clone(),
                }
            }
        }
    }

    fn apply_offset(line_number: usize, offset: i32) -> usize {
        if offset < 0 {
            let abs_offset = (-offset) as usize;
            if abs_offset >= line_number {
                1 // Minimum line number is 1
            } else {
                line_number - abs_offset
            }
        } else {
            line_number + offset as usize
        }
    }

    pub fn validate_file_modifications(repo_path: &str, file_path: &str, changes: &[LineChange]) -> Result<(), Box<dyn std::error::Error>> {
        let full_path = format!("{}/{}", repo_path, file_path).replace("//", "/");
        let content = fs::read_to_string(&full_path)?;
        let original_lines: Vec<String> = content.lines().map(|s| s.to_string()).collect();

        Self::validate_changes(changes, &original_lines)?;

        Self::simulate_changes_application(changes, &original_lines)?;

        Ok(())
    }

    fn simulate_changes_application(changes: &[LineChange], original_lines: &[String]) -> Result<(), Box<dyn std::error::Error>> {

        let mut sorted_changes = changes.to_vec();
        sorted_changes.sort_by_key(|change| Self::get_change_line_number(change));

        let mut simulated_line_count = original_lines.len();
        let mut cumulative_offset: i32 = 0;

        for (i, change) in sorted_changes.iter().enumerate() {
            let adjusted_change = Self::adjust_change_line_numbers(change, cumulative_offset);

            match &adjusted_change {
                LineChange::Replace { line_number, .. } |
                LineChange::Delete { line_number } => {
                    if *line_number == 0 || *line_number > simulated_line_count {
                        return Err(format!(
                            "After applying previous changes, change {} would reference invalid line {} (file has {} lines)",
                            i + 1, line_number, simulated_line_count
                        ).into());
                    }
                }
                LineChange::InsertAfter { line_number, .. } => {
                    if *line_number > simulated_line_count {
                        return Err(format!(
                            "After applying previous changes, change {} would insert after invalid line {} (file has {} lines)",
                            i + 1, line_number, simulated_line_count
                        ).into());
                    }
                }
                LineChange::InsertBefore { line_number, .. } => {
                    if *line_number == 0 || *line_number > simulated_line_count + 1 {
                        return Err(format!(
                            "After applying previous changes, change {} would insert before invalid line {} (file has {} lines)",
                            i + 1, line_number, simulated_line_count
                        ).into());
                    }
                }
                LineChange::ReplaceRange { start_line, end_line, .. } => {
                    if *start_line == 0 || *end_line > simulated_line_count || start_line > end_line {
                        return Err(format!(
                            "After applying previous changes, change {} would use invalid range {}-{} (file has {} lines)",
                            i + 1, start_line, end_line, simulated_line_count
                        ).into());
                    }
                }
            }

            let line_offset = match &adjusted_change {
                LineChange::Replace { .. } => 0,
                LineChange::InsertAfter { .. } | LineChange::InsertBefore { .. } => 1,
                LineChange::Delete { .. } => -1,
                LineChange::ReplaceRange { start_line, end_line, new_content, .. } => {
                    let old_line_count = end_line - start_line + 1;
                    let new_line_count = new_content.len();
                    new_line_count as i32 - old_line_count as i32
                }
            };

            cumulative_offset += line_offset;
            simulated_line_count = (simulated_line_count as i32 + line_offset) as usize;
        }

        Ok(())
    }

    fn validate_changes(changes: &[LineChange], lines: &[String]) -> Result<Vec<LineChange>, Box<dyn std::error::Error>> {
        let mut validated_changes = Vec::new();

        for (i, change) in changes.iter().enumerate() {
            match Self::validate_single_change(change, lines) {
                Ok(validated) => {
                    validated_changes.push(validated);
                }
                Err(e) => {
                    return Err(format!("❌ Change {} validation failed: {}", i + 1, e).into());
                }
            }
        }

        Ok(validated_changes)
    }

    fn validate_single_change(change: &LineChange, lines: &[String]) -> Result<LineChange, String> {
        match change {
            LineChange::Replace { line_number, old_content, new_content: _new_content } => {
                if *line_number == 0 || *line_number > lines.len() {
                    return Err(format!("Line number {} is out of range (1-{})", line_number, lines.len()));
                }

                let actual_line = &lines[*line_number - 1];
                let trimmed_actual = actual_line.trim();
                let trimmed_expected = old_content.trim();

                if trimmed_actual != trimmed_expected {
                    return Err(format!(
                        "Line {} content mismatch.\nExpected: '{}'\nActual: '{}'",
                        line_number, trimmed_expected, trimmed_actual
                    ));
                }

                Ok(change.clone())
            }
            LineChange::InsertAfter { line_number, .. } => {
                if *line_number > lines.len() {
                    return Err(format!("Line number {} is out of range (0-{})", line_number, lines.len()));
                }
                Ok(change.clone())
            }
            LineChange::InsertBefore { line_number, .. } => {
                if *line_number == 0 || *line_number > lines.len() + 1 {
                    return Err(format!("Line number {} is out of range (1-{})", line_number, lines.len() + 1));
                }
                Ok(change.clone())
            }
            LineChange::Delete { line_number } => {
                if *line_number == 0 || *line_number > lines.len() {
                    return Err(format!("Line number {} is out of range (1-{})", line_number, lines.len()));
                }
                Ok(change.clone())
            }
            LineChange::ReplaceRange { start_line, end_line, old_content, .. } => {
                if *start_line == 0 || *end_line > lines.len() || start_line > end_line {
                    return Err(format!("Invalid range {}-{} for {} lines", start_line, end_line, lines.len()));
                }

                for (i, expected_line) in old_content.iter().enumerate() {
                    let line_index = (*start_line - 1) + i;
                    if line_index >= lines.len() {
                        return Err(format!("Range extends beyond file length"));
                    }

                    let actual_line = &lines[line_index];
                    if actual_line.trim() != expected_line.trim() {
                        return Err(format!(
                            "Line {} in range mismatch.\nExpected: '{}'\nActual: '{}'",
                            line_index + 1, expected_line.trim(), actual_line.trim()
                        ));
                    }
                }

                Ok(change.clone())
            }
        }
    }

    fn get_change_line_number(change: &LineChange) -> usize {
        match change {
            LineChange::Replace { line_number, .. } => *line_number,
            LineChange::InsertAfter { line_number, .. } => *line_number,
            LineChange::InsertBefore { line_number, .. } => *line_number,
            LineChange::Delete { line_number } => *line_number,
            LineChange::ReplaceRange { start_line, .. } => *start_line,
        }
    }

    fn apply_replace(lines: &mut Vec<String>, line_number: usize, _old_content: &str, new_content: &str) -> Result<(), String> {
        if line_number == 0 || line_number > lines.len() {
            return Err(format!("Line number {} out of range", line_number));
        }

        let index = line_number - 1;
        lines[index] = new_content.to_string();
        Ok(())
    }

    fn apply_insert_after(lines: &mut Vec<String>, line_number: usize, new_content: &str) -> Result<(), String> {
        if line_number > lines.len() {
            return Err(format!("Line number {} out of range", line_number));
        }

        lines.insert(line_number, new_content.to_string());
        Ok(())
    }

    fn apply_insert_before(lines: &mut Vec<String>, line_number: usize, new_content: &str) -> Result<(), String> {
        if line_number == 0 || line_number > lines.len() + 1 {
            return Err(format!("Line number {} out of range", line_number));
        }

        let index = line_number - 1;
        lines.insert(index, new_content.to_string());
        Ok(())
    }

    fn apply_delete(lines: &mut Vec<String>, line_number: usize) -> Result<(), String> {
        if line_number == 0 || line_number > lines.len() {
            return Err(format!("Line number {} out of range", line_number));
        }

        let index = line_number - 1;
        lines.remove(index);
        Ok(())
    }

    fn apply_replace_range(
        lines: &mut Vec<String>,
        start_line: usize,
        end_line: usize,
        _old_content: &[String],
        new_content: &[String]
    ) -> Result<(), String> {
        if start_line == 0 || end_line > lines.len() || start_line > end_line {
            return Err(format!("Invalid range {}-{}", start_line, end_line));
        }

        let start_index = start_line - 1;
        let end_index = end_line - 1;

        for _ in start_index..=end_index {
            lines.remove(start_index);
        }

        for (i, line) in new_content.iter().enumerate() {
            lines.insert(start_index + i, line.clone());
        }

        Ok(())
    }

    pub fn create_file(repo_path: &String, file_path: &str, content: &str) -> Result<(), Box<dyn std::error::Error>> {
        let full_path = Path::new(repo_path).join(file_path);

        if let Some(parent) = full_path.parent() {
            fs::create_dir_all(parent)?;
        }

        fs::write(&full_path, content)?;
        Ok(())
    }

    pub fn delete_file(repo_path: &str, file_path: &str) -> Result<(), Box<dyn std::error::Error>> {
        let full_path = Path::new(repo_path).join(file_path);
        if full_path.exists() {
            fs::remove_file(&full_path)?;
        }
        Ok(())
    }

}