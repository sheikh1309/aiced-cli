use crate::structs::cli::Cli;
use clap::{Parser};
use crate::workers::command_runner::CommandRunner;

mod structs;
mod services;
mod helpers;
mod enums;
mod prompts;
mod logger;
mod config;
mod workers;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();
    CommandRunner::run_command(cli.command).await
}