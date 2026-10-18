mod parse;
mod term;

use jiff::ToSpan;
use jiff::civil::Weekday;

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

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Term {
    Done,
    Overdue,
    Kind(Kind),
    Due(DateSel),
    Deadline(DateSel),
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

pub fn matches(expr: &Expr, object: &Object, today: Date) -> bool {
    match expr {
        Expr::And(a, b) => matches(a, object, today) && matches(b, object, today),
        Expr::Or(a, b) => matches(a, object, today) || matches(b, object, today),
        Expr::Not(a) => !matches(a, object, today),
        Expr::Term(term) => term_matches(term, object, today),
    }
}

fn term_matches(term: &Term, object: &Object, today: Date) -> bool {
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

fn week_start(date: Date) -> Date {
    let back = date.weekday().since(Weekday::Monday) as i32;
    date.checked_sub(back.days()).unwrap_or(date)
}

#[cfg(test)]
mod tests;
