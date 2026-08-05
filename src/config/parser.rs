use confy::ConfyError;
use core::fmt;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, path::PathBuf};
use thiserror::Error;

use crate::calendar_cli::arg_parser::ConfigCommands;

#[derive(Error, Debug)]
pub enum ConfigError {
    MissingAlias(String),
    #[error(transparent)]
    Confy(#[from] ConfyError),
}

impl fmt::Display for ConfigError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ConfigError::MissingAlias(err) => write!(f, "The item '{err}' was not found."),
            ConfigError::Confy(err) => write!(f, "{err}"),
        }
    }
}

pub enum ConfigDefaults {
    ProgConfigName,
    CalConfigName,
}

impl ConfigDefaults {
    fn get_path(&self) -> &str {
        match self {
            ConfigDefaults::CalConfigName => "calendars",
            ConfigDefaults::ProgConfigName => "config",
        }
    }
}

static GLOBAL_CONFIG_NAME: ConfigDefaults = ConfigDefaults::ProgConfigName;
static CALENDAR_CONFIG_NAME: ConfigDefaults = ConfigDefaults::CalConfigName;

#[derive(Debug)]
pub struct Config {
    program: ProgConfig,
    calendar: CalConfig,
}

pub struct ResolvedCalendar {
    pub alias: String,
    pub path: PathBuf,
}

impl Config {
    pub fn load() -> Result<Self, ConfyError> {
        let prog_config =
            confy::load::<ProgConfig>(clap::crate_name!(), GLOBAL_CONFIG_NAME.get_path())?;
        let cal_config =
            confy::load::<CalConfig>(clap::crate_name!(), CALENDAR_CONFIG_NAME.get_path())?;
        Ok(Config {
            program: prog_config,
            calendar: cal_config,
        })
    }

    pub fn has_default<'a>(&'a self) -> Option<ResolvedCalendar> {
        let alias = self.program.default_alias.clone()?;
        if let Some(entry) = self.calendar.calendars.get(&alias) {
            Some(ResolvedCalendar {
                path: entry.path.to_path_buf(),
                alias,
            })
        } else {
            None
        }
    }

    pub fn resolve_aliases(
        &self,
        aliases: &[String],
    ) -> Result<Vec<ResolvedCalendar>, ConfigError> {
        let mut out = Vec::new();

        for alias in aliases {
            let entry = self
                .calendar
                .calendars
                .get(alias)
                .ok_or(ConfigError::MissingAlias(alias.clone()))?;

            out.push(ResolvedCalendar {
                alias: alias.clone(),
                path: entry.path.clone(),
            });
        }

        Ok(out)
    }

    pub fn handle_config(&mut self, command: ConfigCommands) -> Result<(), ConfigError> {
        match command {
            ConfigCommands::Add { alias, path } => self.add_calendar(&alias, &path),
            ConfigCommands::Default { alias } => self.set_default(&alias),
            ConfigCommands::List => self.list_aliases(),
            ConfigCommands::Remove { alias } => self.remove_alias(&alias),
            ConfigCommands::Rename { old, new } => self.rename_alias(&old, &new),
        }
    }

    fn add_calendar(&mut self, alias: &str, path: &PathBuf) -> Result<(), ConfigError> {
        self.calendar.calendars.insert(
            alias.to_string(),
            CalendarEntry {
                path: path.to_path_buf(),
            },
        );
        self.save()
    }

    fn rename_alias(&mut self, old: &str, new: &str) -> Result<(), ConfigError> {
        let old_value = self
            .calendar
            .calendars
            .remove(old)
            .ok_or(ConfigError::MissingAlias(old.to_string()))?;
        if let Some(def) = &self.program.default_alias
            && old == def
        {
            self.program.default_alias = None;
        }
        self.calendar.calendars.insert(
            new.to_string(),
            CalendarEntry {
                path: old_value.path.to_path_buf(),
            },
        );
        self.save()
    }

    fn remove_alias(&mut self, alias: &str) -> Result<(), ConfigError> {
        self.calendar.calendars.remove(alias);
        self.save()
    }

    fn set_default(&mut self, alias: &str) -> Result<(), ConfigError> {
        self.calendar
            .calendars
            .get(alias)
            .ok_or(ConfigError::MissingAlias(alias.to_string()))?;
        self.program.default_alias = Some(alias.to_string());
        self.save()
    }

    fn list_aliases(&self) -> Result<(), ConfigError> {
        if let Some(alias) = &self.program.default_alias {
            println!("DEFAULT: '{}'", alias);
        } else {
            println!("DEFAULT: none");
        }

        for (alias, value) in &self.calendar.calendars {
            println!("{}: {}", alias, value.path.to_string_lossy());
        }

        Ok(())
    }

    fn save(&self) -> Result<(), ConfigError> {
        confy::store(
            clap::crate_name!(),
            GLOBAL_CONFIG_NAME.get_path(),
            &self.program,
        )?;
        confy::store(
            clap::crate_name!(),
            CALENDAR_CONFIG_NAME.get_path(),
            &self.calendar,
        )?;

        Ok(())
    }
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct ProgConfig {
    #[serde(default)]
    pub default_alias: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct CalendarEntry {
    pub path: PathBuf,
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct CalConfig {
    #[serde(default)]
    pub calendars: HashMap<String, CalendarEntry>,
}
