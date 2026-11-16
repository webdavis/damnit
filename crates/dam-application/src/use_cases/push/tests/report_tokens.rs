use dam_domain::Permission::{ConflictRead, EventRead, RemotePush, TaskRead};
use dam_domain::{Object, Permission, Task};

use crate::access::Holder;
use crate::ports::Conflict;
use crate::service::Dam;
use crate::testing::prelude::*;
use crate::testing::{
    Scenario, allowed, holder, oid, pushing, refusing_all, remotes_launched_by, task_in, todoist,
};
use crate::use_cases::push::PushReport;

const REFUSAL: &str = "no project named home";
const REFUSED: &str = "refused by the remote; ask the owner to run dam status for the reason";

fn work_and_home() -> Scenario {
    let s = Scenario::default();
    s.committed(Object::Task(task_in(1, "report", "work")));
    s.committed(Object::Task(task_in(2, "laundry", "home")));
    s
}

fn token(permissions: &[Permission], paths: Option<&[&str]>) -> Holder {
    holder(vec![
        allowed(&[RemotePush], None),
        allowed(permissions, paths),
    ])
}

fn refused_for(s: &Scenario, token: &Holder) -> PushReport {
    let dam = Dam {
        access: token,
        ..s.dam()
    };
    let (l, _) = pushing(refusing_all(REFUSAL));
    dam.push(remotes_launched_by(&l), None).unwrap().remove(0)
}

#[test]
fn a_token_that_reads_nothing_learns_only_that_something_did_not_go_through() {
    let report = refused_for(&work_and_home(), &token(&[], None));
    assert_eq!((report.sent, report.succeeded, report.skipped), (0, 0, 0));
    assert!(report.failed.is_empty());
    assert!(report.unseen_failed);
}

#[test]
fn a_token_that_reads_part_sees_its_failures_in_full_and_no_count_of_the_others() {
    let report = refused_for(&work_and_home(), &token(&[TaskRead], Some(&["work"])));
    assert_eq!((report.sent, report.succeeded, report.skipped), (1, 0, 0));
    assert_eq!(report.failed, [(oid(1), REFUSED.to_string())]);
    assert!(report.unseen_failed);
}

#[test]
fn a_task_whose_committed_copy_is_hidden_is_left_out_like_a_hidden_one() {
    let s = work_and_home();
    s.working(Object::Task(task_in(2, "laundry", "work")));
    let report = refused_for(&s, &token(&[TaskRead], Some(&["work"])));
    assert_eq!(report.sent, 1);
    assert_eq!(report.failed, [(oid(1), REFUSED.to_string())]);
    assert!(report.unseen_failed);
}

#[test]
fn a_token_that_reads_all_sees_every_failure_and_nothing_unseen() {
    let report = refused_for(&work_and_home(), &token(&[TaskRead, EventRead], None));
    assert_eq!(report.sent, 2);
    assert!(report.failed.iter().all(|(_, why)| why == REFUSAL));
    assert_eq!(report.failed.len(), 2);
    assert!(!report.unseen_failed);
}

#[test]
fn a_remote_reason_about_a_task_under_a_hidden_parent_is_not_shown() {
    let s = work_and_home();
    let mut child = task_in(3, "slides", "work");
    child.base.parent = Some(oid(2));
    s.committed(Object::Task(child));
    let report = refused_for(&s, &token(&[TaskRead], Some(&["work"])));
    let shown: Vec<_> = report.failed.iter().filter(|(o, _)| *o == oid(3)).collect();
    assert!(
        shown.iter().all(|(_, why)| !why.contains(REFUSAL)),
        "{shown:?}"
    );
}

#[test]
fn a_token_is_told_of_a_held_task_it_cannot_see_only_by_the_sentence() {
    let s = work_and_home();
    s.store
        .mark_conflict(&Conflict {
            remote: todoist(),
            oid: oid(2),
            ours: Some(Object::Task(task_in(2, "laundry", "home"))),
            theirs: Object::Task(Task::new(oid(2), "theirs")),
        })
        .unwrap();
    let conflicts_and_work = holder(vec![
        allowed(&[RemotePush, ConflictRead], None),
        allowed(&[TaskRead], Some(&["work"])),
    ]);
    let part = refused_for(&s, &conflicts_and_work);
    assert_eq!(part.held, Some(0));
    assert!(part.unseen_failed);
    let blind = refused_for(&s, &token(&[TaskRead], None));
    assert_eq!(blind.held, None);
    assert!(blind.unseen_failed);
}

#[test]
fn the_owner_sees_every_failure_and_no_sentence() {
    let s = work_and_home();
    let (l, _) = pushing(refusing_all(REFUSAL));
    let report = s
        .dam()
        .push(remotes_launched_by(&l), None)
        .unwrap()
        .remove(0);
    assert_eq!((report.sent, report.failed.len()), (2, 2));
    assert!(!report.unseen_failed);
}
