use super::*;
use crate::testing::prelude::*;
use crate::testing::{Scenario, event, oid, todoist};
use dam_domain::{Kind, Object, Op, Task};

fn named(byte: u8, subject: &str) -> Object {
    Object::Task(Task::new(oid(byte), subject))
}

fn committing(s: &Scenario, object: Object) -> CommitId {
    s.working(object.clone());
    let dam = s.dam();
    dam.add(&[object.oid().clone()]).unwrap();
    dam.commit("m").unwrap().id
}

#[test]
fn a_fully_staged_change_is_not_also_unstaged() {
    let s = Scenario::default();
    s.working(named(1, "a"));
    let dam = s.dam();
    dam.add(&[oid(1)]).unwrap();
    let status = dam.status(&[]).unwrap();
    assert_eq!(status.staged.len(), 1);
    assert!(status.unstaged.is_empty());
}

#[test]
fn an_edit_after_staging_shows_as_unstaged_on_top_of_the_stage() {
    let s = Scenario::default();
    s.working(named(1, "a"));
    let dam = s.dam();
    dam.add(&[oid(1)]).unwrap();
    s.working(named(1, "b"));
    let status = dam.status(&[]).unwrap();
    assert_eq!(status.staged[0].after().unwrap().base().subject, "a");
    assert_eq!(status.unstaged[0].op(), Op::Update);
    assert_eq!(status.unstaged[0].after().unwrap().base().subject, "b");
}

#[test]
fn unpushed_names_each_commit_newest_first() {
    let s = Scenario::default().drawing_from(5);
    let first = committing(&s, named(1, "a"));
    let second = committing(&s, named(2, "b"));
    let status = s.dam().status(&[todoist()]).unwrap();
    assert_eq!(status.unpushed[0].remote, todoist());
    assert_eq!(status.unpushed[0].commit_ids, vec![second, first]);
}

#[test]
fn unpushed_names_each_object_once_in_first_appearance_order() {
    let s = Scenario::default();
    s.committed(named(1, "a"))
        .committed(named(2, "a"))
        .committed(named(2, "changed again"));
    let status = s.dam().status(&[todoist()]).unwrap();
    assert_eq!(status.unpushed[0].commit_ids.len(), 3);
    assert_eq!(status.unpushed[0].oids, vec![oid(2), oid(1)]);
}

#[test]
fn a_remote_owed_nothing_gets_a_row_with_empty_lists() {
    let s = Scenario::default();
    let status = s.dam().status(&[todoist()]).unwrap();
    assert_eq!(status.unpushed.len(), 1);
    assert!(status.unpushed[0].commit_ids.is_empty());
    assert!(status.unpushed[0].oids.is_empty());
}

#[test]
fn unpushed_counts_per_remote() {
    let s = Scenario::default();
    s.committed(named(1, "a"));
    let status = s.dam().status(&[todoist()]).unwrap();
    assert_eq!(status.unpushed.len(), 1);
    assert_eq!(status.unpushed[0].remote, todoist());
    assert_eq!(status.unpushed[0].commit_ids.len(), 1);
}

#[test]
fn a_working_delete_not_yet_staged_shows_as_an_unstaged_delete() {
    let s = Scenario::default();
    s.committed(named(1, "a"));
    s.store.delete(&oid(1)).unwrap();
    let dam = s.dam();
    let unstaged = dam.diff_working().unwrap();
    assert_eq!(unstaged.len(), 1);
    assert_eq!(unstaged[0].op(), Op::Delete);
    let staged = dam.add_all().unwrap();
    assert_eq!(staged.len(), 1);
    assert_eq!(staged[0].op(), Op::Delete);
}

#[test]
fn a_commit_is_owed_only_to_the_remotes_that_accept_one_of_its_kinds() {
    let s = Scenario::default();
    let task_commit = committing(&s, named(1, "a"));
    let event_commit = committing(&s, event(2));
    let gcal = RemoteName("gcal".into());
    let never_connected = RemoteName("new".into());
    s.store
        .set_accepted_kinds(&todoist(), &[Kind::Task])
        .unwrap();
    s.store.set_accepted_kinds(&gcal, &[]).unwrap();
    let status = s.dam().status(&[todoist(), gcal, never_connected]).unwrap();
    assert_eq!(status.unpushed[0].commit_ids, vec![task_commit.clone()]);
    assert_eq!(status.unpushed[0].oids, vec![oid(1)]);
    assert!(status.unpushed[1].commit_ids.is_empty());
    assert!(status.unpushed[1].oids.is_empty());
    assert_eq!(
        status.unpushed[2].commit_ids,
        vec![event_commit, task_commit]
    );
}

#[test]
fn a_mixed_commit_owes_a_remote_only_the_objects_of_kinds_it_accepts() {
    let s = Scenario::default().drawing_from(3);
    s.working(named(1, "a")).working(event(2));
    let dam = s.dam();
    dam.add(&[oid(1), oid(2)]).unwrap();
    let both = dam.commit("m").unwrap();
    s.store
        .set_accepted_kinds(&todoist(), &[Kind::Task])
        .unwrap();
    let status = dam.status(&[todoist()]).unwrap();
    assert_eq!(status.unpushed[0].commit_ids, vec![both.id]);
    assert_eq!(status.unpushed[0].oids, vec![oid(1)]);
}
