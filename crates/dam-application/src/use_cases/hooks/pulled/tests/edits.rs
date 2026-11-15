use std::path::PathBuf;

use super::*;
use crate::ports::{
    CommitRepository, EditRequest, FunctionFailure, FunctionFault, StageRepository,
};
use crate::use_cases::edit::EditFields;

fn requesting(s: &Scenario, subject: &str) {
    *s.fixture.hooks.requesting.borrow_mut() = Some(EditRequest {
        json: String::new(),
        file: PathBuf::from("config.lua"),
        line: Some(3),
    });
    *s.fixture.edit_requests.reading.borrow_mut() = Some(Ok(EditFields {
        subject: Some(subject.into()),
        ..EditFields::default()
    }));
}

fn failing(s: &Scenario) {
    *s.fixture.hooks.failing.borrow_mut() = Some(FunctionFailure {
        file: PathBuf::from("config.lua"),
        line: Some(2),
        fault: FunctionFault::Raised {
            message: "no bell".into(),
        },
    });
}

#[test]
fn an_edit_is_left_as_a_working_change_on_top_of_the_pull() {
    let s = pulled_hooked();
    tracked(&s, 1, "old");
    requesting(&s, "filed");
    let report = pull_objects(&s, vec![incoming("new", "r1")]);
    assert_eq!(subject(&s, &oid(1)), "filed");
    assert_eq!(committed_subject(&s, &oid(1)), "new");
    assert!(s.store.staged().unwrap().is_empty());
    let run = report.task_pulled.unwrap();
    assert_eq!((run.ran, run.changed, run.failed), (1, 1, 0));
}

#[test]
fn a_failure_drops_only_that_tasks_edit_and_the_pull_is_kept() {
    let s = pulled_hooked();
    failing(&s);
    requesting(&s, "filed");
    let report = pull_objects(&s, vec![incoming("a", "r1"), incoming("b", "r2")]);
    assert_eq!(report.created, 2);
    let first = s
        .store
        .oid_for_remote_id(&todoist(), "r1")
        .unwrap()
        .unwrap();
    let second = s
        .store
        .oid_for_remote_id(&todoist(), "r2")
        .unwrap()
        .unwrap();
    assert_eq!(subject(&s, &first), "a");
    assert_eq!(subject(&s, &second), "filed");
    assert_eq!(committed_subject(&s, &first), "a");
    assert_eq!(s.store.log().unwrap().len(), 1);
    let run = report.task_pulled.unwrap();
    assert_eq!((run.ran, run.changed, run.failed), (2, 1, 1));
}

#[test]
fn an_edit_that_changes_nothing_counts_as_no_change() {
    let s = pulled_hooked();
    requesting(&s, "a");
    let report = pull_objects(&s, vec![incoming("a", "r1")]);
    let run = report.task_pulled.unwrap();
    assert_eq!((run.ran, run.changed, run.failed), (1, 0, 0));
}
