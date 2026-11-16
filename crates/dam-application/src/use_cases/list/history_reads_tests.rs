use dam_domain::Object;
use dam_domain::Permission::TaskRead;

use crate::config::Config;
use crate::errors::UseCaseError;
use crate::ports::{History, StoreError};
use crate::service::Dam;
use crate::testing::{FakeFilterFunctions, FaultyStore, Scenario, allowed, holder, task_in};

fn three_tasks() -> Scenario {
    let mut s = Scenario::default().configured(Config {
        filters: vec![crate::testing::saved_function("slipping", &[])],
        ..Config::default()
    });
    s.fixture.filter_functions = FakeFilterFunctions::keeping(&[]);
    for (n, subject) in [(1, "report"), (2, "invoice"), (3, "laundry")] {
        s.committed(Object::Task(task_in(n, subject, "work")));
    }
    s
}

#[test]
fn a_token_without_history_read_never_has_the_log_read() {
    let s = three_tasks();
    let store = FaultyStore::new(s.store.clone());
    let reader = holder(vec![allowed(&[TaskRead], None)]);
    Dam {
        access: &reader,
        ..s.fixture.dam(&store)
    }
    .list_saved("slipping")
    .unwrap();
    assert_eq!(store.log_reads(), 0);
}

#[test]
fn a_listing_reads_the_log_once_however_many_objects_ask_for_their_history() {
    let s = three_tasks();
    let store = FaultyStore::new(s.store.clone());
    s.fixture.dam(&store).list_saved("slipping").unwrap();
    assert_eq!(store.log_reads(), 1);
    let seen = s.fixture.filter_functions.seen.borrow();
    assert_eq!(seen.len(), 3);
    assert!(
        seen.iter()
            .all(|(_, h)| matches!(h, Some(History::Recorded(c)) if c.len() == 1))
    );
}

#[test]
fn a_match_that_never_asks_for_history_never_has_the_log_read() {
    let mut s = three_tasks();
    s.fixture.filter_functions.never_reading_history = true;
    let store = FaultyStore::new(s.store.clone());
    s.fixture.dam(&store).list_saved("slipping").unwrap();
    assert_eq!(store.log_reads(), 0);
}

#[test]
fn a_log_that_will_not_read_fails_the_listing_as_the_store_failing() {
    let s = three_tasks();
    let store = FaultyStore::new(s.store.clone());
    store.fail_reading_the_log();
    assert_eq!(
        s.fixture.dam(&store).list_saved("slipping"),
        Err(UseCaseError::Store(StoreError::Failed(
            "the log could not be read".into()
        )))
    );
    assert_eq!(
        s.fixture.filter_functions.seen.borrow()[0].1,
        Some(History::Unread)
    );
}
