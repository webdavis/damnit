use super::*;
use crate::ports::{HelperError, Notice, StoreError};
use crate::remote::{IncomingObject, PullOutcome};
use crate::testing::prelude::*;
use crate::testing::{
    FaultyStore, MemoryStore, Scenario, ScriptedLauncher, answering, config, oid,
    remotes_launched_by, todoist,
};
use dam_domain::{Object, Task};

const MINUTE: Duration = Duration::from_secs(60);

fn stale_after(window: Option<Duration>) -> Scenario {
    let mut config = config();
    config.remotes[0].stale = window;
    Scenario::default().configured(config)
}

fn one_object() -> Result<PullOutcome, HelperError> {
    Ok(PullOutcome {
        objects: vec![IncomingObject {
            remote_id: "r1".into(),
            object: Object::Task(Task::new(oid(0), "from upstream")),
        }],
        ..PullOutcome::default()
    })
}

fn refresh(s: &Scenario, launcher: &ScriptedLauncher) -> Result<(), UseCaseError> {
    s.dam()
        .refresh_stale(remotes_launched_by(launcher), |e| format!("{e:?}"))
}

fn launches(launcher: &ScriptedLauncher) -> usize {
    launcher.launched_with.borrow().len()
}

#[test]
fn a_stale_remote_is_pulled_and_a_fresh_one_is_not() {
    let s = stale_after(Some(MINUTE));
    let launcher = answering(one_object());
    refresh(&s, &launcher).unwrap();
    assert_eq!(s.store.all().unwrap().len(), 1);
    refresh(&s, &launcher).unwrap();
    assert_eq!(launches(&launcher), 1, "a fresh remote is not pulled again");
}

#[test]
fn a_remote_without_a_stale_window_is_never_pulled() {
    let s = stale_after(None);
    let launcher = answering(one_object());
    refresh(&s, &launcher).unwrap();
    assert_eq!(launches(&launcher), 0);
}

#[test]
fn an_unreachable_stale_remote_leaves_a_notice_and_no_last_pull() {
    let s = stale_after(Some(MINUTE));
    let launcher = answering(Err(HelperError::Io("down".into())));
    refresh(&s, &launcher).unwrap();
    assert_eq!(s.store.last_pull(&todoist()).unwrap(), None);
    assert!(matches!(
        s.store.notices().unwrap().last(),
        Some(Notice::PullFailed { remote: r, .. }) if *r == todoist()
    ));
}

#[test]
fn a_cancelled_stale_refresh_stops_the_read_and_leaves_no_notice() {
    let s = stale_after(Some(MINUTE));
    let launcher = answering(Err(HelperError::Cancelled));
    assert_eq!(
        refresh(&s, &launcher),
        Err(UseCaseError::Helper(HelperError::Cancelled))
    );
    assert_eq!(s.store.notices().unwrap(), vec![]);
    assert_eq!(s.store.last_pull(&todoist()).unwrap(), None);
}

fn pulled_ago(ago: Duration) -> Scenario {
    let s = stale_after(Some(MINUTE));
    s.pulled_ago(jiff::SignedDuration::try_from(ago).unwrap());
    s
}

#[test]
fn a_remote_pulled_exactly_one_window_ago_is_fresh_and_one_second_later_stale() {
    let launcher = answering(one_object());
    refresh(&pulled_ago(MINUTE), &launcher).unwrap();
    assert_eq!(
        launches(&launcher),
        0,
        "a pull exactly one window old is fresh"
    );
    refresh(&pulled_ago(MINUTE + Duration::from_secs(1)), &launcher).unwrap();
    assert_eq!(
        launches(&launcher),
        1,
        "a pull older than the window is stale"
    );
}

#[test]
fn a_store_error_reading_the_last_pull_fails_the_read_and_pulls_nothing() {
    let s = stale_after(Some(MINUTE));
    let store = FaultyStore::new(MemoryStore::new());
    store.fail_reading_the_last_pull();
    let launcher = answering(one_object());
    let refused = s
        .fixture
        .dam(&store)
        .refresh_stale(remotes_launched_by(&launcher), |e| format!("{e:?}"));
    assert_eq!(
        refused,
        Err(UseCaseError::Store(StoreError::Failed(
            "the last pull could not be read".into()
        )))
    );
    assert_eq!(launches(&launcher), 0);
}
