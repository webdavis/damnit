mod parse;
mod term;

use jiff::ToSpan;
use jiff::tz::TimeZone;

use crate::weekday::week_start;
use crate::{Date, Kind, Object, Oid, Path, Priority, Transparency};

pub use parse::{QueryError, parse};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum DateSel {
    Today,
    Tomorrow,
    Yesterday,
    On(Date),
    ThisWeek,
    NextWeek,
    Before(Date),
    After(Date),
    None,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EstimateSel {
    Max(u32),
    Min(u32),
    None,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Term {
    Done,
    Overdue,
    Kind(Kind),
    Due(DateSel),
    Deadline(DateSel),
    Estimate(EstimateSel),
    Focus(DateSel),
    Completed(DateSel),
    Start(DateSel),
    Path(Path),
    Label(String),
    Priority(Priority),
    Transparency(Transparency),
    Attached(Oid),
    Subject(String),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Expr {
    And(Box<Expr>, Box<Expr>),
    Or(Box<Expr>, Box<Expr>),
    Not(Box<Expr>),
    Term(Term),
}

pub fn matches(expr: &Expr, object: &Object, today: Date, zone: &TimeZone) -> bool {
    match expr {
        Expr::And(a, b) => matches(a, object, today, zone) && matches(b, object, today, zone),
        Expr::Or(a, b) => matches(a, object, today, zone) || matches(b, object, today, zone),
        Expr::Not(a) => !matches(a, object, today, zone),
        Expr::Term(term) => term_matches(term, object, today, zone),
    }
}

fn term_matches(term: &Term, object: &Object, today: Date, zone: &TimeZone) -> bool {
    let task = object.as_task();
    let event = object.as_event();
    match term {
        Term::Done => task.is_some_and(|t| t.completion.is_done()),
        Term::Overdue => task.is_some_and(|t| {
            !t.completion.is_done() && t.due.as_ref().is_some_and(|d| d.date() < today)
        }),
        Term::Kind(kind) => object.kind() == *kind,
        Term::Due(sel) => {
            task.is_some_and(|t| date_matches(sel, t.due.as_ref().map(|d| d.date()), today))
        }
        Term::Deadline(sel) => task.is_some_and(|t| date_matches(sel, t.deadline, today)),
        Term::Completed(sel) => task.is_some_and(|t| {
            let day = t.completion.at().map(|at| at.to_zoned(zone.clone()).date());
            date_matches(sel, day, today)
        }),
        Term::Focus(sel) => {
            task.is_some_and(|t| date_matches(sel, t.focus.as_ref().map(|f| f.day), today))
        }
        Term::Estimate(sel) => task.is_some_and(|t| match (sel, t.estimate) {
            (EstimateSel::None, estimate) => estimate.is_none(),
            (EstimateSel::Max(most), Some(m)) => m <= *most,
            (EstimateSel::Min(least), Some(m)) => m >= *least,
            (_, None) => false,
        }),
        Term::Start(sel) => event.is_some_and(|e| date_matches(sel, Some(e.start.date()), today)),
        Term::Path(prefix) => object.base().path.is_within(prefix),
        Term::Label(name) => object.base().labels.contains(name),
        Term::Priority(p) => task.is_some_and(|t| t.priority == *p),
        Term::Transparency(t) => event.is_some_and(|e| e.transparency == *t),
        Term::Attached(oid) => task.is_some_and(|t| t.event.as_ref() == Some(oid)),
        Term::Subject(needle) => object
            .base()
            .subject
            .to_lowercase()
            .contains(&needle.to_lowercase()),
    }
}

fn date_matches(sel: &DateSel, value: Option<Date>, today: Date) -> bool {
    let Some(d) = value else {
        return matches!(sel, DateSel::None);
    };
    let week = week_start(today);
    match sel {
        DateSel::None => false,
        DateSel::Today => d == today,
        DateSel::Tomorrow => Some(d) == today.tomorrow().ok(),
        DateSel::Yesterday => Some(d) == today.yesterday().ok(),
        DateSel::On(on) => d == *on,
        DateSel::ThisWeek => in_week(d, week),
        DateSel::NextWeek => week
            .checked_add(1.week())
            .ok()
            .is_some_and(|w| in_week(d, w)),
        DateSel::Before(b) => d < *b,
        DateSel::After(a) => d > *a,
    }
}

fn in_week(d: Date, monday: Date) -> bool {
    monday
        .checked_add(6.days())
        .ok()
        .is_some_and(|sunday| d >= monday && d <= sunday)
}

#[cfg(test)]
mod completed_tests;
#[cfg(test)]
mod tests;
