use clap::{Parser, Subcommand};

#[derive(Parser)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    Upcoming {
        #[arg(short, long, default_value = "false")]
        all: bool,
    },
    Calendar {
        #[arg(short, long, default_value = "false")]
        all: bool,
    },
    List {
        #[arg(short = 'i', long)]
        include_any: Vec<String>,
        #[arg(short = 'I', long)]
        include_all: Vec<String>,
    },
}
