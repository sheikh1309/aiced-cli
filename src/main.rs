use crate::services::code_analyzer::CodeAnalyzer;
use std::io::{self, Write};

mod structs;
mod services;
mod helpers;
mod traits;
mod enums;
mod prompts;
mod logger;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>>  {
    let anthropic_token = std::env::var("ANTHROPIC_API_KEY").expect("ANTHROPIC_API_KEY environment variable must be set");
    let home_dir = std::env::var("HOME").unwrap_or_else(|_| "default_path".to_string());
    // todo - get from .toml file
    let repo_path = std::env::var("REPO_PATH")
        .unwrap_or_else(|_| format!("{}/Projects/creator/creator-api-websites", home_dir));
    
    println!("Analyzing project at: {}\n", repo_path);
    
    let analyzer = CodeAnalyzer::new(anthropic_token, repo_path)?;
    // todo - sometimes ai return empty
    let analysis = analyzer.analyze_repository().await?;
    analyzer.print_analysis_report(&analysis);
    // todo - can apply all changes
    println!("security_issues {:?}", &analysis.security_issues);
    println!("performance_improvements {:?}", &analysis.performance_improvements);
    
    for (_, change) in analysis.changes.iter().enumerate() {
        analyzer.print_change_report(&change)?;
    
        print!("\nApply this change? (y/N): ");
        io::stdout().flush()?;
    
        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
    
        if input.trim().to_lowercase() == "y" {
            analyzer.apply_change(&change)?;
        }
    }
    
    // todo - print security issues and performance improvements and apply them
    
    Ok(())
    
}