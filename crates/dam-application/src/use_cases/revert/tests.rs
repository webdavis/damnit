use dam_domain::{Authored, Change, CommitId, CommitRecord, Field, Object, Priority, Task};

use super::*;
use crate::errors::{Refusal, UseCaseError};
use crate::ports::{CommitRepository, ObjectRepository, StageRepository};
use crate::service::Dam;
use crate::testing::{Scenario, holder, oid, task};

fn titled(byte: u8, subject: &str) -> Object {
    Object::Task(Task::new(oid(byte), subject))
}

fn retitled_and_ranked(byte: u8, subject: &str) -> Object {
    let mut t = Task::new(oid(byte), subject);
    t.priority = Priority::new(1).unwrap();
    Object::Task(t)
}

fn newest(s: &Scenario) -> String {
    s.store.log().unwrap()[0].id.to_string()
}

fn working(s: &Scenario, byte: u8) -> Option<Object> {
    s.store.get(&oid(byte)).unwrap()
}

fn fields(byte: u8, fields: &[Field]) -> Fields {
    Fields {
        oid: oid(byte),
        fields: fields.to_vec(),
    }
}

#[test]
fn a_field_still_as_committed_goes_back() {
    let s = Scenario::default();
    s.committed(task(1)).committed(retitled_and_ranked(1, "b"));
    let out = s.dam().revert(&newest(&s), &[]).unwrap();
    assert_eq!(
        out.reverted,
        vec![fields(1, &[Field::Subject, Field::Priority])]
    );
    assert!(out.kept.is_empty());
    assert_eq!(working(&s, 1), Some(task(1)));
}

#[test]
fn a_field_changed_since_is_kept_and_named() {
    let s = Scenario::default();
    s.committed(task(1)).committed(retitled_and_ranked(1, "b"));
    let commit = newest(&s);
    let mut since = Task::new(oid(1), "c");
    since.priority = Priority::new(1).unwrap();
    s.working(Object::Task(since));
    let out = s.dam().revert(&commit, &[]).unwrap();
    assert_eq!(out.reverted, vec![fields(1, &[Field::Priority])]);
    assert_eq!(out.kept, vec![fields(1, &[Field::Subject])]);
    assert_eq!(working(&s, 1), Some(titled(1, "c")));
}

#[test]
fn naming_one_task_of_two_reverts_only_that_one() {
    let s = Scenario::default();
    s.committed(task(1)).committed(task(2));
    s.working(titled(1, "b")).committed(titled(2, "b"));
    let named = oid(2).as_str()[..7].to_string();
    let out = s.dam().revert(&newest(&s), &[named]).unwrap();
    assert_eq!(out.reverted, vec![fields(2, &[Field::Subject])]);
    assert_eq!(working(&s, 1), Some(titled(1, "b")));
    assert_eq!(working(&s, 2), Some(task(2)));
}

#[test]
fn a_created_task_untouched_since_is_removed_and_one_edited_since_is_kept() {
    let s = Scenario::default();
    s.working(task(1)).committed(task(2));
    let commit = newest(&s);
    s.working(titled(2, "edited"));
    let out = s.dam().revert(&commit, &[]).unwrap();
    assert_eq!(out.reverted, vec![fields(1, &[Field::Subject])]);
    assert_eq!(out.kept, vec![fields(2, &[Field::Subject])]);
    assert_eq!(working(&s, 1), None);
    assert_eq!(working(&s, 2), Some(titled(2, "edited")));
}

#[test]
fn a_created_task_another_now_depends_on_is_kept() {
    let s = Scenario::default();
    s.committed(task(1));
    let commit = newest(&s);
    let mut waiting = Task::new(oid(2), "t2");
    waiting.base.depends = vec![oid(1)];
    s.working(Object::Task(waiting));
    let out = s.dam().revert(&commit, &[]).unwrap();
    assert!(out.reverted.is_empty());
    assert_eq!(out.kept, vec![fields(1, &[Field::Subject])]);
    assert_eq!(working(&s, 1), Some(task(1)));
}

#[test]
fn a_removed_task_comes_back_as_it_was() {
    let s = Scenario::default();
    s.committed(retitled_and_ranked(1, "b")).deleted(&oid(1));
    let out = s.dam().revert(&newest(&s), &[]).unwrap();
    assert_eq!(
        out.reverted,
        vec![fields(1, &[Field::Subject, Field::Priority])]
    );
    assert_eq!(working(&s, 1), Some(retitled_and_ranked(1, "b")));
}

#[test]
fn the_revert_lands_as_working_changes_and_commits_nothing() {
    let s = Scenario::default();
    s.committed(task(1)).committed(titled(1, "b"));
    s.dam().revert(&newest(&s), &[]).unwrap();
    assert!(s.store.staged().unwrap().is_empty());
    assert_eq!(s.store.log().unwrap().len(), 2);
    assert_eq!(s.store.committed(&oid(1)).unwrap(), Some(titled(1, "b")));
}

#[test]
fn a_revert_once_committed_is_reverted_the_same_way() {
    let s = Scenario::default();
    s.committed(task(1)).committed(titled(1, "b"));
    s.dam().revert(&newest(&s), &[]).unwrap();
    s.dam().add_all().unwrap();
    s.dam().commit("revert").unwrap();
    let out = s.dam().revert(&newest(&s), &[]).unwrap();
    assert_eq!(out.reverted, vec![fields(1, &[Field::Subject])]);
    assert_eq!(working(&s, 1), Some(titled(1, "b")));
}

#[test]
fn a_pulls_commit_is_refused() {
    let s = Scenario::default();
    let pulled = CommitRecord {
        id: CommitId::generate(&mut |b: &mut [u8]| b.fill(9)),
        message: "pull from todoist".into(),
        pulled: true,
        at: dam_domain::Timestamp::UNIX_EPOCH,
        changes: vec![Authored::by_owner(Change::Create { after: task(1) })],
    };
    s.store.commit(&pulled).unwrap();
    assert_eq!(
        s.dam().revert(pulled.id.as_str(), &[]),
        Err(Refusal::PulledCommit(pulled.id.clone()).into())
    );
    assert_eq!(working(&s, 1), None);
}

#[test]
fn a_task_not_in_the_commit_is_refused() {
    let s = Scenario::default();
    s.committed(task(1)).committed(task(2));
    let commit = s.store.log().unwrap()[0].id.clone();
    let named = oid(1).to_string();
    assert_eq!(
        s.dam()
            .revert(commit.as_str(), std::slice::from_ref(&named)),
        Err(Refusal::NotInCommit {
            commit,
            text: named
        }
        .into())
    );
}

#[test]
fn a_token_is_refused() {
    let s = Scenario::default();
    s.committed(task(1));
    let tool = holder(vec![]);
    let as_tool = Dam {
        access: &tool,
        ..s.dam()
    };
    assert_eq!(
        as_tool.revert(&newest(&s), &[]),
        Err(UseCaseError::Refused(Refusal::NotOpenToTokens("revert")))
    );
}
