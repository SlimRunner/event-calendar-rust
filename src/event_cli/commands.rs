use std::collections::{HashMap, HashSet};

use comfy_table::{self, Cell, CellAlignment, Color, Row, Table};
use time::formatting::Formattable;
use time::macros::format_description;
use time::{Duration, OffsetDateTime, UtcOffset};

use crate::event_core::query::{CalendarEvent, EventDatabase, LeanCalendarEvent};

fn format_signed_duration(time: Duration) -> String {
    let total_seconds = time.whole_seconds().abs();
    let days = total_seconds / (3600 * 24);
    let hours = (total_seconds % (3600 * 24)) / 3600;
    let minutes = (total_seconds % 3600) / 60;
    let seconds = total_seconds % 60;

    match (days, time.is_negative()) {
        (0, false) => format!("{:0>2}:{:0>2}:{:0>2}", hours, minutes, seconds),
        (1, false) => format!(
            "{} day, {:0>2}:{:0>2}:{:0>2}",
            days, hours, minutes, seconds
        ),
        (_, true) => format!(
            "-{} days, {:0>2}:{:0>2}:{:0>2}",
            days, hours, minutes, seconds
        ),
        _ => format!(
            "{} days, {:0>2}:{:0>2}:{:0>2}",
            days, hours, minutes, seconds
        ),
    }
}

fn print_upcoming_from_list<'vec, 'db, I>(cal: I)
where
    'db: 'vec,
    I: IntoIterator<Item = &'vec LeanCalendarEvent<'db>>,
{
    let fmt_date = format_description!("[year]-[month]-[day] [weekday repr:short]");

    let offset = match UtcOffset::current_local_offset() {
        Ok(offset) => offset,
        Err(_) => UtcOffset::UTC,
    };

    let today: OffsetDateTime = OffsetDateTime::now_utc();
    let mut table = Table::new();
    table.set_header(Row::from(vec!["Title", "Countdown", "Start", "Total"]));

    for item in cal {
        let diff = format_signed_duration(item.start_date - today);
        let count = item.event.schedule.get_count();
        let count_str = count.map_or(String::from("?"), |n| format!("{}", n));

        table.add_row(vec![
            truncate(&item.event.title, 30),
            diff,
            apply_date_format(item.start_date.to_offset(offset), fmt_date),
            count_str,
        ]);
    }

    table.load_preset(comfy_table::presets::UTF8_FULL);
    table.apply_modifier(comfy_table::modifiers::UTF8_ROUND_CORNERS);
    for i in vec![1, 2, 3] {
        table
            .column_mut(i)
            .unwrap()
            .set_cell_alignment(CellAlignment::Right);
    }

    println!("{}", table.to_string());
}

fn print_list_w_tags<'vec, 'db, I>(cal: I, tags: &[String])
where
    'db: 'vec,
    I: IntoIterator<Item = &'vec LeanCalendarEvent<'db>>,
{
    let fmt_date = format_description!("[year]-[month]-[day] [weekday repr:short]");

    let offset = match UtcOffset::current_local_offset() {
        Ok(offset) => offset,
        Err(_) => UtcOffset::UTC,
    };

    let mut table = Table::new();
    table.set_header(Row::from(vec!["Title", "Start", "Total", "Tags"]));

    for item in cal {
        let count = item.event.schedule.get_count();
        let count_str = count.map_or(String::from("?"), |n| format!("{}", n));

        let mut item_tags = HashSet::new();
        item.event.tags.iter().for_each(|s| {
            item_tags.insert(s.to_string());
        });
        let mut filter_tags: HashSet<String> = HashSet::new();
        tags.iter().for_each(|s| {
            filter_tags.insert(s.to_string());
        });
        let tag_diff = &item_tags - &filter_tags;
        let mut tag_cell = tag_diff.iter().map(|s| s.as_str()).collect::<Vec<_>>();
        tag_cell.sort();
        let tag_cell = tag_cell.join(", ");

        table.add_row(vec![
            truncate(&item.event.title, 30),
            apply_date_format(item.start_date.to_offset(offset), fmt_date),
            count_str,
            tag_cell,
        ]);
    }

    table.load_preset(comfy_table::presets::UTF8_FULL);
    table.apply_modifier(comfy_table::modifiers::UTF8_ROUND_CORNERS);
    for i in vec![1, 2] {
        table
            .column_mut(i)
            .unwrap()
            .set_cell_alignment(CellAlignment::Right);
    }

    println!("{}", table.to_string());
}

fn print_tag_tallies<'vec, 'db, I>(cal: I, max_comb: Option<usize>)
where
    'db: 'vec,
    I: IntoIterator<Item = &'vec LeanCalendarEvent<'db>>,
{
    let mut tallies: HashMap<Vec<&String>, usize> = HashMap::new();
    let mut table = Table::new();
    table.set_header(Row::from(vec!["Tag", "Count"]));

    for item in cal {
        let tags = &item.event.tags;
        let n = tags.len();

        for mask in 0..(1usize << n) {
            if let Some(max_size) = max_comb
                && mask.count_ones() as usize > max_size
            {
                continue;
            }
            let mut subset = tags
                .iter()
                .enumerate()
                .filter_map(|(i, tag)| ((mask >> i) & 1 == 1).then_some(tag))
                .collect::<Vec<_>>();
            subset.sort();
            *tallies.entry(subset).or_insert(0) += 1;
        }
    }

    let mut rows = tallies.iter().collect::<Vec<_>>();
    rows.sort();

    for (v, i) in rows {
        table.add_row(vec![format!("{:?}", v), i.to_string()]);
    }

    table.load_preset(comfy_table::presets::UTF8_FULL);
    table.apply_modifier(comfy_table::modifiers::UTF8_ROUND_CORNERS);
    table
        .column_mut(1)
        .unwrap()
        .set_cell_alignment(CellAlignment::Right);

    println!("{}", table.to_string());
}

