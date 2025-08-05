use clap::Subcommand;
use crate::config::constants::{DEFAULT_DASHBOARD_PORT, DEFAULT_HISTORY_DAYS};

#[derive(Subcommand)]
pub enum Commands {
    Init,
    Analyze {
        #[clap(short, long)]
        repo: Option<String>,
        #[clap(short, long)]
        tags: Vec<String>,
        #[clap(short, long)]
        profile: Option<String>,
    },
    List,
    Dashboard {
        #[clap(short, long, default_value_t = DEFAULT_DASHBOARD_PORT)]
        port: u16,
    },
    Validate,
    History {
        repo: Option<String>,
        #[clap(short, long, default_value_t = DEFAULT_HISTORY_DAYS)]
        days: u32,
    },
}