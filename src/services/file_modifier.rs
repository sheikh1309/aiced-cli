use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::rc::Rc;
use std::sync::Arc;
use crate::enums::file_change::FileChange;
use crate::enums::line_change::LineChange;
use crate::errors::{AicedError, AicedResult};
use crate::structs::config::repository_config::RepositoryConfig;
use crate::structs::validation_result::ValidationResult;

pub struct FileModifier;

impl FileModifier {

    pub fn validate_changes_batch(repository_config: &RepositoryConfig, file_changes: &[FileChange]) -> AicedResult<ValidationResult> {
        let mut result = ValidationResult::default();

        let mut file_groups: HashMap<String, Vec<&FileChange>> = HashMap::new();
        for change in file_changes {
            file_groups.entry(change.get_file_path().to_string())
                .or_insert_with(Vec::new)
                .push(change);
        }

        for (file_path, changes) in file_groups {
            let full_path = format!("{}/{}", repository_config.path, file_path);
            let file_exists = Path::new(&full_path).exists();

            for change in &changes {
                match change {
                    FileChange::ModifyFile { line_changes, .. } => {
                        if !file_exists {
                            result.errors.push(format!("File does not exist: {}", file_path));
                            continue;
                        }

                        for (i, line_change1) in line_changes.iter().enumerate() {
                            for line_change2 in line_changes.iter().skip(i + 1) {
                                if line_change1.conflicts_with(line_change2) {
                                    result.warnings.push(format!(
                                        "Conflicting line changes in {}: {} and {}",
                                        file_path,
                                        line_change1.get_description(),
                                        line_change2.get_description()
                                    ));
                                }
                            }

                            if let Err(e) = line_change1.validate() {
                                result.errors.push(format!("Invalid line change in {}: {}", file_path, e));
                            }
                        }
                    }
                    FileChange::CreateFile { .. } => {
                        if file_exists {
                            result.warnings.push(format!("File already exists, will be overwritten: {}", file_path));
                        }
                    }
                    FileChange::DeleteFile { .. } => {
                        if !file_exists {
                            result.warnings.push(format!("File does not exist, cannot delete: {}", file_path));
                        }
                    }
                }
            }
        }

        result.is_valid = result.errors.is_empty();

        if !result.warnings.is_empty() {
            log::info!("⚠️ {} validation warnings", result.warnings.len());
        }

        if !result.errors.is_empty() {
            log::error!("❌ {} validation errors", result.errors.len());
        }

        Ok(result)
    }

    pub fn apply_change_with_logging(repository_config: Arc<RepositoryConfig>, file_change: &FileChange) -> AicedResult<()> {
        match file_change {
            FileChange::ModifyFile { file_path, reason: _reason, severity: _severity, category: _category, line_changes } => {
                let references: Rc<Vec<&LineChange>> = Rc::new(line_changes.iter().collect());
                FileModifier::validate_file_modifications(&repository_config.path, file_path, Rc::clone(&references))?;
                FileModifier::apply_file_modifications(&repository_config.path, file_path, Rc::clone(&references))?;
            }
            FileChange::CreateFile { file_path, reason: _reason, severity: _severity, category: _category, content } => {
                FileModifier::create_file(&repository_config.path, file_path, content)?;
            }
            FileChange::DeleteFile { file_path, reason: _reason, severity: _severity, category: _category } => {
                FileModifier::delete_file(&repository_config.path, file_path)?;
            }
        }
        Ok(())
    }

