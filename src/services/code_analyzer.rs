use std::fs;
use std::time::{Duration, Instant};
use futures::StreamExt;
use crate::structs::message::Message;
use crate::helpers::prompt_generator;
use crate::enums::file_change::FileChange;
use crate::traits::ai_provider::AiProvider;
use crate::constants::prompts::SYSTEM_PROMPT;
use crate::logger::animated_logger::AnimatedLogger;
use crate::services::ai_providers::anthropic::AnthropicProvider;
use crate::services::custom_parser::Parser;
use crate::services::repo_scanner::RepoScanner;
use crate::services::file_modifier::FileModifier;
use crate::structs::analysis_response::AnalysisResponse;
use crate::structs::performance_improvement::PerformanceImprovement;
use crate::structs::security_issue::SecurityIssue;

pub struct CodeAnalyzer {
    ai_provider: AnthropicProvider,
    repo_scanner: RepoScanner,
    repo_path: String,
}

impl CodeAnalyzer {
    pub fn new( api_key: String, repo_path: String) -> Result<Self, Box<dyn std::error::Error>> {
        Ok(Self {
            ai_provider: AnthropicProvider::new(api_key),
            repo_scanner: RepoScanner::new(repo_path.clone()),
            repo_path,
        })
    }

    pub async fn analyze_repository(&self) -> Result<AnalysisResponse, Box<dyn std::error::Error>> {
        let files = self.repo_scanner.scan_files();

        let system_prompt = Message {
            role: "system".to_string(),
            content: SYSTEM_PROMPT.to_string(),
        };

        let user_prompt = Message {
            role: "user".to_string(),
            content: prompt_generator::generate_prompt(files, &self.repo_path),
        };

        fs::write("prompt.txt", &user_prompt.content)
            .map_err(|e| format!("Failed to write prompt to file: {}", e))?;

        let messages = vec![system_prompt, user_prompt];

        let mut logger = AnimatedLogger::new("Analyzing Repository".to_string());

        logger.start();

        let mut response_text = String::new();
        let mut last_update = Instant::now();
        let update_interval = Duration::from_millis(150);
        let mut stream = self.ai_provider.create_stream_request(&messages).await?;

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
        fs::write("response_text.txt", &response_text).map_err(|e| format!("Failed to write Response file: {}", e))?;

        let mut parser = Parser::new(&response_text);
        let analysis = parser.parse()
            .map_err(|e| {
                fs::write("failed_response.txt", &response_text).ok();
                format!("Failed to parse custom format: {}", e)
            })?;


        Ok(analysis)
    }

    pub fn apply_change(&self, file_change: &FileChange) -> Result<(), Box<dyn std::error::Error>> {
        match file_change {
            FileChange::ModifyFile { file_path, reason, severity, line_changes } => {
                println!("📝 [{}] Modifying {}: {}", severity, file_path, reason);
                println!("   🔍 Validating {} line changes", line_changes.len());
                FileModifier::validate_file_modifications(&self.repo_path, file_path, line_changes)?;
                println!("   ✅ All changes validated successfully");
                FileModifier::apply_file_modifications(&self.repo_path, file_path, line_changes)?;
            }
            FileChange::CreateFile { file_path, reason, severity, content } => {
                println!("📁 [{}] Creating {}: {}", severity, file_path, reason);
                FileModifier::create_file(&self.repo_path, file_path, content)?;
            }
            FileChange::DeleteFile { file_path, reason, severity } => {
                println!("🗑️ [{}] Deleting {}: {}", severity, file_path, reason);
                FileModifier::delete_file(&self.repo_path, file_path)?;
            }
        }
        Ok(())
    }

    pub fn print_analysis_report(&self, analysis: &AnalysisResponse) {
        println!("🔍 CODE ANALYSIS REPORT");
        println!("======================");
        println!("{}\n", analysis.analysis_summary);
        println!("🔧 CHANGES REQUIRED ({} total):", analysis.changes.len());
    }

    pub fn print_change_report(&self, change: &FileChange) {
        let severity = match change {
            FileChange::ModifyFile { severity, .. } => severity,
            FileChange::CreateFile { severity, .. } => severity,
            FileChange::DeleteFile { severity, .. } => severity,
        };
        match severity.as_str() {
            "critical" => self.print_change_summary(change, "\n  🚨 CRITICAL"),
            "high" => self.print_change_summary(change, "\n  ⚠️ HIGH"),
            "medium" => self.print_change_summary(change, "\n  📋 MEDIUM"),
            "low" => self.print_change_summary(change, "\n  💡 LOW"),
            _ => self.print_change_summary(change, "\n  📋 MEDIUM"),
        }
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

    fn print_change_summary(&self, change: &FileChange, log_message: &str) {
        println!("{}", log_message);
        match change {
            FileChange::ModifyFile { file_path, reason, line_changes, .. } => {
                println!("    📝 {}: {}", file_path, reason);
                println!("        {} line changes", line_changes.len());
            }
            FileChange::CreateFile { file_path, reason, .. } => {
                println!("    📁 {}: {}", file_path, reason);
            }
            FileChange::DeleteFile { file_path, reason, .. } => {
                println!("    🗑️ {}: {}", file_path, reason);
            }
        }
    }

}