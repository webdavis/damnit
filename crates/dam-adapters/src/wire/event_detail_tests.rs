use super::*;
use dam_domain::{Conference, Event, Object, Oid, Person, Reminder, When};
use jiff::civil::date;

#[test]
fn an_events_reminders_organizer_and_conference_come_back_from_the_wire() {
    let mut event = Event::new(
        Oid::generate(&mut |x: &mut [u8]| x.fill(4)),
        "planning",
        When::Day(date(2026, 9, 25)),
        When::Day(date(2026, 9, 26)),
    );
    event.base.reminders = vec![Reminder { minutes_before: 10 }];
    event.organizer = Some(Person {
        email: "ada@example.com".into(),
        name: Some("Ada".into()),
    });
    event.conference = Some(Conference {
        provider: "meet".into(),
        url: "https://meet.example/abc".into(),
    });
    let object = Object::Event(event);
    assert_eq!(from_wire(&to_wire(&object, None)).unwrap(), object);
}
