use super::differing::{event_differing_in_every_field, oid, task_differing_in_every_field};
use super::*;
use crate::{Completion, Event, Field, Kind, Object, Priority, Task, When};
use jiff::civil::date;

#[test]
fn only_named_fields_come_from_theirs() {
    let mut ours = Task::new(oid(1), "ours");
    ours.priority = Priority::HIGHEST;
    ours.base.depends.push(oid(9));
    let mut theirs = Task::new(oid(1), "theirs");
    theirs.priority = Priority::LOWEST;
    let out = merge_fields(
        &Object::Task(ours),
        &Object::Task(theirs),
        &[Field::Subject],
    );
    let t = out.as_task().unwrap();
    assert_eq!(t.base.subject, "theirs");
    assert_eq!(t.priority, Priority::HIGHEST);
    assert_eq!(t.base.depends, vec![oid(9)]);
}

#[test]
fn a_kind_mismatch_keeps_ours_entirely_and_is_reported() {
    let ours = Object::Task(Task::new(oid(1), "ours"));
    let theirs = Object::Event(Event::new(
        oid(1),
        "e",
        When::Day(date(2026, 1, 1)),
        When::Day(date(2026, 1, 2)),
    ));
    assert_eq!(merge_fields(&ours, &theirs, &[Field::Subject]), ours);
    assert_eq!(kind_change(&ours, &theirs), Some((Kind::Task, Kind::Event)));
    assert_eq!(kind_change(&ours, &ours), None);
}

#[test]
fn every_task_field_moves_alone_and_nothing_else_does() {
    let ours = Object::Task(task_differing_in_every_field("a"));
    let theirs = Object::Task(task_differing_in_every_field("b"));
    let fields = changed_fields(&ours, &theirs);
    assert_eq!(
        fields.len(),
        12,
        "expected every task field to differ: {fields:?}"
    );
    for f in &fields {
        let merged = merge_fields(&ours, &theirs, &[*f]);
        assert_eq!(changed_fields(&ours, &merged), vec![*f], "field {f}");
    }
}

#[test]
fn every_event_field_moves_alone_and_nothing_else_does() {
    let ours = Object::Event(event_differing_in_every_field("a"));
    let theirs = Object::Event(event_differing_in_every_field("b"));
    let fields = changed_fields(&ours, &theirs);
    assert_eq!(
        fields.len(),
        20,
        "expected every event field to differ: {fields:?}"
    );
    for f in &fields {
        let merged = merge_fields(&ours, &theirs, &[*f]);
        assert_eq!(changed_fields(&ours, &merged), vec![*f], "field {f}");
    }
}

#[test]
fn a_reopen_from_a_remote_clears_the_completion_time() {
    let mut ours = Task::new(oid(1), "t");
    ours.completion = Completion::Done {
        at: Some(jiff::Timestamp::UNIX_EPOCH),
    };
    let theirs = Task::new(oid(1), "t");
    let merged = merge_fields(&Object::Task(ours), &Object::Task(theirs), &[Field::Done]);
    let merged = merged.as_task().unwrap();
    assert_eq!(merged.completion, Completion::Open);
}

#[test]
fn a_remote_that_agrees_it_is_done_leaves_the_completion_time_alone() {
    let mut ours = Task::new(oid(1), "t");
    ours.completion = Completion::Done {
        at: Some(jiff::Timestamp::UNIX_EPOCH),
    };
    let mut theirs = Task::new(oid(1), "t");
    theirs.completion = Completion::Done { at: None };
    let merged = merge_fields(&Object::Task(ours), &Object::Task(theirs), &[Field::Done]);
    assert_eq!(
        merged.as_task().unwrap().completion,
        Completion::Done {
            at: Some(jiff::Timestamp::UNIX_EPOCH)
        }
    );
}

#[test]
fn a_remote_completion_time_replaces_the_local_one() {
    let mut ours = Task::new(oid(1), "t");
    ours.completion = Completion::Done {
        at: Some(jiff::Timestamp::UNIX_EPOCH),
    };
    let mut theirs = Task::new(oid(1), "t");
    theirs.completion = Completion::Done {
        at: Some(jiff::Timestamp::MAX),
    };
    let merged = merge_fields(&Object::Task(ours), &Object::Task(theirs), &[Field::Done]);
    assert_eq!(
        merged.as_task().unwrap().completion,
        Completion::Done {
            at: Some(jiff::Timestamp::MAX)
        }
    );
}

#[test]
fn a_remote_completion_time_is_kept_when_the_task_was_open() {
    let ours = Task::new(oid(1), "t");
    let mut theirs = Task::new(oid(1), "t");
    theirs.completion = Completion::Done {
        at: Some(jiff::Timestamp::MAX),
    };
    let merged = merge_fields(&Object::Task(ours), &Object::Task(theirs), &[Field::Done]);
    assert_eq!(
        merged.as_task().unwrap().completion,
        Completion::Done {
            at: Some(jiff::Timestamp::MAX)
        }
    );
}
