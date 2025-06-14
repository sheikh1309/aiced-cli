use std::fs;
use std::rc::Rc;
use std::sync::Arc;
use futures::StreamExt;
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
use crate::structs::analyze_repository_response::AnalyzeRepositoryResponse;
use crate::structs::config::repository_config::RepositoryConfig;
use crate::structs::performance_improvement::PerformanceImprovement;
use crate::structs::security_issue::SecurityIssue;

pub struct CodeAnalyzer {
    anthropic_provider: Arc<AnthropicProvider>,
    repo_scanner: RepoScanner,
    repository_config: Arc<RepositoryConfig>,
}

impl CodeAnalyzer {

    pub fn new(api_key: String, repository_config: Arc<RepositoryConfig>) -> Result<Self, Box<dyn std::error::Error>> {
        let anthropic_provider = Arc::new(AnthropicProvider::new(api_key.clone(), Arc::new(ApiRateLimiter::new())));
        Ok(Self {
            anthropic_provider: Arc::clone(&anthropic_provider),
            repo_scanner: RepoScanner::new(anthropic_provider, Arc::clone(&repository_config)),
            repository_config,
        })
    }

    pub async fn analyze_repository(&self) -> Result<Rc<AnalyzeRepositoryResponse>, Box<dyn std::error::Error>> {
        let files = self.repo_scanner.scan_files().await?;

        let user_prompt = prompt_generator::generate_analysis_user_prompt(files, &self.repository_config.path);
        let mut logger = AnimatedLogger::new("Analyzing Repository".to_string());
        logger.start();
        let mut full_content = String::new();
        let mut input_tokens = 0u32;
        let mut output_tokens = 0u32;
        let mut stream = self.anthropic_provider.trigger_stream_request(SYSTEM_ANALYSIS_PROMPT.to_string(), vec![user_prompt]).await?;

        while let Some(result) = stream.next().await {
            match result {
                Ok(item) => {
                    if !item.content.is_empty() {
                        full_content.push_str(&item.content);
                    }

                    if let Some(input) = item.input_tokens {
                        input_tokens = input;
                    }

                    if let Some(output) = item.output_tokens {
                        output_tokens = output;
                    }

                    if item.is_complete {
                        println!("is_complete {:?}", item);
                        break;
                    }
                }
                Err(_e) => {},
            }
        }

        logger.stop("Analysis complete").await;

        println!("Input tokens: {}", input_tokens);
        println!("Output tokens: {}", output_tokens);
        fs::write("ai_output.txt", &full_content)?;
        let mut parser = Parser::new(&full_content);
        let analysis = parser.parse().map_err(|e| { format!("Failed to parse custom format: {}", e) })?;

        Ok(Rc::new(AnalyzeRepositoryResponse { repository_analysis: Rc::new(analysis), repository_config: Rc::new((*self.repository_config).clone()) }))
    }

    pub fn apply_change(&self, file_change: &FileChange) -> Result<(), Box<dyn std::error::Error>> {
        match file_change {
            FileChange::ModifyFile { file_path, reason: _, severity: _, line_changes } => {
                FileModifier::validate_file_modifications(&self.repository_config.path, file_path, line_changes)?;
                FileModifier::apply_file_modifications(&self.repository_config.path, file_path, line_changes)?;
            }
            FileChange::CreateFile { file_path, reason: _, severity: _, content } => {
                self.print_new_file_preview(file_path, content);
                FileModifier::create_file(&self.repository_config.path, file_path, content)?;
            }
            FileChange::DeleteFile { file_path, reason: _, severity: _ } => {
                FileModifier::delete_file(&self.repository_config.path, file_path)?;
            }
        }
        Ok(())
    }

    fn print_diff_preview(&self, file_path: &str, changes: &[LineChange]) -> Result<(), Box<dyn std::error::Error>> {
        println!("\n📄 Diff preview for {}:", file_path);
        println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");

        // Load the original file content
        let full_path = format!("{}/{}", self.repository_config.path, file_path).replace("//", "/");
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

    pub fn print_analysis_report(&self, analyze_repository_response: Rc<AnalyzeRepositoryResponse>) {
        println!("🔍 CODE ANALYSIS REPORT");
        println!("======================");
        println!("{}\n", analyze_repository_response.repository_analysis.analysis_summary);
        println!("🔧 CHANGES REQUIRED ({} total):", analyze_repository_response.repository_analysis.changes.len());
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