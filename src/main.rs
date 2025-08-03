//! Aiced - AI-powered code analysis tool

use clap::Parser;
use std::process;
use crate::errors::{AicedResult, ErrorHandler, ErrorSeverity};
use crate::structs::cli::Cli;
use crate::workers::command_runner::CommandRunner;

mod structs;
mod services;
mod helpers;
mod enums;
mod logger;
mod config;
mod workers;
mod errors;
mod adapters;
mod ui;
mod prompts;
mod traits;

#[tokio::main]
async fn main() {
    env_logger::Builder::from_default_env()
        .filter_level(log::LevelFilter::Info)
        .init();

    // Run the actual application
    if let Err(e) = run().await {
        ErrorHandler::handle_error(&e);

        // Exit with appropriate code based on error severity
        let exit_code = match e.severity() {
            ErrorSeverity::Critical => 1,
            ErrorSeverity::High => 2,
            ErrorSeverity::Medium => 3,
            ErrorSeverity::Low => 4,
        };

        process::exit(exit_code);
    }
}

async fn run() -> AicedResult<()> {
    log::info!("Starting aiced...");

    let cli = Cli::parse();
    let mut command_runner = CommandRunner::new();

    command_runner.run_command(cli.command).await?;

    log::info!("Command completed successfully");
    Ok(())
}
