use super::*;
use crate::{Categories, Path, Task};
use jiff::civil::date;
use jiff::tz::TimeZone;

fn kitchen_task() -> Object {
    let mut t = Task::new(
        Oid::generate(&mut |x: &mut [u8]| x.fill(1)),
        "get \"best\" quotes",
    );
    t.base.path = Path::parse("kitchen remodel/tiles").unwrap();
    Object::Task(t)
}

fn hit(query: &str) -> bool {
    let expr = parse(query, &Categories::default()).unwrap();
    matches(&expr, &kitchen_task(), date(2026, 9, 18), &TimeZone::UTC)
}

#[test]
fn a_quoted_value_may_hold_spaces() {
    assert!(hit("path:\"kitchen remodel/\""));
    assert!(hit("subject:\"get \\\"best\" & !done"));
    assert!(!hit("path:\"other place/\""));
}

#[test]
fn a_quoted_bare_word_works_where_a_bare_word_works() {
    assert!(hit("!\"done\""));
    assert!(hit("\"subject:get\""));
}

#[test]
fn a_backslash_inside_quotes_keeps_the_next_character() {
    assert!(hit("subject:\"\\\"best\\\"\""));
    assert!(!hit("subject:\"\\\\\""));
}

#[test]
fn an_unclosed_quote_names_what_was_typed() {
    let err = parse("path:\"kitchen remodel/", &Categories::default()).unwrap_err();
    assert_eq!(err, QueryError::UnclosedQuote("\"kitchen remodel/".into()));
    assert_eq!(
        err.to_string(),
        "the quote in \"kitchen remodel/ is not closed"
    );
}
