use super::*;
use dam_domain::{
    Attendee, Event, Object, Oid, Priority, ResponseStatus, Task, Transparency, When,
};
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
fn a_completion_time_round_trips_as_rfc_3339_since_the_store_is_wire_json() {
    let mut t = Task::new(oid(1), "milk");
    t.done = true;
    t.completed_at = Some("2026-09-18T15:04:05Z".parse().unwrap());
    let object = Object::Task(t);
    let wire = to_wire(&object, None);
    assert_eq!(
        wire.task.as_ref().unwrap().completed_at.as_deref(),
        Some("2026-09-18T15:04:05Z")
    );
    assert_eq!(from_wire(&wire).unwrap(), object);
}

#[test]
fn an_absent_completion_time_as_an_older_helper_sends_reads_as_none_and_is_left_out() {
    let object = Object::Task(Task::new(oid(1), "milk"));
    let wire = to_wire(&object, None);
    assert_eq!(wire.task.as_ref().unwrap().completed_at, None);
    let text = serde_json::to_string(&wire).unwrap();
    assert!(!text.contains("completed_at"), "{text}");
    assert_eq!(from_wire(&wire).unwrap(), object);
}

#[test]
fn an_open_task_does_not_take_a_completion_time_from_the_wire() {
    let mut wire = to_wire(&Object::Task(Task::new(oid(1), "milk")), None);
    let task = wire.task.as_mut().unwrap();
    task.done = false;
    task.completed_at = Some("2020-01-01T00:00:00Z".into());
    let object = from_wire(&wire).unwrap();
    let task = object.as_task().unwrap();
    assert!(!task.done);
    assert_eq!(task.completed_at, None);
}

#[test]
fn an_unreadable_completion_time_is_rejected_naming_the_field() {
    let mut done = Task::new(oid(1), "milk");
    done.done = true;
    let mut wire = to_wire(&Object::Task(done), None);
    wire.task.as_mut().unwrap().completed_at = Some("last tuesday".into());
    let err = from_wire(&wire).unwrap_err();
    assert_eq!(err.field(), "completed_at");
    assert!(err.to_string().contains("last tuesday"), "{err}");
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

#[test]
fn each_rejection_class_is_distinguishable_without_matching_a_sentence() {
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

#[test]
fn a_pull_hands_its_cancellations_on() {
    let response = dam_protocol::PullResponse {
        cancelled: vec!["primary/e1".into()],
        ..Default::default()
    };
    assert_eq!(
        pull_from_wire(response).cancelled,
        vec!["primary/e1".to_string()]
    );
}

#[test]
fn the_calendars_own_attendee_round_trips() {
    let mut event = dam_domain::Event::new(
        oid(1),
        "standup",
        When::Day(date(2026, 9, 25)),
        When::Day(date(2026, 9, 26)),
    );
    event.attendees = vec![
        Attendee {
            email: "me@x".into(),
            response: ResponseStatus::Declined,
            is_self: true,
        },
        Attendee {
            email: "you@x".into(),
            response: ResponseStatus::Accepted,
            is_self: false,
        },
    ];
    let object = Object::Event(event);
    assert_eq!(from_wire(&to_wire(&object, None)).unwrap(), object);
}

#[test]
fn a_kind_or_field_this_dam_does_not_know_is_dropped_rather_than_refused() {
    let caps = capabilities_from_wire(&dam_protocol::Capabilities {
        protocol: dam_protocol::PROTOCOL_VERSION,
        kinds: vec!["task".into(), "note".into()],
        fields: vec!["subject".into(), "mood".into()],
        credentials: vec!["api_token".into()],
        incremental: true,
    });
    assert_eq!(caps.kinds, vec![dam_domain::Kind::Task]);
    assert_eq!(caps.fields, vec![dam_domain::Field::Subject]);
    assert_eq!(caps.credentials, vec!["api_token".to_string()]);
    assert!(caps.incremental);
}

#[test]
fn a_pulled_object_with_no_remote_id_is_rejected_under_an_empty_name() {
    let response = dam_protocol::PullResponse {
        objects: vec![to_wire(&Object::Task(Task::new(oid(1), "milk")), None)],
        ..Default::default()
    };
    let outcome = pull_from_wire(response);
    assert!(outcome.objects.is_empty());
    assert_eq!(
        outcome.rejected,
        vec![dam_application::RejectedObject {
            remote_id: String::new(),
            why: "the object has no remote_id".into(),
        }]
    );
}

#[test]
fn a_push_result_naming_no_valid_oid_is_dropped_since_no_change_can_own_it() {
    let result = |oid: String, remote_id: &str| dam_protocol::MutationResult {
        oid,
        ok: true,
        remote_id: Some(remote_id.into()),
        why: None,
    };
    let outcomes = outcomes_from_wire(vec![
        result("nothex".into(), "r0"),
        result(oid(2).to_string(), "r2"),
    ]);
    assert_eq!(outcomes.len(), 1);
    assert_eq!(outcomes[0].oid, oid(2));
    assert_eq!(outcomes[0].remote_id.as_deref(), Some("r2"));
}
