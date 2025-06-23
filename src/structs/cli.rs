use clap::Parser;
use crate::enums::commands::Commands;

#[derive(Parser)]
#[clap(name = "aiced")]
#[clap(about = "AI-powered code analysis tool", long_about = None)]
pub struct Cli {
    #[clap(subcommand)]
    pub command: Commands,
}