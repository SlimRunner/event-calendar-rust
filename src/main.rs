use std::fs::{self};

mod event_core;

use event_core::parser::EventsFile;
use event_core::query::EventDatabase;
use time::{Duration, OffsetDateTime};

fn main() {
    // TEMP PATH
    let path = "E:/dev/event-calendar-rust/data.yml";
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
    };

    let db = EventDatabase::new(&events);
    let today: OffsetDateTime = OffsetDateTime::now_utc();

    // db.debug();
    for item in db.upcoming(today) {
        let text = format!("{}: {}", item.event.title, item.start_date);
        println!("{}", text);
    }
    let week_window = Duration::new(3600 * 24 * 7, 0);

    let mut cal = db.calendar(today - week_window, today + week_window);
    cal.sort_by(|a, b| a.from.cmp(&b.from));
    let mut today_flag = true;

    for item in cal {
        let text = format!(
            "[{}/{}] {}: {}",
            item.index,
            item.event
                .schedule
                .get_count()
                .map_or(String::from("?"), |n| format!("{}", n)),
            item.event.title,
            item.from.weekday()
        );
        if today_flag && today < item.from {
            today_flag = false;
            println!("TODAY");
        }
        println!("{}", text);
    }
}
