use super::*;
use crate::{Categories, Category, Event, Object, Oid, Path, Priority, Task, Transparency, When};
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

fn effort() -> Categories {
    Categories::new(vec![Category {
        name: "effort".into(),
        values: vec!["light".into(), "deep".into()],
        exclusive: true,
    }])
    .unwrap()
}

fn parse(query: &str) -> Result<Expr, QueryError> {
    super::parse(query, &effort())
}

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
    assert_eq!(TODAY.weekday(), Weekday::Friday);
    let (monday, sunday, next_monday) = (date(2026, 9, 14), date(2026, 9, 20), date(2026, 9, 21));
    assert!(hit("due:this-week", &task(Some(monday), false)));
    assert!(hit("due:this-week", &task(Some(sunday), false)));
    assert!(!hit("due:this-week", &task(Some(next_monday), false)));
    assert!(hit("due:next-week", &task(Some(next_monday), false)));
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

#[test]
fn a_category_term_matches_only_a_value_of_that_category() {
    let mut t = Task::new(oid(7), "x");
    t.base.labels.insert("home".into());
    let o = Object::Task(t);
    assert!(hit("@home", &o));
    assert_eq!(
        parse("effort:home"),
        Err(QueryError::NotAValue {
            category: "effort".into(),
            value: "home".into()
        })
    );
    assert_eq!(
        parse("mood:happy"),
        Err(QueryError::UnknownCategory("mood".into()))
    );
    assert_eq!(
        parse("effort:home").unwrap_err().to_string(),
        "home is not a value of category effort"
    );
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
    assert_eq!(parse("due:today |"), Err(QueryError::EndsEarly));
}

#[test]
fn an_error_names_what_was_typed_in_plain_words() {
    let said = |q: &str| parse(q).unwrap_err().to_string();
    assert_eq!(
        said("due:today due:tomorrow"),
        "expected & or | before due:tomorrow"
    );
    assert_eq!(said("p1 &"), "the query ends where a term was expected");
    assert_eq!(said("& p1"), "unexpected & in query");
    assert_eq!(said("p1)"), "unexpected ) in query");
    assert_eq!(said("due:someday"), "due: cannot read \"someday\"");
}

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
