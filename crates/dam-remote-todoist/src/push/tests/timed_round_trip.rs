use serde_json::{Value, json};

use super::fixtures::{mutation, sync_body, task};
use crate::api::fake::FakeTodoist;
use crate::pull::pull;
use crate::push::push;

fn sent_due(rule: Option<&str>, due: &str) -> Value {
    let mut object = task(&"1".repeat(40), "Work/", "call", false);
    object.recurrence = rule.map(str::to_string);
    object.task.as_mut().unwrap().due = Some(due.into());
    let api = FakeTodoist::new(sync_body());
    let m = mutation("create", 1, None, Some(object));
    assert!(push(&api, vec![m], &mut |_| {}).unwrap().results[0].ok());
    api.recorded.borrow().command("item_add")["args"]["due"].clone()
}

fn pulled(due: Value) -> (Option<String>, Option<String>) {
    let todoist = FakeTodoist::new(json!({
        "sync_token": "t1", "full_sync": true,
        "projects": [{"id": "p1", "name": "Work"}],
        "sections": [],
        "items": [{"id": "i1", "content": "call", "project_id": "p1", "checked": false, "due": due}]
    }));
    let object = pull(&todoist, None)
        .unwrap()
        .objects
        .into_iter()
        .find(|o| o.remote_id.as_deref() == Some("i:i1"))
        .unwrap();
    (object.recurrence, object.task.unwrap().due)
}

fn as_todoist_stores(sent: &Value, date: &str) -> Value {
    json!({
        "date": date.replace('Z', ".000000Z"),
        "timezone": sent["timezone"],
        "is_recurring": sent.get("string").is_some(),
        "string": sent.get("string").cloned().unwrap_or(json!("")),
    })
}

#[test]
fn a_timed_due_on_either_side_of_a_clock_change_comes_back_unchanged() {
    for due in [
        "2027-03-13T20:00:00-05:00[America/New_York]",
        "2027-03-15T20:00:00-04:00[America/New_York]",
    ] {
        let sent = sent_due(None, due);
        let date = sent["date"].as_str().unwrap().to_string();
        assert!(date.ends_with('Z'), "{date}");
        assert_eq!(sent["timezone"], "America/New_York");
        assert_eq!(
            pulled(as_todoist_stores(&sent, &date)),
            (None, Some(due.into()))
        );
    }
}

#[test]
fn a_repeating_time_keeps_its_clock_time_in_its_zone_after_a_clock_change() {
    let sent = sent_due(
        Some("every day"),
        "2027-03-13T20:00:00-05:00[America/New_York]",
    );
    assert_eq!(sent["string"], "every day at 20:00 starting 13 Mar 2027");
    assert_eq!(sent["timezone"], "America/New_York");
    assert_eq!(
        pulled(as_todoist_stores(&sent, "2027-03-16T00:00:00Z")),
        (
            Some("every day".into()),
            Some("2027-03-15T20:00:00-04:00[America/New_York]".into())
        )
    );
}
