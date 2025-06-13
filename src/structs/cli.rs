use clap::Parser;
use crate::enums::commands::Commands;

#[derive(Parser)]
#[clap(name = "ailyzer")]
#[clap(about = "AI-powered code analysis tool", long_about = None)]
pub struct Cli {
    #[clap(subcommand)]
    pub command: Commands,
}