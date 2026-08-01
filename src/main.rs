use std::fs::{self};

mod event_core;

use event_core::parser::{EventsFile, Schedule};

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
                println!("{}", t);
            }
            println!("...");
        } else {
            for t in ev.schedule.occurrences() {
                println!("{}", t);
            }
        }
    }
}
