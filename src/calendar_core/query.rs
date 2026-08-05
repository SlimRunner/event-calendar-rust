use core::fmt;
use std::{
    fs::{self},
    path::Path,
};
use thiserror::Error;
use time::OffsetDateTime;

use super::{
    occurrence::duration_from_interval,
    parser::{Event, EventsFile, Schedule},
};

pub struct LeanCalendarEvent<'a> {
    pub event: &'a Event,
    pub start_date: OffsetDateTime,
}

pub struct CalendarEvent<'a> {
    pub event: &'a Event,
    pub index: usize,
    pub from: OffsetDateTime,
    pub to: Option<OffsetDateTime>,
}

pub struct EventDatabase {
    data: EventsFile,
    pub title: String,
}

#[derive(Error, Debug)]
pub enum DatabaseError {
    #[allow(dead_code)]
    Io(std::io::Error),
    #[allow(dead_code)]
    Parse(yaml_serde::Error),
}

impl fmt::Display for DatabaseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DatabaseError::Io(e) => write!(f, "{e}"),
            DatabaseError::Parse(e) => write!(f, "{e}"),
        }
    }
}

impl From<std::io::Error> for DatabaseError {
    fn from(err: std::io::Error) -> Self {
        DatabaseError::Io(err)
    }
}

impl From<yaml_serde::Error> for DatabaseError {
    fn from(err: yaml_serde::Error) -> Self {
        DatabaseError::Parse(err)
    }
}

impl EventDatabase {
    pub fn new(path: impl AsRef<Path>, title: &str) -> Result<Self, DatabaseError> {
        let yaml_str = fs::read_to_string(path)?;
        let data = yaml_serde::from_str::<EventsFile>(&yaml_str)?;
        let title = title.to_string();
        Ok(Self { data, title })
    }

    pub fn get_calendar(&self, from: OffsetDateTime, to: OffsetDateTime) -> Vec<CalendarEvent<'_>> {
        let mut output: Vec<CalendarEvent> = Vec::new();
        for entry in &self.data.events {
            for event in entry.schedule.occurrences() {
                let start_time = event.date.0;
                if from <= start_time && start_time < to {
                    let end_time: Option<OffsetDateTime> = match &entry.schedule {
                        super::parser::Schedule::Repeating { repeats } => {
                            if let Some(duration) = &repeats.duration {
                                Some(start_time + duration_from_interval(duration))
                            } else {
                                None
                            }
                        }
                        _ => None,
                    };
                    output.push(CalendarEvent {
                        event: entry,
                        index: event.index,
                        from: start_time,
                        to: end_time,
                    });
                } else if start_time >= to {
                    break;
                }
            }
        }

        output
    }

    pub fn list_all(&self) -> Vec<LeanCalendarEvent<'_>> {
        let mut output: Vec<LeanCalendarEvent> = Vec::new();
        for ev in &self.data.events {
            match ev.schedule.occurrences().next() {
                Some(first) => {
                    output.push(LeanCalendarEvent {
                        event: ev,
                        start_date: first.date.0,
                    });
                }
                None => {}
            }
        }

        output
    }

    #[allow(dead_code)]
    pub fn debug(&self) {
        for ev in &self.data.events {
            match &ev.schedule {
                Schedule::Manual { date_list: _ } => {
                    println!("[M]: {}", ev.title);
                }
                Schedule::Repeating { repeats: _ } => {
                    println!("[R]: {}", ev.title);
                }
            }
            if ev.schedule.is_indefinite() {
                for item in ev.schedule.occurrences().take(10) {
                    println!("[{}]: {}", item.index, item.date);
                }
                println!("...");
            } else {
                for item in ev.schedule.occurrences() {
                    println!("[{}]: {}", item.index, item.date);
                }
            }
        }
    }
}
