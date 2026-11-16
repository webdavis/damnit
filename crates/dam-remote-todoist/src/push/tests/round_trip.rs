use serde_json::json;

use super::fixtures::{mutation, sync_body, task};
use crate::api::fake::FakeTodoist;
use crate::pull::pull;
use crate::push::push;

const RULES: [&str; 15] = [
    "every day",
    "every! day",
    "every 3 days",
    "every week",
    "every 2 weeks",
    "every week on mon,fri",
    "every 2 weeks on fri",
    "every 2 weeks on mon,fri",
    "every month",
    "every 2 months",
    "every month on 1,15",
    "every 2 months on 1,15",
    "every! month on 1tue,-1fri",
    "every year",
    "every 2 years",
];

fn sent_string(rule: &str, due: &str) -> String {
    let mut object = task(&"1".repeat(40), "Work/", "all hands", false);
    object.recurrence = Some(rule.into());
    object.task.as_mut().unwrap().due = Some(due.into());
    let api = FakeTodoist::new(sync_body());
    assert!(
        push(
            &api,
            vec![mutation("create", 1, None, Some(object))],
            &mut |_| {}
        )
        .unwrap()
        .results[0]
            .ok()
    );
    let string = api.recorded.borrow().command("item_add")["args"]["due"]["string"].clone();
    string.as_str().unwrap().to_string()
}

fn pulled(string: &str, date: &str) -> (Option<String>, Option<String>) {
    let todoist = FakeTodoist::new(json!({
        "sync_token": "t1", "full_sync": true,
        "projects": [{"id": "p1", "name": "Work"}],
        "sections": [],
        "items": [{"id": "i1", "content": "all hands", "project_id": "p1", "checked": false,
            "due": {"date": date, "is_recurring": true, "string": string}}]
    }));
    let object = pull(&todoist, None)
        .unwrap()
        .objects
        .into_iter()
        .find(|o| o.remote_id.as_deref() == Some("i:i1"))
        .unwrap();
    (object.recurrence, object.task.unwrap().due)
}

#[test]
fn every_rule_shape_with_a_date_or_a_date_and_time_comes_back_as_the_same_rule_and_due() {
    let ends = ["", " until 2027-06-30"];
    let dues = [
        ("2026-12-04", "2026-12-04"),
        (
            "2026-12-04T20:00:00-05:00[America/New_York]",
            "2026-12-04T20:00:00",
        ),
    ];
    for rule in RULES
        .iter()
        .flat_map(|r| ends.map(|end| format!("{r}{end}")))
    {
        for (dam_due, todoist_date) in dues {
            let string = sent_string(&rule, dam_due);
            assert!(string.ends_with(" starting 4 Dec 2026"), "{string}");
            assert_eq!(
                string.contains(" at 20:00"),
                dam_due.contains('T'),
                "{string}"
            );
            assert_eq!(
                pulled(&string, todoist_date),
                (Some(rule.clone()), Some(todoist_date.to_string())),
                "{string}"
            );
        }
    }
}
