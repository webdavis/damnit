use std::time::Duration;

use dam_protocol::{Mutation, MutationResult, Verdict};

use super::fixtures::{mutation, sync_body, task, with_fields};
use crate::api::ApiError;
use crate::api::fake::FakeTodoist;
use crate::push::push;

const SERVICE: &str = "Todoist";

fn timed_out() -> ApiError {
    ApiError::TimedOut {
        service: SERVICE,
        after: Duration::from_secs(30),
    }
}

fn unanswered_why(result: &MutationResult) -> &str {
    match &result.verdict {
        Verdict::Unanswered { why } => why,
        other => panic!("{} was answered {other:?}", result.oid),
    }
}

fn create(n: u8, path: &str, subject: &str) -> Mutation {
    let object = task(&format!("{n:x}").repeat(40), path, subject, false);
    mutation("create", n, None, Some(object))
}

fn rename_milk(n: u8) -> Mutation {
    let object = task(&format!("{n:x}").repeat(40), "Work/", "oat", false);
    with_fields(
        mutation("update", n, Some("i:i1"), Some(object)),
        &["subject"],
    )
}

#[test]
fn after_an_unanswered_create_only_what_depends_on_it_is_left_out() {
    let api = FakeTodoist::new(sync_body()).failing_write(1, timed_out());
    let mutations = vec![
        create(1, "Work/", "P"),
        rename_milk(2),
        create(3, "Work/P/", "child"),
        create(4, "Work/P/child/", "grandchild"),
        mutation("delete", 1, Some("i:i9"), None),
    ];
    let response = push(&api, mutations, &mut |_| {}).unwrap();
    assert_eq!(response.results.len(), 5, "{:?}", response.results);
    assert_eq!(
        unanswered_why(&response.results[0]),
        "Todoist did not answer within 30s"
    );
    assert!(response.results[1].ok(), "{:?}", response.results[1]);
    for left_out in &response.results[2..] {
        assert_eq!(
            unanswered_why(left_out),
            "waits on Work/P, which Todoist left unanswered"
        );
    }
    assert_eq!(
        api.recorded.borrow().writes,
        2,
        "only the rename went out after"
    );
}

#[test]
fn a_delete_of_an_object_whose_update_is_unanswered_is_left_out() {
    let api = FakeTodoist::new(sync_body()).failing_write(1, timed_out());
    let mutations = vec![rename_milk(1), mutation("delete", 2, Some("i:i1"), None)];
    let response = push(&api, mutations, &mut |_| {}).unwrap();
    assert_eq!(
        unanswered_why(&response.results[1]),
        "waits on Work/oat, which Todoist left unanswered"
    );
    assert_eq!(api.recorded.borrow().writes, 1);
}

#[test]
fn a_call_that_cannot_reach_todoist_ends_and_the_rest_says_so() {
    let unreachable = ApiError::Unreachable {
        service: SERVICE,
        why: "io: Connection refused".into(),
    };
    let api = FakeTodoist::new(sync_body()).failing_write(1, unreachable);
    let mutations = vec![
        create(1, "Work/", "P"),
        rename_milk(2),
        create(3, "Work/", "Q"),
    ];
    let response = push(&api, mutations, &mut |_| {}).unwrap();
    assert_eq!(response.results.len(), 3);
    for result in &response.results {
        assert_eq!(
            unanswered_why(result),
            "Todoist could not be reached: io: Connection refused"
        );
    }
    assert_eq!(api.recorded.borrow().writes, 1);
}
