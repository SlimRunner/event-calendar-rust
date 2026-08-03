mod event_cli;
mod event_core;

use clap::Parser;
use event_cli::{
    arg_parser::{Cli, Commands},
    commands::{list_upcoming, show_weekly_calendar},
};
use event_core::query::{DatabaseError, EventDatabase};

use crate::event_cli::commands::{list_all, list_filtered_by_tags};

fn main() -> Result<(), DatabaseError> {
    let path = "E:/dev/personal-schedules/entertainment/seasonal-anime.yaml";

    let db = EventDatabase::new(path)?;
    let cli = Cli::parse();

    match cli.command {
        Commands::Calendar { all } => show_weekly_calendar(&db, all),
        Commands::List { exclude, include } => match (exclude.is_empty(), include.is_empty()) {
            (false, true) => list_filtered_by_tags(&db, &exclude, false),
            (true, false) => list_filtered_by_tags(&db, &include, true),
            _ => list_all(&db),
        },
        Commands::Upcoming { all } => list_upcoming(&db, all),
    }

    Ok(())
}
