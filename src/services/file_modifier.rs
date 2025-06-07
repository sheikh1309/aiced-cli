use std::fs;
use std::path::Path;
use crate::enums::line_change::LineChange;

pub struct FileModifier;

impl FileModifier {
    pub fn apply_file_modifications(
        repo_path: &str,
        file_path: &str,
        changes: &[LineChange]
    ) -> Result<(), Box<dyn std::error::Error>> {

        let full_path = Path::new(repo_path).join(file_path);

        if !full_path.exists() {
            return Err(format!("File does not exist: {}", full_path.display()).into());
        }

        let content = fs::read_to_string(&full_path)?;
        let original_lines: Vec<String> = content.lines().map(|s| s.to_string()).collect();

        println!("📁 Applying {} changes to {}", changes.len(), file_path);
        println!("📊 Original file has {} lines", original_lines.len());

        // Validate all changes first
        let validated_changes = Self::validate_changes(changes, &original_lines)?;

        // Sort changes by line number (DESCENDING) to avoid index shifting issues
        let mut sorted_changes = validated_changes;
        sorted_changes.sort_by_key(|change| std::cmp::Reverse(Self::get_change_line_number(change)));

        // Create backup with timestamp
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        let backup_path = format!("{}.backup.{}", full_path.display(), timestamp);
        fs::write(&backup_path, &content)?;
        println!("💾 Backup created: {}", backup_path);

        let mut lines = original_lines.clone();

        // Apply changes one by one
        for (i, change) in sorted_changes.iter().enumerate() {
            println!("🔧 Applying change {} of {}", i + 1, sorted_changes.len());

            match change {
                LineChange::Replace { line_number, old_content, new_content } => {
                    Self::apply_replace(&mut lines, *line_number, old_content, new_content)?;
                }
                LineChange::InsertAfter { line_number, new_content } => {
                    Self::apply_insert_after(&mut lines, *line_number, new_content)?;
                }
                LineChange::InsertBefore { line_number, new_content } => {
                    Self::apply_insert_before(&mut lines, *line_number, new_content)?;
                }
                LineChange::Delete { line_number } => {
                    Self::apply_delete(&mut lines, *line_number)?;
                }
                LineChange::ReplaceRange { start_line, end_line, old_content, new_content } => {
                    Self::apply_replace_range(&mut lines, *start_line, *end_line, old_content, new_content)?;
                }
            }

            println!("   ✅ Applied. File now has {} lines", lines.len());
        }

        // Write the modified content back
        let new_content = lines.join("\n");
        fs::write(&full_path, new_content)?;
        println!("✅ File {} successfully modified", file_path);

        Ok(())
    }

    pub fn validate_file_modifications(
        repo_path: &str,
        file_path: &str,
        changes: &[LineChange]
    ) -> Result<(), Box<dyn std::error::Error>> {

        let full_path = Path::new(repo_path).join(file_path);
        let content = fs::read_to_string(&full_path)?;
        let lines: Vec<String> = content.lines().map(|s| s.to_string()).collect();

        Self::validate_changes(changes, &lines)?;
        println!("✅ All {} changes validated for {}", changes.len(), file_path);
        Ok(())
    }

