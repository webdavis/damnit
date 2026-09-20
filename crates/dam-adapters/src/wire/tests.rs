use super::*;
use dam_domain::{Event, Object, Oid, Priority, Task, Transparency, When};
use jiff::civil::date;

fn oid(b: u8) -> Oid {
    Oid::generate(&mut |x: &mut [u8]| x.fill(b))
}

#[test]
fn a_task_round_trips_with_a_day_due() {
    let mut t = Task::new(oid(1), "milk");
    t.due = Some(When::Day(date(2026, 9, 25)));
    t.priority = Priority::HIGHEST;
    t.base.labels.insert("errand".into());
    let object = Object::Task(t);
    let wire = to_wire(&object, Some("r1".into()));
    assert_eq!(wire.kind, "task");
    assert_eq!(
        wire.task.as_ref().unwrap().due.as_deref(),
        Some("2026-09-25")
    );
    assert_eq!(wire.task.as_ref().unwrap().priority, 1);
    assert_eq!(from_wire(&wire).unwrap(), object);
}

#[test]
fn an_event_round_trips_with_a_zoned_start() {
    let start = date(2026, 9, 25)
        .at(14, 0, 0, 0)
        .in_tz("America/Denver")
        .unwrap();
    let end = date(2026, 9, 25)
        .at(15, 0, 0, 0)
        .in_tz("America/Denver")
        .unwrap();
    let mut e = Event::new(oid(2), "dentist", When::At(start), When::At(end));
    e.transparency = Transparency::Free;
    let object = Object::Event(e);
    let wire = to_wire(&object, None);
    assert_eq!(wire.kind, "event");
    assert_eq!(
        wire.event.as_ref().unwrap().start,
        "2026-09-25T14:00:00-06:00[America/Denver]"
    );
    assert_eq!(wire.event.as_ref().unwrap().transparency, "free");
    assert_eq!(from_wire(&wire).unwrap(), object);
}

#[test]
fn a_bad_date_is_a_date_rejection_that_names_the_field() {
    let mut wire = to_wire(&Object::Task(Task::new(oid(3), "x")), None);
    wire.task.as_mut().unwrap().due = Some("someday".into());
    let err = from_wire(&wire).unwrap_err();
    assert_eq!(
        err,
        WireError::Date {
            field: "due",
            value: "someday".into()
        }
    );
    assert_eq!(err.to_string(), "due: cannot read \"someday\"");
}

#[test]
fn an_unknown_kind_is_its_own_rejection() {
    let mut wire = to_wire(&Object::Task(Task::new(oid(4), "x")), None);
    wire.kind = "note".into();
    let err = from_wire(&wire).unwrap_err();
    assert_eq!(err, WireError::UnknownKind("note".into()));
    assert_eq!(err.to_string(), "kind: cannot read \"note\"");
}

/// Each of the four rejections answers which class it belongs to, so a
/// caller can branch or count by class rather than match on a sentence.
#[test]
fn each_rejection_class_is_distinguishable() {
    let good = to_wire(&Object::Task(Task::new(oid(5), "x")), None);

    let mut bad_oid = good.clone();
    bad_oid.oid = "nothex".into();
    assert!(matches!(
        from_wire(&bad_oid).unwrap_err(),
        WireError::Oid { field: "oid", .. }
    ));

    let mut bad_path = good.clone();
    bad_path.path = "a//b".into();
    assert!(matches!(
        from_wire(&bad_path).unwrap_err(),
        WireError::Path { .. }
    ));

    let mut bad_priority = good.clone();
    bad_priority.task.as_mut().unwrap().priority = 9;
    assert!(matches!(
        from_wire(&bad_priority).unwrap_err(),
        WireError::Priority { .. }
    ));

    let mut bad_depends = good.clone();
    bad_depends.depends = vec!["nope".into()];
    assert!(matches!(
        from_wire(&bad_depends).unwrap_err(),
        WireError::Oid {
            field: "depends",
            ..
        }
    ));
}

#[test]
fn an_unknown_event_word_names_the_field_it_came_from() {
    let start = date(2026, 9, 25)
        .at(14, 0, 0, 0)
        .in_tz("America/Denver")
        .unwrap();
    let object = Object::Event(Event::new(
        oid(6),
        "standup",
        When::At(start.clone()),
        When::At(start),
    ));
    let mut wire = to_wire(&object, None);
    wire.event.as_mut().unwrap().transparency = "opaque".into();
    assert_eq!(
        from_wire(&wire).unwrap_err(),
        WireError::UnknownEnum {
            field: "transparency",
            value: "opaque".into()
        }
    );
}
