use serde::{Deserialize, Serialize};

use super::occurrence::{Occurrences, RepeatOccurrences};
use super::schedule_kinds::{Repeat, Rfc3339Date};

#[derive(Debug, Serialize, Deserialize)]
pub struct EventsFile {
    pub events: Vec<Event>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Event {
    pub title: String,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(flatten)]
    pub schedule: Schedule,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Schedule {
    Repeating { repeats: Repeat },
    Manual { date_list: Vec<Rfc3339Date> },
}

impl Schedule {
    pub fn occurrences(&self) -> Occurrences<'_> {
        match self {
            Schedule::Manual { date_list } => Occurrences::Manual(date_list.iter()),
            Schedule::Repeating { repeats } => {
                Occurrences::Repeating(RepeatOccurrences::new(repeats))
            }
        }
    }

    pub fn is_indefinite(&self) -> bool {
        match self {
            Schedule::Manual { date_list: _ } => false,
            Schedule::Repeating { repeats } => repeats.times.is_none(),
        }
    }

    pub fn get_count(&self) -> Option<usize> {
        match self {
            Schedule::Manual { date_list } => Some(date_list.len()),
            Schedule::Repeating { repeats } => repeats.times,
        }
    }
}
