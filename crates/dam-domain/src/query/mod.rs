mod parse;

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
    Category { name: String, value: String },
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
        Term::Done => task.is_some_and(|t| t.done),
        Term::Overdue => {
            task.is_some_and(|t| !t.done && t.due.as_ref().is_some_and(|d| d.date() < today))
        }
        Term::Kind(kind) => object.kind() == *kind,
        Term::Due(sel) => {
            task.is_some_and(|t| date_matches(sel, t.due.as_ref().map(|d| d.date()), today))
        }
        Term::Deadline(sel) => task.is_some_and(|t| date_matches(sel, t.deadline, today)),
        Term::Start(sel) => event.is_some_and(|e| date_matches(sel, Some(e.start.date()), today)),
        Term::Path(prefix) => object.base().path.is_within(prefix),
        Term::Label(name) | Term::Category { value: name, .. } => {
            object.base().labels.contains(name)
        }
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
mod tests {
    use super::*;
    use crate::{Event, Object, Oid, Path, Priority, Task, Transparency, When};
    use jiff::civil::date;

    fn oid(b: u8) -> Oid {
        Oid::generate(&mut |x: &mut [u8]| x.fill(b))
    }

    fn task(due: Option<Date>, done: bool) -> Object {
        let mut t = Task::new(oid(1), "x");
        t.due = due.map(When::Day);
        t.done = done;
        Object::Task(t)
    }

    const TODAY: Date = date(2026, 9, 18);

    fn hit(query: &str, object: &Object) -> bool {
        matches(&parse(query).unwrap(), object, TODAY)
    }

    #[test]
    fn due_today_and_overdue() {
        assert!(hit("due:today", &task(Some(TODAY), false)));
        assert!(!hit("due:today", &task(Some(date(2026, 9, 17)), false)));
        assert!(hit("overdue", &task(Some(date(2026, 9, 17)), false)));
        assert!(!hit("overdue", &task(Some(date(2026, 9, 17)), true)));
        assert!(!hit("overdue", &task(None, false)));
    }

    #[test]
    fn or_and_not_and_parentheses() {
        let yesterday = task(Some(date(2026, 9, 17)), false);
        assert!(hit("due:today | overdue", &yesterday));
        assert!(!hit("due:today & overdue", &yesterday));
        assert!(hit("!done", &yesterday));
        assert!(hit("(due:today | overdue) & !done", &yesterday));
    }

    #[test]
    fn this_week_is_monday_to_sunday_of_today() {
        // 2026-09-18 is a Friday; the week is 14 to 20
        assert!(hit("due:this-week", &task(Some(date(2026, 9, 14)), false)));
        assert!(hit("due:this-week", &task(Some(date(2026, 9, 20)), false)));
        assert!(!hit("due:this-week", &task(Some(date(2026, 9, 21)), false)));
        assert!(hit("due:next-week", &task(Some(date(2026, 9, 21)), false)));
    }

    #[test]
    fn path_is_a_prefix_match_and_labels_and_priority_and_subject() {
        let mut t = Task::new(oid(2), "Buy oat milk");
        t.base.path = Path::parse("webdavis/dotfiles").unwrap();
        t.base.labels.insert("deep".into());
        t.priority = Priority::HIGHEST;
        let o = Object::Task(t);
        assert!(hit("path:webdavis/", &o));
        assert!(!hit("path:other/", &o));
        assert!(hit("@deep", &o) && hit("label:deep", &o) && hit("effort:deep", &o));
        assert!(hit("p1", &o) && hit("priority:1", &o) && !hit("p2", &o));
        assert!(hit("subject:oat", &o) && hit("subject:OAT", &o));
    }

    #[test]
    fn kind_start_transparency_and_attached() {
        let mut e = Event::new(
            oid(3),
            "meet",
            When::Day(TODAY),
            When::Day(date(2026, 9, 19)),
        );
        e.transparency = Transparency::Free;
        let event = Object::Event(e);
        assert!(hit("kind:event", &event) && !hit("kind:task", &event));
        assert!(hit("start:today", &event));
        assert!(hit("transparency:free", &event));
        let mut t = Task::new(oid(4), "prep");
        t.event = Some(oid(3));
        let attached = Object::Task(t);
        assert!(hit(&format!("attached:{}", oid(3)), &attached));
    }

    #[test]
    fn due_none_and_before_after_and_on() {
        assert!(hit("due:none", &task(None, false)));
        assert!(hit(
            "due:before:2026-09-18",
            &task(Some(date(2026, 9, 17)), false)
        ));
        assert!(hit(
            "due:after:2026-09-18",
            &task(Some(date(2026, 9, 19)), false)
        ));
        assert!(hit("due:2026-09-19", &task(Some(date(2026, 9, 19)), false)));
    }

    #[test]
    fn a_date_term_on_the_wrong_kind_never_matches() {
        let event = Object::Event(Event::new(
            oid(5),
            "meet",
            When::Day(TODAY),
            When::Day(TODAY),
        ));
        assert!(!hit("due:none", &event));
        assert!(!hit("deadline:none", &event));
        assert!(hit("due:none", &task(None, false)));
        let t = Object::Task(Task::new(oid(6), "x"));
        assert!(!hit("start:none", &t));
    }

    /// The parser cannot know a user's declared categories, so an unknown key parses as a
    /// `Term::Category` a caller can refuse by checking `name` against `Categories`.
    #[test]
    fn an_unrecognized_key_parses_as_a_category_term_a_caller_can_check() {
        let expr = parse("pat:work/").unwrap();
        match expr {
            Expr::Term(Term::Category { name, value }) => {
                assert_eq!(name, "pat");
                assert_eq!(value, "work/");
                assert!(
                    crate::Categories::default()
                        .category_of_name(&name)
                        .is_none()
                );
            }
            other => panic!("{other:?}"),
        }
    }

    #[test]
    fn errors_are_named() {
        assert_eq!(parse(""), Err(QueryError::Empty));
        assert_eq!(parse("(due:today"), Err(QueryError::Unclosed));
        assert_eq!(
            parse("due:someday"),
            Err(QueryError::BadValue {
                key: "due".into(),
                value: "someday".into()
            })
        );
        assert_eq!(
            parse("due:today |"),
            Err(QueryError::Unexpected("end".into()))
        );
    }

    /// The query is operator-controlled argv, so this is a refusal rather
    /// than a defence, but a deep enough one used to abort the process.
    #[test]
    fn nesting_past_the_bound_is_refused_instead_of_recursed_into() {
        let deep = format!("{}due:today{}", "(".repeat(200), ")".repeat(200));
        assert_eq!(parse(&deep), Err(QueryError::TooDeep));
        let negated = format!("{}due:today", "!".repeat(200));
        assert_eq!(parse(&negated), Err(QueryError::TooDeep));
    }

    #[test]
    fn nesting_up_to_the_bound_still_parses() {
        let ok = format!("{}due:today{}", "(".repeat(64), ")".repeat(64));
        assert!(parse(&ok).is_ok(), "64 levels should parse");
    }
}
