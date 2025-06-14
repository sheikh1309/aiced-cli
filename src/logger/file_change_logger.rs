use std::fs;
use std::rc::Rc;
use crate::enums::file_change::FileChange;
use crate::enums::line_change::LineChange;
use crate::structs::analyze_repository_response::AnalyzeRepositoryResponse;
use crate::structs::config::repository_config::RepositoryConfig;
use crate::structs::performance_improvement::PerformanceImprovement;
use crate::structs::security_issue::SecurityIssue;

pub struct FileChangeLogger {}

impl FileChangeLogger {

    fn print_diff_preview(repository_config: Rc<RepositoryConfig>, file_path: &str, changes: &[LineChange]) -> Result<(), Box<dyn std::error::Error>> {
        println!("\n📄 Diff preview for {}:", file_path);
        println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");

        let full_path = format!("{}/{}", repository_config.path, file_path).replace("//", "/");
        let content = fs::read_to_string(&full_path)?;
        let lines: Vec<&str> = content.lines().collect();

        let mut sorted_changes = changes.to_vec();
        sorted_changes.sort_by_key(|change| match change {
            LineChange::Replace { line_number, .. } => *line_number,
            LineChange::InsertAfter { line_number, .. } => *line_number,
            LineChange::InsertBefore { line_number, .. } => *line_number,
            LineChange::Delete { line_number } => *line_number,
            LineChange::ReplaceRange { start_line, .. } => *start_line,
        });

        for change in &sorted_changes {
            match change {
                LineChange::Replace { line_number, old_content, new_content } => {
                    println!("\n@@ Line {} @@", line_number);
                    println!("\x1b[31m- {:<4} | {}\x1b[0m", line_number, old_content);
                    println!("\x1b[32m+ {:<4} | {}\x1b[0m", line_number, new_content);
                }
                LineChange::InsertAfter { line_number, new_content } => {
                    println!("\n@@ Insert after line {} @@", line_number);
                    if *line_number > 0 && *line_number <= lines.len() {
                        println!("  {:<4} | {}", line_number, lines[*line_number - 1]);
                    }
                    println!("\x1b[32m+ {:<4} | {}\x1b[0m", line_number + 1, new_content);
                }
                LineChange::InsertBefore { line_number, new_content } => {
                    println!("\n@@ Insert before line {} @@", line_number);
                    println!("\x1b[32m+ {:<4} | {}\x1b[0m", line_number, new_content);
                    if *line_number > 0 && *line_number <= lines.len() {
                        println!("  {:<4} | {}", line_number, lines[*line_number - 1]);
                    }
                }
                LineChange::Delete { line_number } => {
                    println!("\n@@ Delete line {} @@", line_number);
                    if *line_number > 0 && *line_number <= lines.len() {
                        println!("\x1b[31m- {:<4} | {}\x1b[0m", line_number, lines[*line_number - 1]);
                    }
                }
                LineChange::ReplaceRange { start_line, end_line, old_content, new_content } => {
                    println!("\n@@ Lines {}-{} @@", start_line, end_line);
                    // Print removed lines
                    for (i, line) in old_content.iter().enumerate() {
                        println!("\x1b[31m- {:<4} | {}\x1b[0m", start_line + i, line);
                    }
                    // Print added lines
                    for (i, line) in new_content.iter().enumerate() {
                        println!("\x1b[32m+ {:<4} | {}\x1b[0m", start_line + i, line);
                    }
                }
            }
        }

        println!("\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
        Ok(())
    }

    pub fn print_new_file_preview(file_path: &str, content: &str) {
        println!("\n📄 New file preview for {}:", file_path);
        println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");

        for (i, line) in content.lines().enumerate() {
            println!("\x1b[32m+ {:<4} | {}\x1b[0m", i + 1, line);
        }

        println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    }

    pub fn print_analysis_report(analyze_repository_response: Rc<AnalyzeRepositoryResponse>) {
        println!("🔍 CODE ANALYSIS REPORT");
        println!("======================");
        println!("{}\n", analyze_repository_response.repository_analysis.analysis_summary);
        println!("🔧 CHANGES REQUIRED ({} total):", analyze_repository_response.repository_analysis.changes.len());
    }

    pub fn print_change_report(repository_config: Rc<RepositoryConfig>, change: &FileChange) -> Result<(), Box<dyn std::error::Error>>{
        let severity = match change {
            FileChange::ModifyFile { severity, .. } => severity,
            FileChange::CreateFile { severity, .. } => severity,
            FileChange::DeleteFile { severity, .. } => severity,
        };
        match severity.as_str() {
            "critical" => FileChangeLogger::print_change_summary(repository_config, change, "\n  🚨 CRITICAL")?,
            "high" => FileChangeLogger::print_change_summary(repository_config, change, "\n  ⚠️ HIGH")?,
            "medium" => FileChangeLogger::print_change_summary(repository_config, change, "\n  📋 MEDIUM")?,
            "low" => FileChangeLogger::print_change_summary(repository_config, change, "\n  💡 LOW")?,
            _ => FileChangeLogger::print_change_summary(repository_config, change, "\n  📋 MEDIUM")?,
        }

        Ok(())
    }

    pub fn print_security_issues_report(security_issue: &SecurityIssue) {
        println!("\n🔒 SECURITY ISSUE:");
        println!("  ⚠️ {}:{} [{}]: {}", security_issue.file_path, security_issue.line_number, security_issue.severity, security_issue.issue);
        println!("      💡 {}", security_issue.recommendation);
    }

    pub fn print_performance_improvements_report(improvement: &PerformanceImprovement) {
        println!("\n⚡ PERFORMANCE IMPROVEMENT");
        println!("  🚀 {}:{}: {}", improvement.file_path, improvement.line_number, improvement.issue);
        println!("      📈 {}", improvement.impact);
    }

    pub fn print_change_summary(repository_config: Rc<RepositoryConfig>, change: &FileChange, log_message: &str) -> Result<(), Box<dyn std::error::Error>> {
        println!("{}", log_message);
        match change {
            FileChange::ModifyFile { file_path, reason, line_changes, .. } => {
                println!("    📝 {}", file_path);
                println!("    ❔  {}", reason);
                println!("    {} line changes", line_changes.len());
                FileChangeLogger::print_diff_preview(repository_config, file_path, line_changes)?;
            }
            FileChange::CreateFile { file_path, reason, .. } => {
                println!("    📁 {}: {}", file_path, reason);
            }
            FileChange::DeleteFile { file_path, reason, .. } => {
                println!("    🗑️ {}: {}", file_path, reason);
            }
        }

        Ok(())
    }
}