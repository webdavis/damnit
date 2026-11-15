use std::collections::BTreeSet;
use std::time::Duration;

use dam_application::{ConfigFile, FunctionFailure, Histories, History};
use dam_domain::{
    Attachment, Attendee, Completion, Conference, Event, Focus, Object, Oid, Path, Person,
    Priority, Recurrence, Reminder, ResponseStatus, Task, Timestamp, When,
};
use jiff::civil::date;
use serde_json::Value;

use super::support::config;

pub(super) fn oid(byte: u8) -> Oid {
    Oid::generate(&mut |b: &mut [u8]| b.fill(byte))
}

pub(super) fn every_field_task() -> Object {
    let mut t = Task::new(oid(1), "report");
    t.base.body = "the body".into();
    t.base.path = Path::parse("work").unwrap();
    t.base.labels = BTreeSet::from(["errand".to_string()]);
    t.base.depends = vec![oid(2)];
    t.base.reminders = vec![Reminder { minutes_before: 10 }];
    t.base.recurrence = Some(Recurrence::parse("every 2 days"));
    t.priority = Priority::HIGHEST;
    t.completion = Completion::Done {
        at: Some(Timestamp::UNIX_EPOCH),
    };
    t.due = Some(When::Day(date(2026, 1, 1)));
    t.deadline = Some(date(2026, 2, 1));
    t.event = Some(oid(3));
    t.estimate = Some(30);
    t.focus = Some(Focus {
        day: date(2026, 1, 1),
        set_by: "owner".into(),
        set_at: Timestamp::UNIX_EPOCH,
    });
    Object::Task(t)
}

pub(super) fn every_field_event() -> Object {
    let mut e = Event::new(
        oid(4),
        "standup",
        When::Day(date(2026, 1, 1)),
        When::Day(date(2026, 1, 2)),
    );
    e.base.body = "the body".into();
    e.base.reminders = vec![Reminder { minutes_before: 5 }];
    e.base.recurrence = Some(Recurrence::parse("every 2 days"));
    e.timezone = Some("UTC".into());
    e.location = Some("Room 1".into());
    e.attendees = vec![Attendee {
        email: "ada@example.com".into(),
        response: ResponseStatus::Accepted,
        is_self: true,
    }];
    e.color = Some("blue".into());
    e.organizer = Some(Person {
        email: "ada@example.com".into(),
        name: Some("Ada".into()),
    });
    e.conference = Some(Conference {
        provider: "video".into(),
        url: "https://example.com/call".into(),
    });
    e.attachments = vec![Attachment {
        url: "https://example.com/doc".into(),
        title: "agenda".into(),
        mime_type: Some("text/plain".into()),
    }];
    Object::Event(e)
}

pub(super) fn field_names(value: &Value) -> BTreeSet<String> {
    let mut names = BTreeSet::new();
    walk(value, "", &mut names);
    names
}

fn walk(value: &Value, prefix: &str, names: &mut BTreeSet<String>) {
    let join = |key: &str| match prefix {
        "" => key.to_string(),
        _ => format!("{prefix}.{key}"),
    };
    match value {
        Value::Object(map) => map.iter().for_each(|(key, v)| match v {
            Value::Object(inner) if !inner.is_empty() => walk(v, &join(key), names),
            Value::Array(items) if items.iter().any(Value::is_object) => items
                .iter()
                .for_each(|item| walk(item, &format!("{}[]", join(key)), names)),
            _ => {
                names.insert(join(key));
            }
        }),
        _ => {
            names.insert(prefix.to_string());
        }
    }
}

pub(super) struct FixedHistory(pub(super) History);

impl Histories for FixedHistory {
    fn of(&self, _: &Oid) -> History {
        self.0.clone()
    }
}

pub(super) fn run_filter(
    match_body: &str,
    objects: &[Object],
) -> Result<Vec<bool>, FunctionFailure> {
    run_filter_with(
        match_body,
        objects,
        &FixedHistory(History::Recorded(vec![])),
    )
}

pub(super) fn run_filter_with(
    match_body: &str,
    objects: &[Object],
    histories: &dyn Histories,
) -> Result<Vec<bool>, FunctionFailure> {
    keeps_within(match_body, objects, histories, Duration::from_secs(5))
}

pub(super) fn run_filter_limited(
    match_body: &str,
    objects: &[Object],
    limit: Duration,
) -> Result<Vec<bool>, FunctionFailure> {
    keeps_within(
        match_body,
        objects,
        &FixedHistory(History::Recorded(vec![])),
        limit,
    )
}

fn keeps_within(
    match_body: &str,
    objects: &[Object],
    histories: &dyn Histories,
    limit: Duration,
) -> Result<Vec<bool>, FunctionFailure> {
    let text = format!(
        "return {{\n  filter = {{\n    probe = {{\n      match = function(task, dam)\n{match_body}\n      end,\n    }},\n  }},\n}}\n"
    );
    let folder = config(&text);
    let loaded = folder.file().limiting_filters_to(limit).load().unwrap();
    loaded.filter_functions.keeps("probe", objects, histories)
}
