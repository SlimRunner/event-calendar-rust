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
        #[arg(short, long, conflicts_with = "exclude")]
        include: Vec<String>,
        #[arg(short, long)]
        exclude: Vec<String>,
    },
}
