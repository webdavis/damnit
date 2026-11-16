use dam_domain::{Object, Reminder, Task, When};
use jiff::tz::TimeZone;

use crate::args::RemindersArgs;
use crate::testing::context;

const SCHEMA: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../docs/schemas/reminders.schema.json"
));

fn reminded_task() -> Object {
    let mut task = Task::new(dam_application::testing::oid(1), "call the bank");
    let due: jiff::Timestamp = "2026-01-02T10:00:00Z".parse().unwrap();
    task.due = Some(When::At(due.to_zoned(TimeZone::UTC)));
    task.base.reminders = [30, 1440]
        .map(|minutes_before| Reminder { minutes_before })
        .to_vec();
    Object::Task(task)
}

#[test]
fn reminders_list_the_ones_that_fall_due_in_the_window_in_the_published_shape() {
    let ctx = context();
    ctx.store.put(&reminded_task()).unwrap();
    let args = RemindersArgs {
        scope: Default::default(),
        place: Default::default(),
        from: Some("2026-01-02T09:00".into()),
        to: Some("2026-01-02T10:00".into()),
    };
    let due = super::run_reminders(&ctx, args).unwrap().data;
    let schema: serde_json::Value = serde_json::from_str(SCHEMA).unwrap();
    assert!(jsonschema::is_valid(&schema, &due), "{due}");
    let reminders = due["reminders"].as_array().unwrap();
    assert_eq!(reminders.len(), 1, "{due}");
    let at = reminders[0]["at"]
        .as_str()
        .unwrap()
        .parse::<jiff::Timestamp>();
    assert_eq!(at.unwrap(), "2026-01-02T09:30:00Z".parse().unwrap());
    assert_eq!(reminders[0]["minutes_before"], 30);
    assert_eq!(reminders[0]["subject"], "call the bank");
}
