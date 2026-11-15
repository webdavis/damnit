use dam_domain::{Event, Field, When};
use jiff::civil::date;

use super::*;
use crate::config::{HookConfig, HookEvent};
use crate::testing::pulling_events;

#[test]
fn it_runs_for_no_task_the_pull_left_unchanged_in_conflict_untouched_or_removed() {
    let s = pulled_hooked();
    tracked(&s, 1, "same");
    tracked(&s, 2, "base");
    tracked(&s, 3, "base");
    tracked(&s, 4, "gone");
    s.working(Object::Task(Task::new(oid(2), "mine")));
    s.working(Object::Task(Task::new(oid(3), "mine too")));
    let report = pull(
        &s,
        PullOutcome {
            objects: vec![
                incoming("same", "r1"),
                incoming("theirs", "r2"),
                incoming("base", "r3"),
            ],
            removed: vec!["r4".into()],
            sync: Some("s1".into()),
            ..PullOutcome::default()
        },
    );
    assert_eq!(report.conflicts, 1);
    assert_eq!(calls(&s), []);
}

#[test]
fn it_runs_for_no_event() {
    let s = pulled_hooked();
    let meeting = Event::new(
        oid(0),
        "meeting",
        When::Day(date(2026, 9, 25)),
        When::Day(date(2026, 9, 26)),
    );
    let l = pulling_events(
        vec![Field::Subject, Field::Start, Field::End],
        PullOutcome {
            objects: vec![IncomingObject {
                remote_id: "e1".into(),
                object: Object::Event(meeting),
            }],
            sync: Some("s1".into()),
            ..PullOutcome::default()
        },
    );
    let report = s.dam().pull(remotes_launched_by(&l), None).unwrap();
    assert_eq!(report[0].created, 1);
    assert_eq!(calls(&s), []);
}

#[test]
fn it_runs_on_no_first_pull_from_a_remote() {
    let s = pulled_hooked();
    s.store.set_sync_token(&todoist(), None).unwrap();
    let report = pull_objects(&s, vec![incoming("brand new", "r1")]);
    assert_eq!(report.created, 1);
    assert_eq!(calls(&s), []);
    pull_objects(&s, vec![incoming("next", "r2")]);
    assert_eq!(calls(&s), [("next".to_string(), None, "new")]);
}

#[test]
fn the_local_hooks_stay_quiet_on_a_pull() {
    let mut s = pulled_hooked();
    s.fixture
        .config
        .hooks
        .extend(HookEvent::LOCAL.map(|event| HookConfig {
            event,
            on_failure: None,
        }));
    tracked(&s, 1, "old");
    pull_objects(&s, vec![incoming("new", "r1"), done("b", "r2")]);
    assert_eq!(s.fixture.hooks.ran(), []);
    assert_eq!(calls(&s).len(), 2);
}
