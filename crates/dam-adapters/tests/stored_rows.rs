use std::collections::BTreeSet;

use dam_adapters::SqliteStore;
use dam_application::ObjectRepository;
use dam_domain::{
    Attachment, Attendee, Completion, Conference, Event, EventStatus, EventType, Object, Oid, Path,
    Person, Priority, Recurrence, Reminder, ResponseStatus, Task, Transparency, Visibility, When,
};

fn oid(byte: u8) -> Oid {
    Oid::generate(&mut |b: &mut [u8]| b.fill(byte))
}

fn at(text: &str) -> When {
    When::At(text.parse().unwrap())
}

fn day(text: &str) -> When {
    When::Day(text.parse().unwrap())
}

fn full_task() -> Task {
    let mut t = Task::new(oid(0x11), "buy oat milk");
    t.base.body = "the barista kind".into();
    t.base.path = Path::parse("home/errands").unwrap();
    t.base.labels = BTreeSet::from(["errand".to_string(), "light".to_string()]);
    t.base.depends = vec![oid(0x22), oid(0x33)];
    t.base.reminders = vec![
        Reminder { minutes_before: 30 },
        Reminder {
            minutes_before: 1440,
        },
    ];
    t.base.recurrence = Some(Recurrence::dam("every 2 weeks on mon").unwrap());
    t.completion = Completion::Done {
        at: Some("2026-09-24T18:30:00Z".parse().unwrap()),
    };
    t.priority = Priority::new(2).unwrap();
    t.due = Some(at("2026-09-25T09:00:00+02:00[Europe/Berlin]"));
    t.deadline = Some("2026-09-30".parse().unwrap());
    t.event = Some(oid(0x44));
    t
}

fn full_event() -> Event {
    let mut e = Event::new(
        oid(0x44),
        "quarterly offsite",
        at("2026-09-25T09:00:00+02:00[Europe/Berlin]"),
        at("2026-09-25T17:30:00+02:00[Europe/Berlin]"),
    );
    e.base.body = "bring the roadmap".into();
    e.base.path = Path::parse("work").unwrap();
    e.base.labels = BTreeSet::from(["team".to_string()]);
    e.base.depends = vec![oid(0x55)];
    e.base.reminders = vec![Reminder { minutes_before: 10 }];
    e.base.recurrence = Some(Recurrence::parse("RRULE:FREQ=YEARLY"));
    e.timezone = Some("Europe/Berlin".into());
    e.location = Some("Room 4".into());
    e.attendees = vec![
        attendee("ana@example.com", ResponseStatus::Accepted, true),
        attendee("bo@example.com", ResponseStatus::Declined, false),
        attendee("cy@example.com", ResponseStatus::Tentative, false),
        attendee("di@example.com", ResponseStatus::NeedsAction, false),
    ];
    e.status = EventStatus::Tentative;
    e.transparency = Transparency::Free;
    e.visibility = Visibility::Confidential;
    e.event_type = EventType::FocusTime;
    e.color = Some("7".into());
    e.organizer = Some(Person {
        email: "ana@example.com".into(),
        name: Some("Ana".into()),
    });
    e.conference = Some(Conference {
        provider: "meet".into(),
        url: "https://meet.example.com/abc".into(),
    });
    e.attachments = vec![Attachment {
        url: "https://files.example.com/roadmap".into(),
        title: "roadmap".into(),
        mime_type: Some("application/pdf".into()),
    }];
    e
}

fn attendee(email: &str, response: ResponseStatus, is_self: bool) -> Attendee {
    Attendee {
        email: email.into(),
        response,
        is_self,
    }
}

fn stored_row(object: &Object) -> String {
    let dir = tempfile::tempdir().unwrap();
    let db = dir.path().join("dam.db");
    SqliteStore::open(&db).unwrap().put(object).unwrap();
    rusqlite::Connection::open(&db)
        .unwrap()
        .query_row(
            "SELECT json FROM objects WHERE oid = ?1",
            [object.oid().as_str()],
            |r| r.get(0),
        )
        .unwrap()
}

