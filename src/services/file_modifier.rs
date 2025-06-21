use std::fs;
use std::path::Path;
use std::rc::Rc;
use std::sync::Arc;
use crate::enums::file_change::FileChange;
use crate::enums::line_change::LineChange;
use crate::errors::{AilyzerError, AilyzerResult};
use crate::logger::file_change_logger::FileChangeLogger;
use crate::structs::apply_result::ApplyResult;
use crate::structs::change_statistics::ChangeStatistics;
use crate::structs::config::repository_config::RepositoryConfig;
use crate::structs::validation_result::ValidationResult;

pub struct FileModifier;

impl FileModifier {

    pub fn apply_change(repository_config: Arc<RepositoryConfig>, file_change: &FileChange) -> AilyzerResult<()> {
        match file_change {
            FileChange::ModifyFile { file_path, reason: _, severity: _, category: _, line_changes } => {
                let references: Rc<Vec<&LineChange>> = Rc::new(line_changes.iter().collect());
                FileModifier::validate_file_modifications(&repository_config.path, file_path, Rc::clone(&references))?;
                FileModifier::apply_file_modifications(&repository_config.path, file_path, Rc::clone(&references))?;
            }
            FileChange::CreateFile { file_path, reason: _, severity: _, category: _, content } => {
                FileChangeLogger::print_new_file_preview(file_path, content);
                FileModifier::create_file(&repository_config.path, file_path, content)?;
            }
            FileChange::DeleteFile { file_path, reason: _, severity: _, category: _ } => {
                FileModifier::delete_file(&repository_config.path, file_path)?;
            }
        }
        Ok(())
    }

    pub fn apply_change_with_logging(repository_config: Arc<RepositoryConfig>, file_change: &FileChange) -> AilyzerResult<()> {
        let category = file_change.get_category().unwrap_or("UNKNOWN");
        let severity = file_change.get_severity();

        println!("🔧 Applying {} change ({}): {}", category, severity, file_change.get_file_path());

        match file_change {
            FileChange::ModifyFile { file_path, reason, severity, category, line_changes } => {
                println!("📝 Modifying file: {} (Category: {}, Severity: {})", file_path, category, severity);
                println!("📋 Reason: {}", reason);
                println!("🔄 Line changes: {}", line_changes.len());

                let references: Rc<Vec<&LineChange>> = Rc::new(line_changes.iter().collect());
                FileModifier::validate_file_modifications(&repository_config.path, file_path, Rc::clone(&references))?;
                FileModifier::apply_file_modifications(&repository_config.path, file_path, Rc::clone(&references))?;
            }
            FileChange::CreateFile { file_path, reason, severity, category, content } => {
                println!("📄 Creating file: {} (Category: {}, Severity: {})", file_path, category, severity);
                println!("📋 Reason: {}", reason);
                println!("📏 Content length: {} characters", content.len());

                FileChangeLogger::print_new_file_preview(file_path, content);
                FileModifier::create_file(&repository_config.path, file_path, content)?;
            }
            FileChange::DeleteFile { file_path, reason, severity, category } => {
                println!("🗑️ Deleting file: {} (Category: {}, Severity: {})", file_path, category, severity);
                println!("📋 Reason: {}", reason);

                FileModifier::delete_file(&repository_config.path, file_path)?;
            }
        }

        println!("✅ Successfully applied {} change", category);
        Ok(())
    }

    pub fn apply_changes_by_category(repository_config: Arc<RepositoryConfig>, file_changes: &[FileChange], target_category: &str) -> AilyzerResult<usize> {
        let mut applied_count = 0;
        let mut failed_count = 0;

        println!("🎯 Applying changes for category: {}", target_category);

        for change in file_changes {
            if let Some(category) = change.get_category() {
                if category == target_category {
                    match Self::apply_change_with_logging(Arc::clone(&repository_config), change) {
                        Ok(_) => applied_count += 1,
                        Err(e) => {
                            eprintln!("❌ Failed to apply change: {}", e);
                            failed_count += 1;
                        }
                    }
                }
            }
        }

        println!("📊 Category '{}': {} applied, {} failed", target_category, applied_count, failed_count);
        Ok(applied_count)
    }

