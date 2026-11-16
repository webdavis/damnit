use dam_domain::Object;
use dam_domain::Permission::{RemotePull, TaskRead};

use crate::access::Holder;
use crate::remote::{IncomingObject, PullOutcome};
use crate::service::Dam;
use crate::testing::{Scenario, allowed, holder, oid, pulling, remotes_launched_by, task_in};
use crate::use_cases::pull::PullReport;

fn work_and_home() -> Scenario {
    let s = Scenario::default().drawing_from(42);
    s.committed(Object::Task(task_in(1, "report", "work")))
        .tracked_as(&oid(1), "r1");
    s.committed(Object::Task(task_in(2, "laundry", "home")))
        .tracked_as(&oid(2), "r2")
        .pushed();
    s
}

fn upstream(remote_id: &str, subject: &str, path: &str) -> IncomingObject {
    IncomingObject {
        remote_id: remote_id.into(),
        object: Object::Task(task_in(0, subject, path)),
    }
}

fn pulled_by(s: &Scenario, token: Option<Holder>) -> PullReport {
    let l = pulling(PullOutcome {
        objects: vec![
            upstream("r1", "report v2", "work"),
            upstream("r2", "laundry v2", "home"),
            upstream("r3", "groceries", "home"),
        ],
        ..PullOutcome::default()
    });
    match &token {
        Some(token) => Dam {
            access: token,
            ..s.dam()
        }
        .pull(remotes_launched_by(&l), None),
        None => s.dam().pull(remotes_launched_by(&l), None),
    }
    .unwrap()
    .remove(0)
}

fn token(paths: Option<&[&str]>) -> Option<Holder> {
    Some(holder(vec![
        allowed(&[RemotePull], None),
        allowed(&[TaskRead], paths),
    ]))
}

#[test]
fn a_token_that_reads_nothing_counts_nothing_and_is_told_more_came_in() {
    let report = pulled_by(&work_and_home(), token(Some(&["nowhere"])));
    assert_eq!(
        (report.created, report.updated, report.unchanged),
        (0, 0, 0)
    );
    assert!(report.unseen_pulled);
}

#[test]
fn a_token_that_reads_part_counts_only_its_objects() {
    let report = pulled_by(&work_and_home(), token(Some(&["work"])));
    assert_eq!((report.created, report.updated), (0, 1));
    assert!(report.unseen_pulled);
}

#[test]
fn a_token_that_reads_all_counts_everything_and_is_told_nothing_more() {
    let report = pulled_by(&work_and_home(), token(None));
    assert_eq!((report.created, report.updated), (1, 2));
    assert!(!report.unseen_pulled);
}

#[test]
fn the_owner_counts_everything() {
    let report = pulled_by(&work_and_home(), None);
    assert_eq!((report.created, report.updated), (1, 2));
    assert!(!report.unseen_pulled);
    assert!(report.task_pulled.is_none());
}
