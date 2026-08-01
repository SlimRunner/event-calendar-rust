use time::OffsetDateTime;

use crate::event_core::{
    occurrence::duration_from_interval,
    parser::{Event, EventsFile, Schedule},
};

pub struct UpcomingEvent<'a> {
    pub event: &'a Event,
    pub start_date: OffsetDateTime,
}

pub struct CalendarEvent<'a> {
    pub event: &'a Event,
    pub index: usize,
    pub from: OffsetDateTime,
    pub to: Option<OffsetDateTime>,
}

pub struct EventDatabase<'a> {
    data: &'a EventsFile,
}

impl<'a> EventDatabase<'a> {
    pub fn new(data: &'a EventsFile) -> Self {
        Self { data }
    }

    pub fn upcoming(&self, now: OffsetDateTime) -> Vec<UpcomingEvent<'a>> {
        let mut output: Vec<UpcomingEvent> = Vec::new();
        for ev in &self.data.events {
            match ev.schedule.occurrences().next() {
                Some(first) => {
                    if first.date.0 > now {
                        output.push(UpcomingEvent {
                            event: ev,
                            start_date: first.date.0,
                        });
                    }
                }
                None => {}
            }
        }

        output
    }

    pub fn calendar(&self, from: OffsetDateTime, to: OffsetDateTime) -> Vec<CalendarEvent<'a>> {
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