    pub fn apply_changes_by_severity(repository_config: Arc<RepositoryConfig>, file_changes: &[FileChange], min_severity: &str) -> AilyzerResult<usize> {
        let severity_order = ["low", "medium", "high", "critical"];
        let min_index = severity_order.iter().position(|&s| s == min_severity).unwrap_or(0);

        let mut applied_count = 0;
        let mut failed_count = 0;

        println!("⚡ Applying changes with severity >= {}", min_severity);

        for change in file_changes {
            let change_severity = change.get_severity();
            if let Some(severity_index) = severity_order.iter().position(|&s| s == change_severity) {
                if severity_index >= min_index {
                    match Self::apply_change_with_logging(Arc::clone(&repository_config), change) {
                        Ok(_) => applied_count += 1,
                        Err(e) => {
                            eprintln!("❌ Failed to apply change: {}", e);
                            failed_count += 1;
                        }
                    }
                }
            }
        }

        println!("📊 Severity >= '{}': {} applied, {} failed", min_severity, applied_count, failed_count);
        Ok(applied_count)
    }

    pub fn apply_changes_by_priority(repository_config: Arc<RepositoryConfig>, file_changes: &[FileChange]) -> AilyzerResult<ApplyResult> {
        let mut result = ApplyResult::default();

        println!("🚀 Applying changes in priority order...");

        // 1. Critical security issues first
        println!("\n🔒 Phase 1: Critical Security Issues");
        let critical_security: Vec<_> = file_changes.iter()
            .filter(|c| c.is_security_related() && c.is_critical())
            .collect();

        for change in critical_security {
            match Self::apply_change_with_logging(Arc::clone(&repository_config), change) {
                Ok(_) => result.security_applied += 1,
                Err(e) => {
                    eprintln!("❌ Failed to apply critical security change: {}", e);
                    result.failed += 1;
                }
            }
        }

        // 2. Critical bugs
        println!("\n🐛 Phase 2: Critical Bug Fixes");
        let critical_bugs: Vec<_> = file_changes.iter()
            .filter(|c| c.is_bug_fix() && c.is_critical())
            .collect();

        for change in critical_bugs {
            match Self::apply_change_with_logging(Arc::clone(&repository_config), change) {
                Ok(_) => result.bugs_applied += 1,
                Err(e) => {
                    eprintln!("❌ Failed to apply critical bug fix: {}", e);
                    result.failed += 1;
                }
            }
        }

        // 3. High priority security and bugs
        println!("\n⚡ Phase 3: High Priority Issues");
        let high_priority: Vec<_> = file_changes.iter()
            .filter(|c| c.is_high_priority() && !c.is_critical() && (c.is_security_related() || c.is_bug_fix()))
            .collect();

        for change in high_priority {
            match Self::apply_change_with_logging(Arc::clone(&repository_config), change) {
                Ok(_) => {
                    if change.is_security_related() {
                        result.security_applied += 1;
                    } else {
                        result.bugs_applied += 1;
                    }
                }
                Err(e) => {
                    eprintln!("❌ Failed to apply high priority change: {}", e);
                    result.failed += 1;
                }
            }
        }

        // 4. Performance improvements
        println!("\n🚀 Phase 4: Performance Improvements");
        let performance: Vec<_> = file_changes.iter()
            .filter(|c| c.is_performance_improvement())
            .collect();

        for change in performance {
            match Self::apply_change_with_logging(Arc::clone(&repository_config), change) {
                Ok(_) => result.performance_applied += 1,
                Err(e) => {
                    eprintln!("❌ Failed to apply performance improvement: {}", e);
                    result.failed += 1;
                }
            }
        }

        // 5. Architecture improvements
        println!("\n🏗️ Phase 5: Architecture Improvements");
        let architecture: Vec<_> = file_changes.iter()
            .filter(|c| c.is_architecture_related())
            .collect();

        for change in architecture {
            match Self::apply_change_with_logging(Arc::clone(&repository_config), change) {
                Ok(_) => result.architecture_applied += 1,
                Err(e) => {
                    eprintln!("❌ Failed to apply architecture improvement: {}", e);
                    result.failed += 1;
                }
            }
        }

        // 6. Clean code improvements
        println!("\n✨ Phase 6: Clean Code Improvements");
        let clean_code: Vec<_> = file_changes.iter()
            .filter(|c| c.is_clean_code_related())
            .collect();

        for change in clean_code {
            match Self::apply_change_with_logging(Arc::clone(&repository_config), change) {
                Ok(_) => result.clean_code_applied += 1,
                Err(e) => {
                    eprintln!("❌ Failed to apply clean code improvement: {}", e);
                    result.failed += 1;
                }
            }
        }

        // 7. Duplicate code fixes
        println!("\n🔄 Phase 7: Duplicate Code Fixes");
        let duplicate_code: Vec<_> = file_changes.iter()
            .filter(|c| c.is_duplicate_code_fix())
            .collect();

        for change in duplicate_code {
            match Self::apply_change_with_logging(Arc::clone(&repository_config), change) {
                Ok(_) => result.duplicate_code_applied += 1,
                Err(e) => {
                    eprintln!("❌ Failed to apply duplicate code fix: {}", e);
                    result.failed += 1;
                }
            }
        }

        result.total_applied = result.security_applied + result.bugs_applied + result.performance_applied
            + result.architecture_applied + result.clean_code_applied + result.duplicate_code_applied;

        println!("\n📊 Priority Application Summary:");
        println!("   🔒 Security: {}", result.security_applied);
        println!("   🐛 Bugs: {}", result.bugs_applied);
        println!("   🚀 Performance: {}", result.performance_applied);
        println!("   🏗️ Architecture: {}", result.architecture_applied);
        println!("   ✨ Clean Code: {}", result.clean_code_applied);
        println!("   🔄 Duplicate Code: {}", result.duplicate_code_applied);
        println!("   ❌ Failed: {}", result.failed);
        println!("   📈 Total Applied: {}", result.total_applied);

        Ok(result)
    }

