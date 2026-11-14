use dam_protocol::{Mutation, Verdict};

use super::fixtures::{mutation, sync_body, task, with_fields};
use crate::api::ApiError;
use crate::api::fake::{FakeTodoist, answered, rate_limited};

const SERVICE: &str = "Todoist";
use crate::push::push;

#[test]
fn a_failure_reading_the_tree_stops_the_push_before_any_mutation() {
    let api = FakeTodoist::new(sync_body()).failing_read(answered(500, "boom"));
    let mutations = vec![mutation(
        "create",
        1,
        None,
        Some(task(&"1".repeat(40), "Work/", "eggs", false)),
    )];
    let err = push(&api, mutations, &mut |_| {}).unwrap_err();
    assert!(
        matches!(err, ApiError::Http { status: 500, .. }),
        "an upstream status reaches the caller as itself: {err:?}"
    );
    assert_eq!(
        api.recorded.borrow().writes,
        0,
        "nothing was attempted after the read failed"
    );
}

pub(super) fn three_subject_updates() -> Vec<Mutation> {
    [(1, "milk"), (2, "oat"), (3, "soy")]
        .into_iter()
        .map(|(n, subject)| {
            let oid = format!("{n}").repeat(40);
            let object = task(&oid, "Work/", subject, false);
            with_fields(
                mutation("update", n, Some("i:i1"), Some(object)),
                &["subject"],
            )
        })
        .collect()
}

#[test]
fn a_request_todoist_turned_away_fails_only_its_own_mutation() {
    let api = FakeTodoist::new(sync_body()).failing_write(2, answered(400, "bad request"));
    let response = push(&api, three_subject_updates(), &mut |_| {}).unwrap();
    assert!(response.results[0].ok());
    assert!(!response.results[1].ok());
    assert!(response.results[1].why().unwrap().contains("400"));
    assert!(
        response.results[2].ok(),
        "the batch goes on after one mutation's refusal"
    );
}

#[test]
fn a_write_with_no_answer_is_unanswered_and_the_rest_is_still_sent() {
    let no_answers = [
        ApiError::transport(SERVICE, "timeout: global"),
        ApiError::transport(SERVICE, "connection closed before the answer"),
        answered(502, "bad gateway"),
        ApiError::decode(SERVICE, "expected value at line 1 column 1"),
        ApiError::TooLarge {
            service: SERVICE,
            limit: 1,
        },
    ];
    for no_answer in no_answers {
        let shown = no_answer.to_string();
        let api = FakeTodoist::new(sync_body()).failing_write(2, no_answer);
        let response = push(&api, three_subject_updates(), &mut |_| {}).unwrap();
        let answered: Vec<_> = response.results.iter().map(|r| r.oid.clone()).collect();
        assert_eq!(
            answered,
            vec!["1".repeat(40), "2".repeat(40), "3".repeat(40)],
            "{shown}"
        );
        assert!(response.results[0].ok(), "{shown}");
        assert!(
            matches!(response.results[1].verdict, Verdict::Unanswered { .. }),
            "{shown}"
        );
        assert!(
            response.results[2].ok(),
            "{shown}: {:?}",
            response.results[2]
        );
        assert_eq!(api.recorded.borrow().writes, 3, "{shown}");
    }
}

#[test]
fn a_rate_limit_hands_back_the_answers_before_it_with_the_stop() {
    let api = FakeTodoist::new(sync_body()).failing_write(2, rate_limited(42));
    let response = push(&api, three_subject_updates(), &mut |_| {}).unwrap();
    let stop = "Todoist is rate limiting this account; retry after 42s";
    assert_eq!(response.stopped.as_deref(), Some(stop));
    assert!(response.results[0].ok(), "{:?}", response.results[0]);
    for after in &response.results[1..] {
        assert_eq!(
            after.verdict,
            Verdict::Unanswered { why: stop.into() },
            "{}",
            after.oid
        );
    }
    assert_eq!(
        api.recorded.borrow().writes,
        2,
        "nothing is sent after the stop"
    );
}
