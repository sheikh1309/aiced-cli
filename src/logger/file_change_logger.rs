use std::{cmp, fs};
use std::rc::Rc;
use terminal_size::{Width, terminal_size};
use crate::enums::file_change::FileChange;
use crate::enums::line_change::LineChange;
use crate::structs::analyze_repository_response::AnalyzeRepositoryResponse;
use crate::structs::config::repository_config::RepositoryConfig;

pub struct FileChangeLogger {}

impl FileChangeLogger {

    fn truncate_line(line: &str, max_width: usize) -> String {
        if line.len() <= max_width {
            line.to_string()
        } else if max_width > 3 {
            format!("{}...", &line[..max_width - 3])
        } else {
            "...".to_string()
        }
    }

    fn print_diff_preview(repository_config: Rc<RepositoryConfig>, file_path: &str, changes: &[LineChange]) -> Result<(), Box<dyn std::error::Error>> {
        println!("\n🔥 Diff preview for {}:", file_path);

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

        // Calculate optimal column widths based on actual terminal size
        let line_number_width = 4;
        let separator_width = 3; // " | "
        let action_width = 20; // Space for action descriptions
        let min_content_width = 30;

        // Get actual terminal width, fallback to reasonable default
        let terminal_width = if let Some((Width(w), _)) = terminal_size() {
            w as usize
        } else {
            120 // Fallback width
        };
        let available_width = terminal_width - 6; // Account for borders

        // Split available width between before and after columns
        let total_column_overhead = (line_number_width + separator_width) * 2 + action_width + 6; // borders
        let content_width = (available_width - total_column_overhead) / 2;
        let column_width = cmp::max(min_content_width, content_width);

        let before_header = format!("🔍 BEFORE ({})", file_path);
        let after_header = format!("🚀 AFTER");

        let section_width = line_number_width + separator_width + column_width;

        println!("┌{:─^width$}┬{:─^width$}┬{:─^20}┐",
                 &before_header, &after_header, "ACTION", width = section_width);

        for change in &sorted_changes {
            match change {
                LineChange::Replace { line_number, old_content, new_content } => {
                    let old_truncated = Self::truncate_line(old_content, column_width);
                    let new_truncated = Self::truncate_line(new_content, column_width);

                    println!("│ {:>4} │ {:<width$} │ {:>4} │ {:<width$} │ {:^18} │",
                             line_number,
                             old_truncated,
                             line_number,
                             new_truncated,
                             "🔄 MODIFIED",
                             width = column_width
                    );
                }
                LineChange::InsertAfter { line_number, new_content } => {
                    let prev_line = if *line_number > 0 && *line_number <= lines.len() {
                        Self::truncate_line(lines[*line_number - 1], column_width)
                    } else {
                        "".to_string()
                    };
                    let new_truncated = Self::truncate_line(new_content, column_width);

                    println!("│ {:>4} │ {:<width$} │ {:>4} │ {:<width$} │ {:^18} │",
                             line_number,
                             prev_line,
                             line_number + 1,
                             new_truncated,
                             "➕ INSERT AFTER",
                             width = column_width
                    );
                }
                LineChange::InsertBefore { line_number, new_content } => {
                    let curr_line = if *line_number > 0 && *line_number <= lines.len() {
                        Self::truncate_line(lines[*line_number - 1], column_width)
                    } else {
                        "".to_string()
                    };
                    let new_truncated = Self::truncate_line(new_content, column_width);

                    println!("│ {:>4} │ {:<width$} │ {:>4} │ {:<width$} │ {:^18} │",
                             line_number,
                             curr_line,
                             line_number,
                             new_truncated,
                             "⬆️ INSERT BEFORE",
                             width = column_width
                    );
                }
                LineChange::Delete { line_number } => {
                    let old_line = if *line_number > 0 && *line_number <= lines.len() {
                        Self::truncate_line(lines[*line_number - 1], column_width)
                    } else {
                        "".to_string()
                    };

                    println!("│ {:>4} │ {:<width$} │ {:>4} │ {:<width$} │ {:^18} │",
                             line_number,
                             old_line,
                             "",
                             "",
                             "🗑️ DELETED",
                             width = column_width
                    );
                }
                LineChange::ReplaceRange { start_line, old_content, new_content, .. } => {
                    let max_lines = old_content.len().max(new_content.len());
                    for i in 0..max_lines {
                        let old = if i < old_content.len() {
                            Self::truncate_line(&old_content[i], column_width)
                        } else {
                            "".to_string()
                        };
                        let new = if i < new_content.len() {
                            Self::truncate_line(&new_content[i], column_width)
                        } else {
                            "".to_string()
                        };
                        let action = if i == 0 { "💥 BLOCK UPDATE" } else { "⚡ ..." };

                        println!("│ {:>4} │ {:<width$} │ {:>4} │ {:<width$} │ {:^18} │",
                                 start_line + i,
                                 old,
                                 start_line + i,
                                 new,
                                 action,
                                 width = column_width
                        );
                    }
                }
            }
        }

        println!("└{:─<width$}┴{:─<width$}┴{:─<20}┘",
                 "", "", "", width = section_width);

        Ok(())
    }

    pub fn print_new_file_preview(file_path: &str, content: &str) {
        println!("\n✨ New file preview for {}:", file_path);

        let max_width = 100; // Configurable max width
        println!("┌{:─^width$}┐", "🆕 NEW FILE", width = max_width);

        for (i, line) in content.lines().enumerate() {
            let truncated_line = Self::truncate_line(line, max_width - 10);
            println!("│\x1b[32m➕ {:>4} │ {:<width$}\x1b[0m│",
                     i + 1,
                     truncated_line,
                     width = max_width - 10);
        }

        println!("└{:─<width$}┘", "", width = max_width);
    }

    pub fn print_analysis_report(analyze_repository_response: Rc<AnalyzeRepositoryResponse>) {
        println!("\n🚀 CODE ANALYSIS REPORT");
        println!("═══════════════════════");
        println!("{}\n", analyze_repository_response.repository_analysis.analysis_summary);
        println!("🔧 CHANGES REQUIRED ({} total):", analyze_repository_response.repository_analysis.changes.len());
        println!("🔥 {}", "─".repeat(50));
    }

    pub fn print_change_summary(repository_config: Rc<RepositoryConfig>, change: &FileChange) -> Result<(), Box<dyn std::error::Error>> {
        match change {
            FileChange::ModifyFile { file_path, reason, line_changes, .. } => {
                println!("\n🔧 MODIFYING: {} - {}", file_path, reason);
                FileChangeLogger::print_diff_preview(repository_config, file_path, line_changes)?;
            }
            FileChange::CreateFile { file_path, reason, .. } => {
                println!("\n✨ CREATING: {} - {}", file_path, reason);
            }
            FileChange::DeleteFile { file_path, reason, .. } => {
                println!("\n💥 DELETING: {} - {}", file_path, reason);
            }
        }

        Ok(())
    }
}