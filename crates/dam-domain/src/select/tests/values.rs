use super::fake::Fake;
use super::objects::{event, oid, task};
use crate::select::{Condition, Problem};
use crate::{Object, Path, Priority};

fn fake() -> Fake {
    Fake::at("2026-11-20T12:00[UTC]")
}

fn holds(key: &str, value: &str, object: &Object) -> bool {
    let condition = Condition::read(key, value).unwrap_or_else(|e| panic!("{e:?}"));
    condition.holds(object, &fake())
}

fn refused(key: &str, value: &str) -> Problem {
    let error = Condition::read(key, value).expect_err(value);
    assert_eq!((error.field.as_str(), error.value.as_str()), (key, value));
    assert!(!error.shapes().is_empty());
    error.problem
}

#[test]
fn text_holds_what_contains_it_in_any_case() {
    let mut t = task(1);
    t.base.subject = "Buy oat milk, now".into();
    let t = Object::Task(t);
    assert!(holds("subject", "OAT", &t));
    assert!(holds("subject", "milk, now", &t));
    assert!(!holds("subject", "rice", &t));
    assert_eq!(refused("subject", ""), Problem::BadValue);
}

#[test]
fn a_choice_is_one_of_the_listed_values() {
    let e = Object::Event(event(1, "2026-11-20T09:00[UTC]"));
    assert!(holds("transparency", "busy", &e));
    assert!(holds("transparency", "free,busy", &e));
    assert!(holds("kind", "event", &e) && holds("kind", "any", &e));
    assert!(!holds("kind", "task", &e));
    assert_eq!(refused("transparency", "opaque"), Problem::BadValue);
    assert_eq!(refused("kind", "action"), Problem::BadValue);
}

#[test]
fn a_number_is_a_value_a_list_or_a_span_with_both_ends_included() {
    let mut t = task(1);
    t.priority = Priority::new(2).unwrap();
    let t = Object::Task(t);
    for yes in ["2", "1,2", "..2", "2..", "1..3"] {
        assert!(holds("priority", yes, &t), "{yes}");
    }
    for no in ["1", "3,4", "..1", "3.."] {
        assert!(!holds("priority", no, &t), "{no}");
    }
    for bad in ["5", "0", "p1", "1..x", "..", "high"] {
        assert_eq!(refused("priority", bad), Problem::BadValue, "{bad}");
    }
}

#[test]
fn a_length_reads_minutes_and_hours() {
    let mut t = task(1);
    t.estimate = Some(45);
    let t = Object::Task(t);
    assert!(holds("estimate", "30m..2h", &t) && holds("estimate", "45", &t));
    assert!(!holds("estimate", "..5m", &t) && !holds("estimate", "1h..", &t));
    assert!(!holds("estimate", "none", &t));
    assert!(holds("estimate", "none", &Object::Task(task(2))));
    assert_eq!(refused("estimate", "an hour"), Problem::BadValue);
}

#[test]
fn a_set_holds_exactly_that_value() {
    let t = super::objects::labelled(Object::Task(task(1)), &["bug", "ui"]);
    assert!(holds("labels", "bug", &t) && holds("labels", "nope,ui", &t));
    assert!(!holds("labels", "bu", &t));
    assert_eq!(refused("labels", ""), Problem::BadValue);
}

#[test]
fn a_path_holds_the_folder_and_everything_under_it() {
    let mut t = task(1);
    t.base.path = Path::parse("work/dam/cli").unwrap();
    let t = Object::Task(t);
    assert!(holds("path", "work/", &t) && holds("path", "home/,work/dam/", &t));
    assert!(!holds("path", "home/", &t));
    assert_eq!(refused("path", "a//b"), Problem::BadValue);
    assert_eq!(refused("path", "../x"), Problem::BadValue);
}

#[test]
fn a_reference_is_an_id_none_or_the_objects_of_a_filter() {
    let mut t = task(1);
    t.base.parent = Some(oid(9));
    let t = Object::Task(t);
    assert!(holds("parent", oid(9).as_str(), &t));
    assert!(!holds("parent", "none", &t));
    assert!(holds("parent", "none", &Object::Task(task(2))));
    let mut fake = fake();
    fake.listed.entry("hot".into()).or_default().insert(oid(9));
    assert!(
        Condition::read("parent", "in:hot")
            .unwrap()
            .holds(&t, &fake)
    );
    assert!(
        !Condition::read("parent", "in:cold")
            .unwrap()
            .holds(&t, &fake)
    );
    assert_eq!(refused("parent", "nope"), Problem::BadValue);
}

#[test]
fn yes_or_no_reads_true_false_or_any() {
    let t = Object::Task(task(1));
    assert!(holds("done", "false", &t) && holds("done", "any", &t));
    assert!(!holds("done", "true", &t));
    assert_eq!(refused("done", "yes"), Problem::BadValue);
    assert_eq!(refused("done", "not:true"), Problem::BadValue);
}

#[test]
fn a_date_takes_one_value_and_refuses_a_list() {
    assert_eq!(refused("due", "today,tomorrow"), Problem::BadValue);
    assert_eq!(refused("start", "none"), Problem::BadValue);
}

#[test]
fn an_unknown_field_names_the_closest_one() {
    let error = Condition::read("lables", "bug").unwrap_err();
    assert_eq!(
        error.problem,
        Problem::UnknownField {
            closest: Some("labels")
        }
    );
}
