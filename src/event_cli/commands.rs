use time::{Duration, OffsetDateTime};

use crate::event_core::query::EventDatabase;

pub fn list_upcoming(db: &EventDatabase) {
    let today: OffsetDateTime = OffsetDateTime::now_utc();

    for item in db.list_all().iter().filter(|ev| ev.start_date > today) {
        let text = format!("{}: {}", item.event.title, item.start_date);
        println!("{}", text);
    }
}

pub fn show_weekly_calendar(db: &EventDatabase) {
    let today: OffsetDateTime = OffsetDateTime::now_utc();
    let week_window = Duration::new(3600 * 24 * 7, 0);

    let mut cal = db.get_calendar(today - week_window, today + week_window);
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