const FULL_TASK_ROW: &str = concat!(
    r#"{"oid":"1111111111111111111111111111111111111111","kind":"task","#,
    r#""subject":"buy oat milk","body":"the barista kind","path":"home/errands/","#,
    r#""labels":["errand","light"],"#,
    r#""depends":["2222222222222222222222222222222222222222","#,
    r#""3333333333333333333333333333333333333333"],"reminders":[30,1440],"#,
    r#""recurrence":"every 2 weeks on mon","task":{"done":true,"#,
    r#""completed_at":"2026-09-24T18:30:00Z","priority":2,"#,
    r#""due":"2026-09-25T09:00:00+02:00[Europe/Berlin]","deadline":"2026-09-30","#,
    r#""event":"4444444444444444444444444444444444444444"}}"#,
);

const FULL_EVENT_ROW: &str = concat!(
    r#"{"oid":"4444444444444444444444444444444444444444","kind":"event","#,
    r#""subject":"quarterly offsite","body":"bring the roadmap","path":"work/","#,
    r#""labels":["team"],"depends":["5555555555555555555555555555555555555555"],"#,
    r#""reminders":[10],"recurrence":"RRULE:FREQ=YEARLY","event":{"#,
    r#""start":"2026-09-25T09:00:00+02:00[Europe/Berlin]","#,
    r#""end":"2026-09-25T17:30:00+02:00[Europe/Berlin]","timezone":"Europe/Berlin","#,
    r#""location":"Room 4","attendees":["#,
    r#"{"email":"ana@example.com","response":"accepted","self":true},"#,
    r#"{"email":"bo@example.com","response":"declined"},"#,
    r#"{"email":"cy@example.com","response":"tentative"},"#,
    r#"{"email":"di@example.com","response":"needs_action"}],"#,
    r#""status":"tentative","transparency":"free","visibility":"confidential","#,
    r#""event_type":"focus_time","color":"7","#,
    r#""organizer":{"email":"ana@example.com","name":"Ana"},"#,
    r#""conference":{"provider":"meet","url":"https://meet.example.com/abc"},"#,
    r#""attachments":[{"url":"https://files.example.com/roadmap","title":"roadmap","#,
    r#""mime_type":"application/pdf"}]}}"#,
);

#[test]
fn a_fully_populated_task_is_stored_as_this_exact_golden_row() {
    assert_eq!(stored_row(&Object::Task(full_task())), FULL_TASK_ROW);
}

#[test]
fn a_fully_populated_event_is_stored_as_this_exact_golden_row() {
    assert_eq!(stored_row(&Object::Event(full_event())), FULL_EVENT_ROW);
}

#[test]
fn every_event_enum_variant_is_stored_as_its_golden_word() {
    use EventStatus as S;
    for (v, w) in [(S::Confirmed, "confirmed"), (S::Cancelled, "cancelled")] {
        assert_event_row_holds(|e| e.status = v, "status", w);
    }
    assert_event_row_holds(
        |e| e.transparency = Transparency::Busy,
        "transparency",
        "busy",
    );
    use Visibility as V;
    for (v, w) in [
        (V::Default, "default"),
        (V::Public, "public"),
        (V::Private, "private"),
    ] {
        assert_event_row_holds(|e| e.visibility = v, "visibility", w);
    }
    use EventType as T;
    for (v, w) in [
        (T::Default, "default"),
        (T::OutOfOffice, "out_of_office"),
        (T::WorkingLocation, "working_location"),
        (T::Birthday, "birthday"),
    ] {
        assert_event_row_holds(|e| e.event_type = v, "event_type", w);
    }
}

fn assert_event_row_holds(set: impl Fn(&mut Event), field: &str, word: &str) {
    let mut e = Event::new(oid(0x66), "day off", day("2026-09-25"), day("2026-09-26"));
    set(&mut e);
    let row = stored_row(&Object::Event(e));
    let pair = format!(r#""{field}":"{word}""#);
    assert!(row.contains(&pair), "{pair} missing from {row}");
}
