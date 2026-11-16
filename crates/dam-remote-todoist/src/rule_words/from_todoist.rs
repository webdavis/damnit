use super::{ORDINALS, is_weekday};

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
    let items: Option<Vec<String>> = days.split(", ").map(dam_item).collect();
    let mut rule = format!("{head} month on {}", items?.join(","));
    if let Some(date) = until {
        rule.push_str(&format!(" until {date}"));
    }
    Some(rule)
}

fn dam_item(item: &str) -> Option<String> {
    let (ordinal, weekday) = item.split_once(' ')?;
    let (nth, ..) = ORDINALS.iter().find(|(_, short, long)| {
        ordinal.eq_ignore_ascii_case(short) || ordinal.eq_ignore_ascii_case(long)
    })?;
    is_weekday(weekday).then(|| format!("{nth}{weekday}"))
}
