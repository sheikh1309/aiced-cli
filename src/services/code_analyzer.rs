use std::fs;
use futures::StreamExt;
use crate::structs::message::Message;
use crate::helpers::prompt_generator;
use crate::enums::file_change::FileChange;
use crate::traits::ai_provider::AiProvider;
use crate::constants::prompts::SYSTEM_PROMPT;
use crate::services::ai_providers::anthropic::AnthropicProvider;
use crate::services::repo_scanner::RepoScanner;
use crate::services::file_modifier::FileModifier;
use crate::structs::analysis_response::AnalysisResponse;

pub struct CodeAnalyzer {
    ai_provider: AnthropicProvider,
    repo_scanner: RepoScanner,
    repo_path: String,
}

impl CodeAnalyzer
{
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
            .map_err(|e| format!("Failed to write JSON file: {}", e))?;

        let messages = vec![system_prompt, user_prompt];

        let mut response_text = String::new();
        let mut stream = self.ai_provider.generate_completion_stream(&messages).await?;
        println!("🤖 Analyzing repository...");
        while let Some(result) = stream.next().await {
            println!("🤖 Analyzing repository...");
            match result {
                Ok(item) => {
                    response_text.push_str(&item.content);
                    if item.is_complete {
                        break;
                    }
                }
                Err(e) => return Err(e.into()),
            }
        }

        fs::write("analysis.txt", &response_text.replace("```json\n", "").replace("\n```", ""))
            .map_err(|e| format!("Failed to write JSON file: {}", e))?;
        
        let analysis: AnalysisResponse = serde_json::from_str(&response_text.replace("```json\n", "").replace("\n```", ""))
            .map_err(|e| format!("Failed to parse AI response as JSON: {}", e))?;

        Ok(analysis)
    }

    pub fn apply_changes(&self, analysis: &AnalysisResponse, dry_run: bool) -> Result<(), Box<dyn std::error::Error>> {
        println!("🔧 Applying {} changes...", analysis.changes.len());

        for (i, change) in analysis.changes.iter().enumerate() {
            println!("\n📋 Change {} of {}", i + 1, analysis.changes.len());

            match change {
                FileChange::ModifyFile { file_path, reason, severity, line_changes } => {
                    println!("📝 [{}] Modifying {}: {}", severity, file_path, reason);

                    if dry_run {
                        println!("   🔍 DRY RUN - Validating {} line changes", line_changes.len());
                        FileModifier::validate_file_modifications(&self.repo_path, file_path, line_changes)?;
                        println!("   ✅ All changes validated successfully");
                    } else {
                        FileModifier::apply_file_modifications(&self.repo_path, file_path, line_changes)?;
                    }
                }
                FileChange::CreateFile { file_path, reason, severity, content } => {
                    println!("📁 [{}] Creating {}: {}", severity, file_path, reason);
                    if !dry_run {
                        FileModifier::create_file(&self.repo_path, file_path, content)?;
                    }
                }
                FileChange::DeleteFile { file_path, reason, severity } => {
                    println!("🗑️ [{}] Deleting {}: {}", severity, file_path, reason);
                    if !dry_run {
                        FileModifier::delete_file(&self.repo_path, file_path)?;
                    }
                }
            }
        }

        if dry_run {
            println!("\n✅ DRY RUN COMPLETE - All changes validated successfully!");
        } else {
            println!("\n🎉 ALL CHANGES APPLIED SUCCESSFULLY!");
        }

        Ok(())
    }

    pub fn print_analysis_report(&self, analysis: &AnalysisResponse) {
        println!("🔍 CODE ANALYSIS REPORT");
        println!("======================");
        println!("{}\n", analysis.analysis_summary);

        // Group changes by severity
        let mut critical = Vec::new();
        let mut high = Vec::new();
        let mut medium = Vec::new();
        let mut low = Vec::new();

        for change in &analysis.changes {
            let severity = match change {
                FileChange::ModifyFile { severity, .. } => severity,
                FileChange::CreateFile { severity, .. } => severity,
                FileChange::DeleteFile { severity, .. } => severity,
            };

            match severity.as_str() {
                "critical" => critical.push(change),
                "high" => high.push(change),
                "medium" => medium.push(change),
                "low" => low.push(change),
                _ => medium.push(change), // Default to medium
            }
        }

        println!("🔧 CHANGES REQUIRED ({} total):", analysis.changes.len());

        if !critical.is_empty() {
            println!("\n  🚨 CRITICAL ({}):", critical.len());
            for change in critical {
                self.print_change_summary(change);
            }
        }

        if !high.is_empty() {
            println!("\n  ⚠️ HIGH ({}):", high.len());
            for change in high {
                self.print_change_summary(change);
            }
        }

        if !medium.is_empty() {
            println!("\n  📋 MEDIUM ({}):", medium.len());
            for change in medium {
                self.print_change_summary(change);
            }
        }

        if !low.is_empty() {
            println!("\n  💡 LOW ({}):", low.len());
            for change in low {
                self.print_change_summary(change);
            }
        }

        if !analysis.security_issues.is_empty() {
            println!("\n🔒 SECURITY ISSUES ({} total):", analysis.security_issues.len());
            for (i, issue) in analysis.security_issues.iter().enumerate() {
                println!("  {}. ⚠️ {}:{} [{}]: {}", i+1, issue.file_path, issue.line_number, issue.severity, issue.issue);
                println!("      💡 {}", issue.recommendation);
            }
        }

        if !analysis.performance_improvements.is_empty() {
            println!("\n⚡ PERFORMANCE IMPROVEMENTS ({} total):", analysis.performance_improvements.len());
            for (i, improvement) in analysis.performance_improvements.iter().enumerate() {
                println!("  {}. 🚀 {}:{}: {}", i+1, improvement.file_path, improvement.line_number, improvement.issue);
                println!("      📈 {}", improvement.impact);
            }
        }
    }

    fn print_change_summary(&self, change: &FileChange) {
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