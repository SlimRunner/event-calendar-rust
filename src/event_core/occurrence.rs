use std::{collections::VecDeque, iter::Peekable};

use time::Duration;

use super::schedule_kinds::{Exception, ExceptionKind, Interval, Repeat, Rfc3339Date, TimeUnit};

pub enum Occurrences<'a> {
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

pub struct RepeatOccurrences<'a> {
    repeat: &'a Repeat,

    current: Rfc3339Date,
    index: u32,
    remaining: Option<u32>,

    exceptions: Peekable<std::slice::Iter<'a, Exception>>,
    date_queue: VecDeque<Rfc3339Date>,
}

impl<'a> RepeatOccurrences<'a> {
    pub(super) fn new(repeat: &'a Repeat) -> Self {
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
