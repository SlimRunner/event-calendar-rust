mod event_cli;
mod event_core;

use clap::Parser;
use event_cli::arg_parser::{Cli, Commands};
use event_core::query::EventDatabase;

use crate::{
    event_cli::commands::{list_upcoming, show_weekly_calendar},
    event_core::query::DatabaseError,
};

fn main() -> Result<(), DatabaseError> {
    let path = "E:/dev/personal-schedules/entertainment/seasonal-anime.yaml";

    let db = EventDatabase::new(path)?;
    let cli = Cli::parse();

    match cli.command {
        Commands::Calendar => show_weekly_calendar(&db),
        Commands::List { all: _ } => println!("list"),
        Commands::Upcoming => list_upcoming(&db),
    }

    Ok(())
}
