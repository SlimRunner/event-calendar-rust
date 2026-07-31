use std::{
    fs::{self},
    num::NonZeroU32,
};

use serde::{Deserialize, Serialize};
use time::OffsetDateTime;
use time::serde::rfc3339;

#[derive(Debug, Serialize, Deserialize)]
struct EventsFile {
    events: Vec<Event>,
}

#[derive(Debug, Serialize, Deserialize)]
struct Event {
    title: String,
    #[serde(default)]
    tags: Vec<String>,
    #[serde(flatten)]
    schedule: Schedule,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
enum Schedule {
    Repeating { repeats: Repeat },
    Manual { date_list: Vec<Rfc3339Date> },
}

#[derive(Debug, Serialize, Deserialize)]
struct Exception {
    index: u32,

    #[serde(flatten)]
    kind: ExceptionKind,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "kind")]
enum ExceptionKind {
    #[serde(rename = "skip")]
    Skip { with_replacement: bool },

    #[serde(rename = "multiplicity")]
    Multiplicity { count: NonZeroU32 },

    #[serde(rename = "shift")]
    Shift { shift: Shift },
}

#[derive(Debug, Serialize, Deserialize)]
struct Repeat {
    starts: Rfc3339Date,
    times: Option<u32>,
    interval: Interval,
    duration: Option<f64>,
    #[serde(rename = "offset-count")]
    offset_count: Option<NonZeroU32>,
    #[serde(default)]
    exceptions: Vec<Exception>,
}

#[derive(Debug, Serialize, Deserialize)]
struct Shift {
    unit: TimeUnit,
    count: i32,
}

#[derive(Debug, Serialize, Deserialize)]
struct Interval {
    unit: TimeUnit,
    length: NonZeroU32,
}

#[derive(Debug, Serialize, Deserialize)]
enum TimeUnit {
    #[serde(rename = "hour")]
    Hour,
    #[serde(rename = "day")]
    Day,
    #[serde(rename = "week")]
    Week,
    #[serde(rename = "month")]
    Month,
    #[serde(rename = "year")]
    Year,
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy)]
struct Rfc3339Date(#[serde(with = "rfc3339")] OffsetDateTime);

fn main() {
    let path = "E:/dev/event-calendar/data/events.yaml";
    let yaml_str = match fs::read_to_string(path) {
        Ok(txt) => txt,
        Err(e) => {
            eprintln!("Failed to load {}: {:?}", path, e);
            std::process::exit(1);
        }
    };
    let events = match yaml_serde::from_str::<EventsFile>(&yaml_str) {
        Ok(e) => e,
        Err(e) => {
            eprintln!("Failed to parse {}: {:?}", path, e);
            std::process::exit(1);
        }
    }
    .events;

    for ev in events {
        match &ev.schedule {
            Schedule::Manual { date_list: _ } => {
                println!("[M]: {}", ev.title);
            }
            Schedule::Repeating { repeats: _ } => {
                println!("[R]: {}", ev.title);
            }
        }
    }
}
