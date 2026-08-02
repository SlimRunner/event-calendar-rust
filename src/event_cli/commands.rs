use comfy_table::{self, Cell, CellAlignment, Color, Row, Table};
use time::formatting::Formattable;
use time::macros::format_description;
use time::{Duration, OffsetDateTime};

use crate::event_core::query::EventDatabase;

pub fn list_upcoming(db: &EventDatabase) {
    let today: OffsetDateTime = OffsetDateTime::now_utc();

    for item in db.list_all().iter().filter(|ev| ev.start_date > today) {
        let text = format!("{}: {}", item.event.title, item.start_date);
        println!("{}", text);
    }
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

pub fn show_weekly_calendar(db: &EventDatabase) {
    let fmt_weekday = format_description!("[weekday]");
    let fmt_date = format_description!("[month repr:long] [day] [year]");
    let fmt_short_time = format_description!("[hour]:[minute]");

    let today: OffsetDateTime = OffsetDateTime::now_utc();
    let week_window = Duration::new(3600 * 24 * 7, 0);

    let mut cal = db.get_calendar(today - week_window, today + week_window);
    cal.sort_by(|a, b| a.from.cmp(&b.from));
    let mut today_flag = true;

    let mut table = Table::new();
    table.set_header(Row::from(vec!["Day", "Date", "Time", "i", "N", "Title"]));

    for item in &cal {
        if today_flag && today < item.from {
            today_flag = false;
            println!("TODAY");
        }

        let count = item.event.schedule.get_count();

        // the order of these checks matter
        let bound_color = match (item.index, count) {
            (i, n) if n.is_some() && i == n.unwrap() => Color::Yellow,
            (1, _) => Color::Cyan,
            _ => Color::White,
        };
        let today_color = match today.date() == item.from.date() {
            true => Color::Green,
            false => Color::White,
        };

        let count_str = count.map_or(String::from("?"), |n| format!("{}", n));

        let row = Row::from(
            vec![
                apply_date_format(item.from, fmt_weekday),
                apply_date_format(item.from, fmt_date),
                apply_date_format(item.from, fmt_short_time),
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
