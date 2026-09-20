//! One word of a query as the thing it selects: a label, a priority, a key
//! and a value, or a bare word looked up as a category.

use super::parse::QueryError;
use super::{DateSel, Term};
use crate::{Kind, Oid, Path, Priority, Transparency};

pub(super) fn term(word: &str) -> Result<Term, QueryError> {
    if let Some(label) = word.strip_prefix('@') {
        return Ok(Term::Label(label.to_string()));
    }
    if let Some(n) = word.strip_prefix('p').and_then(|n| n.parse::<u8>().ok()) {
        return Priority::new(n)
            .map(Term::Priority)
            .map_err(|_| bad("priority", word));
    }
    let (key, value) = match word.split_once(':') {
        Some(kv) => kv,
        None => {
            return match word {
                "done" => Ok(Term::Done),
                "overdue" => Ok(Term::Overdue),
                other => Err(QueryError::Unexpected(other.to_string())),
            };
        }
    };
    Ok(match key {
        "kind" => Term::Kind(match value {
            "task" => Kind::Task,
            "event" => Kind::Event,
            _ => return Err(bad(key, value)),
        }),
        "due" => Term::Due(date_sel(key, value)?),
        "deadline" => Term::Deadline(date_sel(key, value)?),
        "start" => Term::Start(date_sel(key, value)?),
        "path" => Term::Path(Path::parse(value).map_err(|_| bad(key, value))?),
        "label" => Term::Label(value.to_string()),
        "priority" => Term::Priority(
            value
                .parse::<u8>()
                .ok()
                .and_then(|n| Priority::new(n).ok())
                .ok_or_else(|| bad(key, value))?,
        ),
        "transparency" => Term::Transparency(match value {
            "busy" => Transparency::Busy,
            "free" => Transparency::Free,
            _ => return Err(bad(key, value)),
        }),
        "attached" => Term::Attached(Oid::parse(value).map_err(|_| bad(key, value))?),
        "subject" => Term::Subject(value.to_string()),
        name => Term::Category {
            name: name.to_string(),
            value: value.to_string(),
        },
    })
}

fn date_sel(key: &str, value: &str) -> Result<DateSel, QueryError> {
    Ok(match value {
        "today" => DateSel::Today,
        "tomorrow" => DateSel::Tomorrow,
        "yesterday" => DateSel::Yesterday,
        "this-week" => DateSel::ThisWeek,
        "next-week" => DateSel::NextWeek,
        "none" => DateSel::None,
        v => {
            if let Some(d) = v.strip_prefix("before:") {
                DateSel::Before(d.parse().map_err(|_| bad(key, value))?)
            } else if let Some(d) = v.strip_prefix("after:") {
                DateSel::After(d.parse().map_err(|_| bad(key, value))?)
            } else {
                DateSel::On(v.parse().map_err(|_| bad(key, value))?)
            }
        }
    })
}

fn bad(key: &str, value: &str) -> QueryError {
    QueryError::BadValue {
        key: key.to_string(),
        value: value.to_string(),
    }
}
