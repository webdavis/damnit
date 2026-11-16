use dam_domain::Permission::{ConflictRead, RemotePush, TaskRead};
use dam_domain::{Object, Permission, Task};

use crate::ports::Conflict;
use crate::service::Dam;
use crate::testing::prelude::*;
use crate::testing::{
    Scenario, accepting_all, allowed, holder, oid, pushing, remotes_launched_by, task, todoist,
};

fn one_conflict_owed() -> Scenario {
    let s = Scenario::default();
    s.committed(task(1));
    s.store
        .mark_conflict(&Conflict {
            remote: todoist(),
            oid: oid(1),
            ours: Some(task(1)),
            theirs: Object::Task(Task::new(oid(1), "theirs")),
        })
        .unwrap();
    s
}

fn held_as_told_to(s: &Scenario, permissions: &[Permission]) -> Option<usize> {
    let token = holder(vec![allowed(permissions, None)]);
    let dam = Dam {
        access: &token,
        ..s.dam()
    };
    let (l, _) = pushing(accepting_all(None));
    dam.push(remotes_launched_by(&l), None).unwrap()[0].held
}

#[test]
fn a_token_without_conflict_read_is_not_told_how_many_were_held() {
    assert_eq!(held_as_told_to(&one_conflict_owed(), &[RemotePush]), None);
}

#[test]
fn a_token_with_conflict_read_is_told_how_many_were_held() {
    assert_eq!(
        held_as_told_to(&one_conflict_owed(), &[RemotePush, ConflictRead, TaskRead]),
        Some(1)
    );
}

#[test]
fn a_token_without_conflict_read_is_told_when_nothing_was_held() {
    let s = Scenario::default();
    s.committed(task(1));
    assert_eq!(held_as_told_to(&s, &[RemotePush]), Some(0));
}
