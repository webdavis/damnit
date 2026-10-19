use super::fixtures::{mutation, sync_body, task, with_fields};
use crate::api::ApiError;
use crate::api::fake::{FakeTodoist, answered, rate_limited};
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
    let err = push(&api, mutations).unwrap_err();
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

#[test]
fn a_failure_part_way_through_belongs_to_its_own_mutation() {
    let api = FakeTodoist::new(sync_body()).failing_write(2, answered(500, "upstream is down"));
    let mutations = vec![
        with_fields(
            mutation(
                "update",
                1,
                Some("i:i1"),
                Some(task(&"1".repeat(40), "Work/", "milk", false)),
            ),
            &["subject"],
        ),
        with_fields(
            mutation(
                "update",
                2,
                Some("i:i1"),
                Some(task(&"2".repeat(40), "Work/", "oat", false)),
            ),
            &["subject"],
        ),
        with_fields(
            mutation(
                "update",
                3,
                Some("i:i1"),
                Some(task(&"3".repeat(40), "Work/", "soy", false)),
            ),
            &["subject"],
        ),
    ];
    let response = push(&api, mutations).unwrap();
    assert!(response.results[0].ok());
    assert!(!response.results[1].ok());
    assert!(response.results[1].why().unwrap().contains("500"));
    assert!(
        response.results[2].ok(),
        "the batch goes on after one mutation's failure"
    );
}

#[test]
fn a_rate_limit_stops_the_push_and_names_the_retry_time() {
    let api = FakeTodoist::new(sync_body()).failing_write(1, rate_limited(42));
    let mutations = vec![mutation(
        "create",
        1,
        None,
        Some(task(&"1".repeat(40), "Work/", "eggs", false)),
    )];
    let err = push(&api, mutations).unwrap_err();
    assert!(matches!(err, ApiError::RateLimited { .. }), "{err:?}");
    assert_eq!(err.retry_after(), Some(std::time::Duration::from_secs(42)));
    assert!(err.to_string().contains("42s"), "{err}");
}
