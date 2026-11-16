use super::fixtures::{mutation, sync_body, task, with_fields};
use crate::api::fake::FakeTodoist;
use crate::push::push;

fn sent_due(op: &str, due: Option<&str>) -> serde_json::Value {
    let mut object = task(&"1".repeat(40), "Work/", "all hands", false);
    object.recurrence = Some("every week on mon,fri until 2027-06-30".into());
    object.task.as_mut().unwrap().due = due.map(str::to_string);
    let remote_id = (op == "update").then_some("i:i1");
    let api = FakeTodoist::new(sync_body());
    let m = with_fields(mutation(op, 1, remote_id, Some(object)), &["due"]);
    assert!(push(&api, vec![m], &mut |_| {}).unwrap().results[0].ok());
    let command = if op == "create" {
        "item_add"
    } else {
        "item_update"
    };
    api.recorded.borrow().command(command)["args"]["due"].clone()
}

#[test]
fn a_repeating_task_starts_in_todoist_on_its_own_due_date_and_time() {
    for op in ["create", "update"] {
        assert_eq!(
            sent_due(op, Some("2026-12-04T20:00:00-05:00[America/New_York]")),
            serde_json::json!({
                "string": "every mon, fri at 20:00 until 2027-06-30 starting 4 Dec 2026",
                "lang": "en",
                "timezone": "America/New_York"
            }),
            "{op}"
        );
    }
}

#[test]
fn a_repeating_task_with_no_due_date_sends_the_rule_alone() {
    assert_eq!(
        sent_due("create", None),
        serde_json::json!({ "string": "every mon, fri until 2027-06-30", "lang": "en" })
    );
}