    pub fn get_change_statistics(file_changes: &[FileChange]) -> ChangeStatistics {
        let mut stats = ChangeStatistics::default();

        for change in file_changes {
            // Count by type
            match change {
                FileChange::ModifyFile { .. } => stats.modify_count += 1,
                FileChange::CreateFile { .. } => stats.create_count += 1,
                FileChange::DeleteFile { .. } => stats.delete_count += 1,
            }

            // Count by severity
            match change.get_severity() {
                "critical" => stats.critical_count += 1,
                "high" => stats.high_count += 1,
                "medium" => stats.medium_count += 1,
                "low" => stats.low_count += 1,
                _ => stats.unknown_severity_count += 1,
            }

            // Count by category
            if let Some(category) = change.get_category() {
                match category {
                    "BUGS" => stats.bugs_count += 1,
                    "SECURITY" => stats.security_count += 1,
                    "PERFORMANCE" => stats.performance_count += 1,
                    "CLEAN_CODE" => stats.clean_code_count += 1,
                    "ARCHITECTURE" => stats.architecture_count += 1,
                    "DUPLICATE_CODE" => stats.duplicate_code_count += 1,
                    _ => stats.other_category_count += 1,
                }
            }

            // Count line changes
            if let Some(line_changes) = change.get_line_changes() {
                stats.total_line_changes += line_changes.len();

                // Count multi-line changes
                for line_change in line_changes {
                    if line_change.is_multi_line() {
                        stats.multi_line_changes += 1;
                    }
                }
            }
        }

        stats.total_count = file_changes.len();
        stats
    }

