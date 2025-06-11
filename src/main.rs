use crate::services::code_analyzer::CodeAnalyzer;
use std::io::{self, Write};

mod structs;
mod services;
mod helpers;
mod traits;
mod enums;
mod constants;
mod logger;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>>  {
    let anthropic_token = std::env::var("ANTHROPIC_API_KEY").expect("ANTHROPIC_API_KEY environment variable must be set");
    let home_dir = std::env::var("HOME").unwrap_or_else(|_| "default_path".to_string());
    // let repo_path = format!("{}/Projects/creator/creator-api-websites", home_dir);
    let repo_path = format!("{}/Projects/rust/codesentry", home_dir);
    println!("Analyzing project at: {}\n", repo_path);

    let analyzer = CodeAnalyzer::new(anthropic_token, repo_path)?;

    let analysis = analyzer.analyze_repository().await?;
    analyzer.print_analysis_report(&analysis);
    // todo - can apply all changes

    for (i, change) in analysis.changes.iter().enumerate() {
        analyzer.print_change_report(&change);

        print!("\nApply these change? (y/N): ");
        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;

        if input.trim().to_lowercase() == "y" {
            println!("\n📋 Change {} of {}", i + 1, analysis.changes.len());
            analyzer.apply_change(&change)?;
            println!("\n🎉 CHANGE APPLIED SUCCESSFULLY!");
        } else {
            println!("📋 no changes made.");
        }
    }

    // todo - print security issues and performance improvements and apply them

    Ok(())
}