use super::*;
use crate::ports::ObjectRepository;
use crate::testing::{Scenario, event, task};
use crate::use_cases::edit::EditFields;
use jiff::civil::date;

fn written(landed: Landed) -> Object {
    match landed {
        Landed::Written(object) => object,
        Landed::Sent(sent) => panic!("the owner's object was sent as {sent:?}"),
    }
}

#[test]
fn a_new_task_takes_the_fields_edit_sets() {
    let s = Scenario::default().drawing_from(7);
    s.working(task(1));
    s.working(event(2));
    let set = EditFields {
        recurrence: Some(Some("every week".into())),
        estimate: Some(Some(30)),
        add_depends: vec![oid(1)],
        attach: Some(Some(oid(2))),
        focus: Some(Some(date(2026, 9, 25))),
        ..EditFields::default()
    };
    let made = written(s.dam().new_task_setting(input("milk", &[]), set).unwrap());
    let task = made.as_task().unwrap();
    assert!(task.base.recurrence.is_some());
    assert_eq!(task.estimate, Some(30));
    assert_eq!(task.base.depends, vec![oid(1)]);
    assert_eq!(task.event, Some(oid(2)));
    assert_eq!(task.focus.as_ref().map(|f| f.day), Some(date(2026, 9, 25)));
    assert_eq!(s.store.get(made.oid()).unwrap(), Some(made));
}

#[test]
fn a_new_event_takes_a_location() {
    let s = Scenario::default().drawing_from(7);
    let day = When::Day(date(2026, 9, 25));
    let set = EditFields {
        location: Some(Some("room 4".into())),
        ..EditFields::default()
    };
    let made = written(
        s.dam()
            .new_event_setting(dentist(day.clone(), day), set)
            .unwrap(),
    );
    let Object::Event(event) = made else {
        panic!("not an event")
    };
    assert_eq!(event.location.as_deref(), Some("room 4"));
}

#[test]
fn a_new_task_attached_to_no_event_is_refused_and_writes_nothing() {
    let s = Scenario::default().drawing_from(7);
    s.working(task(1));
    let set = EditFields {
        attach: Some(Some(oid(1))),
        ..EditFields::default()
    };
    assert!(s.dam().new_task_setting(input("milk", &[]), set).is_err());
    assert_eq!(s.store.all().unwrap().len(), 1);
}
