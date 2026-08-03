mod event_cli;
mod event_core;

use clap::Parser;
use event_cli::{
    arg_parser::{Cli, Commands},
    commands::{list_upcoming, show_weekly_calendar},
};
use event_core::query::{DatabaseError, EventDatabase};

use crate::event_cli::commands::{list_filtered_by_tags, tag_summary};

fn main() -> Result<(), DatabaseError> {
    let path = "E:/dev/personal-schedules/entertainment/seasonal-anime.yaml";

    let db = EventDatabase::new(path)?;
    let cli = Cli::parse();

    match cli.command {
        Commands::Calendar { all } => show_weekly_calendar(&db, all),
        Commands::List {
            include_any,
            include_all,
        } => list_filtered_by_tags(&db, &include_any, &include_all),
        Commands::Upcoming { all } => list_upcoming(&db, all),
        Commands::Tags {
            include_any,
            include_all,
            max_size,
        } => tag_summary(&db, &include_any, &include_all, max_size),
    }

    Ok(())
}
