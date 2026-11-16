const MONTHS: [&str; 12] = [
    "Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec",
];

const START_WORDS: [&str; 2] = [" starting ", " from "];

pub fn with_start(words: &str, due: &str) -> String {
    let Some(start) = due.get(..10).and_then(start_date) else {
        return words.to_string();
    };
    let mut sent = match due.get(10..16).and_then(|t| t.strip_prefix('T')) {
        Some(time) if is_clock(time) => match words.split_once(" until ") {
            Some((days, end)) => format!("{days} at {time} until {end}"),
            None => format!("{words} at {time}"),
        },
        _ => words.to_string(),
    };
    sent.push_str(&format!(" starting {start}"));
    sent
}

pub fn without_start(text: &str) -> String {
    let rule = START_WORDS
        .iter()
        .find_map(|w| text.rsplit_once(w).filter(|(_, d)| is_start_date(d)))
        .map_or(text, |(rule, _)| rule);
    match rule.split_once(" at ") {
        Some((days, rest)) => match rest.split_once(' ') {
            None if is_clock(rest) => days.to_string(),
            Some((time, end)) if is_clock(time) && end.starts_with("until ") => {
                format!("{days} {end}")
            }
            _ => rule.to_string(),
        },
        None => rule.to_string(),
    }
}

fn start_date(iso: &str) -> Option<String> {
    let mut parts = iso.splitn(3, '-');
    let (year, month, day) = (parts.next()?, parts.next()?, parts.next()?);
    let month = MONTHS.get(month.parse::<usize>().ok()?.checked_sub(1)?)?;
    let day = day.parse::<u8>().ok().filter(|d| (1..=31).contains(d))?;
    (year.len() == 4 && year.parse::<u16>().is_ok()).then(|| format!("{day} {month} {year}"))
}

fn is_start_date(text: &str) -> bool {
    match text.split(' ').collect::<Vec<_>>()[..] {
        [day, month, year] => {
            matches!(day.parse::<u8>(), Ok(1..=31))
                && !day.starts_with('0')
                && MONTHS.contains(&month)
                && year.len() == 4
                && year.parse::<u16>().is_ok()
        }
        _ => false,
    }
}

fn is_clock(text: &str) -> bool {
    match text.split_once(':') {
        Some((h, m)) => {
            h.len() == 2
                && m.len() == 2
                && matches!(h.parse::<u8>(), Ok(0..=23))
                && matches!(m.parse::<u8>(), Ok(0..=59))
        }
        None => false,
    }
}
