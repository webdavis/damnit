use dam_domain::{Attendee, Conference, Event, Object, Oid, Person, ResponseStatus, When};
use serde_json::json;

use super::run_agenda;
use crate::args::AgendaArgs;
use crate::testing::context;

const AGENDA: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../docs/schemas/agenda.schema.json"
));
const OBJECT: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../docs/schemas/object.schema.json"
));

fn design_review() -> Event {
    let at = |t: &str| When::At(t.parse().unwrap());
    let mut e = Event::new(
        Oid::generate(&mut |b: &mut [u8]| b.fill(0x2c)),
        "design review",
        at("2026-09-18T14:00:00+00:00[UTC]"),
        at("2026-09-18T15:00:00+00:00[UTC]"),
    );
    e.attendees = vec![
        Attendee {
            email: "ada@example.com".into(),
            response: ResponseStatus::Accepted,
            is_self: true,
        },
        Attendee {
            email: "grace@example.com".into(),
            response: ResponseStatus::Tentative,
            is_self: false,
        },
    ];
    e.organizer = Some(Person {
        email: "grace@example.com".into(),
        name: Some("Grace".into()),
    });
    e.location = Some("Room 4".into());
    e.conference = Some(Conference {
        provider: "meet".into(),
        url: "https://meet.example.com/abc".into(),
    });
    e
}

#[test]
fn an_agenda_row_names_the_people_place_and_link_of_its_meeting() {
    let ctx = context();
    ctx.store.put(&Object::Event(design_review())).unwrap();
    let args = AgendaArgs {
        query: None,
        from: None,
        to: None,
        max_age: vec![],
    };
    let document = run_agenda(&ctx, args).unwrap().data;
    let object: serde_json::Value = serde_json::from_str(OBJECT).unwrap();
    let id = object["$id"].as_str().unwrap().to_string();
    let validator = jsonschema::options()
        .with_resource(id, jsonschema::Resource::from_contents(object).unwrap())
        .build(&serde_json::from_str(AGENDA).unwrap())
        .unwrap();
    let errors: Vec<String> = validator
        .iter_errors(&document)
        .map(|e| e.to_string())
        .collect();
    assert!(errors.is_empty(), "{document}: {errors:?}");
    let meeting = &document["events"][0];
    assert_eq!(
        meeting["attendees"],
        json!([
            {"email": "ada@example.com", "response": "accepted", "self": true},
            {"email": "grace@example.com", "response": "tentative"}
        ])
    );
    assert_eq!(
        meeting["organizer"],
        json!({"email": "grace@example.com", "name": "Grace"})
    );
    assert_eq!(meeting["location"], "Room 4");
    assert_eq!(
        meeting["conference"],
        json!({"provider": "meet", "url": "https://meet.example.com/abc"})
    );
    assert_eq!(meeting["start"], "2026-09-18T14:00:00+00:00");
    assert_eq!(meeting["busy"], true);
}
