use super::{ORDINALS, is_weekday};

#[derive(Debug, PartialEq, Eq)]
pub enum Unheld {
    Interval { days: String },
    CountedBack { weekday: String },
}

pub fn to_todoist(rule: &str) -> Result<String, Unheld> {
    let words: Vec<&str> = rule.split_whitespace().collect();
    let lists: Vec<&str> = words
        .windows(2)
        .filter(|pair| pair[0] == "on" && pair[1].split(',').any(|i| weekday_of_month(i).is_some()))
        .map(|pair| pair[1])
        .collect();
    let (Some(head), false) = (words.first(), lists.is_empty()) else {
        return Ok(rule.to_string());
    };
    let interval = words
        .get(1)
        .and_then(|w| w.parse::<u32>().ok())
        .unwrap_or(1);
    if interval != 1 {
        return Err(Unheld::Interval {
            days: lists.join(","),
        });
    }
    let items: Result<Vec<String>, Unheld> = lists
        .iter()
        .flat_map(|list| list.split(','))
        .filter_map(weekday_of_month)
        .map(todoist_item)
        .collect();
    let mut sent = format!("{head} {}", items?.join(", "));
    if let Some(until) = words.windows(2).find(|pair| pair[0] == "until") {
        sent.push_str(&format!(" until {}", until[1]));
    }
    Ok(sent)
}

fn weekday_of_month(item: &str) -> Option<(i8, &str)> {
    let unsigned = item.strip_prefix('-').unwrap_or(item);
    let weekday = unsigned.trim_start_matches(|c: char| c.is_ascii_digit());
    let number = &item[..item.len() - weekday.len()];
    match (number.parse::<i8>(), is_weekday(weekday)) {
        (Ok(nth @ (-5..=-1 | 1..=5)), true) => Some((nth, weekday)),
        _ => None,
    }
}

fn todoist_item((nth, weekday): (i8, &str)) -> Result<String, Unheld> {
    ORDINALS
        .iter()
        .find(|(n, ..)| *n == nth)
        .map(|(_, short, _)| format!("{short} {weekday}"))
        .ok_or_else(|| Unheld::CountedBack {
            weekday: weekday.to_string(),
        })
}
