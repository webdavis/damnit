use super::{ORDINALS, is_month_day, is_weekday};

pub fn from_todoist(text: &str) -> String {
    dam_rule(text).unwrap_or_else(|| text.to_string())
}

fn dam_rule(text: &str) -> Option<String> {
    let (head, rest) = text.split_once(' ')?;
    if head != "every" && head != "every!" {
        return None;
    }
    let (days, until) = match rest.rsplit_once(" until ") {
        Some((days, date)) if !date.contains(' ') => (days, Some(date)),
        _ => (rest, None),
    };
    let mut rule = format!("{head} {}", dam_days(days)?);
    if let Some(date) = until {
        rule.push_str(&format!(" until {date}"));
    }
    Some(rule)
}

fn dam_days(days: &str) -> Option<String> {
    if let Some(other) = every_other(days) {
        return Some(other);
    }
    let items: Vec<&str> = days.split(", ").collect();
    if items.iter().all(|i| is_weekday(i)) {
        return Some(format!("week on {}", items.join(",")));
    }
    if items.iter().all(|i| is_month_day(i)) {
        return Some(format!("month on {}", items.join(",")));
    }
    let nth: Option<Vec<String>> = items.into_iter().map(dam_item).collect();
    Some(format!("month on {}", nth?.join(",")))
}

fn every_other(days: &str) -> Option<String> {
    let (other, what) = days.split_once(' ')?;
    if !other.eq_ignore_ascii_case("other") {
        return None;
    }
    if is_weekday(what) {
        return Some(format!("2 weeks on {what}"));
    }
    let unit = what.to_ascii_lowercase();
    ["day", "week", "month", "year"]
        .contains(&unit.as_str())
        .then(|| format!("2 {unit}s"))
}

fn dam_item(item: &str) -> Option<String> {
    let (ordinal, weekday) = item.split_once(' ')?;
    let (nth, ..) = ORDINALS.iter().find(|(_, short, long)| {
        ordinal.eq_ignore_ascii_case(short) || ordinal.eq_ignore_ascii_case(long)
    })?;
    is_weekday(weekday).then(|| format!("{nth}{weekday}"))
}