    fn validate_changes(changes: &[LineChange], lines: &[String]) -> Result<Vec<LineChange>, Box<dyn std::error::Error>> {
        let mut validated_changes = Vec::new();

        for (i, change) in changes.iter().enumerate() {
            println!("📊 Validating change {:?}", change);
            match Self::validate_single_change(change, lines) {
                Ok(validated) => {
                    println!("   ✅ Change {} validated", i + 1);
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

                let actual_line = &lines[*line_number - 1]; // Convert to 0-based
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

                // Validate that old_content matches the actual lines
                for (i, expected_line) in old_content.iter().enumerate() {
                    let line_index = (*start_line - 1) + i; // Convert to 0-based
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

    fn apply_replace(
        lines: &mut Vec<String>,
        line_number: usize,
        _old_content: &str,
        new_content: &str
    ) -> Result<(), String> {
        if line_number == 0 || line_number > lines.len() {
            return Err(format!("Line number {} out of range", line_number));
        }

        let index = line_number - 1; // Convert to 0-based
        println!("     🔄 Line {}: '{}' → '{}'", line_number, lines[index].trim(), new_content.trim());
        lines[index] = new_content.to_string();
        Ok(())
    }

    fn apply_insert_after(
        lines: &mut Vec<String>,
        line_number: usize,
        new_content: &str
    ) -> Result<(), String> {
        if line_number > lines.len() {
            return Err(format!("Line number {} out of range", line_number));
        }

        println!("     ➕ After line {}: '{}'", line_number, new_content.trim());
        lines.insert(line_number, new_content.to_string()); // Insert after means insert at line_number index
        Ok(())
    }

    fn apply_insert_before(
        lines: &mut Vec<String>,
        line_number: usize,
        new_content: &str
    ) -> Result<(), String> {
        if line_number == 0 || line_number > lines.len() + 1 {
            return Err(format!("Line number {} out of range", line_number));
        }

        let index = line_number - 1; // Convert to 0-based
        println!("     ➕ Before line {}: '{}'", line_number, new_content.trim());
        lines.insert(index, new_content.to_string());
        Ok(())
    }

    fn apply_delete(lines: &mut Vec<String>, line_number: usize) -> Result<(), String> {
        if line_number == 0 || line_number > lines.len() {
            return Err(format!("Line number {} out of range", line_number));
        }

        let index = line_number - 1; // Convert to 0-based
        println!("     ❌ Delete line {}: '{}'", line_number, lines[index].trim());
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

        let start_index = start_line - 1; // Convert to 0-based
        let end_index = end_line - 1;

        println!("     🔄 Replace lines {}-{} ({} lines) → {} lines",
                 start_line, end_line, end_line - start_line + 1, new_content.len());

        // Remove old lines (in reverse order to maintain indices)
        for _ in start_index..=end_index {
            lines.remove(start_index);
        }

        // Insert new lines
        for (i, line) in new_content.iter().enumerate() {
            lines.insert(start_index + i, line.clone());
        }

        Ok(())
    }

    pub fn create_file(repo_path: &str, file_path: &str, content: &str) -> Result<(), Box<dyn std::error::Error>> {
        let full_path = Path::new(repo_path).join(file_path);

        // Create parent directories if they don't exist
        if let Some(parent) = full_path.parent() {
            fs::create_dir_all(parent)?;
        }

        fs::write(&full_path, content)?;
        println!("✅ Created file: {}", file_path);
        Ok(())
    }

    pub fn delete_file(repo_path: &str, file_path: &str) -> Result<(), Box<dyn std::error::Error>> {
        let full_path = Path::new(repo_path).join(file_path);
        if full_path.exists() {
            fs::remove_file(&full_path)?;
            println!("✅ Deleted file: {}", file_path);
        } else {
            println!("⚠️ File already deleted: {}", file_path);
        }
        Ok(())
    }

    pub fn show_diff(original: &[String], modified: &[String]) {
        println!("\n📋 DIFF PREVIEW:");
        println!("================");

        let max_lines = original.len().max(modified.len());

        for i in 0..max_lines {
            let orig_line = original.get(i).map(|s| s.as_str()).unwrap_or("");
            let mod_line = modified.get(i).map(|s| s.as_str()).unwrap_or("");

            if orig_line != mod_line {
                if orig_line.is_empty() {
                    println!("+ {}: {}", i + 1, mod_line);
                } else if mod_line.is_empty() {
                    println!("- {}: {}", i + 1, orig_line);
                } else {
                    println!("- {}: {}", i + 1, orig_line);
                    println!("+ {}: {}", i + 1, mod_line);
                }
            }
        }
        println!("================\n");
    }
}