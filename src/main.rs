use crate::services::code_analyzer::CodeAnalyzer;
use std::io::{self, Write};

mod structs;
mod services;
mod helpers;
mod traits;
mod enums;
mod constants;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>>  {
    let anthropic_token: String = String::from("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");
    let home_dir = std::env::var("HOME").unwrap_or_else(|_| "default_path".to_string());
    let repo_path = format!("{}/Projects/creator/creator-api-websites", home_dir);
    println!("Analyzing project at: {}\n", repo_path);

    let analyzer = CodeAnalyzer::new(anthropic_token, repo_path)?;

    let analysis = analyzer.analyze_repository().await?;
    analyzer.print_analysis_report(&analysis);

    print!("\nApply these changes? (y/N): ");
    io::stdout().flush()?;
    let mut input = String::new();
    io::stdin().read_line(&mut input)?;

    if input.trim().to_lowercase() == "y" {
        analyzer.apply_changes(&analysis, false)?;
        println!("✅ Changes applied successfully!");
    } else {
        println!("📋 Dry run - no changes made.");
        analyzer.apply_changes(&analysis, true)?;
    }


    Ok(())
}