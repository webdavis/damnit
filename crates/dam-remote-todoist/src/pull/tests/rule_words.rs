use serde_json::json;

use super::super::pull;
use crate::api::fake::FakeTodoist;

fn pulled_rule(string: &str) -> Option<String> {
    let todoist = FakeTodoist::new(json!({
        "sync_token": "t1", "full_sync": true,
        "projects": [{"id": "p1", "name": "Work"}],
        "sections": [],
        "items": [{"id": "i1", "content": "all hands", "project_id": "p1", "checked": false,
            "due": {"date": "2026-12-01", "is_recurring": true, "string": string}}]
    }));
    let response = pull(&todoist, None).unwrap();
    response
        .objects
        .iter()
        .find(|o| o.remote_id.as_deref() == Some("i:i1"))
        .and_then(|o| o.recurrence.clone())
}

#[test]
fn a_weekday_of_the_month_in_todoists_words_comes_in_as_the_dam_rule() {
    assert_eq!(
        pulled_rule("every! 1st mon, last fri until 2027-06-30").as_deref(),
        Some("every! month on 1mon,-1fri until 2027-06-30")
    );
    assert_eq!(
        pulled_rule("every last workday").as_deref(),
        Some("every last workday")
    );
}

#[test]
fn weekdays_month_days_and_every_other_in_todoists_words_come_in_as_the_dam_rule() {
    for (todoist, dam) in [
        ("every mon, fri", "every week on mon,fri"),
        ("every 1, 15", "every month on 1,15"),
        (
            "every! other fri until 2027-06-30",
            "every! 2 weeks on fri until 2027-06-30",
        ),
        ("every other month", "every 2 months"),
    ] {
        assert_eq!(pulled_rule(todoist).as_deref(), Some(dam), "{todoist}");
    }
}
