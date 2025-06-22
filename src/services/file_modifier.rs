use std::collections::HashMap;
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

    pub fn apply_changes_by_category(repository_config: Arc<RepositoryConfig>, file_changes: &[FileChange], target_category: &str) -> AilyzerResult<usize> {
        let mut applied_count = 0;
        let mut failed_count = 0;

        log::info!("🎯 Applying changes for category: {}", target_category);

        for change in file_changes {
            if let Some(category) = change.get_category() {
                if category == target_category {
                    match Self::apply_change_with_logging(Arc::clone(&repository_config), change) {
                        Ok(_) => applied_count += 1,
                        Err(e) => {
                            log::error!("❌ Failed to apply change: {}", e);
                            failed_count += 1;
                        }
                    }
                }
            }
        }

        log::info!("📊 Category '{}': {} applied, {} failed", target_category, applied_count, failed_count);
        Ok(applied_count)
    }

    pub fn apply_changes_by_severity(repository_config: Arc<RepositoryConfig>, file_changes: &[FileChange], min_severity: &str) -> AilyzerResult<usize> {
        let severity_order = ["low", "medium", "high", "critical"];
        let min_index = severity_order.iter().position(|&s| s == min_severity).unwrap_or(0);

        let mut applied_count = 0;
        let mut failed_count = 0;

        log::info!("⚡ Applying changes with severity >= {}", min_severity);

        for change in file_changes {
            let change_severity = change.get_severity();
            if let Some(severity_index) = severity_order.iter().position(|&s| s == change_severity) {
                if severity_index >= min_index {
                    match Self::apply_change_with_logging(Arc::clone(&repository_config), change) {
                        Ok(_) => applied_count += 1,
                        Err(e) => {
                            log::error!("❌ Failed to apply change: {}", e);
                            failed_count += 1;
                        }
                    }
                }
            }
        }

        log::info!("📊 Severity >= '{}': {} applied, {} failed", min_severity, applied_count, failed_count);
        Ok(applied_count)
    }

    pub fn apply_changes_by_priority(repository_config: Arc<RepositoryConfig>, file_changes: &[FileChange]) -> AilyzerResult<ApplyResult> {
        let mut result = ApplyResult::default();

        log::info!("🚀 Applying changes in priority order...");

        // 1. Critical security issues first
        log::info!("\n🔒 Phase 1: Critical Security Issues");
        let critical_security: Vec<_> = file_changes.iter()
            .filter(|c| c.is_security_related() && c.is_critical())
            .collect();

        for change in critical_security {
            match Self::apply_change_with_logging(Arc::clone(&repository_config), change) {
                Ok(_) => result.security_applied += 1,
                Err(e) => {
                    log::error!("❌ Failed to apply critical security change: {}", e);
                    result.failed += 1;
                }
            }
        }

        // 2. Critical bugs
        log::info!("\n🐛 Phase 2: Critical Bug Fixes");
        let critical_bugs: Vec<_> = file_changes.iter()
            .filter(|c| c.is_bug_fix() && c.is_critical())
            .collect();

        for change in critical_bugs {
            match Self::apply_change_with_logging(Arc::clone(&repository_config), change) {
                Ok(_) => result.bugs_applied += 1,
                Err(e) => {
                    log::error!("❌ Failed to apply critical bug fix: {}", e);
                    result.failed += 1;
                }
            }
        }

        // 3. High priority security and bugs
        log::info!("\n⚡ Phase 3: High Priority Issues");
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
                    log::error!("❌ Failed to apply high priority change: {}", e);
                    result.failed += 1;
                }
            }
        }

        // 4. Performance improvements
        log::info!("\n🚀 Phase 4: Performance Improvements");
        let performance: Vec<_> = file_changes.iter()
            .filter(|c| c.is_performance_improvement())
            .collect();

        for change in performance {
            match Self::apply_change_with_logging(Arc::clone(&repository_config), change) {
                Ok(_) => result.performance_applied += 1,
                Err(e) => {
                    log::error!("❌ Failed to apply performance improvement: {}", e);
                    result.failed += 1;
                }
            }
        }

        // 5. Architecture improvements
        log::info!("\n🏗️ Phase 5: Architecture Improvements");
        let architecture: Vec<_> = file_changes.iter()
            .filter(|c| c.is_architecture_related())
            .collect();

        for change in architecture {
            match Self::apply_change_with_logging(Arc::clone(&repository_config), change) {
                Ok(_) => result.architecture_applied += 1,
                Err(e) => {
                    log::error!("❌ Failed to apply architecture improvement: {}", e);
                    result.failed += 1;
                }
            }
        }

        // 6. Clean code improvements
        log::info!("\n✨ Phase 6: Clean Code Improvements");
        let clean_code: Vec<_> = file_changes.iter()
            .filter(|c| c.is_clean_code_related())
            .collect();

        for change in clean_code {
            match Self::apply_change_with_logging(Arc::clone(&repository_config), change) {
                Ok(_) => result.clean_code_applied += 1,
                Err(e) => {
                    log::error!("❌ Failed to apply clean code improvement: {}", e);
                    result.failed += 1;
                }
            }
        }

        // 7. Duplicate code fixes
        log::info!("\n🔄 Phase 7: Duplicate Code Fixes");
        let duplicate_code: Vec<_> = file_changes.iter()
            .filter(|c| c.is_duplicate_code_fix())
            .collect();

        for change in duplicate_code {
            match Self::apply_change_with_logging(Arc::clone(&repository_config), change) {
                Ok(_) => result.duplicate_code_applied += 1,
                Err(e) => {
                    log::error!("❌ Failed to apply duplicate code fix: {}", e);
                    result.failed += 1;
                }
            }
        }

        result.total_applied = result.security_applied + result.bugs_applied + result.performance_applied
            + result.architecture_applied + result.clean_code_applied + result.duplicate_code_applied;

        log::info!("\n📊 Priority Application Summary:");
        log::info!("   🔒 Security: {}", result.security_applied);
        log::info!("   🐛 Bugs: {}", result.bugs_applied);
        log::info!("   🚀 Performance: {}", result.performance_applied);
        log::info!("   🏗️ Architecture: {}", result.architecture_applied);
        log::info!("   ✨ Clean Code: {}", result.clean_code_applied);
        log::info!("   🔄 Duplicate Code: {}", result.duplicate_code_applied);
        log::info!("   ❌ Failed: {}", result.failed);
        log::info!("   📈 Total Applied: {}", result.total_applied);

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

        log::info!("🔍 Validating {} changes...", file_changes.len());

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
            log::info!("❌ {} validation errors", result.errors.len());
        } else {
            log::info!("✅ All changes validated successfully");
        }

        Ok(result)
    }

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

        log::info!("🔧 Applying {} change ({}): {}", category, severity, file_change.get_file_path());

        match file_change {
            FileChange::ModifyFile { file_path, reason, severity, category, line_changes } => {
                log::info!("📝 Modifying file: {} (Category: {}, Severity: {})", file_path, category, severity);
                log::info!("📋 Reason: {}", reason);
                log::info!("🔄 Line changes: {}", line_changes.len());

                let references: Rc<Vec<&LineChange>> = Rc::new(line_changes.iter().collect());
                FileModifier::validate_file_modifications(&repository_config.path, file_path, Rc::clone(&references))?;
                FileModifier::apply_file_modifications(&repository_config.path, file_path, Rc::clone(&references))?;
            }
            FileChange::CreateFile { file_path, reason, severity, category, content } => {
                log::info!("📄 Creating file: {} (Category: {}, Severity: {})", file_path, category, severity);
                log::info!("📋 Reason: {}", reason);
                log::info!("📏 Content length: {} characters", content.len());

                FileChangeLogger::print_new_file_preview(file_path, content);
                FileModifier::create_file(&repository_config.path, file_path, content)?;
            }
            FileChange::DeleteFile { file_path, reason, severity, category } => {
                log::info!("🗑️ Deleting file: {} (Category: {}, Severity: {})", file_path, category, severity);
                log::info!("📋 Reason: {}", reason);

                FileModifier::delete_file(&repository_config.path, file_path)?;
            }
        }

        log::info!("✅ Successfully applied {} change", category);
        Ok(())
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

        // CRITICAL FIX: Sort changes by line number to ensure proper sequential application
        let mut sorted_changes = validated_changes;
        sorted_changes.sort_by_key(|change| Self::get_change_line_number(change));

        log::info!("🔧 Applying {} changes to {}", sorted_changes.len(), file_path);
        for (i, change) in sorted_changes.iter().enumerate() {
            log::info!("   {}. {} (original line {})",
                i + 1,
                change.get_description(),
                Self::get_change_line_number(change)
            );
        }

        let mut lines = original_lines.clone();
        let mut cumulative_offset: i32 = 0;

        // CRITICAL FIX: Apply changes one by one with proper offset tracking
        for (change_index, change) in sorted_changes.iter().enumerate() {
            log::info!("🔄 Applying change {} of {}: {} (cumulative offset: {})",
                change_index + 1,
                sorted_changes.len(),
                change.get_description(),
                cumulative_offset
            );

            // CRITICAL FIX: Adjust line numbers based on cumulative offset from previous changes
            let adjusted_change = Self::adjust_change_line_numbers(change, cumulative_offset);

            log::info!("   📍 Original line: {}, Adjusted line: {}",
                Self::get_change_line_number(change),
                Self::get_change_line_number(&adjusted_change)
            );

            // Apply the change and calculate the line offset it introduces
            let line_offset = match &adjusted_change {
                LineChange::Replace { line_number, old_content, new_content } => {
                    if new_content.contains('\n') {
                        let new_lines: Vec<String> = new_content.lines().map(|s| s.to_string()).collect();
                        let old_lines = vec![old_content.clone()];
                        Self::apply_replace_range(&mut lines, *line_number, *line_number, &old_lines, &new_lines)?;
                        let offset = new_lines.len() as i32 - 1;
                        log::info!("   ✅ Replace with multi-line: {} lines added", offset);
                        offset
                    } else {
                        Self::apply_replace(&mut lines, *line_number, old_content, new_content)?;
                        log::info!("   ✅ Replace single line: no offset");
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
                        log::info!("   ✅ Insert after multi-line: {} lines added", offset);
                        offset
                    } else {
                        Self::apply_insert_after(&mut lines, *line_number, new_content)?;
                        log::info!("   ✅ Insert after single line: 1 line added");
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
                        log::info!("   ✅ Insert before multi-line: {} lines added", offset);
                        offset
                    } else {
                        Self::apply_insert_before(&mut lines, *line_number, new_content)?;
                        log::info!("   ✅ Insert before single line: 1 line added");
                        1
                    }
                }
                // Handle multi-line insert actions
                LineChange::InsertManyAfter { line_number, new_lines } => {
                    Self::apply_insert_many_after(&mut lines, *line_number, new_lines)?;
                    let offset = new_lines.len() as i32;
                    log::info!("   ✅ Insert many after: {} lines added", offset);
                    offset
                }
                LineChange::InsertManyBefore { line_number, new_lines } => {
                    Self::apply_insert_many_before(&mut lines, *line_number, new_lines)?;
                    let offset = new_lines.len() as i32;
                    log::info!("   ✅ Insert many before: {} lines added", offset);
                    offset
                }
                LineChange::Delete { line_number } => {
                    Self::apply_delete(&mut lines, *line_number)?;
                    log::info!("   ✅ Delete single line: 1 line removed");
                    -1
                }
                // Handle multi-line delete action
                LineChange::DeleteMany { start_line, end_line } => {
                    let deleted_count = end_line - start_line + 1;
                    Self::apply_delete_many(&mut lines, *start_line, *end_line)?;
                    let offset = -(deleted_count as i32);
                    log::info!("   ✅ Delete many: {} lines removed", deleted_count);
                    offset
                }
                LineChange::ReplaceRange { start_line, end_line, old_content, new_content } => {
                    let old_line_count = end_line - start_line + 1;
                    let new_line_count = new_content.len();
                    Self::apply_replace_range(&mut lines, *start_line, *end_line, old_content, new_content)?;
                    let offset = new_line_count as i32 - old_line_count as i32;
                    log::info!("   ✅ Replace range: {} old lines → {} new lines (offset: {})",
                        old_line_count, new_line_count, offset);
                    offset
                }
            };

            // CRITICAL FIX: Update cumulative offset for subsequent changes
            cumulative_offset += line_offset;
            log::info!("   📊 New cumulative offset: {}", cumulative_offset);
            log::info!("   📏 File now has {} lines", lines.len());
        }

        let new_content = lines.join("\n");
        fs::write(&full_path, new_content)?;

        log::info!("✅ Successfully applied all {} changes to {}", sorted_changes.len(), file_path);
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
                    log::info!("❌ Change in {} Failed", full_path);
                    log::info!("❌ {}", e);
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
            // Validate multi-line insert actions
            LineChange::InsertManyAfter { line_number, new_lines } => {
                if *line_number > lines.len() {
                    return Err(AilyzerError::validation_error(
                        "line_number",
                        line_number.to_string().as_str(),
                        "Line Wrong",
                        Some(&format!("Line number {} is out of range (1-{})", line_number, lines.len()))
                    ));
                }
                if new_lines.is_empty() {
                    return Err(AilyzerError::validation_error(
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
                    return Err(AilyzerError::validation_error(
                        "line_number",
                        line_number.to_string().as_str(),
                        "Line Wrong",
                        Some(&format!("Line number {} is out of range (1-{})", line_number, lines.len()))
                    ));
                }
                if new_lines.is_empty() {
                    return Err(AilyzerError::validation_error(
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
                    return Err(AilyzerError::validation_error(
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
                    return Err(AilyzerError::validation_error(
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
                LineChange::InsertAfter { line_number, .. } |
                LineChange::InsertManyAfter { line_number, .. } => {
                    if *line_number > simulated_line_count {
                        return Err(AilyzerError::validation_error(
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
                        return Err(AilyzerError::validation_error(
                            "line_number",
                            line_number.to_string().as_str(),
                            "Line Wrong",
                            Some(&format!("After applying previous changes, change {} would insert before invalid line {} (file has {} lines)", i + 1, line_number, simulated_line_count))
                        ));
                    }
                }
                LineChange::DeleteMany { start_line, end_line } => {
                    if *start_line == 0 || *end_line > simulated_line_count || start_line > end_line {
                        return Err(AilyzerError::validation_error(
                            "line_range",
                            start_line.to_string().as_str(),
                            "Invalid Range",
                            Some(&format!("After applying previous changes, change {} would use invalid range {}-{} (file has {} lines)", i + 1, start_line, end_line, simulated_line_count))
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

    fn apply_insert_many_after(lines: &mut Vec<String>, line_number: usize, new_lines: &[String]) -> AilyzerResult<()> {
        if line_number > lines.len() {
            return Err(AilyzerError::validation_error(
                "line_number",
                line_number.to_string().as_str(),
                "Line Wrong",
                Some(&format!("Line number {} out of range", line_number))
            ));
        }

        if new_lines.is_empty() {
            return Err(AilyzerError::validation_error(
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

    fn apply_insert_many_before(lines: &mut Vec<String>, line_number: usize, new_lines: &[String]) -> AilyzerResult<()> {
        if line_number == 0 || line_number > lines.len() + 1 {
            return Err(AilyzerError::validation_error(
                "line_number",
                line_number.to_string().as_str(),
                "Line Wrong",
                Some(&format!("Line number {} out of range", line_number))
            ));
        }

        if new_lines.is_empty() {
            return Err(AilyzerError::validation_error(
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

    fn apply_delete_many(lines: &mut Vec<String>, start_line: usize, end_line: usize) -> AilyzerResult<()> {
        if start_line == 0 || end_line > lines.len() || start_line > end_line {
            return Err(AilyzerError::validation_error(
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

    fn apply_single_change(lines: &mut Vec<String>, change: &LineChange) -> AilyzerResult<i32> {
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

    pub fn apply_changes_grouped_by_file(repository_config: Arc<RepositoryConfig>, file_changes: Vec<&FileChange>) -> AilyzerResult<usize> {
        let mut applied_count = 0;
        let mut failed_count = 0;

        let mut file_groups: std::collections::HashMap<String, Vec<&FileChange>> = std::collections::HashMap::new();
        for change in file_changes {
            file_groups.entry(change.get_file_path().to_string())
                .or_insert_with(Vec::new)
                .push(change);
        }

        log::info!("🔧 Applying changes to {} files", file_groups.len());

        for (file_path, changes) in file_groups {
            log::info!("📁 Processing file: {} ({} changes)", file_path, changes.len());

            match Self::apply_changes_to_single_file(Arc::clone(&repository_config), &file_path, &changes) {
                Ok(count) => {
                    applied_count += count;
                    log::info!("✅ Successfully applied {} changes to {}", count, file_path);
                }
                Err(e) => {
                    log::error!("❌ Failed to apply changes to {}: {}", file_path, e);
                    failed_count += changes.len();
                }
            }
        }

        log::info!("📊 Total: {} applied, {} failed", applied_count, failed_count);
        Ok(applied_count)
    }

    pub fn apply_file_modifications_with_smart_validation(repo_path: &str, file_path: &str, changes: Rc<Vec<&LineChange>>) -> AilyzerResult<()> {
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

        // Sort changes by line number
        let mut sorted_changes: Vec<LineChange> = changes.iter().map(|&c| c.clone()).collect();
        sorted_changes.sort_by_key(|change| Self::get_change_line_number(change));

        log::info!("🔧 Applying {} changes to {} (smart validation)", sorted_changes.len(), file_path);

        // Show original file context
        Self::show_original_file_context(&original_lines, &sorted_changes);

        let mut lines = original_lines.clone();
        let mut line_offset_map: HashMap<usize, i32> = HashMap::new();

        for (change_index, change) in sorted_changes.iter().enumerate() {
            log::info!("🔄 Applying change {} of {}: {}",
                change_index + 1,
                sorted_changes.len(),
                change.get_description()
            );

            let original_line_number = Self::get_change_line_number(change);
            let cumulative_offset = Self::calculate_cumulative_offset(&line_offset_map, original_line_number);

            log::info!("   📍 Original line: {}, Cumulative offset: {}",
                original_line_number, cumulative_offset);

            let adjusted_change = Self::adjust_change_line_numbers(change, cumulative_offset);
            log::info!("   📍 Adjusted line: {}", Self::get_change_line_number(&adjusted_change));

            // Show context around the change
            Self::debug_file_state_around_change(&adjusted_change, &lines);

            // SMART VALIDATION: Try exact match first, then fuzzy match
            match Self::validate_single_change_against_current_state(&adjusted_change, &lines) {
                Ok(_) => {
                    log::info!("   ✅ Exact validation passed");
                }
                Err(exact_error) => {
                    log::warn!("   ⚠️ Exact validation failed: {}", exact_error);

                    // Try smart/fuzzy validation
                    match Self::smart_validate_and_adjust_change(&adjusted_change, &lines) {
                        Ok(smart_adjusted_change) => {
                            log::info!("   ✅ Smart validation found a match");
                            // Use the smart-adjusted change instead
                            let line_offset = Self::apply_single_change(&mut lines, &smart_adjusted_change)?;
                            line_offset_map.insert(original_line_number, line_offset);
                            continue;
                        }
                        Err(smart_error) => {
                            log::error!("❌ Both exact and smart validation failed");
                            log::error!("   Exact error: {}", exact_error);
                            log::error!("   Smart error: {}", smart_error);
                            return Err(AilyzerError::validation_error(
                                "line_number",
                                "0",
                                "Line Wrong",
                                Some(&format!("Change {} validation failed", change_index + 1))
                            ));
                        }
                    }
                }
            }

            // Apply the change
            let line_offset = Self::apply_single_change(&mut lines, &adjusted_change)?;
            line_offset_map.insert(original_line_number, line_offset);

            log::info!("   📊 Applied with offset: {}", line_offset);
            log::info!("   📏 File now has {} lines", lines.len());
        }

        let new_content = lines.join("\n");
        fs::write(&full_path, new_content)?;

        log::info!("✅ Successfully applied all {} changes to {}", sorted_changes.len(), file_path);
        Ok(())
    }

    fn validate_single_change_against_current_state(change: &LineChange, current_lines: &[String]) -> AilyzerResult<()> {
        match change {
            LineChange::Replace { line_number, old_content, .. } => {
                if *line_number == 0 || *line_number > current_lines.len() {
                    return Err(AilyzerError::validation_error(
                        "line_number",
                        &line_number.to_string(),
                        "Line Wrong",
                        Some(&format!("Line {} is out of bounds (file has {} lines)", line_number, current_lines.len()))
                    ));
                }

                let actual_content = &current_lines[*line_number - 1];
                if actual_content.trim() != old_content.trim() {
                    return Err(AilyzerError::validation_error(
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
                    return Err(AilyzerError::validation_error(
                        "line_number",
                        &start_line.to_string(),
                        "Line Wrong",
                        Some(&format!("Line range {}-{} is invalid (file has {} lines)", start_line, end_line, current_lines.len()))
                    ));
                }

                for (i, expected_line) in old_content.iter().enumerate() {
                    let line_index = start_line - 1 + i;
                    if line_index >= current_lines.len() {
                        return Err(AilyzerError::validation_error(
                            "line_number",
                            &(line_index + 1).to_string(),
                            "Line Wrong",
                            Some(&format!("Line {} is out of bounds", line_index + 1))
                        ));
                    }

                    let actual_line = &current_lines[line_index];
                    if actual_line.trim() != expected_line.trim() {
                        return Err(AilyzerError::validation_error(
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
                    return Err(AilyzerError::validation_error(
                        "line_number",
                        &line_number.to_string(),
                        "Line Wrong",
                        Some(&format!("Line {} is out of bounds (file has {} lines)", line_number, current_lines.len()))
                    ));
                }
            }
            LineChange::DeleteMany { start_line, end_line } => {
                if *start_line == 0 || *end_line > current_lines.len() || start_line > end_line {
                    return Err(AilyzerError::validation_error(
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
                    return Err(AilyzerError::validation_error(
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

    fn debug_file_state_around_change(change: &LineChange, lines: &[String]) {
        let (start_line, end_line) = match change {
            LineChange::Replace { line_number, .. } => (*line_number, *line_number),
            LineChange::ReplaceRange { start_line, end_line, .. } => (*start_line, *end_line),
            LineChange::Delete { line_number } => (*line_number, *line_number),
            LineChange::DeleteMany { start_line, end_line } => (*start_line, *end_line),
            LineChange::InsertAfter { line_number, .. } |
            LineChange::InsertBefore { line_number, .. } |
            LineChange::InsertManyAfter { line_number, .. } |
            LineChange::InsertManyBefore { line_number, .. } => (*line_number, *line_number),
        };

        log::info!("🔍 Current file state around lines {}-{}", start_line, end_line);
        log::info!("   File has {} total lines", lines.len());

        let context_start = start_line.saturating_sub(3).max(1);
        let context_end = (end_line + 3).min(lines.len());

        for i in context_start..=context_end {
            if i > 0 && i <= lines.len() {
                let marker = if i >= start_line && i <= end_line { ">>>" } else { "   " };
                log::info!("   {} {}: '{}'", marker, i, lines[i-1]);
            }
        }
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

    fn show_original_file_context(lines: &[String], changes: &[LineChange]) {
        log::info!("📄 Original file context:");
        log::info!("   File has {} lines", lines.len());

        for change in changes {
            let line_num = Self::get_change_line_number(change);
            log::info!("   Change at line {}: {}", line_num, change.get_description());

            let context_start = line_num.saturating_sub(2).max(1);
            let context_end = (line_num + 2).min(lines.len());

            for i in context_start..=context_end {
                if i > 0 && i <= lines.len() {
                    let marker = if i == line_num { ">>>" } else { "   " };
                    log::info!("     {} {}: '{}'", marker, i, lines[i-1]);
                }
            }
        }
    }

    fn apply_changes_to_single_file(repository_config: Arc<RepositoryConfig>, file_path: &str, changes: &[&FileChange]) -> AilyzerResult<usize> {
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
            log::info!("🔧 Applying {}: {}",
                match change {
                    FileChange::CreateFile { .. } => "CREATE",
                    FileChange::DeleteFile { .. } => "DELETE",
                    _ => "OTHER"
                },
                change.get_file_path()
            );

            match Self::apply_change_with_logging(Arc::clone(&repository_config), change) {
                Ok(_) => applied_count += 1,
                Err(e) => {
                    log::error!("❌ Failed to apply {}: {}", change.get_file_path(), e);
                    return Err(e);
                }
            }
        }

        if !modify_changes.is_empty() {
            log::info!("🔧 Applying {} line changes to {}", modify_changes.len(), file_path);

            let changes_refs: Rc<Vec<&LineChange>> = Rc::new(modify_changes.clone());

            match Self::apply_file_modifications_with_smart_validation(&repository_config.path, file_path, changes_refs) {
                Ok(_) => {
                    applied_count += modify_changes.len();
                    log::info!("✅ Successfully applied {} line changes to {}", modify_changes.len(), file_path);
                }
                Err(e) => {
                    log::error!("❌ Failed to apply line changes to {}: {}", file_path, e);
                    return Err(e);
                }
            }
        }

        Ok(applied_count)
    }

    pub fn apply_changes_by_priority_grouped(repository_config: Arc<RepositoryConfig>, file_changes: &[FileChange]) -> AilyzerResult<ApplyResult> {
        let mut result = ApplyResult::default();

        log::info!("🚀 Applying changes in priority order (grouped by file)...");

        // Group changes by priority and category
        let critical_security: Vec<&FileChange> = file_changes.iter()
            .filter(|c| c.is_security_related() && c.is_critical())
            .collect();

        let critical_bugs: Vec<&FileChange> = file_changes.iter()
            .filter(|c| c.is_bug_fix() && c.is_critical())
            .collect();

        let high_priority: Vec<&FileChange> = file_changes.iter()
            .filter(|c| c.is_high_priority() && !c.is_critical() && (c.is_security_related() || c.is_bug_fix()))
            .collect();

        let performance: Vec<&FileChange> = file_changes.iter()
            .filter(|c| c.is_performance_improvement())
            .collect();

        let architecture: Vec<&FileChange> = file_changes.iter()
            .filter(|c| c.is_architecture_related())
            .collect();

        let clean_code: Vec<&FileChange> = file_changes.iter()
            .filter(|c| c.is_clean_code_related())
            .collect();

        let duplicate_code: Vec<&FileChange> = file_changes.iter()
            .filter(|c| c.is_duplicate_code_fix())
            .collect();


        if !critical_security.is_empty() {
            log::info!("\n🔒 Phase 1: Critical Security ({} changes)", critical_security.len());
            match Self::apply_changes_grouped_by_file(Arc::clone(&repository_config), critical_security.clone()) {
                Ok(applied) => {
                    result.security_applied += applied;
                    log::info!("✅ Applied {} critical security changes", applied);
                }
                Err(e) => {
                    log::error!("❌ Failed to apply critical security changes: {}", e);
                    result.failed += critical_security.len();
                }
            }
        }

        if !critical_bugs.is_empty() {
            log::info!("\n🐛 Phase 2: Critical Bugs ({} changes)", critical_bugs.len());
            match Self::apply_changes_grouped_by_file(Arc::clone(&repository_config), critical_bugs.clone()) {
                Ok(applied) => {
                    result.bugs_applied += applied;
                    log::info!("✅ Applied {} critical bug fixes", applied);
                }
                Err(e) => {
                    log::error!("❌ Failed to apply critical bug fixes: {}", e);
                    result.failed += critical_bugs.len();
                }
            }
        }

        if !high_priority.is_empty() {
            log::info!("\n⚡ Phase 3: High Priority ({} changes)", high_priority.len());
            match Self::apply_changes_grouped_by_file(Arc::clone(&repository_config), high_priority.clone()) {
                Ok(applied) => {
                    // Split the applied count between security and bugs based on the actual changes
                    let mut security_count = 0;
                    let mut bugs_count = 0;

                    for change in &high_priority {
                        if change.is_security_related() {
                            security_count += 1;
                        } else if change.is_bug_fix() {
                            bugs_count += 1;
                        }
                    }

                    result.security_applied += security_count;
                    result.bugs_applied += bugs_count;
                    log::info!("✅ Applied {} high priority changes ({} security, {} bugs)", applied, security_count, bugs_count);
                }
                Err(e) => {
                    log::error!("❌ Failed to apply high priority changes: {}", e);
                    result.failed += high_priority.len();
                }
            }
        }

        if !performance.is_empty() {
            log::info!("\n🚀 Phase 4: Performance ({} changes)", performance.len());
            match Self::apply_changes_grouped_by_file(Arc::clone(&repository_config), performance.clone()) {
                Ok(applied) => {
                    result.performance_applied += applied;
                    log::info!("✅ Applied {} performance improvements", applied);
                }
                Err(e) => {
                    log::error!("❌ Failed to apply performance improvements: {}", e);
                    result.failed += performance.len();
                }
            }
        }

        if !architecture.is_empty() {
            log::info!("\n🏗️ Phase 5: Architecture ({} changes)", architecture.len());
            match Self::apply_changes_grouped_by_file(Arc::clone(&repository_config), architecture.clone()) {
                Ok(applied) => {
                    result.architecture_applied += applied;
                    log::info!("✅ Applied {} architecture improvements", applied);
                }
                Err(e) => {
                    log::error!("❌ Failed to apply architecture improvements: {}", e);
                    result.failed += architecture.len();
                }
            }
        }

        if !clean_code.is_empty() {
            log::info!("\n✨ Phase 6: Clean Code ({} changes)", clean_code.len());
            match Self::apply_changes_grouped_by_file(Arc::clone(&repository_config), clean_code.clone()) {
                Ok(applied) => {
                    result.clean_code_applied += applied;
                    log::info!("✅ Applied {} clean code improvements", applied);
                }
                Err(e) => {
                    log::error!("❌ Failed to apply clean code improvements: {}", e);
                    result.failed += clean_code.len();
                }
            }
        }

        if !duplicate_code.is_empty() {
            log::info!("\n🔄 Phase 7: Duplicate Code ({} changes)", duplicate_code.len());
            match Self::apply_changes_grouped_by_file(Arc::clone(&repository_config), duplicate_code.clone()) {
                Ok(applied) => {
                    result.duplicate_code_applied += applied;
                    log::info!("✅ Applied {} duplicate code fixes", applied);
                }
                Err(e) => {
                    log::error!("❌ Failed to apply duplicate code fixes: {}", e);
                    result.failed += duplicate_code.len();
                }
            }
        }

        result.total_applied = result.security_applied + result.bugs_applied + result.performance_applied
            + result.architecture_applied + result.clean_code_applied + result.duplicate_code_applied;

        log::info!("\n📊 Priority Application Summary:");
        log::info!("   🔒 Security: {}", result.security_applied);
        log::info!("   🐛 Bugs: {}", result.bugs_applied);
        log::info!("   🚀 Performance: {}", result.performance_applied);
        log::info!("   🏗️ Architecture: {}", result.architecture_applied);
        log::info!("   ✨ Clean Code: {}", result.clean_code_applied);
        log::info!("   🔄 Duplicate Code: {}", result.duplicate_code_applied);
        log::info!("   ❌ Failed: {}", result.failed);
        log::info!("   📈 Total Applied: {}", result.total_applied);

        Ok(result)
    }

    fn smart_validate_and_adjust_change(change: &LineChange, current_lines: &[String]) -> AilyzerResult<LineChange> {
        match change {
            LineChange::Replace { line_number, old_content, new_content } => {
                // Try to find the old_content in nearby lines
                let search_start = line_number.saturating_sub(5).max(1);
                let search_end = (line_number + 5).min(current_lines.len());

                log::info!("🔍 Smart search for single line content around line {} (range {}-{})",
                line_number, search_start, search_end);

                for offset in 0..10 {
                    for direction in [0i32, 1i32, -1i32] {
                        let line_offset = direction * offset as i32;
                        let new_line_number = (*line_number as i32 + line_offset).max(1) as usize;

                        if new_line_number > 0 && new_line_number <= current_lines.len() {
                            let actual_line = &current_lines[new_line_number - 1];
                            if actual_line.trim() == old_content.trim() {
                                log::info!("   ✅ Found matching content at line {} (offset: {})",
                                new_line_number, line_offset);
                                return Ok(LineChange::Replace {
                                    line_number: new_line_number,
                                    old_content: old_content.clone(),
                                    new_content: new_content.clone(),
                                });
                            }
                        }
                    }
                }

                Err(AilyzerError::validation_error(
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
                    return Err(AilyzerError::validation_error(
                        "smart_validation",
                        "0",
                        "RangeMismatch",
                        Some(&format!("Old content has {} lines but range is {} lines",
                                      old_content.len(), original_range_size))
                    ));
                }

                let search_start = start_line.saturating_sub(10).max(1);
                let search_end = (end_line + 10).min(current_lines.len());

                log::info!("🔍 Smart search for range content around lines {}-{} (search range {}-{})",
                start_line, end_line, search_start, search_end);

                // Try different starting positions within the search range
                for start_offset in 0..20 {
                    for direction in [0i32, 1i32, -1i32] {
                        let offset = direction * start_offset as i32;
                        let new_start = (*start_line as i32 + offset).max(1) as usize;
                        let new_end = new_start + (end_line - start_line);

                        // Check bounds
                        if new_end <= current_lines.len() && new_start >= 1 {
                            // Check if all lines in the range match
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
                                log::info!("   ✅ Found matching range at lines {}-{} (offset: {})",
                                new_start, new_end, offset);
                                return Ok(LineChange::ReplaceRange {
                                    start_line: new_start,
                                    end_line: new_end,
                                    old_content: old_content.clone(),
                                    new_content: new_content.clone(),
                                });
                            }

                            // Debug: Show why this position didn't match
                            if offset == 0 && start_offset < 3 {
                                log::info!("   🔍 Position {}-{} doesn't match:", new_start, new_end);
                                for (i, expected_line) in old_content.iter().enumerate() {
                                    let line_index = new_start - 1 + i;
                                    if line_index < current_lines.len() {
                                        let actual_line = &current_lines[line_index];
                                        let matches = actual_line.trim() == expected_line.trim();
                                        log::info!("     {} {}: '{}' vs '{}'",
                                        if matches { "✓" } else { "✗" },
                                        new_start + i,
                                        expected_line.trim(),
                                        actual_line.trim()
                                    );
                                    }
                                }
                            }
                        }
                    }
                }

                Err(AilyzerError::validation_error(
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
                    // Try to find a nearby valid line number
                    let search_range = 5;
                    for offset in 1..=search_range {
                        for direction in [1i32, -1i32] {
                            let new_line_number = (*line_number as i32 + direction * offset).max(1) as usize;
                            if new_line_number > 0 && new_line_number <= current_lines.len() {
                                log::info!("   ✅ Adjusted delete from line {} to line {}", line_number, new_line_number);
                                return Ok(LineChange::Delete {
                                    line_number: new_line_number,
                                });
                            }
                        }
                    }

                    Err(AilyzerError::validation_error(
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
                        log::info!("   ✅ Adjusted delete range from {}-{} to {}-{}",
                        start_line, end_line, adjusted_start, adjusted_end);
                        Ok(LineChange::DeleteMany {
                            start_line: adjusted_start,
                            end_line: adjusted_end,
                        })
                    } else {
                        Err(AilyzerError::validation_error(
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
                    // Insert at the end of file instead
                    log::info!("   ✅ Adjusted insert from after line {} to after line {} (end of file)",
                    line_number, max_line);
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
                    log::info!("   ✅ Adjusted insert from before line {} to before line {} (end of file)",
                    line_number, max_line);
                    Ok(LineChange::InsertBefore {
                        line_number: max_line,
                        new_content: new_content.clone(),
                    })
                } else {
                    // Insert at the beginning
                    log::info!("   ✅ Adjusted insert from before line {} to before line 1 (beginning of file)",
                    line_number);
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
                    log::info!("   ✅ Adjusted insert many from after line {} to after line {} (end of file)",
                    line_number, max_line);
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
                    log::info!("   ✅ Adjusted insert many from before line {} to before line {} (end of file)",
                    line_number, max_line);
                    Ok(LineChange::InsertManyBefore {
                        line_number: max_line,
                        new_lines: new_lines.clone(),
                    })
                } else {
                    log::info!("   ✅ Adjusted insert many from before line {} to before line 1 (beginning of file)",
                    line_number);
                    Ok(LineChange::InsertManyBefore {
                        line_number: 1,
                        new_lines: new_lines.clone(),
                    })
                }
            }
        }
    }

}