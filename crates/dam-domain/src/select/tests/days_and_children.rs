use super::fake::Fake;
use super::objects::{event, oid, task};
use crate::select::{OpenChildren, worked_out};
use crate::{Completion, Object, When};

fn due_on(day: (i16, i8, i8)) -> Object {
    let mut t = task(1);
    t.due = Some(When::Day(jiff::civil::date(day.0, day.1, day.2)));
    Object::Task(t)
}

fn child_of(b: u8, parent: u8, done: bool) -> Object {
    let mut t = task(b);
    t.base.parent = Some(oid(parent));
    if done {
        t.completion = Completion::Done { at: None };
    }
    Object::Task(t)
}

#[test]
fn days_until_due_counts_calendar_days_from_today() {
    let fake = Fake::at("2026-11-20T23:30[UTC]");
    let days = |day| worked_out(&due_on(day), &fake).days_until_due;
    assert_eq!(days((2026, 11, 20)), Some(0));
    assert_eq!(days((2026, 11, 21)), Some(1));
    assert_eq!(days((2026, 11, 18)), Some(-2));
    assert_eq!(days((2027, 1, 1)), Some(42));
}

#[test]
fn a_due_with_a_time_counts_by_its_day_in_the_persons_zone() {
    let fake = Fake::at("2026-11-20T12:00[America/New_York]");
    let mut t = task(1);
    t.due = Some(When::At("2026-11-21T03:00[UTC]".parse().unwrap()));
    let got = worked_out(&Object::Task(t), &fake).days_until_due;
    assert_eq!(got, Some(0));
}

#[test]
fn days_until_deadline_counts_to_the_deadline_and_a_done_task_keeps_both() {
    let fake = Fake::at("2026-11-20T12:00[UTC]");
    let mut t = task(1);
    t.deadline = Some(jiff::civil::date(2026, 11, 23));
    t.due = Some(When::Day(jiff::civil::date(2026, 11, 19)));
    t.completion = Completion::Done { at: None };
    let got = worked_out(&Object::Task(t), &fake);
    assert_eq!(got.days_until_deadline, Some(3));
    assert_eq!(got.days_until_due, Some(-1));
}

#[test]
fn a_task_without_dates_and_an_event_have_no_days() {
    let fake = Fake::at("2026-11-20T12:00[UTC]");
    for object in [
        Object::Task(task(1)),
        Object::Event(event(2, "2026-11-21T09:00[UTC]")),
    ] {
        let got = worked_out(&object, &fake);
        assert_eq!(got.days_until_due, None);
        assert_eq!(got.days_until_deadline, None);
    }
}

#[test]
fn open_children_counts_each_parents_children_that_are_tasks_not_done() {
    let objects = [
        child_of(2, 1, false),
        child_of(3, 1, false),
        child_of(4, 1, true),
        child_of(5, 2, false),
        Object::Event({
            let mut e = event(6, "2026-11-21T09:00[UTC]");
            e.base.parent = Some(oid(1));
            e
        }),
    ];
    let children = OpenChildren::of(objects.iter().filter_map(Object::as_task));
    assert_eq!(children.of_parent(&oid(1)), 2);
    assert_eq!(children.of_parent(&oid(2)), 1);
    assert_eq!(children.of_parent(&oid(5)), 0);
}

#[test]
fn children_open_is_printed_on_every_object_and_zero_when_none_are_open() {
    let mut fake = Fake::at("2026-11-20T12:00[UTC]");
    let objects = [child_of(2, 1, false), child_of(3, 7, false)];
    fake.children = OpenChildren::of(objects.iter().filter_map(Object::as_task));
    let parent = Object::Task(task(1));
    let lone = Object::Task(task(4));
    let meeting = Object::Event(event(7, "2026-11-21T09:00[UTC]"));
    assert_eq!(worked_out(&parent, &fake).children_open, Some(1));
    assert_eq!(worked_out(&lone, &fake).children_open, Some(0));
    assert_eq!(worked_out(&meeting, &fake).children_open, Some(1));
}

#[test]
fn the_counts_are_refused_as_fields_as_an_unknown_field_is() {
    for field in ["days_until_due", "days_until_deadline", "children_open"] {
        let refused = crate::select::Condition::read(field, "1").unwrap_err();
        assert!(
            matches!(refused.problem, crate::select::Problem::UnknownField { .. }),
            "{field}"
        );
    }
}
