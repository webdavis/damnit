use super::{DateSel, EstimateSel, Term};
use crate::{Categories, Kind, Oid, Path, Priority, Transparency, parse_minutes};

pub(super) fn term(word: &str, categories: &Categories) -> Option<Term> {
    if let Some(label) = word.strip_prefix('@') {
        return Some(Term::Label(label.to_string()));
    }
    if let Some(n) = word.strip_prefix('p').and_then(|n| n.parse::<u8>().ok()) {
        return Priority::new(n).ok().map(Term::Priority);
    }
    let (key, value) = match word.split_once(':') {
        Some((_, "")) => return None,
        Some(kv) => kv,
        None => {
            return match word {
                "done" => Some(Term::Done),
                "overdue" => Some(Term::Overdue),
                _ => None,
            };
        }
    };
    Some(match key {
        "kind" => Term::Kind(match value {
            "task" => Kind::Task,
            "event" => Kind::Event,
            _ => return None,
        }),
        "due" => Term::Due(date_sel(value)?),
        "deadline" => Term::Deadline(date_sel(value)?),
        "focus" => Term::Focus(date_sel(value)?),
        "completed" | "completed_at" => Term::Completed(date_sel(value)?),
        "estimate" => Term::Estimate(estimate_sel(value)?),
        "start" => Term::Start(date_sel(value)?),
        "path" => Term::Path(Path::parse(value).ok()?),
        "label" | "labels" => Term::Label(value.to_string()),
        "priority" => Term::Priority(
            value
                .parse::<u8>()
                .ok()
                .and_then(|n| Priority::new(n).ok())?,
        ),
        "transparency" => Term::Transparency(match value {
            "busy" => Transparency::Busy,
            "free" => Transparency::Free,
            _ => return None,
        }),
        "attached" => Term::Attached(Oid::parse(value).ok()?),
        "subject" => Term::Subject(value.to_string()),
        name => category_value(name, value, categories)?,
    })
}

fn category_value(name: &str, value: &str, categories: &Categories) -> Option<Term> {
    let category = categories.category_of_name(name)?;
    category
        .values
        .iter()
        .any(|v| v == value)
        .then(|| Term::Label(value.to_string()))
}

fn date_sel(value: &str) -> Option<DateSel> {
    Some(match value {
        "today" => DateSel::Today,
        "tomorrow" => DateSel::Tomorrow,
        "yesterday" => DateSel::Yesterday,
        "this-week" => DateSel::ThisWeek,
        "next-week" => DateSel::NextWeek,
        "none" => DateSel::None,
        v => {
            if let Some(d) = v.strip_prefix("before:") {
                DateSel::Before(d.parse().ok()?)
            } else if let Some(d) = v.strip_prefix("after:") {
                DateSel::After(d.parse().ok()?)
            } else {
                DateSel::On(v.parse().ok()?)
            }
        }
    })
}

fn estimate_sel(value: &str) -> Option<EstimateSel> {
    if value == "none" {
        return Some(EstimateSel::None);
    }
    let (bound, minutes) = value.split_once(':')?;
    let minutes = parse_minutes(minutes)?;
    match bound {
        "max" => Some(EstimateSel::Max(minutes)),
        "min" => Some(EstimateSel::Min(minutes)),
        _ => None,
    }
}
