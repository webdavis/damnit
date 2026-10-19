use super::*;
use dam_application::{MutationOp, RemoteMutation};
use dam_domain::{
    Attendee, Event, EventStatus, EventType, Field, Kind, Object, Oid, ResponseStatus, Task,
    Transparency, Visibility, When,
};
use dam_protocol::{WireEvent, WireObject};
use jiff::civil::date;

fn oid() -> Oid {
    Oid::generate(&mut |x: &mut [u8]| x.fill(2))
}

fn event(edit: impl FnOnce(&mut Event)) -> Object {
    let day = When::Day(date(2026, 9, 25));
    let mut e = Event::new(oid(), "dentist", day.clone(), day);
    edit(&mut e);
    Object::Event(e)
}

fn wire_event(wire: &WireObject) -> &WireEvent {
    wire.event.as_ref().unwrap()
}

fn words_after_a_round_trip<T>(
    all: impl IntoIterator<Item = T>,
    object: impl Fn(T) -> Object,
    word: impl Fn(&WireObject) -> &str,
) -> Vec<String> {
    all.into_iter()
        .map(|value| {
            let object = object(value);
            let wire = to_wire(&object, None);
            assert_eq!(from_wire(&wire).unwrap(), object);
            word(&wire).to_string()
        })
        .collect()
}

#[test]
fn every_kind_round_trips_through_the_wire() {
    let words = words_after_a_round_trip(
        Kind::ALL,
        |kind| match kind {
            Kind::Task => Object::Task(Task::new(oid(), "milk")),
            Kind::Event => event(|_| {}),
        },
        |w| &w.kind,
    );
    assert_eq!(words, ["task", "event"]);
}

#[test]
fn every_event_status_round_trips_through_the_wire() {
    let words = words_after_a_round_trip(
        EventStatus::ALL,
        |status| event(|e| e.status = status),
        |w| &wire_event(w).status,
    );
    assert_eq!(words, ["confirmed", "tentative", "cancelled"]);
}

#[test]
fn every_transparency_round_trips_through_the_wire() {
    let words = words_after_a_round_trip(
        Transparency::ALL,
        |transparency| event(|e| e.transparency = transparency),
        |w| &wire_event(w).transparency,
    );
    assert_eq!(words, ["busy", "free"]);
}

#[test]
fn every_visibility_round_trips_through_the_wire() {
    let words = words_after_a_round_trip(
        Visibility::ALL,
        |visibility| event(|e| e.visibility = visibility),
        |w| &wire_event(w).visibility,
    );
    assert_eq!(words, ["default", "public", "private", "confidential"]);
}

#[test]
fn every_event_type_round_trips_through_the_wire() {
    let words = words_after_a_round_trip(
        EventType::ALL,
        |event_type| event(|e| e.event_type = event_type),
        |w| &wire_event(w).event_type,
    );
    assert_eq!(
        words,
        [
            "default",
            "focus_time",
            "out_of_office",
            "working_location",
            "birthday"
        ]
    );
}

#[test]
fn every_attendee_response_round_trips_through_the_wire() {
    let words = words_after_a_round_trip(
        ResponseStatus::ALL,
        |response| {
            event(|e| {
                e.attendees = vec![Attendee {
                    email: "me@example.com".into(),
                    response,
                    is_self: true,
                }]
            })
        },
        |w| &wire_event(w).attendees[0].response,
    );
    assert_eq!(words, ["needs_action", "accepted", "declined", "tentative"]);
}

#[test]
fn every_field_a_mutation_names_reads_back_from_a_helpers_capabilities() {
    let mutation = mutation_to_wire(RemoteMutation {
        op: MutationOp::Update,
        oid: oid(),
        idempotency_key: "k".into(),
        remote_id: Some("r1".into()),
        object: None,
        fields: Field::ALL.to_vec(),
    });
    let caps = capabilities_from_wire(&dam_protocol::Capabilities {
        protocol: dam_protocol::PROTOCOL_VERSION,
        kinds: vec![],
        fields: mutation.fields,
        credentials: vec![],
        incremental: false,
    });
    assert_eq!(caps.fields, Field::ALL);
}
