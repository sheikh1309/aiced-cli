use clap::Subcommand;

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
        #[clap(short, long, default_value = "8080")]
        port: u16,
    },
    Validate,
    History {
        repo: Option<String>,
        #[clap(short, long, default_value = "7")]
        days: u32,
    },
}