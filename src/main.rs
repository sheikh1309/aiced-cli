use crate::structs::cli::Cli;
use clap::{Parser};
use crate::workers::command_runner::CommandRunner;

mod structs;
mod services;
mod helpers;
mod enums;
mod logger;
mod config;
mod workers;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();
    let command_runner = CommandRunner::new();
    command_runner.run_command(cli.command).await
}