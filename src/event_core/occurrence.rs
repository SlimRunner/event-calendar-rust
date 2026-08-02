use std::{collections::VecDeque, iter::{Enumerate, Peekable}};

use time::Duration;

use super::schedule_kinds::{Exception, ExceptionKind, Interval, Repeat, Rfc3339Date, TimeUnit};

pub enum Occurrences<'a> {
    Manual(Enumerate<std::slice::Iter<'a, Rfc3339Date>>),
    Repeating(RepeatOccurrences<'a>),
}

pub struct OccurrenceItem {
    pub(super) date: Rfc3339Date,
    pub(super) index: usize,
}

impl<'a> Iterator for Occurrences<'a> {
    type Item = OccurrenceItem;

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            Occurrences::Manual(iter) => match iter.next() {
                Some((i, date)) => Some(OccurrenceItem {
                    date: *date,
                    index: i + 1,
                }),
                None => None,
            },
            Occurrences::Repeating(iter) => match iter.next() {
                Some(item) => Some(item),
                None => None,
            },
        }
    }
}

pub struct RepeatOccurrences<'a> {
    repeat: &'a Repeat,

    current: Rfc3339Date,
    index: usize,
    remaining: Option<usize>,

    exceptions: Peekable<std::slice::Iter<'a, Exception>>,
    date_queue: VecDeque<OccurrenceItem>,
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

pub(super) fn duration_from_interval(interval: &Interval) -> Duration {
    let count = interval.length.get() as i64;
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

pub(super) fn duration_from_count(count: isize, unit: &TimeUnit) -> Duration {
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
    type Item = OccurrenceItem;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(item) = self.date_queue.pop_back() {
            return Some(item);
        }
        let can_continue = match self.remaining {
            Some(t) => self.index < t,
            None => true,
        };
        // println!("{}", self.index);
        if can_continue {
            let mut items: usize = 1;
            let date_interval: Duration = duration_from_interval(&self.repeat.interval);
            let mut offset_interval = Duration::nanoseconds(0);
            let mut skip_interval = Duration::nanoseconds(0);

            while let Some(exception) = self.exceptions.peek() {
                if exception.index <= (self.index + 1) {
                    let exception = self.exceptions.next().unwrap();

                    // BUGBUG: if you mix skips with other exceptions
                    // your warranty is void

                    match &exception.kind {
                        ExceptionKind::Shift { shift } => {
                            offset_interval += duration_from_count(shift.count, &shift.unit);
                        }
                        ExceptionKind::Skip { with_replacement } => {
                            if !*with_replacement {
                                self.index += 1;
                            }
                            skip_interval += duration_from_interval(&self.repeat.interval);
                            offset_interval = skip_interval;
                            items = 1;
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
                // pre-added to naturally make index 1-based
                self.index += 1;
                self.date_queue.push_front(OccurrenceItem {
                    date: self.current,
                    index: (self.index + self.repeat.offset_count.unwrap_or(0)),
                });

                // prevent enqueuing more events than max
                if let Some(rem) = self.remaining
                    && self.index >= rem
                {
                    break;
                }
            }
            self.current.0 += date_interval - offset_interval + skip_interval;

            if let Some(item) = self.date_queue.pop_back() {
                Some(item)
            } else {
                // 100% should never execute
                None
            }
        } else {
            None
        }
    }
}
