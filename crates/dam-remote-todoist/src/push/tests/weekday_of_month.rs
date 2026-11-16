use super::fixtures::{mutation, sync_body, task, with_fields};
use crate::api::fake::FakeTodoist;
use crate::push::push;

fn repeating(n: u8, rule: &str) -> dam_protocol::WireObject {
    let mut object = task(&format!("{n:x}").repeat(40), "Work/", "all hands", false);
    object.recurrence = Some(rule.into());
    object
}

#[test]
fn a_weekday_of_the_month_is_sent_in_todoists_words() {
    for (op, remote_id, command) in [
        ("create", None, "item_add"),
        ("update", Some("i:i1"), "item_update"),
    ] {
        let api = FakeTodoist::new(sync_body());
        let m = with_fields(
            mutation(
                op,
                1,
                remote_id,
                Some(repeating(1, "every! month on 1mon,-1fri")),
            ),
            &["recurrence"],
        );
        let response = push(&api, vec![m], &mut |_| {}).unwrap();
        assert!(response.results[0].ok(), "{op}");
        assert_eq!(
            api.recorded.borrow().command(command)["args"]["due"],
            serde_json::json!({ "string": "every! 1st mon, last fri" })
        );
    }
}

#[test]
fn a_rule_todoist_cannot_hold_is_refused_for_that_task_and_sends_nothing() {
    let oid = "1".repeat(40);
    for (rule, why) in [
        (
            "every 2 months on 1mon,3mon",
            format!(
                "Todoist cannot combine a monthly interval with a weekday of the month, \
                 so it cannot hold \"every 2 months on 1mon,3mon\"; nothing was sent for this task; \
                 give it a rule Todoist can hold, such as: \
                 dam edit {oid} --recurrence \"every month on 1mon,3mon\""
            ),
        ),
        (
            "every month on -2fri",
            format!(
                "Todoist counts back from the end of a month only to its last weekday, \
                 so it cannot hold \"every month on -2fri\"; nothing was sent for this task; \
                 give it a rule Todoist can hold, such as: \
                 dam edit {oid} --recurrence \"every month on -1fri\""
            ),
        ),
    ] {
        let api = FakeTodoist::new(sync_body());
        let m = mutation("create", 1, None, Some(repeating(1, rule)));
        let response = push(&api, vec![m], &mut |_| {}).unwrap();
        assert!(!response.results[0].ok(), "{rule}");
        assert_eq!(response.results[0].why(), Some(why.as_str()));
        assert!(api.recorded.borrow().executed.is_empty(), "{rule}");
    }
}

#[test]
fn a_change_that_leaves_the_rule_and_due_alone_is_sent() {
    let api = FakeTodoist::new(sync_body());
    let m = with_fields(
        mutation(
            "update",
            1,
            Some("i:i1"),
            Some(repeating(1, "every! month on -1fri")),
        ),
        &["subject"],
    );
    let response = push(&api, vec![m], &mut |_| {}).unwrap();
    assert!(response.results[0].ok());
    assert_eq!(api.recorded.borrow().types(), vec!["item_update"]);
}

#[test]
fn rules_without_a_weekday_of_the_month_are_sent_as_before() {
    for rule in [
        "every 2 weeks on mon",
        "every month on 1,15",
        "every other week",
    ] {
        let api = FakeTodoist::new(sync_body());
        let m = mutation("create", 1, None, Some(repeating(1, rule)));
        let response = push(&api, vec![m], &mut |_| {}).unwrap();
        assert!(response.results[0].ok(), "{rule}");
        assert_eq!(
            api.recorded.borrow().command("item_add")["args"]["due"],
            serde_json::json!({ "string": rule })
        );
    }
}