    pub fn apply_file_modifications(repo_path: &str, file_path: &str, changes: Rc<Vec<&LineChange>>) -> AicedResult<()> {
        let str_path = format!("{}/{}", repo_path, file_path).replace("//", "/");
        let full_path = Path::new(&*str_path);

        if !full_path.exists() {
            return Err(AicedError::file_error(
                full_path.to_str().unwrap_or("<invalid_path>"),
                "not_found",
                &format!("File does not exist: {}", full_path.display())
            ));
        }

        let content = fs::read_to_string(&full_path)?;
        let original_lines: Vec<String> = content.lines().map(|s| s.to_string()).collect();

        let validated_changes = Self::validate_changes(Rc::clone(&changes), &original_lines, full_path.display().to_string())?;

        let mut sorted_changes = validated_changes;
        sorted_changes.sort_by_key(|change| Self::get_change_line_number(change));

        let mut lines = original_lines.clone();
        let mut cumulative_offset: i32 = 0;

        for (_change_index, change) in sorted_changes.iter().enumerate() {
            let adjusted_change = Self::adjust_change_line_numbers(change, cumulative_offset);

            let line_offset = match &adjusted_change {
                LineChange::Replace { line_number, old_content, new_content } => {
                    if new_content.contains('\n') {
                        let new_lines: Vec<String> = new_content.lines().map(|s| s.to_string()).collect();
                        let old_lines = vec![old_content.clone()];
                        Self::apply_replace_range(&mut lines, *line_number, *line_number, &old_lines, &new_lines)?;
                        let offset = new_lines.len() as i32 - 1;
                        offset
                    } else {
                        Self::apply_replace(&mut lines, *line_number, old_content, new_content)?;
                        0
                    }
                }
                LineChange::InsertAfter { line_number, new_content } => {
                    if new_content.contains('\n') {
                        let new_lines: Vec<String> = new_content.lines().map(|s| s.to_string()).collect();
                        for (i, line) in new_lines.iter().enumerate() {
                            Self::apply_insert_after(&mut lines, *line_number + i, line)?;
                        }
                        let offset = new_lines.len() as i32;
                        offset
                    } else {
                        Self::apply_insert_after(&mut lines, *line_number, new_content)?;
                        1
                    }
                }
                LineChange::InsertBefore { line_number, new_content } => {
                    if new_content.contains('\n') {
                        let new_lines: Vec<String> = new_content.lines().map(|s| s.to_string()).collect();
                        for (i, line) in new_lines.iter().enumerate() {
                            Self::apply_insert_before(&mut lines, *line_number + i, line)?;
                        }
                        let offset = new_lines.len() as i32;
                        offset
                    } else {
                        Self::apply_insert_before(&mut lines, *line_number, new_content)?;
                        1
                    }
                }
                // Handle multi-line insert actions
                LineChange::InsertManyAfter { line_number, new_lines } => {
                    Self::apply_insert_many_after(&mut lines, *line_number, new_lines)?;
                    let offset = new_lines.len() as i32;
                    offset
                }
                LineChange::InsertManyBefore { line_number, new_lines } => {
                    Self::apply_insert_many_before(&mut lines, *line_number, new_lines)?;
                    let offset = new_lines.len() as i32;
                    offset
                }
                LineChange::Delete { line_number } => {
                    Self::apply_delete(&mut lines, *line_number)?;
                    -1
                }
                // Handle multi-line delete action
                LineChange::DeleteMany { start_line, end_line } => {
                    let deleted_count = end_line - start_line + 1;
                    Self::apply_delete_many(&mut lines, *start_line, *end_line)?;
                    let offset = -(deleted_count as i32);
                    offset
                }
                LineChange::ReplaceRange { start_line, end_line, old_content, new_content } => {
                    let old_line_count = end_line - start_line + 1;
                    let new_line_count = new_content.len();
                    Self::apply_replace_range(&mut lines, *start_line, *end_line, old_content, new_content)?;
                    let offset = new_line_count as i32 - old_line_count as i32;
                    offset
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
            // Handle multi-line insert adjustments
            LineChange::InsertManyAfter { line_number, new_lines } => {
                LineChange::InsertManyAfter {
                    line_number: Self::apply_offset(*line_number, offset),
                    new_lines: new_lines.clone(),
                }
            }
            LineChange::InsertManyBefore { line_number, new_lines } => {
                LineChange::InsertManyBefore {
                    line_number: Self::apply_offset(*line_number, offset),
                    new_lines: new_lines.clone(),
                }
            }
            LineChange::Delete { line_number } => {
                LineChange::Delete {
                    line_number: Self::apply_offset(*line_number, offset),
                }
            }
            // Handle multi-line delete adjustments
            LineChange::DeleteMany { start_line, end_line } => {
                LineChange::DeleteMany {
                    start_line: Self::apply_offset(*start_line, offset),
                    end_line: Self::apply_offset(*end_line, offset),
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

    pub fn validate_file_modifications(repo_path: &str, file_path: &str, changes: Rc<Vec<&LineChange>>) -> AicedResult<()> {
        let full_path = format!("{}/{}", repo_path, file_path).replace("//", "/");
        let content = fs::read_to_string(&full_path)?;
        let original_lines: Vec<String> = content.lines().map(|s| s.to_string()).collect();
        Self::validate_changes(Rc::clone(&changes), &original_lines, full_path)?;
        Self::simulate_changes_application(Rc::clone(&changes), &original_lines)?;
        Ok(())
    }

    fn validate_changes(changes: Rc<Vec<&LineChange>>, lines: &[String], full_path: String) -> AicedResult<Vec<LineChange>> {
        let mut validated_changes = Vec::new();

        for (i, change) in changes.iter().enumerate() {
            match Self::validate_single_change(change, lines) {
                Ok(validated) => {
                    validated_changes.push(validated);
                }
                Err(e) => {
                    log::error!("❌ Change in {} Failed, {}", full_path, e);
                    log::error!("❌ {}", e);
                    return Err(AicedError::validation_error(
                        "line_number",
                        i.to_string().as_str(),
                        "Line Wrong",
                        Some(&format!("Change {} validation failed", i + 1))
                    ));
                }
            }
        }

        Ok(validated_changes)
    }

    fn validate_single_change(change: &LineChange, lines: &[String]) -> AicedResult<LineChange> {
        match change {
            LineChange::Replace { line_number, old_content, .. } => {
                if *line_number == 0 || *line_number > lines.len() {
                    return Err(AicedError::validation_error(
                        "line_number",
                        line_number.to_string().as_str(),
                        "Line Wrong",
                        Some(&format!("Line number {} is out of range (1-{})", line_number, lines.len()))
                    ));
                }

                let actual_line = &lines[*line_number - 1];
                let trimmed_actual = actual_line.trim();
                let trimmed_expected = old_content.trim();

                if trimmed_actual != trimmed_expected {
                    return Err(AicedError::validation_error(
                        "line_content",
                        line_number.to_string().as_str(),
                        "Line Content mismatch",
                        Some(&format!("Line {} content mismatch.\nExpected: '{}'\nActual: '{}'", line_number, trimmed_expected, trimmed_actual))
                    ));
                }

                Ok(change.clone())
            }
            LineChange::InsertAfter { line_number, .. } => {
                if *line_number > lines.len() {
                    return Err(AicedError::validation_error(
                        "line_number",
                        line_number.to_string().as_str(),
                        "Line Wrong",
                        Some(&format!("Line number {} is out of range (1-{})", line_number, lines.len()))
                    ));
                }
                Ok(change.clone())
            }
            LineChange::InsertBefore { line_number, .. } => {
                if *line_number == 0 || *line_number > lines.len() + 1 {
                    return Err(AicedError::validation_error(
                        "line_number",
                        line_number.to_string().as_str(),
                        "Line Wrong",
                        Some(&format!("Line number {} is out of range (1-{})", line_number, lines.len()))
                    ));
                }
                Ok(change.clone())
            }
            // Validate multi-line insert actions
            LineChange::InsertManyAfter { line_number, new_lines } => {
                if *line_number > lines.len() {
                    return Err(AicedError::validation_error(
                        "line_number",
                        line_number.to_string().as_str(),
                        "Line Wrong",
                        Some(&format!("Line number {} is out of range (1-{})", line_number, lines.len()))
                    ));
                }
                if new_lines.is_empty() {
                    return Err(AicedError::validation_error(
                        "new_lines",
                        line_number.to_string().as_str(),
                        "Empty Lines",
                        Some("Cannot insert empty lines collection")
                    ));
                }
                Ok(change.clone())
            }
            LineChange::InsertManyBefore { line_number, new_lines } => {
                if *line_number == 0 || *line_number > lines.len() + 1 {
                    return Err(AicedError::validation_error(
                        "line_number",
                        line_number.to_string().as_str(),
                        "Line Wrong",
                        Some(&format!("Line number {} is out of range (1-{})", line_number, lines.len()))
                    ));
                }
                if new_lines.is_empty() {
                    return Err(AicedError::validation_error(
                        "new_lines",
                        line_number.to_string().as_str(),
                        "Empty Lines",
                        Some("Cannot insert empty lines collection")
                    ));
                }
                Ok(change.clone())
            }
            LineChange::Delete { line_number } => {
                if *line_number == 0 || *line_number > lines.len() {
                    return Err(AicedError::validation_error(
                        "line_number",
                        line_number.to_string().as_str(),
                        "Line Wrong",
                        Some(&format!("Line number {} is out of range (1-{})", line_number, lines.len()))
                    ));
                }
                Ok(change.clone())
            }
            // Validate multi-line delete action
            LineChange::DeleteMany { start_line, end_line } => {
                if *start_line == 0 || *end_line > lines.len() || start_line > end_line {
                    return Err(AicedError::validation_error(
                        "line_range",
                        start_line.to_string().as_str(),
                        "Invalid Range",
                        Some(&format!("Invalid range {}-{} for {} lines", start_line, end_line, lines.len()))
                    ));
                }
                Ok(change.clone())
            }
            LineChange::ReplaceRange { start_line, end_line, old_content, .. } => {
                if *start_line == 0 || *end_line > lines.len() || start_line > end_line {
                    return Err(AicedError::validation_error(
                        "line_number",
                        start_line.to_string().as_str(),
                        "Line Wrong",
                        Some(&format!("Invalid range {}-{} for {} lines", start_line, end_line, lines.len()))
                    ));
                }

                for (i, expected_line) in old_content.iter().enumerate() {
                    let line_index = (*start_line - 1) + i;
                    if line_index >= lines.len() {
                        return Err(AicedError::validation_error(
                            "line_number",
                            start_line.to_string().as_str(),
                            "Line Wrong",
                            Some(&"Range extends beyond file length")
                        ));
                    }

                    let actual_line = &lines[line_index];
                    if actual_line.trim() != expected_line.trim() {
                        return Err(AicedError::validation_error(
                            "line_number",
                            start_line.to_string().as_str(),
                            "Line Wrong",
                            Some(&format!("Line {} in range mismatch.\nExpected: '{}'\nActual: '{}'", line_index + 1, expected_line.trim(), actual_line.trim()))
                        ));
                    }
                }

                Ok(change.clone())
            }
        }
    }

    fn simulate_changes_application(changes: Rc<Vec<&LineChange>>, original_lines: &[String]) -> AicedResult<()> {
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
                        return Err(AicedError::validation_error(
                            "line_number",
                            line_number.to_string().as_str(),
                            "Line Wrong",
                            Some(&format!("After applying previous changes, change {} would reference invalid line {} (file has {} lines)", i + 1, line_number, simulated_line_count))
                        ));
                    }
                }
                LineChange::InsertAfter { line_number, .. } |
                LineChange::InsertManyAfter { line_number, .. } => {
                    if *line_number > simulated_line_count {
                        return Err(AicedError::validation_error(
                            "line_number",
                            line_number.to_string().as_str(),
                            "Line Wrong",
                            Some(&format!("After applying previous changes, change {} would reference invalid line {} (file has {} lines)", i + 1, line_number, simulated_line_count))
                        ));
                    }
                }
                LineChange::InsertBefore { line_number, .. } |
                LineChange::InsertManyBefore { line_number, .. } => {
                    if *line_number == 0 || *line_number > simulated_line_count + 1 {
                        return Err(AicedError::validation_error(
                            "line_number",
                            line_number.to_string().as_str(),
                            "Line Wrong",
                            Some(&format!("After applying previous changes, change {} would insert before invalid line {} (file has {} lines)", i + 1, line_number, simulated_line_count))
                        ));
                    }
                }
                LineChange::DeleteMany { start_line, end_line } => {
                    if *start_line == 0 || *end_line > simulated_line_count || start_line > end_line {
                        return Err(AicedError::validation_error(
                            "line_range",
                            start_line.to_string().as_str(),
                            "Invalid Range",
                            Some(&format!("After applying previous changes, change {} would use invalid range {}-{} (file has {} lines)", i + 1, start_line, end_line, simulated_line_count))
                        ));
                    }
                }
                LineChange::ReplaceRange { start_line, end_line, .. } => {
                    if *start_line == 0 || *end_line > simulated_line_count || start_line > end_line {
                        return Err(AicedError::validation_error(
                            "line_number",
                            start_line.to_string().as_str(),
                            "Line Wrong",
                            Some(&format!("After applying previous changes, change {} would use invalid range {}-{} (file has {} lines)",i + 1, start_line, end_line, simulated_line_count))
                        ));
                    }
                }
            }

            let line_offset = match &adjusted_change {
                LineChange::Replace { new_content, .. } => {
                    if new_content.contains('\n') {
                        new_content.lines().count() as i32 - 1
                    } else {
                        0
                    }
                }
                LineChange::InsertAfter { new_content, .. } | LineChange::InsertBefore { new_content, .. } => {
                    if new_content.contains('\n') {
                        new_content.lines().count() as i32
                    } else {
                        1
                    }
                }
                // Handle multi-line insert simulation
                LineChange::InsertManyAfter { new_lines, .. } | LineChange::InsertManyBefore { new_lines, .. } => {
                    new_lines.len() as i32
                }
                LineChange::Delete { .. } => -1,
                // Handle multi-line delete simulation
                LineChange::DeleteMany { start_line, end_line } => {
                    let deleted_count = end_line - start_line + 1;
                    -(deleted_count as i32)
                }
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

    fn get_change_line_number(change: &LineChange) -> usize {
        match change {
            LineChange::Replace { line_number, .. } => *line_number,
            LineChange::InsertAfter { line_number, .. } => *line_number,
            LineChange::InsertBefore { line_number, .. } => *line_number,
            LineChange::InsertManyAfter { line_number, .. } => *line_number,
            LineChange::InsertManyBefore { line_number, .. } => *line_number,
            LineChange::Delete { line_number } => *line_number,
            LineChange::DeleteMany { start_line, .. } => *start_line,
            LineChange::ReplaceRange { start_line, .. } => *start_line,
        }
    }

    fn apply_replace(lines: &mut Vec<String>, line_number: usize, _old_content: &str, new_content: &str) -> AicedResult<()> {
        if line_number == 0 || line_number > lines.len() {
            return Err(AicedError::validation_error(
                "line_number",
                line_number.to_string().as_str(),
                "Line Wrong",
                Some(&format!("Line number {} out of range", line_number))
            ));
        }

        let index = line_number - 1;
        lines[index] = new_content.to_string();
        Ok(())
    }

    fn apply_insert_after(lines: &mut Vec<String>, line_number: usize, new_content: &str) -> AicedResult<()> {
        if line_number > lines.len() {
            return Err(AicedError::validation_error(
                "line_number",
                line_number.to_string().as_str(),
                "Line Wrong",
                Some(&format!("Line number {} out of range", line_number))
            ));
        }

        lines.insert(line_number, new_content.to_string());
        Ok(())
    }

    fn apply_insert_before(lines: &mut Vec<String>, line_number: usize, new_content: &str) -> AicedResult<()> {
        if line_number == 0 || line_number > lines.len() + 1 {
            return Err(AicedError::validation_error(
                "line_number",
                line_number.to_string().as_str(),
                "Line Wrong",
                Some(&format!("Line number {} out of range", line_number))
            ));
        }

        let index = line_number - 1;
        lines.insert(index, new_content.to_string());
        Ok(())
    }

    fn apply_insert_many_after(lines: &mut Vec<String>, line_number: usize, new_lines: &[String]) -> AicedResult<()> {
        if line_number > lines.len() {
            return Err(AicedError::validation_error(
                "line_number",
                line_number.to_string().as_str(),
                "Line Wrong",
                Some(&format!("Line number {} out of range", line_number))
            ));
        }

        if new_lines.is_empty() {
            return Err(AicedError::validation_error(
                "new_lines",
                line_number.to_string().as_str(),
                "Empty Lines",
                Some("Cannot insert empty lines collection")
            ));
        }

        // Insert lines in order after the specified line
        for (i, line) in new_lines.iter().enumerate() {
            lines.insert(line_number + i + 1, line.clone());
        }

        Ok(())
    }

    fn apply_insert_many_before(lines: &mut Vec<String>, line_number: usize, new_lines: &[String]) -> AicedResult<()> {
        if line_number == 0 || line_number > lines.len() + 1 {
            return Err(AicedError::validation_error(
                "line_number",
                line_number.to_string().as_str(),
                "Line Wrong",
                Some(&format!("Line number {} out of range", line_number))
            ));
        }

        if new_lines.is_empty() {
            return Err(AicedError::validation_error(
                "new_lines",
                line_number.to_string().as_str(),
                "Empty Lines",
                Some("Cannot insert empty lines collection")
            ));
        }

        let index = line_number - 1;

        for (i, line) in new_lines.iter().enumerate() {
            lines.insert(index + i, line.clone());
        }

        Ok(())
    }

    fn apply_delete(lines: &mut Vec<String>, line_number: usize) -> AicedResult<()> {
        if line_number == 0 || line_number > lines.len() {
            return Err(AicedError::validation_error(
                "line_number",
                line_number.to_string().as_str(),
                "Line Wrong",
                Some(&format!("Line number {} out of range", line_number))
            ));
        }

        let index = line_number - 1;
        lines.remove(index);
        Ok(())
    }

    fn apply_delete_many(lines: &mut Vec<String>, start_line: usize, end_line: usize) -> AicedResult<()> {
        if start_line == 0 || end_line > lines.len() || start_line > end_line {
            return Err(AicedError::validation_error(
                "line_range",
                start_line.to_string().as_str(),
                "Invalid Range",
                Some(&format!("Invalid range {}-{} for {} lines", start_line, end_line, lines.len()))
            ));
        }

        let start_index = start_line - 1;
        let delete_count = end_line - start_line + 1;

        // Remove lines from start_index, delete_count times
        for _ in 0..delete_count {
            lines.remove(start_index);
        }

        Ok(())
    }

    fn apply_single_change(lines: &mut Vec<String>, change: &LineChange) -> AicedResult<i32> {
        match change {
            LineChange::Replace { line_number, old_content, new_content } => {
                if new_content.contains('\n') {
                    let new_lines: Vec<String> = new_content.lines().map(|s| s.to_string()).collect();
                    let old_lines = vec![old_content.clone()];
                    Self::apply_replace_range(lines, *line_number, *line_number, &old_lines, &new_lines)?;
                    Ok(new_lines.len() as i32 - 1)
                } else {
                    Self::apply_replace(lines, *line_number, old_content, new_content)?;
                    Ok(0)
                }
            }
            LineChange::InsertAfter { line_number, new_content } => {
                if new_content.contains('\n') {
                    let new_lines: Vec<String> = new_content.lines().map(|s| s.to_string()).collect();
                    for (i, line) in new_lines.iter().enumerate() {
                        Self::apply_insert_after(lines, *line_number + i, line)?;
                    }
                    Ok(new_lines.len() as i32)
                } else {
                    Self::apply_insert_after(lines, *line_number, new_content)?;
                    Ok(1)
                }
            }
            LineChange::InsertBefore { line_number, new_content } => {
                if new_content.contains('\n') {
                    let new_lines: Vec<String> = new_content.lines().map(|s| s.to_string()).collect();
                    for (i, line) in new_lines.iter().enumerate() {
                        Self::apply_insert_before(lines, *line_number + i, line)?;
                    }
                    Ok(new_lines.len() as i32)
                } else {
                    Self::apply_insert_before(lines, *line_number, new_content)?;
                    Ok(1)
                }
            }
            LineChange::InsertManyAfter { line_number, new_lines } => {
                Self::apply_insert_many_after(lines, *line_number, new_lines)?;
                Ok(new_lines.len() as i32)
            }
            LineChange::InsertManyBefore { line_number, new_lines } => {
                Self::apply_insert_many_before(lines, *line_number, new_lines)?;
                Ok(new_lines.len() as i32)
            }
            LineChange::Delete { line_number } => {
                Self::apply_delete(lines, *line_number)?;
                Ok(-1)
            }
            LineChange::DeleteMany { start_line, end_line } => {
                let deleted_count = end_line - start_line + 1;
                Self::apply_delete_many(lines, *start_line, *end_line)?;
                Ok(-(deleted_count as i32))
            }
            LineChange::ReplaceRange { start_line, end_line, old_content, new_content } => {
                let old_line_count = end_line - start_line + 1;
                let new_line_count = new_content.len();
                Self::apply_replace_range(lines, *start_line, *end_line, old_content, new_content)?;
                Ok(new_line_count as i32 - old_line_count as i32)
            }
        }
    }

    fn apply_replace_range(lines: &mut Vec<String>, start_line: usize, end_line: usize, _old_content: &[String], new_content: &[String]) -> AicedResult<()> {
        if start_line == 0 || end_line > lines.len() || start_line > end_line {
            return Err(AicedError::validation_error(
                "replace_range",
                start_line.to_string().as_str(),
                "Line Wrong",
                Some(&format!("Invalid range {}-{}", start_line, end_line))
            ));
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

    fn create_file(repo_path: &str, file_path: &str, content: &str) -> AicedResult<()> {
        let full_path = format!("{}/{}", repo_path, file_path).replace("//", "/");
        let path = Path::new(&full_path);

        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }

        fs::write(path, content)?;
        Ok(())
    }

    fn delete_file(repo_path: &str, file_path: &str) -> AicedResult<()> {
        let full_path = format!("{}/{}", repo_path, file_path).replace("//", "/");
        let path = Path::new(&full_path);

        if path.exists() {
            fs::remove_file(path)?;
        } else {
            return Err(AicedError::validation_error(
                "file_exists",
                &full_path,
                "Line Wrong",
                Some("File does not exist")
            ));
        }

        Ok(())
    }

    pub fn apply_changes_grouped_by_file(repository_config: Arc<RepositoryConfig>, file_changes: Vec<&FileChange>) -> AicedResult<usize> {
        let mut applied_count = 0;

        let mut file_groups: HashMap<String, Vec<&FileChange>> = HashMap::new();
        for change in file_changes {
            file_groups.entry(change.get_file_path().to_string())
                .or_insert_with(Vec::new)
                .push(change);
        }

        for (file_path, changes) in file_groups {

            match Self::apply_changes_to_single_file(Arc::clone(&repository_config), &file_path, &changes) {
                Ok(count) => {
                    applied_count += count;
                }
                Err(e) => {
                    log::error!("❌ Failed to apply changes to {}: {}", file_path, e);
                }
            }
        }

        Ok(applied_count)
    }

    pub fn apply_file_modifications_with_smart_validation(repo_path: &str, file_path: &str, changes: Rc<Vec<&LineChange>>) -> AicedResult<()> {
        let str_path = format!("{}/{}", repo_path, file_path).replace("//", "/");
        let full_path = Path::new(&*str_path);

        if !full_path.exists() {
            return Err(AicedError::file_error(
                full_path.to_str().unwrap_or("<invalid_path>"),
                "not_found",
                &format!("File does not exist: {}", full_path.display())
            ));
        }

        let content = fs::read_to_string(&full_path)?;
        let original_lines: Vec<String> = content.lines().map(|s| s.to_string()).collect();

        let mut sorted_changes: Vec<LineChange> = changes.iter().map(|&c| c.clone()).collect();
        sorted_changes.sort_by_key(|change| Self::get_change_line_number(change));

        let mut lines = original_lines.clone();
        let mut line_offset_map: HashMap<usize, i32> = HashMap::new();

        for (change_index, change) in sorted_changes.iter().enumerate() {
            let original_line_number = Self::get_change_line_number(change);
            let cumulative_offset = Self::calculate_cumulative_offset(&line_offset_map, original_line_number);

            let adjusted_change = Self::adjust_change_line_numbers(change, cumulative_offset);

            match Self::validate_single_change_against_current_state(&adjusted_change, &lines) {
                Ok(_) => {
                }
                Err(exact_error) => {
                    log::warn!("   ⚠️ Exact validation failed: {}", exact_error);

                    // Try smart/fuzzy validation
                    match Self::smart_validate_and_adjust_change(&adjusted_change, &lines) {
                        Ok(smart_adjusted_change) => {
                            let line_offset = Self::apply_single_change(&mut lines, &smart_adjusted_change)?;
                            line_offset_map.insert(original_line_number, line_offset);
                            continue;
                        }
                        Err(smart_error) => {
                            log::error!("❌ Both exact and smart validation failed");
                            log::error!("   Exact error: {}", exact_error);
                            log::error!("   Smart error: {}", smart_error);
                            return Err(AicedError::validation_error(
                                "line_number",
                                "0",
                                "Line Wrong",
                                Some(&format!("Change {} validation failed", change_index + 1))
                            ));
                        }
                    }
                }
            }

            let line_offset = Self::apply_single_change(&mut lines, &adjusted_change)?;
            line_offset_map.insert(original_line_number, line_offset);
        }

        let new_content = lines.join("\n");
        fs::write(&full_path, new_content)?;

        Ok(())
    }

    fn validate_single_change_against_current_state(change: &LineChange, current_lines: &[String]) -> AicedResult<()> {
        match change {
            LineChange::Replace { line_number, old_content, .. } => {
                if *line_number == 0 || *line_number > current_lines.len() {
                    return Err(AicedError::validation_error(
                        "line_number",
                        &line_number.to_string(),
                        "Line Wrong",
                        Some(&format!("Line {} is out of bounds (file has {} lines)", line_number, current_lines.len()))
                    ));
                }

                let actual_content = &current_lines[*line_number - 1];
                if actual_content.trim() != old_content.trim() {
                    return Err(AicedError::validation_error(
                        "line_number",
                        &line_number.to_string(),
                        "Line Wrong",
                        Some(&format!("Line {} content mismatch.\n    Expected: '{}'\n    Actual: '{}'",
                                      line_number, old_content, actual_content))
                    ));
                }
            }
            LineChange::ReplaceRange { start_line, end_line, old_content, .. } => {
                if *start_line == 0 || *end_line > current_lines.len() || start_line > end_line {
                    return Err(AicedError::validation_error(
                        "line_number",
                        &start_line.to_string(),
                        "Line Wrong",
                        Some(&format!("Line range {}-{} is invalid (file has {} lines)", start_line, end_line, current_lines.len()))
                    ));
                }

                for (i, expected_line) in old_content.iter().enumerate() {
                    let line_index = start_line - 1 + i;
                    if line_index >= current_lines.len() {
                        return Err(AicedError::validation_error(
                            "line_number",
                            &(line_index + 1).to_string(),
                            "Line Wrong",
                            Some(&format!("Line {} is out of bounds", line_index + 1))
                        ));
                    }

                    let actual_line = &current_lines[line_index];
                    if actual_line.trim() != expected_line.trim() {
                        return Err(AicedError::validation_error(
                            "line_number",
                            &(line_index + 1).to_string(),
                            "Line Wrong",
                            Some(&format!("Line {} content mismatch.\n    Expected: '{}'\n    Actual: '{}'",
                                          line_index + 1, expected_line, actual_line))
                        ));
                    }
                }
            }
            LineChange::Delete { line_number } => {
                if *line_number == 0 || *line_number > current_lines.len() {
                    return Err(AicedError::validation_error(
                        "line_number",
                        &line_number.to_string(),
                        "Line Wrong",
                        Some(&format!("Line {} is out of bounds (file has {} lines)", line_number, current_lines.len()))
                    ));
                }
            }
            LineChange::DeleteMany { start_line, end_line } => {
                if *start_line == 0 || *end_line > current_lines.len() || start_line > end_line {
                    return Err(AicedError::validation_error(
                        "line_number",
                        &start_line.to_string(),
                        "Line Wrong",
                        Some(&format!("Line range {}-{} is invalid (file has {} lines)", start_line, end_line, current_lines.len()))
                    ));
                }
            }
            LineChange::InsertAfter { line_number, .. } |
            LineChange::InsertBefore { line_number, .. } |
            LineChange::InsertManyAfter { line_number, .. } |
            LineChange::InsertManyBefore { line_number, .. } => {
                if *line_number > current_lines.len() {
                    return Err(AicedError::validation_error(
                        "line_number",
                        &line_number.to_string(),
                        "Line Wrong",
                        Some(&format!("Line {} is out of bounds (file has {} lines)", line_number, current_lines.len()))
                    ));
                }
            }
        }
        Ok(())
    }

    fn calculate_cumulative_offset(offset_map: &HashMap<usize, i32>, target_line: usize) -> i32 {
        let mut cumulative_offset = 0;

        for (&line_num, &offset) in offset_map.iter() {
            if line_num < target_line {
                cumulative_offset += offset;
            }
        }

        cumulative_offset
    }

    fn apply_changes_to_single_file(repository_config: Arc<RepositoryConfig>, file_path: &str, changes: &[&FileChange]) -> AicedResult<usize> {
        let mut applied_count = 0;

        let mut modify_changes = Vec::new();
        let mut other_changes = Vec::new();

        for change in changes {
            match change {
                FileChange::ModifyFile { line_changes, .. } => {
                    modify_changes.extend(line_changes);
                }
                _ => other_changes.push(*change),
            }
        }

        for change in other_changes {
            match Self::apply_change_with_logging(Arc::clone(&repository_config), change) {
                Ok(_) => applied_count += 1,
                Err(e) => {
                    log::error!("❌ Failed to apply {}: {}", change.get_file_path(), e);
                    return Err(e);
                }
            }
        }

        if !modify_changes.is_empty() {
            let changes_refs: Rc<Vec<&LineChange>> = Rc::new(modify_changes.clone());

            match Self::apply_file_modifications_with_smart_validation(&repository_config.path, file_path, changes_refs) {
                Ok(_) => {
                    applied_count += modify_changes.len();
                }
                Err(e) => {
                    log::error!("❌ Failed to apply line changes to {}: {}", file_path, e);
                    return Err(e);
                }
            }
        }

        Ok(applied_count)
    }

    fn smart_validate_and_adjust_change(change: &LineChange, current_lines: &[String]) -> AicedResult<LineChange> {
        match change {
            LineChange::Replace { line_number, old_content, new_content } => {
                for offset in 0..10 {
                    for direction in [0i32, 1i32, -1i32] {
                        let line_offset = direction * offset as i32;
                        let new_line_number = (*line_number as i32 + line_offset).max(1) as usize;

                        if new_line_number > 0 && new_line_number <= current_lines.len() {
                            let actual_line = &current_lines[new_line_number - 1];
                            if actual_line.trim() == old_content.trim() {
                                return Ok(LineChange::Replace {
                                    line_number: new_line_number,
                                    old_content: old_content.clone(),
                                    new_content: new_content.clone(),
                                });
                            }
                        }
                    }
                }

                Err(AicedError::validation_error(
                    "smart_validation",
                    "0",
                    "ContentNotFound",
                    Some(&format!("Could not find matching content '{}' near line {}", old_content.trim(), line_number))
                ))
            }

            LineChange::ReplaceRange { start_line, end_line, old_content, new_content } => {
                let original_range_size = end_line - start_line + 1;

                // Validate that old_content matches the expected range size
                if old_content.len() != original_range_size {
                    return Err(AicedError::validation_error(
                        "smart_validation",
                        "0",
                        "RangeMismatch",
                        Some(&format!("Old content has {} lines but range is {} lines",
                                      old_content.len(), original_range_size))
                    ));
                }

                for start_offset in 0..20 {
                    for direction in [0i32, 1i32, -1i32] {
                        let offset = direction * start_offset as i32;
                        let new_start = (*start_line as i32 + offset).max(1) as usize;
                        let new_end = new_start + (end_line - start_line);

                        if new_end <= current_lines.len() && new_start >= 1 {
                            let mut all_match = true;

                            for (i, expected_line) in old_content.iter().enumerate() {
                                let line_index = new_start - 1 + i;
                                if line_index >= current_lines.len() {
                                    all_match = false;
                                    break;
                                }

                                let actual_line = &current_lines[line_index];
                                if actual_line.trim() != expected_line.trim() {
                                    all_match = false;
                                    break;
                                }
                            }

                            if all_match {
                                return Ok(LineChange::ReplaceRange {
                                    start_line: new_start,
                                    end_line: new_end,
                                    old_content: old_content.clone(),
                                    new_content: new_content.clone(),
                                });
                            }
                        }
                    }
                }

                Err(AicedError::validation_error(
                    "smart_validation",
                    "0",
                    "ContentNotFound",
                    Some(&format!("Could not find matching range content for lines {}-{}", start_line, end_line))
                ))
            }

            LineChange::Delete { line_number } => {
                // For delete operations, we just need to verify the line exists
                // We don't need to match specific content
                if *line_number > 0 && *line_number <= current_lines.len() {
                    Ok(change.clone())
                } else {
                    let search_range = 5;
                    for offset in 1..=search_range {
                        for direction in [1i32, -1i32] {
                            let new_line_number = (*line_number as i32 + direction * offset).max(1) as usize;
                            if new_line_number > 0 && new_line_number <= current_lines.len() {
                                return Ok(LineChange::Delete { line_number: new_line_number });
                            }
                        }
                    }

                    Err(AicedError::validation_error(
                        "smart_validation",
                        "0",
                        "LineNotFound",
                        Some(&format!("Could not find valid line near {}", line_number))
                    ))
                }
            }

            LineChange::DeleteMany { start_line, end_line } => {
                // Check if the range is valid, adjust if needed
                if *start_line > 0 && *end_line <= current_lines.len() && start_line <= end_line {
                    Ok(change.clone())
                } else {
                    // Try to adjust the range to fit within file bounds
                    let adjusted_start = (*start_line).max(1);
                    let adjusted_end = (*end_line).min(current_lines.len());

                    if adjusted_start <= adjusted_end {
                        Ok(LineChange::DeleteMany {
                            start_line: adjusted_start,
                            end_line: adjusted_end,
                        })
                    } else {
                        Err(AicedError::validation_error(
                            "smart_validation",
                            "0",
                            "InvalidRange",
                            Some(&format!("Could not adjust range {}-{} to valid bounds", start_line, end_line))
                        ))
                    }
                }
            }

            LineChange::InsertAfter { line_number, new_content } => {
                // For insert operations, adjust line number to valid range
                let max_line = current_lines.len();

                if *line_number <= max_line {
                    Ok(change.clone())
                } else {
                    Ok(LineChange::InsertAfter {
                        line_number: max_line,
                        new_content: new_content.clone(),
                    })
                }
            }

            LineChange::InsertBefore { line_number, new_content } => {
                // For insert before, line_number should be between 1 and len+1
                let max_line = current_lines.len() + 1;

                if *line_number >= 1 && *line_number <= max_line {
                    Ok(change.clone())
                } else if *line_number > max_line {
                    // Insert at the end of file
                    Ok(LineChange::InsertBefore {
                        line_number: max_line,
                        new_content: new_content.clone(),
                    })
                } else {
                    Ok(LineChange::InsertBefore {
                        line_number: 1,
                        new_content: new_content.clone(),
                    })
                }
            }

            LineChange::InsertManyAfter { line_number, new_lines } => {
                let max_line = current_lines.len();

                if *line_number <= max_line {
                    Ok(change.clone())
                } else {
                    Ok(LineChange::InsertManyAfter {
                        line_number: max_line,
                        new_lines: new_lines.clone(),
                    })
                }
            }

            LineChange::InsertManyBefore { line_number, new_lines } => {
                let max_line = current_lines.len() + 1;

                if *line_number >= 1 && *line_number <= max_line {
                    Ok(change.clone())
                } else if *line_number > max_line {
                    Ok(LineChange::InsertManyBefore {
                        line_number: max_line,
                        new_lines: new_lines.clone(),
                    })
                } else {
                    Ok(LineChange::InsertManyBefore {
                        line_number: 1,
                        new_lines: new_lines.clone(),
                    })
                }
            }
        }
    }

}