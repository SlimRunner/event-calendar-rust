mod event_cli;
mod event_core;

use clap::Parser;
use event_cli::{
    arg_parser::{Cli, Commands},
    commands::{list_upcoming, show_weekly_calendar},
};
use event_core::query::{DatabaseError, EventDatabase};

fn main() -> Result<(), DatabaseError> {
    let path = "E:/dev/personal-schedules/entertainment/seasonal-anime.yaml";

    let db = EventDatabase::new(path)?;
    let cli = Cli::parse();

    match cli.command {
        Commands::Calendar{all} => show_weekly_calendar(&db, all),
        Commands::List { all: _ } => println!("list"),
        Commands::Upcoming{all} => list_upcoming(&db, all),
    }

    Ok(())
}
