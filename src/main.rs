mod calendar_cli;
mod calendar_core;
mod config;

use crate::calendar_core::query::{DatabaseError, EventDatabase};
use crate::config::parser::ConfigError;
use crate::{
    calendar_cli::{
        arg_parser::{Cli, Commands},
        commands::{list_filtered_by_tags, list_upcoming, show_weekly_calendar, tag_summary},
    },
    config::parser::Config,
};
use clap::Parser;
use confy::ConfyError;

#[derive(thiserror::Error, Debug)]
enum ProgramError {
    #[error(transparent)]
    Database(#[from] DatabaseError),

    #[error(transparent)]
    Confy(#[from] ConfyError),

    #[error(transparent)]
    Config(#[from] ConfigError),
}

fn main() -> Result<(), ProgramError> {
    let mut config = Config::load().map_err(ProgramError::Confy)?;
    let cli = Cli::parse();
    let dbs = if let Some(aliases) = cli.command.aliases()
        && !aliases.is_empty()
    {
        let mut out = Vec::new();
        for res in config
            .resolve_aliases(aliases)
            .map_err(ProgramError::Config)?
        {
            out.push(EventDatabase::new(res.path, &res.alias).map_err(ProgramError::Database)?);
        }
        out
    } else {
        if let Some(default_alias) = config.has_default() {
            let title = default_alias.alias;
            let path = default_alias.path;
            Vec::from(vec![EventDatabase::new(path, &title)?])
        } else {
            Vec::new()
        }
    };

    match cli.command {
        Commands::Config { command } => {
            config.handle_config(command)?;
        }

        command => match command {
            Commands::Calendar { aliases: _, all } => {
                dbs.iter().for_each(|db| show_weekly_calendar(&db, all))
            }
            Commands::Upcoming { aliases: _, all } => {
                dbs.iter().for_each(|db| list_upcoming(&db, all))
            }
            Commands::List {
                aliases: _,
                include_any,
                include_all,
            } => dbs
                .iter()
                .for_each(|db| list_filtered_by_tags(&db, &include_any, &include_all)),
            Commands::Tags {
                aliases: _,
                include_any,
                include_all,
                max_size,
            } => dbs
                .iter()
                .for_each(|db| tag_summary(&db, &include_any, &include_all, max_size)),

            Commands::Config { command: _ } => unreachable!(),
        },
    }

    Ok(())
}
