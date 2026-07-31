use std::{
    collections::VecDeque,
    fs::{self},
    iter::Peekable,
    num::NonZeroU32,
};

use serde::{Deserialize, Serialize};
use time::serde::rfc3339;
use time::{Duration, OffsetDateTime};

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

impl Schedule {
    fn occurrences(&self) -> Occurrences<'_> {
        match self {
            Schedule::Manual { date_list } => Occurrences::Manual(date_list.iter()),
            Schedule::Repeating { repeats } => {
                Occurrences::Repeating(RepeatOccurrences::new(repeats))
            }
        }
    }

    fn is_indefinite(&self) -> bool {
        match self {
            Schedule::Manual { date_list: _ } => false,
            Schedule::Repeating { repeats } => repeats.times.is_none(),
        }
    }
}

enum Occurrences<'a> {
    Manual(std::slice::Iter<'a, Rfc3339Date>),
    Repeating(RepeatOccurrences<'a>),
}

impl<'a> Iterator for Occurrences<'a> {
    type Item = Rfc3339Date;

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            Occurrences::Manual(iter) => iter.next().copied(),
            Occurrences::Repeating(iter) => iter.next(),
        }
    }
}

struct RepeatOccurrences<'a> {
    repeat: &'a Repeat,

    current: Rfc3339Date,
    index: u32,
    remaining: Option<u32>,

    exceptions: Peekable<std::slice::Iter<'a, Exception>>,
    date_queue: VecDeque<Rfc3339Date>,
}

impl<'a> RepeatOccurrences<'a> {
    fn new(repeat: &'a Repeat) -> Self {
        Self {
            repeat,
            current: repeat.starts,
            index: 0,
            remaining: repeat.times,
            exceptions: repeat.exceptions.iter().peekable(),
            date_queue: VecDeque::new(),
        }
    }
}

fn duration_from_interval(interval: &Interval) -> Duration {
    let count = i64::from(interval.length.get());
    match interval.unit {
        TimeUnit::Hour => Duration::new(count * 3600, 0),
        TimeUnit::Day => Duration::new(count * 3600 * 24, 0),
        TimeUnit::Week => Duration::new(count * 3600 * 24 * 7, 0),
        TimeUnit::Month => {
            panic!("Month intervals are not implemented yet");
        }
        TimeUnit::Year => {
            panic!("Year intervals are not implemented yet");
        }
    }
}

fn duration_from_count(count: i32, unit: &TimeUnit) -> Duration {
    let count: i64 = count as i64;
    match unit {
        TimeUnit::Hour => Duration::new(count * 3600, 0),
        TimeUnit::Day => Duration::new(count * 3600 * 24, 0),
        TimeUnit::Week => Duration::new(count * 3600 * 24 * 7, 0),
        TimeUnit::Month => {
            panic!("Month intervals are not implemented yet");
        }
        TimeUnit::Year => {
            panic!("Year intervals are not implemented yet");
        }
    }
}

impl<'a> Iterator for RepeatOccurrences<'a> {
    type Item = Rfc3339Date;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(date) = self.date_queue.pop_back() {
            return Some(date);
        }
        let can_continue = match self.remaining {
            Some(t) => self.index < t,
            None => true,
        };
        if can_continue {
            let mut items: u32 = 1;
            let date_interval: Duration = duration_from_interval(&self.repeat.interval);
            let mut offset_interval = Duration::nanoseconds(0);

            while let Some(exception) = self.exceptions.peek() {
                if exception.index <= self.index {
                    let exception = self.exceptions.next().unwrap();

                    match &exception.kind {
                        ExceptionKind::Shift { shift } => {
                            offset_interval += duration_from_count(shift.count, &shift.unit);
                        }
                        ExceptionKind::Skip { with_replacement } => {
                            if !*with_replacement {
                                self.index += 1;
                            }
                            offset_interval = duration_from_interval(&self.repeat.interval);
                            items = 1;
                            break;
                        }
                        ExceptionKind::Multiplicity { count } => {
                            items = count.get();
                        }
                    }
                } else {
                    break;
                }
            }
            self.current.0 += offset_interval;
            for _ in 0..items {
                self.index += 1;
                self.date_queue.push_front(self.current);
            }
            self.current.0 += date_interval - offset_interval;
            if let Some(date) = self.date_queue.pop_back() {
                Some(date)
            } else {
                // 100% should never execute
                None
            }
        } else {
            None
        }
    }
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
        if ev.schedule.is_indefinite() {
            for t in ev.schedule.occurrences().take(10) {
                println!("{}", t.0);
            }
            println!("...");
        } else {
            for t in ev.schedule.occurrences() {
                println!("{}", t.0);
            }
        }
    }
}
