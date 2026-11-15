use std::path::PathBuf;

use dam_domain::Permission::RemotePull;

use super::slow::SlowHooks;
use super::*;
use crate::ports::{FunctionFailure, FunctionFault, Notice, NoticeRepository};
use crate::service::Dam;
use crate::testing::{allowed, holder};

fn no_bell() -> FunctionFailure {
    FunctionFailure {
        file: PathBuf::from("config.lua"),
        line: Some(2),
        fault: FunctionFault::Raised {
            message: "no bell".into(),
        },
    }
}

fn noted(failed: usize, not_reached: usize, error: Option<FunctionFailure>) -> Vec<Notice> {
    vec![Notice::TaskPulledFailed {
        remote: todoist(),
        failed,
        not_reached,
        error,
    }]
}

fn failing_twice(s: &Scenario) {
    *s.fixture.hooks.failing_every_call.borrow_mut() = Some(no_bell());
    pull_objects(s, vec![incoming("a", "r1"), incoming("b", "r2")]);
}

#[test]
fn a_run_that_failed_is_noted_with_its_first_error_and_counts() {
    let s = pulled_hooked();
    failing_twice(&s);
    assert_eq!(s.store.notices().unwrap(), noted(2, 0, Some(no_bell())));
}

#[test]
fn a_later_run_that_failed_replaces_the_note() {
    let s = pulled_hooked();
    failing_twice(&s);
    *s.fixture.hooks.failing_every_call.borrow_mut() = None;
    *s.fixture.hooks.failing.borrow_mut() = Some(no_bell());
    pull_objects(&s, vec![incoming("c", "r3"), incoming("d", "r4")]);
    assert_eq!(s.store.notices().unwrap(), noted(1, 0, Some(no_bell())));
}

#[test]
fn a_later_run_with_no_failure_that_reached_every_task_clears_the_note() {
    let s = pulled_hooked();
    failing_twice(&s);
    *s.fixture.hooks.failing_every_call.borrow_mut() = None;
    pull_objects(&s, vec![incoming("c", "r3")]);
    assert_eq!(s.store.notices().unwrap(), []);
}

#[test]
fn a_run_the_total_stopped_is_noted_with_what_it_did_not_reach() {
    let s = pulled_hooked();
    let slow = SlowHooks::taking(&s.fixture.hooks, 60);
    let l = pulling(PullOutcome {
        objects: vec![incoming("a", "r1"), incoming("b", "r2")],
        sync: Some("s1".into()),
        ..PullOutcome::default()
    });
    let dam = Dam {
        clock: &slow,
        hooks: &slow,
        ..s.dam()
    };
    dam.pull(remotes_launched_by(&l), None).unwrap();
    assert_eq!(s.store.notices().unwrap(), noted(0, 1, None));
}

#[test]
fn a_run_a_tokens_command_caused_is_noted_for_the_owner() {
    let s = pulled_hooked();
    *s.fixture.hooks.failing_every_call.borrow_mut() = Some(no_bell());
    let tool = holder(vec![allowed(&[RemotePull], None)]);
    let l = pulling(PullOutcome {
        objects: vec![incoming("a", "r1")],
        sync: Some("s1".into()),
        ..PullOutcome::default()
    });
    let dam = Dam {
        access: &tool,
        ..s.dam()
    };
    dam.pull(remotes_launched_by(&l), None).unwrap();
    assert_eq!(s.store.notices().unwrap(), noted(1, 0, Some(no_bell())));
}
