use std::fs;
use std::sync::Arc;
use std::time::{Duration, Instant};
use futures::StreamExt;
use crate::structs::message::Message;
use crate::helpers::prompt_generator;
use crate::enums::file_change::FileChange;
use crate::enums::line_change::LineChange;
use crate::prompts::system_analysis_prompt::SYSTEM_ANALYSIS_PROMPT;
use crate::logger::animated_logger::AnimatedLogger;
use crate::services::anthropic::AnthropicProvider;
use crate::services::custom_parser::Parser;
use crate::services::repo_scanner::RepoScanner;
use crate::services::file_modifier::FileModifier;
use crate::services::rate_limiter::ApiRateLimiter;
use crate::structs::analysis_response::AnalysisResponse;
use crate::structs::config::config::Config;
use crate::structs::performance_improvement::PerformanceImprovement;
use crate::structs::security_issue::SecurityIssue;

pub struct CodeAnalyzer {
    anthropic_provider: Arc<AnthropicProvider>,
    repo_scanner: RepoScanner,
    repo_path: String,
}

impl CodeAnalyzer {
    
    pub fn new(api_key: String, repo_path: String, config: &Config) -> Result<Self, Box<dyn std::error::Error>> {
        let anthropic_provider = Arc::new(AnthropicProvider::new(api_key.clone(), Arc::new(ApiRateLimiter::new())));
        Ok(Self {
            anthropic_provider: anthropic_provider.clone(),
            repo_scanner: RepoScanner::new(anthropic_provider, repo_path.clone(), config),
            repo_path,
        })
    }

    pub async fn analyze_repository(&self) -> Result<AnalysisResponse, Box<dyn std::error::Error>> {
        let files = self.repo_scanner.scan_files_async().await?;

        let system_prompt = Message {
            role: "system".to_string(),
            content: SYSTEM_ANALYSIS_PROMPT.to_string(),
        };

        let user_prompt = Message {
            role: "user".to_string(),
            content: prompt_generator::generate_analysis_user_prompt(files, &self.repo_path),
        };

        let messages = vec![system_prompt, user_prompt];

        let mut logger = AnimatedLogger::new("Analyzing Repository".to_string());

        logger.start();

        let mut response_text = String::new();
        let mut last_update = Instant::now();
        let update_interval = Duration::from_millis(150);
        let mut stream = self.anthropic_provider.trigger_stream_request(&messages).await?;

        while let Some(result) = stream.next().await {
            if last_update.elapsed() >= update_interval {
                last_update = Instant::now();
            }

            match result {
                Ok(item) => {
                    response_text.push_str(&item.content);
                },
                Err(_e) => {},
            }
        }

        logger.stop("Analysis complete").await;

        let mut parser = Parser::new(&response_text);
        let analysis = parser.parse()
            .map_err(|e| {
                format!("Failed to parse custom format: {}", e)
            })?;


        Ok(analysis)
    }

    pub fn apply_change(&self, file_change: &FileChange) -> Result<(), Box<dyn std::error::Error>> {
        match file_change {
            FileChange::ModifyFile { file_path, reason: _, severity: _, line_changes } => {
                FileModifier::validate_file_modifications(&self.repo_path, file_path, line_changes)?;
                FileModifier::apply_file_modifications(&self.repo_path, file_path, line_changes)?;
            }
            FileChange::CreateFile { file_path, reason: _, severity: _, content } => {
                self.print_new_file_preview(file_path, content);
                FileModifier::create_file(&self.repo_path, file_path, content)?;
            }
            FileChange::DeleteFile { file_path, reason: _, severity: _ } => {
                FileModifier::delete_file(&self.repo_path, file_path)?;
            }
        }
        Ok(())
    }

    fn print_diff_preview(&self, file_path: &str, changes: &[LineChange]) -> Result<(), Box<dyn std::error::Error>> {
        println!("\n📄 Diff preview for {}:", file_path);
        println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");

        // Load the original file content
        let full_path = format!("{}/{}", self.repo_path, file_path).replace("//", "/");
        let content = fs::read_to_string(&full_path)?;
        let lines: Vec<&str> = content.lines().collect();

        // Sort changes by line number to display them in order
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

    fn print_new_file_preview(&self, file_path: &str, content: &str) {
        println!("\n📄 New file preview for {}:", file_path);
        println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");

        for (i, line) in content.lines().enumerate() {
            println!("\x1b[32m+ {:<4} | {}\x1b[0m", i + 1, line);
        }

        println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    }

    pub fn print_analysis_report(&self, analysis: &AnalysisResponse) {
        println!("🔍 CODE ANALYSIS REPORT");
        println!("======================");
        println!("{}\n", analysis.analysis_summary);
        println!("🔧 CHANGES REQUIRED ({} total):", analysis.changes.len());
    }

    pub fn print_change_report(&self, change: &FileChange) -> Result<(), Box<dyn std::error::Error>>{
        let severity = match change {
            FileChange::ModifyFile { severity, .. } => severity,
            FileChange::CreateFile { severity, .. } => severity,
            FileChange::DeleteFile { severity, .. } => severity,
        };
        match severity.as_str() {
            "critical" => self.print_change_summary(change, "\n  🚨 CRITICAL")?,
            "high" => self.print_change_summary(change, "\n  ⚠️ HIGH")?,
            "medium" => self.print_change_summary(change, "\n  📋 MEDIUM")?,
            "low" => self.print_change_summary(change, "\n  💡 LOW")?,
            _ => self.print_change_summary(change, "\n  📋 MEDIUM")?,
        }
        
        Ok(())
    }

    pub fn print_security_issues_report(&self, security_issue: &SecurityIssue) {
        println!("\n🔒 SECURITY ISSUE:");
        println!("  ⚠️ {}:{} [{}]: {}", security_issue.file_path, security_issue.line_number, security_issue.severity, security_issue.issue);
        println!("      💡 {}", security_issue.recommendation);
    }

    pub fn print_performance_improvements_report(&self, improvement: &PerformanceImprovement) {
        println!("\n⚡ PERFORMANCE IMPROVEMENT");
        println!("  🚀 {}:{}: {}", improvement.file_path, improvement.line_number, improvement.issue);
        println!("      📈 {}", improvement.impact);
    }

    fn print_change_summary(&self, change: &FileChange, log_message: &str) -> Result<(), Box<dyn std::error::Error>> {
        println!("{}", log_message);
        match change {
            FileChange::ModifyFile { file_path, reason, line_changes, .. } => {
                println!("    📝 {}", file_path);
                println!("    ❔  {}", reason);
                println!("    {} line changes", line_changes.len());
                self.print_diff_preview(file_path, line_changes)?;
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