    pub fn validate_changes_batch(repository_config: &RepositoryConfig, file_changes: &[FileChange]) -> AilyzerResult<ValidationResult> {
        let mut result = ValidationResult::default();

        println!("🔍 Validating {} changes...", file_changes.len());

        // Group changes by file
        let mut file_groups: std::collections::HashMap<String, Vec<&FileChange>> = std::collections::HashMap::new();
        for change in file_changes {
            file_groups.entry(change.get_file_path().to_string())
                .or_insert_with(Vec::new)
                .push(change);
        }

        for (file_path, changes) in file_groups {
            // Check if file exists for modify operations
            let full_path = format!("{}/{}", repository_config.path, file_path);
            let file_exists = Path::new(&full_path).exists();

            for change in &changes {
                match change {
                    FileChange::ModifyFile { line_changes, .. } => {
                        if !file_exists {
                            result.errors.push(format!("File does not exist: {}", file_path));
                            continue;
                        }

                        // Check for line change conflicts
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

                            // Validate individual line change
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
            println!("⚠️ {} validation warnings", result.warnings.len());
        }

        if !result.errors.is_empty() {
            println!("❌ {} validation errors", result.errors.len());
        } else {
            println!("✅ All changes validated successfully");
        }

        Ok(result)
    }

    pub fn apply_file_modifications(repo_path: &str, file_path: &str, changes: Rc<Vec<&LineChange>>) -> AilyzerResult<()> {
        let str_path = format!("{}/{}", repo_path, file_path).replace("//", "/");
        let full_path = Path::new(&*str_path);

        if !full_path.exists() {
            return Err(AilyzerError::file_error(
                full_path.to_str().unwrap(),
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

        for (_, change) in sorted_changes.iter().enumerate() {
            let adjusted_change = Self::adjust_change_line_numbers(change, cumulative_offset);

            let line_offset = match &adjusted_change {
                LineChange::Replace { line_number, old_content, new_content } => {
                    if new_content.contains('\n') {
                        let new_lines: Vec<String> = new_content.lines().map(|s| s.to_string()).collect();
                        let old_lines = vec![old_content.clone()];
                        Self::apply_replace_range(&mut lines, *line_number, *line_number, &old_lines, &new_lines)?;
                        new_lines.len() as i32 - 1
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
                        new_lines.len() as i32
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
                        new_lines.len() as i32
                    } else {
                        Self::apply_insert_before(&mut lines, *line_number, new_content)?;
                        1
                    }
                }
                LineChange::Delete { line_number } => {
                    Self::apply_delete(&mut lines, *line_number)?;
                    -1
                }
                LineChange::ReplaceRange { start_line, end_line, old_content, new_content } => {
                    let old_line_count = end_line - start_line + 1;
                    let new_line_count = new_content.len();
                    Self::apply_replace_range(&mut lines, *start_line, *end_line, old_content, new_content)?;
                    new_line_count as i32 - old_line_count as i32
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
                1
            } else {
                line_number - abs_offset
            }
        } else {
            line_number + offset as usize
        }
    }

    pub fn validate_file_modifications(repo_path: &str, file_path: &str, changes: Rc<Vec<&LineChange>>) -> AilyzerResult<()> {
        let full_path = format!("{}/{}", repo_path, file_path).replace("//", "/");
        let content = fs::read_to_string(&full_path)?;
        let original_lines: Vec<String> = content.lines().map(|s| s.to_string()).collect();
        Self::validate_changes(Rc::clone(&changes), &original_lines, full_path)?;
        Self::simulate_changes_application(Rc::clone(&changes), &original_lines)?;
        Ok(())
    }

    fn validate_changes(changes: Rc<Vec<&LineChange>>, lines: &[String], full_path: String) -> AilyzerResult<Vec<LineChange>> {
        let mut validated_changes = Vec::new();

        for (i, change) in changes.iter().enumerate() {
            match Self::validate_single_change(change, lines) {
                Ok(validated) => {
                    validated_changes.push(validated);
                }
                Err(e) => {
                    println!("❌ Change in {} Failed", full_path);
                    println!("❌ {}", e);
                    return Err(AilyzerError::validation_error(
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

    fn validate_single_change(change: &LineChange, lines: &[String]) -> AilyzerResult<LineChange> {
        match change {
            LineChange::Replace { line_number, old_content, .. } => {
                if *line_number == 0 || *line_number > lines.len() {
                    return Err(AilyzerError::validation_error(
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
                    return Err(AilyzerError::validation_error(
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
                    return Err(AilyzerError::validation_error(
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
                    return Err(AilyzerError::validation_error(
                        "line_number",
                        line_number.to_string().as_str(),
                        "Line Wrong",
                        Some(&format!("Line number {} is out of range (1-{})", line_number, lines.len()))
                    ));
                }
                Ok(change.clone())
            }
            LineChange::Delete { line_number } => {
                if *line_number == 0 || *line_number > lines.len() {
                    return Err(AilyzerError::validation_error(
                        "line_number",
                        line_number.to_string().as_str(),
                        "Line Wrong",
                        Some(&format!("Line number {} is out of range (1-{})", line_number, lines.len()))
                    ));
                }
                Ok(change.clone())
            }
            LineChange::ReplaceRange { start_line, end_line, old_content, .. } => {
                if *start_line == 0 || *end_line > lines.len() || start_line > end_line {
                    return Err(AilyzerError::validation_error(
                        "line_number",
                        start_line.to_string().as_str(),
                        "Line Wrong",
                        Some(&format!("Invalid range {}-{} for {} lines", start_line, end_line, lines.len()))
                    ));
                }

                for (i, expected_line) in old_content.iter().enumerate() {
                    let line_index = (*start_line - 1) + i;
                    if line_index >= lines.len() {
                        return Err(AilyzerError::validation_error(
                            "line_number",
                            start_line.to_string().as_str(),
                            "Line Wrong",
                            Some(&"Range extends beyond file length")
                        ));
                    }

                    let actual_line = &lines[line_index];
                    if actual_line.trim() != expected_line.trim() {
                        return Err(AilyzerError::validation_error(
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

    fn simulate_changes_application(changes: Rc<Vec<&LineChange>>, original_lines: &[String]) -> AilyzerResult<()> {
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
                        return Err(AilyzerError::validation_error(
                            "line_number",
                            line_number.to_string().as_str(),
                            "Line Wrong",
                            Some(&format!("After applying previous changes, change {} would reference invalid line {} (file has {} lines)", i + 1, line_number, simulated_line_count))
                        ));
                    }
                }
                LineChange::InsertAfter { line_number, .. } => {
                    if *line_number > simulated_line_count {
                        return Err(AilyzerError::validation_error(
                            "line_number",
                            line_number.to_string().as_str(),
                            "Line Wrong",
                            Some(&format!("After applying previous changes, change {} would reference invalid line {} (file has {} lines)", i + 1, line_number, simulated_line_count))
                        ));
                    }
                }
                LineChange::InsertBefore { line_number, .. } => {
                    if *line_number == 0 || *line_number > simulated_line_count + 1 {
                        return Err(AilyzerError::validation_error(
                            "line_number",
                            line_number.to_string().as_str(),
                            "Line Wrong",
                            Some(&format!("After applying previous changes, change {} would insert before invalid line {} (file has {} lines)", i + 1, line_number, simulated_line_count))
                        ));
                    }
                }
                LineChange::ReplaceRange { start_line, end_line, .. } => {
                    if *start_line == 0 || *end_line > simulated_line_count || start_line > end_line {
                        return Err(AilyzerError::validation_error(
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

    fn get_change_line_number(change: &LineChange) -> usize {
        match change {
            LineChange::Replace { line_number, .. } => *line_number,
            LineChange::InsertAfter { line_number, .. } => *line_number,
            LineChange::InsertBefore { line_number, .. } => *line_number,
            LineChange::Delete { line_number } => *line_number,
            LineChange::ReplaceRange { start_line, .. } => *start_line,
        }
    }

    fn apply_replace(lines: &mut Vec<String>, line_number: usize, _old_content: &str, new_content: &str) -> AilyzerResult<()> {
        if line_number == 0 || line_number > lines.len() {
            return Err(AilyzerError::validation_error(
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

    fn apply_insert_after(lines: &mut Vec<String>, line_number: usize, new_content: &str) -> AilyzerResult<()> {
        if line_number > lines.len() {
            return Err(AilyzerError::validation_error(
                "line_number",
                line_number.to_string().as_str(),
                "Line Wrong",
                Some(&format!("Line number {} out of range", line_number))
            ));
        }

        lines.insert(line_number, new_content.to_string());
        Ok(())
    }

    fn apply_insert_before(lines: &mut Vec<String>, line_number: usize, new_content: &str) -> AilyzerResult<()> {
        if line_number == 0 || line_number > lines.len() + 1 {
            return Err(AilyzerError::validation_error(
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

    fn apply_delete(lines: &mut Vec<String>, line_number: usize) -> AilyzerResult<()> {
        if line_number == 0 || line_number > lines.len() {
            return Err(AilyzerError::validation_error(
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

    fn apply_replace_range(
        lines: &mut Vec<String>,
        start_line: usize,
        end_line: usize,
        _old_content: &[String],
        new_content: &[String]
    ) -> AilyzerResult<()> {
        if start_line == 0 || end_line > lines.len() || start_line > end_line {
            return Err(AilyzerError::validation_error(
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

    fn create_file(repo_path: &str, file_path: &str, content: &str) -> AilyzerResult<()> {
        let full_path = format!("{}/{}", repo_path, file_path).replace("//", "/");
        let path = Path::new(&full_path);

        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }

        fs::write(path, content)?;
        Ok(())
    }

    fn delete_file(repo_path: &str, file_path: &str) -> AilyzerResult<()> {
        let full_path = format!("{}/{}", repo_path, file_path).replace("//", "/");
        let path = Path::new(&full_path);

        if path.exists() {
            fs::remove_file(path)?;
        } else {
            return Err(AilyzerError::validation_error(
                "file_exists",
                &full_path,
                "Line Wrong",
                Some("File does not exist")
            ));
        }

        Ok(())
    }
}