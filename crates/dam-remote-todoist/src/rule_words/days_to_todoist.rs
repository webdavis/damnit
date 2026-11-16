use super::{is_month_day, is_weekday};

pub fn days_to_todoist(rule: &str) -> Option<String> {
    let words: Vec<&str> = rule.split_whitespace().collect();
    let (head, rest) = words.split_first()?;
    if *head != "every" && *head != "every!" {
        return None;
    }
    let (interval, rest) = match rest.first()?.parse::<u32>() {
        Ok(n) => (n, &rest[1..]),
        Err(_) => (1, rest),
    };
    let (unit, list, until) = match rest {
        [unit, "on", list] => (*unit, *list, None),
        [unit, "on", list, "until", date] => (*unit, *list, Some(*date)),
        _ => return None,
    };
    let days: Vec<&str> = list.split(',').collect();
    let said = match (interval, unit) {
        (1, "week" | "weeks") if days.iter().all(|d| is_weekday(d)) => days.join(", "),
        (2, "week" | "weeks") if days.len() == 1 && is_weekday(days[0]) => {
            format!("other {}", days[0])
        }
        (1, "month" | "months") if days.iter().all(|d| is_month_day(d)) => days.join(", "),
        _ => return None,
    };
    let mut sent = format!("{head} {said}");
    if let Some(date) = until {
        sent.push_str(&format!(" until {date}"));
    }
    Some(sent)
}