pub fn list_upcoming(db: &EventDatabase, show_all: bool) {
    let today: OffsetDateTime = OffsetDateTime::now_utc();
    let mut cal = db.list_all();
    cal.sort_by(|a, b| a.start_date.cmp(&b.start_date));
    let iter_cal = cal.iter().filter(|ev| {
        ev.start_date > today && (show_all || ev.event.tags.contains(&"public".to_string()))
    });

    print_upcoming_from_list(iter_cal);
}

pub fn list_filtered_by_tags(db: &EventDatabase, any_list: &[String], strict_list: &[String]) {
    let mut cal = db.list_all();
    cal.sort_by(|a, b| a.start_date.cmp(&b.start_date));

    let iter_cal = cal.iter().filter(|ev| {
        let has_at_least =
            any_list.is_empty() || ev.event.tags.iter().any(|tag| any_list.contains(tag));
        let has_all = strict_list.iter().all(|tag| ev.event.tags.contains(tag));
        has_at_least && has_all
    });

    print_list_w_tags(iter_cal, &strict_list);
}

pub fn tag_summary(
    db: &EventDatabase,
    any_list: &[String],
    strict_list: &[String],
    max_comb: Option<usize>,
) {
    let mut cal = db.list_all();
    cal.sort_by(|a, b| a.start_date.cmp(&b.start_date));

    let iter_cal = cal.iter().filter(|ev| {
        let has_at_least =
            any_list.is_empty() || ev.event.tags.iter().any(|tag| any_list.contains(tag));
        let has_all = strict_list.iter().all(|tag| ev.event.tags.contains(tag));
        has_at_least && has_all
    });

    print_tag_tallies(iter_cal, max_comb);
}

fn apply_date_format<T: Formattable>(date: OffsetDateTime, format: T) -> String {
    return format!("{}", date.format(&format).unwrap_or(date.to_string()));
}

fn truncate(s: &str, max: usize) -> String {
    if s.chars().count() <= max {
        return s.to_owned();
    }

    s.chars().take(max - 3).collect::<String>() + "..."
}

fn show_weekly_calendar_and_filter(db: &EventDatabase, filter: impl Fn(&&CalendarEvent) -> bool) {
    let fmt_weekday = format_description!("[weekday]");
    let fmt_date = format_description!("[month repr:long] [day] [year]");
    let fmt_short_time = format_description!("[hour repr:12]:[minute] [period]");

    let offset = match UtcOffset::current_local_offset() {
        Ok(offset) => offset,
        Err(_) => UtcOffset::UTC,
    };
    let today: OffsetDateTime = OffsetDateTime::now_utc();
    let week_window = Duration::new(3600 * 24 * 7, 0);

    let mut cal = db.get_calendar(today - week_window, today + week_window);
    cal.sort_by(|a, b| a.from.cmp(&b.from));

    let mut table = Table::new();
    table.set_header(Row::from(vec!["Day", "Date", "Time", "i", "N", "Title"]));

    for item in cal.iter().filter(filter) {
        let count = item.event.schedule.get_count();

        // the order of these checks matter
        let bound_color = match (item.index, count) {
            (i, n) if n.is_some() && i == n.unwrap() => Color::Yellow,
            (1, _) => Color::Cyan,
            _ => Color::White,
        };
        let today_color = match today.to_offset(offset).date() == item.from.to_offset(offset).date()
        {
            true => Color::Green,
            false => Color::White,
        };

        let count_str = count.map_or(String::from("?"), |n| format!("{}", n));

        let time_disp = match (item.from, item.to) {
            (from, Some(to)) if from.date() == to.date() => {
                format!(
                    "{} - {}",
                    apply_date_format(from.to_offset(offset), fmt_short_time),
                    apply_date_format(to.to_offset(offset), fmt_short_time)
                )
            }
            (from, _) => apply_date_format(from.to_offset(offset), fmt_short_time),
        };

        let row = Row::from(
            vec![
                apply_date_format(item.from.to_offset(offset), fmt_weekday),
                apply_date_format(item.from.to_offset(offset), fmt_date),
                time_disp,
                format!("{}", item.index),
                format!("{}", count_str),
                truncate(&item.event.title, 30),
            ]
            .iter()
            .enumerate()
            .map(|(i, c)| match i {
                3 | 4 | 5 => Cell::from(c).fg(bound_color),
                0 | 1 | 2 => Cell::from(c).fg(today_color),
                _ => Cell::from(c),
            }),
        );

        table.add_row(Row::from(row));
    }

    table.load_preset(comfy_table::presets::UTF8_FULL);
    table.apply_modifier(comfy_table::modifiers::UTF8_ROUND_CORNERS);
    for i in vec![1, 2, 3, 4] {
        table
            .column_mut(i)
            .unwrap()
            .set_cell_alignment(CellAlignment::Right);
    }

    println!("{}", table.to_string());
}

pub fn show_weekly_calendar(db: &EventDatabase, show_all: bool) {
    show_weekly_calendar_and_filter(db, |ev| {
        // if flag is false and public tag is not => filter out (true)
        show_all || ev.event.tags.contains(&"public".to_string())
    });
}
