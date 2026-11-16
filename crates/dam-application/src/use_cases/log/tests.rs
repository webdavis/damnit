use dam_domain::Permission::{HistoryRead, TaskRead};
use dam_domain::{CommitRecord, Object};

use crate::service::Dam;
use crate::testing::prelude::*;
use crate::testing::{Scenario, allowed, holder, task_in};

fn three_commits() -> Scenario {
    let s = Scenario::default();
    s.committed(Object::Task(task_in(1, "report", "work")));
    s.committed(Object::Task(task_in(2, "laundry", "home")));
    s.working(Object::Task(task_in(1, "report v2", "work")));
    s.working(Object::Task(task_in(2, "laundry v2", "home")));
    let dam = s.dam();
    dam.add_all().unwrap();
    dam.commit("both").unwrap();
    s
}

fn logged_for(s: &Scenario, paths: Option<&[&str]>) -> Vec<CommitRecord> {
    let token = holder(vec![
        allowed(&[HistoryRead], None),
        allowed(&[TaskRead], paths),
    ]);
    let dam = Dam {
        access: &token,
        ..s.dam()
    };
    dam.seen_log().unwrap()
}

fn sizes(log: &[CommitRecord]) -> Vec<usize> {
    log.iter().map(|r| r.changes.len()).collect()
}

#[test]
fn a_token_that_reads_nothing_sees_no_commit() {
    assert!(logged_for(&three_commits(), Some(&["nowhere"])).is_empty());
}

#[test]
fn a_token_that_reads_part_sees_only_its_changes_and_no_message_of_a_mixed_commit() {
    let s = three_commits();
    let all = s.store.log().unwrap();
    let log = logged_for(&s, Some(&["work"]));
    assert_eq!(sizes(&log), [1, 1]);
    let both = all.iter().find(|r| r.message == "both").unwrap();
    let mixed = log.iter().find(|r| r.id == both.id).unwrap();
    assert_eq!(mixed.message, "");
}

#[test]
fn a_token_that_reads_all_and_the_owner_see_the_whole_log() {
    let s = three_commits();
    let all = s.store.log().unwrap();
    assert_eq!(sizes(&logged_for(&s, None)), sizes(&all));
    assert_eq!(s.dam().seen_log().unwrap(), all);
}
