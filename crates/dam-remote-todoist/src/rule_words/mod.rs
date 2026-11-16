mod days_to_todoist;
mod from_todoist;
mod start;
mod to_todoist;

pub use from_todoist::from_todoist;
pub use start::with_start;
pub use to_todoist::{Unheld, to_todoist};

const ORDINALS: [(i8, &str, &str); 6] = [
    (1, "1st", "first"),
    (2, "2nd", "second"),
    (3, "3rd", "third"),
    (4, "4th", "fourth"),
    (5, "5th", "fifth"),
    (-1, "last", "last"),
];

const WEEKDAYS: [&str; 14] = [
    "mon",
    "tue",
    "wed",
    "thu",
    "fri",
    "sat",
    "sun",
    "monday",
    "tuesday",
    "wednesday",
    "thursday",
    "friday",
    "saturday",
    "sunday",
];

fn is_weekday(word: &str) -> bool {
    WEEKDAYS.contains(&word.to_ascii_lowercase().as_str())
}

fn is_month_day(word: &str) -> bool {
    matches!(word.parse::<u8>(), Ok(1..=31))
}

#[cfg(test)]
mod tests;
