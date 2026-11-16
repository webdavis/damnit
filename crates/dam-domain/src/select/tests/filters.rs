use super::fake::Fake;
use super::objects::{labelled, task};
use super::tables::{either, many, one, table};
use crate::select::{Filter, FilterTable, Problem};
use crate::{Object, Priority};

fn hit(t: &FilterTable, object: &Object) -> bool {
    let fake = Fake::at("2026-11-20T12:00[UTC]");
    Filter::read(t).unwrap().matches(object, &fake)
}

#[test]
fn every_condition_must_hold() {
    let mut t = task(1);
    t.priority = Priority::HIGHEST;
    let t = labelled(Object::Task(t), &["bug"]);
    assert!(hit(
        &table(&[("labels", one("bug")), ("priority", one("1"))]),
        &t
    ));
    assert!(!hit(
        &table(&[("labels", one("bug")), ("priority", one("2"))]),
        &t
    ));
}

#[test]
fn a_repeated_field_holds_every_value() {
    let both = labelled(Object::Task(task(1)), &["bug", "ui"]);
    let bug = labelled(Object::Task(task(2)), &["bug"]);
    let filter = table(&[("labels", many(&["bug", "ui"]))]);
    assert!(hit(&filter, &both) && !hit(&filter, &bug));
    let narrowed = table(&[("labels", many(&["bug", "not:waiting"]))]);
    assert!(hit(&narrowed, &bug));
}

#[test]
fn one_field_asked_to_be_two_things_is_refused_with_the_list_to_use() {
    let error = Filter::read(&table(&[("priority", many(&["1", "2"]))])).unwrap_err();
    assert_eq!(error.field, "priority");
    assert_eq!(
        error.problem,
        Problem::TwoAtOnce {
            fix: Some("1,2".into())
        }
    );
    let dates = Filter::read(&table(&[("due", many(&["today", "tomorrow"]))])).unwrap_err();
    assert_eq!(dates.problem, Problem::TwoAtOnce { fix: None });
}

#[test]
fn a_repeat_that_narrows_or_repeats_the_same_value_is_kept() {
    for values in [
        &["1", "1"][..],
        &["..2", "2.."],
        &["1", "not:2"],
        &["1,2", "2,3"],
    ] {
        assert!(Filter::read(&table(&[("priority", many(values))])).is_ok());
    }
    assert!(Filter::read(&table(&[("due", many(&["week", "not:today"]))])).is_ok());
    assert!(Filter::read(&table(&[("subject", many(&["oat", "milk"]))])).is_ok());
}

#[test]
fn an_empty_list_is_refused() {
    let error = Filter::read(&table(&[("labels", many(&[]))])).unwrap_err();
    assert_eq!(error.problem, Problem::BadValue);
}

#[test]
fn reading_then_writing_gives_the_same_table() {
    let nested = either(
        table(&[("path", one("work/"))]),
        vec![
            table(&[("labels", one("urgent"))]),
            either(
                table(&[("due", one("..today"))]),
                vec![
                    table(&[("priority", one("1,2"))]),
                    table(&[("labels", many(&["a", "not:=none"]))]),
                ],
            ),
        ],
    );
    for t in [
        table(&[]),
        table(&[
            ("labels", many(&["bug", "not:waiting"])),
            ("due", one("week")),
        ]),
        table(&[("category", one("effort=deep,quick")), ("kind", one("any"))]),
        nested,
    ] {
        assert_eq!(Filter::read(&t).unwrap().write(), t);
    }
}

#[test]
fn an_unknown_field_inside_any_is_named() {
    let t = either(table(&[]), vec![table(&[("lables", one("x"))])]);
    let error = Filter::read(&t).unwrap_err();
    assert_eq!(
        error.problem,
        Problem::UnknownField {
            closest: Some("labels")
        }
    );
}
