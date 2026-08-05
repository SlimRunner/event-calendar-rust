use std::path::PathBuf;

use clap::{Parser, Subcommand};

#[derive(Parser)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    Upcoming {
        aliases: Vec<String>,
        #[arg(short, long, default_value = "false")]
        all: bool,
    },
    Calendar {
        aliases: Vec<String>,
        #[arg(short, long, default_value = "false")]
        all: bool,
    },
    List {
        aliases: Vec<String>,
        #[arg(short = 'i', long)]
        include_any: Vec<String>,
        #[arg(short = 'I', long)]
        include_all: Vec<String>,
    },
    Tags {
        aliases: Vec<String>,
        #[arg(short = 'i', long)]
        include_any: Vec<String>,
        #[arg(short = 'I', long)]
        include_all: Vec<String>,
        #[arg(short, long)]
        max_size: Option<usize>,
    },
    Config {
        #[command(subcommand)]
        command: ConfigCommands,
    },
}

impl Commands {
    pub fn aliases(&self) -> Option<&[String]> {
        match self {
            Commands::Calendar { aliases, .. }
            | Commands::Upcoming { aliases, .. }
            | Commands::List { aliases, .. }
            | Commands::Tags { aliases, .. } => Some(aliases),

            Commands::Config { .. } => None,
        }
    }
}

#[derive(Subcommand)]
pub enum ConfigCommands {
    List,
    Add { alias: String, path: PathBuf },
    Remove { alias: String },
    Rename { old: String, new: String },
    Default { alias: String },
}
